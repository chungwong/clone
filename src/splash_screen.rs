use bevy::prelude::*;

use crate::{asset::FontAssets, state::*};

#[derive(Deref, DerefMut, Resource)]
struct SplashTimer(Timer);

pub(crate) struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::Splash, splash_setup)
            .add_system(countdown.run_in_state(AppState::Splash));
    }
}

fn splash_setup(mut cmd: Commands, font_assets: Res<FontAssets>) {
    cmd.spawn(Camera2dBundle::default());

    let font = font_assets.monogram.clone();

    let text_style = TextStyle {
        font,
        font_size: 60.0,
        color: Color::WHITE,
    };

    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };

    cmd.spawn(Text2dBundle {
        text: Text::from_section("Splash Screen", text_style).with_alignment(text_alignment),
        ..default()
    });

    cmd.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));
}

fn countdown(mut cmd: Commands, time: Res<Time>, mut timer: ResMut<SplashTimer>) {
    if timer.tick(time.delta()).finished() {
        cmd.insert_resource(NextState(AppState::MainMenuAssetLoading));
    }
}
