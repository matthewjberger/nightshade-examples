use crate::ecs::{GameWorld, InputMode, LEVER};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

pub(super) fn update_manipulated_lever(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
) {
    let Some(lever_game_entity) = game_world.resources.interaction.manipulated_entity_of_kind(&crate::ecs::InteractableKind::Lever) else {
        return;
    };

    let Some(pivot_position) = game_world
        .get_lever(lever_game_entity)
        .map(|lever| lever.pivot_position)
    else {
        return;
    };

    if nalgebra_glm::distance(&camera_position, &pivot_position) > game_world.resources.config.interact_range * 3.0 {
        game_world.resources.interaction.manipulated = None;
        return;
    }

    let dt = world.resources.physics.fixed_timestep;

    let mouse_input = if game_world.resources.input_mode == InputMode::MouseKeyboard {
        world.resources.input.mouse.raw_mouse_delta.y * 1.5
    } else {
        0.0
    };

    let gamepad_input = if game_world.resources.input_mode == InputMode::Gamepad {
        if let Some(gamepad) = query_active_gamepad(world) {
            let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
            let deadzone = 0.15;
            if right_stick_y.abs() > deadzone {
                -right_stick_y * 3.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    } else {
        0.0
    };

    let torque = mouse_input + gamepad_input;
    let friction = 5.0;

    let Some(lever) = game_world.get_lever_mut(lever_game_entity) else {
        return;
    };

    lever.angular_velocity += torque * dt;
    lever.angular_velocity -= lever.angular_velocity * friction * dt;

    let angle_delta = lever.angular_velocity * dt;
    let new_angle = (lever.current_angle + angle_delta).clamp(lever.min_angle, lever.max_angle);

    if (new_angle - lever.min_angle).abs() < 0.001 && lever.angular_velocity < 0.0 {
        lever.angular_velocity = -lever.angular_velocity * 0.2;
    }
    if (new_angle - lever.max_angle).abs() < 0.001 && lever.angular_velocity > 0.0 {
        lever.angular_velocity = -lever.angular_velocity * 0.2;
    }

    lever.current_angle = new_angle;

    apply_lever_transform(game_world, world, lever_game_entity);
}

pub fn apply_lever_transform(
    game_world: &GameWorld,
    world: &mut World,
    lever_game_entity: freecs::Entity,
) {
    let Some(lever) = game_world.get_lever(lever_game_entity) else {
        return;
    };

    let rotation =
        nalgebra_glm::quat_angle_axis(lever.current_angle, &nalgebra_glm::vec3(1.0, 0.0, 0.0));

    if let Some(transform) = world.core.get_local_transform_mut(lever.pivot_entity) {
        transform.rotation = rotation;
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, lever.pivot_entity);

    let local_offset = nalgebra_glm::vec3(0.0, 0.0, lever.arm_half_length);
    let rotated_offset = nalgebra_glm::quat_rotate_vec3(&rotation, &local_offset);
    let center_pos = lever.pivot_position + rotated_offset;

    if let Some(transform) = world.core.get_local_transform_mut(lever.collider_entity) {
        transform.translation = center_pos;
        transform.rotation = rotation;
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(
        world,
        lever.collider_entity,
    );

    if let Some(rb) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(lever.collider_rb_handle)
    {
        let rapier_rotation =
            rapier3d::na::UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(
                rotation.w,
                rotation.coords.x,
                rotation.coords.y,
                rotation.coords.z,
            ));
        rb.set_position(
            rapier3d::prelude::Isometry::from_parts(
                rapier3d::prelude::Translation::new(center_pos.x, center_pos.y, center_pos.z),
                rapier_rotation,
            ),
            true,
        );
    }
}

pub fn update_levers_momentum(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.physics.fixed_timestep;
    let friction = 2.5;

    let lever_entities: Vec<freecs::Entity> =
        game_world.query_entities(LEVER).collect();

    let manipulated_lever = game_world.resources.interaction.manipulated_entity_of_kind(&crate::ecs::InteractableKind::Lever);

    for game_entity in lever_entities {
        if manipulated_lever == Some(game_entity) {
            continue;
        }

        let Some(lever) = game_world.get_lever_mut(game_entity) else {
            continue;
        };

        if lever.angular_velocity.abs() < 0.01 {
            lever.angular_velocity = 0.0;
            continue;
        }

        lever.angular_velocity *= (-friction * dt).exp();

        let angle_delta = lever.angular_velocity * dt;
        let new_angle =
            (lever.current_angle + angle_delta).clamp(lever.min_angle, lever.max_angle);

        if (new_angle - lever.min_angle).abs() < 0.001
            || (new_angle - lever.max_angle).abs() < 0.001
        {
            lever.angular_velocity = -lever.angular_velocity * 0.3;
        }

        lever.current_angle = new_angle;

        apply_lever_transform(game_world, world, game_entity);
    }
}
