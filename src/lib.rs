use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{WindowMode, WindowResizeConstraints},
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMesh, RayCastSource};
use bevy_polyline::{Polyline, PolylineBundle, PolylineMaterial, PolylinePlugin};
use draw_debug::clean_up_debug_lines;
use heron::PhysicsPlugin;
use planets::{planitary_physics, spawn_planets};
use player::{FireEvent, FlyCam, MovementSettings, PlayerPlugin};
use rand::Rng;
use ui::scoreboard::ScoreboardEvent;

pub mod assets;
pub mod draw_debug;
pub mod planets;
pub mod player;
pub mod ui;
pub mod world;

#[derive(Component, Debug)]
struct Target {
    velocity: Vec3,
    do_move: bool,
    use_gravity: bool,
    health: f32,
    bounce: bool,
}

#[derive(Component, Debug)]

struct PlayerBeam;

fn setup_player_entities(
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    let material = polyline_materials.add(PolylineMaterial {
        width: 4.0,
        color: Color::RED,
        perspective: true,
    });
    let polyline = polylines.add(Polyline {
        vertices: vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)],
    });
    commands
        .spawn_bundle(PolylineBundle {
            polyline,
            material,
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(PlayerBeam);
}

// Mark our generic `RayCastMesh`s and `RayCastSource`s as part of the same group, or "RayCastSet".
struct MyRaycastSet;

fn move_targets(
    mut commands: Commands,
    time: Res<Time>,
    mut target_query: Query<(Entity, &mut Target, &mut Transform)>,
) {
    for (entity, mut target, mut transform) in target_query.iter_mut() {
        transform.rotate(Quat::from_rotation_x(
            target.velocity.x * time.delta_seconds(),
        ));
        transform.rotate(Quat::from_rotation_z(
            target.velocity.z * time.delta_seconds(),
        ));

        if target.do_move {
            let hit = transform.translation.x < -10.0
                || transform.translation.x > 10.0
                || transform.translation.y < 3.2
                || transform.translation.y > 15.0
                || transform.translation.z < -10.0
                || transform.translation.z > 10.0;
            if hit {
                if target.bounce {
                    target.velocity *= -1.0;
                } else {
                    commands.entity(entity).despawn()
                }
            }
            transform.translation += target.velocity * time.delta_seconds();
            if target.use_gravity {
                target.velocity.y -= 9.807 * time.delta_seconds()
            }
        }
    }
}

fn fire_event(
    time: Res<Time>,
    mut commands: Commands,
    mut game_setup: ResMut<GameSetup>,
    mut ev_fire: EventReader<FireEvent>,
    mut scoreboard: EventWriter<ScoreboardEvent>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    //mut target_query: Query<(&Target), With<RayCastSource<MyRaycastSet>>>,
    raycast_query: Query<&mut RayCastSource<MyRaycastSet>>,
    //meshes: ResMut<Assets<Mesh>>,
    mut target_query: Query<(&mut Target, &mut Transform, Without<FlyCam>)>,
    mut playerbeam_query: Query<(
        &mut PlayerBeam,
        &mut Visibility,
        &mut Handle<Polyline>,
        &mut Handle<PolylineMaterial>,
    )>,
    flycam_query: Query<(&mut Transform, With<FlyCam>)>,
) {
    for (transform, _) in flycam_query.iter() {
        let t = transform;
        if let Some((_, _, polyline, _)) = playerbeam_query.iter().last() {
            let polyline = polylines.get_mut(polyline).unwrap();
            polyline.vertices[0] = t.translation + t.right() + t.down() + t.forward();
            polyline.vertices[1] = t.translation + t.forward() * 10.0;
        }

        //TODO should these really be loops?
        let mut fired_this_time = false;
        for e in ev_fire.iter() {
            if e.release {
                if let Some((_, mut visibility, _, _)) = playerbeam_query.iter_mut().last() {
                    visibility.is_visible = false;
                }
                game_setup.is_firing = false;
                continue;
            } else if (time.time_since_startup().as_secs_f64() - game_setup.last_shot
                < game_setup.fire_rate.into())
                && !game_setup.trace_mode
            {
                game_setup.is_firing = false;
            } else {
                game_setup.is_firing = true;
                fired_this_time = true;
                game_setup.last_shot = time.time_since_startup().as_secs_f64();
            }
        }
        if game_setup.is_firing && game_setup.trace_mode || fired_this_time {
            let mut hit = false;
            for raycast in raycast_query.iter() {
                if let Some((entity, _intersections)) = raycast.intersect_top() {
                    if let Ok((mut target, _transform, _)) = target_query.get_mut(entity) {
                        hit = true;
                        if game_setup.trace_mode {
                            target.health -= game_setup.gun_damage * time.delta_seconds() * 10.0;
                            if target.health <= 0.00001 {
                                commands.entity(entity).despawn();
                            }
                        } else {
                            target.health -= game_setup.gun_damage;
                            if target.health <= 0.00001 {
                                commands.entity(entity).despawn();
                            }
                        }
                    }
                    if let Some((_, mut visibility, _, _)) = playerbeam_query.iter_mut().last() {
                        visibility.is_visible = true;
                    }
                    if let Some((_, _, _, material)) = playerbeam_query.iter().last() {
                        let mut material = polyline_materials.get_mut(material).unwrap();
                        //let polyline = polylines.get_mut(polyline).unwrap();
                        //polyline.vertices[0] = t.translation + t.right() + t.down() + t.forward();
                        //polyline.vertices[1] = t.translation + t.forward() * 10.0;
                        if hit {
                            material.color = Color::RED;
                        } else {
                            material.color = Color::rgba(1.0, 1.0, 1.0, 0.02)
                        }
                    }
                }
            }

            if hit {
                scoreboard.send(ScoreboardEvent::Hit);
            } else {
                scoreboard.send(ScoreboardEvent::Miss);
            }
        }
    }
}

fn spawn_targets(
    mut commands: Commands,
    game_setup: Res<GameSetup>,
    target_query: Query<(&Target, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    let targets_short = game_setup.min_targets as i64 - target_query.iter().len() as i64;
    let targets_short = targets_short.max(0);
    if targets_short > 0 {
        let h = game_setup.target_spawn_height / 2.0;
        let d = game_setup.target_spawn_depth / 2.0;
        let w = game_setup.target_spawn_width / 2.0;
        for _ in 0..targets_short {
            let (x, y, z) = if game_setup.target_jump {
                (
                    rng.gen_range(-d..d),
                    rng.gen_range(-h..0.0),
                    rng.gen_range(-w..w),
                )
            } else {
                (
                    rng.gen_range(-d..d),
                    rng.gen_range(-h..h),
                    rng.gen_range(-w..w),
                )
            };
            let m = game_setup.target_move_speed;
            let mut do_move = false;
            let (mx, my, mz) = if m > 0.0 {
                do_move = true;
                if game_setup.target_jump {
                    (
                        rng.gen_range(-m..m),
                        rng.gen_range(m * 5.0..m * 10.0),
                        rng.gen_range(-m..m),
                    )
                } else {
                    (
                        rng.gen_range(-m..m),
                        rng.gen_range(-m..m),
                        rng.gen_range(-m..m),
                    )
                }
            } else {
                (0.0, 0.0, 0.0)
            };
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube {
                        //radius: 0.45,
                        //subdivisions: 32,
                        size: game_setup.target_size,
                    })),
                    material: materials.add(StandardMaterial {
                        //base_color: Color::hex("ffd891").unwrap(),
                        base_color: Color::rgb(
                            game_setup.target_color[0],
                            game_setup.target_color[1],
                            game_setup.target_color[2],
                        ),
                        //metallic: 0.0,
                        perceptual_roughness: 0.5,
                        ..Default::default()
                    }),
                    transform: Transform::from_xyz(x, y + 7.0, z),
                    ..Default::default()
                })
                .insert(Target {
                    velocity: Vec3::new(mx, my, mz),
                    do_move,
                    health: 1.0,
                    use_gravity: game_setup.target_jump,
                    bounce: game_setup.target_bounce,
                })
                .insert(RayCastMesh::<MyRaycastSet>::default());
        }
    }
}

pub struct GameSetup {
    pub min_targets: u32,
    pub target_size: f32,
    pub target_move_speed: f32,
    pub target_spawn_height: f32,
    pub target_spawn_depth: f32,
    pub target_spawn_width: f32,
    pub gun_damage: f32,
    pub target_jump: bool,
    pub target_color: [f32; 3],
    pub target_bounce: bool,
    pub trace_mode: bool,
    pub fire_rate: f32,
    //make player state component in entity?
    pub last_shot: f64,
    pub is_firing: bool,
}
