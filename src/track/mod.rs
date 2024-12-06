use bevy::prelude::*;
use crate::track::point::Point;
use crate::track::segment::Segment;

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
            .add_systems(Startup, rendering::init_render_params)
            .add_systems(Update, (segment::update_points, segment::update_segments).chain())
            .add_systems(Update, point::render_points)
            .add_systems(Update, rendering::update_track_meshes);
    }
}

pub fn create_track(
    mut commands: Commands,
) {
    let points = [
        Vec3::new(480.0, 216.02, 1800.0),
        Vec3::new(480.0, 215.98, 1760.0),
        Vec3::new(475.0, 216.07, 1730.0),
        Vec3::new(460.0, 216.05, 1690.0),
        Vec3::new(445.0, 215.93, 1665.0),
        Vec3::new(410.0, 216.33, 1640.0),
        Vec3::new(390.0, 216.53, 1630.0),
        Vec3::new(367.0, 216.13, 1632.0),
        Vec3::new(356.0, 215.03, 1638.0),
        Vec3::new(347.0, 212.60, 1657.0),
        Vec3::new(353.0, 210.37, 1676.0),
        Vec3::new(386.0, 204.94, 1704.0),
        Vec3::new(403.0, 201.14, 1736.0),
    ];

    let point_ids = points.iter()
        .map(|pt| commands.spawn((Point, Transform::from_translation(*pt))).id())
        .collect::<Vec<_>>();

    for w in point_ids.windows(2) {
        let [pt1, pt2, ..] = w else { continue; };
        commands.spawn(Segment {
            from_point: *pt1,
            to_point: *pt2,
            length: 0.0,
            next_segment: Entity::from_raw(0),
            prev_segment: Entity::from_raw(0),
        });
    }
}
