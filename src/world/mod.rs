use bevy::prelude::*;

use crate::assets::custom_material::{CustomMaterial, MaterialProperties};

use self::level1::LevelOnePlugin;

mod level1;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LevelOnePlugin);
    }
}

#[derive(Component, Debug)]
pub struct LevelAsset {
    pub material_properties: MaterialProperties,
    pub material_handle: Handle<CustomMaterial>,
}