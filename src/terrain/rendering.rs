use bevy::color::palettes::basic::{BLUE, GREEN, RED};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use ndarray::{s, ArrayView};

use crate::terrain::heightmap::heightmap_to_mesh;
use crate::terrain::terrain::Terrain;

#[derive(Component)]
pub struct TerrainMesh {
    block_num: (usize, usize),
}

#[derive(Component)]
pub struct TerrainMeshAlternates {
    cutoff: f32,
    high_res: Handle<Mesh>,
    low_res: Handle<Mesh>,
}

const RENDERS_PER_FRAME: usize = 1;

pub fn update_meshes(
    mut terrain: ResMut<Terrain>,
    terrain_meshes: Query<(Entity, &TerrainMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    /* Explicitly borrow the object so we can simultaneously borrow different fields  of it later */
    let terrain = &mut *terrain;

    for block_info in terrain.block_info.iter_mut()
        .filter(|bi| bi.dirty)
        .take(RENDERS_PER_FRAME) {
        block_info.dirty = false;

        let entity = block_info.mesh_entity
            .filter(|e| terrain_meshes.contains(*e))
            .unwrap_or_else(|| {
            let entity = commands.spawn(TerrainMesh { block_num: block_info.block_num }).id();
            block_info.mesh_entity = Some(entity);
            entity
        });

        let high_res = {
            let spacing = 1;
            let elevation_view = terrain.elevation.slice(s!(block_info.range.0.clone();spacing, block_info.range.1.clone();spacing));
            meshes.add(create_mesh(elevation_view, &Vec3::new(spacing as f32, 1.0, spacing as f32)))
        };

        let low_res = {
            let spacing = 4;
            let elevation_view = terrain.elevation.slice(s!(block_info.range.0.clone();spacing, block_info.range.1.clone();spacing));
            meshes.add(create_mesh(elevation_view, &Vec3::new(spacing as f32, 1.0, spacing as f32)))
        };

        let colour = if block_info.block_num.0 + block_info.block_num.1 == 0 { RED }
        else if (block_info.block_num.0 + block_info.block_num.1) & 1 == 0 { GREEN }
        else { BLUE };
        let material = materials.add(create_material(Color::Srgba(colour)));

        let xp = block_info.block_num.1 as f32 * terrain.block_size as f32;
        let yp = block_info.block_num.0 as f32 * terrain.block_size as f32;
        let transform = Transform::from_xyz(xp, 0.0, yp);

        let mesh = low_res.clone();

        let alternates = TerrainMeshAlternates {
            cutoff: 1000.0,
            high_res,
            low_res,
        };

        commands.entity(entity).insert(MaterialMeshBundle {
            material,
            mesh,
            transform,
            ..default()
        }).insert(alternates);

        info!("spawned terrain mesh");
    }

    /* Clean up orphaned terrain meshes */
    for (entity, terrain_mesh) in terrain_meshes.iter() {
        let block_info = &terrain.block_info[terrain_mesh.block_num];
        if block_info.mesh_entity != Some(entity) {
            info!("despawn orphaned terrain mesh");
            commands.entity(entity).despawn();
        }
    }
}

pub fn swap_mesh_alternates(
    camera_query: Query<&GlobalTransform, (With<Camera>, Changed<GlobalTransform>)>,
    mesh_alternates: Query<(Entity, &Handle<Mesh>, &GlobalTransform, &TerrainMeshAlternates)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    const TOLERANCE: f32 = 50.0;

    let Ok(camera_transform) = camera_query.get_single() else { return };

    for (entity, current, mesh_transform, alternates) in mesh_alternates.iter() {
        let dist = mesh_transform.translation().distance(camera_transform.translation());

        let diff = dist - alternates.cutoff;

        let (preferred, colour) = if diff < 0.0 {
            (&alternates.high_res, GREEN)
        } else {
            (&alternates.low_res, BLUE)
        };

        if !current.eq(preferred) {
            if diff.abs() < TOLERANCE {
                continue;
            }

            let material = materials.add(create_material(Color::Srgba(colour)));
            commands.entity(entity).insert((preferred.clone(), material));
        }
    }
}

fn create_mesh(data: ndarray::ArrayView2<f32>, scale: &Vec3) -> Mesh {
    let mut heights = Vec::new();
    for row in data.rows() {
        let mut height_row = Vec::new();
        for x in row {
            height_row.push(*x);
        }
        heights.push(height_row);
    }

    let mesh = heightmap_to_mesh(&heights, scale);

    mesh
}

fn create_material(colour: Color) -> StandardMaterial {
    StandardMaterial::from(colour)
}
