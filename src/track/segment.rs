use std::collections::HashMap;
use bevy::log::info;
use bevy::prelude::{Changed, Component, DetectChanges, DetectChangesMut, Entity, Query, Ref, Reflect, Transform, Vec3, Visibility, With, Without};

use crate::track::point::Point;

#[derive(Component, Reflect)]
#[require(SegmentLinkage, Transform, Visibility)]
pub struct Segment {
    pub from_point: Entity,
    pub to_point: Entity,
    pub length: f32,
}

#[derive(Component, Default, Reflect)]
pub struct SegmentLinkage {
    pub next_segment: Option<Entity>,
    pub prev_segment: Option<(Entity, f32)>,
}

pub fn update_points(
    changed_points: Query<(), Changed<Point>>,
    mut segments: Query<&mut Segment>,
) {
    if changed_points.is_empty() { return }

    for mut seg in segments.iter_mut() {
        if changed_points.contains(seg.from_point) || changed_points.contains(seg.to_point) {
            seg.set_changed();
        }
    }
}

pub fn update_segment_linkage(
    mut segments: Query<(Entity, Ref<Segment>, &mut SegmentLinkage)>,
) {
    if !segments.iter().any(|(_, s, _)| s.is_changed()) {
        return;
    }

    info!("calculating segment linkage");

    let mut point_begins = HashMap::new();
    let mut point_ends = HashMap::new();
    for (seg_id, seg, _) in segments.iter() {
        point_begins.insert(seg.from_point, seg_id);
        point_ends.insert(seg.to_point, (seg_id, seg.length));
    }

    for (_, seg, mut linkage) in segments.iter_mut() {
        linkage.next_segment = point_begins.get(&seg.to_point).copied();
        linkage.prev_segment = point_ends.get(&seg.from_point).copied();
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
    }
}
