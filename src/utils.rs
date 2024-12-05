use std::ops::Range;

use bevy::color::palettes::basic::{GRAY, YELLOW};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub fn show_fps(diagnostics: Res<DiagnosticsStore>, mut windows: Query<&mut Window>) {
    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
        else { return; };

    let Some(value) = fps.smoothed()
        else { return; };

    if let Ok(mut window) = windows.get_single_mut() {
        window.title = format!("FPS: {}", value);
    }
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

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 20.0,
        color: Color::Srgba(YELLOW),
    };

    commands.spawn(TextBundle::from_section(version_str, text_style)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        })
    );
}

pub fn show_help_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 16.0,
        color: Color::Srgba(GRAY),
    };

    const HELP_STR: &str = "Controls: WASD move, QE rotate, ZX zoom, PgUp/Dn pitch";

    commands.spawn(TextBundle::from_section(HELP_STR, text_style)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            right: Val::Px(10.0),
            ..default()
        })
    );
}

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

#[derive(Component)]
pub struct ConstantApparentSize(pub Range<f32>);

pub fn fix_apparent_size(
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut query: Query<(&mut Transform, &GlobalTransform, &ConstantApparentSize)>,
) {
    let Ok(camera_transform) = camera_query.get_single()
    else { return; };

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

        transform.scale = Vec3::splat(scale);
    }
}
