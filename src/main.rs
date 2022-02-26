use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{WindowMode, WindowResizeConstraints},
};
mod custom_material;
mod draw_debug;
mod emissive_material;
mod level1;
mod material_util;
mod planets;
mod player_plugin;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_mod_raycast::{DefaultPluginState, DefaultRaycastingPlugin, RayCastMesh, RayCastSource};
use bevy_polyline::{Polyline, PolylineBundle, PolylineMaterial, PolylinePlugin};
use custom_material::{CustomMaterial, MaterialProperties, MaterialSetProp, MaterialTexture};
use draw_debug::clean_up_debug_lines;
use emissive_material::EmissiveMaterial;
use heron::PhysicsPlugin;
use planets::{planitary_physics, spawn_planets};
use player_plugin::{FireEvent, FlyCam, MovementSettings, PlayerPlugin};

use rand::Rng;

#[derive(Component, Debug)]
struct Target {
    velocity: Vec3,
    do_move: bool,
    use_gravity: bool,
    health: f32,
    bounce: bool,
}

#[derive(Component, Debug)]
struct LevelAsset {
    pub material_properties: MaterialProperties,
    pub material_handle: Handle<CustomMaterial>,
}

#[derive(Component, Debug)]

struct PlayerBeam();

fn setup_player_entities(
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    let material = polyline_materials.add(PolylineMaterial {
        width: 4.0,
        color: Color::RED,
        perspective: true,
        ..Default::default()
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
        .insert(PlayerBeam());
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    // scoreboard
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: font.clone(),
                        font_size: 32.0,
                        color: Color::rgb(0.6, 0.6, 0.9),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: font.clone(),
                        font_size: 32.0,
                        color: Color::rgb(0.9, 0.6, 0.6),
                    },
                },
            ],
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
    });
}

pub struct Scoreboard {
    pub hits: usize,
    pub misses: usize,
    pub start_time: f64,
    pub last_hit_time: f64,
}

fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    let hits = scoreboard.hits as f64;
    let misses = scoreboard.misses as f64;
    if scoreboard.hits > 0 || scoreboard.misses > 0 {
        text.sections[0].value = format!("{:.1}% | ", (hits / (hits + misses)) * 100.0);
        if scoreboard.last_hit_time != -1.0 {
            text.sections[1].value = format!(
                "{:.2} ttk",
                (scoreboard.last_hit_time - scoreboard.start_time) / hits
            );
        }
    } else {
        text.sections[0].value = format!("Fire To Start");
        text.sections[1].value = format!("");
    }
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
    mut scoreboard: ResMut<Scoreboard>,
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
            } else {
                if (time.time_since_startup().as_secs_f64() - game_setup.last_shot
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
                scoreboard.hits += 1;
            } else {
                scoreboard.misses += 1;
            }
            if scoreboard.start_time == -1.0 {
                scoreboard.start_time = time.seconds_since_startup()
            }
            scoreboard.last_hit_time = time.seconds_since_startup()
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

fn menu_ui(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    mut game_setup: ResMut<GameSetup>,
    target_query: Query<(Entity, &Target)>,
    mut movement_settings: ResMut<MovementSettings>,
    mut scoreboard: ResMut<Scoreboard>,
    mut egui_context: ResMut<EguiContext>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut level_asset_query: Query<&mut LevelAsset>,
    asset_server: Res<AssetServer>,
) {
    let window = windows.get_primary_mut().unwrap();
    if window.is_focused() && !window.cursor_locked() {
        egui::Window::new("materials").show(egui_context.ctx_mut(), |ui| {
            if let Some(mut main) = level_asset_query.iter_mut().next() {
                let mat_props = {
                    ui.collapsing("material properties", |ui| {
                        main.material_properties.build_ui(ui);
                    });
                    main.material_properties.clone()
                };
                for mat in level_asset_query.iter_mut() {
                    if let Some(mat) = custom_materials.get_mut(&mat.material_handle) {
                        mat.material_properties = mat_props;
                        //ui.collapsing("main material", |ui| {
                        //    mat.build_ui(ui, &asset_server);
                        //});
                    }
                }
            }
        });
        egui::Window::new("Setup")
            .current_pos((10.0, 60.0))
            .show(egui_context.ctx_mut(), |ui| {
                if ui.button("Start").clicked() {
                    window.set_cursor_lock_mode(true);
                    window.set_cursor_visibility(false);
                    scoreboard.misses = 0;
                    scoreboard.hits = 0;
                    for (entity, _target) in target_query.iter() {
                        commands.entity(entity).despawn()
                    }
                }
                ui.label("Game Settings");
                ui.add(
                    egui::Slider::new(&mut game_setup.gun_damage, 0.05..=1.0).text("Gun Damage"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.min_targets, 0..=10).text("Minimum Targets"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.target_size, 0.1..=2.0).text("Target Size"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.target_move_speed, 0.0..=5.0)
                        .text("Target Move Speed"),
                );
                ui.add(egui::Checkbox::new(
                    &mut game_setup.target_jump,
                    "Targets Jump",
                ));
                ui.add(egui::Checkbox::new(
                    &mut game_setup.target_bounce,
                    "Targets Bounce",
                ));

                ui.label("Cube Color");
                ui.color_edit_button_rgb(&mut game_setup.target_color);
                ui.label("Target Spawn Area");
                ui.add(
                    egui::Slider::new(&mut game_setup.target_spawn_height, 0.1..=7.0)
                        .text("Height"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.target_spawn_depth, 0.1..=7.0).text("Depth"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.target_spawn_width, 0.1..=7.0).text("Width"),
                );
                ui.label("Fire Settings");
                ui.label("Fire Rate (minimum time between shots)");
                ui.add(egui::Slider::new(&mut game_setup.fire_rate, 0.0..=1.0).text(""));
                ui.add(egui::Checkbox::new(
                    &mut game_setup.trace_mode,
                    "Trace Mode",
                ));
                ui.label("Input Settings");
                ui.add(
                    egui::Slider::new(&mut movement_settings.sensitivity, 0.001..=0.1)
                        .text("Mouse Sensitivity"),
                );
                ui.add(
                    egui::Slider::new(&mut movement_settings.speed, 0.1..=100.0).text("Move Speed"),
                );
            });
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "app".to_string(),
            width: 1280.,
            height: 720.,
            position: None,
            resize_constraints: WindowResizeConstraints {
                min_width: 256.0,
                min_height: 256.0,
                max_width: f32::INFINITY,
                max_height: f32::INFINITY,
            },
            scale_factor_override: Some(1.),
            //present_mode: PresentMode::Immediate,
            vsync: false,
            resizable: true,
            decorations: true,
            cursor_locked: false,
            cursor_visible: true,
            mode: WindowMode::Windowed,
            transparent: false,
            #[cfg(target_arch = "wasm32")]
            canvas: Some(String::from("#can")),
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(PolylinePlugin)
        .add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
        .insert_resource(MovementSettings {
            sensitivity: 0.03, // default: 0.00012
            speed: 18.0,       // default: 12.0
            lock_y: true,
            run_multiplier: 1.8,
        })
        .add_startup_system(level1::setup)
        .add_startup_system(setup_ui)
        .add_startup_system(setup_player_entities)
        .add_startup_system(spawn_planets)
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_plugin(MaterialPlugin::<EmissiveMaterial>::default())
        .add_system(move_targets)
        .add_event::<FireEvent>()
        .add_system(fire_event)
        .add_system(spawn_targets)
        .add_system(clean_up_debug_lines)
        .add_system(menu_ui)
        .add_plugin(PhysicsPlugin::default())
        .add_system(planitary_physics)
        .insert_resource(Scoreboard {
            hits: 0,
            misses: 0,
            start_time: -1.0,
            last_hit_time: -1.0,
        })
        .insert_resource(GameSetup {
            gun_damage: 1.0,
            min_targets: 2,
            target_size: 1.0,
            target_move_speed: 0.0,
            target_spawn_height: 6.0,
            target_spawn_depth: 6.0,
            target_spawn_width: 6.0,
            target_jump: false,
            target_color: [1.0, 0.847, 0.569],
            target_bounce: true,
            trace_mode: false,
            fire_rate: 0.0,
            last_shot: 0.0,
            is_firing: false,
        })
        .add_system(scoreboard_system)
        .run();
}
