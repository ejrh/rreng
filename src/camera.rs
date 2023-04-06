use bevy::prelude::*;

use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

#[derive(Default)]
pub(crate) struct CameraPlugin {
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(create_camera);
   }
}

fn create_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-100.0, 50.0, -100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(FlyCamera {
        pitch: 19.0,
        yaw: 584.0,
        sensitivity: 10.0,
        ..default()
    });
}
