use std::collections::HashMap;
use std::mem::take;
use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        primitives::Aabb,
        render_asset::RenderAssetUsages
}};
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use bevy::tasks::futures_lite::future;
use ndarray::{s, Array2};

use crate::terrain::heightmap::heightmap_to_mesh;
use crate::terrain::rtin::{triangulate_rtin, Triangle, Triangulation};
use crate::terrain::{BlockInfo, Terrain, TerrainLayer};
use crate::terrain::utils::Range2;

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

const RENDERS_PER_FRAME: usize = 10;

#[derive(Resource)]
pub struct TerrainRenderParams {
    parent_id: HashMap<TerrainLayer, Entity>,
    dirt_material: Handle<StandardMaterial>,
    grass_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    high_res_cutoff: f32,
}

pub struct MeshTask(Handle<Mesh>, Task<Mesh>);

#[derive(Default, Resource)]
pub struct MeshTaskQueue(Vec<MeshTask>);

pub fn init_render_params(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mut dirt_material = StandardMaterial::from(Color::srgb(0.51, 0.25, 0.03));
    dirt_material.perceptual_roughness = 0.5;
    dirt_material.reflectance = 0.1;
    let mut grass_material = StandardMaterial::from(Color::srgb(0.3, 0.6, 0.2));
    grass_material.perceptual_roughness = 0.75;
    grass_material.reflectance = 0.25;
    let mut water_material = StandardMaterial::from(Color::srgb(0.25, 0.41, 0.88));
    water_material.perceptual_roughness = 0.75;
    water_material.reflectance = 0.25;
    let params = TerrainRenderParams {
        parent_id: HashMap::new(),
        dirt_material: materials.add(dirt_material),
        grass_material: materials.add(grass_material),
        water_material: materials.add(water_material),
        high_res_cutoff: 1000.0,
    };
    commands.insert_resource(params);
}

