use std::time::Duration;

use bevy::app::{Events, ManualEventReader};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioSource};
use bevy_polyline::{Polyline, PolylineBundle, PolylineMaterial};
use heron::rapier_plugin::convert::IntoRapier;
use heron::rapier_plugin::rapier3d::prelude::RigidBodySet;
use heron::rapier_plugin::{PhysicsWorld, RigidBodyHandle};
use heron::{CollisionLayers, CollisionShape, PhysicMaterial, RigidBody, RotationConstraints};
use rand::prelude::SliceRandom;

use crate::assets::{AudioAssets, GameState, ModelAssets};
use crate::Layer;

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_event::<PlayerEvent>()
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup_player))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(enable_ccd)
                    .with_system(player_move)
                    .with_system(player_look)
                    .with_system(player_fire)
                    .with_system(hide_player_polylines)
                    .with_system(cursor_grab)
                    .with_system(player_change_speed)
                    .with_system(footsteps),
            );
    }
}

pub enum PlayerEvent {
    Hit,
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
            sensitivity: 0.03,
            speed: 10.0,
            lock_y: true,
            run_multiplier: 1.5,
        }
    }
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    footsteps: Footsteps,
    transform: Transform,
    global_tranform: GlobalTransform,
    rigid_body: RigidBody,
    collision_layers: CollisionLayers,
    collision_shape: CollisionShape,
    rotation_constraints: RotationConstraints,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        PlayerBundle {
            player: Player::default(),
            footsteps: Footsteps::default(),
            transform: Transform::from_xyz(0.0, 3.0, 0.0),
            global_tranform: GlobalTransform::default(),
            rigid_body: RigidBody::Dynamic,
            collision_layers: CollisionLayers::none()
                .with_group(Layer::Player)
                .with_masks([Layer::Bullet, Layer::Enemy, Layer::World]),
            collision_shape: CollisionShape::Capsule {
                half_segment: 1.0,
                radius: 0.5,
            },
            rotation_constraints: RotationConstraints::restrict_to_y_only(),
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub health: u32,
    pub max_health: u32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            health: 1000,
            max_health: 1000,
        }
    }
}

#[derive(Component, Default)]
struct Footsteps {
    move_distance: f32,
}

#[derive(Component, Default)]
pub struct PlayerCam;

#[derive(Component)]
struct PlayerPolyline;

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(
    mut commands: Commands,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    model_assets: Res<ModelAssets>,
) {
    let lasergun = model_assets.lasergun.clone();
    commands
        .spawn_bundle(PlayerBundle::default())
        .with_children(|parent| {
            parent
                .spawn_bundle(PerspectiveCameraBundle {
                    transform: Transform::from_xyz(0.0, 1.82, 0.0),
                    perspective_projection: PerspectiveProjection {
                        fov: (75.0f32).to_radians(),
                        aspect_ratio: 1.0,
                        near: 0.1,
                        far: 1000.0,
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle((
                            Transform::from_xyz(0.28, -0.14, -0.12),
                            GlobalTransform::identity(),
                        ))
                        .with_children(|parent| {
                            parent.spawn_scene(lasergun.clone());
                        });
                })
                .insert(PlayerCam);
        });

    commands
        .spawn_bundle(PolylineBundle {
            polyline: polylines.add(Polyline {
                vertices: vec![Vec3::ZERO, Vec3::ZERO],
            }),
            material: polyline_materials.add(PolylineMaterial {
                width: 1.0,
                color: Color::RED,
                perspective: true,
            }),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Timer::new(Duration::from_secs(3), false))
        .insert(PlayerPolyline);

    commands.spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(4.0), Val::Px(4.0)),
            margin: Rect::all(Val::Auto),
            ..Default::default()
        },
        color: Color::rgb(1.0, 1.0, 1.0).into(),
        ..Default::default()
    });
}

fn enable_ccd(
    mut rigid_bodies: ResMut<RigidBodySet>,
    new_handles: Query<&RigidBodyHandle, (With<Player>, Added<RigidBodyHandle>)>,
) {
    for handle in new_handles.iter() {
        if let Some(body) = rigid_bodies.get_mut(handle.into_rapier()) {
            body.enable_ccd(true);
        }
    }
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
    mut commands: Commands,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    physics_world: PhysicsWorld,
    state: Res<InputState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_cams: Query<&GlobalTransform, With<PlayerCam>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polylines_query: Query<
        (&mut Handle<Polyline>, &mut Visibility, &mut Timer),
        With<PlayerPolyline>,
    >,
) {
    let window = windows.get_primary().unwrap();
    if !window.is_focused() || !window.cursor_locked() {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        for cam_translation in player_cams.iter() {
            let pitch = state.pitch;
            let yaw = -state.yaw;
            let xz = f32::cos(pitch);
            let looking_dir = -Vec3::new(-xz * f32::sin(yaw), -f32::sin(pitch), xz * f32::cos(yaw));

            if let Some(collision) = physics_world.ray_cast_with_filter(
                cam_translation.translation,
                looking_dir * 200.0,
                true,
                CollisionLayers::none()
                    .with_group(Layer::Raycast)
                    .with_masks([Layer::World, Layer::Enemy]),
                |_| true,
            ) {
                for (polyline, mut visibility, mut timer) in polylines_query.iter_mut() {
                    if let Some(polyline) = polylines.get_mut(&*polyline) {
                        polyline.vertices[0] = cam_translation.translation;
                        polyline.vertices[1] = collision.collision_point;
                    }

                    visibility.is_visible = true;
                    timer.reset();
                }

                let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
                let material = materials.add(StandardMaterial {
                    base_color: Color::PINK,
                    ..Default::default()
                });

                commands
                    .spawn_bundle(PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform: Transform::from_translation(collision.collision_point),
                        ..Default::default()
                    })
                    .insert(RigidBody::Dynamic)
                    .insert(CollisionShape::Cuboid {
                        half_extends: Vec3::new(0.5, 0.5, 0.5),
                        border_radius: None,
                    })
                    .insert(PhysicMaterial {
                        restitution: 0.7,
                        ..Default::default()
                    });
            }
        }
    }
}

fn hide_player_polylines(
    time: Res<Time>,
    mut polylines_query: Query<(&mut Visibility, &mut Timer), With<PlayerPolyline>>,
) {
    for (mut visibility, mut timer) in polylines_query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            visibility.is_visible = false;
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
    if !window.is_focused() || !window.cursor_locked() {
        window.set_cursor_position(Vec2::new(window.width() / 4.0, window.height() / 4.0));
    }

    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

fn footsteps(
    mut footsteps: Query<&mut Footsteps>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    for mut footsteps in footsteps.iter_mut() {
        if footsteps.move_distance > 4.0 {
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
