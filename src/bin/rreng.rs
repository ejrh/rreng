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
        ).add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(sky::SkyPlugin)
        .add_plugins(terrain::TerrainPlugin)
        .add_plugins(track::TrackPlugin)
        .add_plugins(debug::DebugPlugin)
        .add_plugins(tools::ToolsPlugin)
        .add_systems(Update, utils::show_fps)
        .add_systems(Startup, utils::show_version)
        .add_systems(Startup, utils::show_help_text)
        .add_systems(Update, utils::fix_apparent_size)
        .add_systems(Update, (train::move_train, train::update_train_position));

    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, utils::close_on_esc);

    app.run();
}
