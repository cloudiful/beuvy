use bevy::prelude::*;
use bevy::text::ComputedTextBlock;
use cosmic_text::{Affinity, Buffer, Cursor, Motion};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InputCaretRect {
    pub x: f32,
    pub top: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InputTextLayoutParams {
    pub width: f32,
    pub height: f32,
    pub multiline: bool,
}

#[derive(Debug, Default)]
pub(crate) struct InputTextEngine;

impl InputTextEngine {
    pub(crate) fn layout(&self, text: &str, block: &ComputedTextBlock) -> InputTextLayout {
        InputTextLayout {
            buffer: block.buffer().0.clone(),
            text: text.to_string(),
        }
    }

    pub(crate) fn hit_byte(&self, text: &str, block: &ComputedTextBlock, x: f32, y: f32) -> usize {
        let layout = self.layout(text, block);
        layout
            .buffer
            .hit(x.max(0.0), y.max(0.0))
            .map(|cursor| byte_for_cursor(&layout.text, cursor))
            .unwrap_or_else(|| if x <= 0.0 { 0 } else { layout.text.len() })
    }

    pub(crate) fn move_byte_vertically(
        &self,
        text: &str,
        block: &ComputedTextBlock,
        byte: usize,
        preferred_x: Option<f32>,
        direction: i32,
    ) -> Option<(usize, f32)> {
        let mut buffer = block.buffer().0.clone();
        let cursor = cursor_for_byte(text, byte);
        let motion = if direction < 0 {
            Motion::Up
        } else if direction > 0 {
            Motion::Down
        } else {
            return Some((byte, preferred_x.unwrap_or(0.0)));
        };
        let preferred = preferred_x.map(|x| x.round() as i32);
        let mut font_system = cosmic_text::FontSystem::new();
        let (cursor, next_x) = buffer.cursor_motion(&mut font_system, cursor, preferred, motion)?;
        let byte = byte_for_cursor(text, cursor);
        Some((
            byte,
            next_x
                .map(|x| x as f32)
                .unwrap_or(preferred_x.unwrap_or(0.0)),
        ))
    }
}

pub(crate) struct InputTextLayout {
    buffer: Buffer,
    text: String,
}

impl InputTextLayout {
    pub(crate) fn caret_rect(&self, byte: usize) -> InputCaretRect {
        let cursor = cursor_for_byte(&self.text, byte);
        let mut fallback = InputCaretRect {
            x: 0.0,
            top: 0.0,
            height: 0.0,
        };

        for run in self.buffer.layout_runs() {
            if run.line_i != cursor.line {
                continue;
            }
            fallback = InputCaretRect {
                x: run
                    .glyphs
                    .last()
                    .map(|glyph| if run.rtl { glyph.x } else { glyph.x + glyph.w })
                    .unwrap_or(0.0),
                top: run.line_top,
                height: run.line_height,
            };
            if let Some((x, _)) = run.highlight(cursor, cursor) {
                return InputCaretRect {
                    x,
                    top: run.line_top,
                    height: run.line_height,
                };
            }
            if run.glyphs.is_empty() || cursor.index == 0 {
                return InputCaretRect {
                    x: 0.0,
                    top: run.line_top,
                    height: run.line_height,
                };
            }
        }

        fallback
    }

    pub(crate) fn selection_rects(&self, start: usize, end: usize) -> Vec<(f32, f32, f32, f32)> {
        if start >= end {
            return Vec::new();
        }
        let start = cursor_for_byte(&self.text, start);
        let end = cursor_for_byte(&self.text, end);
        let mut rects = Vec::new();
        for run in self.buffer.layout_runs() {
            if let Some((left, width)) = run.highlight(start, end) {
                if width > 0.0 {
                    rects.push((left, run.line_top, width, run.line_height));
                }
            }
        }
        rects
    }

    pub(crate) fn size(&self) -> Vec2 {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for run in self.buffer.layout_runs() {
            width = width.max(run.line_w);
            height = height.max(run.line_top + run.line_height);
        }
        Vec2::new(width, height)
    }
}

pub(crate) fn input_layout_params(
    input_computed: &ComputedNode,
    multiline: bool,
) -> InputTextLayoutParams {
    let input_scale = input_computed.inverse_scale_factor();
    let input_inset = input_computed.content_inset();
    InputTextLayoutParams {
        width: (input_computed.size().x * input_scale
            - input_inset.min_inset.x
            - input_inset.max_inset.x)
            .max(0.0),
        height: (input_computed.size().y * input_scale
            - input_inset.min_inset.y
            - input_inset.max_inset.y)
            .max(0.0),
        multiline,
    }
}

fn cursor_for_byte(text: &str, byte: usize) -> Cursor {
    let byte = clamp_char_boundary(text, byte);
    let mut line = 0usize;
    let mut line_start = 0usize;
    for (index, chr) in text.char_indices() {
        if index >= byte {
            break;
        }
        if chr == '\n' {
            line += 1;
            line_start = index + chr.len_utf8();
        }
    }
    Cursor::new_with_affinity(line, byte - line_start, Affinity::Before)
}

fn byte_for_cursor(text: &str, cursor: Cursor) -> usize {
    let mut line = 0usize;
    let mut line_start = 0usize;
    let mut line_end = text.len();
    for (index, chr) in text.char_indices() {
        if chr == '\n' {
            if line == cursor.line {
                line_end = index;
                break;
            }
            line += 1;
            line_start = index + chr.len_utf8();
        }
    }
    let target = line_start + cursor.index.min(line_end.saturating_sub(line_start));
    clamp_char_boundary(text, target)
}

fn clamp_char_boundary(text: &str, byte: usize) -> usize {
    let mut byte = byte.min(text.len());
    while byte > 0 && !text.is_char_boundary(byte) {
        byte -= 1;
    }
    byte
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic_text::{Attrs, FontSystem, Metrics, Shaping, Wrap};

    fn layout(text: &str, multiline: bool) -> InputTextLayout {
        let mut font_system = FontSystem::new();
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 24.0));
        buffer.set_wrap(
            &mut font_system,
            if multiline {
                Wrap::WordOrGlyph
            } else {
                Wrap::None
            },
        );
        buffer.set_size(
            &mut font_system,
            multiline.then_some(200.0),
            multiline.then_some(120.0),
        );
        buffer.set_text(
            &mut font_system,
            text,
            &Attrs::new(),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut font_system, false);
        InputTextLayout {
            buffer,
            text: text.to_string(),
        }
    }

    #[test]
    fn caret_end_uses_text_end_for_varied_text() {
        for text in ["abc", "abc ", "中文", "a😀", "👨‍👩‍👧‍👦", "cafe\u{0301}"]
        {
            let layout = layout(text, false);
            let before_end = layout.caret_rect(text.len().saturating_sub(1));
            let at_end = layout.caret_rect(text.len());

            assert!(at_end.x >= before_end.x, "{text:?}");
        }
    }

    #[test]
    fn hit_after_text_returns_end_byte() {
        let text = "abc";
        let layout = layout(text, false);

        let byte = layout
            .buffer
            .hit(10_000.0, 0.0)
            .map(|cursor| byte_for_cursor(&layout.text, cursor))
            .unwrap_or(layout.text.len());

        assert_eq!(byte, text.len());
    }

    #[test]
    fn byte_cursor_round_trip_multiline() {
        let text = "a\nbc\n";

        for byte in [0, 1, 2, 3, 4, 5] {
            assert_eq!(byte_for_cursor(text, cursor_for_byte(text, byte)), byte);
        }
    }
}
