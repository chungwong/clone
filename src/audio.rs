use std::marker::PhantomData;

use bevy::{ecs::system::Resource, prelude::*};
use bevy_kira_audio::{
    AudioApp, AudioChannel, AudioControl, AudioInstance, AudioPlugin as KiraAudioPlugin,
    AudioSource,
};

use crate::state::{AppLooplessStateExt, GameState};

trait Channel = Sync + Send + Resource;

pub(crate) struct ChannelState<T> {
    pub(crate) handle: Option<Handle<AudioSource>>,
    pub(crate) instance_handle: Option<Handle<AudioInstance>>,
    pub(crate) looped: bool,
    pub(crate) paused: bool,
    pub(crate) resumed: bool,
    pub(crate) stopped: bool,
    pub(crate) volume: f64,
    _marker: PhantomData<T>,
}

impl<T> ChannelState<T> {
    fn reset(&mut self) {
        self.stopped = false;
        self.looped = false;
        self.paused = false;
        self.resumed = false;
    }
}

impl<T> Default for ChannelState<T> {
    fn default() -> Self {
        Self {
            handle: None,
            instance_handle: None,
            looped: false,
            resumed: false,
            paused: false,
            stopped: false,
            volume: 1.0,
            _marker: PhantomData,
        }
    }
}

#[derive(Component, Debug, Default, Clone)]
pub(crate) struct MusicChannel;

#[derive(Component, Debug, Default, Clone)]
pub(crate) struct EffectsChannel;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(KiraAudioPlugin)
            .insert_resource(ChannelState::<MusicChannel>::default())
            .insert_resource(ChannelState::<EffectsChannel>::default())
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<EffectsChannel>()
            .add_enter_system(GameState::MainMenu, play_menu_music)
            .add_enter_system(GameState::InGame, play_game_music)
            .add_system_set(setup_controls::<MusicChannel>())
            .add_system_set(setup_controls::<EffectsChannel>());
    }
}

fn setup_controls<T: Channel>() -> SystemSet {
    SystemSet::new()
        .with_system(play_channel::<T>)
        .with_system(pause_channel::<T>)
        .with_system(resume_channel::<T>)
        .with_system(stop_channel::<T>)
        .with_system(set_channel_volume::<T>)
}

#[allow(clippy::only_used_in_recursion)]
fn play_menu_music(
    mut channel_state: ResMut<ChannelState<MusicChannel>>,
    asset_server: Res<AssetServer>,
    audio: Res<AudioChannel<MusicChannel>>,
) {
    audio.stop();
    channel_state.reset();
    channel_state.handle = Some(asset_server.load("music/TownTheme.mp3"));
    channel_state.stopped = true;
    channel_state.looped = true;
}

#[allow(clippy::only_used_in_recursion)]
fn play_game_music(
    mut channel_state: ResMut<ChannelState<MusicChannel>>,
    asset_server: Res<AssetServer>,
    audio: Res<AudioChannel<MusicChannel>>,
) {
    audio.stop();
    channel_state.reset();
    channel_state.handle = Some(asset_server.load("music/ThemeForest.mp3"));
    channel_state.stopped = true;
    channel_state.looped = true;
}

fn play_channel<T: Channel>(
    mut channel_state: ResMut<ChannelState<T>>,
    audio: Res<AudioChannel<T>>,
) {
    if let ChannelState {
        handle: Some(handle),
        looped,
        ..
    } = &*channel_state
    {
        let instance_handle = if *looped {
            audio.play(handle.clone()).looped().handle()
        } else {
            audio.play(handle.clone()).handle()
        };

        channel_state.reset();
        channel_state.instance_handle = Some(instance_handle);
        channel_state.handle = None;
    }
}

fn pause_channel<T: Channel>(
    mut channel_state: ResMut<ChannelState<T>>,
    audio: Res<AudioChannel<T>>,
) {
    if channel_state.paused {
        audio.pause();
        channel_state.reset();
    }
}

fn resume_channel<T: Channel>(
    mut channel_state: ResMut<ChannelState<T>>,
    audio: Res<AudioChannel<T>>,
) {
    if channel_state.resumed {
        audio.resume();
        channel_state.reset();
    }
}

fn stop_channel<T: Channel>(
    mut channel_state: ResMut<ChannelState<T>>,
    audio: Res<AudioChannel<T>>,
) {
    if channel_state.stopped {
        audio.stop();
        channel_state.reset();
    }
}

fn set_channel_volume<T: Channel>(
    mut channel_state: ResMut<ChannelState<T>>,
    audio: Res<AudioChannel<T>>,
) {
    if channel_state.volume != 0.0 {
        audio.set_volume(channel_state.volume);
        channel_state.volume = 0.0;
    }
}