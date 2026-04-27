use crate::runtime::state::{DeclarativeRefRects, DeclarativeResolvedRef};
use crate::value::UiValue;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

pub(crate) fn sync_declarative_ref_rects(
    mut rects: ResMut<DeclarativeRefRects>,
    refs: Query<(&DeclarativeResolvedRef, &ComputedNode, &UiGlobalTransform)>,
) {
    rects.clear();
    for (ref_id, computed, transform) in &refs {
        let rect = ui_rect_value(computed, transform);
        rects.set_rect(ref_id.0.clone(), rect);
    }
}

fn ui_rect_value(computed: &ComputedNode, transform: &UiGlobalTransform) -> UiValue {
    let (_, _, translation) = transform.to_scale_angle_translation();
    let scale = computed.inverse_scale_factor();
    let size = computed.size() * scale;
    let center = translation;
    let left = center.x - size.x * 0.5;
    let top = center.y - size.y * 0.5;
    UiValue::object([
        ("left", UiValue::from(left)),
        ("top", UiValue::from(top)),
        ("right", UiValue::from(left + size.x)),
        ("bottom", UiValue::from(top + size.y)),
        ("x", UiValue::from(left)),
        ("y", UiValue::from(top)),
        ("width", UiValue::from(size.x)),
        ("height", UiValue::from(size.y)),
    ])
}
