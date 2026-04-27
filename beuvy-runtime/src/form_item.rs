use crate::build_pending::UiBuildPending;
use crate::style::{
    apply_utility_patch, form_item_compact_width, resolve_class_patch_or_empty,
    resolve_classes_with_fallback, root_visual_styles_from_patch,
};
use crate::text::AddText;
use bevy::prelude::*;
use bevy::text::TextLayout;
use bevy::ui::UiScale;
use bevy_localization::TextKey;

const DEFAULT_FORM_ITEM_CLASS: &str = "form-item";
const DEFAULT_FORM_ITEM_COMPACT_CLASS: &str = "form-item-compact";
const DEFAULT_FORM_ITEM_LABEL_CLASS: &str = "form-item-label";
const DEFAULT_FORM_ITEM_LABEL_COMPACT_CLASS: &str = "form-item-label-compact";

#[derive(Component, Default, Debug, Clone)]
pub struct FormItem {
    compact: Option<bool>,
}

#[derive(Component, Default, Debug, Clone)]
pub struct FormItemLabel;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormItemSet {
    Build,
}

impl Plugin for FormItem {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_form_item.in_set(FormItemSet::Build),
                sync_form_item_layouts,
            ),
        );
    }
}

#[derive(Component, Default, Debug, Clone, PartialEq, Eq)]
pub struct AddFormItem {
    pub text: String,
    pub text_key: Option<TextKey>,
}

impl AddFormItem {
    pub fn localized(text: impl Into<String>, text_key: Option<TextKey>) -> Self {
        Self {
            text: text.into(),
            text_key,
        }
    }
}

pub fn form_item_node() -> Node {
    let mut node = Node::default();
    let patch = resolve_class_patch_or_empty(DEFAULT_FORM_ITEM_CLASS, "form item");
    apply_utility_patch(&mut node, &patch);
    node
}

pub fn form_item_label_node() -> Node {
    let mut node = Node::default();
    let patch = resolve_class_patch_or_empty(DEFAULT_FORM_ITEM_LABEL_CLASS, "form item label");
    apply_utility_patch(&mut node, &patch);
    node
}

fn add_form_item(mut commands: Commands, query: Query<(Entity, &AddFormItem)>) {
    for (entity, add_form_item) in query {
        let add_form_item = add_form_item.clone();
        commands
            .entity(entity)
            .queue_silenced(move |mut entity: EntityWorldMut| {
                let patch = resolve_class_patch_or_empty(DEFAULT_FORM_ITEM_CLASS, "form item");
                entity.insert((
                    FormItem::default(),
                    form_item_node(),
                    Visibility::Visible,
                    BackgroundColor(Color::NONE),
                ));
                if let Some(styles) = root_visual_styles_from_patch(&patch) {
                    entity.insert(styles);
                }

                let child = entity.world_scope(|world| {
                    world
                        .spawn((
                            FormItemLabel,
                            form_item_label_node(),
                            TextLayout::default(),
                            AddText {
                                text: add_form_item.text.clone(),
                                localized_text: add_form_item.text_key,
                                ..default()
                            },
                        ))
                        .id()
                });

                entity.insert_child(0, child);
                entity.remove::<AddFormItem>().remove::<UiBuildPending>();
            });
    }
}

fn sync_form_item_layouts(
    ui_scale: Res<UiScale>,
    mut items: Query<
        (&mut Node, Ref<ComputedNode>, &Children, &mut FormItem),
        (With<FormItem>, Without<FormItemLabel>),
    >,
    mut label_nodes: Query<&mut Node, (With<FormItemLabel>, Without<FormItem>)>,
) {
    let sync_all = ui_scale.is_changed();
    let compact_width = form_item_compact_width();

    for (mut item_node, computed, children, mut item) in &mut items {
        if !(sync_all || item.is_added() || computed.is_changed()) {
            continue;
        }

        let compact = computed.size().x < compact_width;
        if item.compact == Some(compact) && !sync_all {
            continue;
        }
        item.compact = Some(compact);

        *item_node = form_item_node();
        let patch = resolve_classes_with_fallback(
            DEFAULT_FORM_ITEM_CLASS,
            compact.then_some(DEFAULT_FORM_ITEM_COMPACT_CLASS),
            "form item",
        );
        apply_utility_patch(&mut item_node, &patch);

        for child in children.iter() {
            if let Ok(mut label_node) = label_nodes.get_mut(child) {
                *label_node = form_item_label_node();
                let label_patch = resolve_classes_with_fallback(
                    DEFAULT_FORM_ITEM_LABEL_CLASS,
                    compact.then_some(DEFAULT_FORM_ITEM_LABEL_COMPACT_CLASS),
                    "form item label",
                );
                apply_utility_patch(&mut label_node, &label_patch);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_item_plugin_initializes_without_conflicting_queries() {
        let mut app = App::new();
        app.init_resource::<UiScale>()
            .add_plugins(FormItem::default());

        app.update();
    }
}
