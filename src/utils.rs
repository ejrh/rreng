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
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    let version_str = format!("RRENG version {VERSION}");

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
