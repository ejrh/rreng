use bevy::app::{App, Startup, Update};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use rreng::{tools, utils, RrengPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugins(RrengPlugin);
    app
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(tools::ToolsPlugin)
        .add_systems(Update, utils::show_fps)
        .add_systems(Startup, utils::show_version)
        .add_systems(Startup, utils::show_help_text);

    app.run();
}
