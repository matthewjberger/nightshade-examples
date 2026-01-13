use crate::ecs::{EdgeProfile, EdgeType, IVec2};
use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::prelude::*;

#[derive(Clone, Copy)]
pub struct TabParams {
    pub depth: f32,
    pub width: f32,
    pub neck_width: f32,
}

impl Default for TabParams {
    fn default() -> Self {
        Self {
            depth: 0.20,
            width: 0.32,
            neck_width: 0.14,
        }
    }
}

fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    p0 * mt3 + p1 * (3.0 * mt2 * t) + p2 * (3.0 * mt * t2) + p3 * t3
}

pub fn generate_edge_points_for_board(
    edge_type: EdgeType,
    samples_per_segment: usize,
    params: TabParams,
) -> Vec<Vec2> {
    generate_edge_points(edge_type, samples_per_segment, params)
}

fn generate_edge_points(
    edge_type: EdgeType,
    samples_per_segment: usize,
    params: TabParams,
) -> Vec<Vec2> {
    let mut points = Vec::new();

    match edge_type {
        EdgeType::Flat => {
            for index in 0..=samples_per_segment {
                let t = index as f32 / samples_per_segment as f32;
                points.push(nalgebra_glm::vec2(t, 0.0));
            }
        }
        EdgeType::Tab => {
            let neck_left = 0.5 - params.neck_width / 2.0;
            let neck_right = 0.5 + params.neck_width / 2.0;
            let head_left = 0.5 - params.width / 2.0;
            let head_right = 0.5 + params.width / 2.0;
            let neck_height = params.depth * 0.35;
            let head_height = params.depth;

            let p0 = nalgebra_glm::vec2(0.0, 0.0);
            let p1 = nalgebra_glm::vec2(neck_left - 0.02, 0.0);
            for index in 0..=samples_per_segment / 5 {
                let t = index as f32 / (samples_per_segment / 5) as f32;
                points.push(cubic_bezier(p0, p0, p1, p1, t));
            }

            let p0 = nalgebra_glm::vec2(neck_left - 0.02, 0.0);
            let p1 = nalgebra_glm::vec2(neck_left, 0.0);
            let p2 = nalgebra_glm::vec2(neck_left, neck_height * 0.5);
            let p3 = nalgebra_glm::vec2(neck_left, neck_height);
            for index in 1..=samples_per_segment / 5 {
                let t = index as f32 / (samples_per_segment / 5) as f32;
                points.push(cubic_bezier(p0, p1, p2, p3, t));
            }

            let p0 = nalgebra_glm::vec2(neck_left, neck_height);
            let p1 = nalgebra_glm::vec2(neck_left - 0.02, neck_height + 0.02);
            let p2 = nalgebra_glm::vec2(head_left, head_height - 0.04);
            let p3 = nalgebra_glm::vec2(head_left, head_height);
            for index in 1..=samples_per_segment / 5 {
                let t = index as f32 / (samples_per_segment / 5) as f32;
                points.push(cubic_bezier(p0, p1, p2, p3, t));
            }

            let p0 = nalgebra_glm::vec2(head_left, head_height);
            let p1 = nalgebra_glm::vec2(head_left, head_height + 0.06);
            let p2 = nalgebra_glm::vec2(head_right, head_height + 0.06);
            let p3 = nalgebra_glm::vec2(head_right, head_height);
            for index in 1..=samples_per_segment / 5 {
                let t = index as f32 / (samples_per_segment / 5) as f32;
                points.push(cubic_bezier(p0, p1, p2, p3, t));
            }

            let p0 = nalgebra_glm::vec2(head_right, head_height);
            let p1 = nalgebra_glm::vec2(head_right, head_height - 0.04);
            let p2 = nalgebra_glm::vec2(neck_right + 0.02, neck_height + 0.02);
            let p3 = nalgebra_glm::vec2(neck_right, neck_height);
            for index in 1..=samples_per_segment / 5 {
                let t = index as f32 / (samples_per_segment / 5) as f32;
                points.push(cubic_bezier(p0, p1, p2, p3, t));
            }

            let p0 = nalgebra_glm::vec2(neck_right, neck_height);
            let p1 = nalgebra_glm::vec2(neck_right, neck_height * 0.5);
            let p2 = nalgebra_glm::vec2(neck_right, 0.0);
            let p3 = nalgebra_glm::vec2(neck_right + 0.02, 0.0);
            for index in 1..=samples_per_segment / 5 {
                let t = index as f32 / (samples_per_segment / 5) as f32;
                points.push(cubic_bezier(p0, p1, p2, p3, t));
            }

            let p0 = nalgebra_glm::vec2(neck_right + 0.02, 0.0);
            let p1 = nalgebra_glm::vec2(1.0, 0.0);
            for index in 1..=samples_per_segment / 5 {
                let t = index as f32 / (samples_per_segment / 5) as f32;
                points.push(cubic_bezier(p0, p0, p1, p1, t));
            }
        }
        EdgeType::Blank => {
            let tab_points = generate_edge_points(EdgeType::Tab, samples_per_segment, params);
            for point in tab_points {
                points.push(nalgebra_glm::vec2(point.x, -point.y));
            }
        }
    }

    points
}

