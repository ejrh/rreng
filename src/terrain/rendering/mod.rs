use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_asset::RenderAssetUsages
    }};
use bevy::render::mesh::MeshAabb;
use bevy::render::primitives::Aabb;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use bevy::tasks::futures_lite::future;
use ndarray::{s, Array2};

use crate::events::GraphicsEvent;
use crate::terrain::heightmap::heightmap_to_mesh;
use crate::terrain::rtin::{triangulate_rtin, Triangle, Triangulation};
use crate::terrain::{Terrain, TerrainData, TerrainLayer};
use crate::terrain::rendering::mesh_tree::{BlockId, BlockKind, MeshTree};
use crate::terrain::utils::Range2;

pub mod mesh_tree;
pub mod water;

pub(crate) struct TerrainRenderingPlugin;

impl Plugin for TerrainRenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<TerrainMesh>()
            .add_systems(Startup, init_render_params)
            .init_resource::<MeshTaskQueue>()
            .add_systems(Update, water::update_water.run_if(resource_changed::<Terrain>))
            .add_systems(Update, (
                update_parents,
                update_meshes,
                handle_mesh_tasks,
                select_meshes,
            ).chain());
    }
}

#[derive(Component, Debug, Reflect)]
pub struct TerrainMesh {
    pub layer: TerrainLayer,
    pub block_id: BlockId,
}

const RENDERS_PER_FRAME: usize = 16;
const MAX_MESH_TREE_LEVEL: usize = 4;

#[derive(Resource)]
pub struct TerrainRenderParams {
    parent_id: HashMap<TerrainLayer, Entity>,
    dirt_material: Handle<StandardMaterial>,
    grass_material: Handle<StandardMaterial>,
    water_id: Option<Entity>,
    water_material: Handle<StandardMaterial>,
}

pub struct MeshTask {
    terrain_mesh: TerrainMesh,
    transform: Transform,
    material: Handle<StandardMaterial>,
    task: Task<Mesh>,
}

#[derive(Default, Resource)]
pub struct MeshTaskQueue(Vec<MeshTask>);

pub fn init_render_params(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mut dirt_material = StandardMaterial::from(Color::srgb(0.51, 0.25, 0.03));
    dirt_material.perceptual_roughness = 0.5;
    dirt_material.reflectance = 0.1;
    let mut grass_material = StandardMaterial::from(Color::srgba(0.3, 0.6, 0.2, 0.75));
    grass_material.perceptual_roughness = 0.75;
    grass_material.reflectance = 0.25;
    let mut water_material = StandardMaterial::from(Color::srgb(0.25, 0.41, 0.88));
    water_material.perceptual_roughness = 0.75;
    water_material.reflectance = 0.25;
    let params = TerrainRenderParams {
        parent_id: HashMap::new(),
        dirt_material: materials.add(dirt_material),
        grass_material: materials.add(grass_material),
        water_id: None,
        water_material: materials.add(water_material),
    };
    commands.insert_resource(params);
}

pub fn update_parents(
    terrain: Res<Terrain>,
    terrain_data: Res<TerrainData>,
    mut params: ResMut<TerrainRenderParams>,
    mut commands: Commands,
) {
    for layer in terrain_data.layers.keys() {
        params.parent_id.entry(*layer).or_insert_with(|| {
            let tree = MeshTree::new(terrain.num_blocks, MAX_MESH_TREE_LEVEL);
            info!("Mesh tree with {} levels", tree.levels.len());

            commands.spawn((
                Name::new(format!("Terrain:{:?}", layer)),
                Visibility::default(),
                Transform::default(),
                tree,
            )).id()
        });
    }
}

pub fn update_meshes(
    terrain: Res<Terrain>,
    mut terrain_data: ResMut<TerrainData>,
    params: Res<TerrainRenderParams>,
    mesh_trees: Query<&MeshTree>,
    mut mesh_task_queue: ResMut<MeshTaskQueue>,
) {
    /* Collect affected blocks */
    let blocks: Vec<_> = terrain_data.block_info.indexed_iter()
        .filter_map(|((i, j), bi)| if bi.dirty { Some((i, j))} else { None })
        .take(RENDERS_PER_FRAME)
        .collect();

    if blocks.is_empty() { return; }

    info!("Updating {} blocks", blocks.len());

    /* Process each layer */
    for (layer, elevation) in &terrain_data.layers {
        /* Get the parent, mesh tree, and various render settings for this layer */
        let parent_id = params.parent_id[layer];
        let tree = mesh_trees.get(parent_id).unwrap();

        let (layer_height_adjust, layer_material) = match layer {
            TerrainLayer::Elevation => (0.0, params.dirt_material.clone()),
            TerrainLayer::Structure => (-1.0, params.grass_material.clone()),
        };

        /* Figure out which blocks are needed */
        let mut blocks_needed = Vec::new();
        for (i, j) in &blocks {
            let block_id = BlockId { row: *i, col: *j, level: 0 };
            blocks_needed.push(block_id);
            blocks_needed.extend(tree.ancestors(block_id));
        }
        blocks_needed.sort_by_key(|b| usize::MAX - b.level);
        blocks_needed.dedup();

        /* For each block, recreate its meshes */
        for block in &blocks_needed {
            if !tree.valid(*block) { continue; }

            let range = block_range(terrain.block_size, *block);
            let (threshold, spacing) = block_quality(*block);

            let terrain_mesh = TerrainMesh {
                layer: *layer,
                block_id: *block,
            };
            let level_size = terrain.block_size * (1 << block.level);
            let xp = block.col as f32 * level_size as f32;
            let yp = block.row as f32 * level_size as f32;
            let transform = Transform::from_xyz(xp, layer_height_adjust, yp);
            let material = layer_material.clone();

            queue_mesh_task(
                terrain_mesh,
                transform,
                material,
                threshold,
                spacing,
                elevation.clone(),
                range.clone(),
                &mut mesh_task_queue.0
            );
        }
    }

    /* Reset these blocks' dirty flag */
    for (i, j) in &blocks {
        terrain_data.block_info[(*i, *j)].dirty = false;
    }
}

