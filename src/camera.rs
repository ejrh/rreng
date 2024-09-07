use std::f32::consts::TAU;
use std::ops::Range;

use bevy::{
    color::palettes::basic::GRAY,
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
};

use crate::terrain::terrain::Terrain;

#[derive(Default)]
pub struct CameraPlugin {
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, create_camera)
            .add_systems(Startup, create_camera_position_text)
            .add_systems(Update, camera_movement)
            .add_systems(Update, update_camera_position);
    }
}

#[derive(Component)]
struct CameraState {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    focus_range: Range<Vec3>,
    pitch_range: Range<f32>,
    distance_range: Range<f32>,
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            focus: Default::default(),
            yaw: Default::default(),
            pitch: -TAU/8.0,
            distance: 1000.0,
            focus_range: Vec3::new(0.0, 0.0, 0.0)..Vec3::new(8.0 * 480.0, 1000.0, 3.0 * 720.0),
            pitch_range: -TAU/4.0..TAU/4.0,
            distance_range: 1.0..2000.0,
        }
    }
}

#[derive(Component)]
struct CameraPositionLabel;

fn create_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        tonemapping: Tonemapping::None,
        ..default()
    }).insert(CameraState::default());
}

fn camera_movement(time: Res<Time>,
                   keyboard_input: Res<ButtonInput<KeyCode>>,
                   mut query: Query<(&mut CameraState, &mut Transform)>
) {
    let (mut state, transform) = query.single_mut();

    let mut movement_delta = Vec3::ZERO;
    let mut yaw_delta = 0.0;
    let mut pitch_delta = 0.0;
    let mut distance_delta: f32 = 0.0;
    if keyboard_input.pressed(KeyCode::KeyW) {
        movement_delta.z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        movement_delta.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement_delta.z += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement_delta.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyQ) {
        yaw_delta += TAU/8.0;
    }
    if keyboard_input.pressed(KeyCode::KeyE) {
        yaw_delta -= TAU/8.0;
    }
    if keyboard_input.pressed(KeyCode::PageUp) {
        pitch_delta -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::PageDown) {
        pitch_delta += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyZ) {
        distance_delta -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyX) {
        distance_delta += 1.0;
    }

    if yaw_delta == 0.0 && pitch_delta == 0.0 && movement_delta == Vec3::ZERO && distance_delta == 0.0 { return; }

    if movement_delta != Vec3::ZERO {
        let movement_delta = transform.rotation.mul_vec3(movement_delta);
        let movement_delta = Vec3 { y: 0.0, ..movement_delta }.normalize() * state.distance;
        state.focus += movement_delta * time.delta_seconds();

        state.focus = state.focus.clamp(state.focus_range.start, state.focus_range.end);
    }

    state.yaw += yaw_delta * time.delta_seconds();
    if state.yaw < 0.0 { state.yaw += TAU; }
    else if state.yaw >= TAU { state.yaw -= TAU; }

    state.pitch += pitch_delta * time.delta_seconds();
    state.pitch = state.pitch.clamp(state.pitch_range.start, state.pitch_range.end);

    if distance_delta != 0.0 {
        state.distance *= 2.0f32.powf(distance_delta * time.delta_seconds());

        state.distance = state.distance.clamp(state.distance_range.start, state.distance_range.end);
    }
}

fn update_camera_position(
    mut query: Query<(&CameraState, &mut Transform), Changed<CameraState>>,
    mut text_query: Query<&mut Text, With<CameraPositionLabel>>,
    terrain: Res<Terrain>,
) {
    let Ok((state, mut transform)) = query.get_single_mut()
    else { return; };

    transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw)
        * Quat::from_axis_angle(Vec3::X, state.pitch);
    let up_to_camera = transform.rotation.mul_vec3(Vec3::Z);
    let mut focus = state.focus;
    focus.y = terrain.elevation_at(focus.xz());
    transform.translation = focus + state.distance * up_to_camera;

    /* Update position text if it exists */
    if let Ok(mut text) = text_query.get_single_mut() {
        text.sections[0].value = format!("Camera: focus {:3.0}, {:3.0}; yaw {:3.0}; pitch {:3.0}",
                                         state.focus.z, state.focus.x, state.yaw.to_degrees(), state.pitch.to_degrees());
    }
}

pub fn create_camera_position_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 16.0,
        color: Color::Srgba(GRAY),
    };

    commands.spawn(TextBundle::from_section("", text_style)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        })
    ).insert(CameraPositionLabel);
}
