use std::env::join_paths;
use std::f32::consts::FRAC_PI_2;
use std::iter::zip;
use std::path::PathBuf;

use bevy::{
    render::mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use noise::{NoiseFn, Simplex};

use crate::datafile;

fn heightmap_to_mesh(heights: &Vec<Vec<f32>>, scale: &Vec3) -> Mesh {
    let height = heights.len();
    let width = heights[0].len();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut verts = Vec::new();
    let mut cols = Vec::new();
    for i in 0..height {
        for j in 0..width {
            let px = i as f32 * scale.x;
            let py = heights[i][j] * scale.y;
            let pz = j as f32 * scale.z;
            verts.push(Vec3::new(px, py, pz));
            cols.push(Vec4::new(0.0, heights[i][j], 0.0, 1.0));
        }
    }
    let centres_offset = verts.len();
    for i in 0..height-1 {
        for j in 0..width-1 {
            let total_height = heights[i][j] + heights[i+1][j] + heights[i][j+1] + heights[i+1][j+1];
            let px = i as f32 * scale.x;
            let py = total_height / 4.0 * scale.y;
            let pz = j as f32 * scale.z;
            verts.push(Vec3::new(px + scale.x/2.0, py, pz + scale.z/2.0));
            cols.push(Vec4::new(0.0, total_height / 4.0, 0.0, 1.0));
        }
    }
    let mut norms = vec![Vec3::default(); verts.len()];
    let mut norm_counts = vec![0; norms.len()];
    let mut tris = Vec::new();
    let stride = width;
    let centres_stride = width - 1;
    for i in 0..height - 1 {
        for j in 0..width - 1 {
            let vert00 = (i*stride + j) as u32;
            let vert01 = (i*stride + j+1) as u32;
            let vert10 = ((i+1)*stride + j) as u32;
            let vert11 = ((i+1)*stride + j+1) as u32;
            let centre = (centres_offset + (i*centres_stride + j)) as u32;

            fn add_triangle(
                inds: &[u32],
                tris: &mut Vec<u32>,
                norms: &mut Vec<Vec3>,
                norm_counts: &mut Vec<usize>,
                verts: &Vec<Vec3>
            ) {
                tris.extend_from_slice(inds);

                let p0 = verts[inds[0] as usize];
                let p1 = verts[inds[1] as usize];
                let p2 = verts[inds[2] as usize];

                let norm = (p1 - p0).cross(p2 - p1).normalize();

                for ind in inds {
                    norms[*ind as usize] += norm;
                    norm_counts[*ind as usize] += 1;
                }
            }

            add_triangle(&[vert00, vert01, centre], &mut tris, &mut norms, &mut norm_counts, &verts);
            add_triangle(&[vert01, vert11, centre], &mut tris, &mut norms, &mut norm_counts, &verts);
            add_triangle(&[vert11, vert10, centre], &mut tris, &mut norms, &mut norm_counts, &verts);
            add_triangle(&[vert10, vert00, centre], &mut tris, &mut norms, &mut norm_counts, &verts);
        }
    }

    for (norm, cnt) in zip(&mut norms, &norm_counts) {
        *norm *= 1.0 / *cnt as f32;
        *norm = norm.normalize();
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION,verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, norms);
    //mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, cols);
    mesh.set_indices(Some(Indices::U32(tris)));

    mesh
}

pub(crate) fn create_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let noise = Simplex::new(42);

    const GRID_SIZE: usize = 10;
    let mut grid = Vec::new();
    for i in 0..GRID_SIZE + 1 {
        grid.push(Vec::new());
        for j in 0..GRID_SIZE + 1 {
            let height = noise.get([i as f64 / GRID_SIZE as f64 * 1.0, j as f64  / GRID_SIZE as f64 * 1.0]) as f32;
            let height = height;
            grid[i].push(height);
        }
    }

    const GRID_SPACING: f32 = 100.0 / GRID_SIZE as f32;

    let mesh= heightmap_to_mesh(&grid, &Vec3::new(GRID_SPACING, 25.0, GRID_SPACING));
    let mesh = meshes.add(mesh);

    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.8, 0.4),
            perceptual_roughness: 0.9,
            ..default()
        }),
        ..default()
    });
}

pub fn load_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let filename = PathBuf::from("data/jvl.json");
    let data = datafile::load_datafile(&filename).unwrap();

    let SCALING = 10;
    let all_width = 480 * 8 / SCALING as usize;
    let all_height=  720 * 3 / SCALING as usize;
    let mut all_elevation_data = vec!(vec!(0.0; all_width); all_height);
    for chunk in &data.chunks {
        let mut elevation_filename = filename.clone();
        elevation_filename.pop();
        elevation_filename.push(&chunk.elevation);
        let elevation_data = datafile::load_elevation(&elevation_filename).unwrap();
        info!("Loaded elevation {}", elevation_filename.display());

        let elevation_data = elevation_data[0..720].iter().step_by(SCALING).map(
            |x| x[0..480].iter().step_by(SCALING).map(|x| *x).collect::<Vec<_>>()
        ).collect::<Vec<_>>();

        let i_offset = chunk.position.1 as usize * 720 / SCALING;
        let j_offset = chunk.position.0 as usize * 480 / SCALING;
        for i in 0..elevation_data.len() {
            for j in 0..elevation_data[i].len() {
                all_elevation_data[i + i_offset][j + j_offset] = elevation_data[i][j];
            }
        }
    }

    let all_elevation_data = all_elevation_data.iter().map(|x| x.clone()).rev().collect::<Vec<_>>();

    let mesh = heightmap_to_mesh(&all_elevation_data, &Vec3::new( SCALING as f32, 1.0, SCALING as f32));
    let mesh = meshes.add(mesh);

    info!("Created mesh");

    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.4, 0.8, 0.6),
            perceptual_roughness: 0.9,
            ..default()
        }),
        ..default()
    });
}