fn create_mesh(data: ndarray::ArrayView2<f32>, scale: &Vec3, threshold: f32) -> Mesh {
    let _span = info_span!("create.mesh").entered();

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

fn queue_mesh_task(
    terrain_mesh: TerrainMesh,
    transform: Transform,
    material: Handle<StandardMaterial>,
    threshold: f32,
    spacing: i32,
    data: Arc<Mutex<Array2<f32>>>,
    range: Range2,
    queue: &mut Vec<MeshTask>
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(async move {
        let data = data.lock().unwrap();
        let elevation_view = data.slice(s!(range.0.clone();spacing, range.1.clone();spacing));
        create_mesh(elevation_view, &Vec3::new(spacing as f32, 1.0, spacing as f32), threshold)
    });

    queue.push(MeshTask {
        terrain_mesh,
        transform,
        material,
        task,
    });
}

pub fn handle_mesh_tasks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_task_queue: ResMut<MeshTaskQueue>,
    params: Res<TerrainRenderParams>,
    mut mesh_trees: Query<&mut MeshTree>,
    mut commands: Commands,
    mut events: EventWriter<GraphicsEvent>,
) {
    if mesh_task_queue.0.is_empty() { return; }

    let old_queue = std::mem::take(&mut mesh_task_queue.0);

    let mut any_change = false;

    for mut mt in old_queue {
        if let Some(mesh) = block_on(future::poll_once(&mut mt.task)) {
            let Some(aabb) = mesh.compute_aabb()
                else { warn!("Could not compute Aabb for terrain mesh"); continue; };

            let handle = meshes.add(mesh);

            let block_id = mt.terrain_mesh.block_id;
            let layer = mt.terrain_mesh.layer;
            let parent_id = params.parent_id[&layer];
            let mut tree = mesh_trees.get_mut(parent_id).unwrap();

            let id = commands.spawn((
                mt.terrain_mesh,
                Mesh3d(handle),
                MeshMaterial3d(mt.material),
                mt.transform,
                Visibility::Hidden,
                aabb,
            ))
                .set_parent(parent_id)
                .id();

            if let Some(old_id) = tree.set_mesh(block_id, BlockKind::Populated(id)) {
                commands.entity(old_id).despawn_recursive();
            }

            any_change = true;
        } else {
            mesh_task_queue.0.push(mt);
        }
    }

    if any_change {
        events.send(GraphicsEvent::RenderTerrain);
    }
}

pub fn select_meshes(
    terrain: Res<Terrain>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mesh_trees: Query<&mut MeshTree>,
    mut meshes: Query<(&GlobalTransform, &Aabb, &mut Visibility), Without<Camera>>,
    mut events: EventReader<GraphicsEvent>,
) {
    const TOLERANCE: f32 = 50.0;

    let Ok(camera_transform) = camera_query.get_single()
    else { return };

    if !events.read().any(|e|
        matches!(e, GraphicsEvent::MoveCamera | GraphicsEvent::RenderTerrain)
    ) {
        return;
    }

    fn set_vis(vis: &mut Mut<Visibility>, new_value: Visibility) {
        if **vis != new_value {
            **vis = new_value;
        }
    }

    for tree in &mesh_trees {
        tree.walk(&mut |tree, block_id| {
            let entry = tree.get_entry(block_id);
            let descend = match entry.kind {
                BlockKind::Populated(entity) => {
                    if let Ok((mesh_transform, aabb, mut vis)) = meshes.get_mut(entity) {
                        let mut too_close = false;
                        if block_id.level > 0 {
                            let centre = mesh_transform.translation() + Vec3::from(aabb.center);
                            let distance = centre.distance(camera_transform.translation());
                            let cutoff = terrain.block_size as f32 * 4.0 * (1 << block_id.level) as f32;
                            too_close = distance < cutoff;

                            /* Check all children are populated; if not, then just use this block */
                            if too_close && !tree.children(block_id).iter()
                                .all(|child| tree.populated(*child)) {
                                too_close = false;
                            }
                        }

                        if too_close {
                            set_vis(&mut vis, Visibility::Hidden);
                        } else {
                            set_vis(&mut vis, Visibility::Inherited);
                        }

                        too_close
                    } else {
                        warn!("no mesh?");
                        true
                    }
                },
                _ => true
            };

            /* Hide all the descendents */
            if !descend {
                for child in tree.descendants(block_id) {
                    if child == block_id { continue; }
                    if let BlockKind::Populated(entity) = tree.get_entry(child).kind {
                        if let Ok((_, _, mut vis)) = meshes.get_mut(entity) {
                            set_vis(&mut vis, Visibility::Hidden);
                        }
                    }
                }
            }

            descend
        });
    }
}

fn block_range(block_size: usize, block_id: BlockId) -> Range2 {
    let (row, col) = (block_id.row, block_id.col);
    let level_block_size = (1 << block_id.level) * block_size;
    Range2(row * level_block_size..(row+1) * level_block_size + 1,
           col * level_block_size..(col+1) * level_block_size + 1)
}

fn block_quality(block_id: BlockId) -> (f32, i32) {
    let quality = 0.125f32 + 1.5f32.powi(block_id.level as i32);
    let spacing = 1 << block_id.level;
    (quality, spacing)
}
