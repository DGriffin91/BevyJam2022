use bevy::app::{Events, ManualEventReader};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioSource};
use rand::prelude::SliceRandom;

use crate::assets::{AudioAssets, GameState};

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_event::<FireEvent>()
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup_player))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(player_move)
                    .with_system(player_look)
                    .with_system(player_fire)
                    .with_system(cursor_grab)
                    .with_system(player_change_speed)
                    .with_system(footsteps),
            );
    }
}

#[derive(Debug)]
pub struct FireEvent {
    pub transform: Transform,
    pub release: bool,
}

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
    pub run_multiplier: f32,
    pub lock_y: bool,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.03, // default: 0.00012
            speed: 10.0,       // default: 12.0
            lock_y: true,
            run_multiplier: 1.5,
        }
    }
}

#[derive(Component, Default)]
struct Player;

#[derive(Component, Default)]
struct Footsteps {
    last_footstep_position: Vec3,
    move_distance: f32,
}

#[derive(Component, Default)]
struct PlayerCam;

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands) {
    commands
        .spawn_bundle((
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ))
        .insert(Player)
        .with_children(|parent| {
            parent
                .spawn_bundle(PerspectiveCameraBundle {
                    transform: Transform::from_xyz(0.0, 3.0, 0.0),
                    perspective_projection: PerspectiveProjection {
                        fov: (80.0f32).to_radians(),
                        aspect_ratio: 1.0,
                        near: 0.1,
                        far: 1000.0,
                    },
                    ..Default::default()
                })
                .insert(PlayerCam);
        });

    commands.spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(4.0), Val::Px(4.0)),
            // center button
            margin: Rect::all(Val::Auto),
            ..Default::default()
        },
        color: Color::rgb(1.0, 1.0, 1.0).into(),
        ..Default::default()
    });
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<(Entity, &mut Transform), With<Player>>,
    mut footsteps: Query<&mut Footsteps>,
) {
    let window = windows.get_primary().unwrap();
    if window.is_focused() && window.cursor_locked() {
        for (entity, mut transform) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let local_z_y = if settings.lock_y { 0.0 } else { local_z.y };
            let forward = -Vec3::new(local_z.x, local_z_y, local_z.z);
            let right = Vec3::new(local_z.z, 0.0, -local_z.x);
            let mut run = 1.0;

            let mut moving_forward = false;
            for key in keys.get_pressed() {
                match key {
                    KeyCode::W => {
                        velocity += forward;
                        moving_forward = true;
                    }
                    KeyCode::S => velocity -= forward,
                    KeyCode::A => velocity -= right,
                    KeyCode::D => velocity += right,
                    _ => (),
                }
            }

            for key in keys.get_pressed() {
                if key == &KeyCode::LShift && moving_forward {
                    run = settings.run_multiplier
                }
            }

            let move_delta = velocity * time.delta_seconds() * run * settings.speed;
            transform.translation += move_delta;
            if let Ok(mut footsteps) = footsteps.get_component_mut::<Footsteps>(entity) {
                let abs_move_delta = move_delta.abs();
                footsteps.move_distance += abs_move_delta.x.max(abs_move_delta.z);
            }
        }
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: QuerySet<(
        QueryState<&mut Transform, With<Player>>,
        QueryState<&mut Transform, With<PlayerCam>>,
    )>,
) {
    let window = windows.get_primary().unwrap();
    if window.is_focused() && window.cursor_locked() {
        // Update InputState
        // Not sure what changed in updating to a newer version of bevy that required this
        // had error[E0499]: cannot borrow `state` as mutable more than once at a time
        let mut pitch = state.pitch;
        let mut yaw = state.yaw;
        for ev in state.reader_motion.iter(&motion) {
            pitch -= (settings.sensitivity * ev.delta.y).to_radians();
            yaw -= (settings.sensitivity * ev.delta.x).to_radians();

            pitch = pitch.clamp(-1.54, 1.54);
        }
        state.pitch = pitch;
        state.yaw = yaw;

        // Update player yaw
        for mut transform in query.q0().iter_mut() {
            transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw);
        }

        // Update camera pitch
        for mut transform in query.q1().iter_mut() {
            transform.rotation = Quat::from_axis_angle(Vec3::X, state.pitch);
        }
    }
}

fn player_fire(
    windows: Res<Windows>,
    mut ev_fire: EventWriter<FireEvent>,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<&Transform, With<Player>>,
) {
    let window = windows.get_primary().unwrap();
    if window.is_focused() && window.cursor_locked() {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            for transform in query.iter() {
                ev_fire.send(FireEvent {
                    transform: *transform,
                    release: false,
                })
            }
        }
        if mouse_button_input.just_released(MouseButton::Left) {
            for transform in query.iter() {
                ev_fire.send(FireEvent {
                    transform: *transform,
                    release: true,
                })
            }
        }
    }
}

fn player_change_speed(
    mut settings: ResMut<MovementSettings>,
    windows: Res<Windows>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    let window = windows.get_primary().unwrap();
    if window.is_focused() && window.cursor_locked() {
        for ev in mouse_wheel.iter() {
            settings.speed = (ev.y + settings.speed).max(0.0);
        }
    }
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if keys.just_pressed(KeyCode::Escape) {
        let window = windows.get_primary_mut().unwrap();
        toggle_grab_cursor(window);
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

fn footsteps(
    mut footsteps: Query<&mut Footsteps>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    for mut footsteps in footsteps.iter_mut() {
        if footsteps.move_distance > 5.0 {
            footsteps.move_distance = 0.0;
            let footstep_audio = audio_assets
                .footsteps
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone()
                .typed::<AudioSource>();
            audio.play(footstep_audio);
        }
    }
}
