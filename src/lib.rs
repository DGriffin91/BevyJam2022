use assets::AssetsPlugin;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_polyline::PolylinePlugin;
use heron::{Gravity, PhysicsPlugin};
use player::PlayerPlugin;
use ui::UiPlugin;
use world::WorldPlugin;

mod assets;
mod player;
mod ui;
mod world;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // External plugins
            .add_plugin(EguiPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(PolylinePlugin)
            .add_plugin(PhysicsPlugin::default())
            .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
            // Game plugins
            .add_plugin(AssetsPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(UiPlugin)
            .add_plugin(WorldPlugin);
    }
}
