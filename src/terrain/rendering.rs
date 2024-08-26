use std::ops::{Deref, DerefMut};
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

pub fn update_meshes(
    mut terrain: ResMut<Terrain>,
    terrain_meshes: Query<(Entity, &TerrainMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let terrain = terrain.deref_mut();

    for block_info in terrain.block_info.iter_mut() {
        if !block_info.dirty { continue; }

        block_info.dirty = false;

        let entity = block_info.mesh_entity
            .filter(|e| terrain_meshes.contains(*e))
            .unwrap_or_else(|| {
            let entity = commands.spawn(TerrainMesh { block_num: block_info.block_num }).id();
            block_info.mesh_entity = Some(entity);
            entity
        });

        let spacing = 1;

        let elevation_view = terrain.elevation.slice(s!(block_info.range.0.clone();spacing, block_info.range.1.clone();spacing));
        let colour = if block_info.block_num.0 + block_info.block_num.1 == 0 { Color::RED }
            else if (block_info.block_num.0 + block_info.block_num.1) & 1 == 0 { Color::GREEN }
            else { Color::BLUE };
        let material = materials.add(create_material(colour));
        let mesh = meshes.add(create_mesh(elevation_view, &Vec3::new(spacing as f32, 1.0, spacing as f32)));
        let xp = block_info.block_num.1 as f32 * terrain.block_size as f32;
        let yp = block_info.block_num.0 as f32 * terrain.block_size as f32;
        let transform = Transform::from_xyz(xp, 0.0, yp);

        commands.entity(entity).insert(MaterialMeshBundle {
            material,
            mesh,
            transform,
            ..default()
        });
        info!("spawned terrain mesh");
    }

    for (entity, terrain_mesh) in terrain_meshes.iter() {
        let block_info = &terrain.block_info[terrain_mesh.block_num];
        if block_info.mesh_entity != Some(entity) {
            info!("despawn orphaned terrain mesh");
            commands.entity(entity).despawn();
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
