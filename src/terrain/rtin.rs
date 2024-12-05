pub struct Triangle {
    pub points: [[usize; 2]; 3]
}

pub struct Triangulation {
    pub triangles: Vec<Triangle>,
}

/**
 * Very simple triangulation that builds exactly two triangles per cell
 */
pub fn triangulate_basic(points: &ndarray::ArrayView2<f32>) -> Triangulation {
    let (h, w) = points.dim();
    let cells_shape = (h - 1, w - 1);

    let mut triangles = Vec::new();
    for i in 0..cells_shape.0 {
        for j in 0..cells_shape.1 {
            let v1 = [i, j];
            let v2 = [i, j + 1];
            let v3 = [i + 1, j];
            let v4 = [i + 1, j + 1];

            let tri1 = Triangle { points: [v1, v2, v3] };
            let tri2 = Triangle { points: [v3, v2, v4] };

            triangles.push(tri1);
            triangles.push(tri2);
        }
    }

    Triangulation { triangles }
}

/**
 * Use the "MARTINI" algorithm to triangulate a square array of height data.
 *
 * Transcribed to Rust from https://observablehq.com/@mourner/martin-real-time-rtin-terrain-mesh
 * internal comments are from the article's code samples
 */
pub fn triangulate_rtin(
    points: &ndarray::ArrayView2<f32>,
    threshold: f32
) -> Triangulation {
    let errors = build_error_map(points);
    let triangles = build_rtin_mesh(points, threshold, &errors.view());
    Triangulation { triangles }
}

fn build_error_map(points: &ndarray::ArrayView2<f32>) -> ndarray::Array2<f32> {
    let grid_size = points.shape()[0];
    let tile_size = grid_size - 1;

    let num_smallest = tile_size * tile_size;
    let num_triangles = num_smallest * 2 - 2;
    let last_level_index = num_triangles - num_smallest;

    let mut errors = ndarray::Array2::zeros(points.dim());

    // iterate over all possible triangles, starting from the smallest level
    for i in (0..num_triangles).rev() {

        // get triangle coordinates from its index in an implicit binary tree
        let mut id = i + 2;
        let (mut ax, mut ay, mut bx, mut by, mut cx, mut cy) = (0, 0, 0, 0, 0, 0);
        if id & 1 == 1 {
            (bx, by, cx) = (tile_size, tile_size, tile_size); // bottom-left triangle
        } else {
            (ax, ay, cy) = (tile_size, tile_size, tile_size); // top-right triangle
        }

        loop {
            id >>= 1;
            if id <= 1 { break; }

            let mx = (ax + bx) >> 1;
            let my = (ay + by) >> 1;

            if id & 1 == 1 { // left half
                bx = ax; by = ay;
                ax = cx; ay = cy;
            } else { // right half
                ax = bx; ay = by;
                bx = cx; by = cy;
            }
            cx = mx; cy = my;
        }

        let interpolated_height = (points[[ay, ax]] + points[[by, bx]]) / 2.0;
        let mx = (ax + bx) >> 1;
        let my = (ay + by) >> 1;
        let middle_error = (interpolated_height - points[[my, mx]]).abs();

        if i >= last_level_index { // smallest triangles
            errors[[my, mx]] = middle_error;
        } else { // bigger triangles; accumulate error with children
            let left_child_error = errors[[(ay + cy) >> 1, (ax + cx) >> 1]];
            let right_child_error = errors[[(by + cy) >> 1, (bx + cx) >> 1]];

            for err in [middle_error, left_child_error, right_child_error] {
                if err > errors[[my, mx]] {
                    errors[[my, mx]] = err;
                }
            }
        }
    }

    errors
}

fn build_rtin_mesh(
    points: &ndarray::ArrayView2<f32>,
    threshold: f32,
    errors: &ndarray::ArrayView2<f32>
) -> Vec<Triangle> {
    let grid_size = points.shape()[0];
    let tile_size = grid_size - 1;

    let mut triangles = Vec::new();

    #[allow(clippy::too_many_arguments)]
    fn process_triangle(
        ax: usize, ay: usize, bx: usize, by: usize, cx: usize, cy: usize,
        threshold: f32,
        errors: &ndarray::ArrayView2<f32>,
        triangles: &mut Vec<Triangle>,
    ) {
        // middle of the long edge
        let mx = (ax + bx) >> 1;
        let my = (ay + by) >> 1;

        if ax.abs_diff(cx) + ay.abs_diff(cy) > 1 && errors[[my, mx]] > threshold {
            // triangle doesn't approximate the surface well enough; split it into two
            process_triangle(cx, cy, ax, ay, mx, my, threshold, errors, triangles);
            process_triangle(bx, by, cx, cy, mx, my, threshold, errors, triangles);
        } else {
            // add a triangle to the final mesh
            // let v1 = Vec3::new(ay as f32, points[[ay, ax]], ax as f32);
            // let v2 = Vec3::new(by as f32, points[[by, bx]], bx as f32);
            // let v3 = Vec3::new(cy as f32, points[[cy, cx]], cx as f32);
            let v1 = [ay, ax];
            let v2 = [by, bx];
            let v3 = [cy, cx];

            let tri1 = Triangle { points: [v1, v2, v3] };

            triangles.push(tri1);
        }
    }

    process_triangle(
        0, 0, tile_size, tile_size, tile_size, 0,
        threshold,
        errors,
        &mut triangles);
    process_triangle(
        tile_size, tile_size, 0, 0, 0, tile_size,
        threshold,
        errors,
        &mut triangles);

    triangles
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn trivial() {
        let points = ndarray::arr2(&[[1.0, 2.0], [1.0, 3.0]]);

        let Triangulation { triangles } = triangulate_basic(&points.view());
        assert_eq!(2, triangles.len());
    }

    #[test]
    fn build_error_map1() {
        let points = ndarray::Array2::zeros([3, 3]);
        let errors = build_error_map(&points.view());
    }
}
