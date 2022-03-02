use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use bevy::prelude::*;
use pathfinding::directed::astar::astar;
use rand::Rng;

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

#[derive(Component)]
pub struct Waypoint;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaypointTimer(Timer::from_seconds(2.0, false)))
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_enemies))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(enemies_look_at_player)
                    .with_system(enemies_fire_at_player)
                    .with_system(handle_bullet_collisions)
                    .with_system(disable_gravity_for_bullets)
                    .with_system(waypoint_debug),
            );
    }
}

struct WaypointTimer(Timer);

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
    .unwrap()
    .0;

    let material = materials.add(StandardMaterial {
        base_color: Color::RED,
        ..Default::default()
    });

    println!("{}", path.len());

    for waypoint in path {
        let waypoint = original_waypoints
            .iter_mut()
            .find(|(entity, ..)| waypoint.entity == *entity)
            .unwrap();
        *waypoint.2 = material.clone();
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
    _health: f32,
    within_range_of_player: bool,
    range: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Enemy {
            _health: 1000.0,
            within_range_of_player: false,
            range: 50.0,
        }
    }
}

#[derive(Component)]
pub struct EnemyLastFired(Timer);

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

fn enemies_look_at_player(
    players: Query<&Transform, With<Player>>,
    mut enemies: Query<(&mut Transform, &mut Enemy), Without<Player>>,
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
                enemy_transform.rotation = enemy_transform.rotation.lerp(target.rotation, 0.04);
            } else {
                enemy.within_range_of_player = false;
            }
        }
    }
}

fn enemies_fire_at_player(
    mut commands: Commands,
    time: Res<Time>,
    mut enemies: Query<(&Transform, &mut EnemyLastFired, &mut Enemy)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (transform, mut enemy_last_fired, enemy) in enemies.iter_mut() {
        enemy_last_fired.0.tick(time.delta());
        if enemy_last_fired.0.just_finished() && enemy.within_range_of_player {
            // Shoot at player
            commands
                .spawn_bundle(BulletBundle::shoot(
                    transform.translation,
                    transform.forward(),
                ))
                .with_children(|parent| {
                    // // Debug hit box
                    let mesh = meshes.add(Mesh::from(shape::Icosphere {
                        radius: 0.1,
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
