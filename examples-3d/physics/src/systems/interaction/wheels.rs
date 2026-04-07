use crate::ecs::{GameWorld, WHEEL};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

pub(super) fn update_manipulated_wheel(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
) {
    let Some(wheel_game_entity) = game_world.resources.interaction.manipulated_entity_of_kind(&crate::ecs::InteractableKind::Wheel) else {
        return;
    };

    let Some(center_position) = game_world
        .get_wheel(wheel_game_entity)
        .map(|wheel| wheel.center_position)
    else {
        return;
    };

    if nalgebra_glm::distance(&camera_position, &center_position) > game_world.resources.config.interact_range * 3.0 {
        game_world.resources.interaction.manipulated = None;
        return;
    }

    let dt = world.resources.physics.fixed_timestep;

    let mouse_input = if world.resources.input.input_mode == InputMode::MouseKeyboard {
        -world.resources.input.mouse.raw_mouse_delta.x * 2.0
    } else {
        0.0
    };

    let gamepad_input = if world.resources.input.input_mode == InputMode::Gamepad {
        if let Some(gamepad) = query_active_gamepad(world) {
            let right_stick_x = gamepad.value(gilrs::Axis::RightStickX);
            let deadzone = 0.15;
            if right_stick_x.abs() > deadzone {
                -right_stick_x * 3.0
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
    let friction = 3.0;

    let Some(wheel) = game_world.get_wheel_mut(wheel_game_entity) else {
        return;
    };

    wheel.angular_velocity += torque * dt;
    wheel.angular_velocity -= wheel.angular_velocity * friction * dt;

    wheel.current_angle += wheel.angular_velocity * dt;

    apply_wheel_transform(game_world, world, wheel_game_entity);
}

fn apply_wheel_transform(
    game_world: &GameWorld,
    world: &mut World,
    wheel_game_entity: freecs::Entity,
) {
    let Some(wheel) = game_world.get_wheel(wheel_game_entity) else {
        return;
    };

    let base_rotation = nalgebra_glm::quat_angle_axis(
        std::f32::consts::FRAC_PI_2,
        &nalgebra_glm::vec3(1.0, 0.0, 0.0),
    );
    let spin_rotation =
        nalgebra_glm::quat_angle_axis(wheel.current_angle, &nalgebra_glm::vec3(0.0, 0.0, 1.0));

    if let Some(transform) = world.core.get_local_transform_mut(wheel.entity) {
        transform.rotation = spin_rotation * base_rotation;
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, wheel.entity);

    for spoke_entity in &wheel.spoke_entities {
        nightshade::ecs::transform::commands::mark_local_transform_dirty(world, *spoke_entity);
    }

    if let Some(rb) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(wheel.rigid_body_handle)
    {
        let base_rot = rapier3d::na::UnitQuaternion::from_axis_angle(
            &rapier3d::na::Vector3::x_axis(),
            std::f32::consts::FRAC_PI_2,
        );
        let spin_rot = rapier3d::na::UnitQuaternion::from_axis_angle(
            &rapier3d::na::Vector3::z_axis(),
            wheel.current_angle,
        );
        rb.set_rotation(spin_rot * base_rot, true);
    }
}

pub fn update_wheels_momentum(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.physics.fixed_timestep;
    let friction = 1.5;

    let wheel_entities: Vec<freecs::Entity> =
        game_world.query_entities(WHEEL).collect();

    let manipulated_wheel = game_world.resources.interaction.manipulated_entity_of_kind(&crate::ecs::InteractableKind::Wheel);

    for game_entity in wheel_entities {
        if manipulated_wheel == Some(game_entity) {
            continue;
        }

        let Some(wheel) = game_world.get_wheel_mut(game_entity) else {
            continue;
        };

        if wheel.angular_velocity.abs() < 0.01 {
            wheel.angular_velocity = 0.0;
            continue;
        }

        wheel.angular_velocity *= (-friction * dt).exp();
        wheel.current_angle += wheel.angular_velocity * dt;

        apply_wheel_transform(game_world, world, game_entity);
    }
}
