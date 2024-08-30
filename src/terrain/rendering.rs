use bevy::prelude::*;
use ndarray::s;

use crate::terrain::heightmap::heightmap_to_mesh;
use crate::terrain::terrain::Terrain;
use crate::terrain::utils::get_copyable_range;

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

#[derive(Resource)]
pub struct TerrainRenderParams {
    grass_material: Handle<StandardMaterial>,
    high_res_cutoff: f32,
}

pub fn init_render_params(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mut grass_material = StandardMaterial::from(Color::srgb(0.3, 0.6, 0.2));
    grass_material.perceptual_roughness = 0.75;
    grass_material.reflectance = 0.25;
    let params = TerrainRenderParams {
        grass_material: materials.add(grass_material),
        high_res_cutoff: 1000.0,
    };
    commands.insert_resource(params);
}

pub fn update_meshes(
    mut terrain: ResMut<Terrain>,
    params: Res<TerrainRenderParams>,
    terrain_meshes: Query<(Entity, &TerrainMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
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

        let xp = block_info.block_num.1 as f32 * terrain.block_size as f32;
        let yp = block_info.block_num.0 as f32 * terrain.block_size as f32;
        let transform = Transform::from_xyz(xp, 0.0, yp);

        let mesh = low_res.clone();

        let alternates = TerrainMeshAlternates {
            cutoff: params.high_res_cutoff,
            high_res,
            low_res,
        };

        commands.entity(entity).insert(MaterialMeshBundle {
            material: params.grass_material.clone(),
            mesh,
            transform,
            ..default()
        }).insert(alternates);
    }

    /* Clean up orphaned terrain meshes */
    for (entity, terrain_mesh) in terrain_meshes.iter() {
        let block_info = &terrain.block_info[terrain_mesh.block_num];
        if block_info.mesh_entity != Some(entity) {
            commands.entity(entity).despawn();
        }
    }
}

pub fn swap_mesh_alternates(
    camera_query: Query<&GlobalTransform, (With<Camera>, Changed<GlobalTransform>)>,
    mesh_alternates: Query<(Entity, &Handle<Mesh>, &GlobalTransform, &TerrainMeshAlternates)>,
    mut commands: Commands,
) {
    const TOLERANCE: f32 = 50.0;

    let Ok(camera_transform) = camera_query.get_single() else { return };

    for (entity, current, mesh_transform, alternates) in mesh_alternates.iter() {
        let dist = mesh_transform.translation().distance(camera_transform.translation());

        let diff = dist - alternates.cutoff;

        let preferred = if diff < 0.0 {
            &alternates.high_res
        } else {
            &alternates.low_res
        };

        if !current.eq(preferred) {
            if diff.abs() < TOLERANCE {
                continue;
            }

            commands.entity(entity).insert(preferred.clone());
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