use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    window::close_on_esc,
    prelude::*,
};

mod camera;
mod datafile;
mod sky;
mod terrain;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(camera::CameraPlugin::default())
        .add_plugin(sky::SkyPlugin::default())
        .add_startup_system(terrain::load_terrain)
        .add_system(show_fps)
        .add_system(close_on_esc)
        .run();
}

fn show_fps(diagnostics: Res<Diagnostics>, mut windows: Query<&mut Window>) {
    let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS)
    else { return; };

    let Some(value) = fps.smoothed()
    else { return; };

    let mut window = windows.single_mut();
    window.title = format!("FPS: {}", value);
}