fn transform_edge_to_side(
    points: &[Vec2],
    side: usize,
    piece_width: f32,
    piece_height: f32,
) -> Vec<Vec2> {
    points
        .iter()
        .map(|point| {
            let scaled_x = point.x;
            let scaled_y = point.y;
            match side {
                0 => nalgebra_glm::vec2(
                    -piece_width / 2.0 + scaled_x * piece_width,
                    piece_height / 2.0 + scaled_y * piece_height,
                ),
                1 => nalgebra_glm::vec2(
                    piece_width / 2.0 + scaled_y * piece_width,
                    piece_height / 2.0 - scaled_x * piece_height,
                ),
                2 => nalgebra_glm::vec2(
                    piece_width / 2.0 - scaled_x * piece_width,
                    -piece_height / 2.0 - scaled_y * piece_height,
                ),
                3 => nalgebra_glm::vec2(
                    -piece_width / 2.0 - scaled_y * piece_width,
                    -piece_height / 2.0 + scaled_x * piece_height,
                ),
                _ => *point,
            }
        })
        .collect()
}

pub fn generate_piece_outline(
    profile: &EdgeProfile,
    piece_width: f32,
    piece_height: f32,
    samples_per_segment: usize,
    params: TabParams,
) -> Vec<Vec2> {
    let mut outline = Vec::new();

    let top_points = generate_edge_points(profile.top, samples_per_segment, params);
    let right_points = generate_edge_points(profile.right, samples_per_segment, params);
    let bottom_points = generate_edge_points(profile.bottom, samples_per_segment, params);
    let left_points = generate_edge_points(profile.left, samples_per_segment, params);

    let transformed_top = transform_edge_to_side(&top_points, 0, piece_width, piece_height);
    let transformed_right = transform_edge_to_side(&right_points, 1, piece_width, piece_height);
    let transformed_bottom = transform_edge_to_side(&bottom_points, 2, piece_width, piece_height);
    let transformed_left = transform_edge_to_side(&left_points, 3, piece_width, piece_height);

    outline.extend(transformed_top.iter());
    outline.extend(transformed_right.iter().skip(1));
    outline.extend(transformed_bottom.iter().skip(1));
    outline.extend(transformed_left.iter().skip(1));

    outline.pop();

    outline
}

fn compute_centroid(outline: &[Vec2]) -> Vec2 {
    if outline.is_empty() {
        return nalgebra_glm::vec2(0.0, 0.0);
    }
    let sum: Vec2 = outline
        .iter()
        .fold(nalgebra_glm::vec2(0.0, 0.0), |acc, p| acc + p);
    sum / outline.len() as f32
}

