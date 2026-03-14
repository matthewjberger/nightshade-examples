use crate::constants::{GRAB_RANGE, INTERACT_CONE_RADIUS, INTERACT_RANGE};
use crate::state::{HorrorDemo, InputMode};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::picking::{PickingOptions, PickingResult, pick_entities};
use nightshade::ecs::text::commands::spawn_ui_text_with_properties;
use nightshade::ecs::text::components::TextProperties;
use nightshade::prelude::*;

pub fn spawn_ui(demo: &mut HorrorDemo, world: &mut World) {
    world.resources.retained_ui.enabled = true;

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

    build_death_overlay(demo, world);
    build_temporary_message_overlay(demo, world);
    build_note_overlay(demo, world);
    build_win_overlay(demo, world);
}

fn build_death_overlay(demo: &mut HorrorDemo, world: &mut World) {
    let mut tree = UiTreeBuilder::new(world);

    let overlay = tree
        .add_node()
        .boundary(
            Vp(nalgebra_glm::Vec2::new(50.0, 50.0)) + Ab(nalgebra_glm::Vec2::new(-150.0, -50.0)),
            Vp(nalgebra_glm::Vec2::new(50.0, 50.0)) + Ab(nalgebra_glm::Vec2::new(150.0, 50.0)),
        )
        .with_rect(8.0, 2.0, nalgebra_glm::Vec4::new(0.784, 0.0, 0.0, 1.0))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.314, 0.0, 0.0, 0.902))
        .with_visible(false)
        .without_pointer_events()
        .entity();

    tree.push_parent(overlay);

    tree.add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_text("YOU DIED", 32.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(1.0, 0.196, 0.196, 1.0))
        .without_pointer_events();

    tree.pop_parent();
    tree.finish();

    demo.death_overlay_entity = Some(overlay);
}

fn build_temporary_message_overlay(demo: &mut HorrorDemo, world: &mut World) {
    let mut tree = UiTreeBuilder::new(world);

    let overlay = tree
        .add_node()
        .boundary(
            Vp(nalgebra_glm::Vec2::new(50.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(-200.0, -120.0)),
            Vp(nalgebra_glm::Vec2::new(50.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(200.0, -60.0)),
        )
        .with_rect(6.0, 1.0, nalgebra_glm::Vec4::new(0.471, 0.392, 0.314, 1.0))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.118, 0.078, 0.059, 0.863))
        .with_visible(false)
        .without_pointer_events()
        .entity();

    tree.push_parent(overlay);

    let text_entity = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(15.0, 10.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(-15.0, -10.0)),
        )
        .with_text("", 18.0)
        .with_text_wrap()
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.863, 0.784, 0.627, 1.0))
        .without_pointer_events()
        .done();

    tree.pop_parent();
    tree.finish();

    demo.temporary_message_overlay_entity = Some(overlay);
    demo.temporary_message_text_entity = Some(text_entity);
}

fn build_note_overlay(demo: &mut HorrorDemo, world: &mut World) {
    let mut tree = UiTreeBuilder::new(world);

    let panel_width = 500.0;
    let panel_height = 400.0;

    let overlay = tree
        .add_node()
        .boundary(
            Vp(nalgebra_glm::Vec2::new(50.0, 50.0))
                + Ab(nalgebra_glm::Vec2::new(
                    -panel_width / 2.0,
                    -panel_height / 2.0,
                )),
            Vp(nalgebra_glm::Vec2::new(50.0, 50.0))
                + Ab(nalgebra_glm::Vec2::new(
                    panel_width / 2.0,
                    panel_height / 2.0,
                )),
        )
        .with_rect(6.0, 2.0, nalgebra_glm::Vec4::new(0.314, 0.235, 0.157, 1.0))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.078, 0.059, 0.039, 0.961))
        .with_visible(false)
        .without_pointer_events()
        .with_clip()
        .entity();

    tree.push_parent(overlay);

    let title_entity = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-20.0, 50.0)),
        )
        .with_text("", 20.0)
        .with_text_wrap()
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Top)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.863, 0.784, 0.627, 1.0))
        .without_pointer_events()
        .done();

    tree.add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 56.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-20.0, 57.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.3, 0.25, 0.2, 0.5))
        .without_pointer_events();

    let content_entity = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 70.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(-20.0, -20.0)),
        )
        .with_text("", 16.0)
        .with_text_wrap()
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.784, 0.745, 0.667, 1.0))
        .without_pointer_events()
        .done();

    tree.pop_parent();
    tree.finish();

    demo.note_overlay_entity = Some(overlay);
    demo.note_title_text_entity = Some(title_entity);
    demo.note_content_text_entity = Some(content_entity);
}

