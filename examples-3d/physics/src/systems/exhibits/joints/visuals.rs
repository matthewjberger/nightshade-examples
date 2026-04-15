use crate::ecs::{GameWorld, ROPE_JOINT_VISUAL, SPHERICAL_JOINT_VISUAL, SPRING_JOINT_VISUAL};
use nightshade::prelude::*;

pub fn update_joint_visuals(game_world: &GameWorld, world: &mut World) {
    let spherical_entities: Vec<freecs::Entity> =
        game_world.query_entities(SPHERICAL_JOINT_VISUAL).collect();

    for game_entity in spherical_entities {
        let Some(visual) = game_world.get_spherical_joint_visual(game_entity) else {
            continue;
        };
        let anchor_pos = world
            .core
            .get_global_transform(visual.anchor_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));
        let ball_pos = world
            .core
            .get_global_transform(visual.ball_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));

        let anchor_attach = anchor_pos - nalgebra_glm::vec3(0.0, 0.15, 0.0);
        let ball_attach = ball_pos + nalgebra_glm::vec3(0.0, 0.2, 0.0);
        let midpoint = (anchor_attach + ball_attach) * 0.5;
        let distance = nalgebra_glm::distance(&anchor_attach, &ball_attach);

        let rotation = rotation_from_to_direction(
            nalgebra_glm::vec3(0.0, 1.0, 0.0),
            ball_attach - anchor_attach,
        );

        if let Some(transform) = world.core.get_local_transform_mut(visual.rod_entity) {
            transform.translation = midpoint;
            transform.rotation = rotation;
            transform.scale = nalgebra_glm::vec3(0.03, distance.max(0.01), 0.03);
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, visual.rod_entity);
    }

    let rope_entities: Vec<freecs::Entity> = game_world.query_entities(ROPE_JOINT_VISUAL).collect();

    for game_entity in rope_entities {
        let Some(visual) = game_world.get_rope_joint_visual(game_entity) else {
            continue;
        };
        let anchor_pos = world
            .core
            .get_global_transform(visual.anchor_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));
        let ball_pos = world
            .core
            .get_global_transform(visual.ball_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));

        let anchor_attach = anchor_pos - nalgebra_glm::vec3(0.0, 0.15, 0.0);
        let midpoint = (anchor_attach + ball_pos) * 0.5;
        let distance = nalgebra_glm::distance(&anchor_attach, &ball_pos);

        let rotation =
            rotation_from_to_direction(nalgebra_glm::vec3(0.0, 1.0, 0.0), ball_pos - anchor_attach);

        if let Some(transform) = world.core.get_local_transform_mut(visual.rope_entity) {
            transform.translation = midpoint;
            transform.rotation = rotation;
            transform.scale = nalgebra_glm::vec3(0.02, distance.max(0.01), 0.02);
        }
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, visual.rope_entity);
    }

    let spring_entities: Vec<freecs::Entity> =
        game_world.query_entities(SPRING_JOINT_VISUAL).collect();

    for game_entity in spring_entities {
        let Some(visual) = game_world.get_spring_joint_visual(game_entity) else {
            continue;
        };
        let anchor_pos = world
            .core
            .get_global_transform(visual.anchor_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));
        let object_pos = world
            .core
            .get_global_transform(visual.object_entity)
            .map(|t| t.translation())
            .unwrap_or(nalgebra_glm::vec3(0.0, 0.0, 0.0));

        let anchor_attach = anchor_pos - nalgebra_glm::vec3(0.0, 0.15, 0.0);
        let object_attach = object_pos + nalgebra_glm::vec3(0.0, 0.2, 0.0);
        let total_distance = nalgebra_glm::distance(&anchor_attach, &object_attach);

        let num_coils = visual.spring_entities.len();
        if num_coils == 0 {
            continue;
        }

        let direction = if total_distance > 0.001 {
            nalgebra_glm::normalize(&(object_attach - anchor_attach))
        } else {
            nalgebra_glm::vec3(0.0, -1.0, 0.0)
        };

        let up = nalgebra_glm::vec3(0.0, 1.0, 0.0);
        let coil_radius = 0.08;

        for (coil_index, &coil_entity) in visual.spring_entities.iter().enumerate() {
            let t = (coil_index as f32 + 0.5) / num_coils as f32;
            let base_pos = anchor_attach + direction * (t * total_distance);

            let angle = coil_index as f32 * std::f32::consts::PI;
            let perpendicular = if direction.y.abs() > 0.999_f32 {
                nalgebra_glm::vec3(1.0, 0.0, 0.0)
            } else {
                nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &up))
            };
            let perpendicular2 = nalgebra_glm::cross(&direction, &perpendicular);

            let offset = perpendicular * (angle.cos() * coil_radius)
                + perpendicular2 * (angle.sin() * coil_radius);
            let coil_pos = base_pos + offset;

            let next_t = ((coil_index + 1) as f32 + 0.5) / num_coils as f32;
            let next_base_pos = anchor_attach + direction * (next_t * total_distance);
            let next_angle = (coil_index + 1) as f32 * std::f32::consts::PI;
            let next_offset = perpendicular * (next_angle.cos() * coil_radius)
                + perpendicular2 * (next_angle.sin() * coil_radius);
            let next_coil_pos = next_base_pos + next_offset;

            let coil_direction_vec = next_coil_pos - coil_pos;
            let coil_length = nalgebra_glm::length(&coil_direction_vec);

            let coil_rotation =
                rotation_from_to_direction(nalgebra_glm::vec3(0.0, 1.0, 0.0), coil_direction_vec);

            let midpoint = (coil_pos + next_coil_pos) * 0.5;

            if let Some(transform) = world.core.get_local_transform_mut(coil_entity) {
                transform.translation = midpoint;
                transform.rotation = coil_rotation;
                transform.scale = nalgebra_glm::vec3(0.015, coil_length.max(0.01), 0.015);
            }
            nightshade::ecs::transform::commands::mark_local_transform_dirty(world, coil_entity);
        }
    }
}

fn rotation_from_to_direction(from: Vec3, to: Vec3) -> nalgebra_glm::Quat {
    let to_normalized = nalgebra_glm::normalize(&to);
    let from_normalized = nalgebra_glm::normalize(&from);

    let dot: f32 = from_normalized.dot(&to_normalized);

    if dot > 0.9999 {
        return nalgebra_glm::quat_identity();
    }

    if dot < -0.9999 {
        let mut axis = nalgebra_glm::cross(&nalgebra_glm::vec3(1.0, 0.0, 0.0), &from_normalized);
        if nalgebra_glm::length(&axis) < 0.0001 {
            axis = nalgebra_glm::cross(&nalgebra_glm::vec3(0.0, 1.0, 0.0), &from_normalized);
        }
        axis = nalgebra_glm::normalize(&axis);
        return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &axis);
    }

    let axis = nalgebra_glm::cross(&from_normalized, &to_normalized);
    let s = ((1.0 + dot) * 2.0).sqrt();
    let inv_s = 1.0 / s;

    nalgebra_glm::quat(axis.x * inv_s, axis.y * inv_s, axis.z * inv_s, s * 0.5)
}
