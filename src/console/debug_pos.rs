use bevy::prelude::*;
use bevy_console::{reply, ConsoleCommand};

use crate::player::{Player, PlayerCam};

/// Print the player position
#[derive(ConsoleCommand)]
#[console_command(name = "debug_pos")]
pub struct DebugPos;

pub fn debug_pos(
    mut cmd: ConsoleCommand<DebugPos>,
    players: Query<(&Transform, &Children), With<Player>>,
    cameras: Query<&Transform, (With<PlayerCam>, Without<Player>)>,
) {
    if cmd.take().is_some() {
        for (player_transform, children) in players.iter() {
            if let Some(cam_transform) = children.iter().find_map(|child| cameras.get(*child).ok())
            {
                reply!(
                    cmd,
                    "Pos: {:.2}, {:.2}, {:.2}\nRotation: {:.2}, {:.2}, {:.2}",
                    player_transform.translation.x,
                    player_transform.translation.y,
                    player_transform.translation.z,
                    cam_transform.rotation.x,
                    player_transform.rotation.y,
                    player_transform.rotation.z,
                );
            }
        }
    }
}
