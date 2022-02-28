use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{Animator, EaseFunction, Lens, Tween, TweeningType};

use crate::{assets::GameState, player::Player};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup_health_bar))
            .add_system_set(
                SystemSet::on_update(GameState::Playing).with_system(update_health_bar), // .with_system(update_health_bar_style),
            );
    }
}

#[derive(Component)]
struct HealthBar {
    percent: f32,
}

struct HealthBarLens {
    start: f32,
    end: f32,
}

// impl Lens<HealthBar> for HealthBarLens {
//     fn lerp(&mut self, target: &mut HealthBar, ratio: f32) {
//         target.percent = self.start + (self.end - self.start) * ratio;
//     }
// }

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
            color: Color::WHITE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    color: Color::RED.into(),
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
                .insert(HealthBar { percent: 100.0 });
        });
}

fn update_health_bar(
    mut commands: Commands,
    players: Query<&Player, Changed<Player>>,
    mut health_bars: Query<(&mut Animator<Style>, &Style), With<HealthBar>>,
) {
    for Player { health, max_health } in players.iter() {
        for (mut animator, style) in health_bars.iter_mut() {
            if let Val::Percent(width) = style.size.width {
                let health_percent = *health as f32 / *max_health as f32;

                // let mut tweenable = animator.tweenable_mut().unwrap();
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

            // println!("Insert");

            // let pp = Animator::new(Tween::new(
            //     EaseFunction::QuadraticOut,
            //     TweeningType::Once,
            //     Duration::from_millis(500),
            //     HealthBarLens {
            //         start: health_bar.percent,
            //         end: health_percent * 100.0,
            //     },
            // ));

            // commands.entity(entity).insert(Animator::new(Tween::new(
            //     EaseFunction::QuadraticOut,
            //     TweeningType::Once,
            //     Duration::from_millis(500),
            //     HealthBarLens {
            //         start: health_bar.percent,
            //         end: health_percent * 100.0,
            //     },
            // )));
            // style.size.width = Val::Percent(health_percent * 100.0);
        }
    }
}

// fn update_health_bar_style(mut health_bars: Query<(&HealthBar, &mut Style), Changed<HealthBar>>) {
//     for (HealthBar { percent }, mut style) in health_bars.iter_mut() {
//         println!("{}", percent);
//         style.size.width = Val::Percent(*percent);
//     }
// }
