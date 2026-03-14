use crate::constants::{
    CROUCHING_CAMERA_HEIGHT, LEAN_AMOUNT, LEAN_ANGLE, LEAN_SPEED, STANDING_CAMERA_HEIGHT,
};
use crate::state::{ImmersiveSim, InputMode};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::world::resources::MouseState;
use nightshade::prelude::*;

pub fn camera_look_system(game: &mut ImmersiveSim, world: &mut World) {
    let Some(camera_entity) = game.camera_entity else {
        return;
    };

    let is_interacting = game.interaction.grabbed_entity.is_some();

    let right_clicked = if game.input_mode == InputMode::MouseKeyboard {
        world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::RIGHT_CLICKED)
    } else {
        false
    };

    let (gamepad_right_stick_x, gamepad_right_stick_y) = if game.input_mode == InputMode::Gamepad {
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

    let should_lock_cursor = right_clicked || is_interacting;

    if should_lock_cursor {
        if let Some(window_handle) = &world.resources.window.handle {
            if window_handle
                .set_cursor_grab(window::CursorGrabMode::Locked)
                .is_err()
            {
                let _ = window_handle.set_cursor_grab(window::CursorGrabMode::Confined);
            }
            window_handle.set_cursor_visible(false);
        }
    } else if let Some(window_handle) = &world.resources.window.handle {
        let _ = window_handle.set_cursor_grab(window::CursorGrabMode::None);
        window_handle.set_cursor_visible(true);
    }

    let can_look_mouse = right_clicked || game.interaction.grabbed_entity.is_some();

    if !can_look_mouse && !has_gamepad_input {
        return;
    }

    let delta_time = world.resources.window.timing.delta_time;

    let delta = if game.input_mode == InputMode::Gamepad && has_gamepad_input {
        let gamepad_sensitivity = 1.2;
        nalgebra_glm::vec2(
            gamepad_right_stick_x * gamepad_sensitivity * delta_time,
            -gamepad_right_stick_y * gamepad_sensitivity * delta_time,
        )
    } else if game.input_mode == InputMode::MouseKeyboard {
        let raw_delta = world.resources.input.mouse.raw_mouse_delta;
        let mouse_sensitivity = 0.002;
        raw_delta * mouse_sensitivity
    } else {
        return;
    };

    let yaw = nalgebra_glm::quat_angle_axis(-delta.x, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
    game.lean_state.base_rotation = yaw * game.lean_state.base_rotation;

    let forward = nalgebra_glm::quat_rotate_vec3(
        &game.lean_state.base_rotation,
        &nalgebra_glm::vec3(0.0, 0.0, -1.0),
    );
    let current_pitch = forward.y.asin();
    let new_pitch = current_pitch - delta.y;

    if new_pitch.abs() <= 85_f32.to_radians() {
        let pitch = nalgebra_glm::quat_angle_axis(-delta.y, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        game.lean_state.base_rotation *= pitch;
    }

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn lean_system(game: &mut ImmersiveSim, world: &mut World) {
    let Some(camera_entity) = game.camera_entity else {
        return;
    };

    let keyboard = &world.resources.input.keyboard;
    let lean_left = keyboard.is_key_pressed(KeyCode::KeyQ);
    let lean_right = keyboard.is_key_pressed(KeyCode::KeyE);

    game.lean_state.target_lean = if lean_left && !lean_right {
        -1.0
    } else if lean_right && !lean_left {
        1.0
    } else {
        0.0
    };

    let delta_time = world.resources.window.timing.delta_time;
    let lean_diff = game.lean_state.target_lean - game.lean_state.current_lean;
    game.lean_state.current_lean += lean_diff * (LEAN_SPEED * delta_time).min(1.0);

    let right_vector = nalgebra_glm::quat_rotate_vec3(
        &game.lean_state.base_rotation,
        &nalgebra_glm::vec3(1.0, 0.0, 0.0),
    );
    let horizontal_right =
        nalgebra_glm::normalize(&nalgebra_glm::vec3(right_vector.x, 0.0, right_vector.z));

    let lean_offset = horizontal_right * (game.lean_state.current_lean * LEAN_AMOUNT);

    let lean_roll = -game.lean_state.current_lean * LEAN_ANGLE;
    let roll_quat = nalgebra_glm::quat_angle_axis(lean_roll, &nalgebra_glm::vec3(0.0, 0.0, 1.0));

    let final_rotation = game.lean_state.base_rotation * roll_quat;

    let Some(camera_transform) = world.core.get_local_transform_mut(camera_entity) else {
        return;
    };

    camera_transform.translation.x = lean_offset.x;
    camera_transform.translation.z = lean_offset.z;
    camera_transform.rotation = final_rotation;

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn crouch_camera_system(game: &ImmersiveSim, world: &mut World) {
    let Some(player_entity) = game.player_entity else {
        return;
    };
    let Some(camera_entity) = game.camera_entity else {
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

    let delta_time = world.resources.window.timing.delta_time;
    if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
        let speed = 8.0;
        let diff = target_height - transform.translation.y;
        transform.translation.y += diff * speed * delta_time;
    }

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}
