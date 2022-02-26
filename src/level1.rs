use bevy::{
    math::{EulerRot, Quat, Vec3},
    pbr::{
        DirectionalLight, DirectionalLightBundle, MaterialMeshBundle, PointLight, PointLightBundle,
    },
    prelude::{
        AssetServer, Assets, Color, Commands, OrthographicProjection, Res, ResMut, Transform,
    },
};
use bevy_mod_raycast::{DefaultPluginState, RayCastMesh};

use crate::{
    custom_material::{CustomMaterial, MaterialProperties, MaterialSetProp, MaterialTexture},
    emissive_material::EmissiveMaterial,
    LevelAsset, MyRaycastSet,
};

pub fn setup(
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut emissive_materials: ResMut<Assets<EmissiveMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(DefaultPluginState::<MyRaycastSet>::default()); //.with_debug_cursor()

    let variation_texture =
        MaterialTexture::new(&asset_server, "textures/detail.jpg", "variation_texture");
    let base_texture = MaterialTexture::new(&asset_server, "textures/concrete.jpg", "base_texture");

    let walls_texture =
        MaterialTexture::new(&asset_server, "textures/concrete3.jpg", "walls_texture");

    let reflection_texture = MaterialTexture::new(
        &asset_server,
        "textures/reflection.jpg",
        "reflection_texture",
    );

    let material_properties = MaterialProperties {
        lightmap: MaterialSetProp {
            scale: 1.0,
            contrast: 2.3,
            brightness: 3.1,
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
            scale: 24.0,
            contrast: 0.14,
            brightness: 1.05,
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
            scale: 0.032,
            contrast: 1.0,
            brightness: 1.0,
            blend: 0.0,
        },
        directional_light_blend: 0.6,
    };
    let model = asset_server.load("models/level1/sky_box.gltf#Mesh0/Primitive0");
    let skybox_texture = asset_server.load("textures/level1/bake/sky_box.jpg");
    commands.spawn().insert_bundle(MaterialMeshBundle {
        mesh: model,
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(10.0, 10.0, 10.0)),
        material: emissive_materials.add(EmissiveMaterial {
            emissive: Color::WHITE,
            emissive_texture: Some(skybox_texture),
        }),
        ..Default::default()
    });

    for name in [
        "pillars",
        "spheres",
        "large_ceiling_supports",
        "walls",
        "spheres_base",
    ] {
        let model = asset_server.load(&format!("models/level1/{}.gltf#Mesh0/Primitive0", name));
        let lightbake = MaterialTexture::new(
            &asset_server,
            &format!("textures/level1/bake/{}.jpg", name),
            name,
        );
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
        commands
            .spawn()
            .insert_bundle(MaterialMeshBundle {
                mesh: model,
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: material.clone(),
                ..Default::default()
            })
            .insert(RayCastMesh::<MyRaycastSet>::default())
            .insert(LevelAsset {
                material_properties,
                material_handle: material,
            });
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

    //Sky Light for PBR
    //commands.spawn_bundle(PointLightBundle {
    //    transform: Transform::from_xyz(0.0, 5.0, 100.0),
    //    point_light: PointLight {
    //        intensity: 30000.0,
    //        range: 1000.0,
    //        radius: 30.0,
    //        color: Color::rgb(0.3, 0.25, 1.0),
    //        shadows_enabled: false,
    //        ..Default::default()
    //    },
    //    ..Default::default()
    //});

    // Only doing a couple light positions because Bevy complains:
    // WARN bevy_pbr::render::light: Cluster light index lists is full!
    // The PointLights in the view are affecting too many clusters.
    //let lamp_locations = [
    //    Vec3::new(-15.0, 17.0, -16.0),
    //    Vec3::new(-10.0, 17.0, -16.0),
    //    Vec3::new(-10.0, 17.0, -16.0),
    //    Vec3::new(-5.0, 17.0, -16.0),
    //    Vec3::new(-5.0, 17.0, -16.0),
    //    Vec3::new(0.0, 17.0, -16.0),
    //    Vec3::new(5.0, 17.0, -16.0),
    //    Vec3::new(10.0, 17.0, -16.0),
    //    Vec3::new(15.0, 17.0, -16.0),
    //];
    //let intensity = 1000.0;
    //dbg!(f32::sqrt(intensity * 10.0 / (4.0 * std::f32::consts::PI)));
    //for lamp_loc in lamp_locations {
    //    commands.spawn_bundle(PointLightBundle {
    //        transform: Transform::from_xyz(lamp_loc.x, lamp_loc.y, lamp_loc.z),
    //        point_light: PointLight {
    //            intensity,
    //            range: f32::sqrt(intensity * 10.0 / (4.0 * std::f32::consts::PI)),
    //            radius: 10.0, //Oversize since we only have 2
    //            color: Color::rgb(1.0, 1.0, 1.0),
    //            shadows_enabled: false,
    //            ..Default::default()
    //        },
    //        ..Default::default()
    //    });
    //}

    // Tell the asset server to watch for asset changes on disk:
    asset_server.watch_for_changes().unwrap();
}
