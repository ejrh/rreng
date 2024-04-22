use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    window::close_on_esc,
    prelude::*,
};

use rreng::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(camera::CameraPlugin::default())
        .add_plugins(sky::SkyPlugin::default())
        .init_asset::<datafile::DataFile>()
        .init_asset_loader::<datafile::DataFileLoader>()
        .init_asset::<datafile::ChunkElevation>()
        .init_asset_loader::<datafile::ChunkElevationLoader>()
        .init_resource::<terrain::TerrainState>()
        .add_systems(Startup, terrain::load_initial_terrain)
        .add_systems(Update, terrain::datafile_loaded)
        .add_systems(Update, terrain::elevation_loaded)
        .add_systems(Update, utils::show_fps)
        .add_systems(Update, close_on_esc)
        .run();
}
