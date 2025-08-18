use bevy::asset::Assets;
use bevy::prelude::{info, ChildOf, Children, Commands, Component, Cuboid, DetectChanges, Entity, Mesh, Mesh3d, MeshMaterial3d, Mut, Res, ResMut, Single, Transform, Vec3, Visibility, With};

use crate::level::LevelLabel;
use crate::terrain::rendering::TerrainRenderParams;
use crate::terrain::Terrain;

#[derive(Component)]
pub struct WaterLabel;

pub fn update_water(
    terrain: Single<Mut<Terrain>, With<LevelLabel>>,
    water: Single<Entity, With<WaterLabel>>,
    params: Res<TerrainRenderParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let water_id = *water;

    if !terrain.is_changed() { return; }

    commands.entity(water_id).despawn_related::<Children>();

    let size = Vec3::new(terrain.size[1] as f32, 0.01, terrain.size[0] as f32);
    let mesh: Mesh = Cuboid::from_size(size).into();
    let mesh = mesh.translated_by(size/2.0);
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(params.water_material.clone()),
        ChildOf(water_id)
   ));

    info!("Creating water of size {size}");
}
