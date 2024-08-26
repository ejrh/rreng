use bevy::asset::Assets;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{default, Color, Commands, Mesh, ResMut};
use noise::{NoiseFn, Simplex};

use crate::terrain::heightmap::heightmap_to_mesh;

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
