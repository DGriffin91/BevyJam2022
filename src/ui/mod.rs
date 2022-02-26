use bevy::prelude::*;

use self::{fps::FpsPlugin, scoreboard::ScoreboardPlugin};

// pub mod menu;
pub mod fps;
pub mod menu;
pub mod scoreboard;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugin(MenuPlugin)
            .add_plugin(FpsPlugin)
            .add_plugin(ScoreboardPlugin)
            .add_startup_system(setup_ui_camera);
    }
}

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}
