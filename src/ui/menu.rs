use bevy::prelude::*;
use bevy_asset_loader::AssetKeys;
use bevy_egui::{egui, EguiContext};

use crate::{
    assets::{
        custom_material::CustomMaterial, light_shaft_material::LightShaftMaterial, GameState,
    },
    world::{level1, LevelAsset},
};

pub struct MenuPlugin;

pub struct GamePreferences {
    pub high_res_textures: bool,
    pub light_shafts: bool,
    pub dynamic_shadows: bool,
    pub potato: bool,
}

impl Default for GamePreferences {
    fn default() -> Self {
        GamePreferences {
            high_res_textures: true,
            light_shafts: true,
            dynamic_shadows: true,
            potato: false,
        }
    }
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Menu).with_system(startup_menu))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(menu_ui))
            .insert_resource(GamePreferences::default());
    }
}

fn startup_menu(
    mut state: ResMut<State<GameState>>,
    mut windows: ResMut<Windows>,
    mut preferences: ResMut<GamePreferences>,
    mut egui_context: ResMut<EguiContext>,
    asset_keys: ResMut<AssetKeys>,
) {
    let window = windows.get_primary_mut().unwrap();
    if window.is_focused() && !window.cursor_locked() {
        egui::Window::new("GAME").show(egui_context.ctx_mut(), |ui| {
            if ui.button("START").clicked() {
                level1::set_textures_res(asset_keys, preferences.high_res_textures);
                state
                    .set(GameState::Loading)
                    .expect("Failed to change state");
                window.set_cursor_position(Vec2::new(window.width() / 4.0, window.height() / 4.0));
                window.set_cursor_lock_mode(true);
                window.set_cursor_visibility(false);
            }
            ui.checkbox(
                &mut preferences.high_res_textures,
                "High resolution textures",
            );
            ui.checkbox(&mut preferences.light_shafts, "Light shafts enabled");
            ui.checkbox(&mut preferences.dynamic_shadows, "Dynamic shadows enabled");
            ui.checkbox(&mut preferences.potato, "Potato Mode");
        });
    }
}

fn menu_ui(
    mut windows: ResMut<Windows>,
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
                        if mat_props.is_none() {
                            ui.collapsing("material properties", |ui| {
                                properties.build_ui(ui);
                            });
                            mat_props = Some(*properties);
                        }
                        if let Some(mat_props) = mat_props {
                            *properties = mat_props;
                            if let Some(mat) = custom_materials.get_mut(handle.clone()) {
                                mat.material_properties = mat_props;
                            }
                        }
                    }
                    LevelAsset::LightShaftMaterial { properties, handle } => {
                        if shaft_props.is_none() {
                            ui.collapsing("light shaft properties", |ui| {
                                properties.build_ui(ui);
                            });
                            shaft_props = Some(*properties);
                        }
                        if let Some(shaft_props) = shaft_props {
                            *properties = shaft_props;
                            if let Some(mat) = light_shaft_materials.get_mut(handle.clone()) {
                                mat.material_properties = shaft_props;
                            }
                        }
                    }
                };
            }
        });
    }
}
