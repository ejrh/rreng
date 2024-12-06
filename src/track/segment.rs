use bevy::log::info;
use bevy::prelude::{Changed, Component, DetectChangesMut, Entity, Query, Transform, Vec3, Visibility, With, Without};

use crate::track::point::Point;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Segment {
    pub(crate) from_point: Entity,
    pub(crate) to_point: Entity,
    pub(crate) length: f32,
}

pub fn update_points(
    changed_points: Query<(), Changed<Point>>,
    mut segments: Query<&mut Segment>,
) {
    if changed_points.is_empty() { return }

    for mut seg in segments.iter_mut() {
        if changed_points.contains(seg.from_point) || changed_points.contains(seg.to_point) {
            seg.set_changed();
            info!("Updated points");
        }
    }
}

pub fn update_segments(
    mut segments: Query<(&mut Segment, &mut Transform), Changed<Segment>>,
    all_points: Query<&Transform, (With<Point>, Without<Segment>)>,
) {
    for (mut seg, mut transform) in segments.iter_mut() {
        let pt1 = all_points.get(seg.from_point).unwrap();
        let pt2 = all_points.get(seg.to_point).unwrap();
        seg.length = pt1.translation.distance(pt2.translation);
        transform.translation = pt1.translation;
        transform.look_to(pt1.translation - pt2.translation, Vec3::Y);
        info!("Updated segment position");
    }
}
