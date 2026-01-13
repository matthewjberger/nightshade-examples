use crate::state::{HorrorDemo, InputMode};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::world::resources::MouseState;
use nightshade::prelude::*;

pub fn detect_input_mode(demo: &mut HorrorDemo, world: &mut World) {
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
