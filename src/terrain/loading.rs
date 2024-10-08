use std::collections::HashMap;

use bevy::prelude::*;

use crate::camera::CameraState;
use crate::terrain::datafile::{Chunk, DataFile, ElevationFile};
use crate::terrain::terrain::Terrain;

#[derive(Debug, Default, Resource)]
pub struct LoadingState {
    datafile_handle: Handle<DataFile>,
    elevation_handles: HashMap<Handle<ElevationFile>, (isize, isize)>,
}

impl LoadingState {
    fn get_chunk_offset(&self, asset_id: AssetId<ElevationFile>) -> Option<(isize, isize)> {
        self.elevation_handles.iter()
            .find(|(h, _)| h.id() == asset_id)
            .map(|(_, ofs)| *ofs)
    }
}

pub fn load_initial_terrain(
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
) {
    loading_state.datafile_handle = asset_server.load::<DataFile>("data/jvl.json");
    info!("datafile loading");
}

pub fn datafile_loaded(
    mut loading_state: ResMut<LoadingState>,
    mut terrain: ResMut<Terrain>,
    mut events: EventReader<AssetEvent<DataFile>>,
    assets: Res<Assets<DataFile>>,
    asset_server: Res<AssetServer>,
) {
    for evt in events.read() {
        if let AssetEvent::Added { id } = evt {
            let datafile = assets.get(*id).unwrap();
            info!("datafile loaded: {id}");

            /* Reset the terrain parameters */
            terrain.reset(&datafile);

            /* Process the data file and load the chunk elevations */
            let datafile_path = asset_server.get_path(*id).unwrap();
            let mut elevation_handles = HashMap::new();
            for chunk in &datafile.chunks {
                let elevation_path = datafile_path.parent().unwrap().resolve(&chunk.elevation).unwrap();
                elevation_handles.insert(asset_server.load::<ElevationFile>(elevation_path), chunk.position);
                info!("elevation loading");
            }
            loading_state.elevation_handles = elevation_handles;
        }
    }
}

pub fn elevation_loaded(
    mut loading_state: ResMut<LoadingState>,
    mut terrain: ResMut<Terrain>,
    mut events: EventReader<AssetEvent<ElevationFile>>,
    assets: Res<Assets<ElevationFile>>,
) {
    for evt in events.read() {
        if let AssetEvent::Added { id } = evt {
            info!("elevation loaded: {id}");

            let elevation_file = assets.get(*id).unwrap();

            let Some((offset_r, offset_c)) = loading_state.get_chunk_offset(*id)
            else { continue; };

            let offset = (offset_r * 720, offset_c * 480);

            terrain.set_elevation(offset, elevation_file.heights.view());
        }
    }
}

pub fn set_camera_range(
    terrain: Res<Terrain>,
    mut camera_query: Query<&mut CameraState>,
) {
    if !terrain.is_changed() { return; }
    let xrange = (terrain.num_blocks[1] * terrain.block_size) as f32;
    let yrange = 1000.0;
    let zrange = (terrain.num_blocks[0] * terrain.block_size) as f32;

    if let Ok(mut camera_state) = camera_query.get_single_mut() {
        camera_state.focus_range = Vec3::ZERO..Vec3::new(xrange, yrange, zrange);
        camera_state.distance_range = 1.0..xrange.max(zrange);
    }
}
