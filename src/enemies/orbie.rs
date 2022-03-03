use bevy::prelude::*;
use heron::{CollisionLayers, CollisionShape, PhysicMaterial, PhysicsLayer, RigidBody};

use crate::{
    assets::{
        orb_material::{OrbMaterial, OrbProperties},
        AudioAssets, ModelAssets,
    },
    player::Player,
    world::LevelAsset,
    Layer,
};

use super::{bullet::BulletBundle, Alive, EnemiesState, Enemy, EnemyBehaviour, EnemyLastFired};

use bevy_kira_audio::Audio;

#[derive(Component, Default)]
pub struct OrbieEnemy;

impl EnemyBehaviour for OrbieEnemy {
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
                health: 1000,
                range: 100.0,
                update_destination_timer: Timer::from_seconds(2.0, true),
                move_speed: 30.0,
                weapon_damage: 40.0,
                weapon_splash_radius: 8.0,
                rotate_lerp: 0.9,
                ..Default::default()
            })
            .insert(OrbieEnemy)
            .insert(Alive)
            .with_children(|parent| {
                parent.spawn_scene(model_assets.unit2.clone());
            })
            .id()
    }
}

pub fn orbie_enemies_fire_at_player(
    mut commands: Commands,
    time: Res<Time>,
    mut enemies: Query<
        (&Transform, &mut EnemyLastFired, &mut Enemy),
        (With<Alive>, With<OrbieEnemy>),
    >,
    mut orb_materials: ResMut<Assets<OrbMaterial>>,
    enemies_state: Res<EnemiesState>,
    mut meshes: ResMut<Assets<Mesh>>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    player: Query<&Player>,
) {
    if let Some(player) = player.iter().next() {
        if player.health <= 0.0 {
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
                    enemy.weapon_damage as f32 * enemies_state.get_level_params().damage_multiplier,
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
