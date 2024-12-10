use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;

use crate::track::segment::{Segment, SegmentLinkage};

/**
 * Tracks are rendered with:
 *    - rail height
 *    - sleeper height
 *    - bed height
 *
 * These fill up the space between ground level and RAIL_HEIGHT.
 * The bed height is expanded below ground level to fully occupy dips in the terrain.
 */
#[derive(Resource)]
pub struct TrackRenderParams {
    rail_material: Handle<StandardMaterial>,
    rail_height: f32,
    rail_profile: Vec<Vec2>,
    sleeper_material: Handle<StandardMaterial>,
    sleeper_dims: Vec3,
    sleeper_height: f32,
    sleeper_spacing: f32,
    sleeper_mesh: Handle<Mesh>,
    bed_material: Handle<StandardMaterial>,
    bed_profile: Vec<Vec2>,
}

pub fn init_render_params(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mut rail_material = StandardMaterial::from(Color::srgb(0.8, 0.8, 0.8));
    rail_material.metallic = 0.8;

    let rail_points_half = [(-6.5, 0.0), (-6.5, 1.0), (-1.0, 2.0), (-0.75, 11.0), (-3.0, 12.0), (-3.0, 14.0), (-2.0, 15.0)];
    let rail_points_otherhalf = rail_points_half.iter().rev().map(|(x, y)| (-x, *y));
    let rail_points = rail_points_half.iter().copied().chain(rail_points_otherhalf);
    let rail_profile = rail_points.map(|(x, y)| Vec2::new(x * 0.01, y * 0.01));

    let rail_height = 0.2 + 0.15;

    let mut sleeper_material = StandardMaterial::from(Color::srgb(0.5, 0.25, 0.1));
    sleeper_material.perceptual_roughness = 0.7;

    let sleeper_dims = Vec3::new(2.0, 0.15, 0.2);
    let sleeper_height = 0.2;
    let sleeper_spacing = 0.7;

    let sleeper_mesh = create_sleeper_mesh(sleeper_dims);
    let sleeper_mesh = meshes.add(sleeper_mesh);

    let mut bed_material = StandardMaterial::from(Color::srgb(0.6, 0.6, 0.5));
    bed_material.perceptual_roughness = 0.5;

    let bed_points_half = [(-2.5, -0.3), (-1.5, 0.2)];
    let bed_points_otherhalf = bed_points_half.iter().rev().map(|(x, y)| (-x, *y));
    let bed_points = bed_points_half.iter().copied().chain(bed_points_otherhalf);
    let bed_profile = bed_points.map(|(x, y)| Vec2::new(x, y));

    let params = TrackRenderParams {
        rail_material: materials.add(rail_material),
        rail_profile: rail_profile.collect(),
        rail_height,
        sleeper_material: materials.add(sleeper_material),
        sleeper_dims,
        sleeper_height,
        sleeper_spacing,
        sleeper_mesh,
        bed_material: materials.add(bed_material),
        bed_profile: bed_profile.collect(),
    };
    commands.insert_resource(params);
}

pub fn update_track_meshes(
    segments: Query<(Entity, &Segment, &SegmentLinkage), Changed<Segment>>,
    params: Res<TrackRenderParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (segment_id, segment, linkage) in segments.iter() {
        commands.entity(segment_id).despawn_descendants();

        let open_start = linkage.prev_segment.is_some();
        let open_end = linkage.next_segment.is_some();

        let rail_mesh = create_rail_mesh(&params, segment.length, open_start, open_end);
        commands.spawn((
            Mesh3d(meshes.add(rail_mesh)),
            MeshMaterial3d(params.rail_material.clone()),
        )).set_parent(segment_id);

        // let num_sleepers = f32::round(segment.length / params.sleeper_spacing) as usize;
        // let sleeper_offset = segment.length / (num_sleepers as f32);
        // for i in 0..num_sleepers {
        //     let sleeper_transform = Transform::from_xyz(0.0, params.sleeper_height + params.sleeper_dims.y/2.0, sleeper_offset * (i as f32 + 0.5));
        //     commands.spawn((
        //         Mesh3d(params.sleeper_mesh.clone()),
        //         MeshMaterial3d(params.sleeper_material.clone()),
        //         sleeper_transform,
        //     )).set_parent(segment_id);
        // }
        //
        // let bed_mesh = create_bed_mesh(&params, segment.length, open_start, open_end);
        // commands.spawn((
        //     Mesh3d(meshes.add(bed_mesh)),
        //     MeshMaterial3d(params.bed_material.clone()),
        // )).set_parent(segment_id);
    }
}

