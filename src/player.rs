use bevy::app::{Events, ManualEventReader};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_event::<FireEvent>()
            .add_startup_system(setup_player)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(player_fire)
            .add_system(cursor_grab)
            .add_system(player_change_speed);
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
            speed: 18.0,       // default: 12.0
            lock_y: true,
            run_multiplier: 1.8,
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerCam;

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands) {
    commands
        .spawn_bundle((Transform::default(), GlobalTransform::default()))
        .insert(Player)
        .with_children(|parent| {
            parent
                .spawn_bundle(PerspectiveCameraBundle {
                    transform: Transform::from_xyz(-20.0, 3.0, 0.0)
                        .looking_at(Vec3::ZERO + Vec3::new(0.0, 0.0, 5.0), Vec3::Y),
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
    mut query: Query<&mut Transform, With<Player>>,
) {
    let window = windows.get_primary().unwrap();
    if window.is_focused() && window.cursor_locked() {
        for mut transform in query.iter_mut() {
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

            transform.translation += velocity * time.delta_seconds() * run * settings.speed;
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

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<PlayerCam>>,
) {
    let window = windows.get_primary().unwrap();
    let mut pitch = state.pitch;
    let mut yaw = state.yaw;
    for mut transform in query.iter_mut() {
        for ev in state.reader_motion.iter(&motion) {
            if window.is_focused() && window.cursor_locked() {
                // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                // let window_scale = window.height().min(window.width());

                pitch -= (settings.sensitivity * ev.delta.y).to_radians(); //* window_scale
                yaw -= (settings.sensitivity * ev.delta.x).to_radians(); //* window_scale
            }

            pitch = pitch.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }
    state.pitch = pitch;
    state.yaw = yaw;
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if keys.just_pressed(KeyCode::Escape) {
        let window = windows.get_primary_mut().unwrap();
        toggle_grab_cursor(window);
    }
}
