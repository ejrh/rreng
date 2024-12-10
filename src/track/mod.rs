use bevy::prelude::*;

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
            .register_type::<point::Point>()
            .register_type::<segment::Segment>()
            .register_type::<segment::SegmentLinkage>()
            .add_systems(Startup, rendering::init_render_params)
            .add_systems(Update, (segment::update_points, segment::update_segments, segment::update_segment_linkage).chain())
            .add_systems(Update, point::render_points)
            .add_systems(Update, point::update_point_angles)
            .add_systems(Update, rendering::update_track_meshes);
    }
}
