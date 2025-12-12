use std::collections::VecDeque;
use std::ops::DerefMut;

use bevy::input::ButtonInput;
use bevy::log::info_span;
use bevy::prelude::{Color, Gizmos, Local, MouseButton, Res, Single, With, Vec2, Vec3};
use ndarray::{Array2, Ix, Ixs};

use crate::level::LevelLabel;
use crate::level::selection::SelectedPoint;
use crate::terrain::{TerrainData, TerrainLayer};
use crate::terrain::utils::Range2;

pub fn click_point(
    buttons: Res<ButtonInput<MouseButton>>,
    selected_point: Res<SelectedPoint>,
    mut terrain_data: Single<&mut TerrainData, With<LevelLabel>>,
) {
    let Some(elevation) = terrain_data.layers.get(&TerrainLayer::Elevation)
    else { return; };

    let mut _guard = elevation.write().unwrap();
    let elevation = _guard.deref_mut();

    let left = buttons.pressed(MouseButton::Left);
    let right = buttons.pressed(MouseButton::Right);

    if !left && !right { return; }

    let row = selected_point.point.z as Ix;
    let col = selected_point.point.x as Ix;

    if row >= elevation.dim().0 || col >= elevation.dim().1 {
        return;
    }

    let _span = info_span!("terraform.height").entered();

    if left && !right { elevation[(row, col)] += 1.0; }
    if right && !left { elevation[(row, col)] -= 1.0; }

    let range = propagate(row, col, elevation);

    drop(_guard);

    terrain_data.dirty_range(range);
}

pub fn drag_point(
    buttons: Res<ButtonInput<MouseButton>>,
    selected_point: Res<SelectedPoint>,
    mut terrain_data: Single<&mut TerrainData, With<LevelLabel>>,
    mut start_point: Local<SelectedPoint>,
    mut gizmos: Gizmos,
) {
    let Some(elevation) = terrain_data.layers.get(&TerrainLayer::Elevation)
    else { return; };

    let mut _guard = elevation.write().unwrap();
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

        let _span = info_span!("terraform.level").entered();

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
        terrain_data.dirty_range(range);
    }
}

pub fn terraform_along_segment(
    mut terrain_data: &mut TerrainData,
    from: Vec3,
    to: Vec3,
    sample_step_m: f32,
    side_offset_m: f32,
) {
    let Some(elevation_arc) = terrain_data.layers.get(&TerrainLayer::Elevation) else { return; };
    let mut guard = elevation_arc.write().unwrap();
    let elevation = guard.deref_mut();

    let _span = info_span!("terraform.track").entered();

    let v = to - from;
    let len = v.length();
    if len <= 0.0 { return; }
    let dir = v / len;
    let step = sample_step_m.max(0.5);
    let steps = (len / step).ceil() as i32;

    let mut total_range = Range2::default();

    for i in 0..=steps {
        let s = (i as f32 * step).min(len);
        let p = from + dir * s;

        // Set target height along straight interpolation between endpoints' y
        let t = if len > 0.0 { s / len } else { 0.0 };
        let target_h = from.y + (to.y - from.y) * t;

        // Helper to write a point (if in-bounds), propagate, and union its dirty range
        let mut apply_point = |pt: Vec3, total_range: &mut Range2| {
            let row = pt.z as Ix;
            let col = pt.x as Ix;
            if row >= elevation.dim().0 || col >= elevation.dim().1 { return; }
            elevation[(row, col)] = target_h;
            let range = propagate(row, col, elevation);
            total_range.expand_to(range.0.start, range.1.start);
            total_range.expand_to(range.0.end, range.1.end);
        };

        // Terraform across the full width by sweeping from left outer edge to right outer edge
        let perp = Vec3::new(-dir.z, 0.0, dir.x);
        let lateral_step = 1.0_f32; // metres
        let left = -side_offset_m;
        let right = side_offset_m;

        let span = right - left;
        let count = (span / lateral_step).floor() as i32; // number of full steps

        for k in 0..=count {
            let d = left + (k as f32) * lateral_step;
            let off = perp * d; // dir is normalized, so perp has unit length in XZ
            apply_point(p + off, &mut total_range);
        }
        // ensure we include the exact outer right edge if the step didn't land exactly
        if left + (count as f32) * lateral_step < right {
            let off = perp * right;
            apply_point(p + off, &mut total_range);
        }
    }

    drop(guard);

    terrain_data.dirty_range(total_range);
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

fn neighbours(row: Ix, col: Ix, dims: (Ix, Ix)) -> impl Iterator<Item=(Ix, Ix)> {
    const ADJUSTMENTS: [(Ixs, Ixs); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

    let row = row as Ixs;
    let col = col as Ixs;

    let row_range = 0..dims.0 as Ixs;
    let col_range = 0..dims.1 as Ixs;

    ADJUSTMENTS.iter()
        .map(move |(r, c)| (row + r, col + c))
        .filter(move |(r, c)| row_range.contains(r) && col_range.contains(c))
        .map(|(r, c)| (r as Ix, c as Ix))
}
