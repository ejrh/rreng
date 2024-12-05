use std::collections::HashMap;

use bevy::prelude::*;

use crate::camera::CameraState;
use crate::terrain::datafile::DataFile;
use crate::terrain::Terrain;
use crate::terrain::tiles::{ElevationFile, Tile, TileSets};

#[derive(Debug, Default, Resource)]
pub struct LoadingState {
    tilesets_handle: Handle<TileSets>,
    datafile_handle: Handle<DataFile>,
    elevation_handles: HashMap<Handle<ElevationFile>, Tile>,
}

impl LoadingState {
    fn get_tile(&self, asset_id: AssetId<ElevationFile>) -> Option<&Tile> {
        self.elevation_handles.iter()
            .find(|(h, _)| h.id() == asset_id)
            .map(|(_, tile)| tile)
    }
}

pub fn load_initial_terrain(
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
) {
    loading_state.tilesets_handle = asset_server.load::<TileSets>("data/tiles.toml");
    loading_state.datafile_handle = asset_server.load::<DataFile>("data/jvl.toml");
}

pub fn tilesets_loaded(
    mut loading_state: ResMut<LoadingState>,
    mut events: EventReader<AssetEvent<TileSets>>,
) {
    for evt in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = evt {
            info!("tilesets loaded: {id}");
            loading_state.set_changed();
        }
    }
}

pub fn datafile_loaded(
    mut loading_state: ResMut<LoadingState>,
    mut events: EventReader<AssetEvent<DataFile>>,
) {
    for evt in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = evt {
            info!("datafile loaded: {id}");
            loading_state.set_changed();
        }
    }
}

pub fn check_loading_state(
    mut loading_state: ResMut<LoadingState>,
    mut terrain: ResMut<Terrain>,
    datafile_assets: Res<Assets<DataFile>>,
    tilesets_assets: Res<Assets<TileSets>>,
    asset_server: Res<AssetServer>,
) {
    let Some(tilesets) = tilesets_assets.get(&loading_state.tilesets_handle)
    else { return };

    let Some(datafile) = datafile_assets.get(&loading_state.datafile_handle)
    else { return };

    let tileset = tilesets.0.get("wgtn_1m_dem").unwrap();

    /* Reset the terrain parameters */
    info!("Level bounds are: {:?}", datafile.bounds);

    terrain.reset(datafile);

    /* Process the data file and load the chunk elevations */
    let tilesets_path = asset_server.get_path(&loading_state.tilesets_handle).unwrap();
    let tileset_path = tilesets_path.parent().unwrap().resolve(&tileset.root).unwrap();
    let mut new_elevation_handles = HashMap::new();
    for (name, tile) in &tileset.files {

        if terrain.bounds.intersect(tile.bounds).is_empty() {
            continue;
        }

        let elevation_path = tileset_path.resolve(name).unwrap();
        let handle = asset_server.load::<ElevationFile>(elevation_path);
        if loading_state.elevation_handles.contains_key(&handle) {
            continue;
        }
        new_elevation_handles.insert(handle, (*tile).clone());
    }
    if !new_elevation_handles.is_empty() {
        loading_state.elevation_handles.extend(new_elevation_handles);
    }
}

pub fn elevation_loaded(
    loading_state: ResMut<LoadingState>,
    mut terrain: ResMut<Terrain>,
    mut events: EventReader<AssetEvent<ElevationFile>>,
    assets: Res<Assets<ElevationFile>>,
    asset_server: Res<AssetServer>,
) {
    for evt in events.read() {
        if let AssetEvent::Added { id } = evt {
            info!("elevation loaded: {:?}", asset_server.get_path(*id));

            let elevation_file = assets.get(*id).unwrap();

            let Some(tile) = loading_state.get_tile(*id)
            else { continue; };

            let tile_corner = Vec2::new(tile.bounds.min.x, tile.bounds.max.y);
            let offset = terrain.coord_to_offset(tile_corner);
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
