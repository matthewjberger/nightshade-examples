use crate::constants::{
    ANGULAR_DAMPING, GRAB_DAMPING_RATIO, GRAB_RANGE, GRAB_STIFFNESS, INTERACT_CONE_RADIUS,
    MAX_GRAB_DISTANCE, MAX_GRAB_FORCE, MIN_GRAB_DISTANCE, SCROLL_DISTANCE_SPEED, THROW_STRENGTH,
};
use crate::data::dialogue::get_dialogue_tree;
use crate::data::npcs::NPC_DEFINITIONS;
use crate::state::ImmersiveSim;
use crate::state::InputMode;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::MouseState;
use nightshade::ecs::picking::{PickingOptions, PickingResult, pick_entities};
use nightshade::prelude::*;

pub fn interaction_system(game: &mut ImmersiveSim, world: &mut World) {
    let mouse = &world.resources.input.mouse;
    let keyboard = &world.resources.input.keyboard;
    let mouse_pos = mouse.position;

    let left_clicked = mouse.state.contains(MouseState::LEFT_CLICKED);
    let left_just_pressed = mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let f_pressed = keyboard.is_key_pressed(KeyCode::KeyF);

    let gamepad_rt_pressed = if let Some(gamepad) = query_active_gamepad(world) {
        let rt_axis = gamepad.value(gilrs::Axis::RightZ);
        let rt_button = gamepad.is_pressed(gilrs::Button::RightTrigger2);
        rt_axis > 0.5 || rt_button
    } else {
        false
    };

    let gamepad_rt_just_pressed = gamepad_rt_pressed && !game.interaction.gamepad_rt_was_pressed;
    game.interaction.gamepad_rt_was_pressed = gamepad_rt_pressed;

    let interact_pressed = left_clicked || f_pressed || gamepad_rt_pressed;
    let interact_just_pressed = left_just_pressed || gamepad_rt_just_pressed;

    if game.interaction.require_interact_release {
        if !interact_pressed {
            game.interaction.require_interact_release = false;
        }
        return;
    }

    let Some(camera_entity) = game.camera_entity else {
        return;
    };
    let Some(camera_transform) = world.core.get_global_transform(camera_entity).cloned() else {
        return;
    };
    let camera_position = camera_transform.translation();
    let camera_forward = camera_transform.forward_vector();

    if !interact_pressed {
        if game.interaction.grabbed_entity.is_some() {
            throw_grabbed_object(game, world, camera_forward);
        }
        game.interaction.grabbed_entity = None;
        return;
    }

    if game.interaction.grabbed_entity.is_some() {
        let scroll_delta = world.resources.input.mouse.wheel_delta.y;
        update_grabbed_object(game, world, camera_position, camera_forward, scroll_delta);
        return;
    }

    if !interact_just_pressed {
        return;
    }

    let screen_pos = if game.input_mode == InputMode::Gamepad {
        let viewport_size = world
            .resources
            .window
            .cached_viewport_size
            .unwrap_or((800, 600));
        nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0)
    } else {
        mouse_pos
    };

    let options = PickingOptions {
        max_distance: GRAB_RANGE,
        ignore_invisible: true,
    };

    let pick_results = if game.input_mode == InputMode::Gamepad {
        pick_entities_cone(game, world, screen_pos, INTERACT_CONE_RADIUS, options)
    } else {
        pick_entities(world, screen_pos, options)
    };

    try_start_interaction(game, world, &pick_results);
}

pub fn pick_entities_cone(
    _game: &ImmersiveSim,
    world: &mut World,
    center: Vec2,
    radius: f32,
    options: PickingOptions,
) -> Vec<PickingResult> {
    let mut all_results = Vec::new();
    let offsets = [
        (0.0, 0.0),
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (0.7, 0.7),
        (-0.7, 0.7),
        (0.7, -0.7),
        (-0.7, -0.7),
    ];

    for (offset_x, offset_y) in offsets {
        let screen_pos =
            nalgebra_glm::vec2(center.x + offset_x * radius, center.y + offset_y * radius);
        let results = pick_entities(world, screen_pos, options);
        for result in results {
            if !all_results
                .iter()
                .any(|r: &PickingResult| r.entity == result.entity)
            {
                all_results.push(result);
            }
        }
    }

    all_results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    all_results
}

