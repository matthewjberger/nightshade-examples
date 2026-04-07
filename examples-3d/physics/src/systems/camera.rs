use crate::ecs::GameWorld;
use nightshade::prelude::*;

pub fn camera_look_system(game_world: &mut GameWorld, world: &mut World) {
    if game_world.resources.interaction.manipulated.is_some() {
        return;
    }

    if world.resources.input.input_mode == InputMode::MouseKeyboard {
        world.set_cursor_locked(true);
        world.set_cursor_visible(false);
    }

    first_person_camera_look_system(world);
}
