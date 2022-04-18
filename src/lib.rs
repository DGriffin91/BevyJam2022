use assets::AssetsPlugin;
use audio::GameAudioPlugin;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_kira_audio::AudioPlugin;
use bevy_polyline::PolylinePlugin;
use bevy_tweening::TweeningPlugin;
//use console::ConsolePlugin;
use enemies::EnemiesPlugin;
use heron::{Gravity, PhysicsLayer, PhysicsPlugin};
use player::PlayerPlugin;
use ui::UiPlugin;
use world::WorldPlugin;

mod assets;
mod audio;
//mod console;
mod enemies;
mod player;
mod ui;
mod world;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // External plugins
            .add_plugin(AudioPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(PolylinePlugin)
            .add_plugin(PhysicsPlugin::default())
            .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
            .add_plugin(TweeningPlugin)
            // Game plugins
            .add_plugin(AssetsPlugin)
            //.add_plugin(ConsolePlugin) // Need 0.7 compatible version
            .add_plugin(EnemiesPlugin)
            .add_plugin(GameAudioPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(UiPlugin)
            .add_plugin(WorldPlugin)
            // Main systems
            .add_startup_system(mouse_startup);
    }
}

#[derive(PhysicsLayer)]
enum Layer {
    Bullet,
    Enemy,
    Player,
    Raycast,
    World,
}

impl Layer {
    fn all() -> [Layer; 5] {
        [
            Layer::Bullet,
            Layer::Enemy,
            Layer::Player,
            Layer::Raycast,
            Layer::World,
        ]
    }
}

fn mouse_startup(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_lock_mode(false);
    window.set_cursor_visibility(true);
}