pub fn try_start_interaction(
    game: &mut ImmersiveSim,
    world: &World,
    pick_results: &[PickingResult],
) {
    if let Some(npc_index) = crate::systems::npcs::get_looked_at_npc(game, world) {
        start_npc_dialogue(game, npc_index);
        return;
    }

    for result in pick_results {
        if game.physics_objects.contains(&result.entity) {
            game.interaction.grabbed_entity = Some(result.entity);
            game.interaction.grab_distance = result.distance.min(MAX_GRAB_DISTANCE);
            return;
        }
    }
}

fn start_npc_dialogue(game: &mut ImmersiveSim, npc_index: usize) {
    if npc_index >= NPC_DEFINITIONS.len() || npc_index >= game.npc_entities.len() {
        return;
    }

    let npc_def = &NPC_DEFINITIONS[npc_index];
    let nodes = get_dialogue_tree(npc_def.dialogue_id);

    game.dialogue.active = true;
    game.dialogue.current_node = 0;
    game.dialogue.current_line = 0;
    game.dialogue.nodes = nodes;
    game.dialogue.speaking_npc = Some(game.npc_entities[npc_index]);
    game.dialogue.npc_name = npc_def.name.to_string();
    game.dialogue.advance_key_was_pressed = true;
    game.interaction.require_interact_release = true;
}

pub fn update_grabbed_object(
    game: &mut ImmersiveSim,
    world: &mut World,
    camera_position: Vec3,
    camera_forward: Vec3,
    scroll_delta: f32,
) {
    game.interaction.grab_distance = (game.interaction.grab_distance
        + scroll_delta * SCROLL_DISTANCE_SPEED)
        .clamp(MIN_GRAB_DISTANCE, MAX_GRAB_DISTANCE);

    let target_position = camera_position + camera_forward * game.interaction.grab_distance;

    let Some(grabbed_entity) = game.interaction.grabbed_entity else {
        return;
    };

    let Some(rigid_body_component) = world.core.get_rigid_body(grabbed_entity) else {
        return;
    };
    let Some(handle) = rigid_body_component.handle else {
        return;
    };
    let Some(rigid_body) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(handle.into())
    else {
        return;
    };

    let current_pos = rigid_body.translation();
    let current_position = nalgebra_glm::vec3(current_pos.x, current_pos.y, current_pos.z);

    let displacement = target_position - current_position;

    let current_vel = rigid_body.linvel();
    let current_velocity = nalgebra_glm::vec3(current_vel.x, current_vel.y, current_vel.z);

    let mass = rigid_body.mass();
    let critical_damping = 2.0 * (GRAB_STIFFNESS * mass).sqrt();
    let damping = critical_damping * GRAB_DAMPING_RATIO;

    let spring_force = displacement * GRAB_STIFFNESS;
    let damping_force = -current_velocity * damping;
    let mut total_force = spring_force + damping_force;

    let force_magnitude = nalgebra_glm::length(&total_force);
    let max_force_for_mass = MAX_GRAB_FORCE * mass.max(0.5);
    if force_magnitude > max_force_for_mass {
        total_force *= max_force_for_mass / force_magnitude;
    }

    let acceleration = total_force / mass;
    let delta_time = world.resources.physics.fixed_timestep;
    let new_velocity = current_velocity + acceleration * delta_time;

    rigid_body.set_linvel(
        rapier3d::math::Vector::new(new_velocity.x, new_velocity.y, new_velocity.z),
        true,
    );

    let current_angvel = rigid_body.angvel();
    let angular_decay = (-ANGULAR_DAMPING * delta_time * 60.0).exp();
    rigid_body.set_angvel(current_angvel * angular_decay, true);
}

pub fn throw_grabbed_object(game: &mut ImmersiveSim, world: &mut World, camera_forward: Vec3) {
    let Some(grabbed_entity) = game.interaction.grabbed_entity else {
        return;
    };

    let Some(rigid_body_component) = world.core.get_rigid_body(grabbed_entity) else {
        return;
    };
    let Some(handle) = rigid_body_component.handle else {
        return;
    };
    let Some(rigid_body) = world
        .resources
        .physics
        .rigid_body_set
        .get_mut(handle.into())
    else {
        return;
    };

    let throw_velocity = camera_forward * THROW_STRENGTH;
    rigid_body.set_linvel(
        rapier3d::math::Vector::new(throw_velocity.x, throw_velocity.y, throw_velocity.z),
        true,
    );

    game.interaction.grabbed_entity = None;
}
