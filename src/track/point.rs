use std::collections::HashMap;

use bevy::asset::Assets;
use bevy::color::{Color, LinearRgba};
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, NotShadowCaster, NotShadowReceiver, StandardMaterial};
use bevy::prelude::{Changed, Commands, Component, Entity, Mesh, Mesh3d, Quat, Query, Reflect, ResMut, Sphere, Transform, With, Without};

use crate::track::segment::Segment;
use crate::utils::ConstantApparentSize;

#[derive(Component, Reflect)]
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

pub fn update_point_angles(
    mut points: Query<(Entity, &mut Transform), With<Point>>,
    segments: Query<(&Segment, &Transform), Without<Point>>,
) {
    let mut angles = HashMap::new();
    for (id, _) in points.iter_mut() {
        angles.insert(id, Vec3::default());
    }

    for (segment, seg_transform) in segments.iter() {
        *(angles.get_mut(&segment.to_point).unwrap()) += seg_transform.forward().as_vec3();
        *(angles.get_mut(&segment.from_point).unwrap()) += seg_transform.forward().as_vec3();
    }

    for (id, mut transform) in points.iter_mut() {
        let angle = angles[&id].normalize();
        transform.rotation = Quat::from_rotation_arc(Vec3::Z, angle);
    }
}
