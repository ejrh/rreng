use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    window::close_on_esc,
};

use rreng::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(camera::CameraPlugin::default())
        .add_plugins(sky::SkyPlugin::default())
        .add_plugins(terrain::TerrainPlugin::default())
        .add_systems(Update, utils::show_fps)
        .add_systems(Startup, utils::show_version)
        .add_systems(Update, close_on_esc)
        .run();
}
