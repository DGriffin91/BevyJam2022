use bevy::prelude::*;
use bevy_console::{AddConsoleCommand, ConsoleConfiguration, ConsoleOpen};

use self::debug_pos::{debug_pos, DebugPos};

mod debug_pos;

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_console::ConsolePlugin)
            .insert_resource(ConsoleConfiguration {
                width: 1520.0,
                height: 880.0,
                ..Default::default()
            })
            .add_console_command::<DebugPos, _, _>(debug_pos)
            .add_system(toggle_cursor_with_console);
    }
}

fn toggle_cursor_with_console(console_open: Res<ConsoleOpen>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    if console_open.open && (window.cursor_locked() || !window.cursor_visible()) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }
}
