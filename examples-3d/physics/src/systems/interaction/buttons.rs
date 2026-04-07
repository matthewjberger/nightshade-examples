use crate::ecs::{BAUBLE_SPAWN, ButtonAction, GameWorld};
use nightshade::prelude::*;

pub(super) fn update_pressed_button(
    game_world: &mut GameWorld,
    world: &mut World,
    button_game_entity: freecs::Entity,
) {
    let delta_time = world.resources.window.timing.delta_time;
    let press_speed = 8.0;
    let max_press = 0.03;

    let Some(button) = game_world.get_button_mut(button_game_entity) else {
        return;
    };
    button.current_press = (button.current_press + press_speed * delta_time).min(max_press);

    let pressed_y = button.base_position.y - button.current_press;
    let current_press = button.current_press;
    let is_pressed = button.is_pressed;
    let entity = button.entity;
    let base_position = button.base_position;

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation.y = pressed_y;
    }
    world.mark_local_transform_dirty(entity);

    if let Some(rb) = world.core.get_rigid_body_mut(entity)
        && let Some(handle) = rb.handle
    {
        let physics = &mut world.resources.physics;
        if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
            rigid_body.set_next_kinematic_translation(rapier3d::prelude::Vector::new(
                base_position.x,
                pressed_y,
                base_position.z,
            ));
        }
    }

    if current_press >= max_press
        && !is_pressed
        && let Some(button) = game_world.get_button_mut(button_game_entity)
    {
        button.is_pressed = true;
        let action = button.action.clone();
        match action {
            ButtonAction::RecallBaubles => recall_baubles(game_world, world),
        }
    }
}

pub(super) fn release_button(
    game_world: &mut GameWorld,
    world: &mut World,
    button_game_entity: freecs::Entity,
) {
    let Some(button) = game_world.get_button_mut(button_game_entity) else {
        return;
    };
    button.current_press = 0.0;
    button.is_pressed = false;
    let entity = button.entity;
    let base_position = button.base_position;

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation.y = base_position.y;
    }
    world.mark_local_transform_dirty(entity);

    if let Some(rb) = world.core.get_rigid_body_mut(entity)
        && let Some(handle) = rb.handle
    {
        let physics = &mut world.resources.physics;
        if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
            rigid_body.set_next_kinematic_translation(rapier3d::prelude::Vector::new(
                base_position.x,
                base_position.y,
                base_position.z,
            ));
        }
    }
}

fn recall_baubles(game_world: &mut GameWorld, world: &mut World) {
    let bauble_entities: Vec<freecs::Entity> = game_world.query_entities(BAUBLE_SPAWN).collect();

    for game_entity in bauble_entities {
        let Some(bauble) = game_world.get_bauble_spawn(game_entity) else {
            continue;
        };
        let entity = bauble.entity;
        let spawn_position = bauble.spawn_position;

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = spawn_position;
        }
        world.mark_local_transform_dirty(entity);

        if let Some(rb) = world.core.get_rigid_body_mut(entity)
            && let Some(handle) = rb.handle
        {
            let physics = &mut world.resources.physics;
            if let Some(rigid_body) = physics.rigid_body_set.get_mut(handle.into()) {
                rigid_body.set_translation(
                    rapier3d::prelude::Vector::new(
                        spawn_position.x,
                        spawn_position.y,
                        spawn_position.z,
                    ),
                    true,
                );
                rigid_body.set_linvel(rapier3d::prelude::Vector::zeros(), true);
                rigid_body.set_angvel(rapier3d::prelude::Vector::zeros(), true);
            }
        }
    }
}
