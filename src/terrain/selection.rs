use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy_mod_raycast::cursor::CursorRay;
use bevy_mod_raycast::immediate::{Raycast, RaycastSettings};

use crate::terrain::rendering::TerrainMesh;
use crate::utils::ConstantApparentSize;

#[derive(Default, Resource)]
pub struct SelectedPoint {
    pub point: Vec3,
}

#[derive(Component)]
pub struct SelectionMarker;

pub fn create_marker(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mesh: Mesh = Sphere::new(2.0).into();
    let mut material = StandardMaterial::from_color(Color::srgb(1.0, 0.5, 0.5));
    material.emissive = LinearRgba::rgb(1.0, 0.5, 0.5);
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(mesh),
            material: materials.add(material),
            ..default()
        },
        SelectionMarker,
        ConstantApparentSize(100.0..250.0),
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

pub fn update_selected_point(
    cursor_ray: Res<CursorRay>,
    mut prev_cursor_ray: Local<Option<Ray3d>>,
    mut selected_point: ResMut<SelectedPoint>,
    mut raycast: Raycast,
    mut marker_query: Query<&mut Transform, With<SelectionMarker>>,
    terrain_meshes: Query<Entity, With<TerrainMesh>>
) {
    let cursor_ray = **cursor_ray;

    if cursor_ray == *prev_cursor_ray { return }
    *prev_cursor_ray = cursor_ray;

    let Some(cursor_ray) = cursor_ray else { return };

    let filter = |e| terrain_meshes.contains(e);
    let settings = RaycastSettings::default()
        .with_filter(&filter);

    let results = raycast.cast_ray(cursor_ray, &settings);

    if results.is_empty() {
        return;
    }
    let (entity, intersection) = &results[0];

    let point = intersection.position();

    selected_point.point = point;

    if let Ok(mut transform) = marker_query.get_single_mut() {
        transform.translation = point;
    }
}