pub fn triangulate_outline(outline: &[Vec2]) -> Vec<u32> {
    if outline.len() < 3 {
        return Vec::new();
    }

    let mut indices = Vec::new();
    let center_index = outline.len() as u32;

    for index in 0..outline.len() {
        let next_index = (index + 1) % outline.len();
        indices.push(next_index as u32);
        indices.push(index as u32);
        indices.push(center_index);
    }

    indices
}

pub fn generate_piece_mesh(
    profile: &EdgeProfile,
    grid_pos: IVec2,
    grid_cols: u32,
    grid_rows: u32,
    piece_width: f32,
    piece_height: f32,
    params: TabParams,
) -> (Mesh, Vec<Vec2>) {
    let samples_per_segment = 16;
    let outline = generate_piece_outline(
        profile,
        piece_width,
        piece_height,
        samples_per_segment,
        params,
    );

    let centroid = compute_centroid(&outline);

    let padding = 0.3;
    let cell_u = 1.0 / grid_cols as f32;
    let cell_v = 1.0 / grid_rows as f32;

    let base_u = grid_pos.x as f32 * cell_u;
    let base_v = grid_pos.y as f32 * cell_v;

    let expanded_u_min = base_u - cell_u * padding;
    let expanded_u_max = base_u + cell_u * (1.0 + padding);
    let expanded_v_min = base_v - cell_v * padding;
    let expanded_v_max = base_v + cell_v * (1.0 + padding);

    let expanded_width = piece_width * (1.0 + 2.0 * padding);
    let expanded_height = piece_height * (1.0 + 2.0 * padding);

    let mut vertices = Vec::new();
    let normal = nalgebra_glm::vec3(0.0, 1.0, 0.0);

    for point in &outline {
        let local_u = (point.x + expanded_width / 2.0) / expanded_width;
        let local_v = 1.0 - (point.y + expanded_height / 2.0) / expanded_height;
        let tex_u = expanded_u_min + local_u * (expanded_u_max - expanded_u_min);
        let tex_v = expanded_v_min + local_v * (expanded_v_max - expanded_v_min);

        vertices.push(Vertex::with_tex_coords(
            nalgebra_glm::vec3(point.x, 0.0, -point.y),
            normal,
            [tex_u.clamp(0.0, 1.0), tex_v.clamp(0.0, 1.0)],
        ));
    }

    let center_local_u = (centroid.x + expanded_width / 2.0) / expanded_width;
    let center_local_v = 1.0 - (centroid.y + expanded_height / 2.0) / expanded_height;
    let center_tex_u = expanded_u_min + center_local_u * (expanded_u_max - expanded_u_min);
    let center_tex_v = expanded_v_min + center_local_v * (expanded_v_max - expanded_v_min);

    vertices.push(Vertex::with_tex_coords(
        nalgebra_glm::vec3(centroid.x, 0.0, -centroid.y),
        normal,
        [center_tex_u.clamp(0.0, 1.0), center_tex_v.clamp(0.0, 1.0)],
    ));

    let indices = triangulate_outline(&outline);

    (Mesh::new(vertices, indices), outline)
}

pub fn point_in_polygon(point: Vec2, polygon: &[Vec2], rotation: u8) -> bool {
    let rotated_point = match rotation {
        0 => point,
        1 => nalgebra_glm::vec2(-point.y, point.x),
        2 => nalgebra_glm::vec2(-point.x, -point.y),
        3 => nalgebra_glm::vec2(point.y, -point.x),
        _ => point,
    };

    let mut inside = false;
    let n = polygon.len();

    let mut j = n - 1;
    for i in 0..n {
        let pi = polygon[i];
        let pj = polygon[j];

        if ((pi.y > rotated_point.y) != (pj.y > rotated_point.y))
            && (rotated_point.x < (pj.x - pi.x) * (rotated_point.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }

    inside
}
