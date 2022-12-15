use std::marker::PhantomData;

use bevy::{ecs::system::Resource, prelude::*};
use bevy_kira_audio::{
    AudioApp, AudioChannel, AudioControl, AudioInstance, AudioPlugin as KiraAudioPlugin,
};

pub(crate) type AudioSource = bevy_kira_audio::AudioSource;

use crate::{
    asset::{AudioAssets, MainMenuAssets},
    state::{AppLooplessStateExt, AppState, CurrentState, NextState},
    ui::menu::GameConfig,
};

trait Channel = Sync + Send + Resource;

#[derive(Resource)]
pub(crate) struct ChannelState<T> {
    pub(crate) handle: Option<Handle<AudioSource>>,
    pub(crate) instance_handle: Option<Handle<AudioInstance>>,
    pub(crate) looped: bool,
    pub(crate) paused: bool,
    pub(crate) resumed: bool,
    pub(crate) stopped: bool,
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
            _marker: PhantomData,
        }
    }
}

#[derive(Component, Debug, Default, Clone, Resource)]
pub(crate) struct MusicChannel;

#[derive(Component, Debug, Default, Clone, Resource)]
pub(crate) struct EffectsChannel;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Resource)]
enum AudioState {
    #[default]
    None,
    MainMenu,
    InGame,
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(KiraAudioPlugin)
            .add_loopless_state(AudioState::default())
            .insert_resource(ChannelState::<MusicChannel>::default())
            .insert_resource(ChannelState::<EffectsChannel>::default())
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<EffectsChannel>()
            .add_enter_system(AppState::MainMenu, update_main_menu_audio_state)
            .add_enter_system(AppState::InGame, update_in_game_audio_state)
            .add_enter_system(AudioState::MainMenu, play_menu_music)
            .add_enter_system(AudioState::InGame, play_game_music)
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
}

fn update_main_menu_audio_state(mut cmd: Commands, audio_state: Res<CurrentState<AudioState>>) {
    if audio_state.0 != AudioState::MainMenu {
        cmd.insert_resource(NextState(AudioState::MainMenu));
    }
}

fn update_in_game_audio_state(mut cmd: Commands, audio_state: Res<CurrentState<AudioState>>) {
    if audio_state.0 != AudioState::InGame {
        cmd.insert_resource(NextState(AudioState::InGame));
    }
}

fn play_menu_music(
    mut channel_state: ResMut<ChannelState<MusicChannel>>,
    main_menu_assets: Res<MainMenuAssets>,
    audio: Res<AudioChannel<MusicChannel>>,
    game_config: Res<GameConfig>,
) {
    audio.stop();
    audio.set_volume(*game_config.audio.music_volume);
    channel_state.reset();
    channel_state.handle = Some(main_menu_assets.bgm.clone());
    channel_state.stopped = true;
    channel_state.looped = true;
}

fn play_game_music(
    mut channel_state: ResMut<ChannelState<MusicChannel>>,
    audio_assets: Res<AudioAssets>,
    audio: Res<AudioChannel<MusicChannel>>,
    game_config: Res<GameConfig>,
) {
    audio.stop();
    audio.set_volume(*game_config.audio.music_volume);
    channel_state.reset();
    channel_state.handle = Some(audio_assets.bgm.clone());
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
