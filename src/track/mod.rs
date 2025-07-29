use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{App, ChildOf, Commands, Entity, IntoScheduleConfigs, Name, Plugin, PostUpdate, Startup, Transform, Update, Visibility};

use crate::track::point::Point;
use crate::track::segment::{Segment, SegmentLinkage};

pub mod point;
pub mod rendering;
pub mod segment;

/**
 * Height of rail surface above ground level.
 * Trains sit at this level; rails, sleepers, and bed are rendered below it down to ground level.
 * Note that rail sections and their control points sit at ground level.
 */
pub const TRACK_HEIGHT: f32 = 0.5;

#[derive(Default)]
pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Point>()
            .register_type::<Segment>()
            .register_type::<SegmentLinkage>()
            .add_systems(Startup, rendering::init_render_params)
            .add_systems(Update, (
                point::move_points,
                segment::update_segments,
                point::update_point_angles,
                segment::update_segment_linkage
            ).chain())
            .add_systems(PostUpdate, rendering::update_track_meshes);
    }
}

pub fn create_track(
    name: &str,
    points: &[Vec3],
    looped: bool,
    commands: &mut Commands
) -> (Entity, Vec<Entity>, Vec<Entity>) {
    let parent_id = commands
        .spawn((
            Name::new(format!("Track:{name}")),
            Visibility::default(),
            Transform::default(),
        )).id();

    let mut point_ids = points.iter()
        .map(|pt| commands.spawn((
            Point,
            Transform::from_translation(*pt),
            ChildOf(parent_id)
        )).id())
        .collect::<Vec<_>>();

    if looped {
        point_ids.push(point_ids[0]);
    }

    let segment_ids: Vec<_> = point_ids.windows(2).map(|w| {
        let [pt1, pt2, ..] = w else { panic!("Expect window of size 2") };
        commands.spawn((
            Segment {
                from_point: *pt1,
                to_point: *pt2,
                length: 0.0,
            },
            ChildOf(parent_id)
        )).id()
    }).collect();
    info!("created track with {} segments", segment_ids.len());

    (parent_id, point_ids, segment_ids)
}
