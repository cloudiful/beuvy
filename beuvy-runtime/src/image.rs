mod build;

use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageSet {
    Build,
}

#[derive(Component, Debug, Clone, Default)]
pub struct AddImage {
    pub src: String,
    pub alt: String,
    pub class: Option<String>,
}

pub struct ImagePlugin;

impl Plugin for ImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, build::add_image.in_set(ImageSet::Build));
    }
}
