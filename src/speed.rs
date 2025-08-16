
use bevy::prelude::{resource_changed, Condition, Query, Single, SpawnRelated, With, Without};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::color::Color;
use bevy::color::palettes::basic::YELLOW;
use bevy::ecs::children;
use bevy::ecs::schedule::And;
use bevy::input::ButtonInput;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::{default, AlignItems, AlignSelf, BorderColor, BorderRadius, Commands, Component, FlexDirection, IntoScheduleConfigs, JustifySelf, KeyCode, Name, Node, ReflectResource, Res, ResMut, StateScoped, Text, TextColor, TextFont, Time, UiRect, Val, Virtual};
use bevy::prelude::{Reflect, Resource};
use bevy::ui::AlignContent;
use crate::screens::{Screen, Theme};

#[derive(Component)]
pub struct GameSpeedPlugin;

macro_rules! any_input_just_pressed {
    ($inp: expr $(, $inps: expr)*) => {
        input_just_pressed($inp) $(.or(input_just_pressed($inps)))*
    }
}

impl Plugin for GameSpeedPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameSpeed>()
            .add_systems(Startup, create_speed_ui)
            .add_systems(Update, update_speed_ui.run_if(resource_changed::<GameSpeed>))
            .add_systems(Update, toggle_pause.run_if(input_just_pressed(KeyCode::Space)))
            .add_systems(Update, change_speed.run_if(any_input_just_pressed!(
                KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4,
                KeyCode::Digit5, KeyCode::Digit6)));
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

#[derive(Component)]
struct SpeedUiText2;

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
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                SpeedUiText,
                Text::default(),
                TextFont::from_font(theme.font.clone()).with_font_size(32.0),
                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
            ),
            (
                SpeedUiText2,
                Text::default(),
                TextFont::from_font(theme.font.clone()).with_font_size(24.0),
                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
            ),
        ]
    ));
}

fn update_speed_ui(
    game_speed: Res<GameSpeed>,
    mut text: Single<&mut Text, With<SpeedUiText>>,
    mut text2: Single<&mut Text, (With<SpeedUiText2>, Without<SpeedUiText>)>,
) {
    let symbol = match game_speed.paused {
        true => "■",
        false => "►",
    };
    text.0 = symbol.into();

    let symbol2 = match game_speed.speed {
        0.25 => "¼",
        0.5 => "½",
        1.0 => "",
        2.0 => "2",
        4.0 => "4",
        8.0 => "8",
        _ => "?",
    };
    text2.0 = symbol2.into();
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

fn change_speed(
    mut game_speed: ResMut<GameSpeed>,
    mut time: ResMut<Time<Virtual>>,
    inputs: Res<ButtonInput<KeyCode>>,
) {
    for (key, spd) in [
        (KeyCode::Digit1, 0.25),
        (KeyCode::Digit2, 0.5),
        (KeyCode::Digit3, 1.0),
        (KeyCode::Digit4, 2.0),
        (KeyCode::Digit5, 4.0),
        (KeyCode::Digit6, 8.0),
    ] {
        if inputs.just_pressed(key) {
            game_speed.speed = spd;
            time.set_relative_speed(spd);
            break;
        }
    }
}
