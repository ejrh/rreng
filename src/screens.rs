use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetServer, Handle};
use bevy::color::Color;
use bevy::color::palettes::basic::{GRAY, YELLOW};
use bevy::color::palettes::css::{GREY, SILVER};
use bevy::ecs::children;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{AppExtStates, Commands, Font, Name, Node, OnEnter, Res, ResMut, Resource, Single, SpawnRelated, StateScoped, Text, TextColor, TextFont, Val};
use bevy::state::state::States;
use bevy::ui::{AlignItems, AlignSelf, BorderColor, BorderRadius, Display, FlexDirection, JustifySelf, UiRect};
use bevy::utils::default;

use crate::camera::{CameraMode, CameraState};
use crate::track::create_track;
use crate::train::create_train;
use crate::{camera, level, tools};
use crate::tools::Tools;

pub(crate) struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<Screen>()
            .init_resource::<Theme>()
            .add_systems(Startup, setup_theme);

        app.add_systems(OnEnter(Screen::Title), setup_title);
        app.add_systems(OnEnter(Screen::Loading), setup_loading);
        app.add_systems(OnEnter(Screen::Playing), setup_playing);
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, States)]
#[states(scoped_entities)]
pub enum Screen {
    #[default]
    None,
    Title,
    Loading,
    Playing
}

#[derive(Default, Resource)]
pub struct Theme {
    font: Handle<Font>,
}

pub fn setup_theme(
    mut theme: ResMut<Theme>,
    asset_server: Res<AssetServer>,
) {
    theme.font = asset_server.load("fonts/FiraMono-Medium.ttf");
}

pub fn setup_title(
    theme: Res<Theme>,
    mut commands: Commands,
    mut camera: Single<&mut CameraState>,
) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const EXTRA: Option<&str> = option_env!("RRENG_VERSION_EXTRA");
    let extra = EXTRA.unwrap_or_default();
    let version_str = format!("version {VERSION}{extra}");

    commands.spawn((
        Name::new("Screen:Title"),
        Node {
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(1.0)),
            width: Val::Percent(99.0),
            height: Val::Percent(99.0),
            ..default()
        },
        BorderRadius::all(Val::Percent(1.0)),
        BorderColor::from(Color::Srgba(YELLOW)),
        StateScoped(Screen::Title),
        children![
            (
                Text("RRENG".to_owned()),
                TextFont::from_font(theme.font.clone()).with_font_size(80.0),
                TextColor(Color::Srgba(YELLOW)),
            ),
            (
                Text(version_str.to_owned()),
                TextFont::from_font(theme.font.clone()).with_font_size(20.0),
                TextColor(Color::Srgba(SILVER)),
            )
        ]
    ));

    /* Create some loop tracks with trains on them */
    for (rad, spd) in [(100.0, 1.0), (90.0, -1.0)] {
        let mut points = vec![];
        for i in 0..72 {
            points.push(Vec3::new(rad * (i as f32 * 5.0).to_radians().sin(), 0.0, rad * (i as f32 * 5.0).to_radians().cos()));
        }
        let (track_id, _, segment_ids) = create_track("Title", &points, true, &mut commands);

        /* Put a train at the start of the first and opposite segment */
        let first_segment_id = segment_ids[0];
        let train1_id = create_train("Title", first_segment_id, 0.0, spd, &mut commands);
        let opposite_segment_id = segment_ids[36];
        let train2_id = create_train("Title", opposite_segment_id, 0.0, spd, &mut commands);

        commands.entity(track_id).insert(StateScoped(Screen::Title));
        commands.entity(train1_id).insert(StateScoped(Screen::Title));
        commands.entity(train2_id).insert(StateScoped(Screen::Title));
    }

    /* Fix camera to be fixed on tracks */
    camera.focus = Vec3::new(0.0, 0.0, 20.0);
    camera.yaw = 0.0;
    camera.pitch = -0.30;
    camera.distance = 200.0;
    camera.mode = CameraMode::Fixed;
    info!("set camera");
}

pub fn setup_loading(
    theme: Res<Theme>,
    mut commands: Commands,
) {
    commands.spawn((
        Name::new("Screen:Loading"),
        Node {
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        },
        Text("Loading...".to_owned()),
        TextFont::from_font(theme.font.clone()).with_font_size(40.0),
        TextColor(Color::Srgba(GREY)),
        StateScoped(Screen::Loading),
    ));
}

pub fn setup_playing(
    theme: Res<Theme>,
    mut commands: Commands,
    tools: Option<Res<Tools>>
) {
    const HELP_STR: &str = "Controls: WASD move, QE rotate, ZX zoom, PgUp/Dn pitch";

    commands.spawn((
        Name::new("Screen:Playing"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Grid,
            ..default()
        },
        StateScoped(Screen::Playing),
        children![
            (
                Node {
                    align_self: AlignSelf::End,
                    justify_self: JustifySelf::End,
                    ..default()
                },
                Text(HELP_STR.to_owned()),
                TextFont {
                    font: theme.font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::Srgba(GRAY)),
            )
        ]
    ));

    if tools.is_some() {
        commands.run_system_cached(tools::create_tools);
        commands.run_system_cached(tools::create_terraform_tools);
    }

    commands.run_system_cached(camera::create_camera_position_text);
    commands.run_system_cached(level::selection::create_cursor_position_text);
}
