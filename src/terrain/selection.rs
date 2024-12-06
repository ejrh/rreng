use bevy::color::palettes::basic::GRAY;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::picking::backend::prelude::RayMap;
use bevy::picking::backend::ray::RayId;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;

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
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        SelectionMarker,
        ConstantApparentSize(100.0..250.0),
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

pub fn update_selected_point(
    ray_map: Res<RayMap>,
    mut prev_cursor_ray: Local<Option<Ray3d>>,
    mut selected_point: ResMut<SelectedPoint>,
    mut raycast: MeshRayCast,
    mut marker_query: Query<&mut Transform, With<SelectionMarker>>,
    terrain_meshes: Query<Entity, With<TerrainMesh>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    let camera = camera_query.single();
    let cursor_ray = ray_map.map().get(&RayId::new(camera, PointerId::Mouse))
        .copied();

    if cursor_ray == *prev_cursor_ray { return }
    *prev_cursor_ray = cursor_ray;

    let Some(cursor_ray) = cursor_ray else { return };

    let filter = |e| terrain_meshes.contains(e);
    let settings = RayCastSettings::default()
        .with_filter(&filter);

    let results = raycast.cast_ray(cursor_ray, &settings);

    if results.is_empty() {
        return;
    }
    let (_entity, intersection) = &results[0];

    let point = intersection.point;

    selected_point.point = point;

    if let Ok(mut transform) = marker_query.get_single_mut() {
        transform.translation = point;
    }
}

#[derive(Component)]
pub struct CursorPositionLabel;

pub fn update_cursor_position(
    mut text_query: Query<&mut Text, With<CursorPositionLabel>>,
    marker_query: Query<&mut Transform, With<SelectionMarker>>,
) {
    let Ok(transform) = marker_query.get_single()
    else { return; };

    /* Update position text if it exists */
    if let Ok(mut text) = text_query.get_single_mut() {
        text.0 = format!(
            "Cursor: {:3.2}, {:3.2}; elevation {:3.2}",
            transform.translation.z, transform.translation.x, transform.translation.y
        );
    }
}

pub fn create_cursor_position_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            right: Val::Px(10.0),
            ..default()
        },
        Text("".to_owned()),
        TextFont {
            font: font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::Srgba(GRAY)),
        CursorPositionLabel
    ));
}
