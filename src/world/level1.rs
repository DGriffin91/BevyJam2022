use bevy::{
    math::{EulerRot, Quat, Vec3},
    pbr::{DirectionalLight, DirectionalLightBundle, MaterialMeshBundle},
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use bevy_asset_loader::{AssetKeys, DynamicAsset};
use heron::{
    rapier_plugin::{
        nalgebra::Point3,
        rapier3d::{math::Real, prelude::ColliderBuilder},
    },
    CollisionLayers, CollisionShape, CustomCollisionShape, PhysicMaterial, PhysicsLayer, RigidBody,
};

use crate::{
    assets::{
        custom_material::CustomMaterialFlags,
        emissive_material::EmissiveMaterial,
        light_shaft_material::{
            update_light_shaft_material_time, LightShaftMaterial, LightShaftProperties,
        },
        ModelAssets,
    },
    assets::{
        custom_material::{CustomMaterial, MaterialProperties, MaterialSetProp},
        GameState, ImageAssets,
    },
    ui::menu::GamePreferences,
    Layer,
};

use super::LevelAsset;

pub struct LevelOnePlugin;

impl Plugin for LevelOnePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(setup_level_one)
                .with_system(spawn_demo_cubes)
                .with_system(spawn_waypoints),
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
            })
            .insert(
                CollisionLayers::none()
                    .with_group(Layer::World)
                    .with_masks(Layer::all()),
            );
    }
}

pub fn set_textures_res(mut asset_keys: ResMut<AssetKeys>, high_res: bool) {
    let sm = if high_res { "" } else { "sm/" };
    for s in [
        "large_ceiling_supports",
        "pillars",
        "sky_box",
        "spheres_base",
        "spheres",
        "walls",
    ] {
        asset_keys.register_asset(
            &format!("level1_{}", s),
            DynamicAsset::File {
                path: format!("textures/level1/bake/{}{}.jpg", sm, s),
            },
        );
    }
    for s in ["concrete", "concrete3", "detail", "reflection"] {
        asset_keys.register_asset(
            s,
            DynamicAsset::File {
                path: format!("textures/{}{}.jpg", sm, s),
            },
        );
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
    preferences: Res<GamePreferences>,
) {
    let variation_texture = image_assets.detail.clone();
    let base_texture = image_assets.concrete.clone();
    let walls_texture = image_assets.concrete3.clone();
    let reflection_texture = image_assets.reflection.clone();

    let mut flags = CustomMaterialFlags::NONE;
    if preferences.dynamic_shadows {
        flags |= CustomMaterialFlags::SHADOWS;
    }
    if preferences.potato {
        flags |= CustomMaterialFlags::POTATO;
    }
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
        flags: flags.bits(),
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
    if preferences.light_shafts {
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
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_scale(Vec3::new(-1.0, -1.0, -1.0)),
                material: light_shaft_material.clone(),
                ..Default::default()
            })
            .insert(LevelAsset::LightShaftMaterial {
                properties: light_shaft_material_props,
                handle: light_shaft_material,
            });
    }

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
            .insert(
                CollisionLayers::none()
                    .with_group(Layer::World)
                    .with_masks(Layer::all()),
            );
    }
    if preferences.dynamic_shadows {
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
    }

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
}

