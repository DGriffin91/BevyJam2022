use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{WindowMode, WindowResizeConstraints},
};
use bevy_egui::EguiPlugin;
use bevy_polyline::PolylinePlugin;
use game::{assets::AssetsPlugin, player::PlayerPlugin, ui::UiPlugin, world::WorldPlugin};
use heron::PhysicsPlugin;

fn main() {
    App::new()
        // External plugins
        .insert_resource(WindowDescriptor {
            title: "app".to_string(),
            width: 1280.0,
            height: 720.0,
            position: None,
            resize_constraints: WindowResizeConstraints {
                min_width: 256.0,
                min_height: 256.0,
                ..Default::default()
            },
            scale_factor_override: Some(1.0),
            vsync: false,
            resizable: true,
            decorations: true,
            cursor_locked: true,
            cursor_visible: false,
            mode: WindowMode::Windowed,
            transparent: false,
            #[cfg(target_arch = "wasm32")]
            canvas: Some(String::from("#can")),
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(PolylinePlugin)
        .add_plugin(PhysicsPlugin::default())
        // Game plugins
        .add_plugin(AssetsPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(WorldPlugin)
        .run();
}
