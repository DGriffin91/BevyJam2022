use std::time::Duration;

use bevy::ecs::event::{Events, ManualEventReader};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_egui::egui::Ui;
use bevy_kira_audio::Audio;
use bevy_polyline::{Polyline, PolylineBundle, PolylineMaterial};
use heron::rapier_plugin::convert::IntoRapier;
use heron::rapier_plugin::rapier3d::prelude::RigidBodySet;
use heron::rapier_plugin::{PhysicsWorld, RigidBodyHandle};
use heron::{
    CollisionLayers, CollisionShape, PhysicMaterial, PhysicsLayer, RigidBody, RotationConstraints,
};

use crate::assets::custom_material::slider;
use crate::assets::{AudioAssets, GameState, ModelAssets};
use crate::enemies::{Enemy, EnemySpawnTimer};
use crate::ui::hud::ScreenMessage;
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
                    .with_system(update_player_polylines)
                    .with_system(update_secondary_player_polylines)
                    .with_system(cursor_grab)
                    //.with_system(player_change_speed)
                    .with_system(footsteps),
            );
    }
}

pub enum PlayerEvent {
    Hit { laser: bool },
    Fire { alt: bool },
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
    pub forward_key: KeyCode,
    pub back_key: KeyCode,
    pub left_key: KeyCode,
    pub right_key: KeyCode,
    pub sprint_key: KeyCode,
    modify_forward: bool,
    modify_back: bool,
    modify_left: bool,
    modify_right: bool,
    modify_sprint: bool,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 3.0,
            speed: 14.0,
            lock_y: true,
            run_multiplier: 1.6,
            forward_key: KeyCode::W,
            back_key: KeyCode::S,
            left_key: KeyCode::A,
            right_key: KeyCode::D,
            sprint_key: KeyCode::LShift,
            modify_forward: false,
            modify_back: false,
            modify_left: false,
            modify_right: false,
            modify_sprint: false,
        }
    }
}

