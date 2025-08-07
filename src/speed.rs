
use bevy::app::{App, Plugin, Update};
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::{Component, IntoScheduleConfigs, KeyCode, ReflectResource, ResMut, Time, Virtual};
use bevy::prelude::{Reflect, Resource};

#[derive(Component)]
pub struct GameSpeedPlugin;

impl Plugin for GameSpeedPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameSpeed>()
            .add_systems(Update, update_game_speed.run_if(input_just_pressed(KeyCode::Space)));
    }
}

#[derive(Clone, Debug, Reflect, Resource)]
#[reflect(Resource)]
struct GameSpeed {
    paused: bool,
    speed: f32,
}

impl Default for GameSpeed {
    fn default() -> Self {
        Self {
            paused: false,
            speed: 1.0,
        }
    }
}

fn update_game_speed(
    mut game_speed: ResMut<GameSpeed>,
    mut time: ResMut<Time<Virtual>>,
) {
    game_speed.paused = !game_speed.paused;
    match game_speed.paused {
        true => time.pause(),
        false => time.unpause(),
    }
}
