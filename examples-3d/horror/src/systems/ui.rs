use crate::constants::{GRAB_RANGE, INTERACT_CONE_RADIUS, INTERACT_RANGE};
use crate::state::{HorrorDemo, InputMode};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::picking::{PickingOptions, PickingResult, pick_entities};
use nightshade::ecs::text::commands::spawn_ui_text_with_properties;
use nightshade::ecs::text::components::TextProperties;
use nightshade::prelude::*;

pub fn spawn_ui(demo: &mut HorrorDemo, world: &mut World) {
    let prompt_entity = spawn_ui_text_with_properties(
        world,
        "",
            Vec2::zeros(),
        TextProperties {
            font_size: 18.0,
            color: Vec4::new(1.0, 1.0, 1.0, 0.9),
            ..Default::default()
        },
    );
    demo.interaction_prompt_entity = Some(prompt_entity);
    if let Some(hud_text) = world.core.get_text(prompt_entity) {
        demo.interaction_prompt_text_index = Some(hud_text.text_index);
    }

    let objective_entity = spawn_ui_text_with_properties(
        world,
        "Find the generator and restore power",
            Vec2::zeros(),
        TextProperties {
            font_size: 20.0,
            color: Vec4::new(0.8, 0.8, 0.6, 0.9),
            ..Default::default()
        },
    );
    demo.objective_text_entity = Some(objective_entity);
    if let Some(hud_text) = world.core.get_text(objective_entity) {
        demo.objective_text_index = Some(hud_text.text_index);
    }
}

pub fn update_interaction_prompt(demo: &HorrorDemo, world: &mut World) {
    let Some(text_index) = demo.interaction_prompt_text_index else {
        return;
    };
    let Some(prompt_entity) = demo.interaction_prompt_entity else {
        return;
    };

    let mouse_pos = world.resources.input.mouse.position;
    let viewport_size = world
        .resources
        .window
        .cached_viewport_size
        .unwrap_or((800, 600));

    if demo.interaction.grabbed_entity.is_some()
        || demo.interaction.manipulated_door_index.is_some()
        || demo.interaction.manipulated_lever_index.is_some()
        || demo.interaction.manipulated_button_index.is_some()
        || demo.reading_note.is_some()
    {
        world.resources.text_cache.set_text(text_index, "");
        if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
            hud_text.dirty = true;
        }
        return;
    }

    let screen_pos = if demo.input_mode == InputMode::Gamepad {
        nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0)
    } else {
        mouse_pos
    };

    let options = PickingOptions {
        max_distance: GRAB_RANGE,
        ignore_invisible: true,
    };

    let pick_results = if demo.input_mode == InputMode::Gamepad {
        pick_entities_cone(demo, world, screen_pos, INTERACT_CONE_RADIUS, options)
    } else {
        pick_entities(world, screen_pos, options)
    };

    let mut prompt_text = "";

    for result in &pick_results {
        if demo.physics_objects.contains(&result.entity) {
            prompt_text = "Grab";
            break;
        }

        for door in &demo.doors {
            if result.entity == door.entity && result.distance <= INTERACT_RANGE {
                prompt_text = if door.locked { "Locked" } else { "Open" };
                break;
            }
        }
        if !prompt_text.is_empty() {
            break;
        }

        for lever in &demo.levers {
            if result.entity == lever.collider_entity && result.distance <= INTERACT_RANGE {
                prompt_text = "Interact";
                break;
            }
        }
        if !prompt_text.is_empty() {
            break;
        }

        for button in &demo.buttons {
            if result.entity == button.entity && result.distance <= INTERACT_RANGE {
                prompt_text = "Press";
                break;
            }
        }
        if !prompt_text.is_empty() {
            break;
        }

        for note in &demo.notes {
            if result.entity == note.entity && result.distance <= INTERACT_RANGE {
                prompt_text = "Read";
                break;
            }
        }
        if !prompt_text.is_empty() {
            break;
        }
    }

    world.resources.text_cache.set_text(text_index, prompt_text);
    if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
        hud_text.dirty = true;
    }
}

pub fn pick_entities_cone(
    demo: &HorrorDemo,
    world: &World,
    center: Vec2,
    radius: f32,
    options: PickingOptions,
) -> Vec<PickingResult> {
    let mut all_results: Vec<PickingResult> = Vec::new();
    let mut seen_entities = std::collections::HashSet::new();

    let offsets = [
        (0.0, 0.0),
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (0.707, 0.707),
        (-0.707, 0.707),
        (0.707, -0.707),
        (-0.707, -0.707),
    ];

    let _ = demo;

    for (offset_x, offset_y) in offsets {
        let screen_pos =
            nalgebra_glm::vec2(center.x + offset_x * radius, center.y + offset_y * radius);
        let results = pick_entities(world, screen_pos, options);
        for result in results {
            if !seen_entities.contains(&result.entity) {
                seen_entities.insert(result.entity);
                all_results.push(result);
            }
        }
    }

    all_results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    all_results
}

pub fn update_objective(demo: &HorrorDemo, world: &mut World) {
    let Some(text_index) = demo.objective_text_index else {
        return;
    };
    let Some(objective_entity) = demo.objective_text_entity else {
        return;
    };

    let objective = if demo.game_won {
        ""
    } else if demo.exit_unlocked {
        "Exit through the door"
    } else if demo.power_restored {
        "Return to main hall and pull the exit lever"
    } else {
        "Find the generator and restore power"
    };

    world.resources.text_cache.set_text(text_index, objective);
    if let Some(hud_text) = world.core.get_text_mut(objective_entity) {
        hud_text.dirty = true;
    }
}

pub fn update_temporary_message(demo: &mut HorrorDemo, world: &mut World) {
    if demo.temporary_message.is_none() {
        return;
    }

    let dt = world.resources.window.timing.delta_time;
    demo.temporary_message_timer -= dt;

    if demo.temporary_message_timer <= 0.0 {
        demo.temporary_message = None;
    }
}

pub fn note_reading_system(demo: &mut HorrorDemo, world: &mut World) {
    let keyboard = &world.resources.input.keyboard;
    let f_pressed = keyboard.is_key_pressed(KeyCode::KeyF);
    let e_pressed = keyboard.is_key_pressed(KeyCode::KeyE);
    let esc_pressed = keyboard.is_key_pressed(KeyCode::Escape);

    let gamepad_rt_pressed = if let Some(gamepad) = query_active_gamepad(world) {
        let rt_axis = gamepad.value(gilrs::Axis::RightZ);
        let rt_button = gamepad.is_pressed(gilrs::Button::RightTrigger2);
        rt_axis > 0.5 || rt_button
    } else {
        false
    };

    let interact_pressed = f_pressed || e_pressed || gamepad_rt_pressed || esc_pressed;

    if !demo.note_close_key_released && !interact_pressed {
        demo.note_close_key_released = true;
    }

    if demo.note_close_key_released && interact_pressed {
        demo.reading_note = None;
    }
}
