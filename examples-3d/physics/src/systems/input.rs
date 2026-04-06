use crate::ecs::{GameWorld, InputMode};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::prelude::*;

pub fn detect_input_mode(game_world: &mut GameWorld, world: &mut World) {
    let keyboard = &world.resources.input.keyboard;
    let mouse = &world.resources.input.mouse;

    let has_keyboard_input = keyboard
        .keystates
        .values()
        .any(|state| *state == ElementState::Pressed);
    let has_mouse_input = mouse.raw_mouse_delta.x.abs() > 0.1
        || mouse.raw_mouse_delta.y.abs() > 0.1
        || mouse.state.contains(MouseState::LEFT_CLICKED)
        || mouse.state.contains(MouseState::RIGHT_CLICKED)
        || mouse.wheel_delta.y.abs() > 0.01;

    let has_gamepad_input = if let Some(gamepad) = query_active_gamepad(world) {
        let left_stick_x = gamepad.value(gilrs::Axis::LeftStickX);
        let left_stick_y = gamepad.value(gilrs::Axis::LeftStickY);
        let right_stick_x = gamepad.value(gilrs::Axis::RightStickX);
        let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
        let rt_value = gamepad.value(gilrs::Axis::RightZ);
        let lt_value = gamepad.value(gilrs::Axis::LeftZ);

        let deadzone = 0.15;
        left_stick_x.abs() > deadzone
            || left_stick_y.abs() > deadzone
            || right_stick_x.abs() > deadzone
            || right_stick_y.abs() > deadzone
            || rt_value > 0.3
            || lt_value > 0.3
            || gamepad.is_pressed(gilrs::Button::South)
            || gamepad.is_pressed(gilrs::Button::East)
            || gamepad.is_pressed(gilrs::Button::West)
            || gamepad.is_pressed(gilrs::Button::North)
            || gamepad.is_pressed(gilrs::Button::LeftTrigger)
            || gamepad.is_pressed(gilrs::Button::RightTrigger)
            || gamepad.is_pressed(gilrs::Button::LeftTrigger2)
            || gamepad.is_pressed(gilrs::Button::RightTrigger2)
            || gamepad.is_pressed(gilrs::Button::LeftThumb)
            || gamepad.is_pressed(gilrs::Button::RightThumb)
    } else {
        false
    };

    let previous_mode = game_world.resources.input_mode;

    if has_gamepad_input && game_world.resources.input_mode != InputMode::Gamepad {
        game_world.resources.input_mode = InputMode::Gamepad;
    } else if (has_keyboard_input || has_mouse_input)
        && game_world.resources.input_mode != InputMode::MouseKeyboard
    {
        game_world.resources.input_mode = InputMode::MouseKeyboard;
    }

    if game_world.resources.input_mode != previous_mode {
        world.resources.graphics.show_cursor = false;
        world.set_cursor_visible(false);

        if let Some(text_index) = game_world.resources.ui.input_mode_text_index {
            let text = match game_world.resources.input_mode {
                InputMode::MouseKeyboard => "Mouse/Keyboard",
                InputMode::Gamepad => "Gamepad",
                #[cfg(feature = "openxr")]
                InputMode::Xr => "VR",
            };
            world.resources.text_cache.set_text(text_index, text);
            if let Some(entity) = game_world.resources.ui.input_mode_text_entity
                && let Some(hud_text) = world.core.get_text_mut(entity)
            {
                hud_text.dirty = true;
            }
        }
    }
}
