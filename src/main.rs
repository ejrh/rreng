use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    window::close_on_esc,
    prelude::*,
};

mod camera;
mod terrain;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(camera::CameraPlugin::default())
        .add_startup_system(create_lights)
        .add_startup_system(terrain::create_terrain)
        .add_system(show_fps)
        .add_system(close_on_esc)
        .run();
}

fn create_lights(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 2500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn show_fps(diagnostics: Res<Diagnostics>, mut windows: Query<&mut Window>) {
    let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS)
    else { return; };

    let Some(value) = fps.smoothed()
    else { return; };

    let mut window = windows.single_mut();
    window.title = format!("FPS: {}", value);
}
