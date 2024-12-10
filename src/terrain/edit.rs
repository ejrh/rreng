use std::collections::VecDeque;
use std::ops::DerefMut;

use bevy::input::ButtonInput;
use bevy::prelude::{Color, Gizmos, Local, MouseButton, Res, ResMut};
use ndarray::{Array2, Ix};

use crate::terrain::selection::SelectedPoint;
use crate::terrain::{Terrain, TerrainLayer};
use crate::terrain::utils::Range2;

pub fn click_point(
    buttons: Res<ButtonInput<MouseButton>>,
    selected_point: Res<SelectedPoint>,
    mut terrain: ResMut<Terrain>,
) {
    let elevation = &terrain.layers[&TerrainLayer::Elevation];
    let mut _guard = elevation.lock().unwrap();
    let elevation = _guard.deref_mut();

    let left = buttons.pressed(MouseButton::Left);
    let right = buttons.pressed(MouseButton::Right);

    if !left && !right { return; }

    let row = selected_point.point.z as Ix;
    let col = selected_point.point.x as Ix;

    if row >= elevation.dim().0 || col >= elevation.dim().1 {
        return;
    }

    if left && !right { elevation[(row, col)] += 1.0; }
    if right && !left { elevation[(row, col)] -= 1.0; }

    let range = propagate(row, col, elevation);

    drop(_guard);

    terrain.dirty_range(range);
}

pub fn drag_point(
    buttons: Res<ButtonInput<MouseButton>>,
    selected_point: Res<SelectedPoint>,
    mut terrain: ResMut<Terrain>,
    mut start_point: Local<SelectedPoint>,
    mut gizmos: Gizmos,
) {
    let elevation = &terrain.layers[&TerrainLayer::Elevation];
    let mut _guard = elevation.lock().unwrap();
    let elevation = _guard.deref_mut();

    if buttons.just_pressed(MouseButton::Left) {
        start_point.point = selected_point.point;
    }

    let mut ranges_to_dirty = Vec::new();

    if buttons.pressed(MouseButton::Left) {
        gizmos.arrow(start_point.point, selected_point.point, Color::srgb(1.0, 0.1, 0.1));
    } else if buttons.just_released(MouseButton::Left) {
        let row = start_point.point.z as Ix;
        let col = start_point.point.x as Ix;

        if row >= elevation.dim().0 || col >= elevation.dim().1 {
            return;
        }

        let start_h = elevation[(row, col)];

        let row = selected_point.point.z as Ix;
        let col = selected_point.point.x as Ix;

        if row >= elevation.dim().0 || col >= elevation.dim().1 {
            return;
        }

        let dist = selected_point.point.distance(start_point.point);
        for i in 1..= dist as usize {
            let point = start_point.point.lerp(selected_point.point, i as f32 / dist);

            let row = point.z as Ix;
            let col = point.x as Ix;

            elevation[(row, col)] = start_h;

            let range = propagate(row, col, elevation);
            ranges_to_dirty.push(range);
        }
    }

    drop(_guard);

    for range in ranges_to_dirty {
        terrain.dirty_range(range);
    }
}

fn propagate(crow: Ix, ccol: Ix, data: &mut Array2<f32>) -> Range2 {
    let mut queue = VecDeque::new();
    queue.push_back((crow, ccol));

    let cheight = data[(crow, ccol)];

    let mut range = Range2::default();

    while !queue.is_empty() {
        let Some((row, col)) = queue.pop_front() else { break };
        range.expand_to(row, col);

        for (nrow, ncol) in neighbours(row, col, data.dim()) {
            let dist = ((nrow.abs_diff(crow) * nrow.abs_diff(crow) + ncol.abs_diff(ccol) * ncol.abs_diff(ccol)) as f32).sqrt();
            let min_h = data[(row, col)].min(cheight - dist);
            let max_h = data[(row, col)].max(cheight + dist);

            if data[(nrow, ncol)] < min_h {
                data[(nrow, ncol)] = min_h;
                queue.push_back((nrow, ncol));
            } else if data[(nrow, ncol)] > max_h {
                data[(nrow, ncol)] = max_h;
                queue.push_back((nrow, ncol));
            }
        }
    }

    range
}

fn neighbours(row: Ix, col: Ix, dims: (Ix, Ix)) -> Vec<(Ix, Ix)> {
    let mut results = Vec::new();

    if row > 0 {
        results.push((row - 1, col));
    }
    if row < dims.0 - 1 {
        results.push((row + 1, col));
    }
    if col > 0 {
        results.push((row, col - 1));
    }
    if col < dims.1 - 1 {
        results.push((row, col + 1));
    }

    if row > 0 && col > 0 {
        results.push((row - 1, col - 1));
    }
    if row > 0 && col < dims.1 - 1 {
        results.push((row - 1, col + 1));
    }
    if row < dims.0 - 1 && col > 0 {
        results.push((row + 1, col - 1));
    }
    if row < dims.0 - 1 && col < dims.1 - 1 {
        results.push((row + 1, col + 1));
    }

    results
}