fn build_win_overlay(demo: &mut HorrorDemo, world: &mut World) {
    let mut tree = UiTreeBuilder::new(world);

    let overlay = tree
        .add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_visible(false)
        .without_pointer_events()
        .entity();

    tree.push_parent(overlay);

    let text_entity = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_text("You Survived", 48.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 0.0))
        .without_pointer_events()
        .done();

    tree.pop_parent();
    tree.finish();

    demo.win_overlay_entity = Some(overlay);
    demo.win_text_entity = Some(text_entity);
}

pub fn update_overlays(demo: &mut HorrorDemo, world: &mut World) {
    update_death_overlay(demo, world);
    update_temporary_message_overlay(demo, world);
    update_note_overlay(demo, world);
    update_win_overlay(demo, world);
}

fn update_death_overlay(demo: &HorrorDemo, world: &mut World) {
    let Some(entity) = demo.death_overlay_entity else {
        return;
    };
    let should_show = demo.monster.active && demo.game_won;
    world.ui_set_visible(entity, should_show);
}

fn update_temporary_message_overlay(demo: &mut HorrorDemo, world: &mut World) {
    let Some(overlay) = demo.temporary_message_overlay_entity else {
        return;
    };

    if let Some(message) = &demo.temporary_message {
        world.ui_set_visible(overlay, true);
        if demo.last_shown_message.as_deref() != Some(message) {
            if let Some(text_entity) = demo.temporary_message_text_entity {
                world.ui_set_text(text_entity, message);
            }
            demo.last_shown_message = Some(message.clone());
        }
    } else {
        world.ui_set_visible(overlay, false);
        if demo.last_shown_message.is_some() {
            demo.last_shown_message = None;
        }
    }
}

fn update_note_overlay(demo: &mut HorrorDemo, world: &mut World) {
    let Some(overlay) = demo.note_overlay_entity else {
        return;
    };

    if let Some(note_index) = demo.reading_note {
        world.ui_set_visible(overlay, true);
        if demo.last_shown_note != Some(note_index) {
            let note = &demo.notes[note_index];
            if let Some(title_entity) = demo.note_title_text_entity {
                world.ui_set_text(title_entity, &note.title);
            }
            if let Some(content_entity) = demo.note_content_text_entity {
                world.ui_set_text(content_entity, &note.content);
            }
            demo.last_shown_note = Some(note_index);
        }
    } else {
        world.ui_set_visible(overlay, false);
        if demo.last_shown_note.is_some() {
            demo.last_shown_note = None;
        }
    }
}

fn update_win_overlay(demo: &HorrorDemo, world: &mut World) {
    let Some(overlay) = demo.win_overlay_entity else {
        return;
    };

    let should_show = demo.game_won && !demo.monster.active && demo.fade_amount > 0.01;
    world.ui_set_visible(overlay, should_show);

    if should_show {
        let fade_alpha = demo.fade_amount;
        if let Some(color) = world.ui.get_ui_node_color_mut(overlay) {
            color.computed_color = nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, fade_alpha);
        }

        if let Some(text_entity) = demo.win_text_entity {
            let text_alpha = if demo.fade_amount > 0.8 {
                ((demo.fade_amount - 0.8) / 0.2).min(1.0)
            } else {
                0.0
            };
            if let Some(color) = world.ui.get_ui_node_color_mut(text_entity) {
                color.computed_color = nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, text_alpha);
            }
        }
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
