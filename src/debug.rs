use std::f32::consts::PI;

use bevy::app::{App, Plugin, Startup, Update};
use bevy::color::Color;
use bevy::ecs::{
    error::Result,
    query::{With},
    reflect::ReflectResource,
    resource::Resource,
    schedule::{Condition, IntoScheduleConfigs},
    system::{Query, Single},
    world::World,
    system::{Res, ResMut},
};
use bevy::gizmos::gizmos::Gizmos;
use bevy::input::{ButtonInput, keyboard::KeyCode, mouse::MouseButton};
use bevy::log::info;
use bevy::math::{Isometry3d, Quat, Vec2, Vec3};
use bevy::pbr::SpotLight;
use bevy::reflect::Reflect;
use bevy::state::{
    app::AppExtStates,
    condition::in_state,
    state::{
        States, NextState, State
    }
};
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::ui::Node;
use bevy_egui::{EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::level::LevelLabel;
use crate::level::selection::SelectedPoint;
use crate::terrain::Terrain;
use crate::terrain::rendering::{LayerLabel, TerrainMesh};
use crate::track::bridge::Bridge;
use crate::track::point::Point;
use crate::track::segment::Segment;
use crate::train::TrainCar;
use crate::worker::Worker;

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

#[derive(Reflect, Resource)]
#[reflect(Resource)]
struct DebugOptions {
    world_stats: bool,
    world_inspector: bool,
    debug_terrain: bool,
    debug_workers: bool,
    show_points: bool,
    show_lights: bool,
    log_click: bool,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            world_stats: true,
            world_inspector: true,
            debug_terrain: false,
            debug_workers: false,
            show_points: false,
            show_lights: false,
            log_click: false,
        }
    }
}

#[macro_export]
macro_rules! debug_option {
    ($name: ident) => (in_state(DebugState::On).and(|options: Res<DebugOptions>| options.$name));
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<DebugOptions>()
            .add_plugins(EguiPlugin::default())
            .init_state::<DebugState>()
            .init_resource::<DebugOptions>()
            .add_systems(Startup, setup_debug)
            .add_systems(Update, toggle_debug)
            .add_systems(EguiPrimaryContextPass, debug_options_ui.run_if(in_state(DebugState::On)));

        app
            .add_systems(EguiPrimaryContextPass, world_stats.run_if(debug_option!(world_stats)))
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
    mut egui_contexts: EguiContexts,
    mut options: ResMut<DebugOptions>,
) -> Result {
    const DEFAULT_POS: (f32, f32) = (720., 16.);

    egui::Window::new("Debug Options")
        .default_pos(DEFAULT_POS)
        .show(egui_contexts.ctx_mut()?, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.checkbox(&mut options.world_stats, "World Stats");
                ui.checkbox(&mut options.world_inspector, "World Inspector");
                ui.heading("Gizmos");
                ui.checkbox(&mut options.debug_terrain, "Debug Terrain");
                ui.checkbox(&mut options.debug_workers, "Debug Workers");
                ui.checkbox(&mut options.show_points, "Show Points");
                ui.checkbox(&mut options.show_lights, "Show Lights");
                ui.heading("Logging");
                ui.checkbox(&mut options.log_click, "Clicks");
            });
        });
    Ok(())
}

fn world_stats(
    world: &mut World,
) {
    const DEFAULT_POS: (f32, f32) = (380., 16.);

    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("World Stats")
        .default_pos(DEFAULT_POS)
        .show(egui_context.get_mut(), |ui| {
            fn row(ui: &mut egui::Ui, name: &str, value: usize) {
                ui.label(name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", value));
                });
                ui.end_row()
            }

            ui.horizontal_top(|ui| {
                egui::Grid::new("stats").show(ui, |ui| {
                    ui.heading("ECS");
                    ui.end_row();
                    row(ui, "Entities", world.entities().len() as usize);
                    row(ui, "Components", world.components().len());
                    row(ui, "Archetypes", world.archetypes().len());
                    row(ui, "UI Nodes", world.query::<&Node>().iter(&world).count());

                    ui.heading("Terrain");
                    ui.end_row();
                    row(ui, "Layers", world.query::<&LayerLabel>().iter(&world).count());
                    row(ui, "Blocks", world.query::<&TerrainMesh>().iter(&world).count());
                });

                egui::Grid::new("stats2").show(ui, |ui| {
                    ui.heading("Tracks");
                    ui.end_row();
                    row(ui, "Points", world.query::<&Point>().iter(&world).count());
                    row(ui, "Segments", world.query::<&Segment>().iter(&world).count());
                    row(ui, "Bridges", world.query::<&Bridge>().iter(&world).count());

                    ui.heading("Trains");
                    ui.end_row();
                    row(ui, "Cars", world.query::<&TrainCar>().iter(&world).count());

                    ui.heading("Workers");
                    ui.end_row();
                    row(ui, "Workers", world.query::<&Worker>().iter(&world).count());
                });
            })
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
