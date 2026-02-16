use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::prelude::*;

use crate::CityDemo;

pub const STANDING_CAMERA_HEIGHT: f32 = 0.8;
const CROUCHING_CAMERA_HEIGHT: f32 = 0.3;
const LEAN_AMOUNT: f32 = 0.4;
const LEAN_ANGLE: f32 = 0.15;
const LEAN_SPEED: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    MouseKeyboard,
    Gamepad,
}

#[derive(Default)]
pub struct LeanState {
    pub current_lean: f32,
    pub target_lean: f32,
    pub base_rotation: nalgebra_glm::Quat,
}

impl LeanState {
    pub fn new() -> Self {
        Self {
            current_lean: 0.0,
            target_lean: 0.0,
            base_rotation: nalgebra_glm::quat_identity(),
        }
    }
}

pub fn detect_input_mode(demo: &mut CityDemo, world: &mut World) {
    let keyboard = &world.resources.input.keyboard;
    let mouse = &world.resources.input.mouse;

    let has_keyboard_input = keyboard
        .keystates
        .values()
        .any(|state| *state == ElementState::Pressed);
    let has_mouse_input = mouse.raw_mouse_delta.x.abs() > 0.1
        || mouse.raw_mouse_delta.y.abs() > 0.1
        || mouse.state.contains(MouseState::LEFT_CLICKED)
        || mouse.state.contains(MouseState::RIGHT_CLICKED);

    let has_gamepad_input = if let Some(gamepad) = query_active_gamepad(world) {
        let left_stick_x = gamepad.value(gilrs::Axis::LeftStickX);
        let left_stick_y = gamepad.value(gilrs::Axis::LeftStickY);
        let right_stick_x = gamepad.value(gilrs::Axis::RightStickX);
        let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
        let deadzone = 0.15;
        left_stick_x.abs() > deadzone
            || left_stick_y.abs() > deadzone
            || right_stick_x.abs() > deadzone
            || right_stick_y.abs() > deadzone
    } else {
        false
    };

    if has_gamepad_input && demo.input_mode != InputMode::Gamepad {
        demo.input_mode = InputMode::Gamepad;
    } else if (has_keyboard_input || has_mouse_input) && demo.input_mode != InputMode::MouseKeyboard
    {
        demo.input_mode = InputMode::MouseKeyboard;
    }
}

