use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_kira_audio::AudioSource;
use rand::prelude::SliceRandom;

use self::{
    custom_material::CustomMaterial, emissive_material::EmissiveMaterial,
    light_shaft_material::LightShaftMaterial, orb_material::OrbMaterial,
    splash_screen::SplashScreenPlugin,
};

pub mod custom_material;
pub mod emissive_material;
pub mod light_shaft_material;
mod material_util;
pub mod orb_material;
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
            .add_plugin(MaterialPlugin::<OrbMaterial>::default())
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
    // General
    #[asset(path = "models/standard_plane2.glb#Mesh0/Primitive0")]
    pub standard_plane: Handle<Mesh>,
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

    // Folder import not working in wasm:
    // #[asset(path = "audio/footsteps", folder)]
    // pub footsteps: Vec<HandleUntyped>,
    // panicked at 'called `Result::unwrap()` on an `Err` value:
    // AssetFolderNotADirectory("audio/footsteps")', src\assets\mod.rs:104:10

    // Steps
    #[asset(path = "audio/footsteps/step01.ogg")]
    pub step01: Handle<AudioSource>,
    #[asset(path = "audio/footsteps/step02.ogg")]
    pub step02: Handle<AudioSource>,
    #[asset(path = "audio/footsteps/step03.ogg")]
    pub step03: Handle<AudioSource>,
    #[asset(path = "audio/footsteps/step04.ogg")]
    pub step04: Handle<AudioSource>,
    #[asset(path = "audio/footsteps/step05.ogg")]
    pub step05: Handle<AudioSource>,
    #[asset(path = "audio/footsteps/step06.ogg")]
    pub step06: Handle<AudioSource>,
    #[asset(path = "audio/footsteps/step07.ogg")]
    pub step07: Handle<AudioSource>,
    #[asset(path = "audio/footsteps/step08.ogg")]
    pub step08: Handle<AudioSource>,
    #[asset(path = "audio/footsteps/step09.ogg")]
    pub step09: Handle<AudioSource>,
    #[asset(path = "audio/footsteps/step10.ogg")]
    pub step10: Handle<AudioSource>,

    // Hurt
    #[asset(path = "audio/hurt/hurt-001.ogg")]
    pub hurt01: Handle<AudioSource>,
    #[asset(path = "audio/hurt/hurt-002.ogg")]
    pub hurt02: Handle<AudioSource>,
    #[asset(path = "audio/hurt/hurt-003.ogg")]
    pub hurt03: Handle<AudioSource>,
    #[asset(path = "audio/hurt/hurt-004.ogg")]
    pub hurt04: Handle<AudioSource>,
    #[asset(path = "audio/hurt/hurt-005.ogg")]
    pub hurt05: Handle<AudioSource>,
    #[asset(path = "audio/hurt/hurt-006.ogg")]
    pub hurt06: Handle<AudioSource>,
    #[asset(path = "audio/hurt/hurt-007.ogg")]
    pub hurt07: Handle<AudioSource>,
    #[asset(path = "audio/hurt/hurt-008.ogg")]
    pub hurt08: Handle<AudioSource>,

    // Lasergun
    #[asset(path = "audio/weapons/lasergun/lasergun01.ogg")]
    pub lasergun01: Handle<AudioSource>,
    #[asset(path = "audio/weapons/lasergun/lasergun02.ogg")]
    pub lasergun02: Handle<AudioSource>,
    #[asset(path = "audio/weapons/lasergun/lasergun03.ogg")]
    pub lasergun03: Handle<AudioSource>,
    #[asset(path = "audio/weapons/lasergun/lasergun04.ogg")]
    pub lasergun04: Handle<AudioSource>,
    #[asset(path = "audio/weapons/lasergun/lasergun05.ogg")]
    pub lasergun05: Handle<AudioSource>,
    #[asset(path = "audio/weapons/lasergun/lasergun06.ogg")]
    pub lasergun06: Handle<AudioSource>,
    #[asset(path = "audio/weapons/lasergun/lasergun07.ogg")]
    pub lasergun07: Handle<AudioSource>,
}

impl AudioAssets {
    pub fn get_hurt(&self) -> &Handle<AudioSource> {
        [
            &self.hurt01,
            &self.hurt02,
            &self.hurt03,
            &self.hurt04,
            &self.hurt05,
            &self.hurt06,
            &self.hurt07,
            &self.hurt08,
        ]
        .choose(&mut rand::thread_rng())
        .unwrap()
    }
    pub fn get_step(&self) -> &Handle<AudioSource> {
        [
            &self.step01,
            &self.step02,
            &self.step03,
            &self.step04,
            &self.step05,
            &self.step06,
            &self.step07,
            &self.step08,
            &self.step09,
            &self.step10,
        ]
        .choose(&mut rand::thread_rng())
        .unwrap()
    }
    pub fn get_lasergun(&self) -> &Handle<AudioSource> {
        [
            &self.lasergun01,
            &self.lasergun02,
            &self.lasergun03,
            &self.lasergun04,
            &self.lasergun05,
            &self.lasergun06,
            &self.lasergun07,
        ]
        .choose(&mut rand::thread_rng())
        .unwrap()
    }
}
