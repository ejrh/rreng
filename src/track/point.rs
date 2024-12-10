use bevy::asset::Assets;
use bevy::color::{Color, LinearRgba};
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, NotShadowCaster, NotShadowReceiver, StandardMaterial};
use bevy::prelude::{Changed, Commands, Component, Entity, Mesh, Mesh3d, Query, Reflect, ResMut, Sphere, Transform};

use crate::track::segment::{Segment, SegmentLinkage};
use crate::utils::ConstantApparentSize;

#[derive(Component, Reflect)]
#[require(PointAngle)]
pub struct Point;

#[derive(Component, Default)]
pub struct PointAngle {
    pub angle: Vec3,
}

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
    mut points: Query<(Entity, &Transform, &mut PointAngle)>,
    segments: Query<(&Segment, &Transform, &SegmentLinkage)>,
) {
    for (_, _, mut pa) in points.iter_mut() {
        pa.angle = Vec3::default();
    }

    for (segment, seg_transform, linkage) in segments.iter() {
        let (_, _, mut point_angle) = points.get_mut(segment.to_point).unwrap();
        point_angle.angle += seg_transform.forward().as_vec3();
        let (_, _, mut point_angle) = points.get_mut(segment.from_point).unwrap();
        point_angle.angle += seg_transform.forward().as_vec3();
    }

    for (_, _, mut pa) in points.iter_mut() {
        pa.angle /= 2.0;
    }
}
