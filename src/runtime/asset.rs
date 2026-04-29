use super::state::{DeclarativeRefRects, DeclarativeUiRuntimeValues};
use super::sync::{
    apply_declarative_local_state_assignments, handle_declarative_label_click,
    infer_wrapped_label_targets, materialize_declarative_overflow_scroll,
    materialize_declarative_refs, sync_declarative_class_bindings, sync_declarative_disabled,
    sync_declarative_field_values, sync_declarative_image_bindings,
    sync_declarative_link_bindings, sync_declarative_node_style_bindings,
    sync_declarative_ref_rects, sync_declarative_text_bindings, sync_declarative_visibility,
    write_input_values_to_runtime_store, write_select_values_to_runtime_store,
};
use crate::ast::DeclarativeUiAsset;
use crate::error::DeclarativeUiAssetLoadError;
use crate::style::{BeuvyStyleSource, replace_style_source};
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use bevy::reflect::TypePath;

#[derive(Default, TypePath)]
pub struct DeclarativeUiAssetLoader;

impl AssetLoader for DeclarativeUiAssetLoader {
    type Asset = DeclarativeUiAsset;
    type Settings = ();
    type Error = DeclarativeUiAssetLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let raw = std::str::from_utf8(&bytes)
            .map_err(|error| DeclarativeUiAssetLoadError::InvalidUtf8(error.to_string()))?;
        crate::parse_declarative_ui_asset(raw)
    }

    fn extensions(&self) -> &[&str] {
        &["vue"]
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeclarativeUiPlugin {
    style_source: Option<BeuvyStyleSource>,
}

impl DeclarativeUiPlugin {
    pub fn with_style_source(style_source: BeuvyStyleSource) -> Self {
        Self {
            style_source: Some(style_source),
        }
    }
}

impl Plugin for DeclarativeUiPlugin {
    fn build(&self, app: &mut App) {
        if let Some(style_source) = &self.style_source {
            app.insert_resource(style_source.clone());
        } else {
            app.init_resource::<BeuvyStyleSource>();
        }
        let style_source = app.world().resource::<BeuvyStyleSource>().clone();
        replace_style_source(style_source);
        app.init_asset::<DeclarativeUiAsset>()
            .register_asset_loader(DeclarativeUiAssetLoader)
            .init_resource::<DeclarativeUiRuntimeValues>()
            .init_resource::<DeclarativeRefRects>()
            .add_systems(
                Update,
                (
                    sync_beuvy_style_source,
                    materialize_declarative_overflow_scroll,
                    materialize_declarative_refs,
                    sync_declarative_visibility,
                    sync_declarative_text_bindings,
                    sync_declarative_disabled,
                    sync_declarative_field_values,
                    sync_declarative_image_bindings,
                    sync_declarative_link_bindings,
                ),
            )
            .add_systems(Update, write_input_values_to_runtime_store)
            .add_systems(Update, write_select_values_to_runtime_store)
            .add_systems(Update, infer_wrapped_label_targets)
            .add_systems(Update, apply_declarative_local_state_assignments)
            .add_systems(Update, handle_declarative_label_click)
            .add_systems(
                Update,
                (
                    sync_declarative_class_bindings,
                    sync_declarative_ref_rects.after(materialize_declarative_refs),
                    sync_declarative_node_style_bindings.after(sync_declarative_class_bindings),
                    sync_declarative_node_style_bindings.after(sync_declarative_ref_rects),
                ),
            );
    }
}

fn sync_beuvy_style_source(source: Res<BeuvyStyleSource>) {
    if source.is_changed() {
        replace_style_source(source.clone());
    }
}
