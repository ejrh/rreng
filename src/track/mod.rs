use bevy::prelude::*;

pub mod rendering;

/**
 * Height of rail surface above ground level.
 * Trains sit at this level; rails, sleepers, and bed are rendered below it down to ground level.
 * Note that rail sections and their control points sit at ground level.
 */
pub const TRACK_HEIGHT: f32 = 0.5;

#[derive(Component)]
pub struct Segment {
    length: f32,
}

#[derive(Default)]
pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, rendering::init_render_params)
            .add_systems(Update, rendering::update_track_meshes);
    }
}

pub fn create_track(
    mut commands: Commands,
) {
    let x = 480.0;
    let z = 1770.0;
    let y = 216.0;

    let length = 19.46;
    commands
        .spawn(Segment { length })
        .insert(TransformBundle::from(Transform::from_xyz(x, y, z - length/2.0)))
        .insert(VisibilityBundle::default());

    commands
        .spawn(Segment { length: 10.0 })
        .insert(TransformBundle::from(Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, 175.0f32.to_radians())).with_translation(Vec3::new(x, y, z - length/2.0))))
        .insert(VisibilityBundle::default());
}
