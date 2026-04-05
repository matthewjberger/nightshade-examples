use crate::constants::{
    ANGULAR_DAMPING, GRAB_DAMPING_RATIO, GRAB_RANGE, GRAB_STIFFNESS, INTERACT_CONE_RADIUS,
    INTERACT_RANGE, MAX_GRAB_DISTANCE, MAX_GRAB_FORCE, MIN_GRAB_DISTANCE, SCROLL_DISTANCE_SPEED,
    THROW_STRENGTH,
};
use crate::ecs::{
    BAUBLE_SPAWN, BUTTON, DOOR, DRAWER, ButtonAction, GameWorld, InputMode, LEVER,
    NOTE, WHEEL,
};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::picking::{PickingOptions, PickingResult, pick_entities};
use nightshade::prelude::*;

pub fn interaction_system(game_world: &mut GameWorld, world: &mut World) {
    let (left_clicked, _left_just_pressed, right_clicked, scroll_delta) =
        if game_world.resources.input_mode == InputMode::MouseKeyboard {
            let mouse = &world.resources.input.mouse;
            (
                mouse.state.contains(MouseState::LEFT_CLICKED),
                mouse.state.contains(MouseState::LEFT_JUST_PRESSED),
                mouse.state.contains(MouseState::RIGHT_CLICKED),
                mouse.wheel_delta.y,
            )
        } else {
            (false, false, false, 0.0)
        };

    let (gamepad_rt_held, gamepad_lt_held, _gamepad_rt_just_pressed, gamepad_dpad_distance) =
        if game_world.resources.input_mode == InputMode::Gamepad {
            if let Some(gamepad) = query_active_gamepad(world) {
                let rt_axis_value = gamepad.value(gilrs::Axis::RightZ);
                let lt_axis_value = gamepad.value(gilrs::Axis::LeftZ);
                let rt_button = gamepad.is_pressed(gilrs::Button::RightTrigger2);
                let lt_button = gamepad.is_pressed(gilrs::Button::LeftTrigger2);
                let rt_held = rt_axis_value > 0.5 || rt_button;
                let lt_held = lt_axis_value > 0.5 || lt_button;
                let rt_just_pressed =
                    rt_held && !game_world.resources.interaction.gamepad_rt_was_pressed;
                let dpad_up = gamepad.is_pressed(gilrs::Button::DPadUp);
                let dpad_down = gamepad.is_pressed(gilrs::Button::DPadDown);
                let dpad_distance: f32 = if dpad_up {
                    1.0
                } else if dpad_down {
                    -1.0
                } else {
                    0.0
                };
                (rt_held, lt_held, rt_just_pressed, dpad_distance)
            } else {
                (false, false, false, 0.0)
            }
        } else {
            (false, false, false, 0.0)
        };

    game_world.resources.interaction.gamepad_rt_was_pressed =
        if game_world.resources.input_mode == InputMode::Gamepad {
            if let Some(gamepad) = query_active_gamepad(world) {
                gamepad.value(gilrs::Axis::RightZ) > 0.5
                    || gamepad.is_pressed(gilrs::Button::RightTrigger2)
            } else {
                false
            }
        } else {
            false
        };

    #[cfg(feature = "openxr")]
    let (xr_grip_held, xr_rt_held, xr_lt_held, xr_thumbstick_y) = {
        if let Some(xr_input) = &world.resources.xr.input {
            let grip_held = xr_input.right_grip_pressed();
            let rt_held = xr_input.right_trigger_pressed();
            let lt_held = xr_input.left_trigger_pressed();
            let thumbstick_y = xr_input.right_thumbstick.y;
            (grip_held, rt_held, lt_held, thumbstick_y)
        } else {
            (false, false, false, 0.0)
        }
    };

    #[cfg(not(feature = "openxr"))]
    let (xr_grip_held, xr_rt_held, xr_lt_held, xr_thumbstick_y) = (false, false, false, 0.0_f32);
    let _ = xr_lt_held;

    let interact_held = left_clicked || gamepad_lt_held || xr_grip_held;
    let throw_pressed = right_clicked || gamepad_rt_held || xr_lt_held;

    let keyboard_shoot_pressed =
        if game_world.resources.input_mode == InputMode::MouseKeyboard {
            let keyboard = &world.resources.input.keyboard;
            keyboard.is_key_pressed(KeyCode::Enter)
        } else {
            false
        };
    let shoot_pressed = keyboard_shoot_pressed || gamepad_rt_held || xr_rt_held;

    let delta_time = world.resources.window.timing.delta_time;
    #[cfg(feature = "openxr")]
    let xr_distance_delta = if xr_thumbstick_y.abs() > 0.1 {
        xr_thumbstick_y * delta_time * 3.0
    } else {
        0.0
    };
    #[cfg(not(feature = "openxr"))]
    let xr_distance_delta = 0.0_f32;
    let _ = xr_thumbstick_y;

    let effective_scroll_delta = if game_world.resources.input_mode == InputMode::Gamepad
        && gamepad_dpad_distance.abs() > 0.0
    {
        gamepad_dpad_distance * delta_time * 3.0
    } else if xr_distance_delta.abs() > 0.0 {
        xr_distance_delta
    } else {
        scroll_delta
    };

    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };
    let Some(camera_transform) = world.core.get_global_transform(camera_entity) else {
        return;
    };

    let camera_position = camera_transform.translation();
    let camera_forward = camera_transform.forward_vector();

    #[cfg(feature = "openxr")]
    let (shoot_origin, shoot_direction) = {
        if let Some(xr_input) = &world.resources.xr.input {
            if let (Some(hand_pos), Some(hand_rot)) = (
                xr_input.right_hand_position(),
                xr_input.right_hand_rotation(),
            ) {
                let forward = nalgebra_glm::quat_rotate_vec3(
                    &hand_rot,
                    &nalgebra_glm::vec3(0.0, 1.0, 0.0),
                );
                (hand_pos, forward)
            } else {
                (camera_position, camera_forward)
            }
        } else {
            (camera_position, camera_forward)
        }
    };
    #[cfg(not(feature = "openxr"))]
    let (shoot_origin, shoot_direction) =
        if let Some(weapon) = game_world.resources.weapon_entity
            && let Some(weapon_transform) = world.core.get_global_transform(weapon)
        {
            let muzzle_local = nalgebra_glm::vec4(0.0, 0.005, -0.20, 1.0);
            let muzzle_world = weapon_transform.0 * muzzle_local;
            (muzzle_world.xyz(), camera_forward)
        } else {
            (camera_position, camera_forward)
        };

    let current_time_ms = world.resources.window.timing.uptime_milliseconds;
    let shoot_just_pressed = shoot_pressed && !game_world.resources.interaction.shoot_was_pressed;
    game_world.resources.interaction.shoot_was_pressed = shoot_pressed;

    if game_world.resources.interaction.grabbed_entity.is_none() {
        if shoot_just_pressed {
            game_world.resources.interaction.shoot_hold_start_ms = Some(current_time_ms);
            game_world.resources.interaction.last_rapid_fire_ms = current_time_ms;
            super::shooting::shoot_bauble(game_world, world, shoot_origin, shoot_direction);
        } else if shoot_pressed {
            if let Some(hold_start) = game_world.resources.interaction.shoot_hold_start_ms {
                let hold_duration = current_time_ms.saturating_sub(hold_start);
                if hold_duration > 200 {
                    let time_since_last_shot = current_time_ms
                        .saturating_sub(game_world.resources.interaction.last_rapid_fire_ms);
                    if time_since_last_shot >= 80 {
                        game_world.resources.interaction.last_rapid_fire_ms = current_time_ms;
                        super::shooting::shoot_bauble(
                            game_world,
                            world,
                            shoot_origin,
                            shoot_direction,
                        );
                    }
                }
            }
        } else {
            game_world.resources.interaction.shoot_hold_start_ms = None;
        }
    }

    if !interact_held {
        if let Some(button_index) = game_world.resources.interaction.manipulated_button {
            release_button(game_world, world, button_index);
        }
        game_world.resources.interaction.grabbed_entity = None;
        game_world.resources.interaction.manipulated_door = None;
        game_world.resources.interaction.manipulated_drawer = None;
        game_world.resources.interaction.manipulated_lever = None;
        game_world.resources.interaction.manipulated_wheel = None;
        game_world.resources.interaction.manipulated_button = None;
        game_world.resources.interaction.require_interact_release = false;
        return;
    }

    if game_world.resources.interaction.require_interact_release {
        return;
    }

    if game_world.resources.interaction.grabbed_entity.is_some() {
        update_grabbed_object(
            game_world,
            world,
            camera_position,
            camera_forward,
            effective_scroll_delta,
        );

        if throw_pressed {
            throw_grabbed_object(game_world, world, camera_forward);
            game_world.resources.interaction.require_interact_release = true;
        }
        return;
    }

    if game_world.resources.interaction.manipulated_door.is_some() {
        update_manipulated_door(game_world, world, camera_position);
        return;
    }

    if game_world.resources.interaction.manipulated_drawer.is_some() {
        update_manipulated_drawer(game_world, world, camera_position);
        return;
    }

    if game_world.resources.interaction.manipulated_lever.is_some() {
        update_manipulated_lever(game_world, world, camera_position);
        return;
    }

    if game_world.resources.interaction.manipulated_wheel.is_some() {
        update_manipulated_wheel(game_world, world, camera_position);
        return;
    }

    if let Some(button_index) = game_world.resources.interaction.manipulated_button {
        update_pressed_button(game_world, world, button_index);
        return;
    }

    let viewport_size = world
        .resources
        .window
        .cached_viewport_size
        .unwrap_or((800, 600));
    let screen_pos =
        nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0);

    let options = PickingOptions {
        max_distance: GRAB_RANGE,
        ignore_invisible: true,
    };

    let pick_results = if game_world.resources.input_mode == InputMode::Gamepad {
        pick_entities_cone(world, screen_pos, INTERACT_CONE_RADIUS, options)
    } else {
        pick_entities(world, screen_pos, options)
    };

    try_start_interaction(game_world, &pick_results);
}

