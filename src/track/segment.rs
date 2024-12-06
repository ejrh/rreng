use std::collections::HashMap;
use bevy::log::info;
use bevy::prelude::{Changed, Component, DetectChanges, DetectChangesMut, Entity, Query, Transform, Vec3, Visibility, With, Without};

use crate::track::point::Point;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Segment {
    pub from_point: Entity,
    pub to_point: Entity,
    pub length: f32,
    pub next_segment: Entity,
    pub prev_segment: Entity,
    pub prev_length: f32,
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
    mut segments: Query<(Entity, &mut Segment, &mut Transform)>,
    all_points: Query<&Transform, (With<Point>, Without<Segment>)>,
) {
    // TODO this is inefficient because:
    //  1. it builds the point-segment maps every frame
    //  2. it processes all segments every frame even if they haven't changed
    //  it needs to do (2) because we need all segments (not just changed) for (1)

    let mut point_begins = HashMap::new();
    let mut point_ends = HashMap::new();
    for (seg_id, seg, _) in segments.iter() {
        point_begins.insert(seg.from_point, seg_id);
        point_ends.insert(seg.to_point, (seg_id, seg.length));
    }

    for (seg_id, mut seg, mut transform) in segments.iter_mut() {
        // if !seg.is_changed() {
        //     continue;
        // }
        let pt1 = all_points.get(seg.from_point).unwrap();
        let pt2 = all_points.get(seg.to_point).unwrap();
        seg.length = pt1.translation.distance(pt2.translation);

        seg.next_segment = *point_begins.get(&seg.to_point).unwrap_or(&Entity::from_raw(0));
        (seg.prev_segment, seg.prev_length) = *point_ends.get(&seg.from_point).unwrap_or(&(Entity::from_raw(0), seg.length));

        transform.translation = pt1.translation;
        transform.look_to(pt1.translation - pt2.translation, Vec3::Y);
    }
}
