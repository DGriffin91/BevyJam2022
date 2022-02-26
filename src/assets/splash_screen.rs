use bevy::prelude::*;

use super::GameState;

pub struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Loading).with_system(show_splash_screen))
            .add_system_set(SystemSet::on_exit(GameState::Loading).with_system(hide_splash_screen));
    }
}

#[derive(Component)]
struct Overlay;

fn show_splash_screen(mut commands: Commands, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(window.width()), Val::Px(window.height())),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: Color::BLACK.into(),
            ..Default::default()
        })
        .insert(Overlay);
}

fn hide_splash_screen(mut commands: Commands, overlays: Query<Entity, With<Overlay>>) {
    for overlay in overlays.iter() {
        commands.entity(overlay).despawn();
    }
}
