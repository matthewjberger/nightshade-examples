use crate::constants::{
    ANGULAR_DAMPING, GRAB_DAMPING_RATIO, GRAB_RANGE, GRAB_STIFFNESS, INTERACT_CONE_RADIUS,
    MAX_GRAB_DISTANCE, MAX_GRAB_FORCE, MIN_GRAB_DISTANCE, SCROLL_DISTANCE_SPEED, THROW_STRENGTH,
};
use crate::ecs::{GameWorld, INTERACTABLE, InputMode, InteractionKind, LeverAction};
use crate::systems::ui::pick_entities_cone;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::picking::{PickingOptions, PickingResult, pick_entities};
use nightshade::prelude::*;

pub fn interaction_system(game_world: &mut GameWorld, world: &mut World) {
    let mouse = &world.resources.input.mouse;
    let keyboard = &world.resources.input.keyboard;
    let mouse_pos = mouse.position;

    let left_clicked = mouse.state.contains(MouseState::LEFT_CLICKED);
    let left_just_pressed = mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let f_pressed = keyboard.is_key_pressed(KeyCode::KeyF);

    let gamepad_rt_pressed = if let Some(gamepad) = query_active_gamepad(world) {
        let rt_axis = gamepad.value(gilrs::Axis::RightZ);
        let rt_button = gamepad.is_pressed(gilrs::Button::RightTrigger2);
        rt_axis > 0.5 || rt_button
    } else {
        false
    };

    let gamepad_rt_just_pressed =
        gamepad_rt_pressed && !game_world.resources.interaction.gamepad_rt_was_pressed;
    game_world.resources.interaction.gamepad_rt_was_pressed = gamepad_rt_pressed;

    let interact_pressed = left_clicked || f_pressed || gamepad_rt_pressed;
    let interact_just_pressed = left_just_pressed || gamepad_rt_just_pressed;

    if game_world.resources.interaction.require_interact_release {
        if !interact_pressed {
            game_world.resources.interaction.require_interact_release = false;
        }
        return;
    }

    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };
    let Some(camera_transform) = world.core.get_global_transform(camera_entity).cloned() else {
        return;
    };
    let camera_position = camera_transform.translation();
    let camera_forward = camera_transform.forward_vector();

    if !interact_pressed {
        if game_world.resources.interaction.grabbed_entity.is_some() {
            throw_grabbed_object(game_world, world, camera_forward);
        }
        game_world.resources.interaction.grabbed_entity = None;

        if game_world.resources.interaction.manipulated_door.is_some()
            && let Some(door_audio_entity) = game_world.resources.door_audio_entity
            && let Some(source) = world.core.get_audio_source_mut(door_audio_entity)
        {
            source.playing = false;
        }
        game_world.resources.interaction.manipulated_door = None;
        game_world.resources.interaction.manipulated_lever = None;

        if let Some(button_game_entity) = game_world.resources.interaction.manipulated_button {
            release_button(game_world, world, button_game_entity);
        }
        game_world.resources.interaction.manipulated_button = None;

        return;
    }

    if game_world.resources.interaction.grabbed_entity.is_some() {
        let scroll_delta = world.resources.input.mouse.wheel_delta.y;
        update_grabbed_object(
            game_world,
            world,
            camera_position,
            camera_forward,
            scroll_delta,
        );
        return;
    }

    if game_world.resources.interaction.manipulated_door.is_some() {
        crate::systems::doors::update_manipulated_door(game_world, world, camera_position);
        return;
    }

    if game_world.resources.interaction.manipulated_lever.is_some() {
        crate::systems::levers::update_manipulated_lever(game_world, world, camera_position);
        return;
    }

    if let Some(button_game_entity) = game_world.resources.interaction.manipulated_button {
        update_pressed_button(game_world, world, button_game_entity);
        return;
    }

    if !interact_just_pressed {
        return;
    }

    let screen_pos = if game_world.resources.input_mode == InputMode::Gamepad {
        let viewport_size = world
            .resources
            .window
            .cached_viewport_size
            .unwrap_or((800, 600));
        nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0)
    } else {
        mouse_pos
    };

    let options = PickingOptions {
        max_distance: GRAB_RANGE,
        ignore_invisible: true,
    };

    let pick_results = if game_world.resources.input_mode == InputMode::Gamepad {
        pick_entities_cone(world, screen_pos, INTERACT_CONE_RADIUS, options)
    } else {
        pick_entities(world, screen_pos, options)
    };

    try_start_interaction(game_world, world, &pick_results);
}

