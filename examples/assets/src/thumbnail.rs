use nightshade::prelude::*;
use std::path::Path;

pub fn generate_model_thumbnail(path: &Path, size: u32) -> Option<egui::ColorImage> {
    let result = nightshade::ecs::prefab::import_gltf_from_path(path).ok()?;

    let mut all_positions: Vec<Vec3> = Vec::new();
    let mut all_normals: Vec<Vec3> = Vec::new();
    let mut all_triangles: Vec<[usize; 3]> = Vec::new();

    let mut vertex_offset = 0;
    for mesh in result.meshes.values() {
        for vertex in &mesh.vertices {
            all_positions.push(Vec3::new(
                vertex.position[0],
                vertex.position[1],
                vertex.position[2],
            ));
            all_normals.push(Vec3::new(
                vertex.normal[0],
                vertex.normal[1],
                vertex.normal[2],
            ));
        }
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                all_triangles.push([
                    chunk[0] as usize + vertex_offset,
                    chunk[1] as usize + vertex_offset,
                    chunk[2] as usize + vertex_offset,
                ]);
            }
        }
        vertex_offset += mesh.vertices.len();
    }

    if all_positions.is_empty() || all_triangles.is_empty() {
        return None;
    }

    let mut aabb_min = all_positions[0];
    let mut aabb_max = all_positions[0];
    for position in &all_positions {
        aabb_min.x = aabb_min.x.min(position.x);
        aabb_min.y = aabb_min.y.min(position.y);
        aabb_min.z = aabb_min.z.min(position.z);
        aabb_max.x = aabb_max.x.max(position.x);
        aabb_max.y = aabb_max.y.max(position.y);
        aabb_max.z = aabb_max.z.max(position.z);
    }

    let center = (aabb_min + aabb_max) * 0.5;
    let extent = aabb_max - aabb_min;
    let diagonal = nalgebra_glm::length(&extent);

    if diagonal < 1e-6 {
        return None;
    }

    let camera_direction = nalgebra_glm::normalize(&Vec3::new(1.0, 0.8, 1.0));
    let camera_position = center + camera_direction * diagonal;
    let up = Vec3::new(0.0, 1.0, 0.0);
    let view = nalgebra_glm::look_at(&camera_position, &center, &up);

    let half_extent = diagonal * 0.6;
    let projection = nalgebra_glm::ortho(
        -half_extent,
        half_extent,
        -half_extent,
        half_extent,
        0.01,
        diagonal * 3.0,
    );

    let view_projection = projection * view;

    let pixel_count = size as usize;
    let mut color_buffer = vec![[40u8, 40, 48, 255]; pixel_count * pixel_count];
    let mut depth_buffer = vec![f32::INFINITY; pixel_count * pixel_count];

    let light_direction = nalgebra_glm::normalize(&Vec3::new(0.6, 0.8, 0.5));
    let base_color = Vec3::new(0.7, 0.75, 0.8);

    for triangle in &all_triangles {
        let p0 = &all_positions[triangle[0]];
        let p1 = &all_positions[triangle[1]];
        let p2 = &all_positions[triangle[2]];
        let n0 = &all_normals[triangle[0]];
        let n1 = &all_normals[triangle[1]];
        let n2 = &all_normals[triangle[2]];

        let clip0 = view_projection * nalgebra_glm::vec4(p0.x, p0.y, p0.z, 1.0);
        let clip1 = view_projection * nalgebra_glm::vec4(p1.x, p1.y, p1.z, 1.0);
        let clip2 = view_projection * nalgebra_glm::vec4(p2.x, p2.y, p2.z, 1.0);

        let ndc0 = Vec3::new(clip0.x / clip0.w, clip0.y / clip0.w, clip0.z / clip0.w);
        let ndc1 = Vec3::new(clip1.x / clip1.w, clip1.y / clip1.w, clip1.z / clip1.w);
        let ndc2 = Vec3::new(clip2.x / clip2.w, clip2.y / clip2.w, clip2.z / clip2.w);

        let screen0 = ndc_to_screen(&ndc0, pixel_count);
        let screen1 = ndc_to_screen(&ndc1, pixel_count);
        let screen2 = ndc_to_screen(&ndc2, pixel_count);

        let min_x = screen0
            .0
            .min(screen1.0)
            .min(screen2.0)
            .max(0.0)
            .floor() as usize;
        let max_x = (screen0.0.max(screen1.0).max(screen2.0).ceil() as usize)
            .min(pixel_count.saturating_sub(1));
        let min_y = screen0
            .1
            .min(screen1.1)
            .min(screen2.1)
            .max(0.0)
            .floor() as usize;
        let max_y = (screen0.1.max(screen1.1).max(screen2.1).ceil() as usize)
            .min(pixel_count.saturating_sub(1));

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                let (bary_u, bary_v, bary_w) =
                    barycentric(screen0, screen1, screen2, (px, py));

                if bary_u < 0.0 || bary_v < 0.0 || bary_w < 0.0 {
                    continue;
                }

                let depth = ndc0.z * bary_u + ndc1.z * bary_v + ndc2.z * bary_w;
                let pixel_index = y * pixel_count + x;

                if depth < depth_buffer[pixel_index] {
                    depth_buffer[pixel_index] = depth;

                    let interpolated_normal = nalgebra_glm::normalize(
                        &(*n0 * bary_u + *n1 * bary_v + *n2 * bary_w),
                    );
                    let ndotl =
                        nalgebra_glm::dot(&interpolated_normal, &light_direction).max(0.0);
                    let ambient = 0.15;
                    let lighting = ambient + (1.0 - ambient) * ndotl;

                    color_buffer[pixel_index] = [
                        (base_color.x * lighting * 255.0).clamp(0.0, 255.0) as u8,
                        (base_color.y * lighting * 255.0).clamp(0.0, 255.0) as u8,
                        (base_color.z * lighting * 255.0).clamp(0.0, 255.0) as u8,
                        255,
                    ];
                }
            }
        }
    }

    let pixels: Vec<u8> = color_buffer.into_iter().flatten().collect();
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [pixel_count, pixel_count],
        &pixels,
    ))
}

fn ndc_to_screen(ndc: &Vec3, size: usize) -> (f32, f32) {
    let x = (ndc.x * 0.5 + 0.5) * size as f32;
    let y = (1.0 - (ndc.y * 0.5 + 0.5)) * size as f32;
    (x, y)
}

fn barycentric(
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    p: (f32, f32),
) -> (f32, f32, f32) {
    let v0 = (b.0 - a.0, b.1 - a.1);
    let v1 = (c.0 - a.0, c.1 - a.1);
    let v2 = (p.0 - a.0, p.1 - a.1);

    let d00 = v0.0 * v0.0 + v0.1 * v0.1;
    let d01 = v0.0 * v1.0 + v0.1 * v1.1;
    let d11 = v1.0 * v1.0 + v1.1 * v1.1;
    let d20 = v2.0 * v0.0 + v2.1 * v0.1;
    let d21 = v2.0 * v1.0 + v2.1 * v1.1;

    let denominator = d00 * d11 - d01 * d01;
    if denominator.abs() < 1e-10 {
        return (-1.0, -1.0, -1.0);
    }
    let inv_denominator = 1.0 / denominator;
    let v = (d11 * d20 - d01 * d21) * inv_denominator;
    let w = (d00 * d21 - d01 * d20) * inv_denominator;
    let u = 1.0 - v - w;
    (u, v, w)
}
