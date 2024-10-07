use bevy::{
    asset::AssetMetaCheck,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
};
use rreng::*;

fn main() {
    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    let window_plugin = WindowPlugin::default();

    app
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin { meta_check: AssetMetaCheck::Never, ..default() })
                .set(window_plugin)
        ).add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(camera::CameraPlugin::default())
        .add_plugins(sky::SkyPlugin::default())
        .add_plugins(terrain::TerrainPlugin::default())
        .add_plugins(track::TrackPlugin)
        .add_plugins(debug::DebugPlugin::default())
        .add_systems(Update, utils::show_fps)
        .add_systems(Startup, utils::show_version)
        .add_systems(Startup, utils::show_help_text)
        .add_systems(Update, utils::fix_apparent_size);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, utils::close_on_esc);

    // make some track and a train just for testing
    app.add_systems(Startup, track::create_track);
    app.add_systems(Startup, train::create_train);

    app.run();
}
