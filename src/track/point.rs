use std::collections::HashMap;

use bevy::asset::Assets;
use bevy::color::{Color, LinearRgba};
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, NotShadowCaster, NotShadowReceiver, StandardMaterial};
use bevy::prelude::{info, Changed, Commands, Component, DetectChangesMut, Entity, Mesh, Mesh3d, Quat, Query, Reflect, ResMut, Sphere, Transform, With, Without};

use crate::track::segment::Segment;
use crate::utils::ConstantApparentSize;

#[derive(Component, Reflect)]
#[require(Transform)]
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

pub fn move_points(
    changed_points: Query<(), (Changed<Transform>, With<Point>, Without<Segment>)>,
    mut segments: Query<(&Segment, &mut Transform)>,
) {
    if changed_points.is_empty() { return; }

    info!("Moving points");

    for (seg, mut seg_transform) in segments.iter_mut() {
        if changed_points.contains(seg.from_point) || changed_points.contains(seg.to_point) {
            seg_transform.set_changed();
        }
    }
}

pub fn update_point_angles(
    mut points: Query<(Entity, &mut Transform), Changed<Point>>,
    segments: Query<(&Segment, &Transform), Without<Point>>,
) {
    let mut angles = HashMap::new();
    for (id, _) in points.iter_mut() {
        angles.insert(id, Vec3::default());
    }

    if angles.is_empty() { return; }

    info!("Updating angles for {} points", angles.len());

    for (segment, seg_transform) in segments.iter() {
        if angles.contains_key(&segment.to_point) {
            *(angles.get_mut(&segment.to_point).unwrap()) += seg_transform.forward().as_vec3();
        }
        if angles.contains_key(&segment.from_point) {
            *(angles.get_mut(&segment.from_point).unwrap()) += seg_transform.forward().as_vec3();
        }
    }

    for (id, mut transform) in points.iter_mut() {
        let angle = angles[&id].normalize();
        let new_rotation = Quat::from_rotation_arc(Vec3::Z, angle);
        if transform.rotation != new_rotation {
            transform.rotation = new_rotation;
        }
    }
}
