use bevy::prelude::{App, IntoSystemConfigs, Plugin, PostUpdate, Startup, Update};

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
            .add_systems(PostUpdate, point::render_points)
            .add_systems(PostUpdate, rendering::update_track_meshes);
    }
}