impl MovementSettings {
    pub fn build_ui(&mut self, ui: &mut Ui, keys: Res<Input<KeyCode>>) {
        // TODO refactor after jam
        slider(ui, &mut self.sensitivity, 0.1..=10.0, "Mouse Sensitivity");
        ui.horizontal(|ui| {
            if ui.button("modify").clicked() {
                self.modify_forward = true;
            }
            ui.label(&format!("{:?}: Forward", self.forward_key));
        });
        ui.horizontal(|ui| {
            if ui.button("modify").clicked() {
                self.modify_back = true;
            }
            ui.label(&format!("{:?}: Back", self.back_key));
        });
        ui.horizontal(|ui| {
            if ui.button("modify").clicked() {
                self.modify_left = true;
            }
            ui.label(&format!("{:?}: Left", self.left_key));
        });
        ui.horizontal(|ui| {
            if ui.button("modify").clicked() {
                self.modify_right = true;
            }
            ui.label(&format!("{:?}: Right", self.right_key));
        });
        ui.horizontal(|ui| {
            if ui.button("modify").clicked() {
                self.modify_sprint = true;
            }
            ui.label(&format!("{:?}: Sprint", self.sprint_key));
        });
        for key in keys.get_pressed() {
            if self.modify_forward {
                self.forward_key = *key;
            } else if self.modify_back {
                self.back_key = *key;
            } else if self.modify_left {
                self.left_key = *key;
            } else if self.modify_right {
                self.right_key = *key;
            } else if self.modify_sprint {
                self.sprint_key = *key;
            }
            self.modify_forward = false;
            self.modify_back = false;
            self.modify_left = false;
            self.modify_right = false;
            self.modify_sprint = false;
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
    physic_material: PhysicMaterial,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        PlayerBundle {
            player: Player::default(),
            footsteps: Footsteps::default(),
            transform: Transform::from_xyz(0.0, 3.0, 100.0),
            global_tranform: GlobalTransform::default(),
            rigid_body: RigidBody::Dynamic,
            collision_layers: CollisionLayers::from_bits(
                Layer::Player.to_bits(),
                Layer::all_bits(),
            )
            .with_group(Layer::Player)
            .with_masks([Layer::Bullet, Layer::Enemy, Layer::World]),
            collision_shape: CollisionShape::Capsule {
                half_segment: 1.0,
                radius: 0.5,
            },
            rotation_constraints: RotationConstraints::lock(),
            physic_material: PhysicMaterial {
                restitution: 0.1,
                density: 10.0,
                friction: 0.5,
            },
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub health: f32,
    pub max_health: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            health: 1000.0,
            max_health: 1000.0,
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

#[derive(Component)]
struct PlayerPolylineSecondary;

#[derive(Component)]
struct PlayerWeapon {
    fire_rate: f32,
    last_shot: f32,
    secondary_fire_rate: f32,
    secondary_fire_last_shot: f32,
}

#[derive(Component, Deref, DerefMut)]
pub struct PolylineTimer(Timer);

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
                        })
                        .insert(PlayerWeapon {
                            fire_rate: 0.5, //120RPM
                            last_shot: 0.0,
                            secondary_fire_rate: 0.08, //750RPM
                            secondary_fire_last_shot: 0.0,
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
                width: 30.0,
                color: Color::rgba(1.0, 0.0, 0.0, 0.9),
                perspective: true,
            }),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(PolylineTimer(Timer::new(
            Duration::from_secs_f32(2.0),
            false,
        )))
        .insert(PlayerPolyline);

    commands
        .spawn_bundle(PolylineBundle {
            polyline: polylines.add(Polyline {
                vertices: vec![Vec3::ZERO, Vec3::ZERO],
            }),
            material: polyline_materials.add(PolylineMaterial {
                width: 10.0,
                color: Color::rgba(1.0, 0.5, 0.0, 0.9),
                perspective: true,
            }),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(PolylineTimer(Timer::new(
            Duration::from_secs_f32(2.0),
            false,
        )))
        .insert(PlayerPolylineSecondary);

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
    //mut rigid_bodies: ResMut<RigidBodySet>,
    mut query: Query<(Entity, &mut Transform, &RigidBodyHandle), With<Player>>,
    mut footsteps: Query<&mut Footsteps>,
) {
    let window = windows.get_primary().unwrap();
    if window.is_focused() && window.cursor_locked() {
        for (entity, mut transform, _rb) in query.iter_mut() {
            //if let Some(body) = rigid_bodies.get_mut(rb.into_rapier()) {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let local_z_y = if settings.lock_y { 0.0 } else { local_z.y };
            let forward = -Vec3::new(local_z.x, local_z_y, local_z.z);
            let right = Vec3::new(local_z.z, 0.0, -local_z.x);
            let mut run = 1.0;
            let mut moving_forward = false;

            for key in keys.get_pressed() {
                if &settings.forward_key == key {
                    velocity += forward;
                    moving_forward = true;
                } else if &settings.back_key == key {
                    velocity -= forward
                } else if &settings.left_key == key {
                    velocity -= right
                } else if &settings.right_key == key {
                    velocity += right
                }
                //Jump? Too floaty
                //if &KeyCode::Space == key {
                //    body.apply_impulse([0.0, 100.0, 0.0].into(), false);
                //    let a = body.angvel()[1];
                //    dbg!(a);
                //}
            }
            for key in keys.get_pressed() {
                if &settings.sprint_key == key && moving_forward {
                    run = settings.run_multiplier
                }
            }

            let move_delta =
                velocity.normalize_or_zero() * time.delta_seconds() * run * settings.speed;

            //move_delta = Vec3::new(body.linvel().x, 0.0, body.linvel().z).lerp(move_delta, 0.2);

            //let lin_vel_move_delta = move_delta * 100.0;
            //body.set_linvel(
            //    [lin_vel_move_delta.x, body.linvel().y, lin_vel_move_delta.z].into(),
            //    false,
            //);
            transform.translation += move_delta;

            if let Ok(mut footsteps) = footsteps.get_component_mut::<Footsteps>(entity) {
                let abs_move_delta = move_delta.abs();
                footsteps.move_distance += abs_move_delta.x.max(abs_move_delta.z);
            }
            //}
        }
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: ParamSet<(
        Query<&mut Transform, With<Player>>,
        Query<&mut Transform, With<PlayerCam>>,
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
            pitch -= (settings.sensitivity * 0.01 * ev.delta.y).to_radians();
            yaw -= (settings.sensitivity * 0.01 * ev.delta.x).to_radians();

            pitch = pitch.clamp(-1.54, 1.54);
        }
        state.pitch = pitch;
        state.yaw = yaw;

        // Update player yaw
        for mut transform in query.p0().iter_mut() {
            transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw);
        }

        // Update camera pitch
        for mut transform in query.p1().iter_mut() {
            transform.rotation = Quat::from_axis_angle(Vec3::X, state.pitch);
        }
    }
}

fn player_fire(
    time: Res<Time>,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    physics_world: PhysicsWorld,
    state: Res<InputState>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    player_cams: Query<&GlobalTransform, With<PlayerCam>>,
    mut player_weapon: Query<(&GlobalTransform, &mut PlayerWeapon)>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polylines_query: Query<
        (
            &mut Handle<Polyline>,
            &mut Visibility,
            &Handle<PolylineMaterial>,
            &mut PolylineTimer,
        ),
        (Without<PlayerPolylineSecondary>, With<PlayerPolyline>),
    >,
    mut secondary_polylines_query: Query<
        (
            &mut Handle<Polyline>,
            &mut Visibility,
            &Handle<PolylineMaterial>,
            &mut PolylineTimer,
        ),
        (With<PlayerPolylineSecondary>, Without<PlayerPolyline>),
    >,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut player_events: EventWriter<PlayerEvent>,
    mut enemies: Query<&mut Enemy>,
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    player: Query<&Player>,
    mut screen_messages: Query<&mut ScreenMessage>,
) {
    let window = windows.get_primary().unwrap();
    if !window.is_focused() || !window.cursor_locked() {
        return;
    }
    if let Some(player) = player.iter().next() {
        if player.health <= 0.0 {
            return;
        }
        if mouse_button_input.pressed(MouseButton::Right) {
            for mut screen_message in screen_messages.iter_mut() {
                if *screen_message != ScreenMessage::Empty {
                    *screen_message = ScreenMessage::Empty;
                }
            }
            enemy_spawn_timer.0.unpause();
            for (cam_transform, (weapon_transform, mut weapon)) in
                player_cams.iter().zip(player_weapon.iter_mut())
            {
                if time.time_since_startup().as_secs_f32() - weapon.secondary_fire_last_shot
                    < weapon.secondary_fire_rate
                {
                    return;
                } else {
                    weapon.secondary_fire_last_shot = time.time_since_startup().as_secs_f32();
                };

                let pitch = state.pitch;
                let yaw = -state.yaw;
                let xz = f32::cos(pitch);
                let looking_dir =
                    -Vec3::new(-xz * f32::sin(yaw), -f32::sin(pitch), xz * f32::cos(yaw));
                player_events.send(PlayerEvent::Fire { alt: true });

                for (polyline, mut visibility, material, mut timer) in
                    secondary_polylines_query.iter_mut()
                {
                    if let Some(polyline) = polylines.get_mut(&*polyline) {
                        polyline.vertices[0] =
                            weapon_transform.translation + weapon_transform.forward() * 0.6;
                        polyline.vertices[1] =
                            weapon_transform.translation + weapon_transform.forward() * 100.0;
                    }
                    if let Some(material) = polyline_materials.get_mut(material) {
                        material.color.set_a(1.0);
                    }
                    visibility.is_visible = true;
                    timer.reset();
                }

                if let Some(collision) = physics_world.ray_cast_with_filter(
                    cam_transform.translation,
                    looking_dir * 100.0,
                    true,
                    CollisionLayers::none()
                        .with_group(Layer::Raycast)
                        .with_masks([Layer::World, Layer::Enemy]),
                    |_| true,
                ) {
                    // TODO move to be triggered by event
                    if let Ok(mut enemy) = enemies.get_mut(collision.entity) {
                        enemy.health -= 334;
                    }
                }
            }
        } else if mouse_button_input.just_pressed(MouseButton::Left) {
            for mut screen_message in screen_messages.iter_mut() {
                if *screen_message != ScreenMessage::Empty {
                    *screen_message = ScreenMessage::Empty;
                }
            }
            enemy_spawn_timer.0.unpause();
            for (cam_transform, (weapon_transform, mut weapon)) in
                player_cams.iter().zip(player_weapon.iter_mut())
            {
                if time.time_since_startup().as_secs_f32() - weapon.last_shot < weapon.fire_rate {
                    return;
                } else {
                    weapon.last_shot = time.time_since_startup().as_secs_f32();
                };

                let pitch = state.pitch;
                let yaw = -state.yaw;
                let xz = f32::cos(pitch);
                let looking_dir =
                    -Vec3::new(-xz * f32::sin(yaw), -f32::sin(pitch), xz * f32::cos(yaw));
                player_events.send(PlayerEvent::Fire { alt: false });

                for (polyline, mut visibility, material, mut timer) in polylines_query.iter_mut() {
                    if let Some(polyline) = polylines.get_mut(&*polyline) {
                        polyline.vertices[0] =
                            weapon_transform.translation + weapon_transform.forward() * 0.6;
                        polyline.vertices[1] =
                            weapon_transform.translation + weapon_transform.forward() * 100.0;
                    }
                    if let Some(material) = polyline_materials.get_mut(material) {
                        material.color.set_a(1.0);
                    }
                    visibility.is_visible = true;
                    timer.reset();
                }

                if let Some(collision) = physics_world.ray_cast_with_filter(
                    cam_transform.translation,
                    looking_dir * 200.0,
                    true,
                    CollisionLayers::none()
                        .with_group(Layer::Raycast)
                        .with_masks([Layer::World, Layer::Enemy]),
                    |_| true,
                ) {
                    // TODO move to be triggered by event
                    if let Ok(mut enemy) = enemies.get_mut(collision.entity) {
                        enemy.health -= 1001;
                    }
                }
            }
        }
    }
}

fn update_player_polylines(
    time: Res<Time>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polylines_query: Query<
        (
            &mut Handle<Polyline>,
            &mut Visibility,
            &Handle<PolylineMaterial>,
            &mut PolylineTimer,
        ),
        With<PlayerPolyline>,
    >,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    for (polyline, mut visibility, material, mut timer) in polylines_query.iter_mut() {
        timer.tick(time.delta());
        let duration = timer.duration().as_secs_f32();
        let elapsed = timer.elapsed().as_secs_f32();
        let progress = (duration - elapsed) / duration;
        if let Some(material) = polyline_materials.get_mut(material) {
            if timer.just_finished() {
                material.color.set_a(1.0);
            } else {
                // fade out laser over time
                material.color.set_a(progress);
            }
        }
        if timer.just_finished() {
            visibility.is_visible = false;
        }
        if visibility.is_visible {
            if let Some(polyline) = polylines.get_mut(&*polyline) {
                let norm = (polyline.vertices[1] - polyline.vertices[0]).normalize();
                //move start of laser out along normal over time
                polyline.vertices[0] += norm * 40.0 * (1.0 - progress);
            }
        }
    }
}

//TODO Refactor
fn update_secondary_player_polylines(
    time: Res<Time>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polylines_query: Query<
        (
            &mut Handle<Polyline>,
            &mut Visibility,
            &Handle<PolylineMaterial>,
            &mut PolylineTimer,
        ),
        With<PlayerPolylineSecondary>,
    >,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    for (polyline, mut visibility, material, mut timer) in polylines_query.iter_mut() {
        timer.tick(time.delta());
        let duration = timer.duration().as_secs_f32();
        let elapsed = timer.elapsed().as_secs_f32();
        let progress = (duration - elapsed) / duration;
        if let Some(material) = polyline_materials.get_mut(material) {
            if timer.just_finished() {
                material.color.set_a(1.0);
            } else {
                // fade out laser over time
                material.color.set_a(progress);
            }
        }
        if timer.just_finished() {
            visibility.is_visible = false;
        }
        if visibility.is_visible {
            if let Some(polyline) = polylines.get_mut(&*polyline) {
                let norm = (polyline.vertices[1] - polyline.vertices[0]).normalize();
                //move start of laser out along normal over time
                polyline.vertices[0] += norm * 40.0 * (1.0 - progress);
            }
        }
    }
}

#[allow(dead_code)]
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
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Tab) {
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
            audio.play(audio_assets.get_step().clone());
        }
    }
}
