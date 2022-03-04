use bevy::prelude::*;
use bevy_polyline::{Polyline, PolylineBundle, PolylineMaterial};
use heron::{
    rapier_plugin::PhysicsWorld, CollisionLayers, CollisionShape, PhysicMaterial, PhysicsLayer,
    RigidBody,
};

use crate::{
    assets::ModelAssets,
    player::{Player, PlayerEvent},
    Layer,
};

use super::{Alive, Dead, EnemiesState, Enemy, EnemyBehaviour, EnemyLastFired};

#[derive(Component, Default)]
pub struct LaserieEnemy;

impl EnemyBehaviour for LaserieEnemy {
    fn spawn(commands: &mut Commands, transform: Transform, model_assets: &ModelAssets) -> Entity {
        commands
            .spawn_bundle((transform, GlobalTransform::default()))
            .insert(RigidBody::Dynamic)
            .insert(CollisionShape::Sphere { radius: 2.7 })
            .insert(CollisionLayers::from_bits(
                Layer::Enemy.to_bits(),
                Layer::all_bits(),
            ))
            .insert(PhysicMaterial {
                density: 1.0, // Value must be greater than 0.0
                ..Default::default()
            })
            .insert(EnemyLastFired(Timer::from_seconds(0.9, true)))
            .insert(Enemy {
                health: 500,
                range: 100.0,
                update_destination_timer: Timer::from_seconds(2.0, true),
                move_speed: 38.0,
                weapon_damage: 15.0,
                weapon_splash_radius: 0.0,
                rotate_lerp: 0.3,
                ..Default::default()
            })
            .insert(LaserieEnemy)
            .insert(Alive)
            .with_children(|parent| {
                parent.spawn_scene(model_assets.unit1.clone());
            })
            .id()
    }
}

pub fn laserie_enemies_fire_at_player(
    //mut commands: Commands,
    time: Res<Time>,
    mut enemies: Query<
        (&Transform, &mut EnemyLastFired, &mut Enemy, &Children),
        (With<Alive>, With<LaserieEnemy>),
    >,
    //enemies_state: Res<EnemiesState>,
    //audio: Res<Audio>,
    //audio_assets: Res<AudioAssets>,
    mut players: Query<&mut Player>,
    mut polylines: ResMut<Assets<Polyline>>,
    physics_world: PhysicsWorld,
    spawned_polys: Query<&Handle<Polyline>>,
    mut player_events: EventWriter<PlayerEvent>,
    enemies_state: Res<EnemiesState>,
) {
    if let Some(player) = players.iter().next() {
        if player.health <= 0.0 {
            for (.., children) in enemies.iter_mut() {
                for &child in children.iter() {
                    if let Ok(polyline_h) = spawned_polys.get(child) {
                        if let Some(polyline) = polylines.get_mut(polyline_h) {
                            polyline.vertices[0] = Vec3::ZERO;
                            polyline.vertices[1] = Vec3::ZERO;
                        }
                    }
                }
            }
            return;
        }
        for (transform, mut enemy_last_fired, enemy, children) in enemies.iter_mut() {
            enemy_last_fired.0.tick(time.delta()); //enemy_last_fired.0.just_finished() &&
            if enemy.within_range_of_player {
                // Shoot at player

                let mut endpoint = Vec3::new(0.5, 0.5, -100.0);
                if let Some(collision) = physics_world.ray_cast_with_filter(
                    transform.translation,
                    transform.forward() * 100.0,
                    true,
                    CollisionLayers::none()
                        .with_group(Layer::Raycast)
                        .with_masks([Layer::World, Layer::Player]),
                    |_| true,
                ) {
                    // TODO move to be triggered by event
                    if let Ok(mut player) = players.get_mut(collision.entity) {
                        player.health -= enemy.weapon_damage
                            * time.delta_seconds()
                            * enemies_state.get_level_params().damage_multiplier;
                        player_events.send(PlayerEvent::Hit { laser: true });
                    } else {
                        endpoint = Vec3::new(
                            0.5,
                            0.5,
                            -collision.collision_point.distance(transform.translation),
                        );
                    }
                }

                for &child in children.iter() {
                    if let Ok(polyline_h) = spawned_polys.get(child) {
                        if let Some(polyline) = polylines.get_mut(polyline_h) {
                            polyline.vertices[0] = Vec3::ZERO;
                            polyline.vertices[1] = endpoint;
                        }
                    }
                }
                // TODO use event
                //audio.play(audio_assets.get_unit2_fire().clone());
            }
        }
    }
}

#[derive(Component)]
pub struct HasLaser;
//TODO make part of initially creating laserie
pub fn add_lasers_to_laserie(
    mut commands: Commands,
    enemies: Query<
        Entity,
        (
            Without<Player>,
            With<Alive>,
            With<Enemy>,
            With<LaserieEnemy>,
            Without<HasLaser>,
        ),
    >,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    for entity in enemies.iter() {
        commands
            .entity(entity)
            .with_children(|parent| {
                parent.spawn_bundle(PolylineBundle {
                    polyline: polylines.add(Polyline {
                        vertices: vec![Vec3::ZERO, Vec3::ZERO],
                    }),
                    material: polyline_materials.add(PolylineMaterial {
                        width: 20.0,
                        color: Color::rgba(1.0, 0.0, 1.0, 0.9),
                        perspective: true,
                    }),
                    visibility: Visibility { is_visible: true },
                    ..Default::default()
                });
            })
            .insert(HasLaser);
    }
}

pub fn turn_off_dead_laser(
    enemies: Query<
        &Children,
        (
            Without<Player>,
            With<Dead>,
            With<HasLaser>,
            With<Enemy>,
            With<LaserieEnemy>,
        ),
    >,
    spawned_polys: Query<&Handle<Polyline>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    for children in enemies.iter() {
        for &child in children.iter() {
            if let Ok(polyline_h) = spawned_polys.get(child) {
                if let Some(polyline) = polylines.get_mut(polyline_h) {
                    polyline.vertices[0] = Vec3::ZERO;
                    polyline.vertices[1] = Vec3::ZERO;
                }
            }
        }
    }
}
