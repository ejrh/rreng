use std::ops::Range;

use bevy::prelude::*;
use ndarray::s;

use crate::terrain::datafile::DataFile;
use crate::terrain::heightmap::heightmap_to_mesh;
use crate::terrain::utils::{Range2, get_copyable_range, restrict_ranges};

#[derive(Default, Debug)]
pub struct BlockInfo {
    pub block_num: (usize, usize),
    pub range: Range2,
    pub dirty: bool,
    pub mesh_entity: Option<Entity>,
}

#[derive(Default, Debug, Resource)]
pub struct Terrain {
    pub bounds: Rect,
    pub size: [usize; 2],
    pub block_size: usize,
    pub resolution: Vec3,
    pub num_blocks: [usize; 2],
    pub elevation: ndarray::Array2<f32>,
    pub block_info: ndarray::Array2<BlockInfo>,
}

impl Terrain {
    pub(crate) fn reset(&mut self, datafile: &DataFile) {
        self.bounds = datafile.bounds;
        self.size = datafile.size;
        self.block_size = 64;
        self.resolution = Vec3::new(1.0, 1.0, 1.0);
        self.num_blocks = [datafile.size[0] / self.block_size, datafile.size[1] / self.block_size];
        let point_dims = self.num_blocks.map(|b| self.block_size * b + 1);
        self.elevation = ndarray::Array2::default(point_dims);
        self.block_info = ndarray::Array2::from_shape_fn(self.num_blocks, |(r, c)| BlockInfo {
            block_num: (r, c),
            range: Range2(r * self.block_size..(r+1) * self.block_size + 1, c * self.block_size..(c+1) * self.block_size + 1),
            dirty: false,
            mesh_entity: None,
        });
    }

    pub fn set_elevation(&mut self, offset: (isize, isize), data: ndarray::ArrayView2<f32>) {
        let data_dims = data.dim();
        let point_dims = self.elevation.dim();
        let (from_rows, to_rows) = get_copyable_range(data_dims.0, offset.0, point_dims.0);
        let (from_cols, to_cols) = get_copyable_range(data_dims.1, offset.1, point_dims.1);

        if from_rows.is_empty() || from_cols.is_empty() { return; }

        let data_range = Range2(to_rows.clone(), to_cols.clone());

        let src = data.slice(s!(from_rows, from_cols));
        let mut dest = self.elevation.slice_mut(s!(to_rows, to_cols));
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

        if r >= 0 && c >= 0 && r < self.elevation.dim().0 && c < self.elevation.dim().1 {
            self.elevation[(r, c)]
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
