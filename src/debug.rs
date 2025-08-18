use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContext, EguiContextPass, EguiGlobalSettings, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::level::LevelLabel;
use crate::level::selection::SelectedPoint;
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

#[derive(Default, Reflect, Resource)]
#[reflect(Resource)]
struct DebugOptions {
    world_inspector: bool,
    debug_terrain: bool,
    debug_workers: bool,
    show_points: bool,
    show_lights: bool,
    log_click: bool,
}

#[macro_export]
macro_rules! debug_option {
    ($name: ident) => (in_state(DebugState::On).and(|options: Res<DebugOptions>| options.$name));
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<DebugOptions>()
            .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
            .init_state::<DebugState>()
            .init_resource::<DebugOptions>()
            .add_systems(Startup, setup_debug)
            .add_systems(Update, toggle_debug)
            .add_systems(EguiContextPass, debug_options_ui.run_if(in_state(DebugState::On)));

        app
            .add_plugins(WorldInspectorPlugin::new().run_if(debug_option!(world_inspector)))
            .add_systems(Update, debug_terrain.run_if(debug_option!(debug_terrain)))
            .add_systems(Update, crate::worker::debug::debug_workers.run_if(debug_option!(debug_workers)))
            .add_systems(Update, show_points.run_if(debug_option!(show_points)))
            .add_systems(Update, show_lights.run_if(debug_option!(show_lights)))
            .add_systems(Update, log_click.run_if(debug_option!(log_click)));
    }
}

fn setup_debug(
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
) {
    egui_global_settings.enable_absorb_bevy_input_system = true;
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

fn debug_options_ui(
    mut egui_context: Single<&mut EguiContext, With<PrimaryWindow>>,
    mut options: ResMut<DebugOptions>,
) {
    const DEFAULT_POS: (f32, f32) = (800., 100.);
    const DEFAULT_SIZE: (f32, f32) = (240., 160.);

    egui::Window::new("Debug Options")
        .default_pos(DEFAULT_POS)
        .default_size(DEFAULT_SIZE)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.checkbox(&mut options.world_inspector, "World Inspector");
                ui.heading("Gizmos");
                ui.checkbox(&mut options.debug_terrain, "Debug Terrain");
                ui.checkbox(&mut options.debug_workers, "Debug Workers");
                ui.checkbox(&mut options.show_points, "Show Points");
                ui.checkbox(&mut options.show_lights, "Show Lights");
                ui.heading("Logging");
                ui.checkbox(&mut options.log_click, "Clicks");
                ui.allocate_space(ui.available_size());
            });
        });
}

fn debug_terrain(
    terrain: Single<&Terrain, With<LevelLabel>>,
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
