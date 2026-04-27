use bevy::math::Rect;
use bevy::prelude::*;
use bevy::text::{PositionedGlyph, TextLayoutInfo};

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

fn glyph_left_x(glyph: &PositionedGlyph, scale: f32) -> f32 {
    (glyph.position.x - glyph.size.x * 0.5) / scale
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
}
