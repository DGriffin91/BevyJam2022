use bevy::prelude::*;
use bevy_asset_loader::AssetKeys;
use bevy_egui::{
    egui::{self, Align2, FontDefinitions, Pos2},
    EguiContext,
};

#[cfg(debug_assertions)]
use crate::assets::{
    custom_material::CustomMaterial, light_shaft_material::LightShaftMaterial,
    orb_material::OrbMaterial,
};
#[cfg(debug_assertions)]
use crate::world::LevelAsset;

use crate::{
    assets::GameState,
    enemies::{EnemiesState, Enemy, EnemySpawnTimer},
    player::{MovementSettings, Player},
    world::level1,
};

use super::{hud::ScreenMessage, scoreboard::ScoreboardEvent};

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
            .insert_resource(GamePreferences::default())
            .add_startup_system(setup_fonts);
    }
}

pub fn setup_fonts(mut egui_context: ResMut<EguiContext>) {
    let mut fonts = FontDefinitions::default();

    for (_text_style, (_family, size)) in fonts.family_and_size.iter_mut() {
        *size = 25.0;
    }
    egui_context.ctx_mut().set_fonts(fonts);
}

fn startup_menu(
    mut state: ResMut<State<GameState>>,
    mut windows: ResMut<Windows>,
    mut preferences: ResMut<GamePreferences>,
    mut egui_context: ResMut<EguiContext>,
    asset_keys: ResMut<AssetKeys>,
    mut movement_settings: ResMut<MovementSettings>,
    keys: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();

    if window.is_focused() && !window.cursor_locked() {
        egui::Window::new("CONFLUENCE OF FUTILITY")
            .resizable(false)
            .collapsible(false)
            .current_pos([
                (window.physical_width() / 2) as f32,
                (window.physical_height() / 2) as f32,
            ])
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(egui_context.ctx_mut(), |ui| {
                ui.vertical_centered_justified(|ui| {
                    if ui.button("START").clicked() {
                        level1::set_textures_res(asset_keys, preferences.high_res_textures);
                        state
                            .set(GameState::Loading)
                            .expect("Failed to change state");
                        window.set_cursor_position(Vec2::new(
                            window.width() / 4.0,
                            window.height() / 4.0,
                        ));
                        window.set_cursor_lock_mode(true);
                        window.set_cursor_visibility(false);
                    }
                });
                ui.collapsing("Preferences", |ui| {
                    movement_settings.build_ui(ui, keys);
                    ui.checkbox(
                        &mut preferences.high_res_textures,
                        "High resolution textures",
                    );
                    ui.checkbox(&mut preferences.light_shafts, "Light shafts enabled");
                    ui.checkbox(&mut preferences.dynamic_shadows, "Dynamic shadows enabled");
                    ui.checkbox(&mut preferences.potato, "Potato Mode");
                })
            });
    }
}

fn menu_ui(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    mut egui_context: ResMut<EguiContext>,
    #[cfg(debug_assertions)] mut custom_materials: ResMut<Assets<CustomMaterial>>,
    #[cfg(debug_assertions)] mut light_shaft_materials: ResMut<Assets<LightShaftMaterial>>,
    #[cfg(debug_assertions)] mut orb_materials: ResMut<Assets<OrbMaterial>>,
    #[cfg(debug_assertions)] mut level_asset_query: Query<&mut LevelAsset>,
    mut movement_settings: ResMut<MovementSettings>,
    keys: Res<Input<KeyCode>>,
    mut players: Query<(&mut Player, &mut Transform)>,
    enemies: Query<Entity, With<Enemy>>,
    mut enemies_state: ResMut<EnemiesState>,
    mut scoreboard_events: EventWriter<ScoreboardEvent>,
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    mut screen_messages: Query<&mut ScreenMessage>,
) {
    let window = windows.get_primary_mut().unwrap();
    if window.is_focused() && !window.cursor_locked() {
        egui::Window::new("Preferences")
            .resizable(false)
            .collapsible(false)
            .default_pos(Pos2::new(10.0, 200.0))
            .show(egui_context.ctx_mut(), |ui| {
                ui.vertical_centered_justified(|ui| {
                    if ui.button("Continue").clicked() {
                        window.set_cursor_lock_mode(true);
                        window.set_cursor_visibility(false);
                    }
                    if ui.button("Restart").clicked() {
                        // TODO move elsewhere, trigger with event
                        if let Some((mut player, mut trans)) = players.iter_mut().next() {
                            player.health = player.max_health;
                            *trans = Transform::from_xyz(0.0, 3.0, 100.0);
                        }
                        for entity in enemies.iter() {
                            commands.entity(entity).despawn_recursive();
                        }
                        scoreboard_events.send(ScoreboardEvent::Reset);
                        *enemies_state = EnemiesState::default();
                        enemy_spawn_timer.0.pause();
                        for mut screen_message in screen_messages.iter_mut() {
                            *screen_message = ScreenMessage::PressFire;
                        }
                    }
                });
                movement_settings.build_ui(ui, keys);
            });

        #[cfg(debug_assertions)]
        egui::Window::new("environment materials").show(egui_context.ctx_mut(), |ui| {
            // TODO Refactor after jam
            let mut mat_props = None;
            let mut shaft_props = None;
            let mut orb_props = None;
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
                    LevelAsset::OrbMaterial { properties, handle } => {
                        if orb_props.is_none() {
                            ui.collapsing("orb properties", |ui| {
                                properties.build_ui(ui);
                            });
                            orb_props = Some(*properties);
                        }
                        if let Some(orb_props) = orb_props {
                            *properties = orb_props;
                            if let Some(mat) = orb_materials.get_mut(handle.clone()) {
                                mat.material_properties = orb_props;
                            }
                        }
                    }
                };
            }
        });
    }
}
