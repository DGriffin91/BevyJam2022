use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use bevy::prelude::*;
use heron::rapier_plugin::{convert::IntoRapier, rapier3d::prelude::RigidBodySet, RigidBodyHandle};
use pathfinding::directed::astar::astar;
use rand::{prelude::SliceRandom, Rng};
use splines::{Interpolation, Spline};

use crate::{
    assets::{
        orb_material::{OrbMaterial, OrbProperties},
        AudioAssets, GameState, ModelAssets,
    },
    player::{Player, PlayerEvent},
    ui::{menu::GamePreferences, scoreboard::ScoreboardEvent},
    world::LevelAsset,
};

use self::{
    bullet::{disable_gravity_for_bullets, handle_bullet_collisions, BulletBundle},
    orbie::OrbieEnemy,
};

use bevy_kira_audio::Audio;

mod bullet;
mod orbie;

#[derive(Default)]
pub struct Waypoints {
    pub inside: Vec<Vec3>,
    pub outside: Vec<Vec3>,
    pub window: Vec<Vec3>,
    pub outfront: Vec<Vec3>,
}

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Waypoints::default())
            .insert_resource(EnemiesState::default())
            .insert_resource(EnemySpawnTimer({
                let mut timer = Timer::from_seconds(1.0, true);
                timer.pause();
                timer
            }))
            //.insert_resource(WaypointTimer(Timer::from_seconds(5.0, false)))
            .insert_resource(UpdateDestinationsTimer(Timer::from_seconds(2.0, true)))
            .add_system_set(SystemSet::on_enter(GameState::Playing))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(enemies_look_at)
                    .with_system(enemies_fire_at_player)
                    .with_system(handle_bullet_collisions)
                    .with_system(disable_gravity_for_bullets)
                    //.with_system(waypoint_debug)
                    .with_system(spawn_enemies_on_timer)
                    .with_system(update_destinations)
                    .with_system(enemies_update_current_destination)
                    .with_system(enemies_move_to_destination)
                    .with_system(kill_enemy)
                    .with_system(progress_explosions)
                    .with_system(clean_up_dead)
                    .with_system(player_takes_damage),
            );
    }
}

pub struct EnemiesState {
    pub enemies_killed: u32,
    pub current_level: usize,
    pub levels: [LevelParams; 14],
    pub destinations: [usize; 3], //Typically, the 3 points closest to the player
    pub last_time_player_took_damage: f32,
}

pub struct LevelParams {
    kills_to_level_up: usize,
    max_enemies: usize,
    damage_multiplier: f32,
}

impl LevelParams {
    fn new(kills_to_level_up: usize, max_enemies: usize, damage_multiplier: f32) -> Self {
        LevelParams {
            kills_to_level_up,
            max_enemies,
            damage_multiplier,
        }
    }
}

impl Default for EnemiesState {
    fn default() -> Self {
        EnemiesState {
            enemies_killed: 0,
            current_level: 0,
            levels: [
                LevelParams::new(20, 6, 1.0),
                LevelParams::new(40, 7, 1.0),
                LevelParams::new(60, 8, 1.0),
                LevelParams::new(80, 9, 1.0),
                LevelParams::new(110, 10, 1.0),
                LevelParams::new(130, 11, 1.1),
                LevelParams::new(160, 12, 1.2),
                LevelParams::new(200, 12, 1.3),
                LevelParams::new(250, 12, 1.4),
                LevelParams::new(300, 12, 1.5),
                LevelParams::new(350, 12, 1.52),
                LevelParams::new(400, 12, 1.54),
                LevelParams::new(450, 13, 1.56),
                LevelParams::new(500, 14, 1.58),
            ],
            destinations: [0, 1, 2],
            last_time_player_took_damage: 0.0,
        }
    }
}

impl EnemiesState {
    pub fn get_level_params(&self) -> &LevelParams {
        &self.levels[self.current_level.min(self.levels.len() - 1)]
    }
}

struct UpdateDestinationsTimer(Timer);

