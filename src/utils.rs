use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub fn show_fps(diagnostics: Res<DiagnosticsStore>, mut windows: Query<&mut Window>) {
    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
        else { return; };

    let Some(value) = fps.smoothed()
        else { return; };

    let mut window = windows.single_mut();
    window.title = format!("FPS: {}", value);
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
        color: Color::GOLD,
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
