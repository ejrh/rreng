use std::ops::Range;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub fn show_fps(diagnostics: Res<DiagnosticsStore>, mut window: Single<&mut Window>) {
    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
        else { return; };

    let Some(value) = fps.smoothed()
        else { return; };

    window.title = format!("FPS: {}", value);
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
