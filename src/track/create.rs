use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::prelude::{GizmoConfigGroup, GizmoConfigStore, Gizmos, Reflect, ResMut};
use std::f32::consts::PI;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct TrackPreviewGizmos;

pub fn setup_track_preview_gizmos(
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (cfg, _) = config_store.config_mut::<TrackPreviewGizmos>();
    cfg.depth_bias = -1.0; // draw on top of everything
}

use crate::level::selection::SelectedPoint;
use crate::level::LevelLabel;
use crate::terrain::{TerrainData, TerrainLayer};
use crate::terrain::utils::Range2;
use ndarray::Array2;
use crate::terrain::edit::terraform_along_segment;
use crate::track::create_track;
use crate::track::create::TrackCreationSessionState::{Idle, Placing};

#[derive(Default, Resource)]
pub struct TrackCreationSession {
    pub points: Vec<Vec3>,
    pub state: TrackCreationSessionState,
    pub terrain_backup: Option<Array2<f32>>,
}

#[derive(Resource, Reflect, Debug, Clone, Copy)]
pub struct TrackCreationConstraints {
    pub min_length_m: f32,
    pub max_turn_deg: f32,
    pub max_clearance_above_m: f32,
    pub max_clearance_below_m: f32,
    pub sample_step_m: f32,
    pub side_offset_m: f32,
}

impl Default for TrackCreationConstraints {
    fn default() -> Self {
        Self {
            min_length_m: 20.0,
            max_turn_deg: 10.0,
            max_clearance_above_m: 1.0,
            max_clearance_below_m: 1.0,
            sample_step_m: 1.0,
            side_offset_m: 2.0,
        }
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum TrackCreationSessionState {
    #[default]
    Idle,
    Placing,
}

pub fn handle_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    selected_point: Res<SelectedPoint>,
    mut session: ResMut<TrackCreationSession>,
    constraints: Res<TrackCreationConstraints>,
    mut terrain_data: Single<&mut TerrainData, With<LevelLabel>>,
    mut commands: Commands,
) {
    // Cancel session
    if keys.just_pressed(KeyCode::Escape) {
        // If we modified terrain during this session, restore it
        if let Some(ref backup) = &session.terrain_backup {
            if let Some(layer) = terrain_data.layers.get(&TerrainLayer::Elevation) {
                let mut guard = layer.write().unwrap();
                // Restore full array
                *guard = backup.clone();
                let dims = guard.dim();
                drop(guard);
                terrain_data.dirty_range(Range2(0..dims.0, 0..dims.1));
            }
        }
        session.points.clear();
        session.terrain_backup = None;
        session.state = Idle;
        return;
    }

    // Undo last point via keyboard
    if keys.just_pressed(KeyCode::Backspace) {
        let had_segment = session.points.len() >= 2;
        session.points.pop();
        if let Some(ref backup) = &session.terrain_backup {
            if had_segment {
                if let Some(layer) = terrain_data.layers.get(&TerrainLayer::Elevation) {
                    let mut guard = layer.write().unwrap();
                    *guard = backup.clone();
                    let dims = guard.dim();
                    drop(guard);
                    // Reapply remaining segments after pop
                    for w in session.points.windows(2) {
                        let [a, b] = [w[0], w[1]];
                        terraform_along_segment(terrain_data.as_mut(), a, b, constraints.sample_step_m, constraints.side_offset_m);
                    }
                    terrain_data.dirty_range(Range2(0..dims.0, 0..dims.1));
                }
            }
        }
        if session.points.is_empty() {
            session.state = Idle;
            session.terrain_backup = None;
        }
        return;
    }

    // Undo last point via right-click
    if buttons.just_pressed(MouseButton::Right) {
        let had_segment = session.points.len() >= 2;
        session.points.pop();
        if let Some(ref backup) = &session.terrain_backup {
            if had_segment {
                if let Some(layer) = terrain_data.layers.get(&TerrainLayer::Elevation) {
                    let mut guard = layer.write().unwrap();
                    *guard = backup.clone();
                    let dims = guard.dim();
                    drop(guard);
                    // Reapply remaining segments after pop
                    for w in session.points.windows(2) {
                        let [a, b] = [w[0], w[1]];
                        terraform_along_segment(terrain_data.as_mut(), a, b, constraints.sample_step_m, constraints.side_offset_m);
                    }
                    terrain_data.dirty_range(Range2(0..dims.0, 0..dims.1));
                }
            }
        }
        if session.points.is_empty() {
            session.state = Idle;
            session.terrain_backup = None;
        }
        return;
    }

    // Commit current session (need at least 2 points)
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        if session.points.len() >= 2 {
            let name = "Track".to_string();
            let pts = session.points.clone();
            create_track(&name, &pts, false, &mut commands);
        }
        session.points.clear();
        session.terrain_backup = None; // keep terrain edits, just drop backup
        session.state = Idle;
        return;
    }

    // Add point on click
    if buttons.just_pressed(MouseButton::Left) {
        let candidate = selected_point.point;
        if session.points.is_empty() {
            // Always allow the first point
            session.points.push(candidate);
            session.state = Placing;
        } else {
            if is_segment_valid(&session, candidate, &constraints) {
                // Take a snapshot of the elevation layer before first terrain modification in this session
                if session.terrain_backup.is_none() {
                    if let Some(layer) = terrain_data.layers.get(&TerrainLayer::Elevation) {
                        let guard = layer.read().unwrap();
                        session.terrain_backup = Some(guard.clone());
                        drop(guard);
                    }
                }
                let last = *session.points.last().unwrap();
                terraform_along_segment(terrain_data.as_mut(), last, candidate, constraints.sample_step_m, constraints.side_offset_m);
                session.points.push(candidate);
                session.state = Placing;
            } else {
                // Invalid segment: ignore click (could add feedback later)
            }
        }
    }
}

fn is_segment_valid(
    session: &TrackCreationSession,
    next: Vec3,
    constraints: &TrackCreationConstraints,
) -> bool {
    let Some(&last) = session.points.last() else { return true }; // no segment yet

    // 1) Length at least min_length_m
    let v = next - last;
    let len = v.length();
    if len < constraints.min_length_m { return false; }

    // 2) Angle <= max_turn_deg vs previous segment (if exists)
    if session.points.len() >= 2 {
        let prev = session.points[session.points.len()-2];
        let v1 = last - prev;
        if v1.length_squared() > 0.0001 {
            let d = v1.normalize().dot(v.normalize()).clamp(-1.0, 1.0);
            let angle = d.acos() * 180.0 / PI; // degrees
            if angle > constraints.max_turn_deg { return false; }
        }
    }

    true
}

pub fn preview_gizmos(
    mut gizmos: Gizmos<TrackPreviewGizmos>,
    selected_point: Res<SelectedPoint>,
    session: Res<TrackCreationSession>,
    constraints: Res<TrackCreationConstraints>,
    terrain_data: Option<Single<&TerrainData, With<LevelLabel>>>,
) {
    if session.points.is_empty() { return; }

    let color_existing = Color::srgb(0.9, 0.9, 0.2);

    // Draw existing polyline
    for w in session.points.windows(2) {
        let [a, b] = [w[0], w[1]];
        gizmos.line(a, b, color_existing);
    }

    // Draw preview from last point to current cursor with validity color
    let last = *session.points.last().unwrap();
    let valid = is_segment_valid(&session, selected_point.point, &constraints);
    let preview_color = if valid { Color::srgb(0.2, 0.9, 0.2) } else { Color::srgb(0.9, 0.2, 0.2) };
    gizmos.line(last, selected_point.point, preview_color.with_alpha(0.85));
}