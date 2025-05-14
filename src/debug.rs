use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::terrain::selection::SelectedPoint;
use crate::terrain::Terrain;
use crate::track::point::Point;

pub struct DebugPlugin;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum DebugState {
    #[default]
    Off,
    On,
}

impl DebugState {
    fn inverse(&self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<DebugState>()
            .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
            .add_plugins(WorldInspectorPlugin::new().run_if(in_state(DebugState::On)))
            .add_systems(Update, (debug_terrain, log_click, show_points, show_lights).run_if(in_state(DebugState::On)))
            .add_systems(Update, (toggle_debug, point_visibility));
    }
}

fn toggle_debug(
    keys: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<DebugState>>,
    mut next_state: ResMut<NextState<DebugState>>,
) {
    if keys.just_pressed(KeyCode::F5) {
        next_state.set(current_state.inverse());
    }
}

fn debug_terrain(
    terrain: Res<Terrain>,
    mut gizmos: Gizmos,
) {
    let pos = Vec3::new(0.5 * terrain.size[1] as f32,0.0,  0.5 * terrain.size[0] as f32);
    let size = Vec2::new(terrain.size[1] as f32, terrain.size[0] as f32);
    gizmos.rect(Isometry3d::new(pos, Quat::from_axis_angle(Vec3::X, PI/2.0)), size, Color::srgb(1.0, 1.0, 0.5));

    let h = terrain.size[0] as f32;
    let w = terrain.size[1] as f32;

    for i in 1..terrain.num_blocks[0] {
        let p1 = Vec3::new(0.0, 0.0, (i * terrain.block_size) as f32);
        let p2 = Vec3::new(w, 0.0, (i * terrain.block_size) as f32);
        gizmos.line(p1, p2, Color::srgb(0.5, 0.5, 0.25));
    }

    for i in 1..terrain.num_blocks[1] {
        let p1 = Vec3::new((i * terrain.block_size) as f32, 0.0, 0.0);
        let p2 = Vec3::new((i * terrain.block_size) as f32, 0.0, h);
        gizmos.line(p1, p2, Color::srgb(0.5, 0.5, 0.25));
    }
}

pub fn log_click(
    buttons: Res<ButtonInput<MouseButton>>,
    selected_point: Res<SelectedPoint>,
) {
    let left = buttons.just_pressed(MouseButton::Left);
    let right = buttons.just_pressed(MouseButton::Right);

    if left || right {
        info!("Clicked on {:?} ({}, {})", selected_point.point, left, right);
    }
}

fn point_visibility(
    current_state: Res<State<DebugState>>,
    mut point_visibility: Query<&mut Visibility, With<Point>>
) {
    let vis = match current_state.get() {
        DebugState::On => Visibility::Inherited,
        DebugState::Off => Visibility::Hidden,
    };

    for mut pv in point_visibility.iter_mut() {
        if vis != *pv {
            *pv = vis;
        }
    }
}

fn show_points(
    points: Query<&Transform, With<Point>>,
    mut gizmos: Gizmos,
) {
    for transform in points.iter() {
        let from_point = transform.translation;
        let to_point = from_point + transform.forward().as_vec3();
        gizmos.arrow(from_point, to_point, Color::srgb(0.0, 1.0, 0.0));
        let pos = transform.to_isometry();
        gizmos.circle(pos, 2.0, Color::srgb(0.0, 1.0, 0.0));
        gizmos.circle(pos, 1.5, Color::srgb(0.0, 1.0, 0.0));
        gizmos.circle(pos, 1.0, Color::srgb(0.0, 1.0, 0.0));
    }
}

fn show_lights(
    spot_lights: Query<(&SpotLight, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    for (_light, transform) in spot_lights.iter() {
        let from_point = transform.translation();
        let to_point = transform.transform_point(Vec3::new(0.0, 0.0, -2.0));
        gizmos.arrow(from_point, to_point, Color::srgb(1.0, 1.0, 0.0));
    }
}
