use crate::ecs::{GameEvents, GameWorld};
use crate::selection::clear_selection;
use crate::systems::action::{apply_input_result, determine_action};
use nightshade::prelude::*;

pub fn input_system(game_world: &mut GameWorld, world: &mut World, events: &mut GameEvents) {
    let mouse = &world.resources.input.mouse;
    let left_clicked = mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let right_clicked = mouse.state.contains(MouseState::RIGHT_JUST_PRESSED);

    if right_clicked {
        clear_selection(game_world);
        return;
    }

    if !left_clicked {
        return;
    }

    let Some(hovered_tile) = game_world.resources.hovered_tile else {
        return;
    };

    let result = determine_action(game_world, hovered_tile);
    apply_input_result(game_world, world, result, events);
}
