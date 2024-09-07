use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use ndarray::s;

use crate::terrain::heightmap::heightmap_to_mesh;
use crate::terrain::rendering::TerrainRenderParams;
use crate::terrain::terrain::Terrain;
use crate::terrain::utils::get_copyable_range;
use crate::track::Segment;

const RENDERS_PER_FRAME: usize = 1;

#[derive(Resource)]
pub struct TrackRenderParams {
    rail_material: Handle<StandardMaterial>,
    rail_profile: Vec<Vec2>,
}

pub fn init_render_params(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mut rail_material = StandardMaterial::from(Color::srgb(0.8, 0.8, 0.8));
    rail_material.metallic = 0.8;

    let rail_points_half = [(-6.5, 0.0), (-6.5, 1.0), (-1.0, 2.0), (-0.75, 11.0), (-3.0, 12.0), (-3.0, 14.0), (-2.0, 15.0)];
    let rail_points_otherhalf = rail_points_half.iter().rev().map(|(x, y)| (-x, *y));
    let rail_points = rail_points_half.iter().copied().chain(rail_points_otherhalf);
    let rail_profile = rail_points.map(|(x, y)| Vec2::new(x * 0.01, y * 0.01));

    let params = TrackRenderParams {
        rail_material: materials.add(rail_material),
        rail_profile: rail_profile.collect(),
    };
    commands.insert_resource(params);
}

pub fn update_track_meshes(
    segments: Query<(Entity, &Segment, &Transform), Changed<Segment>>,
    params: Res<TrackRenderParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (segment_id, segment, transform) in segments.iter() {
        commands.entity(segment_id).despawn_descendants();

        let rail_mesh = create_rail_mesh(&params, segment.length, false, false);

        commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(rail_mesh),
            material: params.rail_material.clone(),
            transform: transform.clone(),
            ..default()
        });
    }
}

fn create_rail_mesh(params: &TrackRenderParams, length: f32, open_start: bool, open_end: bool) -> Mesh {
    const GAUGE: f32 = 1.435;

    let lateral_step = Vec3::new(GAUGE/2.0, 0.0, 0.0);

    let rail_profile = BoxedPolyline2d::new(params.rail_profile.clone());
    let mut mesh = open_extrusion(&rail_profile.vertices, length).translated_by(lateral_step);

    mesh.merge(&open_extrusion(&rail_profile.vertices, length).translated_by(-lateral_step));

    if !open_start {
        mesh.merge(&polygon(&rail_profile.vertices).translated_by(lateral_step));
        mesh.merge(&polygon(&rail_profile.vertices).translated_by(-lateral_step));
    }
    if !open_end {
        mesh.merge(&polygon(&rail_profile.vertices)
            .rotated_by(Quat::from_axis_angle(Vec3::Y, TAU/2.0))
            .translated_by(Vec3::new(0.0, 0.0, length))
            .translated_by(lateral_step));
        mesh.merge(&polygon(&rail_profile.vertices)
            .rotated_by(Quat::from_axis_angle(Vec3::Y, TAU/2.0))
            .translated_by(Vec3::new(0.0, 0.0, length))
            .translated_by(-lateral_step));
    }

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

    // if let Ok(tri) = triangulation_from_2d_vertices(&vertices, config) {
    //     let tris: Vec<_> = tri.triangles.iter()
    //         .flat_map(|x| x.iter().map(|vid| {
    //             let pt = profile[*vid as usize];
    //             Vec3::new(pt.x, pt.y, 0.0)
    //         }))
    //         .collect();
    //     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tris);
    //     mesh.compute_flat_normals();
    // } else {
    //     error!("Couldn't triangulate polygon at end of rail");
    // }

    mesh
}
