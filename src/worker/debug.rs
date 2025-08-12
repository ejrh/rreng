use bevy::color::Color;
use bevy::prelude::{GizmoConfigGroup, GizmoConfigStore, Gizmos, Query, Reflect, ResMut, Transform};

use crate::worker::Behaviour;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct WorkerDebug;

pub fn setup_debug(
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (gc, _) = config_store.config_mut::<WorkerDebug>();
    gc.depth_bias = -1.0;
}

pub fn debug_workers(
    workers: Query<(&Behaviour, &Transform)>,
    mut gizmos: Gizmos<WorkerDebug>,
) {
    for (b, wt) in workers.iter() {
        match b {
            Behaviour::WalkingTo(target) => {
                gizmos.arrow(wt.translation, *target, Color::srgb(0.0, 1.0, 0.5));
            },
            _ => {}
        }
    }
}
