use crate::state::LevelDemo;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::prelude::*;

pub fn check_dialogue_triggers(demo: &mut LevelDemo, world: &mut World) {
    let keyboard = &world.resources.input.keyboard;
    let interact_pressed = keyboard.is_key_pressed(KeyCode::KeyE);

    let gamepad_interact = if let Some(gamepad) = query_active_gamepad(world) {
        gamepad.is_pressed(gilrs::Button::South)
    } else {
        false
    };

    let any_pressed = interact_pressed || gamepad_interact;
    let _just_pressed = any_pressed && !demo.dialogue.interact_key_was_pressed;
    demo.dialogue.interact_key_was_pressed = any_pressed;
}

pub fn dialogue_system(demo: &mut LevelDemo, world: &mut World) {
    if !demo.dialogue.active {
        demo.dialogue.advance_key_was_pressed = false;
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
        (any_advance_pressed && !demo.dialogue.advance_key_was_pressed) || mouse_clicked;
    demo.dialogue.advance_key_was_pressed = any_advance_pressed;

    if cancel_pressed {
        demo.dialogue.active = false;
        demo.dialogue.speaking_npc = None;
        return;
    }

    if !just_pressed {
        return;
    }

    if demo.dialogue.current_node >= demo.dialogue.nodes.len() {
        demo.dialogue.active = false;
        demo.dialogue.speaking_npc = None;
        return;
    }

    let node = &demo.dialogue.nodes[demo.dialogue.current_node];

    if demo.dialogue.current_line < node.lines.len() {
        demo.dialogue.current_line += 1;
    } else if !node.choices.is_empty() {
    } else {
        demo.dialogue.active = false;
        demo.dialogue.speaking_npc = None;
    }
}

pub fn select_dialogue_choice(demo: &mut LevelDemo, choice_index: usize) {
    if !demo.dialogue.active {
        return;
    }

    if demo.dialogue.current_node >= demo.dialogue.nodes.len() {
        return;
    }

    let node = &demo.dialogue.nodes[demo.dialogue.current_node];

    if choice_index >= node.choices.len() {
        return;
    }

    let choice = &node.choices[choice_index];

    if let Some(next_node) = choice.next_node {
        demo.dialogue.current_node = next_node;
        demo.dialogue.current_line = 0;
    } else {
        demo.dialogue.active = false;
        demo.dialogue.speaking_npc = None;
    }
}
