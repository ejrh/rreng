use std::sync::{Arc, Mutex};
use bevy::prelude::*;
use ndarray::s;
use serde::{Deserialize, Serialize};
use crate::terrain::datafile::DataFile;
use crate::terrain::utils::{get_copyable_range, Range2};

pub mod creation;
pub mod datafile;
pub mod edit;
pub mod heightmap;
pub mod loading;
pub mod rendering;
pub mod rtin;
pub mod selection;
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
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Terrain>()
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
            .add_systems(Update, (loading::check_loading_state, loading::set_camera_range)
                .run_if(resource_changed::<loading::LoadingState>))
            .init_resource::<rendering::MeshTaskQueue>()
            .add_systems(Update, (rendering::update_meshes, rendering::swap_mesh_alternates, rendering::handle_mesh_tasks))
            .init_resource::<selection::SelectedPoint>();

        app
            .add_systems(Startup, selection::create_marker)
            .add_systems(Update, selection::update_selected_point)
            .add_systems(Update, selection::update_cursor_position)
            .add_systems(Startup, selection::create_cursor_position_text);
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainLayer {
    Elevation,
    Structure,
    MAX
}

#[derive(Default, Debug)]
pub struct BlockInfo {
    pub block_num: (usize, usize),
    pub range: Range2,
    pub dirty: bool,
}

#[derive(Default, Debug, Resource)]
pub struct Terrain {
    pub bounds: Rect,
    pub size: [usize; 2],
    pub block_size: usize,
    pub resolution: Vec3,
    pub num_blocks: [usize; 2],
    pub point_dims: [usize; 2],
    pub layers: [Arc<Mutex<ndarray::Array2<f32>>>; TerrainLayer::MAX as usize],
    pub block_info: ndarray::Array2<BlockInfo>,
}

impl Terrain {
    pub(crate) fn reset(&mut self, datafile: &DataFile) {
        self.bounds = datafile.bounds;
        self.size = datafile.size;
        self.block_size = 64;
        self.resolution = Vec3::new(1.0, 1.0, 1.0);
        self.num_blocks = [datafile.size[0] / self.block_size, datafile.size[1] / self.block_size];
        self.point_dims = self.num_blocks.map(|b| self.block_size * b + 1);
        for i in 0..TerrainLayer::MAX as usize {
            self.layers[i] = Arc::new(Mutex::new(ndarray::Array2::default(self.point_dims)));
        }
        self.block_info = ndarray::Array2::from_shape_fn(self.num_blocks, |(r, c)| BlockInfo {
            block_num: (r, c),
            range: Range2(r * self.block_size..(r+1) * self.block_size + 1, c * self.block_size..(c+1) * self.block_size + 1),
            dirty: false,
        });
    }

    pub fn set_elevation(&mut self, offset: (isize, isize), data: ndarray::ArrayView2<f32>, layer: TerrainLayer) {
        let target_layer = &mut self.layers[layer as usize];
        let mut target_layer = target_layer.lock().unwrap();

        let data_dims = data.dim();
        let (from_rows, to_rows) = get_copyable_range(data_dims.0, offset.0, self.point_dims[0]);
        let (from_cols, to_cols) = get_copyable_range(data_dims.1, offset.1, self.point_dims[1]);

        if from_rows.is_empty() || from_cols.is_empty() { return; }

        let data_range = Range2(to_rows.clone(), to_cols.clone());

        let src = data.slice(s!(from_rows, from_cols));
        let mut dest = target_layer.slice_mut(s!(to_rows, to_cols));
        dest.assign(&src);

        for i in 0..self.num_blocks[0] {
            for j in 0..self.num_blocks[1] {
                let block_info = &mut self.block_info[(i, j)];
                if !block_info.range.overlaps(&data_range) {
                    continue;
                }

                block_info.dirty = true;
            }
        }
    }

    pub fn dirty_range(&mut self, range: Range2) {
        for bi in self.block_info.iter_mut() {
            if bi.range.overlaps(&range) {
                bi.dirty = true;
            }
        }
    }

    /**
     * Point is in world space!
     */
    pub fn elevation_at(&self, point: Vec2) -> f32 {
        let r = point.y as usize;
        let c = point.x as usize;

        let elevation = &self.layers[TerrainLayer::Elevation as usize];
        let elevation = elevation.lock().unwrap();

        if r < elevation.dim().0 && c < elevation.dim().1 {
            elevation[(r, c)]
        } else {
            -1.0
        }
    }

    /**
     * Coord in in coordinate system space
     */
    pub fn coord_to_offset(&self, coord: Vec2) -> (isize, isize) {
        let c = coord - self.bounds.min;

        (self.size[0] as isize - c.y as isize, c.x as isize)
    }
}
