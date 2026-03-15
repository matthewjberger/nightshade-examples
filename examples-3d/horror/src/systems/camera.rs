use crate::constants::{
    CROUCHING_CAMERA_HEIGHT, LEAN_AMOUNT, LEAN_ANGLE, LEAN_SPEED, STANDING_CAMERA_HEIGHT,
};
use crate::ecs::{GameWorld, InputMode};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

pub fn camera_look_system(game_world: &mut GameWorld, world: &mut World) {
    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };

    let is_manipulating = game_world.resources.interaction.manipulated_door.is_some()
        || game_world.resources.interaction.manipulated_lever.is_some();

    let (gamepad_right_stick_x, gamepad_right_stick_y) =
        if game_world.resources.input_mode == InputMode::Gamepad && !is_manipulating {
            if let Some(gamepad) = query_active_gamepad(world) {
                let deadzone = 0.15;
                let raw_x = gamepad.value(gilrs::Axis::RightStickX);
                let raw_y = gamepad.value(gilrs::Axis::RightStickY);
                let magnitude = (raw_x * raw_x + raw_y * raw_y).sqrt();
                if magnitude > deadzone {
                    let normalized = (magnitude - deadzone) / (1.0 - deadzone);
                    (
                        raw_x * normalized / magnitude,
                        raw_y * normalized / magnitude,
                    )
                } else {
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            }
        } else {
            (0.0, 0.0)
        };

    let has_gamepad_input = gamepad_right_stick_x.abs() > 0.0 || gamepad_right_stick_y.abs() > 0.0;

    let can_look_mouse = game_world.resources.input_mode == InputMode::MouseKeyboard;

    if !can_look_mouse && !has_gamepad_input {
        return;
    }

    let dt = world.resources.window.timing.delta_time;

    let delta = if game_world.resources.input_mode == InputMode::Gamepad && has_gamepad_input {
        let gamepad_sensitivity = 1.2;
        nalgebra_glm::vec2(
            gamepad_right_stick_x * gamepad_sensitivity * dt,
            -gamepad_right_stick_y * gamepad_sensitivity * dt,
        )
    } else if game_world.resources.input_mode == InputMode::MouseKeyboard {
        let raw_delta = world.resources.input.mouse.raw_mouse_delta;
        let mouse_sensitivity = 0.002;
        raw_delta * mouse_sensitivity
    } else {
        return;
    };

    let yaw = nalgebra_glm::quat_angle_axis(-delta.x, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
    game_world.resources.lean_state.base_rotation =
        yaw * game_world.resources.lean_state.base_rotation;

    let forward = nalgebra_glm::quat_rotate_vec3(
        &game_world.resources.lean_state.base_rotation,
        &nalgebra_glm::vec3(0.0, 0.0, -1.0),
    );
    let current_pitch = forward.y.asin();
    let new_pitch = current_pitch - delta.y;

    if new_pitch.abs() <= 85_f32.to_radians() {
        let pitch = nalgebra_glm::quat_angle_axis(-delta.y, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        game_world.resources.lean_state.base_rotation *= pitch;
    }

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn lean_system(game_world: &mut GameWorld, world: &mut World) {
    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };

    if !game_world.resources.cutscene.active {
        let keyboard = &world.resources.input.keyboard;
        let lean_left = keyboard.is_key_pressed(KeyCode::KeyQ);
        let lean_right = keyboard.is_key_pressed(KeyCode::KeyE);

        let (gamepad_lean_left, gamepad_lean_right) =
            if let Some(gamepad) = query_active_gamepad(world) {
                (
                    gamepad.is_pressed(gilrs::Button::LeftTrigger),
                    gamepad.is_pressed(gilrs::Button::RightTrigger),
                )
            } else {
                (false, false)
            };

        let any_lean_left = lean_left || gamepad_lean_left;
        let any_lean_right = lean_right || gamepad_lean_right;

        game_world.resources.lean_state.target_lean = if any_lean_left && !any_lean_right {
            -1.0
        } else if any_lean_right && !any_lean_left {
            1.0
        } else {
            0.0
        };
    } else {
        game_world.resources.lean_state.target_lean = 0.0;
    }

    let dt = world.resources.window.timing.delta_time;
    let lean_diff =
        game_world.resources.lean_state.target_lean - game_world.resources.lean_state.current_lean;
    game_world.resources.lean_state.current_lean += lean_diff * (LEAN_SPEED * dt).min(1.0);

    let right_vector = nalgebra_glm::quat_rotate_vec3(
        &game_world.resources.lean_state.base_rotation,
        &nalgebra_glm::vec3(1.0, 0.0, 0.0),
    );
    let horizontal_right =
        nalgebra_glm::normalize(&nalgebra_glm::vec3(right_vector.x, 0.0, right_vector.z));

    let lean_offset =
        horizontal_right * (game_world.resources.lean_state.current_lean * LEAN_AMOUNT);

    let lean_roll = -game_world.resources.lean_state.current_lean * LEAN_ANGLE;
    let roll_quat = nalgebra_glm::quat_angle_axis(lean_roll, &nalgebra_glm::vec3(0.0, 0.0, 1.0));

    let final_rotation = game_world.resources.lean_state.base_rotation * roll_quat;

    let Some(camera_transform) = world.core.get_local_transform_mut(camera_entity) else {
        return;
    };

    camera_transform.translation.x = lean_offset.x;
    camera_transform.translation.z = lean_offset.z;
    camera_transform.rotation = final_rotation;

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn crouch_camera_system(game_world: &GameWorld, world: &mut World) {
    let Some(player_entity) = game_world.resources.player_entity else {
        return;
    };
    let Some(camera_entity) = game_world.resources.camera_entity else {
        return;
    };

    let is_crouching = world
        .core
        .get_character_controller(player_entity)
        .map(|cc| cc.is_crouching)
        .unwrap_or(false);

    let target_height = if is_crouching {
        CROUCHING_CAMERA_HEIGHT
    } else {
        STANDING_CAMERA_HEIGHT
    };

    let dt = world.resources.window.timing.delta_time;
    if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
        let speed = 8.0;
        let diff = target_height - transform.translation.y;
        transform.translation.y += diff * speed * dt;
    }

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}