fn create_rail_mesh(params: &TrackRenderParams, length: f32, open_start: bool, open_end: bool) -> Mesh {
    const GAUGE: f32 = 1.435;

    let lateral_step = Vec3::new(GAUGE/2.0, 0.0, 0.0);

    let rail_profile = BoxedPolyline2d::new(params.rail_profile.clone());
    let mut mesh = open_extrusion(&rail_profile.vertices, length).translated_by(lateral_step);

    mesh.merge(&open_extrusion(&rail_profile.vertices, length).translated_by(-lateral_step));

    if !open_start {
        mesh.merge(&polygon(&rail_profile.vertices)
            .rotated_by(Quat::from_axis_angle(Vec3::Y, TAU/2.0))
            .translated_by(lateral_step));
        mesh.merge(&polygon(&rail_profile.vertices)
            .rotated_by(Quat::from_axis_angle(Vec3::Y, TAU/2.0))
            .translated_by(-lateral_step));
    }
    if !open_end {
        mesh.merge(&polygon(&rail_profile.vertices)
            .translated_by(Vec3::new(0.0, 0.0, length))
            .translated_by(lateral_step));
        mesh.merge(&polygon(&rail_profile.vertices)
            .translated_by(Vec3::new(0.0, 0.0, length))
            .translated_by(-lateral_step));
    }

    mesh.translate_by(Vec3::new(0.0, params.rail_height, 0.0));

    mesh
}

fn open_extrusion(profile: &[Vec2], length: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    let mut tris = Vec::new();

    let z = length;

    for points in profile.windows(2) {
        let pt1 = points[0];
        let pt2 = points[1];

        tris.extend([
            Vec3::new(pt1.x, pt1.y, 0.0),
            Vec3::new(pt1.x, pt1.y, z),
            Vec3::new(pt2.x, pt2.y, 0.0),
        ]);
        tris.extend([
            Vec3::new(pt2.x, pt2.y, 0.0),
            Vec3::new(pt1.x, pt1.y, z),
            Vec3::new(pt2.x, pt2.y, z),
        ]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tris);
    mesh.compute_flat_normals();

    mesh
}

fn polygon(profile: &[Vec2]) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    let floats: Vec<_> = profile.iter().flat_map(|v| [v.x, v.y]).collect();
    let Ok(tri_idx) = earcutr::earcut(&floats, &[], 2) else {
        error!("Can't triangulate rail polygon");
        return mesh;
    };

    let tris: Vec<_> = tri_idx.iter()
        .map(|ix| profile[*ix])
        .map(|v: Vec2| v.extend(0.0))
        .collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tris);
    mesh.compute_flat_normals();

    mesh
}

fn create_bed_mesh(params: &TrackRenderParams, length: f32, open_start: bool, open_end: bool) -> Mesh {
    let bed_profile = BoxedPolyline2d::new(params.bed_profile.clone());
    let mut mesh = open_extrusion(&bed_profile.vertices, length);

    if !open_start {
        mesh.merge(&polygon(&bed_profile.vertices)
            .rotated_by(Quat::from_axis_angle(Vec3::Y, TAU/2.0)));
    }
    if !open_end {
        mesh.merge(&polygon(&bed_profile.vertices)
            .translated_by(Vec3::new(0.0, 0.0, length)));
    }

    mesh
}

fn create_sleeper_mesh(sleeper_dims: Vec3) -> Mesh {
    let mut mesh: Mesh = Cuboid::from_size(sleeper_dims).into();

    /* Remove bottom face */
    if let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        pos.truncate(30);
    }
    if let Some(VertexAttributeValues::Float32x3(norm)) = mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        norm.truncate(30);
    }
    mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0);
    if let Some(Indices::U32(inds)) = mesh.indices_mut() {
        inds.truncate(30);
    }

    mesh
}
