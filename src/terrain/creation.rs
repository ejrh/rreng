use bevy::asset::Assets;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{default, Color, Commands, Mesh, ResMut};
use noise::{NoiseFn, Simplex};

use crate::terrain::heightmap::heightmap_to_mesh;

pub(crate) fn _create_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let noise = Simplex::new(42);
    const GRID_SIZE: usize = 10;
    let grid = ndarray::Array2::from_shape_fn(
        (GRID_SIZE + 1, GRID_SIZE + 1),
        |(r, c)| {
            noise.get([r as f64 / GRID_SIZE as f64 * 1.0, c as f64 / GRID_SIZE as f64 * 1.0]) as f32
        }
    );

    const GRID_SPACING: f32 = 100.0 / GRID_SIZE as f32;

    let mesh = heightmap_to_mesh(&grid.view(), &Vec3::new(GRID_SPACING, 25.0, GRID_SPACING));
    let mesh = meshes.add(mesh);

    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.8, 0.4),
            perceptual_roughness: 0.9,
            ..default()
        }),
        ..default()
    });
}
