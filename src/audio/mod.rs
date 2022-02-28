use bevy::{prelude::*, utils::HashMap};
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use rand::prelude::SliceRandom;

use crate::{
    assets::{AudioAssets, GameState},
    player::PlayerEvent,
};

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .insert_resource(AudioState::default())
            .insert_resource(EnvironmentAudio::default())
            .add_system_set(
                SystemSet::on_enter(GameState::Playing).with_system(setup_audio_channels),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(fade_in_atmosphere)
                    .with_system(player_audio_events),
            );
    }
}

fn setup_audio_channels(
    mut audio_state: ResMut<AudioState>,
    mut env_audio: ResMut<EnvironmentAudio>,
) {
    let atmosphere = AudioChannel::new("atmosphere".to_owned());
    env_audio.atmosphere = Some(atmosphere.clone());
    audio_state.channels.insert(
        atmosphere,
        ChannelAudioState {
            volume: 0.0,
            stopped: true,
            loop_started: false,
            paused: false,
            pan: 0.0,
            time_started: 0.0,
            fade_in_time: 30.0,
            final_volume: db_to_lin(-6.0),
        },
    );
}

fn fade_in_atmosphere(
    time: Res<Time>,
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    audio_assets: Res<AudioAssets>,
    env_audio: Res<EnvironmentAudio>,
) {
    if let Some(atmosphere_ch) = &env_audio.atmosphere {
        if let Some(atmosphere_state) = audio_state.channels.get_mut(atmosphere_ch) {
            if atmosphere_state.stopped {
                audio.play_looped_in_channel(audio_assets.atmosphere.clone(), atmosphere_ch);
                atmosphere_state.time_started = time.time_since_startup().as_secs_f32();
                atmosphere_state.stopped = false;
            }
            let mut level = from_range(
                0.0,
                atmosphere_state.fade_in_time,
                time.time_since_startup().as_secs_f32() - atmosphere_state.time_started,
            )
            .clamp(0.0, 1.0);
            level *= db_to_lin(atmosphere_state.final_volume);
            atmosphere_state.volume = level;
            audio.set_volume_in_channel(atmosphere_state.volume, atmosphere_ch);
        }
    }
}

fn player_audio_events(
    mut player_events: EventReader<PlayerEvent>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    for player_event in player_events.iter() {
        match player_event {
            PlayerEvent::Hit => {
                let hurt_audio = audio_assets
                    .hurt
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .clone()
                    .typed::<AudioSource>();
                audio.play(hurt_audio);
            }
            PlayerEvent::Fire => {
                let fire_audio = audio_assets
                    .lasergun
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .clone()
                    .typed::<AudioSource>();
                audio.play(fire_audio);
            }
        }
    }
}

#[allow(dead_code)]
pub fn db_to_lin(decibels: f32) -> f32 {
    (10.0f32).powf(decibels * 0.05)
}

#[allow(dead_code)]
pub fn lin_to_db(gain: f32) -> f32 {
    gain.max(0.0).log(10.0) * 20.0
}

#[allow(dead_code)]
pub fn to_range(bottom: f32, top: f32, x: f32) -> f32 {
    x * (top - bottom) + bottom
}

#[allow(dead_code)]
pub fn from_range(bottom: f32, top: f32, x: f32) -> f32 {
    (x - bottom) / (top - bottom)
}

#[allow(dead_code)]
struct ChannelAudioState {
    stopped: bool,
    paused: bool,
    loop_started: bool,
    volume: f32,
    pan: f32,
    time_started: f32,
    fade_in_time: f32,
    final_volume: f32,
}

impl Default for ChannelAudioState {
    fn default() -> Self {
        ChannelAudioState {
            volume: 1.0,
            stopped: true,
            loop_started: false,
            paused: false,
            pan: 0.0,
            time_started: 0.0,
            fade_in_time: 0.0,
            final_volume: 1.0,
        }
    }
}

#[derive(Default)]
struct AudioState {
    pub channels: HashMap<AudioChannel, ChannelAudioState>,
}

#[derive(Default)]
struct EnvironmentAudio {
    pub atmosphere: Option<AudioChannel>,
}

pub struct GameAudioPlugin;
