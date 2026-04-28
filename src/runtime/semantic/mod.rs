mod fieldset;
mod form;
mod label;

pub(crate) use fieldset::sync_fieldset_disabled_states;
pub(crate) use form::{handle_form_button_clicks, handle_form_input_submits};
pub(crate) use label::handle_label_clicks;

use bevy::prelude::*;

pub(crate) fn nearest_ancestor(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    predicate: impl Fn(Entity) -> bool,
) -> Option<Entity> {
    while let Ok(parent) = parents.get(entity) {
        entity = parent.parent();
        if predicate(entity) {
            return Some(entity);
        }
    }
    None
}

pub(crate) fn walk_descendants(
    entity: Entity,
    children_query: &Query<&Children>,
    visit: &mut impl FnMut(Entity),
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            visit(child);
            walk_descendants(child, children_query, visit);
        }
    }
}
