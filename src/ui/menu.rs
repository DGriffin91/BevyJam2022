use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{
    assets::{
        custom_material::CustomMaterial, light_shaft_material::LightShaftMaterial, GameState,
    },
    world::LevelAsset,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Playing).with_system(menu_ui));
    }
}

fn menu_ui(
    mut windows: ResMut<Windows>,
    //mut game_setup: ResMut<GameSetup>,
    //mut scoreboard: EventWriter<ScoreboardEvent>,
    mut egui_context: ResMut<EguiContext>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut light_shaft_materials: ResMut<Assets<LightShaftMaterial>>,
    mut level_asset_query: Query<&mut LevelAsset>,
) {
    let window = windows.get_primary_mut().unwrap();
    if window.is_focused() && !window.cursor_locked() {
        egui::Window::new("environment materials").show(egui_context.ctx_mut(), |ui| {
            let mut mat_props = None;
            let mut shaft_props = None;
            for mut level_asset in level_asset_query.iter_mut() {
                match level_asset.as_mut() {
                    LevelAsset::CustomMaterial {
                        ref mut properties,
                        handle,
                    } => {
                        if let Some(mat_props) = mat_props {
                            *properties = mat_props;
                            if let Some(mat) = custom_materials.get_mut(handle.clone()) {
                                mat.material_properties = mat_props;
                            }
                        } else {
                            ui.collapsing("material properties", |ui| {
                                properties.build_ui(ui);
                            });
                            mat_props = Some(*properties);
                        }
                    }
                    LevelAsset::LightShaftMaterial { properties, handle } => {
                        if let Some(shaft_props) = shaft_props {
                            *properties = shaft_props;
                            if let Some(mat) = light_shaft_materials.get_mut(handle.clone()) {
                                mat.material_properties = shaft_props;
                            }
                        } else {
                            ui.collapsing("light shaft properties", |ui| {
                                properties.build_ui(ui);
                            });
                            shaft_props = Some(*properties);
                        }
                    }
                };
            }
            //if let Some(mut level_asset) = level_asset_query.iter_mut().next() {
            //    let mat_props = {
            //        ui.collapsing("material properties", |ui| {
            //            main.material_properties.build_ui(ui);
            //        });
            //        main.material_properties
            //    };
            //    for mat in level_asset_query.iter_mut() {
            //        if let Some(mat) = custom_materials.get_mut(&mat.material_handle) {
            //            mat.material_properties = mat_props;
            //            //ui.collapsing("main material", |ui| {
            //            //    mat.build_ui(ui, &asset_server);
            //            //});
            //        }
            //    }
            //}
        });
        /*
        egui::Window::new("Setup")
            .current_pos((10.0, 60.0))
            .show(egui_context.ctx_mut(), |ui| {
                if ui.button("Start").clicked() {
                    window.set_cursor_lock_mode(true);
                    window.set_cursor_visibility(false);
                    scoreboard.send(ScoreboardEvent::Reset);
                }
                ui.label("Game Settings");
                ui.add(
                    egui::Slider::new(&mut game_setup.gun_damage, 0.05..=1.0).text("Gun Damage"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.min_targets, 0..=10).text("Minimum Targets"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.target_size, 0.1..=2.0).text("Target Size"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.target_move_speed, 0.0..=5.0)
                        .text("Target Move Speed"),
                );
                ui.add(egui::Checkbox::new(
                    &mut game_setup.target_jump,
                    "Targets Jump",
                ));
                ui.add(egui::Checkbox::new(
                    &mut game_setup.target_bounce,
                    "Targets Bounce",
                ));

                ui.label("Cube Color");
                ui.color_edit_button_rgb(&mut game_setup.target_color);
                ui.label("Target Spawn Area");
                ui.add(
                    egui::Slider::new(&mut game_setup.target_spawn_height, 0.1..=7.0)
                        .text("Height"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.target_spawn_depth, 0.1..=7.0).text("Depth"),
                );
                ui.add(
                    egui::Slider::new(&mut game_setup.target_spawn_width, 0.1..=7.0).text("Width"),
                );
                ui.label("Fire Settings");
                ui.label("Fire Rate (minimum time between shots)");
                ui.add(egui::Slider::new(&mut game_setup.fire_rate, 0.0..=1.0).text(""));
                ui.add(egui::Checkbox::new(
                    &mut game_setup.trace_mode,
                    "Trace Mode",
                ));
                ui.label("Input Settings");
                // ui.add(
                //     egui::Slider::new(&mut movement_settings.sensitivity, 0.001..=0.1)
                //         .text("Mouse Sensitivity"),
                // );
                // ui.add(
                //     egui::Slider::new(&mut movement_settings.speed, 0.1..=100.0).text("Move Speed"),
                // );
            });*/
    }
}