fn update_destinations(
    time: Res<Time>,
    mut timer: ResMut<UpdateDestinationsTimer>,
    mut enemies_state: ResMut<EnemiesState>,
    waypoints: Res<Waypoints>,
    players: Query<&Transform, With<Player>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    if let Some(player_transform) = players.iter().next() {
        // Find the 3 closest inside waypoints to the player
        let mut distances = Vec::new(); //TODO don't allocate
        for (i, loc) in waypoints.inside.iter().enumerate() {
            distances.push((player_transform.translation.distance(*loc), i));
        }
        distances.sort_by(|a, b| (a.0).partial_cmp(&b.0).unwrap());
        if time.seconds_since_startup() as f32 - enemies_state.last_time_player_took_damage > 10.0 {
            //Pick the closest waypoints if we haven't hit the player in a while
            enemies_state.destinations[0] = distances[0].1;
            enemies_state.destinations[1] = distances[1].1;
            enemies_state.destinations[2] = distances[2].1;
        } else {
            enemies_state.destinations[0] = distances[5].1; //Don't pick the closest one
            enemies_state.destinations[1] = distances[6].1;
            enemies_state.destinations[2] = distances[7].1;
        }
    }
}

pub struct EnemySpawnTimer(pub Timer);

fn spawn_enemies_on_timer(
    time: Res<Time>,
    mut timer: ResMut<EnemySpawnTimer>,
    mut commands: Commands,
    model_assets: Res<ModelAssets>,
    waypoints: Res<Waypoints>,
    enemies_state: Res<EnemiesState>,
    enemies: Query<&Transform, (With<Enemy>, Without<Player>, With<Alive>)>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    if enemies.iter().count() >= enemies_state.get_level_params().max_enemies {
        return;
    }
    //Try 3 times to spawn an enemy where there is enough space
    'outer: for _ in 0..3 {
        let spawn_point = waypoints.outfront.choose(&mut rand::thread_rng()).unwrap();
        for enemy in enemies.iter() {
            if spawn_point.distance(enemy.translation) < 5.0 {
                continue 'outer;
            }
        }
        OrbieEnemy::spawn(
            &mut commands,
            Transform::from_xyz(spawn_point.x, spawn_point.y, spawn_point.z),
            &model_assets,
        );
        return;
    }
}

#[derive(Component)]
pub struct Waypoint;
#[allow(dead_code)]
struct WaypointTimer(Timer);

#[allow(dead_code)]
fn waypoint_debug(
    time: Res<Time>,
    mut waypoint_timer: ResMut<WaypointTimer>,
    mut waypoints: Query<(Entity, &Transform, &mut Handle<StandardMaterial>), With<Waypoint>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    waypoint_timer.0.tick(time.delta());
    if !waypoint_timer.0.just_finished() {
        return;
    }

    let mut original_waypoints: Vec<_> = waypoints.iter_mut().collect();
    let waypoints: Vec<_> = original_waypoints
        .iter()
        .map(|(ent, transform, _)| WaypointForPathfinding {
            entity: *ent,
            pos: transform.translation,
        })
        .collect();
    if waypoints.len() <= 1 {
        error!("need at least 2 waypoints");
        return;
    }

    let mut rng = rand::thread_rng();
    let start_index: usize = rng.gen_range(0..waypoints.len());
    let end_index = loop {
        let end_index: usize = rng.gen_range(0..waypoints.len());
        if end_index != start_index {
            break end_index;
        }
    };

    let end = waypoints.get(end_index).unwrap();

    let path = astar(
        waypoints.get(start_index).unwrap(),
        |waypoint| {
            let mut successors = waypoints
                .clone()
                .into_iter()
                .filter(|other| waypoint != other)
                .map(|other| (other, waypoint.pos.distance(other.pos) as i32))
                .collect::<Vec<_>>();
            successors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
            successors.truncate(8);

            successors
        },
        |waypoint| waypoint.pos.distance(end.pos) as i32,
        |waypoint| waypoint == end,
    )
    .unwrap() //TODO 'called `Option::unwrap()` on a `None` value'
    .0;

    let material = materials.add(StandardMaterial {
        base_color: Color::RED,
        ..Default::default()
    });

    println!("{}", path.len());

    for waypoint in &path {
        let waypoint = original_waypoints
            .iter_mut()
            .find(|(entity, ..)| waypoint.entity == *entity)
            .unwrap();
        *waypoint.2 = material.clone();
    }

    let spline_keys = path.iter().enumerate().map(|(i, waypoint)| {
        println!("> {}", i as f32 / (path.len() as f32 - 1.0));
        splines::Key::new(
            i as f32 / (path.len() as f32 - 1.0),
            waypoint.pos,
            Interpolation::Cosine,
        )
    });
    let spline = Spline::from_iter(spline_keys);

    for i in 0..100 {
        println!(">>> {}", i as f32 / 100.0);
        let p = spline.sample(i as f32 / 100.0);
        println!("{:?}", p);
    }
}

