use nightshade::ecs::mesh::{Mesh, Vertex};
use nightshade::prelude::*;

const TUBE_SEGMENTS: u32 = 8;
const SPHERE_RINGS: u32 = 6;
const SPHERE_SECTORS: u32 = 8;

pub fn create_tube_between_points(
    start: &Vec3,
    end: &Vec3,
    radius: f32,
) -> (Vec<Vertex>, Vec<u32>) {
    let direction = end - start;
    let length = nalgebra_glm::length(&direction);
    if length < 1e-6 {
        return (Vec::new(), Vec::new());
    }
    let forward = direction / length;

    let up_candidate = if forward.y.abs() > 0.99 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&forward, &up_candidate));
    let up = nalgebra_glm::cross(&right, &forward);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring_index in 0..=1 {
        let center = if ring_index == 0 { *start } else { *end };
        let v_coord = ring_index as f32;

        for segment_index in 0..=TUBE_SEGMENTS {
            let angle = (segment_index as f32 / TUBE_SEGMENTS as f32) * std::f32::consts::TAU;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let normal = right * cos_a + up * sin_a;
            let position = center + normal * radius;
            let u_coord = segment_index as f32 / TUBE_SEGMENTS as f32;

            vertices.push(Vertex::with_tex_coords(
                position,
                nalgebra_glm::normalize(&normal),
                [u_coord, v_coord],
            ));
        }
    }

    let ring_verts = TUBE_SEGMENTS + 1;
    for segment_index in 0..TUBE_SEGMENTS {
        let current = segment_index;
        let next = segment_index + 1;
        let bottom = current;
        let bottom_next = next;
        let top = current + ring_verts;
        let top_next = next + ring_verts;

        indices.extend_from_slice(&[bottom, top, bottom_next, bottom_next, top, top_next]);
    }

    (vertices, indices)
}

pub fn create_sphere_at(center: &Vec3, radius: f32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring_index in 0..=SPHERE_RINGS {
        let phi = (ring_index as f32 / SPHERE_RINGS as f32) * std::f32::consts::PI;
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        for sector_index in 0..=SPHERE_SECTORS {
            let theta = (sector_index as f32 / SPHERE_SECTORS as f32) * std::f32::consts::TAU;
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();

            let normal = Vec3::new(sin_phi * cos_theta, cos_phi, sin_phi * sin_theta);
            let position = center + normal * radius;

            vertices.push(Vertex::with_tex_coords(
                position,
                normal,
                [
                    sector_index as f32 / SPHERE_SECTORS as f32,
                    ring_index as f32 / SPHERE_RINGS as f32,
                ],
            ));
        }
    }

    let sector_count = SPHERE_SECTORS + 1;
    for ring_index in 0..SPHERE_RINGS {
        for sector_index in 0..SPHERE_SECTORS {
            let current = ring_index * sector_count + sector_index;
            let next_ring = (ring_index + 1) * sector_count + sector_index;

            indices.extend_from_slice(&[
                current,
                next_ring,
                current + 1,
                current + 1,
                next_ring,
                next_ring + 1,
            ]);
        }
    }

    (vertices, indices)
}

pub fn build_neon_tube_mesh(polylines: &[Vec<Vec3>], tube_radius: f32) -> Mesh {
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();

    for polyline in polylines {
        for point in polyline {
            let (sphere_verts, sphere_idxs) = create_sphere_at(point, tube_radius);
            let base = all_vertices.len() as u32;
            all_vertices.extend(sphere_verts);
            all_indices.extend(sphere_idxs.iter().map(|index| index + base));
        }

        for segment_index in 0..polyline.len().saturating_sub(1) {
            let (tube_verts, tube_idxs) = create_tube_between_points(
                &polyline[segment_index],
                &polyline[segment_index + 1],
                tube_radius,
            );
            let base = all_vertices.len() as u32;
            all_vertices.extend(tube_verts);
            all_indices.extend(tube_idxs.iter().map(|index| index + base));
        }
    }

    Mesh::new(all_vertices, all_indices)
}
