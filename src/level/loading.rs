use std::collections::HashMap;

use bevy::prelude::*;

use crate::events::{GameEvent, GraphicsEvent};
use crate::level::datafile::{DataFile, TrackToLoad};
use crate::level::LevelLabel;
use crate::screens::Screen;
use crate::terrain::{Terrain, TerrainData, TerrainLayer};
use crate::terrain::rendering::{LayerLabel, MeshTaskQueue};
use crate::terrain::rendering::mesh_tree::MeshTree;
use crate::terrain::rendering::water::WaterLabel;
use crate::terrain::tiles::{ElevationFile, Tile, TileSets};
use crate::track::create_track;
use crate::train::create_train;

const TILESETS_ASSET_PATH: &str = "data/tiles.ron";

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, check_loading_state.run_if(in_state(Screen::Loading)))
            .add_systems(Update, update_loading_progress.run_if(in_state(Screen::Loading)));
    }
}

#[derive(Copy, Clone, Debug, Reflect)]
pub enum LoadingStage {
    LoadingData,
    LoadingTerrain,
    ReticulatingSplines,
    CreatingObjects,
}

#[derive(Component)]
pub struct LoadingStageLabel(pub LoadingStage);

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct LoadingState {
    stage: LoadingStage,
    datafile_handle: Handle<DataFile>,
    tilesets_handle: Handle<TileSets>,
    elevation_handles: HashMap<Handle<ElevationFile>, (Tile, TerrainLayer)>,
    files_loaded: u32,
    files_expected: u32,
    tiles_loaded: u32,
    tiles_expected: u32,
    meshes_built: u32,
    meshes_expected: u32,
}

impl LoadingState {
    pub(crate) fn new(datafile_handle: Handle<DataFile>) -> Self {
        LoadingState {
            stage: LoadingStage::LoadingData,
            datafile_handle,
            tilesets_handle: default(),
            elevation_handles: HashMap::new(),
            files_loaded: 0,
            files_expected: 2,
            tiles_loaded: 0,
            tiles_expected: 0,
            meshes_built: 0,
            meshes_expected: 0,
        }
    }
}

pub fn check_loading_state(
    mut level: Single<(Entity, &mut Terrain, &mut TerrainData, &mut LoadingState), With<LevelLabel>>,
    datafile_assets: Res<Assets<DataFile>>,
    tilesets_assets: Res<Assets<TileSets>>,
    elevation_assets: Res<Assets<ElevationFile>>,
    asset_server: Res<AssetServer>,
    mesh_trees: Query<&MeshTree>,
    mut commands: Commands,
) {
    let (level_id, terrain, terrain_data, loading_state) = &mut *level;

    match loading_state.stage {
        LoadingStage::LoadingData => {
            loading_state.files_loaded = 0;
            let Some(datafile) = datafile_assets.get(&loading_state.datafile_handle)
            else { return; };
            loading_state.files_loaded += 1;

            loading_state.tilesets_handle = asset_server.load(TILESETS_ASSET_PATH);

            let Some(tilesets) = tilesets_assets.get(&loading_state.tilesets_handle)
            else { return; };
            loading_state.files_loaded += 1;

            info!("Level bounds are: {:?}", datafile.bounds);
            terrain.reset(datafile);
            terrain_data.reset(terrain, datafile);

            let tilesets_path = asset_server.get_path(&loading_state.tilesets_handle).unwrap();

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

                    loading_state.elevation_handles.insert(handle, ((*tile).clone(), tileset.layer));
                    loading_state.tiles_expected += 1;
                }
            }

            loading_state.stage = LoadingStage::LoadingTerrain;
        }
        LoadingStage::LoadingTerrain => {
            let elevation_handles = std::mem::take(&mut loading_state.elevation_handles);
            for (handle, (tile, layer)) in elevation_handles {
                if let Some(elevation_file) = elevation_assets.get(&handle) {
                    let tile_corner = Vec2::new(tile.bounds.min.x, tile.bounds.max.y);
                    let offset = terrain.coord_to_offset(tile_corner);
                    terrain_data.set_elevation(offset, elevation_file.heights.view(), layer);
                    loading_state.tiles_loaded += 1;
                } else {
                    loading_state.elevation_handles.insert(handle, (tile, layer));
                }
            }

            if loading_state.elevation_handles.is_empty() {
                loading_state.stage = LoadingStage::ReticulatingSplines;
            }
        }
        LoadingStage::ReticulatingSplines => {
            loading_state.meshes_built = 0;
            loading_state.meshes_expected = 0;
            for tree in mesh_trees {
                tree.walk(&mut |t, id| {
                    if t.valid(id) {
                        loading_state.meshes_expected += 1;
                        if t.populated(id) {
                            loading_state.meshes_built += 1;
                        }
                    }
                    true
                });
            }
            if loading_state.meshes_built == loading_state.meshes_expected {
                loading_state.stage = LoadingStage::CreatingObjects;
            }
        }
        LoadingStage::CreatingObjects => {
            let Some(datafile) = datafile_assets.get(&loading_state.datafile_handle)
            else { return; };

            /* Create existing tracks */
            for (name, TrackToLoad {points }) in datafile.tracks.iter() {
                let (track_id, _, segment_ids) = create_track(name, points, false, &mut commands);

                /* Put a train at the start of the first segment */
                let first_segment_id = segment_ids[0];
                let train_id = create_train(name, first_segment_id, 0.0, 0.01, &mut commands);

                commands.entity(track_id).insert(ChildOf(*level_id));
                commands.entity(train_id).insert(ChildOf(*level_id));
            }

            crate::worker::create_workers(*level_id, terrain, &mut commands, 1);

            commands.send_event(GameEvent::LoadingComplete);
        }
    }
}

pub fn update_loading_progress(
    loading_state: Single<&LoadingState, With<LevelLabel>>,
    texts: Query<(&LoadingStageLabel, &mut Text), Without<LevelLabel>>,
) {
    for (label, mut text) in texts {
        let (done, expected) = match label.0 {
            LoadingStage::LoadingData => (loading_state.files_loaded, loading_state.files_expected),
            LoadingStage::LoadingTerrain => (loading_state.tiles_loaded, loading_state.tiles_expected),
            LoadingStage::ReticulatingSplines => (loading_state.meshes_built, loading_state.meshes_expected),
            LoadingStage::CreatingObjects => (0, 0),
        };

        let pct = if expected == 0 { 0 } else { done * 100 / expected };
        text.0 = format!("{}%", pct);
    }
}

pub(crate) fn new_level(datafile_handle: Handle<DataFile>) -> impl Bundle {
    (
        LevelLabel,
        Name::new("Level"),
        Transform::default(),
        Visibility::Hidden,
        Terrain::default(),
        TerrainData::default(),
        LoadingState::new(datafile_handle),
        children![
            (
                WaterLabel,
                Name::new("Water"),
                Visibility::default(),
                Transform::default(),
            ),
        ],
    )
}