#[derive(Clone, Copy, Debug)]
struct WaypointForPathfinding {
    entity: Entity,
    pos: Vec3,
}

impl PartialEq for WaypointForPathfinding {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}

impl Eq for WaypointForPathfinding {}

impl Hash for WaypointForPathfinding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entity.hash(state)
    }
}

#[derive(Component)]
pub struct Enemy {
    pub health: i32,
    within_range_of_player: bool,
    range: f32,
    current_destination: usize,
    current_random_offset: Vec3,
    update_destination_timer: Timer,
    move_speed: f32,
    weapon_damage: f32,
    weapon_splash_radius: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Enemy {
            health: 1000,
            within_range_of_player: false,
            range: 100.0,
            current_destination: 0,
            update_destination_timer: Timer::from_seconds(2.0, true),
            move_speed: 30.0,
            current_random_offset: Vec3::new(0.0, 0.0, 0.0),
            weapon_damage: 40.0,
            weapon_splash_radius: 8.0,
        }
    }
}

#[derive(Component)]
pub struct EnemyLastFired(Timer);

#[derive(Component)]
pub struct Alive;

#[derive(Component)]
pub struct Dead {
    time_to_despawn: f32,
}

trait EnemyBehaviour {
    fn spawn(commands: &mut Commands, transform: Transform, model_assets: &ModelAssets) -> Entity;
}

fn enemies_update_current_destination(
    time: Res<Time>,
    mut enemies: Query<&mut Enemy, (Without<Player>, With<Alive>)>,
    enemies_state: Res<EnemiesState>,
) {
    for mut enemy in enemies.iter_mut() {
        enemy.update_destination_timer.tick(time.delta());
        if !enemy.update_destination_timer.just_finished() {
            continue;
        }
        enemy.current_destination = *enemies_state
            .destinations
            .choose(&mut rand::thread_rng())
            .unwrap();

        let mut rng = rand::thread_rng();
        enemy.current_random_offset.x = rng.gen_range(-5.0f32..=5.0f32);
        enemy.current_random_offset.y = rng.gen_range(-20.0f32..=0.0f32);
        enemy.current_random_offset.z = rng.gen_range(-5.0f32..=5.0f32);
    }
}

fn enemies_move_to_destination(
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut enemies: Query<
        (&mut Transform, &mut Enemy, &RigidBodyHandle),
        (Without<Player>, With<Alive>),
    >,
    waypoints: Res<Waypoints>,
) {
    for (mut enemy_transform, enemy, rb) in enemies.iter_mut() {
        if let Some(body) = rigid_bodies.get_mut(rb.into_rapier()) {
            let destination =
                waypoints.inside[enemy.current_destination] + enemy.current_random_offset;

            let dist = enemy_transform.translation.distance(destination);
            let mut move_speed = enemy.move_speed;
            if dist > 100.0 {
                // enemies move faster if they have to go far.
                move_speed *= 3.0;
            }
            let mut move_trans = enemy_transform.looking_at(destination, Vec3::Y).forward()
                * (move_speed * (dist - 2.0)).min(move_speed).max(0.0);

            move_trans =
                Vec3::new(body.linvel().x, body.linvel().y, body.linvel().z).lerp(move_trans, 0.04);
            if !move_trans.is_finite() {
                move_trans.x = 0.0;
                move_trans.y = 0.0;
                move_trans.z = 0.0;
            }
            body.set_linvel([move_trans.x, move_trans.y, move_trans.z].into(), false);

            if !enemy.within_range_of_player {
                let target = enemy_transform.looking_at(destination, Vec3::Y);
                enemy_transform.rotation = enemy_transform.rotation.lerp(target.rotation, 0.04);
            }
        }
    }
}

