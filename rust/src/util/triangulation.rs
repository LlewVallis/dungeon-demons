use std::collections::HashSet;

use crate::util::vector::Vec2;
use crate::vec2;

pub fn triangulate(vertices: &[Vec2]) -> Vec<(Vec2, Vec2, Vec2)> {
    let super_triangle = create_super_triangle(vertices);
    let mut triangles = vec![super_triangle];

    for vertex in vertices.iter() {
        let mut polygon_edges = Vec::new();

        triangles.retain(|triangle| {
            let bad = in_triangle_circumference(*vertex, *triangle);

            if bad {
                polygon_edges.extend([
                    (triangle.0, triangle.1),
                    (triangle.1, triangle.2),
                    (triangle.2, triangle.0),
                ]);
            }

            !bad
        });

        let mut bad_edge_indices = HashSet::new();

        for (i, first_edge) in polygon_edges.iter().enumerate() {
            for (j, second_edge) in polygon_edges[i + 1..].iter().enumerate() {
                if edge_almost_equal(*first_edge, *second_edge) {
                    bad_edge_indices.insert(i);
                    bad_edge_indices.insert(j + i + 1);
                }
            }
        }

        for (i, edge) in polygon_edges.iter().enumerate() {
            if !bad_edge_indices.contains(&i) {
                triangles.push((edge.0, edge.1, *vertex));
            }
        }
    }

    triangles.retain(|triangle| {
        let vertices = [triangle.0, triangle.1, triangle.2];
        let super_triangle_vertices = [super_triangle.0, super_triangle.1, super_triangle.2];

        for vertex in vertices {
            for super_triangle_vertex in super_triangle_vertices {
                if vector_almost_equal(vertex, super_triangle_vertex) {
                    return false;
                }
            }
        }

        true
    });

    triangles
}

fn create_super_triangle(vertices: &[Vec2]) -> (Vec2, Vec2, Vec2) {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for vertex in vertices {
        min_x = min_x.min(vertex.x());
        min_y = min_y.min(vertex.y());
        max_x = max_x.max(vertex.x());
        max_y = max_y.max(vertex.y());
    }

    let delta_x = max_x - min_x;
    let delta_y = max_y - min_y;
    let delta_max = f64::max(delta_x, delta_y);

    let mid_x = (min_x + max_x) / 2.0;
    let mid_y = (min_y + max_y) / 2.0;

    let p1 = vec2(mid_x - 20.0 * delta_max, mid_y - delta_max);
    let p2 = vec2(mid_x, mid_y + 20.0 * delta_max);
    let p3 = vec2(mid_x + 20.0 + delta_max, mid_y - delta_max);

    (p1, p2, p3)
}

fn in_triangle_circumference(position: Vec2, (a, b, c): (Vec2, Vec2, Vec2)) -> bool {
    let ab = a.length_squared();
    let cd = b.length_squared();
    let ef = c.length_squared();

    let center_x = (ab * (c.y() - b.y()) + cd * (a.y() - c.y()) + ef * (b.y() - a.y()))
        / (a.x() * (c.y() - b.y()) + b.x() * (a.y() - c.y()) + c.x() * (b.y() - a.y()));

    let center_y = (ab * (c.x() - b.x()) + cd * (a.x() - c.x()) + ef * (b.x() - a.x()))
        / (a.y() * (c.x() - b.x()) + b.y() * (a.x() - c.x()) + c.y() * (b.x() - a.x()));

    let center = vec2(center_x / 2.0, center_y / 2.0);
    let radius = a.distance_squared(center);
    let dist = position.distance_squared(center);
    dist <= radius
}

fn almost_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.001
}

fn vector_almost_equal(a: Vec2, b: Vec2) -> bool {
    almost_equal(a.x(), b.x()) && almost_equal(a.y(), b.y())
}

fn edge_almost_equal(a: (Vec2, Vec2), b: (Vec2, Vec2)) -> bool {
    (vector_almost_equal(a.0, b.0) && vector_almost_equal(a.1, b.1))
        || (vector_almost_equal(a.0, b.1) && vector_almost_equal(a.1, b.0))
}
