use std::f32::consts::{PI, TAU};

use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Handle};
use bevy::color::Color;
use bevy::color::palettes::basic::{GRAY, YELLOW};
use bevy::color::palettes::css::{GREY, SILVER};
use bevy::ecs::children;
use bevy::input::common_conditions::input_just_pressed;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{in_state, AppExtStates, Commands, Condition, Font, IntoScheduleConfigs, KeyCode, Name, Node, OnEnter, Res, ResMut, Resource, Single, SpawnRelated, StateScoped, Text, TextColor, TextFont, TextSpan, Time, Val};
use bevy::state::state::States;
use bevy::ui::{AlignItems, AlignSelf, BorderColor, BorderRadius, Display, FlexDirection, JustifySelf, UiRect};
use bevy::utils::default;
use rand::{thread_rng, Rng, seq::SliceRandom};

use crate::camera::{CameraMode, CameraState};
use crate::track::bridge::Bridge;
use crate::track::create_track;
use crate::train::create_train;
use crate::{camera, level, tools};
use crate::events::GameEvent;
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

        app.add_systems(Update, load_level.run_if(in_state(Screen::Title).and(input_just_pressed(KeyCode::KeyJ))));
    }
}

fn load_level(mut commands: Commands) {
    commands.send_event(GameEvent::LoadLevel("data/jvl.ron".to_owned()));
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
            ),
            Node {
                height: Val::Percent(100.0),
                ..default()
            },
            (
                Text::default(),
                TextFont::from_font(theme.font.clone()).with_font_size(20.0),
                TextColor(Color::Srgba(GREY)),
                children![
                    (TextSpan("Press ".into()),
                     TextFont::from_font(theme.font.clone()).with_font_size(20.0),
                     TextColor(Color::Srgba(GREY))),
                    (TextSpan("J".into()),
                     TextFont::from_font(theme.font.clone()).with_font_size(20.0),
                     TextColor(Color::Srgba(YELLOW))),
                    (TextSpan(" to load the level".into()),
                     TextFont::from_font(theme.font.clone()).with_font_size(20.0),
                     TextColor(Color::Srgba(GREY)))
                ],
            ),
            Node {
                height: Val::Percent(5.0),
                ..default()
            },
        ]
    ));

    /* Create some loop tracks with trains on them */
    fn make_circle(radius: f32, height: f32, segments: usize) -> Vec<Vec3> {
        let mut points = vec![];
        let segment_angle = 360.0 / segments as f32;
        for i in 0..segments {
            let angle = (i as f32 * segment_angle).to_radians();
            let x = radius * angle.cos();
            let y = height;
            let z = radius * angle.sin();
            points.push(Vec3::new(x, y, z));
        }
        points
    }

    fn make_figure_8(radius: f32, height: f32, segments: usize, minus_segments: usize) -> Vec<Vec3> {
        let remaining_segments = segments - minus_segments;

        let circle = make_circle(radius, 0.0, segments);
        let circle: Vec<_> = circle.iter()
            .skip(minus_segments / 2).take(remaining_segments + 1)
            .collect();

        let angle = TAU / segments as f32 * (minus_segments - 1) as f32 / 2.0;
        let last_pt = circle[0];
        let offset = last_pt.x + last_pt.z * angle.tan();

        let curve = circle.iter().enumerate()
            .map(|(i, pt)| *pt + Vec3::new(-offset, (1.0 - (i as f32 / remaining_segments as f32 * PI).cos()) / 2.0 * height, 0.0));
        let curve2 = curve.clone().rev()
            .map(|pt| pt * Vec3::new(-1.0, 1.0, -1.0));
        let points = curve.chain(curve2).collect();
        points
    }

    let layouts = [
        vec![
            (make_circle(100.0,0.0, 72), 2, 1.0, false),
            (make_circle(90.0,5.0, 68), 2, -1.0, false),
        ],
        vec![
            (make_figure_8(60.0, 10.0, 52, 6), 1, -1.0, true),
        ],
    ];
    if let Some(parts) = layouts.choose(&mut thread_rng()) {
        for (points, trains, spd, bridge) in parts {
            let (track_id, _, segment_ids) = create_track("Title", &points, true, &mut commands);

            if *bridge {
                let bridge_id = segment_ids[segment_ids.len() / 2 - 1];
                commands.entity(bridge_id).insert(
                    Bridge { pillars: 4 },
                );
            }

            commands.entity(track_id).insert(StateScoped(Screen::Title));

            for i in 0..*trains {
                let segment_id = segment_ids[i * (segment_ids.len() / *trains)];
                let train_id = create_train("Title", segment_id, 0.0, *spd, &mut commands);
                commands.entity(train_id).insert(StateScoped(Screen::Title));
            }
        }
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
