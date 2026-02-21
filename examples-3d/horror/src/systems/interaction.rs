use crate::constants::{
    ANGULAR_DAMPING, GRAB_DAMPING_RATIO, GRAB_RANGE, GRAB_STIFFNESS, INTERACT_CONE_RADIUS,
    INTERACT_RANGE, MAX_GRAB_DISTANCE, MAX_GRAB_FORCE, MIN_GRAB_DISTANCE, SCROLL_DISTANCE_SPEED,
    THROW_STRENGTH,
};
use crate::state::{HorrorDemo, InputMode, LeverAction};
use crate::systems::ui::pick_entities_cone;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::picking::{PickingOptions, PickingResult, pick_entities};
use nightshade::prelude::*;

pub fn interaction_system(demo: &mut HorrorDemo, world: &mut World) {
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

    let gamepad_rt_just_pressed = gamepad_rt_pressed && !demo.interaction.gamepad_rt_was_pressed;
    demo.interaction.gamepad_rt_was_pressed = gamepad_rt_pressed;

    let interact_pressed = left_clicked || f_pressed || gamepad_rt_pressed;
    let interact_just_pressed = left_just_pressed || gamepad_rt_just_pressed;

    if demo.interaction.require_interact_release {
        if !interact_pressed {
            demo.interaction.require_interact_release = false;
        }
        return;
    }

    let Some(camera_entity) = demo.camera_entity else {
        return;
    };
    let Some(camera_transform) = world.get_global_transform(camera_entity).cloned() else {
        return;
    };
    let camera_position = camera_transform.translation();
    let camera_forward = camera_transform.forward_vector();

    if !interact_pressed {
        if let Some(grabbed_entity) = demo.interaction.grabbed_entity {
            throw_grabbed_object(demo, world, camera_forward);
            let _ = grabbed_entity;
        }
        demo.interaction.grabbed_entity = None;

        if demo.interaction.manipulated_door_index.is_some()
            && let Some(door_audio_entity) = demo.door_audio_entity
            && let Some(source) = world.get_audio_source_mut(door_audio_entity)
        {
            source.playing = false;
        }
        demo.interaction.manipulated_door_index = None;
        demo.interaction.manipulated_lever_index = None;

        if let Some(button_index) = demo.interaction.manipulated_button_index {
            release_button(demo, world, button_index);
        }
        demo.interaction.manipulated_button_index = None;

        return;
    }

    if demo.interaction.grabbed_entity.is_some() {
        let scroll_delta = world.resources.input.mouse.wheel_delta.y;
        update_grabbed_object(demo, world, camera_position, camera_forward, scroll_delta);
        return;
    }

    if demo.interaction.manipulated_door_index.is_some() {
        crate::systems::doors::update_manipulated_door(demo, world, camera_position);
        return;
    }

    if demo.interaction.manipulated_lever_index.is_some() {
        crate::systems::levers::update_manipulated_lever(demo, world, camera_position);
        return;
    }

    if let Some(button_index) = demo.interaction.manipulated_button_index {
        update_pressed_button(demo, world, button_index);
        return;
    }

    if !interact_just_pressed {
        return;
    }

    let screen_pos = if demo.input_mode == InputMode::Gamepad {
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

    let pick_results = if demo.input_mode == InputMode::Gamepad {
        pick_entities_cone(demo, world, screen_pos, INTERACT_CONE_RADIUS, options)
    } else {
        pick_entities(world, screen_pos, options)
    };

    try_start_interaction(demo, world, &pick_results);
}

pub fn try_start_interaction(
    demo: &mut HorrorDemo,
    world: &mut World,
    pick_results: &[PickingResult],
) {
    for result in pick_results {
        if demo.physics_objects.contains(&result.entity) {
            demo.interaction.grabbed_entity = Some(result.entity);
            demo.interaction.grab_distance = result.distance.min(MAX_GRAB_DISTANCE);
            return;
        }

        for (index, door) in demo.doors.iter().enumerate() {
            if result.entity == door.entity && result.distance <= INTERACT_RANGE {
                if !door.locked {
                    demo.interaction.manipulated_door_index = Some(index);
                    if let Some(door_audio_entity) = demo.door_audio_entity {
                        world.resources.audio.stop_sound(door_audio_entity);
                        if let Some(source) = world.get_audio_source_mut(door_audio_entity) {
                            source.playing = true;
                        }
                    }
                }
                return;
            }
        }

        for (index, lever) in demo.levers.iter().enumerate() {
            if result.entity == lever.collider_entity && result.distance <= INTERACT_RANGE {
                if matches!(lever.action, LeverAction::UnlockExit) && !demo.power_restored {
                    demo.temporary_message =
                        Some("The lever won't budge. Find the power switch first.".to_string());
                    demo.temporary_message_timer = 3.0;
                    demo.interaction.require_interact_release = true;
                    return;
                }
                demo.interaction.manipulated_lever_index = Some(index);
                return;
            }
        }

        for (index, button) in demo.buttons.iter().enumerate() {
            if result.entity == button.entity && result.distance <= INTERACT_RANGE {
                demo.interaction.manipulated_button_index = Some(index);
                return;
            }
        }

        for (index, note) in demo.notes.iter().enumerate() {
            if result.entity == note.entity && result.distance <= INTERACT_RANGE {
                demo.reading_note = Some(index);
                demo.note_close_key_released = false;
                demo.interaction.require_interact_release = true;
                return;
            }
        }
    }
}

pub fn update_grabbed_object(
    demo: &mut HorrorDemo,
    world: &mut World,
    camera_position: Vec3,
    camera_forward: Vec3,
    scroll_delta: f32,
) {
    demo.interaction.grab_distance = (demo.interaction.grab_distance
        + scroll_delta * SCROLL_DISTANCE_SPEED)
        .clamp(MIN_GRAB_DISTANCE, MAX_GRAB_DISTANCE);

    let target_position = camera_position + camera_forward * demo.interaction.grab_distance;

    let Some(grabbed_entity) = demo.interaction.grabbed_entity else {
        return;
    };

    let Some(rigid_body_component) = world.get_rigid_body(grabbed_entity) else {
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

pub fn throw_grabbed_object(demo: &mut HorrorDemo, world: &mut World, camera_forward: Vec3) {
    let Some(grabbed_entity) = demo.interaction.grabbed_entity else {
        return;
    };

    let Some(rigid_body_component) = world.get_rigid_body(grabbed_entity) else {
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

    demo.interaction.grabbed_entity = None;
}

pub fn update_pressed_button(demo: &mut HorrorDemo, world: &mut World, button_index: usize) {
    let delta_time = world.resources.window.timing.delta_time;
    let press_speed = 8.0;
    let max_press = 0.03;

    let button = &mut demo.buttons[button_index];
    button.current_press = (button.current_press + press_speed * delta_time).min(max_press);

    let pressed_y = button.base_position.y - button.current_press;
    if let Some(transform) = world.get_local_transform_mut(button.entity) {
        transform.translation.y = pressed_y;
    }
    world.mark_local_transform_dirty(button.entity);

    if let Some(rb) = world.get_rigid_body_mut(button.entity)
        && let Some(handle) = rb.handle
    {
        let physics = &mut world.resources.physics;
        if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
            rigid_body.set_next_kinematic_translation(rapier3d::prelude::Vector::new(
                button.base_position.x,
                pressed_y,
                button.base_position.z,
            ));
        }
    }

    if button.current_press >= max_press && !button.is_pressed {
        button.is_pressed = true;
    }
}

pub fn release_button(demo: &mut HorrorDemo, world: &mut World, button_index: usize) {
    let button = &mut demo.buttons[button_index];
    button.current_press = 0.0;
    button.is_pressed = false;

    if let Some(transform) = world.get_local_transform_mut(button.entity) {
        transform.translation.y = button.base_position.y;
    }
    world.mark_local_transform_dirty(button.entity);

    if let Some(rb) = world.get_rigid_body_mut(button.entity)
        && let Some(handle) = rb.handle
    {
        let physics = &mut world.resources.physics;
        if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
            rigid_body.set_next_kinematic_translation(rapier3d::prelude::Vector::new(
                button.base_position.x,
                button.base_position.y,
                button.base_position.z,
            ));
        }
    }
}
