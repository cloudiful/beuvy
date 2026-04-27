use bevy::math::Rect;
use bevy::prelude::*;
use bevy::text::{PositionedGlyph, TextLayoutInfo};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TextLineMetrics {
    pub line_index: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

pub(super) fn node_global_rect(computed: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    let (_, _, center) = transform.to_scale_angle_translation();
    Rect::from_center_size(center, computed.size())
}

pub(super) fn node_logical_rect(computed: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    let physical = node_global_rect(computed, transform);
    let inverse = computed.inverse_scale_factor();
    Rect {
        min: physical.min * inverse,
        max: physical.max * inverse,
    }
}

pub(super) fn text_x_for_byte(layout: &TextLayoutInfo, text: &str, byte: usize) -> f32 {
    if byte == 0 || layout.glyphs.is_empty() {
        return 0.0;
    }
    let glyph_scale = layout.scale_factor.max(f32::EPSILON);

    for glyph in &layout.glyphs {
        if byte == glyph.byte_index {
            return glyph_left_x(glyph, glyph_scale);
        }
    }

    let mut current_x = 0.0;
    for (index, glyph) in layout.glyphs.iter().enumerate() {
        let glyph_x = glyph_left_x(glyph, glyph_scale);
        if byte <= glyph.byte_index {
            return glyph_x;
        }
        current_x = glyph_trailing_x(layout, text, index, glyph_scale);
        if byte <= glyph.byte_index + glyph.byte_length {
            return current_x;
        }
    }

    current_x
}

pub(super) fn text_byte_for_x(layout: &TextLayoutInfo, text: &str, x: f32) -> usize {
    if layout.glyphs.is_empty() {
        return 0;
    }
    if x <= 0.0 {
        return 0;
    }
    let glyph_scale = layout.scale_factor.max(f32::EPSILON);

    for (index, glyph) in layout.glyphs.iter().enumerate() {
        let start = glyph_left_x(glyph, glyph_scale);
        let end = glyph_trailing_x(layout, text, index, glyph_scale);
        let width = end - start;
        let midpoint = start + width * 0.5;
        if x < midpoint {
            return glyph.byte_index;
        }
        if x <= end {
            return glyph.byte_index + glyph.byte_length;
        }
    }

    layout
        .glyphs
        .last()
        .map(|glyph| glyph.byte_index + glyph.byte_length)
        .unwrap_or(0)
}

pub(super) fn text_byte_for_point(
    layout: &TextLayoutInfo,
    text: &str,
    x: f32,
    y: f32,
) -> usize {
    let lines = text_line_metrics(layout, text);
    if lines.is_empty() {
        return 0;
    }

    let line = lines
        .iter()
        .find(|line| y >= line.top && y <= line.bottom)
        .or_else(|| {
            lines
                .iter()
                .min_by(|a, b| line_distance(y, a).total_cmp(&line_distance(y, b)))
        })
        .copied()
        .unwrap_or(lines[0]);

    text_byte_for_x_in_line(layout, text, x, line.line_index, line.start_byte, line.end_byte)
}

pub(super) fn caret_geometry_for_byte(
    layout: &TextLayoutInfo,
    text: &str,
    byte: usize,
) -> Option<(f32, f32, f32)> {
    let lines = text_line_metrics(layout, text);
    if lines.is_empty() {
        return None;
    }
    let line = line_for_caret_byte(&lines, byte)?;
    let x = text_x_for_byte_in_line(layout, text, byte, line.line_index);
    Some((x, line.top, line.bottom - line.top))
}

pub(super) fn move_byte_vertically(
    layout: &TextLayoutInfo,
    text: &str,
    byte: usize,
    current_x: Option<f32>,
    direction: i32,
) -> Option<(usize, f32)> {
    if direction == 0 {
        return Some((byte, current_x.unwrap_or(0.0)));
    }
    let lines = text_line_metrics(layout, text);
    if lines.is_empty() {
        return None;
    }
    let current = line_for_caret_byte(&lines, byte)?;
    let target_index = current.line_index as i32 + direction;
    if target_index < 0 {
        return Some((0, current_x.unwrap_or(current.left)));
    }
    let target = lines.iter().find(|line| line.line_index == target_index as usize);
    let Some(target) = target.copied() else {
        let end = text.len();
        return Some((end, current_x.unwrap_or(current.right)));
    };
    let preferred_x =
        current_x.unwrap_or_else(|| text_x_for_byte_in_line(layout, text, byte, current.line_index));
    let target_byte = text_byte_for_x_in_line(
        layout,
        text,
        preferred_x,
        target.line_index,
        target.start_byte,
        target.end_byte,
    );
    Some((target_byte, preferred_x))
}

pub(super) fn selection_rects_for_range(
    layout: &TextLayoutInfo,
    text: &str,
    start: usize,
    end: usize,
) -> Vec<(f32, f32, f32, f32)> {
    if start >= end {
        return Vec::new();
    }
    let mut rects = Vec::new();
    for line in text_line_metrics(layout, text) {
        let range_start = start.max(line.start_byte);
        let range_end = end.min(line.end_byte);
        if range_start >= range_end {
            continue;
        }
        let left = text_x_for_byte_in_line(layout, text, range_start, line.line_index);
        let right = text_x_for_byte_in_line(layout, text, range_end, line.line_index);
        rects.push((left, line.top, (right - left).max(0.0), line.bottom - line.top));
    }
    rects
}

fn glyph_left_x(glyph: &PositionedGlyph, scale: f32) -> f32 {
    (glyph.position.x - glyph.size.x * 0.5) / scale
}

fn text_line_metrics(layout: &TextLayoutInfo, text: &str) -> Vec<TextLineMetrics> {
    if layout.glyphs.is_empty() {
        return vec![TextLineMetrics {
            line_index: 0,
            start_byte: 0,
            end_byte: text.len(),
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: layout.size.y.max(0.0),
        }];
    }

    let scale = layout.scale_factor.max(f32::EPSILON);
    let mut lines = Vec::new();
    let mut index = 0;
    while index < layout.glyphs.len() {
        let first = &layout.glyphs[index];
        let line_index = first.line_index;
        let start_index = index;
        let mut end_index = index;
        while end_index + 1 < layout.glyphs.len() && layout.glyphs[end_index + 1].line_index == line_index
        {
            end_index += 1;
        }
        let line_glyphs = &layout.glyphs[start_index..=end_index];
        let left = line_glyphs
            .iter()
            .map(|glyph| glyph_left_x(glyph, scale))
            .fold(f32::INFINITY, f32::min);
        let right = line_glyphs
            .iter()
            .enumerate()
            .map(|(offset, _)| glyph_trailing_x(layout, text, start_index + offset, scale))
            .fold(0.0, f32::max);
        let top = line_glyphs
            .iter()
            .map(|glyph| (glyph.position.y - glyph.size.y * 0.5) / scale)
            .fold(f32::INFINITY, f32::min);
        let bottom = line_glyphs
            .iter()
            .map(|glyph| (glyph.position.y + glyph.size.y * 0.5) / scale)
            .fold(0.0, f32::max);
        lines.push(TextLineMetrics {
            line_index,
            start_byte: first.byte_index,
            end_byte: layout.glyphs[end_index].byte_index + layout.glyphs[end_index].byte_length,
            left: if left.is_finite() { left } else { 0.0 },
            right,
            top: if top.is_finite() { top } else { 0.0 },
            bottom,
        });
        index = end_index + 1;
    }
    lines
}

fn text_x_for_byte_in_line(layout: &TextLayoutInfo, text: &str, byte: usize, line_index: usize) -> f32 {
    let scale = layout.scale_factor.max(f32::EPSILON);
    let glyphs = layout
        .glyphs
        .iter()
        .enumerate()
        .filter(|(_, glyph)| glyph.line_index == line_index)
        .collect::<Vec<_>>();
    if glyphs.is_empty() {
        return 0.0;
    }
    let first = glyphs[0].1;
    if byte <= first.byte_index {
        return glyph_left_x(first, scale);
    }
    let mut current_x = glyph_left_x(first, scale);
    for (index, glyph) in glyphs {
        if byte <= glyph.byte_index {
            return glyph_left_x(glyph, scale);
        }
        current_x = glyph_trailing_x(layout, text, index, scale);
        if byte <= glyph.byte_index + glyph.byte_length {
            return current_x;
        }
    }
    current_x
}

fn text_byte_for_x_in_line(
    layout: &TextLayoutInfo,
    text: &str,
    x: f32,
    line_index: usize,
    line_start: usize,
    line_end: usize,
) -> usize {
    let scale = layout.scale_factor.max(f32::EPSILON);
    let glyphs = layout
        .glyphs
        .iter()
        .enumerate()
        .filter(|(_, glyph)| glyph.line_index == line_index)
        .collect::<Vec<_>>();
    if glyphs.is_empty() {
        return line_start.min(line_end);
    }
    if x <= glyph_left_x(glyphs[0].1, scale) {
        return line_start;
    }
    for (index, glyph) in glyphs {
        let start = glyph_left_x(glyph, scale);
        let end = glyph_trailing_x(layout, text, index, scale);
        let midpoint = start + (end - start) * 0.5;
        if x < midpoint {
            return glyph.byte_index;
        }
        if x <= end {
            return (glyph.byte_index + glyph.byte_length).min(line_end);
        }
    }
    line_end
}

fn line_distance(y: f32, line: &TextLineMetrics) -> f32 {
    if y < line.top {
        line.top - y
    } else if y > line.bottom {
        y - line.bottom
    } else {
        0.0
    }
}

fn line_for_caret_byte(lines: &[TextLineMetrics], byte: usize) -> Option<TextLineMetrics> {
    for (index, line) in lines.iter().enumerate() {
        if byte >= line.start_byte && byte <= line.end_byte {
            if let Some(next) = lines.get(index + 1)
                && byte == next.start_byte
            {
                return Some(*next);
            }
            return Some(*line);
        }
        if let Some(next) = lines.get(index + 1)
            && byte > line.end_byte
            && byte < next.start_byte
        {
            return Some(*line);
        }
    }
    lines.last().copied().or_else(|| lines.first().copied())
}

fn glyph_right_x(glyph: &PositionedGlyph, scale: f32) -> f32 {
    (glyph.position.x + glyph.size.x * 0.5) / scale
}

fn glyph_trailing_x(layout: &TextLayoutInfo, text: &str, index: usize, scale: f32) -> f32 {
    let glyphs = &layout.glyphs;
    let glyph = &glyphs[index];
    let next_index = index + 1;
    if let Some(next) = glyphs.get(next_index)
        && next.line_index == glyph.line_index
        && next.byte_index == glyph.byte_index + glyph.byte_length
    {
        return glyph_left_x(next, scale);
    }

    if glyph_text(text, glyph).chars().all(char::is_whitespace) {
        if let Some(previous) = previous_contiguous_glyph(glyphs, index, glyph) {
            let previous_left = glyph_left_x(previous, scale);
            let current_left = glyph_left_x(glyph, scale);
            let inferred_advance = (current_left - previous_left).max(0.0);
            return current_left + inferred_advance;
        }
        return layout_right_edge(layout).max(glyph_right_x(glyph, scale));
    }

    glyph_right_x(glyph, scale)
}

fn layout_right_edge(layout: &TextLayoutInfo) -> f32 {
    let run_max_x = layout
        .run_geometry
        .iter()
        .map(|run| run.bounds.max.x)
        .fold(0.0, f32::max);
    layout.size.x.max(run_max_x)
}

fn glyph_text<'a>(text: &'a str, glyph: &PositionedGlyph) -> &'a str {
    text.get(glyph.byte_index..glyph.byte_index + glyph.byte_length)
        .unwrap_or("")
}