fn kill_enemy(
    time: Res<Time>,
    mut commands: Commands,
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut enemies: Query<
        (Entity, &mut Transform, &mut Enemy, &RigidBodyHandle),
        (Without<Player>, With<Alive>),
    >,
    mut enemies_state: ResMut<EnemiesState>,
    mut orb_materials: ResMut<Assets<OrbMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    mut scoreboard_events: EventWriter<ScoreboardEvent>,
    preferences: Res<GamePreferences>,
) {
    for (entity, enemy_transform, enemy, rb) in enemies.iter_mut() {
        if enemy.health > 0 {
            continue;
        }
        if let Some(body) = rigid_bodies.get_mut(rb.into_rapier()) {
            let mut rng = rand::thread_rng();
            body.apply_torque_impulse(
                [
                    rng.gen_range(-2000.0f32..=2000.0f32),
                    rng.gen_range(-2000.0f32..=2000.0f32),
                    rng.gen_range(-2000.0f32..=2000.0f32),
                ]
                .into(),
                false,
            );
            body.apply_impulse([0.0, -1500.0, 0.0].into(), false);
        }

        let orb_material_props = OrbProperties {
            color_tint: Vec3::new(1.0, 0.9, 0.5),
            alpha: 1.0,
            ..Default::default()
        };
        let orb_material = orb_materials.add(OrbMaterial {
            material_properties: orb_material_props,
            noise_texture: None,
        });
        let mesh = meshes.add(Mesh::from(shape::Icosphere {
            radius: 2.5,
            subdivisions: 1,
        })); //TODO use billboard
        commands
            .spawn()
            .insert_bundle(MaterialMeshBundle {
                mesh,
                transform: *enemy_transform,
                material: orb_material.clone(),
                ..Default::default()
            })
            .insert(Explosion {
                progress: 0.0,
                speed: 3.0,
                scale: 0.033,
                handle: orb_material,
            });

        commands.entity(entity).remove::<Alive>();

        let time_till_despawn = if preferences.potato { 2.0 } else { 12.0 };
        commands.entity(entity).insert(Dead {
            time_to_despawn: time.seconds_since_startup() as f32 + time_till_despawn,
        });
        // TODO use event
        audio.play(audio_assets.get_unit2_explosion().clone());
        enemies_state.enemies_killed += 1;
        scoreboard_events.send(ScoreboardEvent::Kill);
        if enemies_state.enemies_killed >= enemies_state.get_level_params().kills_to_level_up as u32
        {
            enemies_state.current_level =
                (enemies_state.current_level + 1).min(enemies_state.levels.len() - 1);
            scoreboard_events.send(ScoreboardEvent::LevelUp);
        }
    }
}

fn enemies_look_at(
    players: Query<&Transform, With<Player>>,
    mut enemies: Query<(&mut Transform, &mut Enemy), (Without<Player>, With<Alive>)>,
    player: Query<&Player>,
) {
    if let Some(player) = player.iter().next() {
        if let Some(player_transform) = players.iter().next() {
            for (mut enemy_transform, mut enemy) in enemies.iter_mut() {
                if enemy_transform
                    .translation
                    .distance(player_transform.translation)
                    <= enemy.range
                    && player.health > 0
                {
                    enemy.within_range_of_player = true;
                    let target = enemy_transform
                        .looking_at(player_transform.translation + Vec3::Y * 1.5, Vec3::Y);
                    enemy_transform.rotation = enemy_transform.rotation.lerp(target.rotation, 0.9);
                } else {
                    enemy.within_range_of_player = false;
                }
            }
        }
    }
}

