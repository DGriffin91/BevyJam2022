use bevy::prelude::*;
use heron::{
    rapier_plugin::{convert::IntoRapier, rapier3d::prelude::RigidBodySet, RigidBodyHandle},
    CollisionData, CollisionEvent, CollisionLayers, CollisionShape, PhysicMaterial, RigidBody,
    Velocity,
};

use crate::{
    player::{Player, PlayerEvent},
    Layer,
};

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
    pub fn shoot(from: Vec3, direction: Vec3, damage: i32, splash_radius: f32) -> Self {
        BulletBundle {
            bullet: Bullet {
                damage,
                splash_radius,
            },
            transform: Transform::from_translation(from).looking_at(direction, Vec3::Y),
            global_transform: GlobalTransform::default(),
            rigid_body: RigidBody::Dynamic,
            collision_shape: CollisionShape::Sphere { radius: 2.0 }, //purposely oversized
            collision_layers: CollisionLayers::none()
                .with_group(Layer::Bullet)
                .with_masks([Layer::World, Layer::Player]),
            velocity: Velocity::from_linear(direction * 80.0),
            physic_material: PhysicMaterial {
                // density: 0.001,
                ..Default::default()
            },
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    damage: i32,
    splash_radius: f32,
}

pub fn disable_gravity_for_bullets(
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut new_bullets: Query<&RigidBodyHandle, (With<Bullet>, Added<RigidBodyHandle>)>,
) {
    for handle in new_bullets.iter_mut() {
        if let Some(body) = rigid_bodies.get_mut(handle.into_rapier()) {
            body.set_gravity_scale(0.0, false);
            body.enable_ccd(true);
        }
    }
}

pub fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut players: Query<(Entity, &Transform, &mut Player)>,
    mut player_events: EventWriter<PlayerEvent>,
    bullets: Query<(&Transform, &Bullet)>,
) {
    for collision in collision_events.iter() {
        match collision {
            CollisionEvent::Started(d1, d2) => {
                let (bullet, other) = if is_bullet(d1) {
                    (d1, d2)
                } else if is_bullet(d2) {
                    (d2, d1)
                } else {
                    continue;
                };

                let (bullet_ent, other_ent) =
                    (bullet.rigid_body_entity(), other.rigid_body_entity());

                if let Ok((bullet_transform, bullet)) = bullets.get(bullet_ent) {
                    if is_player(other) {
                        if let Ok((_, _, mut player)) = players.get_mut(other_ent) {
                            player_events.send(PlayerEvent::Hit);
                            player.health -= bullet.damage;
                        }
                    } else if let Some((_entity, player_transform, mut player)) =
                        players.iter_mut().next()
                    {
                        if bullet_transform
                            .translation
                            .distance(player_transform.translation)
                            < bullet.splash_radius
                        //Splash Damage
                        {
                            player_events.send(PlayerEvent::Hit);
                            player.health -= (bullet.damage as f32 * 0.5) as i32;
                        }
                    }
                }

                commands.entity(bullet_ent).despawn_recursive();
            }
            CollisionEvent::Stopped(..) => {}
        }
    }
}

#[inline]
fn is_bullet(collision_data: &CollisionData) -> bool {
    collision_data
        .collision_layers()
        .contains_group(Layer::Bullet)
}

#[inline]
fn is_player(collision_data: &CollisionData) -> bool {
    collision_data
        .collision_layers()
        .contains_group(Layer::Player)
}

// #[inline]
// fn is_world(collision_data: &CollisionData) -> bool {
//     collision_data
//         .collision_layers()
//         .contains_group(Layer::World)
// }
