use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;

use crate::track::point::Point;
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
    mut segments: Query<(Entity, &mut Segment, &SegmentLinkage, &Transform), Or<(Changed<Segment>, Changed<SegmentLinkage>, Changed<Transform>)>>,
    points: Query<&Transform, With<Point>>,
    params: Res<TrackRenderParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (segment_id, mut segment, linkage, segment_transform) in segments.iter_mut() {

        if let Some(rendered_id) = segment.rendered_id {
            commands.entity(rendered_id).despawn_related::<Children>();
        } else {
            segment.rendered_id = Some(commands.spawn((
                Name::new("Track"),
                Transform::default(),
                Visibility::default(),
                ChildOf(segment_id),
            )).id());
        }
        let parent_id = segment.rendered_id.unwrap();

        let open_start = linkage.prev_segment.is_some();
        let open_end = linkage.next_segment.is_some();

        let point_transform = points.get(segment.from_point).unwrap();
        let mut transform = Transform::from_rotation(segment_transform.rotation);
        transform.rotate(point_transform.rotation.inverse());
        let start_normal = transform.forward().as_vec3();

        let point_transform = points.get(segment.to_point).unwrap();
        let mut transform = Transform::from_rotation(segment_transform.rotation);
        transform.rotate(point_transform.rotation.inverse());
        let end_normal = transform.forward().as_vec3();

        let rail_mesh = create_rail_mesh(&params, segment.length, open_start, open_end, start_normal, end_normal);
        commands.spawn((
            Mesh3d(meshes.add(rail_mesh)),
            MeshMaterial3d(params.rail_material.clone()),
            ChildOf(parent_id)
        ));

        let num_sleepers = f32::round(segment.length / params.sleeper_spacing) as usize;
        let sleeper_offset = segment.length / (num_sleepers as f32);
        for i in 0..num_sleepers {
            let sleeper_transform = Transform::from_xyz(0.0, params.sleeper_height + params.sleeper_dims.y/2.0, sleeper_offset * (i as f32 + 0.5));
            commands.spawn((
                Mesh3d(params.sleeper_mesh.clone()),
                MeshMaterial3d(params.sleeper_material.clone()),
                sleeper_transform,
                ChildOf(parent_id)
            ));
        }

        let bed_mesh = create_bed_mesh(&params, segment.length, open_start, open_end, start_normal, end_normal);
        commands.spawn((
            Mesh3d(meshes.add(bed_mesh)),
            MeshMaterial3d(params.bed_material.clone()),
            ChildOf(parent_id)
        ));
    }
}

fn create_rail_mesh(params: &TrackRenderParams, length: f32, open_start: bool, open_end: bool, start_normal: Vec3, end_normal: Vec3) -> Mesh {
    const GAUGE: f32 = 1.435;

    let rail_profile = BoxedPolyline2d::new(params.rail_profile.clone());
    let verts: Vec<_> = rail_profile.vertices.iter().map(|pt| Vec2::new(pt.x + GAUGE/2.0, pt.y + params.rail_height)).collect::<Vec<_>>();
    let mut mesh = extrusion(&verts, length, open_start, open_end, start_normal, end_normal);

    let verts: Vec<_> = rail_profile.vertices.iter().map(|pt| Vec2::new(pt.x - GAUGE/2.0, pt.y + params.rail_height)).collect::<Vec<_>>();
    mesh.merge(&extrusion(&verts, length, open_start, open_end, start_normal, end_normal)).expect("mesh.merge");

    mesh
}

/**
 * Project a 2D point onto a plane represented by a normal vector.
 * The origin is assumed to be in the plane.
 *
 * Adapted from https://stackoverflow.com/a/53437900, but slightly
 * simplified as our rays are always in the direction of Z, and we
 * assume they are never parallel to the plane.
 */
fn project_point(point: Vec2, plane_normal: Vec3) -> Vec3 {
    let ray = Vec3::new(point.x, point.y, 0.0);
    let ray_dest = Vec3::new(0.0, 0.0, 1.0);
    let t = plane_normal.dot(ray) / plane_normal.dot(ray_dest);
    ray + t * ray_dest
}

fn extrusion(profile: &[Vec2], length: f32, open_start: bool, open_end: bool, start_normal: Vec3, end_normal: Vec3) -> Mesh {
    /* Transform points to be cut according to the plane normal at each end */
    let start_points: Vec<_> = profile.iter().map(|pt| project_point(*pt, start_normal)).collect();
    let end_points: Vec<_> = profile.iter().map(|pt| project_point(*pt, end_normal) + length * Vec3::Z).collect();

    let mut tris = Vec::new();
    for i in 0..start_points.len() - 1 {
        let start0 = start_points[i];
        let start1 = start_points[i + 1];
        let end0 = end_points[i];
        let end1 = end_points[i + 1];

        tris.extend([start0, end0, start1]);
        tris.extend([start1, end0, end1]);
    }

    /* Close ends if necessary */
    // TODO the triangulation could be done once at startup for each polygon
    let floats: Vec<_> = profile.iter().flat_map(|v| [v.x, v.y]).collect();
    if let Ok(tri_idx) = earcutr::earcut(&floats, &[], 2) {
        if !open_start {
            let new_tris = tri_idx.iter()
                .rev()
                .map(|ix| start_points[*ix]);
            tris.extend(new_tris);
        }
        if !open_end {
            let new_tris = tri_idx.iter()
                .map(|ix| end_points[*ix]);
            tris.extend(new_tris);
        }
    } else {
        error!("Can't triangulate polygon for extrusion ends");
    }

    /* Create a mesh from the triangles */
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, tris)
        .with_computed_flat_normals()
}

fn create_bed_mesh(params: &TrackRenderParams, length: f32, open_start: bool, open_end: bool, start_normal: Vec3, end_normal: Vec3) -> Mesh {
    let bed_profile = BoxedPolyline2d::new(params.bed_profile.clone());
    extrusion(&bed_profile.vertices, length, open_start, open_end, start_normal, end_normal)
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
