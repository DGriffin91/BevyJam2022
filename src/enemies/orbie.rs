use bevy::prelude::*;
use heron::{CollisionLayers, CollisionShape, PhysicMaterial, PhysicsLayer, RigidBody};

use crate::{assets::ModelAssets, Layer};

use super::{Enemy, EnemyBehaviour, EnemyLastFired};

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
            .insert(EnemyLastFired(Timer::from_seconds(0.8, true)))
            .insert(Enemy::default())
            .insert(OrbieEnemy)
            .with_children(|parent| {
                parent.spawn_scene(model_assets.unit2.clone());
            })
            .id()
    }
}
