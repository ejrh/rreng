use bevy::app::{App, Startup, Update};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::{in_state, Commands, IntoScheduleConfigs, Local, NextState, Res, ResMut, Time};

use rreng::{tools, utils, RrengPlugin};
use rreng::events::GameEvent;
use rreng::screens::Screen;

fn main() {
    let mut app = App::new();

    app.add_plugins(RrengPlugin);
    app
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(tools::ToolsPlugin)
        .add_systems(Update, utils::show_fps);

    app.add_systems(Startup, load_title_screen);
    app.add_systems(Update, load_after_5_seconds.run_if(in_state(Screen::Title)));

    app.run();
}

fn load_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn load_after_5_seconds(mut commands: Commands, time: Res<Time>, mut fired: Local<bool>) {
    if !*fired && time.elapsed() > std::time::Duration::from_secs(5) {
        commands.send_event(GameEvent::LoadLevel("data/jvl.ron".to_owned()));
        *fired = true;
    }
}
