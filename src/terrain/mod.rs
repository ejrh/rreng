use bevy::prelude::*;

pub mod datafile;
pub mod loading;
pub mod heightmap;
pub mod terrain;
pub mod creation;
pub mod utils;
pub mod rendering;

/**
 * The terrain is set of elevation data for a fixed area.
 *
 * It is represented by a 2D array with m rows and n columns.  The data is split into
 * regularly spaced blocks.
 *
 * Elevations are recorded at points, with the same elevation used in adjacent blocks
 * along the edges of each block.  Therefore each block overlaps the next by 1 element,
 * and the size of the array is a multiple of block size, plus 1 element.
 *
 * The first coordinate of terrain space represents the North-South direction, starting
 * in the South.  The second represents West-East, starting in the West.  So point (0, 0)
 * is at the "top-left" corner of the map.
 *
 * In 3D world space, W-E is the x-axis and N-S is the z-axis.  (The y-axis is the height.)
 */
#[derive(Default)]
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<terrain::Terrain>()
            .init_resource::<loading::LoadingState>()
            .init_asset::<datafile::DataFile>()
            .init_asset_loader::<datafile::DataFileLoader>()
            .init_asset::<datafile::ElevationFile>()
            .init_asset_loader::<datafile::ElevationFileLoader>()
            .add_systems(Startup, loading::load_initial_terrain)
            .add_systems(Update, loading::datafile_loaded)
            .add_systems(Update, loading::elevation_loaded)
            .add_systems(Update, rendering::update_meshes)
            .add_systems(Update, rendering::swap_mesh_alternates);
    }
}
