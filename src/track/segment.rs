use std::collections::HashMap;

use bevy::log::info;
use bevy::prelude::{Changed, Component, DetectChanges, DetectChangesMut, Entity, Or, Query, Ref, Reflect, Transform, Vec3, Visibility, Without};

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

pub fn update_segment_linkage(
    mut segments: Query<(Entity, Ref<Segment>, &mut SegmentLinkage)>,
) {
    if !segments.iter().any(|(_, s, _)| s.is_changed()) {
        return;
    }

    info!("Calculating segment linkage");

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
    mut segments: Query<(&mut Segment, &mut Transform), Or<(Changed<Segment>, Changed<Transform>)>>,
    mut all_points: Query<(&Transform, &mut Point), Without<Segment>>,
) {
    if segments.is_empty() { return; }

    info!("Updating segments");

    for (mut seg, mut transform) in segments.iter_mut() {
        let (pt1, _) = all_points.get(seg.from_point).unwrap();
        let (pt2, _) = all_points.get(seg.to_point).unwrap();
        seg.length = pt1.translation.distance(pt2.translation);

        transform.translation = pt1.translation;
        transform.look_to(pt1.translation - pt2.translation, Vec3::Y);

        all_points.get_mut(seg.from_point).unwrap().1.set_changed();
        all_points.get_mut(seg.to_point).unwrap().1.set_changed();
    }
}
