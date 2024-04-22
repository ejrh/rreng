use bevy::{
    prelude::*,
};
use noise::{NoiseFn, Simplex};

use crate::datafile::{Chunk, ChunkElevation, DataFile};
use crate::heightmap::heightmap_to_mesh;

#[derive(Debug, Default, Resource)]
pub struct TerrainState {
    datafile_handle: Handle<DataFile>,
    datafile: DataFile,
    chunk_handles: Vec<Handle<ChunkElevation>>,
    all_elevation_data: Vec<Vec<f32>>,
}

impl TerrainState {
    fn get_chunk(&self, asset_id: AssetId<ChunkElevation>) -> Option<&Chunk> {
        for (idx, handle) in self.chunk_handles.iter().enumerate() {
            if handle.id() == asset_id {
                return Some(&self.datafile.chunks[idx]);
            }
        }
        None
    }
}

pub fn load_initial_terrain(
    mut terrain_state: ResMut<TerrainState>,
    asset_server: Res<AssetServer>,
) {
    terrain_state.datafile_handle = asset_server.load::<DataFile>("data/jvl.json");
    info!("datafile loading");
}

pub fn datafile_loaded(
    mut terrain_state: ResMut<TerrainState>,
    mut events: EventReader<AssetEvent<DataFile>>,
    assets: Res<Assets<DataFile>>,
    asset_server: Res<AssetServer>,
) {
    for evt in events.read() {
        if let AssetEvent::Added { id } = evt {
            info!("datafile loaded: {id}");
            terrain_state.datafile = assets.get(*id).unwrap().clone();
            let datafile_path = asset_server.get_path(*id).unwrap();

            /* Process the data file and load the chunk elevations */
            let SCALING = 10;
            let all_width = 480 * 8 / SCALING as usize;
            let all_height=  720 * 3 / SCALING as usize;
            terrain_state.all_elevation_data = vec!(vec!(0.0; all_width); all_height);

            let mut chunk_handles = Vec::new();
            for chunk in &terrain_state.datafile.chunks {
                let elevation_path = datafile_path.parent().unwrap().resolve(&chunk.elevation).unwrap();
                chunk_handles.push(asset_server.load::<ChunkElevation>(elevation_path));
                info!("elevation loading");
            }
            terrain_state.chunk_handles = chunk_handles;
        }
    }
}

pub fn elevation_loaded(
    mut terrain_state: ResMut<TerrainState>,
    mut events: EventReader<AssetEvent<ChunkElevation>>,
    assets: Res<Assets<ChunkElevation>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let SCALING = 10;

    let mut assets_added = 0;

    for evt in events.read() {
        if let AssetEvent::Added { id } = evt {
            info!("elevation loaded: {id}");

            let chunk_elevation  = assets.get(*id).unwrap();
            let data = &chunk_elevation.heights;

            let Some(chunk) = terrain_state.get_chunk(*id)
            else { continue };

            let elevation_data = data[0..720].iter().step_by(SCALING).map(
                |x| x[0..480].iter().step_by(SCALING).map(|x| *x).collect::<Vec<_>>()
            ).collect::<Vec<_>>();

            let i_offset = chunk.position.1 as usize * 720 / SCALING;
            let j_offset = chunk.position.0 as usize * 480 / SCALING;
            for i in 0..elevation_data.len() {
                for j in 0..elevation_data[i].len() {
                    terrain_state.all_elevation_data[i + i_offset][j + j_offset] = elevation_data[i][j];
                }
            }

            assets_added += 1;
        }
    }

    if assets_added == 0 { return }

    let all_elevation_data = terrain_state.all_elevation_data.iter().map(|x| x.clone()).rev().collect::<Vec<_>>();

    let mesh = heightmap_to_mesh(&all_elevation_data, &Vec3::new( SCALING as f32, 1.0, SCALING as f32));
    let mesh = meshes.add(mesh);

    info!("Created mesh");

    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.8, 0.4),
            perceptual_roughness: 0.9,
            ..default()
        }),
        ..default()
    });

}

pub(crate) fn create_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let noise = Simplex::new(42);

    const GRID_SIZE: usize = 10;
    let mut grid = Vec::new();
    for i in 0..GRID_SIZE + 1 {
        grid.push(Vec::new());
        for j in 0..GRID_SIZE + 1 {
            let height = noise.get([i as f64 / GRID_SIZE as f64 * 1.0, j as f64  / GRID_SIZE as f64 * 1.0]) as f32;
            let height = height;
            grid[i].push(height);
        }
    }

    const GRID_SPACING: f32 = 100.0 / GRID_SIZE as f32;

    let mesh= heightmap_to_mesh(&grid, &Vec3::new(GRID_SPACING, 25.0, GRID_SPACING));
    let mesh = meshes.add(mesh);

    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.8, 0.4),
            perceptual_roughness: 0.9,
            ..default()
        }),
        ..default()
    });
}
