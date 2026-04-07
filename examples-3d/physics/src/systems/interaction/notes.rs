use crate::ecs::GameWorld;
use nightshade::prelude::*;

pub fn note_reading_system(game_world: &mut GameWorld, world: &mut World) {
    let keyboard_close = world.resources.input.keyboard.just_pressed(KeyCode::Tab);
    let gamepad_close = world.resources.input.gamepad.just_pressed(gilrs::Button::East);

    if keyboard_close || gamepad_close {
        game_world.resources.ui.reading_note = None;
    }
}
