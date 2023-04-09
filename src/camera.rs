use std::f32::consts::TAU;
use std::ops::Range;

use bevy::prelude::*;

#[derive(Default)]
pub(crate) struct CameraPlugin {
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_startup_system(create_camera)
        .add_system(camera_movement)
        .add_system(update_camera_position);
   }
}

#[derive(Component)]
struct CameraState {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    focus_range: Range<Vec3>,
    distance_range: Range<f32>,
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            focus: Default::default(),
            yaw: Default::default(),
            pitch: -TAU/8.0,
            distance: 50.0,
            focus_range: Vec3::new(0.0, 0.0, 0.0)..Vec3::new(100.0, 100.0, 100.0),
            distance_range: 10.0..120.0,
        }
    }
}

fn create_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle::default()).insert(CameraState::default());
}

fn camera_movement(time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut CameraState, &mut Transform)>
) {
    let (mut state, transform) = query.single_mut();

    let mut movement_delta = Vec3::ZERO;
    let mut yaw_delta = 0.0;
    let mut distance_delta: f32 = 0.0;
    if keyboard_input.pressed(KeyCode::W) {
        movement_delta.z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::A) {
        movement_delta.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) {
        movement_delta.z += 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
        movement_delta.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Q) {
        yaw_delta += TAU/8.0;
    }
    if keyboard_input.pressed(KeyCode::E) {
        yaw_delta -= TAU/8.0;
    }
    if keyboard_input.pressed(KeyCode::Z) {
        distance_delta -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::X) {
        distance_delta += 1.0;
    }

    if yaw_delta == 0.0 && movement_delta == Vec3::ZERO && distance_delta == 0.0 { return; }

    if movement_delta != Vec3::ZERO {
        let movement_delta = transform.rotation.mul_vec3(movement_delta);
        let movement_delta = Vec3 { y: 0.0, ..movement_delta }.normalize() * state.distance;
        state.focus += movement_delta * time.delta_seconds();

        state.focus = state.focus.clamp(state.focus_range.start, state.focus_range.end);
    }

    state.yaw += yaw_delta * time.delta_seconds();
    if state.yaw < 0.0 { state.yaw += TAU; }
    else if state.yaw >= TAU { state.yaw -= TAU; }

    if distance_delta != 0.0 {
        state.distance *= 2.0f32.powf(distance_delta * time.delta_seconds());

        state.distance = state.distance.clamp(state.distance_range.start, state.distance_range.end);
    }
}

fn update_camera_position(mut query: Query<(&CameraState, &mut Transform), Changed<CameraState>>) {
    let Ok((state, mut transform)) = query.get_single_mut()
        else { return; };

    transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw)
        * Quat::from_axis_angle(Vec3::X, state.pitch);
    let up_to_camera = transform.rotation.mul_vec3(Vec3::Z);
    transform.translation = state.focus + state.distance * up_to_camera;
}
