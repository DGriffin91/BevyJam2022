use bevy::{
    math::{EulerRot, Quat, Vec3},
    pbr::{DirectionalLight, DirectionalLightBundle, MaterialMeshBundle},
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use heron::{
    rapier_plugin::{
        nalgebra::Point3,
        rapier3d::{math::Real, prelude::ColliderBuilder},
    },
    CollisionLayers, CollisionShape, CustomCollisionShape, PhysicMaterial, RigidBody,
};

use crate::{
    assets::{
        custom_material::{CustomMaterial, MaterialProperties, MaterialSetProp},
        GameState, ImageAssets,
    },
    assets::{
        emissive_material::EmissiveMaterial,
        light_shaft_material::{
            update_light_shaft_material_time, LightShaftMaterial, LightShaftProperties,
        },
        ModelAssets,
    },
    Layer,
};

use super::LevelAsset;

pub struct LevelOnePlugin;

impl Plugin for LevelOnePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(setup_level_one)
                .with_system(spawn_demo_cubes),
        )
        .add_system(update_light_shaft_material_time);
    }
}

fn spawn_demo_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let material = materials.add(StandardMaterial {
        base_color: Color::PINK,
        ..Default::default()
    });

    for i in 0..10 {
        commands
            .spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(0.0, i as f32 * 6.0 + 5.0, -50.0),
                ..Default::default()
            })
            .insert(RigidBody::Dynamic)
            .insert(CollisionShape::Cuboid {
                half_extends: Vec3::new(0.5, 0.5, 0.5),
                border_radius: None,
            })
            .insert(PhysicMaterial {
                restitution: 0.7,
                ..Default::default()
            });
    }
}

