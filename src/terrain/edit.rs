use std::collections::VecDeque;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::log::info;
use bevy::prelude::{MouseButton, Res, ResMut};
use ndarray::{Array2, Ix};

use crate::terrain::selection::SelectedPoint;
use crate::terrain::terrain::Terrain;
use crate::terrain::utils::Range2;

pub fn click_point(
    buttons: Res<ButtonInput<MouseButton>>,
    mut selected_point: Res<SelectedPoint>,
    mut terrain: ResMut<Terrain>,
) {
    if !buttons.pressed(MouseButton::Left) { return; }

    let row = selected_point.point.z as Ix;
    let col = selected_point.point.x as Ix;

    if row < 0 || col < 0 || row >= terrain.elevation.dim().0 || col >= terrain.elevation.dim().1 {
        return;
    }

    terrain.elevation[(row, col)] += 1.0;
    let range = propagate(row, col, &mut terrain.elevation);

    terrain.dirty_range(range);
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

            if data[(nrow, ncol)] < min_h {
                data[(nrow, ncol)] = min_h;
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
