use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::{Query, Res, Window};

pub fn show_fps(diagnostics: Res<DiagnosticsStore>, mut windows: Query<&mut Window>) {
    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
        else { return; };

    let Some(value) = fps.smoothed()
        else { return; };

    let mut window = windows.single_mut();
    window.title = format!("FPS: {}", value);
}