pub fn camera_look_system(demo: &mut CityDemo, world: &mut World) {
    let Some(camera_entity) = demo.player_camera_entity else {
        return;
    };

    let right_clicked = if demo.input_mode == InputMode::MouseKeyboard {
        world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::RIGHT_CLICKED)
    } else {
        false
    };

    let (gamepad_right_stick_x, gamepad_right_stick_y) = if demo.input_mode == InputMode::Gamepad {
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

    let should_lock_cursor = right_clicked;

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

    if !right_clicked && !has_gamepad_input {
        return;
    }

    let delta_time = world.resources.window.timing.delta_time;

    let delta = if demo.input_mode == InputMode::Gamepad && has_gamepad_input {
        let gamepad_sensitivity = 1.2;
        nalgebra_glm::vec2(
            gamepad_right_stick_x * gamepad_sensitivity * delta_time,
            -gamepad_right_stick_y * gamepad_sensitivity * delta_time,
        )
    } else if demo.input_mode == InputMode::MouseKeyboard {
        let raw_delta = world.resources.input.mouse.raw_mouse_delta;
        let mouse_sensitivity = 0.002;
        raw_delta * mouse_sensitivity
    } else {
        return;
    };

    let yaw = nalgebra_glm::quat_angle_axis(-delta.x, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
    demo.lean_state.base_rotation = yaw * demo.lean_state.base_rotation;

    let forward = nalgebra_glm::quat_rotate_vec3(
        &demo.lean_state.base_rotation,
        &nalgebra_glm::vec3(0.0, 0.0, -1.0),
    );
    let current_pitch = forward.y.asin();
    let new_pitch = current_pitch - delta.y;

    if new_pitch.abs() <= 85_f32.to_radians() {
        let pitch = nalgebra_glm::quat_angle_axis(-delta.y, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        demo.lean_state.base_rotation *= pitch;
    }

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn lean_system(demo: &mut CityDemo, world: &mut World) {
    let Some(camera_entity) = demo.player_camera_entity else {
        return;
    };

    let keyboard = &world.resources.input.keyboard;
    let lean_left = keyboard.is_key_pressed(KeyCode::KeyQ);
    let lean_right = keyboard.is_key_pressed(KeyCode::KeyE);

    demo.lean_state.target_lean = if lean_left && !lean_right {
        -1.0
    } else if lean_right && !lean_left {
        1.0
    } else {
        0.0
    };

    let delta_time = world.resources.window.timing.delta_time;
    let lean_diff = demo.lean_state.target_lean - demo.lean_state.current_lean;
    demo.lean_state.current_lean += lean_diff * (LEAN_SPEED * delta_time).min(1.0);

    let right_vector = nalgebra_glm::quat_rotate_vec3(
        &demo.lean_state.base_rotation,
        &nalgebra_glm::vec3(1.0, 0.0, 0.0),
    );
    let horizontal_right =
        nalgebra_glm::normalize(&nalgebra_glm::vec3(right_vector.x, 0.0, right_vector.z));

    let lean_offset = horizontal_right * (demo.lean_state.current_lean * LEAN_AMOUNT);

    let lean_roll = -demo.lean_state.current_lean * LEAN_ANGLE;
    let roll_quat = nalgebra_glm::quat_angle_axis(lean_roll, &nalgebra_glm::vec3(0.0, 0.0, 1.0));

    let final_rotation = demo.lean_state.base_rotation * roll_quat;

    let Some(camera_transform) = world.get_local_transform_mut(camera_entity) else {
        return;
    };

    camera_transform.translation.x = lean_offset.x;
    camera_transform.translation.z = lean_offset.z;
    camera_transform.rotation = final_rotation;

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn crouch_camera_system(demo: &CityDemo, world: &mut World) {
    let Some(player_entity) = demo.player_entity else {
        return;
    };
    let Some(camera_entity) = demo.player_camera_entity else {
        return;
    };

    let is_crouching = world
        .get_character_controller(player_entity)
        .map(|cc| cc.is_crouching)
        .unwrap_or(false);

    let target_height = if is_crouching {
        CROUCHING_CAMERA_HEIGHT
    } else {
        STANDING_CAMERA_HEIGHT
    };

    let delta_time = world.resources.window.timing.delta_time;
    if let Some(transform) = world.get_local_transform_mut(camera_entity) {
        let speed = 8.0;
        let diff = target_height - transform.translation.y;
        transform.translation.y += diff * speed * delta_time;
    }

    nightshade::ecs::transform::commands::mark_local_transform_dirty(world, camera_entity);
}

pub fn spawn_flashlight(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];

    world.set_light(
        entity,
        Light {
            light_type: LightType::Spot,
            color: nalgebra_glm::vec3(1.0, 0.95, 0.8),
            intensity: 150.0,
            range: 50.0,
            inner_cone_angle: 0.15,
            outer_cone_angle: 0.4,
            cast_shadows: true,
            shadow_bias: 0.0001,
        },
    );

    world.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    world.set_global_transform(entity, GlobalTransform::default());
    world.set_local_transform_dirty(entity, LocalTransformDirty);

    entity
}

pub fn update_flashlight(demo: &mut CityDemo, world: &mut World) {
    let Some(flashlight_entity) = demo.flashlight_entity else {
        return;
    };

    let f_pressed = world.resources.input.keyboard.is_key_pressed(KeyCode::KeyF);
    let gamepad_y_pressed = query_active_gamepad(world)
        .map(|gamepad| gamepad.is_pressed(gilrs::Button::North))
        .unwrap_or(false);
    let toggle_pressed = f_pressed || gamepad_y_pressed;

    if toggle_pressed && !demo.flashlight_key_was_pressed {
        demo.flashlight_on = !demo.flashlight_on;
        if let Some(light) = world.get_light_mut(flashlight_entity) {
            light.intensity = if demo.flashlight_on { 150.0 } else { 0.0 };
        }
    }
    demo.flashlight_key_was_pressed = toggle_pressed;
}
