use bevy::asset::Assets;
use bevy::prelude::{BuildChildren, Commands, Cuboid, Mesh, Mesh3d, MeshMaterial3d, Name, Res, ResMut, Transform, Vec3, Visibility};

use crate::terrain::rendering::TerrainRenderParams;
use crate::terrain::Terrain;

pub fn update_water(
    terrain: Res<Terrain>,
    mut params: ResMut<TerrainRenderParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if params.water_id.is_some() {
        return;
    }

    let water_id = commands.spawn((
        Name::new("Water"),
        Visibility::default(),
        Transform::default(),
    )).id();
    params.water_id = Some(water_id);

    let size = Vec3::new(terrain.size[1] as f32, 0.01, terrain.size[0] as f32);
    let mesh: Mesh = Cuboid::from_size(size).into();
    let mesh = mesh.translated_by(size/2.0);
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(params.water_material.clone()),
   )).set_parent(water_id);
}
