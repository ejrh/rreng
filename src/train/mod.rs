use bevy::prelude::*;

use crate::train::rendering::{render_trains, setup_render_params, TrainRenderParams};

mod movement;
mod rendering;

#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
pub struct TrainCar {
    pub segment_id: Entity,
    pub segment_position: f32,
    pub speed: f32,
    pub acceleration: f32,
    pub max_speed: f32,
    pub length: f32,
}

#[derive(Default)]
pub struct TrainPlugin;

impl Plugin for TrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<TrainCar>()
            .init_resource::<TrainRenderParams>()
            .add_systems(Startup, setup_render_params)
            .add_systems(Update, render_trains)
            .add_systems(Update, (movement::move_train, movement::update_train_position));
    }
}


pub fn create_train(
    name: &str,
    segment_id: Entity,
    segment_position: f32,
    speed: f32,
    commands: &mut Commands
) -> Entity {
    let parent_id = commands.spawn((
        Name::new(format!("Train:{name}")),
        TrainCar {
            segment_id,
            segment_position,
            speed,
            acceleration: 1.0,
            max_speed: 100_000.0 / 3600.0,
            length: 12.0,
        },
    )).id();
    info!("created train");
    
    parent_id
}
