use std::time::Duration;

use bevy::prelude::{default, ChildOf, IntoScheduleConfigs, Name};
use bevy::app::{App, Plugin, PostUpdate, Startup, Update};
use bevy::gizmos::AppGizmoBuilder;
use bevy::math::Vec2;
use bevy::prelude::{Commands, Entity, Transform, Vec3, World};
use bevy::prelude::Visibility;
use bevy::prelude::Component;
use bevy::reflect::Reflect;
use rand::Rng;

use crate::terrain::Terrain;

pub mod debug;
pub mod movement;
pub mod rendering;
pub mod behaviour;

#[derive(Component, Default, Reflect)]
#[require(Transform, Visibility)]
#[require(Behaviour)]
pub struct Worker {
    behaviour_since: Duration,
    velocity: Vec3,
    target_velocity: Vec2,
    acceleration: f32,
}

#[derive(Component, Default, Reflect)]
pub enum Behaviour {
    #[default]
    Idle,
    WalkingTo(Vec3),
}

#[derive(Default)]
pub struct WorkerPlugin;

impl Plugin for WorkerPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Worker>()
            .register_type::<Behaviour>()
            .init_resource::<rendering::WorkerRenderParams>()
            .add_systems(Startup, rendering::setup_render_params)
            .add_systems(PostUpdate, rendering::render_workers)
            .add_systems(Update, (behaviour::update_workers, movement::move_workers).chain());

        app
            .init_gizmo_group::<debug::WorkerDebug>()
            .add_systems(Startup, debug::setup_debug)
            .add_systems(PostUpdate, debug::debug_workers);
    }
}

pub fn create_workers(
    terrain: &Terrain,
    mut commands: Commands,
    num_workers: usize,
) {
    /* Delete existing workers */
    //TODO shouldn't really be using the Name to identify the parent entity
    commands.queue(|w: &mut World| {
        let mut q = w.query::<(Entity, &Name)>();
        if let Some(id) = q.iter(w).find_map(|(id, n) | (n.as_str() == "Workers").then_some(id)) {
            w.despawn(id);
        }
    });

    let parent_id = commands
        .spawn((
            Name::new("Workers"),
            Visibility::default(),
            Transform::default()
        )).id();

    for _ in 0..num_workers {
        let mut rng = rand::thread_rng();
        let pos = Vec3::new(
            rng.gen_range(0.0..(terrain.size[1] as f32)),
            0.0,
            rng.gen_range(0.0..(terrain.size[0] as f32))
        );
        commands.spawn((
            Worker { acceleration: 1.0, ..default() }, Transform::from_translation(pos),
            ChildOf(parent_id)
        ));
    }
}
