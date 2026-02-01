use crate::state::ImmersiveSim;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::prelude::*;

pub fn dialogue_system(game: &mut ImmersiveSim, world: &mut World) {
    if !game.dialogue.active {
        game.dialogue.advance_key_was_pressed = false;
        return;
    }

    let advance_key_pressed = {
        let keyboard = &world.resources.input.keyboard;
        keyboard.is_key_pressed(KeyCode::Space) || keyboard.is_key_pressed(KeyCode::Enter)
    };

    let mouse_clicked = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::LEFT_JUST_PRESSED);

    let cancel_pressed = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Escape);

    let gamepad_advance = if let Some(gamepad) = query_active_gamepad(world) {
        gamepad.is_pressed(gilrs::Button::South)
    } else {
        false
    };

    let any_advance_pressed = advance_key_pressed || gamepad_advance;
    let just_pressed =
        (any_advance_pressed && !game.dialogue.advance_key_was_pressed) || mouse_clicked;
    game.dialogue.advance_key_was_pressed = any_advance_pressed;

    if cancel_pressed {
        game.dialogue.active = false;
        game.dialogue.speaking_npc = None;
        return;
    }

    if !just_pressed {
        return;
    }

    if game.dialogue.current_node >= game.dialogue.nodes.len() {
        game.dialogue.active = false;
        game.dialogue.speaking_npc = None;
        return;
    }

    let node = &game.dialogue.nodes[game.dialogue.current_node];

    if game.dialogue.current_line < node.lines.len() {
        game.dialogue.current_line += 1;
    } else if node.choices.is_empty() {
        game.dialogue.active = false;
        game.dialogue.speaking_npc = None;
    }
}

pub fn select_dialogue_choice(game: &mut ImmersiveSim, choice_index: usize) {
    if !game.dialogue.active {
        return;
    }

    if game.dialogue.current_node >= game.dialogue.nodes.len() {
        return;
    }

    let node = &game.dialogue.nodes[game.dialogue.current_node];

    if choice_index >= node.choices.len() {
        return;
    }

    let choice = &node.choices[choice_index];

    if let Some(next_node) = choice.next_node {
        game.dialogue.current_node = next_node;
        game.dialogue.current_line = 0;
    } else {
        game.dialogue.active = false;
        game.dialogue.speaking_npc = None;
    }
}
