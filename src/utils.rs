use std::ops::Range;

use bevy::color::palettes::basic::{GRAY, YELLOW};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub fn show_fps(diagnostics: Res<DiagnosticsStore>, mut window: Single<&mut Window>) {
    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
        else { return; };

    let Some(value) = fps.smoothed()
        else { return; };

    window.title = format!("FPS: {}", value);
}

pub fn show_version(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const EXTRA: Option<&str> = option_env!("RRENG_VERSION_EXTRA");
    let extra = EXTRA.unwrap_or_default();
    let version_str = format!("RRENG version {VERSION}{extra}");

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        Text(version_str.to_owned()),
        TextFont {
            font: font.clone(),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::Srgba(YELLOW)),
    ));
}

pub fn show_help_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    const HELP_STR: &str = "Controls: WASD move, QE rotate, ZX zoom, PgUp/Dn pitch";

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            right: Val::Px(10.0),
            ..default()
        },
        Text(HELP_STR.to_owned()),
        TextFont {
            font: font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::Srgba(GRAY)),
    ));
}

pub fn close_on_esc(
    input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if input.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

#[derive(Component)]
pub struct ConstantApparentSize(pub Range<f32>);

pub fn fix_apparent_size(
    camera_transform: Single<&GlobalTransform, With<Camera>>,
    mut query: Query<(&mut Transform, &GlobalTransform, &ConstantApparentSize)>,
) {
    for (mut transform, global_transform, size) in query.iter_mut() {
        let dist = camera_transform.translation().distance(global_transform.translation());

        let Range { start, end } = size.0;

        let scale = if dist < start {
            dist / start
        } else if dist > end {
            dist / end
        } else {
            1.0
        };

        let new_scale = Vec3::splat(scale);
        if (transform.scale - new_scale).length_squared() > 0.001 {
            transform.scale = new_scale;
        }
    }
}