fn previous_contiguous_glyph<'a>(
    glyphs: &'a [PositionedGlyph],
    index: usize,
    glyph: &PositionedGlyph,
) -> Option<&'a PositionedGlyph> {
    let previous_index = index.checked_sub(1)?;
    let previous = glyphs.get(previous_index)?;
    (previous.line_index == glyph.line_index
        && previous.byte_index + previous.byte_length == glyph.byte_index)
        .then_some(previous)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::AssetId;
    use bevy::image::Image;
    use bevy::math::{IVec2, Vec2};
    use bevy::text::{GlyphAtlasInfo, GlyphAtlasLocation};

    #[test]
    fn text_x_for_byte_uses_glyph_edges_not_centers() {
        let layout = layout_with_glyphs(&[(0, 1, 15.0, 10.0), (1, 1, 25.0, 10.0)]);
        let text = "ab";

        assert_eq!(text_x_for_byte(&layout, text, 0), 0.0);
        assert_eq!(text_x_for_byte(&layout, text, 1), 20.0);
        assert_eq!(text_x_for_byte(&layout, text, 2), 30.0);
    }

    #[test]
    fn text_byte_for_x_uses_glyph_edge_midpoints() {
        let layout = layout_with_glyphs(&[(0, 1, 15.0, 10.0), (1, 1, 25.0, 10.0)]);
        let text = "ab";

        assert_eq!(text_byte_for_x(&layout, text, 9.9), 0);
        assert_eq!(text_byte_for_x(&layout, text, 15.0), 1);
        assert_eq!(text_byte_for_x(&layout, text, 24.9), 1);
        assert_eq!(text_byte_for_x(&layout, text, 25.0), 2);
    }

    #[test]
    fn text_positions_use_next_glyph_edge_after_narrow_space_glyph() {
        let layout = layout_with_glyphs(&[(0, 1, 1.0, 2.0), (1, 1, 15.0, 10.0)]);
        let text = " a";

        assert_eq!(text_x_for_byte(&layout, text, 0), 0.0);
        assert_eq!(text_x_for_byte(&layout, text, 1), 10.0);
        assert_eq!(text_x_for_byte(&layout, text, 2), 20.0);
        assert_eq!(text_byte_for_x(&layout, text, 4.9), 0);
        assert_eq!(text_byte_for_x(&layout, text, 5.0), 1);
        assert_eq!(text_byte_for_x(&layout, text, 15.0), 2);
    }

    #[test]
    fn text_positions_use_layout_edge_for_trailing_narrow_space_glyph() {
        let mut layout = layout_with_glyphs(&[(0, 1, 1.0, 2.0)]);
        layout.size.x = 10.0;
        let text = " ";

        assert_eq!(text_x_for_byte(&layout, text, 0), 0.0);
        assert_eq!(text_x_for_byte(&layout, text, 1), 10.0);
        assert_eq!(text_byte_for_x(&layout, text, 4.9), 0);
        assert_eq!(text_byte_for_x(&layout, text, 5.0), 1);
    }

    #[test]
    fn trailing_space_after_regular_glyph_uses_neighbor_advance_not_layout_width() {
        let mut layout = layout_with_glyphs(&[(0, 1, 5.0, 10.0), (1, 1, 11.0, 2.0)]);
        layout.size.x = 80.0;
        let text = "a ";

        assert_eq!(text_x_for_byte(&layout, text, 1), 10.0);
        assert_eq!(text_x_for_byte(&layout, text, 2), 20.0);
        assert_eq!(text_byte_for_x(&layout, text, 14.9), 1);
        assert_eq!(text_byte_for_x(&layout, text, 15.0), 2);
    }

    #[test]
    fn caret_geometry_uses_next_line_when_byte_is_line_start() {
        let layout = TextLayoutInfo {
            scale_factor: 1.0,
            glyphs: vec![
                positioned_glyph(0, 1, 0, 5.0, 8.0, 10.0, 16.0),
                positioned_glyph(1, 1, 1, 5.0, 26.0, 10.0, 16.0),
            ],
            run_geometry: Vec::new(),
            size: Vec2::new(20.0, 34.0),
        };

        let (_, top, height) = caret_geometry_for_byte(&layout, "ab", 1).expect("caret geometry");

        assert_eq!(top, 18.0);
        assert_eq!(height, 16.0);
    }

    fn layout_with_glyphs(glyphs: &[(usize, usize, f32, f32)]) -> TextLayoutInfo {
        TextLayoutInfo {
            scale_factor: 1.0,
            glyphs: glyphs
                .iter()
                .map(
                    |(byte_index, byte_length, center_x, width)| PositionedGlyph {
                        position: Vec2::new(*center_x, 12.0),
                        size: Vec2::new(*width, 16.0),
                        atlas_info: GlyphAtlasInfo {
                            texture: AssetId::<Image>::invalid(),
                            texture_atlas: AssetId::invalid(),
                            location: GlyphAtlasLocation {
                                glyph_index: 0,
                                offset: IVec2::ZERO,
                            },
                        },
                        span_index: 0,
                        line_index: 0,
                        byte_index: *byte_index,
                        byte_length: *byte_length,
                    },
                )
                .collect(),
            run_geometry: Vec::new(),
            size: Vec2::new(30.0, 16.0),
        }
    }

    fn positioned_glyph(
        byte_index: usize,
        byte_length: usize,
        line_index: usize,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
    ) -> PositionedGlyph {
        PositionedGlyph {
            position: Vec2::new(center_x, center_y),
            size: Vec2::new(width, height),
            atlas_info: GlyphAtlasInfo {
                texture: AssetId::<Image>::invalid(),
                texture_atlas: AssetId::invalid(),
                location: GlyphAtlasLocation {
                    glyph_index: 0,
                    offset: IVec2::ZERO,
                },
            },
            span_index: 0,
            line_index,
            byte_index,
            byte_length,
        }
    }
}
