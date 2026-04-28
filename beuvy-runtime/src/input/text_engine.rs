use bevy::prelude::*;
use bevy::text::ComputedTextBlock;
use cosmic_text::{Affinity, Buffer, Cursor, Motion};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InputCaretRect {
    pub x: f32,
    pub top: f32,
    pub height: f32,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct InputTextEngine;

impl InputTextEngine {
    pub(crate) fn layout(
        &self,
        block: &ComputedTextBlock,
        inverse_scale_factor: f32,
    ) -> InputTextLayout {
        let buffer = block.buffer().0.clone();
        let text = buffer_text(&buffer);
        InputTextLayout {
            buffer,
            text,
            inverse_scale_factor,
        }
    }

    pub(crate) fn hit_byte(
        &self,
        block: &ComputedTextBlock,
        inverse_scale_factor: f32,
        x: f32,
        y: f32,
    ) -> usize {
        let layout = self.layout(block, inverse_scale_factor);
        let physical_x = x.max(0.0) / inverse_scale_factor;
        let physical_y = y.max(0.0) / inverse_scale_factor;
        layout
            .buffer
            .hit(physical_x, physical_y)
            .map(|cursor| byte_for_cursor(&layout.text, cursor))
            .unwrap_or_else(|| if x <= 0.0 { 0 } else { layout.text.len() })
    }

    pub(crate) fn move_byte_vertically(
        &self,
        block: &ComputedTextBlock,
        byte: usize,
        preferred_x: Option<f32>,
        direction: i32,
    ) -> Option<(usize, f32)> {
        let mut buffer = block.buffer().0.clone();
        let text = buffer_text(&buffer);
        let cursor = cursor_for_byte(&text, byte);
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
        let byte = byte_for_cursor(&text, cursor);
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
    inverse_scale_factor: f32,
}

impl InputTextLayout {
    pub(crate) fn caret_rect(&self, byte: usize) -> InputCaretRect {
        let cursor = cursor_for_byte(&self.text, byte);
        let fallback = InputCaretRect {
            x: 0.0,
            top: 0.0,
            height: 0.0,
        };

        for run in self.buffer.layout_runs() {
            if run.line_i != cursor.line {
                continue;
            }
            if run.glyphs.is_empty() || cursor.index == 0 {
                return InputCaretRect {
                    x: 0.0,
                    top: run.line_top * self.inverse_scale_factor,
                    height: run.line_height * self.inverse_scale_factor,
                };
            }
            let run_end = run.glyphs.last().map(|glyph| glyph.end).unwrap_or(0);
            let x = if cursor.index >= run_end {
                run.glyphs
                    .last()
                    .map(|glyph| glyph_trailing_edge(run.rtl, glyph))
                    .unwrap_or(0.0)
            } else {
                run.glyphs
                    .iter()
                    .find_map(|glyph| {
                        if cursor.index <= glyph.start {
                            Some(glyph_leading_edge(run.rtl, glyph))
                        } else if cursor.index <= glyph.end {
                            Some(glyph_trailing_edge(run.rtl, glyph))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        run.glyphs
                            .last()
                            .map(|glyph| glyph_trailing_edge(run.rtl, glyph))
                            .unwrap_or(0.0)
                    })
            };
            return InputCaretRect {
                x: x * self.inverse_scale_factor,
                top: run.line_top * self.inverse_scale_factor,
                height: run.line_height * self.inverse_scale_factor,
            };
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
                    rects.push((
                        left * self.inverse_scale_factor,
                        run.line_top * self.inverse_scale_factor,
                        width * self.inverse_scale_factor,
                        run.line_height * self.inverse_scale_factor,
                    ));
                }
            }
        }
        rects
    }

    pub(crate) fn size(&self) -> Vec2 {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for run in self.buffer.layout_runs() {
            width = width.max(run.line_w * self.inverse_scale_factor);
            height = height.max((run.line_top + run.line_height) * self.inverse_scale_factor);
        }
        Vec2::new(width, height)
    }
}

pub(crate) fn scroll_caret_rect(rect: InputCaretRect, scroll: Vec2) -> InputCaretRect {
    InputCaretRect {
        x: rect.x - scroll.x,
        top: rect.top - scroll.y,
        height: rect.height,
    }
}

pub(crate) fn scroll_selection_rects(
    rects: &[(f32, f32, f32, f32)],
    scroll: Vec2,
) -> Vec<(f32, f32, f32, f32)> {
    rects
        .iter()
        .map(|(left, top, width, height)| (left - scroll.x, top - scroll.y, *width, *height))
        .collect()
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

fn glyph_leading_edge(rtl: bool, glyph: &cosmic_text::LayoutGlyph) -> f32 {
    if rtl { glyph.x + glyph.w } else { glyph.x }
}

fn glyph_trailing_edge(rtl: bool, glyph: &cosmic_text::LayoutGlyph) -> f32 {
    if rtl { glyph.x } else { glyph.x + glyph.w }
}

fn buffer_text(buffer: &Buffer) -> String {
    let mut text = String::new();
    for line in &buffer.lines {
        text.push_str(line.text());
        text.push_str(line.ending().as_str());
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic_text::{Attrs, FontSystem, Metrics, Shaping, Wrap};

    fn layout(text: &str, multiline: bool) -> InputTextLayout {
        layout_with_scale(text, multiline, 1.0)
    }

    fn layout_with_scale(
        text: &str,
        multiline: bool,
        inverse_scale_factor: f32,
    ) -> InputTextLayout {
        let mut font_system = FontSystem::new();
        let scale_factor = inverse_scale_factor.recip();
        let mut buffer = Buffer::new(
            &mut font_system,
            Metrics::new(16.0 * scale_factor, 24.0 * scale_factor),
        );
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
            text: buffer_text(&buffer),
            buffer,
            inverse_scale_factor,
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
    fn caret_rect_is_reported_in_logical_pixels() {
        let normal = layout_with_scale("1234", false, 1.0);
        let scaled = layout_with_scale("1234", false, 0.5);

        let normal_end = normal.caret_rect("1234".len());
        let scaled_end = scaled.caret_rect("1234".len());

        assert!((normal_end.x - scaled_end.x).abs() < 0.01);
        assert!((normal_end.height - scaled_end.height).abs() < 0.01);
    }

    #[test]
    fn byte_cursor_round_trip_multiline() {
        let text = "a\nbc\n";

        for byte in [0, 1, 2, 3, 4, 5] {
            assert_eq!(byte_for_cursor(text, cursor_for_byte(text, byte)), byte);
        }
    }

    #[test]
    fn scroll_caret_rect_offsets_viewport_position() {
        let rect = InputCaretRect {
            x: 80.0,
            top: 24.0,
            height: 18.0,
        };

        assert_eq!(
            scroll_caret_rect(rect, Vec2::new(30.0, 10.0)),
            InputCaretRect {
                x: 50.0,
                top: 14.0,
                height: 18.0,
            }
        );
    }

    #[test]
    fn scroll_selection_rects_offsets_viewport_position() {
        let rects = vec![(80.0, 24.0, 40.0, 18.0)];

        assert_eq!(
            scroll_selection_rects(&rects, Vec2::new(30.0, 10.0)),
            vec![(50.0, 14.0, 40.0, 18.0)]
        );
    }
}
