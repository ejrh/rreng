use bevy::app::App;
use bevy::asset::AssetApp;
use bevy::prelude::{Plugin, Startup, Update};

pub mod heightmap;
pub mod terrain;
pub mod datafile;

#[derive(Default)]
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<terrain::TerrainState>()
            .init_asset::<datafile::DataFile>()
            .init_asset_loader::<datafile::DataFileLoader>()
            .init_asset::<datafile::ChunkElevation>()
            .init_asset_loader::<datafile::ChunkElevationLoader>()
            .add_systems(Startup, terrain::load_initial_terrain)
            .add_systems(Update, terrain::datafile_loaded)
            .add_systems(Update, terrain::elevation_loaded);
    }
}