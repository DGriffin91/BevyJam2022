use bevy::prelude::*;
use bevy_polyline::{Polyline, PolylineBundle, PolylineMaterial};
use heron::{
    rapier_plugin::PhysicsWorld, CollisionLayers, CollisionShape, PhysicMaterial, PhysicsLayer,
    RigidBody,
};

use crate::{assets::ModelAssets, player::Player, Layer};

use super::{Alive, Enemy, EnemyBehaviour, EnemyKind, EnemyLastFired};

#[derive(Component, Default)]
pub struct LaserieEnemy;
use bevy_kira_audio::Audio;

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
                move_speed: 35.0,
                weapon_damage: 10.0,
                weapon_splash_radius: 0.0,
                kind: EnemyKind::Laserie,
                rotate_lerp: 0.08,
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
        (
            &Transform,
            &mut EnemyLastFired,
            &mut Enemy,
            &mut Handle<Polyline>,
        ),
        (With<Alive>, With<LaserieEnemy>),
    >,
    //enemies_state: Res<EnemiesState>,
    //audio: Res<Audio>,
    //audio_assets: Res<AudioAssets>,
    mut players: Query<&mut Player>,
    mut polylines: ResMut<Assets<Polyline>>,
    physics_world: PhysicsWorld,
) {
    if let Some(player) = players.iter().next() {
        if player.health <= 0.0 {
            return;
        }
        for (transform, mut enemy_last_fired, enemy, polyline) in enemies.iter_mut() {
            enemy_last_fired.0.tick(time.delta()); //enemy_last_fired.0.just_finished() &&
            if enemy.within_range_of_player {
                // Shoot at player
                if let Some(poly) = polylines.get_mut(&*polyline) {
                    poly.vertices[0] = transform.translation;
                    poly.vertices[1] = transform.translation + transform.forward() * 100.0;
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
                            player.health -= enemy.weapon_damage * time.delta_seconds();
                        } else {
                            poly.vertices[0] = transform.translation;
                            poly.vertices[1] = collision.collision_point;
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
pub struct LaserPolyline {
    pub laserie: Option<Entity>,
    pub polyline: Handle<Polyline>,
}

pub fn add_lasers_to_laserie(
    mut commands: Commands,
    enemies: Query<
        (Entity, &mut Enemy),
        (
            Without<Player>,
            With<Alive>,
            Without<Handle<Polyline>>,
            With<LaserieEnemy>,
        ),
    >,
    mut polylines: Query<(&Handle<Polyline>, &mut LaserPolyline)>,
) {
    'outer: for (entity, enemy) in enemies.iter() {
        if let EnemyKind::Laserie = enemy.kind {
            for (polyline, mut laser_polyline) in polylines.iter_mut() {
                if laser_polyline.laserie.is_none() {
                    commands.entity(entity).insert(polyline.clone());
                    laser_polyline.laserie = Some(entity.clone());
                    continue 'outer;
                }
            }
        }
    }
}

pub fn create_laser_polylines(
    mut commands: Commands,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    for _ in 0..16 {
        let polyline = polylines.add(Polyline {
            vertices: vec![Vec3::ZERO, Vec3::ZERO],
        });
        commands
            .spawn_bundle(PolylineBundle {
                polyline: polyline.clone(),
                material: polyline_materials.add(PolylineMaterial {
                    width: 5.0,
                    color: Color::rgba(1.0, 0.0, 1.0, 0.9),
                    perspective: true,
                }),
                visibility: Visibility { is_visible: true },
                ..Default::default()
            })
            .insert(LaserPolyline {
                laserie: None,
                polyline,
            });
    }
}
