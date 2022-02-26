use bevy::prelude::*;

use bevy_polyline::{Polyline, PolylineBundle, PolylineMaterial};

#[derive(Component)]
pub struct DebugTimeout(pub f64);

pub fn clean_up_debug_lines(
    mut commands: Commands,
    time: Res<Time>,
    debug_query: Query<(Entity, &mut DebugTimeout)>,
) {
    for (e, d) in debug_query.iter() {
        if d.0 < time.time_since_startup().as_secs_f64() && d.0 > 0.0 {
            commands.entity(e).despawn();
        }
    }
}

pub fn create_debug_line(
    commands: &mut Commands,
    polyline_materials: &mut ResMut<Assets<PolylineMaterial>>,
    polylines: &mut ResMut<Assets<Polyline>>,
    time: &Res<Time>,
    start: Vec3,
    end: Vec3,
    life: f64,
    color: Color,
) {
    let life = if life >= 0.0 {
        time.time_since_startup().as_secs_f64() + life
    } else {
        life
    };
    commands
        .spawn_bundle(PolylineBundle {
            polyline: polylines.add(Polyline {
                vertices: vec![start, end],
            }),
            material: polyline_materials.add(PolylineMaterial {
                width: 4.0,
                color,
                perspective: true,
            }),
            ..Default::default()
        })
        .insert(DebugTimeout(life));
}
