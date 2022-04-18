use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use self::{fps::FpsPlugin, hud::HudPlugin, menu::MenuPlugin, scoreboard::ScoreboardPlugin};

pub mod fps;
pub mod hud;
pub mod menu;
pub mod scoreboard;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(HudPlugin)
            .add_plugin(FpsPlugin)
            .add_plugin(ScoreboardPlugin)
            .add_startup_system(setup_ui_camera);
    }
}

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}
