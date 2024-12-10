use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
};

use rreng::*;
use rreng::terrain::datafile::DataFile;
use rreng::terrain::loading::LoadingState;

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
        )
        .add_plugins(camera::CameraPlugin)
        .add_plugins(sky::SkyPlugin)
        .add_plugins(terrain::TerrainPlugin)
        .add_plugins(track::TrackPlugin)
        .add_plugins(debug::DebugPlugin)
        .add_systems(Update, utils::fix_apparent_size)
        .register_type::<train::TrainCar>()
        .add_systems(Update, (train::move_train, train::update_train_position));

    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, utils::close_on_esc);

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(
    mut loading_state: ResMut<LoadingState>,
    mut datafile_assets: ResMut<Assets<DataFile>>,
) {
    let datafile = toml::from_str(LEVEL_FILE).unwrap();
    datafile_assets.insert(LEVEL_FILE_HANDLE.id(), datafile);
    loading_state.datafile_handle = LEVEL_FILE_HANDLE;
}

pub const LEVEL_FILE_HANDLE: Handle<DataFile> = Handle::weak_from_u128(19781219_1);

const LEVEL_FILE: &str = r#"
size = [64, 64]
layers = ["Elevation"]

[bounds]
min = [1749280.0, 5429860.0]
max = [1749760.0, 5430580.0]

[tracks.TEST]
points = [
    [4.0, 2.5, 4.0],
    [36.0, 2.2, 28.0],
    [60.0, 2.2, 60.0],
]
"#;
