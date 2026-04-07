use crate::ecs::{DRAWER, GameWorld};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

pub(super) fn update_manipulated_drawer(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
) {
    let Some(drawer_game_entity) = game_world.resources.interaction.manipulated_entity_of_kind(&crate::ecs::InteractableKind::Drawer) else {
        return;
    };

    let Some(drawer_info) = game_world.get_drawer(drawer_game_entity).map(|drawer| {
        (
            drawer.closed_position,
            drawer.current_offset,
        )
    }) else {
        return;
    };

    let current_pos = nalgebra_glm::vec3(
        drawer_info.0.x,
        drawer_info.0.y,
        drawer_info.0.z + drawer_info.1,
    );
    let distance_to_drawer = nalgebra_glm::distance(&camera_position, &current_pos);

    if distance_to_drawer > game_world.resources.config.interact_range * 3.0 {
        game_world.resources.interaction.manipulated = None;
        return;
    }

    let dt = world.resources.physics.fixed_timestep;

    let mouse_input = if world.resources.input.input_mode == InputMode::MouseKeyboard {
        world.resources.input.mouse.raw_mouse_delta.y * 1.2
    } else {
        0.0
    };

    let gamepad_input = if world.resources.input.input_mode == InputMode::Gamepad {
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

    let pull_force = mouse_input + gamepad_input;
    let friction = 8.0;

    let Some(drawer) = game_world.get_drawer_mut(drawer_game_entity) else {
        return;
    };

    drawer.velocity += pull_force * dt;
    drawer.velocity -= drawer.velocity * friction * dt;

    let offset_delta = drawer.velocity * dt;
    let new_offset = (drawer.current_offset + offset_delta).clamp(0.0, drawer.max_offset);

    if new_offset <= 0.001 && drawer.velocity < 0.0 {
        drawer.velocity = -drawer.velocity * 0.3;
    }
    if (new_offset - drawer.max_offset).abs() < 0.001 && drawer.velocity > 0.0 {
        drawer.velocity = -drawer.velocity * 0.3;
    }

    drawer.current_offset = new_offset;

    apply_drawer_transform(game_world, world, drawer_game_entity);
}

fn apply_drawer_transform(
    game_world: &GameWorld,
    world: &mut World,
    drawer_game_entity: freecs::Entity,
) {
    let Some(drawer) = game_world.get_drawer(drawer_game_entity) else {
        return;
    };

    let new_z = drawer.closed_position.z + drawer.current_offset;

    if let Some(transform) = world.core.get_local_transform_mut(drawer.entity) {
        transform.translation.z = new_z;
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, drawer.entity);

    if let Some(rb) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(drawer.rigid_body_handle)
    {
        rb.set_translation(
            rapier3d::math::Vector::new(drawer.closed_position.x, drawer.closed_position.y, new_z),
            true,
        );
    }
}

pub fn update_drawers_momentum(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.physics.fixed_timestep;
    let friction = 3.0;

    let drawer_entities: Vec<freecs::Entity> =
        game_world.query_entities(DRAWER).collect();

    let manipulated_drawer = game_world.resources.interaction.manipulated_entity_of_kind(&crate::ecs::InteractableKind::Drawer);

    for game_entity in drawer_entities {
        if manipulated_drawer == Some(game_entity) {
            continue;
        }

        let Some(drawer) = game_world.get_drawer_mut(game_entity) else {
            continue;
        };

        if drawer.velocity.abs() < 0.01 {
            drawer.velocity = 0.0;
            continue;
        }

        drawer.velocity *= (-friction * dt).exp();

        let offset_delta = drawer.velocity * dt;
        let new_offset = (drawer.current_offset + offset_delta).clamp(0.0, drawer.max_offset);

        if new_offset <= 0.001 || (new_offset - drawer.max_offset).abs() < 0.001 {
            drawer.velocity = -drawer.velocity * 0.2;
        }

        drawer.current_offset = new_offset;

        apply_drawer_transform(game_world, world, game_entity);
    }
}