fn try_start_interaction(game_world: &mut GameWorld, pick_results: &[PickingResult]) {
    let door_entities: Vec<freecs::Entity> =
        game_world.query_entities(DOOR).collect();
    let drawer_entities: Vec<freecs::Entity> =
        game_world.query_entities(DRAWER).collect();
    let lever_entities: Vec<freecs::Entity> =
        game_world.query_entities(LEVER).collect();
    let wheel_entities: Vec<freecs::Entity> =
        game_world.query_entities(WHEEL).collect();
    let button_entities: Vec<freecs::Entity> =
        game_world.query_entities(BUTTON).collect();
    let note_entities: Vec<freecs::Entity> =
        game_world.query_entities(NOTE).collect();

    for result in pick_results {
        if game_world.resources.physics_objects.contains(&result.entity) {
            game_world.resources.interaction.grabbed_entity = Some(result.entity);
            game_world.resources.interaction.grab_distance =
                result.distance.min(MAX_GRAB_DISTANCE);
            return;
        }

        for &game_entity in &door_entities {
            if let Some(door) = game_world.get_door(game_entity)
                && result.entity == door.entity
                && result.distance <= INTERACT_RANGE
            {
                game_world.resources.interaction.manipulated_door = Some(game_entity);
                return;
            }
        }

        for &game_entity in &drawer_entities {
            if let Some(drawer) = game_world.get_drawer(game_entity)
                && result.entity == drawer.front_entity
                && result.distance <= INTERACT_RANGE
            {
                game_world.resources.interaction.manipulated_drawer = Some(game_entity);
                return;
            }
        }

        for &game_entity in &lever_entities {
            if let Some(lever) = game_world.get_lever(game_entity)
                && result.entity == lever.collider_entity
                && result.distance <= INTERACT_RANGE
            {
                game_world.resources.interaction.manipulated_lever = Some(game_entity);
                return;
            }
        }

        for &game_entity in &wheel_entities {
            if let Some(wheel) = game_world.get_wheel(game_entity)
                && result.entity == wheel.entity
                && result.distance <= INTERACT_RANGE
            {
                game_world.resources.interaction.manipulated_wheel = Some(game_entity);
                return;
            }
        }

        for &game_entity in &button_entities {
            if let Some(button) = game_world.get_button(game_entity)
                && result.entity == button.entity
                && result.distance <= INTERACT_RANGE
            {
                game_world.resources.interaction.manipulated_button = Some(game_entity);
                return;
            }
        }

        for &game_entity in &note_entities {
            if let Some(note) = game_world.get_note(game_entity)
                && result.entity == note.entity
                && result.distance <= INTERACT_RANGE
            {
                game_world.resources.reading_note = Some(game_entity);
                game_world.resources.note_close_key_released = false;
                game_world.resources.interaction.require_interact_release = true;
                return;
            }
        }
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

pub fn update_manipulated_door(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
) {
    let Some(door_game_entity) = game_world.resources.interaction.manipulated_door else {
        return;
    };

    let Some(hinge_position) = game_world
        .get_door(door_game_entity)
        .map(|door| door.hinge_position)
    else {
        return;
    };

    if nalgebra_glm::distance(&camera_position, &hinge_position) > INTERACT_RANGE * 3.0 {
        game_world.resources.interaction.manipulated_door = None;
        return;
    }

    let dt = world.resources.physics.fixed_timestep;

    let mouse_input = if game_world.resources.input_mode == InputMode::MouseKeyboard {
        -world.resources.input.mouse.raw_mouse_delta.x * 0.8
    } else {
        0.0
    };

    let gamepad_input = if game_world.resources.input_mode == InputMode::Gamepad {
        if let Some(gamepad) = query_active_gamepad(world) {
            let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
            let deadzone = 0.15;
            if right_stick_y.abs() > deadzone {
                right_stick_y * 3.0
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
    let friction = 6.0;

    let Some(door) = game_world.get_door_mut(door_game_entity) else {
        return;
    };
    door.angular_velocity += torque * dt;
    door.angular_velocity -= door.angular_velocity * friction * dt;

    let angle_delta = door.angular_velocity * dt;
    let new_angle = (door.current_angle + angle_delta).clamp(door.min_angle, door.max_angle);

    if (new_angle - door.min_angle).abs() < 0.001 && door.angular_velocity < 0.0 {
        door.angular_velocity = -door.angular_velocity * 0.2;
    }
    if (new_angle - door.max_angle).abs() < 0.001 && door.angular_velocity > 0.0 {
        door.angular_velocity = -door.angular_velocity * 0.2;
    }

    door.current_angle = new_angle;

    apply_door_transform(game_world, world, door_game_entity);
}

pub fn apply_door_transform(
    game_world: &GameWorld,
    world: &mut World,
    door_game_entity: freecs::Entity,
) {
    let Some(door) = game_world.get_door(door_game_entity) else {
        return;
    };

    let cos_angle = door.current_angle.cos();
    let sin_angle = door.current_angle.sin();
    let new_center_x = door.hinge_position.x + door.door_half_width * cos_angle;
    let new_center_z = door.hinge_position.z - door.door_half_width * sin_angle;

    if let Some(transform) = world.core.get_local_transform_mut(door.entity) {
        transform.translation.x = new_center_x;
        transform.translation.z = new_center_z;
        transform.rotation = nalgebra_glm::quat_angle_axis(
            door.current_angle,
            &nalgebra_glm::vec3(0.0, 1.0, 0.0),
        );
    }
    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, door.entity);

    if let Some(rb) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(door.rigid_body_handle)
    {
        let rotation = rapier3d::na::UnitQuaternion::from_axis_angle(
            &rapier3d::na::Vector3::y_axis(),
            door.current_angle,
        );
        rb.set_translation(
            rapier3d::math::Vector::new(new_center_x, door.hinge_position.y, new_center_z),
            true,
        );
        rb.set_rotation(rotation, true);
    }
}

pub fn update_doors_momentum(game_world: &mut GameWorld, world: &mut World) {
    let dt = world.resources.physics.fixed_timestep;
    let friction = 2.0;

    let door_entities: Vec<freecs::Entity> =
        game_world.query_entities(DOOR).collect();

    let manipulated_door = game_world.resources.interaction.manipulated_door;

    for game_entity in door_entities {
        if manipulated_door == Some(game_entity) {
            continue;
        }

        let Some(door) = game_world.get_door_mut(game_entity) else {
            continue;
        };

        if door.angular_velocity.abs() < 0.01 {
            door.angular_velocity = 0.0;
            continue;
        }

        door.angular_velocity *= (-friction * dt).exp();

        let angle_delta = door.angular_velocity * dt;
        let new_angle = (door.current_angle + angle_delta).clamp(door.min_angle, door.max_angle);

        if (new_angle - door.min_angle).abs() < 0.001
            || (new_angle - door.max_angle).abs() < 0.001
        {
            door.angular_velocity = -door.angular_velocity * 0.3;
        }

        door.current_angle = new_angle;

        apply_door_transform(game_world, world, game_entity);
    }
}

fn update_manipulated_drawer(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
) {
    let Some(drawer_game_entity) = game_world.resources.interaction.manipulated_drawer else {
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

    if distance_to_drawer > INTERACT_RANGE * 3.0 {
        game_world.resources.interaction.manipulated_drawer = None;
        return;
    }

    let dt = world.resources.physics.fixed_timestep;

    let mouse_input = if game_world.resources.input_mode == InputMode::MouseKeyboard {
        world.resources.input.mouse.raw_mouse_delta.y * 1.2
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

    let manipulated_drawer = game_world.resources.interaction.manipulated_drawer;

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

fn update_manipulated_lever(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
) {
    let Some(lever_game_entity) = game_world.resources.interaction.manipulated_lever else {
        return;
    };

    let Some(pivot_position) = game_world
        .get_lever(lever_game_entity)
        .map(|lever| lever.pivot_position)
    else {
        return;
    };

    if nalgebra_glm::distance(&camera_position, &pivot_position) > INTERACT_RANGE * 3.0 {
        game_world.resources.interaction.manipulated_lever = None;
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

    let manipulated_lever = game_world.resources.interaction.manipulated_lever;

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

fn update_manipulated_wheel(
    game_world: &mut GameWorld,
    world: &mut World,
    camera_position: Vec3,
) {
    let Some(wheel_game_entity) = game_world.resources.interaction.manipulated_wheel else {
        return;
    };

    let Some(center_position) = game_world
        .get_wheel(wheel_game_entity)
        .map(|wheel| wheel.center_position)
    else {
        return;
    };

    if nalgebra_glm::distance(&camera_position, &center_position) > INTERACT_RANGE * 3.0 {
        game_world.resources.interaction.manipulated_wheel = None;
        return;
    }

    let dt = world.resources.physics.fixed_timestep;

    let mouse_input = if game_world.resources.input_mode == InputMode::MouseKeyboard {
        -world.resources.input.mouse.raw_mouse_delta.x * 2.0
    } else {
        0.0
    };

    let gamepad_input = if game_world.resources.input_mode == InputMode::Gamepad {
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

    let manipulated_wheel = game_world.resources.interaction.manipulated_wheel;

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

pub fn update_lantern_light(game_world: &GameWorld, world: &mut World) {
    let Some(lantern_entity) = game_world.resources.lantern_entity else {
        return;
    };
    let Some(light_entity) = game_world.resources.lantern_light_entity else {
        return;
    };

    let lantern_position =
        if let Some(global_transform) = world.core.get_global_transform(lantern_entity) {
            global_transform.translation()
        } else {
            return;
        };

    if let Some(transform) = world.core.get_local_transform_mut(light_entity) {
        transform.translation = lantern_position;
    }
    world.mark_local_transform_dirty(light_entity);
}

fn update_pressed_button(
    game_world: &mut GameWorld,
    world: &mut World,
    button_game_entity: freecs::Entity,
) {
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
    let entity = button.entity;
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
        let action = button.action.clone();
        match action {
            ButtonAction::RecallBaubles => recall_baubles(game_world, world),
        }
    }
}

fn release_button(
    game_world: &mut GameWorld,
    world: &mut World,
    button_game_entity: freecs::Entity,
) {
    let Some(button) = game_world.get_button_mut(button_game_entity) else {
        return;
    };
    button.current_press = 0.0;
    button.is_pressed = false;
    let entity = button.entity;
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

fn recall_baubles(game_world: &mut GameWorld, world: &mut World) {
    let bauble_entities: Vec<freecs::Entity> = game_world
        .query_entities(BAUBLE_SPAWN)
        .collect();

    for game_entity in bauble_entities {
        let Some(bauble) = game_world.get_bauble_spawn(game_entity) else {
            continue;
        };
        let entity = bauble.entity;
        let spawn_position = bauble.spawn_position;

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = spawn_position;
        }
        world.mark_local_transform_dirty(entity);

        if let Some(rb) = world.core.get_rigid_body_mut(entity)
            && let Some(handle) = rb.handle
        {
            let physics = &mut world.resources.physics;
            if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
                rigid_body.set_translation(
                    rapier3d::prelude::Vector::new(
                        spawn_position.x,
                        spawn_position.y,
                        spawn_position.z,
                    ),
                    true,
                );
                rigid_body.set_linvel(rapier3d::prelude::Vector::zeros(), true);
                rigid_body.set_angvel(rapier3d::prelude::Vector::zeros(), true);
            }
        }
    }
}

pub fn update_interaction_prompt(game_world: &GameWorld, world: &mut World) {
    let Some(text_index) = game_world.resources.interaction_prompt_text_index else {
        return;
    };
    let Some(prompt_entity) = game_world.resources.interaction_prompt_entity else {
        return;
    };

    let viewport_size = world
        .resources
        .window
        .cached_viewport_size
        .unwrap_or((800, 600));

    if game_world.resources.interaction.grabbed_entity.is_some()
        || game_world
            .resources
            .interaction
            .manipulated_door
            .is_some()
        || game_world
            .resources
            .interaction
            .manipulated_drawer
            .is_some()
        || game_world
            .resources
            .interaction
            .manipulated_lever
            .is_some()
        || game_world
            .resources
            .interaction
            .manipulated_wheel
            .is_some()
        || game_world
            .resources
            .interaction
            .manipulated_button
            .is_some()
        || game_world.resources.reading_note.is_some()
    {
        world.resources.text_cache.set_text(text_index, "");
        if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
            hud_text.dirty = true;
        }
        return;
    }

    let screen_pos =
        nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0);

    let options = PickingOptions {
        max_distance: GRAB_RANGE,
        ignore_invisible: true,
    };

    let pick_results = if game_world.resources.input_mode == InputMode::Gamepad {
        pick_entities_cone(world, screen_pos, INTERACT_CONE_RADIUS, options)
    } else {
        pick_entities(world, screen_pos, options)
    };

    let door_entities: Vec<freecs::Entity> =
        game_world.query_entities(DOOR).collect();
    let drawer_entities: Vec<freecs::Entity> =
        game_world.query_entities(DRAWER).collect();
    let lever_entities: Vec<freecs::Entity> =
        game_world.query_entities(LEVER).collect();
    let wheel_entities: Vec<freecs::Entity> =
        game_world.query_entities(WHEEL).collect();
    let button_entities: Vec<freecs::Entity> =
        game_world.query_entities(BUTTON).collect();
    let note_entities: Vec<freecs::Entity> =
        game_world.query_entities(NOTE).collect();

    let mut can_interact = false;
    let mut can_read = false;

    'outer: for result in &pick_results {
        if game_world.resources.physics_objects.contains(&result.entity) {
            can_interact = true;
            break;
        }

        for &game_entity in &door_entities {
            if let Some(door) = game_world.get_door(game_entity)
                && result.entity == door.entity
                && result.distance <= INTERACT_RANGE
            {
                can_interact = true;
                break 'outer;
            }
        }

        for &game_entity in &drawer_entities {
            if let Some(drawer) = game_world.get_drawer(game_entity)
                && result.entity == drawer.front_entity
                && result.distance <= INTERACT_RANGE
            {
                can_interact = true;
                break 'outer;
            }
        }

        for &game_entity in &lever_entities {
            if let Some(lever) = game_world.get_lever(game_entity)
                && result.entity == lever.collider_entity
                && result.distance <= INTERACT_RANGE
            {
                can_interact = true;
                break 'outer;
            }
        }

        for &game_entity in &wheel_entities {
            if let Some(wheel) = game_world.get_wheel(game_entity)
                && result.entity == wheel.entity
                && result.distance <= INTERACT_RANGE
            {
                can_interact = true;
                break 'outer;
            }
        }

        for &game_entity in &button_entities {
            if let Some(button) = game_world.get_button(game_entity)
                && result.entity == button.entity
                && result.distance <= INTERACT_RANGE
            {
                can_interact = true;
                break 'outer;
            }
        }

        for &game_entity in &note_entities {
            if let Some(note) = game_world.get_note(game_entity)
                && result.entity == note.entity
                && result.distance <= INTERACT_RANGE
            {
                can_read = true;
                break 'outer;
            }
        }
    }

    let prompt_text = if can_read {
        "Read"
    } else if can_interact {
        "Interact"
    } else {
        ""
    };

    world.resources.text_cache.set_text(text_index, prompt_text);
    if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
        hud_text.dirty = true;
    }

    let crosshair_color = if can_interact || can_read {
        nalgebra_glm::Vec4::new(0.2, 1.0, 0.2, 0.9)
    } else {
        nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 0.7)
    };
    for &arm in &game_world.resources.crosshair_arms {
        if let Some(color) = world.ui.get_ui_node_color_mut(arm) {
            color.colors[0] = Some(crosshair_color);
            color.computed_color = crosshair_color;
        }
    }
}

pub fn pick_entities_cone(
    world: &World,
    center: Vec2,
    radius: f32,
    options: PickingOptions,
) -> Vec<PickingResult> {
    let mut all_results: Vec<PickingResult> = Vec::new();
    let mut seen_entities = std::collections::HashSet::new();

    let offsets = [
        (0.0, 0.0),
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (0.707, 0.707),
        (-0.707, 0.707),
        (0.707, -0.707),
        (-0.707, -0.707),
        (0.5, 0.0),
        (-0.5, 0.0),
        (0.0, 0.5),
        (0.0, -0.5),
    ];

    for (offset_x, offset_y) in offsets {
        let screen_pos =
            nalgebra_glm::vec2(center.x + offset_x * radius, center.y + offset_y * radius);

        let results = pick_entities(world, screen_pos, options);
        for result in results {
            if !seen_entities.contains(&result.entity) {
                seen_entities.insert(result.entity);
                all_results.push(result);
            }
        }
    }

    all_results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    all_results
}

pub fn note_reading_system(game_world: &mut GameWorld, world: &mut World) {
    let keyboard = &world.resources.input.keyboard;
    let f_pressed = keyboard.is_key_pressed(KeyCode::KeyF);

    let gamepad_rt_pressed = if let Some(gamepad) = query_active_gamepad(world) {
        let rt_axis = gamepad.value(gilrs::Axis::RightZ);
        let rt_button = gamepad.is_pressed(gilrs::Button::RightTrigger2);
        rt_axis > 0.5 || rt_button
    } else {
        false
    };

    let interact_pressed = f_pressed || gamepad_rt_pressed;

    if !game_world.resources.note_close_key_released && !interact_pressed {
        game_world.resources.note_close_key_released = true;
    }

    if game_world.resources.note_close_key_released && interact_pressed {
        game_world.resources.reading_note = None;
    }
}

pub fn check_fall_reset(game_world: &GameWorld, world: &mut World) {
    let Some(player_entity) = game_world.resources.player_entity else {
        return;
    };

    let Some(transform) = world.core.get_local_transform(player_entity) else {
        return;
    };

    if transform.translation.y < -20.0 {
        let spawn_position = nalgebra_glm::vec3(0.0, 1.2, 8.0);

        if let Some(transform) = world.core.get_local_transform_mut(player_entity) {
            transform.translation = spawn_position;
        }

        if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
            controller.velocity = nalgebra_glm::vec3(0.0, 0.0, 0.0);
        }
    }
}
