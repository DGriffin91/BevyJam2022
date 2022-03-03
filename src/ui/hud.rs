use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{Animator, EaseFunction, Lens, Tween, TweeningType};

use crate::{
    assets::{FontAssets, GameState},
    player::Player,
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(setup_health_bar)
                .with_system(setup_fail_message),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(update_health_bar)
                .with_system(update_fail_message)
                .with_system(update_message),
        );
    }
}

#[derive(Component)]
struct HealthBar;

struct HealthBarLens {
    start: f32,
    end: f32,
}

impl Lens<Style> for HealthBarLens {
    fn lerp(&mut self, target: &mut Style, ratio: f32) {
        target.size.width = Val::Percent(self.start + (self.end - self.start) * ratio);
    }
}

fn setup_health_bar(mut commands: Commands) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(90.0), Val::Px(20.0)),
                padding: Rect {
                    top: Val::Px(2.0),
                    right: Val::Px(2.0),
                    bottom: Val::Px(2.0),
                    left: Val::Px(2.0),
                },
                position: Rect {
                    left: Val::Percent(5.0),
                    bottom: Val::Px(40.0),
                    ..Default::default()
                },
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            color: Color::rgba(0.9, 0.9, 0.9, 0.2).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    color: Color::rgba(1.0, 0.0, 0.0, 0.7).into(),
                    ..Default::default()
                })
                .insert(Animator::new(Tween::new(
                    EaseFunction::QuadraticOut,
                    TweeningType::Once,
                    Duration::from_millis(400),
                    HealthBarLens {
                        start: 100.0,
                        end: 100.0,
                    },
                )))
                .insert(HealthBar);
        });
}

fn update_health_bar(
    players: Query<&Player, Changed<Player>>,
    mut health_bars: Query<(&mut Animator<Style>, &Style), With<HealthBar>>,
) {
    for Player { health, max_health } in players.iter() {
        for (mut animator, style) in health_bars.iter_mut() {
            if let Val::Percent(width) = style.size.width {
                let health_percent = *health as f32 / *max_health as f32;

                animator.set_tweenable(Tween::new(
                    EaseFunction::QuadraticOut,
                    TweeningType::Once,
                    Duration::from_millis(400),
                    HealthBarLens {
                        start: width,
                        end: health_percent * 100.0,
                    },
                ));
            }
        }
    }
}

#[derive(Component, PartialEq)]
pub enum ScreenMessage {
    Empty,
    Failed,
    PressFire,
}

fn setup_fail_message(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Fire to begin".to_string(),
                    style: TextStyle {
                        font: font_assets.fira_mono_medium.clone(),
                        font_size: 48.0,
                        color: Color::rgba(0.0, 0.0, 0.0, 1.0),
                    },
                }],
                // TODO tried to align center, gave up for now
                //alignment: TextAlignment {
                //    vertical: VerticalAlign::Center,
                //    horizontal: HorizontalAlign::Center,
                //},
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(10.0),
                    left: Val::Percent(10.0),
                    ..Default::default()
                },
                //align_content: AlignContent::Center,
                //align_items: AlignItems::Center,
                //align_self: AlignSelf::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScreenMessage::PressFire);
}

fn update_fail_message(
    players: Query<&Player, Changed<Player>>,
    mut screen_messages: Query<&mut ScreenMessage>,
) {
    for Player {
        health,
        max_health: _,
    } in players.iter()
    {
        for mut screen_message in screen_messages.iter_mut() {
            if health <= &0 {
                *screen_message = ScreenMessage::Failed;
            }
        }
    }
}

fn update_message(mut screen_messages: Query<(&mut Text, &ScreenMessage), Changed<ScreenMessage>>) {
    for (mut text, screen_message) in screen_messages.iter_mut() {
        text.sections[0].value = match *screen_message {
            ScreenMessage::Empty => "",
            ScreenMessage::Failed => "You have failed to achieve victory.\nPress Tab to open menu",
            ScreenMessage::PressFire => "Press fire to begin",
        }
        .into();
    }
}
