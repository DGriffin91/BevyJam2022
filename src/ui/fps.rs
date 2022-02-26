use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::assets::{FontAssets, GameState};

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup_fps_counter))
            .add_system_set(
                SystemSet::on_update(GameState::Playing).with_system(update_fps_counter),
            );
    }
}

#[derive(Component)]
struct FpsCounter;

fn setup_fps_counter(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "0".to_string(),
                    style: TextStyle {
                        font: font_assets.fira_mono_medium.clone(),
                        font_size: 32.0,
                        color: Color::WHITE,
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FpsCounter);
}

fn update_fps_counter(
    diagnostics: Res<Diagnostics>,
    mut fps_counters: Query<&mut Text, With<FpsCounter>>,
) {
    if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps_diagnostic.value() {
            for mut fps_counter in fps_counters.iter_mut() {
                if let Some(mut section) = fps_counter.sections.get_mut(0) {
                    section.value = format!("{:.0}", fps);
                }
            }
        }
    }
}
