use crate::state::*;
use bevy::prelude::*;

#[derive(Component)]
struct OnSplashScreen;

#[derive(Deref, DerefMut)]
struct SplashTimer(Timer);

pub(crate) struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Splash, splash_setup)
            .add_system(countdown.run_in_state(GameState::Splash))
            .add_exit_system(GameState::Splash, despawn_with::<OnSplashScreen>);
    }
}

fn splash_setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/monogram.ttf");

    let text_style = TextStyle {
        font,
        font_size: 60.0,
        color: Color::WHITE,
    };

    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };

    cmd.spawn_bundle(Text2dBundle {
        text: Text::with_section("Splash Screen", text_style, text_alignment),
        ..default()
    })
    .insert(OnSplashScreen);

    cmd.insert_resource(SplashTimer(Timer::from_seconds(1.0, false)));
}

fn countdown(mut cmd: Commands, time: Res<Time>, mut timer: ResMut<SplashTimer>) {
    if timer.tick(time.delta()).finished() {
        cmd.insert_resource(NextState(GameState::MainMenu));
    }
}
