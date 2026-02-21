use crate::state::{ImmersiveSim, InputMode};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

pub fn detect_input_mode(game: &mut ImmersiveSim, world: &mut World) {
    let mouse_moved = world.resources.input.mouse.raw_mouse_delta.magnitude() > 0.1;
    let keyboard = &world.resources.input.keyboard;
    let key_pressed = keyboard.is_key_pressed(KeyCode::KeyW)
        || keyboard.is_key_pressed(KeyCode::KeyA)
        || keyboard.is_key_pressed(KeyCode::KeyS)
        || keyboard.is_key_pressed(KeyCode::KeyD)
        || keyboard.is_key_pressed(KeyCode::Space);

    if mouse_moved || key_pressed {
        game.input_mode = InputMode::MouseKeyboard;
    } else if let Some(gamepad) = query_active_gamepad(world) {
        let left_stick_x = gamepad.value(gilrs::Axis::LeftStickX).abs();
        let left_stick_y = gamepad.value(gilrs::Axis::LeftStickY).abs();
        let right_stick_x = gamepad.value(gilrs::Axis::RightStickX).abs();
        let right_stick_y = gamepad.value(gilrs::Axis::RightStickY).abs();

        let any_stick_input =
            left_stick_x > 0.2 || left_stick_y > 0.2 || right_stick_x > 0.2 || right_stick_y > 0.2;

        if any_stick_input || gamepad.is_pressed(gilrs::Button::South) {
            game.input_mode = InputMode::Gamepad;
        }
    }
}
