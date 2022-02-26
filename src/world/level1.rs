use bevy::{
    math::{EulerRot, Quat, Vec3},
    pbr::{DirectionalLight, DirectionalLightBundle, MaterialMeshBundle},
    prelude::*,
};

use crate::{
    assets::{
        custom_material::{CustomMaterial, MaterialProperties, MaterialSetProp},
        GameState, ImageAssets,
    },
    assets::{emissive_material::EmissiveMaterial, ModelAssets},
};

use super::LevelAsset;

pub struct LevelOnePlugin;

impl Plugin for LevelOnePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup_level_one));
    }
}

fn setup_level_one(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    model_assets: Res<ModelAssets>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut emissive_materials: ResMut<Assets<EmissiveMaterial>>,
) {
    let variation_texture = image_assets.detail.clone();
    let base_texture = image_assets.concrete.clone();
    let walls_texture = image_assets.concrete3.clone();
    let reflection_texture = image_assets.reflection.clone();

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

        commands
            .spawn()
            .insert_bundle(MaterialMeshBundle {
                mesh: model,
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: material.clone(),
                ..Default::default()
            })
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
    // asset_server.watch_for_changes().unwrap();
}
