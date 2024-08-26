use std::iter::zip;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

pub fn heightmap_to_mesh(heights: &Vec<Vec<f32>>, scale: &Vec3) -> Mesh {
    let height = heights.len();
    let width = heights[0].len();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    let mut verts = Vec::new();
    let mut cols = Vec::new();
    for i in 0..height {
        for j in 0..width {
            let px = j as f32 * scale.x;
            let py = heights[i][j] * scale.y;
            let pz = i as f32 * scale.z;
            verts.push(Vec3::new(px, py, pz));
            cols.push(Vec4::new(0.0, heights[i][j], 0.0, 1.0));
        }
    }
    let centres_offset = verts.len();
    for i in 0..height-1 {
        for j in 0..width-1 {
            let total_height = heights[i][j] + heights[i+1][j] + heights[i][j+1] + heights[i+1][j+1];
            let px = j as f32 * scale.x;
            let py = total_height / 4.0 * scale.y;
            let pz = i as f32 * scale.z;
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
                let p0 = verts[inds[0] as usize];
                let p1 = verts[inds[1] as usize];
                let p2 = verts[inds[2] as usize];

                if p0.y < 0.01 && p1.y < 0.01 && p1.y < 0.01 { return; }

                tris.extend_from_slice(inds);

                let norm = (p1 - p0).cross(p2 - p1).normalize();

                for ind in inds {
                    norms[*ind as usize] += norm;
                    norm_counts[*ind as usize] += 1;
                }
            }

            add_triangle(&[vert00, centre, vert01], &mut tris, &mut norms, &mut norm_counts, &verts);
            add_triangle(&[vert01, centre, vert11], &mut tris, &mut norms, &mut norm_counts, &verts);
            add_triangle(&[vert11, centre, vert10], &mut tris, &mut norms, &mut norm_counts, &verts);
            add_triangle(&[vert10, centre, vert00], &mut tris, &mut norms, &mut norm_counts, &verts);
        }
    }

    for (norm, cnt) in zip(&mut norms, &norm_counts) {
        *norm *= 1.0 / *cnt as f32;
        *norm = norm.normalize();
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION,verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, norms);
    //mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, cols);
    mesh.insert_indices(Indices::U32(tris));

    mesh
}
