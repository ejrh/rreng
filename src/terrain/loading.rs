use std::collections::HashMap;

use bevy::prelude::*;

use crate::camera::CameraState;
use crate::terrain::datafile::{DataFile, Track};
use crate::terrain::{Terrain, TerrainLayer};
use crate::terrain::tiles::{ElevationFile, Tile, TileSets};
use crate::track::point::Point;
use crate::track::segment::Segment;
use crate::train::TrainCar;

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

pub fn load_initial_terrain(
    mut loading_state: ResMut<LoadingState>,
    asset_server: Res<AssetServer>,
) {
    loading_state.tilesets_handle = asset_server.load::<TileSets>("data/tiles.toml");
    match loading_state.datafile_handle.id() {
        AssetId::Uuid { uuid } if uuid == AssetId::<DataFile>::DEFAULT_UUID =>
            loading_state.datafile_handle = asset_server.load::<DataFile>("data/jvl.toml"),
        _ => (),
    };
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
    mut commands: Commands,
) {
    let Some(tilesets) = tilesets_assets.get(&loading_state.tilesets_handle)
    else { return };

    let Some(datafile) = datafile_assets.get(&loading_state.datafile_handle)
    else { return };

    /* Reset the terrain parameters */
    info!("Level bounds are: {:?}", datafile.bounds);

    terrain.reset(datafile);

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

    if !loading_state.created_tracks {
        create_initial_tracks(datafile, &asset_server, &mut commands);
        loading_state.created_tracks = true;
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

            let Some((tile, layer)) = loading_state.get_tile(*id)
            else { continue; };

            let tile_corner = Vec2::new(tile.bounds.min.x, tile.bounds.max.y);
            let offset = terrain.coord_to_offset(tile_corner);
            terrain.set_elevation(offset, elevation_file.heights.view(), *layer);
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

    if let Ok(mut state) = camera_query.get_single_mut() {
        state.focus_range = Vec3::ZERO..Vec3::new(xrange, yrange, zrange);
        state.focus = state.focus_range.start + state.focus_range.end / 2.0;
        state.distance_range = 1.0..xrange.max(zrange).max(1.0);
        state.distance = state.distance_range.end;
    }
}

fn create_initial_tracks(
    datafile: &DataFile,
    asset_server: &AssetServer,
    commands: &mut Commands
) {
    /* Create existing tracks */
    for (name, Track {points }) in datafile.tracks.iter() {
        let parent_id = commands
            .spawn((
                Name::new(format!("Track:{name}")),
                Visibility::default(),
                Transform::default()
            )).id();

        let point_ids = points.iter()
            .map(|pt| commands.spawn((Point, Transform::from_translation(*pt))).set_parent(parent_id).id())
            .collect::<Vec<_>>();

        let segment_ids: Vec<_> = point_ids.windows(2).map(|w| {
            let [pt1, pt2, ..] = w else { panic!("Expect window of size 2") };
            commands.spawn(Segment {
                from_point: *pt1,
                to_point: *pt2,
                length: 0.0,
            }).set_parent(parent_id).id()
        }).collect();
        info!("created track with {} segments", segment_ids.len());

        /* Put a train at the start of the first segment */
        const TRAIN_PATH: &str = "models/lowpoly_train.glb";

        let first_segment_id = segment_ids[0];

        let train_id = commands.spawn((
            Name::new("Train"),
            TrainCar {
                segment_id: first_segment_id,
                segment_position: 0.0,
                speed: 0.001,
                acceleration: 1.0,
                max_speed: 100_000.0 / 3600.0,
            },
            Transform::default(),
            Visibility::default(),
        )).id();
        info!("created train");

        /* Spawn train model and fix up its silly model transform */
        commands.spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(TRAIN_PATH))),
            Transform::default()
                .with_scale(Vec3::splat(3.28084))
                .with_rotation(Quat::from_axis_angle(Vec3::Y, 54.0f32.to_radians())),
        )).set_parent(train_id);

        /* Put some spot lights for the train's headlamps */
        const LIGHT_POSITION: Vec3 = Vec3::new(0.85, 1.7, 9.3);
        for (xs, zs) in [
            (-1.0, -1.0),
            (-1.0, 1.0),
            (1.0, -1.0),
            (1.0, 1.0),
        ] {
            let pos = LIGHT_POSITION * Vec3::new(xs, 1.0, zs);
            let target = (LIGHT_POSITION + Vec3::new(0.0, 0.0, 10.0)) * Vec3::new(xs, 1.0, zs);
            commands.spawn((
                SpotLight::default(),
                Transform::from_translation(pos).looking_at(target, Vec3::Y),
            )).set_parent(train_id);
        }
    }
}
