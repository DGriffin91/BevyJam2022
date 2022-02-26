use bevy::prelude::*;
use rand::Rng;

#[derive(Component, Debug)]
pub struct Planet {
    velocity: Vec3,
    mass: f32,
}

pub fn spawn_planets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();

    let n = 6.0;

    for _ in 0..30 {
        let x = rng.gen_range(-n..n);
        let y = rng.gen_range(4.0..7.0);
        let z = rng.gen_range(-n..n);

        let mass = rng.gen_range(0.05..5.0);

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: mass * 0.1,
                    ..Default::default()
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.1, 0.1, 0.1),
                    ..Default::default()
                }),
                transform: Transform::from_xyz(x, 10.0 + y, z),
                ..Default::default()
            })
            .insert(Planet {
                velocity: Vec3::new(0.0, 0.0, 0.0),
                mass,
            });
    }
}

pub fn planitary_physics(time: Res<Time>, mut planet_query: Query<(&mut Planet, &mut Transform)>) {
    let mut pos_mass = Vec::new(); //todo, don't allocate
    for (planet, transform) in planet_query.iter() {
        pos_mass.push((transform.translation, planet.mass));
    }
    for (i, (mut planet, mut transform)) in planet_query.iter_mut().enumerate() {
        for (j, (transform_a, mass_a)) in pos_mass.iter().enumerate() {
            if i != j {
                let difference = transform.translation - *transform_a;
                let d = difference.powf(2.0);
                let distance_squared = d.x + d.y + d.z;

                planet.velocity -= (difference).normalize()
                    * ((1.0 / mass_a.powf(2.0).max(distance_squared)) * mass_a)
                    * time.delta_seconds();
            }
        }
        transform.translation += planet.velocity * time.delta_seconds();
        let r = planet.mass * 0.1;
        //bounce off walls-ish
        let hit = transform.translation.x < -50.0 + r
            || transform.translation.x > 50.0 - r
            || transform.translation.y < 0.0 + r
            || transform.translation.y > 50.0 - r
            || transform.translation.z < -100.0 + r
            || transform.translation.z > 100.0 - r;
        if hit {
            planet.velocity *= -1.0;
        }
    }
}
