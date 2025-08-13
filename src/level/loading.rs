use std::collections::HashMap;
use std::f32::consts::TAU;
use bevy::prelude::*;

use crate::camera::{CameraMode, CameraState};
use crate::events::GameEvent;
use crate::level::datafile::{DataFile, TrackToLoad};
use crate::screens::Screen;
use crate::terrain::{Terrain, TerrainData, TerrainLayer};
use crate::terrain::rendering::TerrainRenderParams;
use crate::terrain::tiles::{ElevationFile, Tile, TileSets};
use crate::track::create_track;
use crate::train::create_train;

#[derive(Debug, Default, Resource)]
pub struct LoadingState {
    tilesets_handle: Handle<TileSets>,
    pub datafile_handle: Handle<DataFile>,
    elevation_handles: HashMap<Handle<ElevationFile>, (Tile, TerrainLayer)>,
    created_tracks: bool,
}

impl LoadingState {
    fn get_tile(&self, asset_id: AssetId<ElevationFile>) -> Option<&(Tile, TerrainLayer)> {
        self.elevation_handles.iter()
            .find(|(h, _)| h.id() == asset_id)
            .map(|(_, tile)| tile)
    }
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
    mut terrain_data: ResMut<TerrainData>,
    datafile_assets: Res<Assets<DataFile>>,
    tilesets_assets: Res<Assets<TileSets>>,
    asset_server: Res<AssetServer>,
    mut render_params: ResMut<TerrainRenderParams>,
    mut commands: Commands,
) {
    let Some(tilesets) = tilesets_assets.get(&loading_state.tilesets_handle)
    else { return };

    let Some(datafile) = datafile_assets.get(&loading_state.datafile_handle)
    else { return };

    /* Reset the terrain parameters */
    info!("Level bounds are: {:?}", datafile.bounds);

    terrain.reset(datafile);
    terrain_data.reset(&terrain, datafile);

    /* Process the data file and load the chunk elevations */
    let tilesets_path = asset_server.get_path(&loading_state.tilesets_handle).unwrap();

    let mut new_elevation_handles = HashMap::new();

    for tileset in tilesets.0.values() {
        if !datafile.layers.contains(&tileset.layer) {
            continue;
        }

        let tileset_path = tilesets_path.parent().unwrap().resolve(&tileset.root).unwrap();
        for (name, tile) in &tileset.files {
            if terrain.bounds.intersect(tile.bounds).is_empty() {
                continue;
            }

            let elevation_path = tileset_path.resolve(name).unwrap();
            let handle = asset_server.load::<ElevationFile>(elevation_path);
            if loading_state.elevation_handles.contains_key(&handle) {
                continue;
            }
            new_elevation_handles.insert(handle, ((*tile).clone(), tileset.layer));
        }
    }

    if !new_elevation_handles.is_empty() {
        loading_state.elevation_handles.extend(new_elevation_handles);
    }

    let parent_id = render_params.level_id.get_or_insert_with(
        || commands.spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Playing),
        )).id()
    );

    if !loading_state.created_tracks {
        /* Create existing tracks */
        for (name, TrackToLoad {points }) in datafile.tracks.iter() {
            let (track_id, _, segment_ids) = create_track(name, points, false, &mut commands);

            /* Put a train at the start of the first segment */
            let first_segment_id = segment_ids[0];
            let train_id = create_train(name, first_segment_id, 0.0, 0.01, &mut commands);

            commands.entity(track_id).insert(ChildOf(*parent_id));
            commands.entity(train_id).insert(ChildOf(*parent_id));
        }

        loading_state.created_tracks = true;
    }

    crate::worker::create_workers(&terrain, &mut commands, 1);

    commands.send_event(GameEvent::LoadingComplete);
}

pub fn elevation_loaded(
    loading_state: ResMut<LoadingState>,
    terrain: Res<Terrain>,
    mut terrain_data: ResMut<TerrainData>,
    mut events: EventReader<AssetEvent<ElevationFile>>,
    assets: Res<Assets<ElevationFile>>,
    asset_server: Res<AssetServer>,
) {
    for evt in events.read() {
        if let AssetEvent::Added { id } = evt {
            info!("elevation loaded: {:?}", asset_server.get_path(*id));

            let elevation_file = assets.get(*id).unwrap();

            let Some((tile, layer)) = loading_state.get_tile(*id)
            else { continue; };

            let tile_corner = Vec2::new(tile.bounds.min.x, tile.bounds.max.y);
            let offset = terrain.coord_to_offset(tile_corner);
            terrain_data.set_elevation(offset, elevation_file.heights.view(), *layer);
        }
    }
}

pub fn set_camera_range(
    terrain: Res<Terrain>,
    mut state: Single<&mut CameraState>,
) {
    if !terrain.is_changed() { return; }
    let xrange = (terrain.num_blocks[1] * terrain.block_size) as f32;
    let yrange = 1000.0;
    let zrange = (terrain.num_blocks[0] * terrain.block_size) as f32;

    state.focus_range = Vec3::ZERO..Vec3::new(xrange, yrange, zrange);
    state.focus = state.focus_range.start + state.focus_range.end / 2.0;
    state.yaw_range = None;
    state.pitch_range = -TAU/4.0..TAU/4.0;
    state.distance_range = 1.0..xrange.max(zrange).max(1.0);
    state.distance = state.distance_range.end;
    state.mode = CameraMode::Steer;
}

pub fn handle_game_events(
    mut events: EventReader<GameEvent>,
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
    mut datafile_assets: ResMut<Assets<DataFile>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    let Some(event) = events.read().next()
    else { return; };

    match event {
        GameEvent::LoadLevel(level_path) => {
            loading_state.tilesets_handle = asset_server.load::<TileSets>("data/tiles.ron");
            loading_state.datafile_handle = asset_server.load::<DataFile>(level_path);
            next_screen.set(Screen::Loading);
        },
        GameEvent::LoadLevelData(datafile) => {
            loading_state.tilesets_handle = asset_server.load::<TileSets>("data/tiles.ron");
            let handle = datafile_assets.reserve_handle();
            datafile_assets.insert(handle.id(), datafile.clone());
            loading_state.datafile_handle = handle;
            next_screen.set(Screen::Loading);
        },
        GameEvent::LoadingComplete => {
            info!("Loading complete");
            next_screen.set(Screen::Playing);
        }
        GameEvent::ExitLevel => {
            info!("Exiting level");
            next_screen.set(Screen::Title);
        }
    }
}