pub fn update_meshes(
    mut terrain: ResMut<Terrain>,
    mut params: ResMut<TerrainRenderParams>,
    terrain_meshes: Query<(Entity, &TerrainMesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_task_queue: ResMut<MeshTaskQueue>,
    mut commands: Commands,
) {
    /* Explicitly borrow the object so we can simultaneously borrow different fields of it later */
    let terrain = &mut *terrain;

    /* Collect affected blocks */
    let blocks: Vec<&mut BlockInfo> = terrain.block_info.iter_mut()
        .filter(|bi| bi.dirty)
        .take(RENDERS_PER_FRAME)
        .collect();

    /* Despawn existing meshes for these blocks */
    for block_info in &blocks {
        for (e, tm) in terrain_meshes.iter() {
            if tm.block_num == block_info.block_num {
                commands.entity(e).despawn_recursive();
            }
        }
    }

    /* Process each layer */
    for (layer, elevation) in &terrain.layers {
        let parent_id = *params.parent_id.entry(*layer).or_insert_with(||
            commands.spawn((
                Name::new(format!("Terrain:{:?}", layer)),
                Visibility::default(),
                Transform::default()
            )).id());

        let (layer_height_adjust, layer_material) = match layer {
            TerrainLayer::Elevation => (0.0, params.dirt_material.clone()),
            TerrainLayer::Structure => (-1.0, params.grass_material.clone()),
        };

        /* For each block, recreate its meshes */
        for block_info in &blocks {
            let high_res = queue_mesh_task(0.1, 1, elevation.clone(), block_info.range.clone(), &mut meshes, &mut mesh_task_queue.0);
            let low_res = queue_mesh_task(0.5, 1, elevation.clone(), block_info.range.clone(), &mut meshes, &mut mesh_task_queue.0);

            let xp = block_info.block_num.1 as f32 * terrain.block_size as f32;
            let yp = block_info.block_num.0 as f32 * terrain.block_size as f32;
            let transform = Transform::from_xyz(xp, layer_height_adjust, yp);

            let mesh = low_res.clone();

            let alternates = TerrainMeshAlternates {
                cutoff: params.high_res_cutoff,
                high_res,
                low_res,
            };

            commands.spawn((
                TerrainMesh { block_num: block_info.block_num },
                Mesh3d(mesh),
                MeshMaterial3d(layer_material.clone()),
                transform,
                alternates,
            ))
                .remove::<Aabb>()
                .set_parent(parent_id);

            if matches!(layer, TerrainLayer::Elevation) {
                let spacing = 1;
                let _guard = elevation.lock().unwrap();
                let elevation = &*_guard;
                let elevation_view = elevation.slice(s!(block_info.range.0.clone();spacing, block_info.range.1.clone();spacing));
                if elevation_view.iter().copied().reduce(f32::min).unwrap() <= 0.01f32 {
                    let size = Vec3::new(terrain.block_size as f32, 0.01, terrain.block_size as f32);
                    let mesh: Mesh = Cuboid::from_size(size).into();
                    let mesh = mesh.translated_by(size/2.0);
                    commands.spawn((
                        TerrainMesh { block_num: block_info.block_num },
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(params.water_material.clone()),
                        transform
                   )).set_parent(parent_id);
                }
            }
        }
    }

    /* Reset these blocks' dirty flag */
    for block_info in blocks {
        block_info.dirty = false;
    }

    /* Clean up orphaned terrain meshes */
    // for (entity, terrain_mesh) in terrain_meshes.iter() {
    //     let block_info = &terrain.block_info[terrain_mesh.block_num];
    //     if block_info.mesh_entity != Some(entity) {
    //         commands.entity(entity).despawn();
    //     }
    // }
}

pub fn swap_mesh_alternates(
    camera_query: Query<&GlobalTransform, (With<Camera>, Changed<GlobalTransform>)>,
    mesh_alternates: Query<(Entity, &Mesh3d, &GlobalTransform, &TerrainMeshAlternates)>,
    mut commands: Commands,
) {
    const TOLERANCE: f32 = 50.0;

    let Ok(camera_transform) = camera_query.get_single() else { return };

    for (entity, Mesh3d(current), mesh_transform, alternates) in mesh_alternates.iter() {
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

            commands.entity(entity).insert(Mesh3d(preferred.clone()));
        }
    }
}

fn create_mesh(data: ndarray::ArrayView2<f32>, scale: &Vec3, threshold: f32) -> Mesh {
    if threshold == 0.0 {
        heightmap_to_mesh(&data, scale)
    } else {
        let Triangulation { triangles } = triangulate_rtin(&data, threshold);

        let mut pos  = Vec::new();
        for Triangle { points } in &triangles {
            let p = points.map(|[r,c]| {
                let h = data[(r, c)];
                Vec3::new(c as f32 * scale.x, h, r as f32 * scale.z)
            });
            pos.extend(p);
        }
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
        mesh.compute_flat_normals();
        mesh
    }
}

fn queue_mesh_task(threshold: f32, spacing: i32, data: Arc<Mutex<Array2<f32>>>, range: Range2, meshes: &mut Assets<Mesh>, queue: &mut Vec<MeshTask>) -> Handle<Mesh> {
    let handle = meshes.reserve_handle();

    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(async move {
        let data = data.lock().unwrap();
        let elevation_view = data.slice(s!(range.0.clone();spacing, range.1.clone();spacing));
        create_mesh(elevation_view, &Vec3::new(spacing as f32, 1.0, spacing as f32), threshold)
    });

    queue.push(MeshTask(handle.clone(), task));

    handle
}

pub fn handle_mesh_tasks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_task_queue: ResMut<MeshTaskQueue>,
) {
    if mesh_task_queue.0.is_empty() { return; }

    let old_queue = take(&mut mesh_task_queue.0);

    for mut mt in old_queue {
        if let Some(mesh) = block_on(future::poll_once(&mut mt.1)) {
            meshes.insert(mt.0.id(), mesh);
        } else {
            mesh_task_queue.0.push(mt);
        }
    }
}