fn find_interactable(
    game_world: &GameWorld,
    picked_entity: Entity,
    distance: f32,
) -> Option<(freecs::Entity, InteractionKind)> {
    for game_entity in game_world.query_entities(INTERACTABLE) {
        let Some(interactable) = game_world.get_interactable(game_entity) else {
            continue;
        };
        if interactable.match_entity == picked_entity
            && (interactable.range == 0.0 || distance <= interactable.range)
        {
            return Some((game_entity, interactable.kind));
        }
    }
    None
}

fn try_start_interaction(
    game_world: &mut GameWorld,
    world: &mut World,
    pick_results: &[PickingResult],
) {
    for result in pick_results {
        let Some((game_entity, kind)) =
            find_interactable(game_world, result.entity, result.distance)
        else {
            continue;
        };

        match kind {
            InteractionKind::Grab => {
                game_world.resources.interaction.grabbed_entity = Some(result.entity);
                game_world.resources.interaction.grab_distance =
                    result.distance.min(MAX_GRAB_DISTANCE);
            }
            InteractionKind::Door => {
                if let Some(door) = game_world.get_door(game_entity)
                    && !door.locked
                {
                    game_world.resources.interaction.manipulated_door = Some(game_entity);
                    if let Some(door_audio_entity) = game_world.resources.door_audio_entity {
                        world.resources.audio.stop_sound(door_audio_entity);
                        if let Some(source) = world.core.get_audio_source_mut(door_audio_entity) {
                            source.playing = true;
                        }
                    }
                }
            }
            InteractionKind::Lever => {
                if let Some(lever) = game_world.get_lever(game_entity)
                    && matches!(lever.action, LeverAction::UnlockExit)
                    && !game_world.resources.power_restored
                {
                    game_world.resources.temporary_message =
                        Some("The lever won't budge. Find the power switch first.".to_string());
                    game_world.resources.temporary_message_timer = 3.0;
                    game_world.resources.interaction.require_interact_release = true;
                    return;
                }
                game_world.resources.interaction.manipulated_lever = Some(game_entity);
            }
            InteractionKind::Button => {
                game_world.resources.interaction.manipulated_button = Some(game_entity);
            }
            InteractionKind::Note => {
                game_world.resources.reading_note = Some(game_entity);
                game_world.resources.note_close_key_released = false;
                game_world.resources.interaction.require_interact_release = true;
            }
        }
        return;
    }
}

fn update_grabbed_object(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
    camera_forward: Vec3,
    scroll_delta: f32,
) {
    game_world.resources.interaction.grab_distance =
        (game_world.resources.interaction.grab_distance + scroll_delta * SCROLL_DISTANCE_SPEED)
            .clamp(MIN_GRAB_DISTANCE, MAX_GRAB_DISTANCE);

    let target_position =
        camera_position + camera_forward * game_world.resources.interaction.grab_distance;

    let Some(grabbed_entity) = game_world.resources.interaction.grabbed_entity else {
        return;
    };

    let Some(rigid_body_component) = world.core.get_rigid_body(grabbed_entity) else {
        return;
    };
    let Some(handle) = rigid_body_component.handle else {
        return;
    };
    let Some(rigid_body) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(handle.into())
    else {
        return;
    };

    let current_pos = rigid_body.translation();
    let current_position = nalgebra_glm::vec3(current_pos.x, current_pos.y, current_pos.z);

    let displacement = target_position - current_position;

    let current_vel = rigid_body.linvel();
    let current_velocity = nalgebra_glm::vec3(current_vel.x, current_vel.y, current_vel.z);

    let mass = rigid_body.mass();
    let critical_damping = 2.0 * (GRAB_STIFFNESS * mass).sqrt();
    let damping = critical_damping * GRAB_DAMPING_RATIO;

    let spring_force = displacement * GRAB_STIFFNESS;
    let damping_force = -current_velocity * damping;
    let mut total_force = spring_force + damping_force;

    let force_magnitude = nalgebra_glm::length(&total_force);
    let max_force_for_mass = MAX_GRAB_FORCE * mass.max(0.5);
    if force_magnitude > max_force_for_mass {
        total_force *= max_force_for_mass / force_magnitude;
    }

    let acceleration = total_force / mass;
    let dt = world.resources.physics.fixed_timestep;
    let new_velocity = current_velocity + acceleration * dt;

    rigid_body.set_linvel(
        rapier3d::math::Vector::new(new_velocity.x, new_velocity.y, new_velocity.z),
        true,
    );

    let current_angvel = rigid_body.angvel();
    let angular_decay = (-ANGULAR_DAMPING * dt * 60.0).exp();
    rigid_body.set_angvel(current_angvel * angular_decay, true);
}

