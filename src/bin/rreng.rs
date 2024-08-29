use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use rreng::*;

fn main() {
    let mut app = App::new();

    app
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(camera::CameraPlugin::default())
        .add_plugins(sky::SkyPlugin::default())
        .add_plugins(terrain::TerrainPlugin::default())
        .add_systems(Update, utils::show_fps)
        .add_systems(Startup, utils::show_version)
        .add_systems(Startup, utils::show_help_text)
        .add_plugins(WorldInspectorPlugin::new());


    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, utils::close_on_esc);

    app.run();
}
