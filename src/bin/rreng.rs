use bevy::{
    asset::AssetMetaCheck,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
};
use rreng::*;

fn main() {
    let mut app = App::new();

    app
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin { meta_check: AssetMetaCheck::Never, ..default() })
        ).add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(camera::CameraPlugin::default())
        .add_plugins(sky::SkyPlugin::default())
        .add_plugins(terrain::TerrainPlugin::default())
        .add_plugins(debug::DebugPlugin::default())
        .add_systems(Update, utils::show_fps)
        .add_systems(Startup, utils::show_version)
        .add_systems(Startup, utils::show_help_text);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, utils::close_on_esc);

    app.run();
}
