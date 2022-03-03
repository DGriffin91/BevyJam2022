use bevy::prelude::*;

use crate::assets::{FontAssets, GameState};

pub struct ScoreboardPlugin;

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Scoreboard {
            kills: 0,
            hits: 0,
            misses: 0,
            level: 0,
        })
        .add_event::<ScoreboardEvent>()
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup_scoreboard))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(scoreboard_ui)
                .with_system(handle_scoreboard_event),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)] // TODO: Remove this when the scoreboard starts being used
pub enum ScoreboardEvent {
    Kill,
    LevelUp,
    _Hit,
    _Miss,
    Reset,
}

#[derive(Component, Default)]
struct Scoreboard {
    pub kills: usize,
    pub hits: usize,
    pub misses: usize,
    pub level: usize,
}

fn setup_scoreboard(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "".to_string(),
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
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Scoreboard::default());
}

fn scoreboard_ui(mut scoreboards: Query<(&mut Text, &Scoreboard), Changed<Scoreboard>>) {
    for (mut text, scoreboard) in scoreboards.iter_mut() {
        let kills = scoreboard.kills;
        text.sections[0].value = format!("Score: {} | Level: {}", kills * 100, scoreboard.level);
    }
}

fn handle_scoreboard_event(
    mut scoreboards: Query<&mut Scoreboard>,
    mut events: EventReader<ScoreboardEvent>,
) {
    for mut scoreboard in scoreboards.iter_mut() {
        for event in events.iter() {
            match event {
                ScoreboardEvent::_Hit => {
                    scoreboard.hits += 1;
                }
                ScoreboardEvent::Kill => {
                    scoreboard.kills += 1;
                }
                ScoreboardEvent::_Miss => {
                    scoreboard.misses += 1;
                }
                ScoreboardEvent::Reset => {
                    scoreboard.hits = 0;
                    scoreboard.misses = 0;
                    scoreboard.kills = 0;
                    scoreboard.level = 0;
                }
                ScoreboardEvent::LevelUp => {
                    scoreboard.level += 1;
                }
            }
        }
    }
}
