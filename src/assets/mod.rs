use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

use self::{custom_material::CustomMaterial, emissive_material::EmissiveMaterial};

pub mod custom_material;
pub mod emissive_material;
mod material_util;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        AssetLoader::new(GameState::Loading)
            .continue_to_state(GameState::Playing)
            .with_collection::<FontAssets>()
            .with_collection::<ImageAssets>()
            .with_collection::<ModelAssets>()
            .build(app);

        app.add_plugin(MaterialPlugin::<CustomMaterial>::default())
            .add_plugin(MaterialPlugin::<EmissiveMaterial>::default())
            .add_state(GameState::Loading);
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Loading,
    Playing,
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraMono-Medium.ttf")]
    pub fira_mono_medium: Handle<Font>,
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans_bold: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct ImageAssets {
    #[asset(path = "textures/concrete.jpg")]
    pub concrete: Handle<Image>,
    #[asset(path = "textures/concrete3.jpg")]
    pub concrete3: Handle<Image>,
    #[asset(path = "textures/detail.jpg")]
    pub detail: Handle<Image>,
    #[asset(path = "textures/reflection.jpg")]
    pub reflection: Handle<Image>,
    #[asset(path = "textures/level1/bake/large_ceiling_supports.jpg")]
    pub level1_large_ceiling_supports: Handle<Image>,
    #[asset(path = "textures/level1/bake/pillars.jpg")]
    pub level1_pillars: Handle<Image>,
    #[asset(path = "textures/level1/bake/sky_box.jpg")]
    pub level1_sky_box: Handle<Image>,
    #[asset(path = "textures/level1/bake/spheres_base.jpg")]
    pub level1_spheres_base: Handle<Image>,
    #[asset(path = "textures/level1/bake/spheres.jpg")]
    pub level1_spheres: Handle<Image>,
    #[asset(path = "textures/level1/bake/walls.jpg")]
    pub level1_walls: Handle<Image>,
}

#[derive(AssetCollection)]
pub struct ModelAssets {
    #[asset(path = "models/level1/large_ceiling_supports.gltf#Mesh0/Primitive0")]
    pub level1_large_ceiling_supports: Handle<Mesh>,
    #[asset(path = "models/level1/pillars.gltf#Mesh0/Primitive0")]
    pub level1_pillars: Handle<Mesh>,
    #[asset(path = "models/level1/sky_box.gltf#Mesh0/Primitive0")]
    pub level1_sky_box: Handle<Mesh>,
    #[asset(path = "models/level1/spheres_base.gltf#Mesh0/Primitive0")]
    pub level1_spheres_base: Handle<Mesh>,
    #[asset(path = "models/level1/spheres.gltf#Mesh0/Primitive0")]
    pub level1_spheres: Handle<Mesh>,
    #[asset(path = "models/level1/walls.gltf#Mesh0/Primitive0")]
    pub level1_walls: Handle<Mesh>,
}
