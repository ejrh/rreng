
use bevy::prelude::{resource_changed, Query, Single, SpawnRelated, With};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::color::Color;
use bevy::color::palettes::basic::YELLOW;
use bevy::ecs::children;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::{default, AlignItems, AlignSelf, BorderColor, BorderRadius, Commands, Component, FlexDirection, IntoScheduleConfigs, JustifySelf, KeyCode, Name, Node, ReflectResource, Res, ResMut, StateScoped, Text, TextColor, TextFont, Time, UiRect, Val, Virtual};
use bevy::prelude::{Reflect, Resource};
use bevy::ui::AlignContent;
use crate::screens::{Screen, Theme};

#[derive(Component)]
pub struct GameSpeedPlugin;

impl Plugin for GameSpeedPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameSpeed>()
            .add_systems(Startup, create_speed_ui)
            .add_systems(Update, update_speed_ui.run_if(resource_changed::<GameSpeed>))
            .add_systems(Update, toggle_pause.run_if(input_just_pressed(KeyCode::Space)));
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

#[derive(Component)]
struct SpeedUiText;

fn create_speed_ui(
    theme: Res<Theme>,
    mut commands: Commands,
) {
    commands.spawn((
        Name::new("Speed"),
        Node {
            left: Val::Px(20.0),
            bottom: Val::Px(10.0),
            align_self: AlignSelf::End,
            ..default()
        },
        children![
            (
                SpeedUiText,
                Text::default(),
                TextFont::from_font(theme.font.clone()).with_font_size(32.0),
                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
            ),
        ]
    ));
}

fn update_speed_ui(
    game_speed: Res<GameSpeed>,
    mut text: Single<&mut Text, With<SpeedUiText>>,
) {
    text.0 = match game_speed.paused {
        true => "■",
        false => "►",
    }.into();
}

fn toggle_pause(
    mut game_speed: ResMut<GameSpeed>,
    mut time: ResMut<Time<Virtual>>,
) {
    game_speed.paused = !game_speed.paused;
    match game_speed.paused {
        true => time.pause(),
        false => time.unpause(),
    }
}
