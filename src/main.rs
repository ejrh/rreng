use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    window::close_on_esc,
    prelude::*,
};
use bevy::diagnostic::{DiagnosticsStore, LogDiagnosticsPlugin};

mod camera;
mod datafile;
mod sky;
mod terrain;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(camera::CameraPlugin::default())
        .add_plugins(sky::SkyPlugin::default())
        .add_systems(Startup, terrain::load_terrain)
        .add_systems(Update, show_fps)
        .add_systems(Update, close_on_esc)
        .run();
}

fn show_fps(diagnostics: Res<DiagnosticsStore>, mut windows: Query<&mut Window>) {
    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
    else { return; };

    let Some(value) = fps.smoothed()
    else { return; };

    let mut window = windows.single_mut();
    window.title = format!("FPS: {}", value);
}
