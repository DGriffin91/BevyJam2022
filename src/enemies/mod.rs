use bevy::prelude::*;

use crate::{
    assets::{GameState, ModelAssets},
    player::Player,
};

use self::{bullet::BulletBundle, orbie::OrbieEnemy};

mod bullet;
mod orbie;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_enemies))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(enemies_look_at_player)
                    .with_system(enemies_fire_at_player),
            );
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct EnemyLastFired(Timer);

trait EnemyBehaviour {
    fn spawn(commands: &mut Commands, transform: Transform, model_assets: &ModelAssets) -> Entity;
}

fn spawn_enemies(mut commands: Commands, model_assets: Res<ModelAssets>) {
    OrbieEnemy::spawn(
        &mut commands,
        Transform::from_xyz(0.0, 18.0, -10.0),
        &model_assets,
    );
}

fn enemies_look_at_player(
    players: Query<&Transform, With<Player>>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {
    if let Some(player_transform) = players.iter().next() {
        for mut enemy in enemies.iter_mut() {
            enemy.look_at(player_transform.translation, Vec3::Y);
        }
    }
}

fn enemies_fire_at_player(
    mut commands: Commands,
    time: Res<Time>,
    mut enemies: Query<(&Transform, &mut EnemyLastFired)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (transform, mut enemy_last_fired) in enemies.iter_mut() {
        enemy_last_fired.0.tick(time.delta());
        if enemy_last_fired.0.just_finished() {
            // Shoot at player
            commands
                .spawn_bundle(BulletBundle::shoot(
                    transform.translation,
                    transform.forward() * 10.0,
                ))
                .with_children(|parent| {
                    // // Debug hit box
                    let mesh = meshes.add(Mesh::from(shape::Cube { size: 0.25 }));
                    let material = materials.add(StandardMaterial {
                        base_color: Color::rgb_u8(100, 50, 180),
                        // alpha_mode: AlphaMode::Blend,
                        ..Default::default()
                    });

                    parent.spawn_bundle(PbrBundle {
                        mesh,
                        material,
                        // visibility: Visibility { is_visible: false },
                        ..Default::default()
                    });
                });
        }
    }
}
