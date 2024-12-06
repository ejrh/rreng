use bevy::asset::Assets;
use bevy::color::{Color, LinearRgba};
use bevy::pbr::{MeshMaterial3d, NotShadowCaster, NotShadowReceiver, StandardMaterial};
use bevy::prelude::{Changed, Commands, Component, Entity, Mesh, Mesh3d, Query, ResMut, Sphere};

use crate::utils::ConstantApparentSize;

#[derive(Component)]
pub struct Point;

pub fn render_points(
    changed_points: Query<Entity, Changed<Point>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for point in changed_points.iter() {
        let mesh: Mesh = Sphere::new(1.0).into();
        let mut material = StandardMaterial::from_color(Color::srgb(0.25, 0.25, 0.5));
        material.emissive = LinearRgba::rgb(0.0, 0.0, 1.0);
        commands.entity(point).insert((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(material)),
            ConstantApparentSize(100.0..250.0),
            NotShadowCaster,
            NotShadowReceiver,
        ));
    }
}