fn enemies_fire_at_player(
    mut commands: Commands,
    time: Res<Time>,
    mut enemies: Query<(&Transform, &mut EnemyLastFired, &mut Enemy), With<Alive>>,
    mut orb_materials: ResMut<Assets<OrbMaterial>>,
    enemies_state: Res<EnemiesState>,
    mut meshes: ResMut<Assets<Mesh>>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    player: Query<&Player>,
) {
    if let Some(player) = player.iter().next() {
        if player.health <= 0 {
            return;
        }
    }
    for (transform, mut enemy_last_fired, enemy) in enemies.iter_mut() {
        enemy_last_fired.0.tick(time.delta());
        if enemy_last_fired.0.just_finished() && enemy.within_range_of_player {
            // Shoot at player
            commands
                .spawn_bundle(BulletBundle::shoot(
                    transform.translation,
                    transform.forward(),
                    (enemy.weapon_damage as f32
                        * enemies_state.get_level_params().damage_multiplier)
                        as i32,
                    enemy.weapon_splash_radius,
                ))
                .with_children(|parent| {
                    // // Debug hit box
                    let orb_material_props = OrbProperties {
                        color_tint: Vec3::new(0.5, 0.5, 1.0),
                        radius: 0.0,
                        inner_radius: 0.28,
                        alpha: 1.0,
                        ..Default::default()
                    };
                    let orb_material = orb_materials.add(OrbMaterial {
                        material_properties: orb_material_props,
                        noise_texture: None,
                    });
                    let mesh = meshes.add(Mesh::from(shape::Icosphere {
                        radius: 2.0,
                        subdivisions: 1,
                    })); //TODO use billboard
                    parent
                        .spawn()
                        .insert_bundle(MaterialMeshBundle {
                            mesh,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            material: orb_material.clone(),
                            ..Default::default()
                        })
                        .insert(LevelAsset::OrbMaterial {
                            properties: orb_material_props,
                            handle: orb_material,
                        });
                });
            // TODO use event
            audio.play(audio_assets.get_unit2_fire().clone());
        }
    }
}

fn clean_up_dead(mut commands: Commands, time: Res<Time>, deads: Query<(Entity, &Dead)>) {
    for (entity, dead) in deads.iter() {
        if time.seconds_since_startup() as f32 >= dead.time_to_despawn {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// TODO move elsewhere
#[derive(Component)]
pub struct Explosion {
    pub progress: f32,
    pub speed: f32,
    pub scale: f32,
    pub handle: Handle<OrbMaterial>,
}

fn progress_explosions(
    mut commands: Commands,
    time: Res<Time>,
    mut explosions: Query<(Entity, &mut Transform, &mut Explosion)>,
    mut orb_materials: ResMut<Assets<OrbMaterial>>,
) {
    for (entity, mut trans, mut explosion) in explosions.iter_mut() {
        explosion.progress += time.delta_seconds() * explosion.speed;
        if explosion.progress >= 1.0 {
            commands.entity(entity).despawn();
        } else {
            trans.scale *= 1.0 + explosion.progress * explosion.scale;
            if let Some(mat) = orb_materials.get_mut(explosion.handle.clone()) {
                mat.material_properties.alpha = 1.0 - explosion.progress;
            }
        }
    }
}

fn player_takes_damage(
    time: Res<Time>,
    mut player_events: EventReader<PlayerEvent>,
    mut enemies_state: ResMut<EnemiesState>,
) {
    for player_event in player_events.iter() {
        if let PlayerEvent::Hit = player_event {
            enemies_state.last_time_player_took_damage = time.seconds_since_startup() as f32;
        }
    }
}
