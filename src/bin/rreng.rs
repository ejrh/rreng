use bevy::app::{App, Startup, Update};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::{NextState, ResMut};

use rreng::{tools, utils, RrengPlugin};
use rreng::screens::Screen;

fn main() {
    let mut app = App::new();

    app.add_plugins(RrengPlugin);
    app
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(tools::ToolsPlugin)
        .add_systems(Update, utils::show_fps);

    app.add_systems(Startup, load_title_screen);

    app.run();
}

fn load_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