fn spawn_waypoints(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //--- WAYPOINTS ---
    //--- inside ---
    let waypoints = [
        Vec3::new(-10.0, 36.4119, -210.641),
        Vec3::new(10.0, 36.4119, -210.641),
        Vec3::new(32.0, 36.4119, -211.335),
        Vec3::new(-32.0, 36.4119, -210.988),
        Vec3::new(-10.0, 36.4119, -170.228),
        Vec3::new(10.0, 36.4119, -169.361),
        Vec3::new(32.0, 36.4119, -170.055),
        Vec3::new(-32.0, 36.4119, -169.708),
        Vec3::new(-10.0, 36.4119, -129.468),
        Vec3::new(10.0, 36.4119, -129.468),
        Vec3::new(32.0, 36.4119, -130.162),
        Vec3::new(-32.0, 36.4119, -129.815),
        Vec3::new(-10.0, 36.4119, -89.2278),
        Vec3::new(10.0, 36.4119, -89.2278),
        Vec3::new(32.0, 36.4119, -89.9216),
        Vec3::new(-32.0, 36.4119, -89.5747),
        Vec3::new(-10.0, 36.4119, -50.0285),
        Vec3::new(10.0, 36.4119, -50.0285),
        Vec3::new(32.0, 36.4119, -50.7223),
        Vec3::new(-32.0, 36.4119, -50.3754),
        Vec3::new(-10.0, 36.4119, -9.44168),
        Vec3::new(10.0, 36.4119, -9.44168),
        Vec3::new(32.0, 36.4119, -10.1355),
        Vec3::new(-32.0, 36.4119, -9.78857),
        Vec3::new(-10.0, 36.4119, 29.0638),
        Vec3::new(10.0, 36.4119, 29.0638),
        Vec3::new(32.0, 36.4119, 28.37),
        Vec3::new(-32.0, 36.4119, 28.7169),
        Vec3::new(-10.0, 36.4119, 70.3444),
        Vec3::new(10.0, 36.4119, 70.3444),
        Vec3::new(32.0, 36.4119, 69.6506),
        Vec3::new(-32.0, 36.4119, 69.9975),
        Vec3::new(-10.0, 36.4119, 109.891),
        Vec3::new(10.0, 36.4119, 109.891),
        Vec3::new(32.0, 36.4119, 109.197),
        Vec3::new(-32.0, 36.4119, 109.544),
        Vec3::new(10.0, 36.4119, -190.175),
        Vec3::new(-10.0, 36.4119, -190.521),
        Vec3::new(10.0, 36.4119, -150.281),
        Vec3::new(-10.0, 36.4119, -150.628),
        Vec3::new(10.0, 36.4119, -110.042),
        Vec3::new(-10.0, 36.4119, -110.388),
        Vec3::new(10.0, 36.4119, -69.8016),
        Vec3::new(-10.0, 36.4119, -70.1485),
        Vec3::new(10.0, 36.4119, -29.5616),
        Vec3::new(-10.0, 36.4119, -29.9085),
        Vec3::new(10.0, 36.4119, 9.63762),
        Vec3::new(-10.0, 36.4119, 9.29073),
        Vec3::new(10.0, 36.4119, 50.2245),
        Vec3::new(-10.0, 36.4119, 49.8776),
        Vec3::new(10.0, 36.4119, 90.1175),
        Vec3::new(-10.0, 36.4119, 89.7706),
        Vec3::new(-10.0, 36.4119, -230.636),
        Vec3::new(10.0, 36.4119, -230.636),
        Vec3::new(32.0, 36.4119, -231.33),
        Vec3::new(-32.0, 36.4119, -230.983),
        Vec3::new(-10.0, 36.4119, -250.39),
        Vec3::new(10.0, 36.4119, -250.39),
        Vec3::new(32.0, 36.4119, -251.084),
        Vec3::new(-32.0, 36.4119, -250.737),
        //--- outside ---
        Vec3::new(-70.0, 36.4119, -130.231),
        Vec3::new(-70.0, 36.4119, -169.869),
        Vec3::new(-70.0, 36.4119, -90.3362),
        Vec3::new(-70.0, 36.4119, -50.1839),
        Vec3::new(-70.0, 36.4119, 30.6253),
        Vec3::new(-70.0, 36.4119, -9.01227),
        Vec3::new(70.0, 36.4119, -169.869),
        Vec3::new(70.0, 36.4119, -90.3362),
        Vec3::new(70.0, 36.4119, -50.1839),
        Vec3::new(70.0, 36.4119, 30.6253),
        Vec3::new(70.0, 36.4119, -9.01227),
        Vec3::new(70.0, 36.4119, 70.5203),
        Vec3::new(-70.0, 36.4119, 70.5203),
        Vec3::new(70.0, 36.4119, -130.231),
        //--- window ---
        Vec3::new(49.4907, 36.4119, -129.961),
        Vec3::new(49.4907, 36.4119, -170.024),
        Vec3::new(49.4907, 36.4119, -90.012),
        Vec3::new(49.7481, 36.4119, -49.9812),
        Vec3::new(49.4907, 36.4119, 30.0028),
        Vec3::new(49.4907, 36.4119, -10.0627),
        Vec3::new(49.4907, 36.4119, 69.9123),
        Vec3::new(-49.4163, 36.4119, -129.983),
        Vec3::new(-49.4163, 36.4119, -170.102),
        Vec3::new(-49.4163, 36.4119, -89.9796),
        Vec3::new(-49.4291, 36.4119, -49.9677),
        Vec3::new(-49.4163, 36.4119, 30.0093),
        Vec3::new(-49.4163, 36.4119, -10.0394),
        Vec3::new(-49.4163, 36.4119, 70.0339),
        //--- out_front ---
        Vec3::new(-91.3197, 36.4119, -323.804),
        Vec3::new(74.844, 36.4119, -325.243),
        Vec3::new(-10.0, 36.4119, -272.966),
        Vec3::new(10.0, 36.4119, -272.966),
        Vec3::new(101.775, 36.4119, -351.138),
        Vec3::new(-114.108, 36.4119, -349.7),
    ];

    let mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.5,
        sectors: 8,
        stacks: 8,
    }));
    let material = materials.add(StandardMaterial {
        base_color: Color::BLUE,
        ..Default::default()
    });

    for pos in waypoints {
        commands.spawn_bundle(PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(pos.x, pos.y, pos.z),
            ..Default::default()
        });
    }
}