fn throw_grabbed_object(game_world: &mut GameWorld, world: &mut World, camera_forward: Vec3) {
    let Some(grabbed_entity) = game_world.resources.interaction.grabbed_entity else {
        return;
    };

    let Some(rigid_body_component) = world.core.get_rigid_body(grabbed_entity) else {
        return;
    };
    let Some(handle) = rigid_body_component.handle else {
        return;
    };
    let Some(rigid_body) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(handle.into())
    else {
        return;
    };

    let throw_velocity = camera_forward * THROW_STRENGTH;
    rigid_body.set_linvel(
        rapier3d::math::Vector::new(throw_velocity.x, throw_velocity.y, throw_velocity.z),
        true,
    );

    game_world.resources.interaction.grabbed_entity = None;
}

fn update_pressed_button(
    game_world: &mut GameWorld,
    world: &mut World,
    button_game_entity: freecs::Entity,
) {
    let Some(engine_entity) = game_world.get_engine_entity(button_game_entity) else {
        return;
    };
    let entity = engine_entity.0;
    let delta_time = world.resources.window.timing.delta_time;
    let press_speed = 8.0;
    let max_press = 0.03;

    let Some(button) = game_world.get_button_mut(button_game_entity) else {
        return;
    };
    button.current_press = (button.current_press + press_speed * delta_time).min(max_press);

    let pressed_y = button.base_position.y - button.current_press;
    let current_press = button.current_press;
    let is_pressed = button.is_pressed;
    let base_position = button.base_position;

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation.y = pressed_y;
    }
    world.mark_local_transform_dirty(entity);

    if let Some(rb) = world.core.get_rigid_body_mut(entity)
        && let Some(handle) = rb.handle
    {
        let physics = &mut world.resources.physics;
        if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
            rigid_body.set_next_kinematic_translation(rapier3d::prelude::Vector::new(
                base_position.x,
                pressed_y,
                base_position.z,
            ));
        }
    }

    if current_press >= max_press
        && !is_pressed
        && let Some(button) = game_world.get_button_mut(button_game_entity)
    {
        button.is_pressed = true;
    }
}

fn release_button(
    game_world: &mut GameWorld,
    world: &mut World,
    button_game_entity: freecs::Entity,
) {
    let Some(engine_entity) = game_world.get_engine_entity(button_game_entity) else {
        return;
    };
    let entity = engine_entity.0;

    let Some(button) = game_world.get_button_mut(button_game_entity) else {
        return;
    };
    button.current_press = 0.0;
    button.is_pressed = false;
    let base_position = button.base_position;

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation.y = base_position.y;
    }
    world.mark_local_transform_dirty(entity);

    if let Some(rb) = world.core.get_rigid_body_mut(entity)
        && let Some(handle) = rb.handle
    {
        let physics = &mut world.resources.physics;
        if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
            rigid_body.set_next_kinematic_translation(rapier3d::prelude::Vector::new(
                base_position.x,
                base_position.y,
                base_position.z,
            ));
        }
    }
}
