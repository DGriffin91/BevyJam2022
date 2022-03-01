use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_kira_audio::AudioSource;

use self::{
    custom_material::CustomMaterial, emissive_material::EmissiveMaterial,
    light_shaft_material::LightShaftMaterial, splash_screen::SplashScreenPlugin,
};

pub mod custom_material;
pub mod emissive_material;
pub mod light_shaft_material;
mod material_util;
mod splash_screen;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        AssetLoader::new(GameState::Loading)
            .continue_to_state(GameState::Playing)
            .with_collection::<FontAssets>()
            .with_collection::<ImageAssets>()
            .with_collection::<ModelAssets>()
            .with_collection::<AudioAssets>()
            .build(app);

        app.add_plugin(SplashScreenPlugin)
            .add_plugin(MaterialPlugin::<CustomMaterial>::default())
            .add_plugin(MaterialPlugin::<EmissiveMaterial>::default())
            .add_plugin(MaterialPlugin::<LightShaftMaterial>::default())
            .add_state(GameState::Menu);
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Loading,
    Menu,
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
    // Shared Textures
    #[asset(key = "concrete")]
    pub concrete: Handle<Image>,
    #[asset(key = "concrete3")]
    pub concrete3: Handle<Image>,
    #[asset(key = "detail")]
    pub detail: Handle<Image>,
    #[asset(key = "reflection")]
    pub reflection: Handle<Image>,
    // Level pieces
    #[asset(key = "level1_large_ceiling_supports")]
    pub level1_large_ceiling_supports: Handle<Image>,
    #[asset(key = "level1_pillars")]
    pub level1_pillars: Handle<Image>,
    #[asset(key = "level1_sky_box")]
    pub level1_sky_box: Handle<Image>,
    #[asset(key = "level1_spheres_base")]
    pub level1_spheres_base: Handle<Image>,
    #[asset(key = "level1_spheres")]
    pub level1_spheres: Handle<Image>,
    #[asset(key = "level1_walls")]
    pub level1_walls: Handle<Image>,
}

#[derive(AssetCollection)]
pub struct ModelAssets {
    // Level pieces
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
    #[asset(path = "models/level1/light_shafts.gltf#Mesh0/Primitive0")]
    pub level1_light_shafts: Handle<Mesh>,
    // Units
    #[asset(path = "models/units/unit1.glb#Scene0")]
    pub unit1: Handle<Scene>,
    #[asset(path = "models/units/unit2.glb#Scene0")]
    pub unit2: Handle<Scene>,
    // Weapons
    #[asset(path = "models/weapons/lasergun.glb#Scene0")]
    pub lasergun: Handle<Scene>,
}

#[derive(AssetCollection)]
pub struct AudioAssets {
    #[asset(path = "audio/atmosphere.ogg")]
    pub atmosphere: Handle<AudioSource>,
    #[asset(path = "audio/footsteps", folder)]
    pub footsteps: Vec<HandleUntyped>,
    #[asset(path = "audio/hurt", folder)]
    pub hurt: Vec<HandleUntyped>,
    #[asset(path = "audio/weapons/lasergun", folder)]
    pub lasergun: Vec<HandleUntyped>,
}
