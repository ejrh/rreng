use bevy::prelude::*;
use bevy_mod_raycast::prelude::CursorRayPlugin;

pub mod creation;
pub mod datafile;
pub mod edit;
pub mod heightmap;
pub mod loading;
pub mod rendering;
pub mod rtin;
pub mod selection;
pub mod terrain;
pub mod tiles;
pub mod utils;

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
            .init_asset::<tiles::TileSets>()
            .init_asset_loader::<tiles::TileSetsLoader>()
            .init_asset::<datafile::DataFile>()
            .init_asset_loader::<datafile::DataFileLoader>()
            .init_asset::<tiles::ElevationFile>()
            .init_asset_loader::<tiles::ElevationFileLoader>()
            .add_systems(Startup, rendering::init_render_params)
            .add_systems(Startup, loading::load_initial_terrain)
            .add_systems(Update, loading::tilesets_loaded)
            .add_systems(Update, loading::datafile_loaded)
            .add_systems(Update, loading::elevation_loaded)
            .add_systems(Update, loading::check_loading_state.run_if(resource_changed::<loading::LoadingState>))
            .add_systems(Update, loading::set_camera_range)
            .add_systems(Update, rendering::update_meshes)
            .add_systems(Update, rendering::swap_mesh_alternates)
            .add_plugins(CursorRayPlugin)
            .init_resource::<selection::SelectedPoint>()
            .add_systems(Startup, selection::create_marker)
            .add_systems(Update, selection::update_selected_point);
    }
}