fn setup_level_one(
    mut commands: Commands,
    mesh_assets: Res<Assets<Mesh>>,
    image_assets: Res<ImageAssets>,
    model_assets: Res<ModelAssets>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut emissive_materials: ResMut<Assets<EmissiveMaterial>>,
    mut light_shaft_materials: ResMut<Assets<LightShaftMaterial>>,
) {
    let variation_texture = image_assets.detail.clone();
    let base_texture = image_assets.concrete.clone();
    let walls_texture = image_assets.concrete3.clone();
    let reflection_texture = image_assets.reflection.clone();

    let material_properties = MaterialProperties {
        lightmap: MaterialSetProp {
            scale: 1.0,
            contrast: 2.3,
            brightness: 2.5,
            blend: 1.0,
        },
        base_a: MaterialSetProp {
            scale: 8.5,
            contrast: 0.33,
            brightness: 2.5,
            blend: 1.0,
        },
        base_b: MaterialSetProp {
            scale: 33.0,
            contrast: 0.3,
            brightness: 2.2,
            blend: 1.0,
        },
        vary_a: MaterialSetProp {
            scale: 0.14,
            contrast: 0.77,
            brightness: 4.2,
            blend: 0.04,
        },
        vary_b: MaterialSetProp {
            scale: 70.0,
            contrast: 0.105,
            brightness: 1.35,
            blend: 1.0,
        },
        reflection: MaterialSetProp {
            scale: 1.0,
            contrast: 1.2,
            brightness: 0.023,
            blend: 1.0,
        },
        walls: MaterialSetProp {
            scale: 20.0,
            contrast: 0.53,
            brightness: 1.6,
            blend: 1.0,
        },
        reflection_mask: MaterialSetProp {
            scale: 0.047,
            contrast: 2.5,
            brightness: 40.0,
            blend: 1.0,
        },
        mist: MaterialSetProp {
            scale: 0.03,
            contrast: 1.0,
            brightness: 0.5,
            blend: 1.0,
        },
        directional_light_blend: 0.6,
    };

    let skybox_model = model_assets.level1_sky_box.clone();
    let skybox_texture = image_assets.level1_sky_box.clone();
    commands.spawn().insert_bundle(MaterialMeshBundle {
        mesh: skybox_model,
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(10.0, 10.0, 10.0)),
        material: emissive_materials.add(EmissiveMaterial {
            emissive: Color::WHITE,
            emissive_texture: Some(skybox_texture),
        }),
        ..Default::default()
    });

    let light_shaft_material_props = LightShaftProperties {
        shaft: MaterialSetProp {
            scale: 1.0,
            contrast: 1.9,
            brightness: 8.0,
            blend: 1.0,
        },
        noise_a: MaterialSetProp {
            scale: 1.0,
            contrast: 5.8,
            brightness: 40.0,
            blend: 0.25,
        },
        noise_b: MaterialSetProp {
            scale: 0.00048,
            contrast: 1.3,
            brightness: 3.6,
            blend: 1.0,
        },
        speed: Vec3::new(-0.004, -0.01, 0.0),
        color_tint: Vec3::new(1.0, 0.783, 0.57),
        time: 0.0,
    };
    let light_shaft_material = light_shaft_materials.add(LightShaftMaterial {
        noise_texture: Some(variation_texture.clone()),
        material_properties: light_shaft_material_props,
    });
    let light_shaft_model = &model_assets.level1_light_shafts;
    commands
        .spawn()
        .insert_bundle(MaterialMeshBundle {
            mesh: light_shaft_model.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            material: light_shaft_material.clone(),
            ..Default::default()
        })
        .insert(LevelAsset::LightShaftMaterial {
            properties: light_shaft_material_props,
            handle: light_shaft_material.clone(),
        });
    commands
        .spawn()
        .insert_bundle(MaterialMeshBundle {
            mesh: light_shaft_model.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(-1.0, -1.0, -1.0)),
            material: light_shaft_material.clone(),
            ..Default::default()
        })
        .insert(LevelAsset::LightShaftMaterial {
            properties: light_shaft_material_props,
            handle: light_shaft_material,
        });

    for (model, lightbake) in [
        (
            model_assets.level1_pillars.clone(),
            image_assets.level1_pillars.clone(),
        ),
        (
            model_assets.level1_spheres.clone(),
            image_assets.level1_spheres.clone(),
        ),
        (
            model_assets.level1_large_ceiling_supports.clone(),
            image_assets.level1_large_ceiling_supports.clone(),
        ),
        (
            model_assets.level1_walls.clone(),
            image_assets.level1_walls.clone(),
        ),
        (
            model_assets.level1_spheres_base.clone(),
            image_assets.level1_spheres_base.clone(),
        ),
    ] {
        let material = custom_materials.add(CustomMaterial {
            material_properties,
            textures: [
                lightbake,
                base_texture.clone(),
                variation_texture.clone(),
                reflection_texture.clone(),
                walls_texture.clone(),
            ],
        });

        let mesh = mesh_assets.get(model.clone()).unwrap();

        let vertices: Vec<Point3<Real>> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match &vertex_values {
                VertexAttributeValues::Float32x3(positions) => positions
                    .iter()
                    .map(|[x, y, z]| Point3::new(*x, *y, *z))
                    .collect(),
                _ => panic!("Unexpected types in {:?}", Mesh::ATTRIBUTE_POSITION),
            },
        };

        let indices: Vec<_> = match mesh.indices().unwrap() {
            Indices::U16(_) => {
                panic!("expected u32 indices");
            }
            Indices::U32(indices) => indices
                .chunks(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect(),
        };

        commands
            .spawn()
            .insert_bundle(MaterialMeshBundle {
                mesh: model,
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: material.clone(),
                ..Default::default()
            })
            .insert(RigidBody::Static)
            .insert(CollisionShape::Custom {
                shape: CustomCollisionShape::new(ColliderBuilder::trimesh(vertices, indices)),
            })
            .insert(LevelAsset::CustomMaterial {
                properties: material_properties,
                handle: material,
            })
            .insert(CollisionLayers::all::<Layer>().with_group(Layer::World));
    }

    //Bevy Sun
    let size: f32 = 100.0;
    let sun_rot_x = -67.0f32;
    let sun_rot_y = 22.0f32;
    //8.0f32;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -size * 4.0,
                right: size * 2.0,
                bottom: -size * 2.0,
                top: size * 2.0,
                near: -size * 2.0,
                far: size * 1.0,
                ..Default::default()
            },
            illuminance: 100000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                (sun_rot_y).to_radians(),
                -(sun_rot_x - 180.0f32).to_radians(),
                0.0,
            ),
            ..Default::default()
        },
        ..Default::default()
    });

    //Sky Light for Bevy PBR
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(0.0, 5.0, -200.0),
        point_light: PointLight {
            intensity: 1000000.0,
            range: 1000.0,
            radius: 100.0,
            color: Color::rgb(0.5, 0.45, 0.65),
            shadows_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    });

    // -----------------
    // --- UNIT TEST ---
    // -----------------

    let unit1 = model_assets.unit1.clone();
    commands
        .spawn_bundle((
            Transform::from_xyz(0.0, 18.0, -100.0),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(unit1.clone());
        });
    commands
        .spawn_bundle((
            Transform::from_xyz(5.0, 18.0, -100.0),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(unit1.clone());
        });
    commands
        .spawn_bundle((
            Transform::from_xyz(-5.0, 18.0, -100.0),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(unit1.clone());
        });

    let unit2 = model_assets.unit2.clone();
    commands
        .spawn_bundle((
            Transform::from_xyz(0.0, 22.0, -120.0),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(unit2.clone());
        });
    commands
        .spawn_bundle((
            Transform::from_xyz(8.0, 22.0, -120.0),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(unit2.clone());
        });
    commands
        .spawn_bundle((
            Transform::from_xyz(-8.0, 22.0, -120.0),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(unit2.clone());
        });
}
