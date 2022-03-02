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
    assets::{GameState, ModelAssets},
    player::Player,
};

use self::{
    bullet::{disable_gravity_for_bullets, handle_bullet_collisions, BulletBundle},
    orbie::OrbieEnemy,
};

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
            .insert_resource(EnemySpawnTimer(Timer::from_seconds(1.0, true)))
            //.insert_resource(WaypointTimer(Timer::from_seconds(5.0, false)))
            .insert_resource(UpdateDestinationsTimer(Timer::from_seconds(2.0, true)))
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_enemies))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(enemies_look_at_player)
                    .with_system(enemies_fire_at_player)
                    .with_system(handle_bullet_collisions)
                    .with_system(disable_gravity_for_bullets)
                    //.with_system(waypoint_debug)
                    .with_system(spawn_enemies_on_timer)
                    .with_system(update_destinations)
                    .with_system(enemies_update_current_destination)
                    .with_system(enemies_move_to_destination)
                    .with_system(kill_enemy),
            );
    }
}

struct EnemiesState {
    pub enemies_killed: u32,
    pub current_level: usize,
    pub levels: [LevelParams; 10],
    pub destinations: [usize; 3], //Typically, the 3 points closest to the player
}

struct LevelParams {
    kills_to_level_up: usize,
    max_enemies: usize,
    dammage_multiplier: f32,
}

impl LevelParams {
    fn new(kills_to_level_up: usize, max_enemies: usize, dammage_multiplier: f32) -> Self {
        LevelParams {
            kills_to_level_up,
            max_enemies,
            dammage_multiplier,
        }
    }
}

impl Default for EnemiesState {
    fn default() -> Self {
        EnemiesState {
            enemies_killed: 0,
            current_level: 0,
            levels: [
                LevelParams::new(8, 6, 1.0),
                LevelParams::new(16, 7, 1.0),
                LevelParams::new(24, 8, 1.0),
                LevelParams::new(32, 9, 1.0),
                LevelParams::new(64, 10, 1.0),
                LevelParams::new(96, 11, 1.1),
                LevelParams::new(128, 12, 1.2),
                LevelParams::new(192, 12, 1.3),
                LevelParams::new(256, 12, 1.4),
                LevelParams::new(384, 12, 1.5),
            ],
            destinations: [0, 1, 2],
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
        enemies_state.destinations[0] = distances[5].1; //Don't pick the closest one
        enemies_state.destinations[1] = distances[6].1;
        enemies_state.destinations[2] = distances[7].1;
    }
}

struct EnemySpawnTimer(Timer);

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
    weapon_dammage: f32,
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
            weapon_dammage: 30.0,
        }
    }
}

#[derive(Component)]
pub struct EnemyLastFired(Timer);

#[derive(Component)]
pub struct Alive;

trait EnemyBehaviour {
    fn spawn(commands: &mut Commands, transform: Transform, model_assets: &ModelAssets) -> Entity;
}

fn spawn_enemies(mut commands: Commands, model_assets: Res<ModelAssets>) {
    OrbieEnemy::spawn(
        &mut commands,
        Transform::from_xyz(0.0, 18.0, -10.0).looking_at(Vec3::ZERO * -Vec3::X, Vec3::Y),
        &model_assets,
    );
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
        enemy.current_random_offset.y = rng.gen_range(-15.0f32..=0.0f32);
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

            let mut move_trans = enemy_transform.looking_at(destination, Vec3::Y).forward()
                * (enemy.move_speed * (dist - 2.0))
                    .min(enemy.move_speed)
                    .max(0.0);

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
    mut commands: Commands,
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut enemies: Query<
        (Entity, &mut Transform, &mut Enemy, &RigidBodyHandle),
        (Without<Player>, With<Alive>),
    >,
    mut enemies_state: ResMut<EnemiesState>,
) {
    for (entity, _enemy_transform, enemy, rb) in enemies.iter_mut() {
        if enemy.health > 0 {
            continue;
        }
        if let Some(body) = rigid_bodies.get_mut(rb.into_rapier()) {
            let mut rng = rand::thread_rng();
            body.apply_torque(
                [
                    rng.gen_range(-5.0f32..=5.0f32),
                    rng.gen_range(-5.0f32..=5.0f32),
                    rng.gen_range(-5.0f32..=5.0f32),
                ]
                .into(),
                false,
            );
            body.apply_impulse([0.0, -500.0, 0.0].into(), false);
        }
        commands.entity(entity).remove::<Alive>();
        // ENEMY KILLED - TODO show kills on screen
        enemies_state.enemies_killed += 1;
        // LEVEL UP - TODO show level on screen
        if enemies_state.enemies_killed >= enemies_state.get_level_params().kills_to_level_up as u32
        {
            enemies_state.current_level =
                (enemies_state.current_level + 1).min(enemies_state.levels.len() - 1);
            dbg!(enemies_state.current_level);
        }
    }
}

fn enemies_look_at_player(
    players: Query<&Transform, With<Player>>,
    mut enemies: Query<(&mut Transform, &mut Enemy), (Without<Player>, With<Alive>)>,
) {
    if let Some(player_transform) = players.iter().next() {
        for (mut enemy_transform, mut enemy) in enemies.iter_mut() {
            if enemy_transform
                .translation
                .distance(player_transform.translation)
                <= enemy.range
            {
                enemy.within_range_of_player = true;
                let target = enemy_transform
                    .looking_at(player_transform.translation + Vec3::Y * 1.5, Vec3::Y);
                enemy_transform.rotation = enemy_transform.rotation.lerp(target.rotation, 0.4);
            } else {
                enemy.within_range_of_player = false;
            }
        }
    }
}

fn enemies_fire_at_player(
    mut commands: Commands,
    time: Res<Time>,
    mut enemies: Query<(&Transform, &mut EnemyLastFired, &mut Enemy), With<Alive>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemies_state: Res<EnemiesState>,
) {
    for (transform, mut enemy_last_fired, enemy) in enemies.iter_mut() {
        enemy_last_fired.0.tick(time.delta());
        if enemy_last_fired.0.just_finished() && enemy.within_range_of_player {
            // Shoot at player
            commands
                .spawn_bundle(BulletBundle::shoot(
                    transform.translation,
                    transform.forward(),
                    (enemy.weapon_dammage as f32
                        * enemies_state.get_level_params().dammage_multiplier)
                        as i32,
                ))
                .with_children(|parent| {
                    // // Debug hit box
                    let mesh = meshes.add(Mesh::from(shape::Icosphere {
                        radius: 1.0,
                        subdivisions: 1,
                    }));
                    let material = materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        ..Default::default()
                    });

                    parent.spawn_bundle(PbrBundle {
                        mesh,
                        material,
                        ..Default::default()
                    });
                });
        }
    }
}
