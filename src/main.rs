use bevy::{
    prelude::*,
    window::{WindowMode, WindowResizeConstraints},
};
use game::GamePlugin;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "app".to_string(),
            width: 1920.0,
            height: 1080.0,
            position: None,
            resize_constraints: WindowResizeConstraints {
                min_width: 256.0,
                min_height: 256.0,
                ..Default::default()
            },
            scale_factor_override: None, //Some(1.0), //Needed for some mobile devices, but disables scaling
            vsync: false,
            resizable: true,
            decorations: true,
            cursor_locked: false,
            cursor_visible: true,
            mode: WindowMode::Windowed,
            transparent: false,
            #[cfg(target_arch = "wasm32")]
            canvas: Some(String::from("#can")),
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}
