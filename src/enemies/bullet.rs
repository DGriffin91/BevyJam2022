use bevy::prelude::*;
use heron::{CollisionLayers, CollisionShape, PhysicMaterial, RigidBody, Velocity};

#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    transform: Transform,
    global_transform: GlobalTransform,
    rigid_body: RigidBody,
    collision_shape: CollisionShape,
    collision_layers: CollisionLayers,
    velocity: Velocity,
    physic_material: PhysicMaterial,
}

impl BulletBundle {
    pub fn shoot(from: Vec3, direction: Vec3) -> Self {
        BulletBundle {
            bullet: Bullet,
            transform: Transform::from_translation(from).looking_at(direction, Vec3::Y),
            global_transform: GlobalTransform::default(),
            rigid_body: RigidBody::Dynamic,
            collision_shape: CollisionShape::Cylinder {
                half_height: 0.5,
                radius: 0.5,
            },
            collision_layers: CollisionLayers::none(),
            velocity: Velocity::from_linear(direction * 10.0),
            physic_material: PhysicMaterial {
                density: 0.0,
                ..Default::default()
            },
        }
    }
}

#[derive(Component)]
pub struct Bullet;
