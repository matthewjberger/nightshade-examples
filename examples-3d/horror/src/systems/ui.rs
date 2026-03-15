use crate::constants::{GRAB_RANGE, INTERACT_CONE_RADIUS};
use crate::ecs::{GameWorld, INTERACTABLE, InputMode, InteractionKind};
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::picking::{PickingOptions, PickingResult, pick_entities};
use nightshade::ecs::text::commands::spawn_ui_text_with_properties;
use nightshade::ecs::text::components::TextProperties;
use nightshade::prelude::*;

pub fn spawn_ui(game_world: &mut GameWorld, world: &mut World) {
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
    game_world.resources.interaction_prompt_entity = Some(prompt_entity);
    if let Some(hud_text) = world.core.get_text(prompt_entity) {
        game_world.resources.interaction_prompt_text_index = Some(hud_text.text_index);
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
    game_world.resources.objective_text_entity = Some(objective_entity);
    if let Some(hud_text) = world.core.get_text(objective_entity) {
        game_world.resources.objective_text_index = Some(hud_text.text_index);
    }

    build_death_overlay(game_world, world);
    build_temporary_message_overlay(game_world, world);
    build_note_overlay(game_world, world);
    build_win_overlay(game_world, world);
}

fn build_death_overlay(game_world: &mut GameWorld, world: &mut World) {
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

    game_world.resources.death_overlay_entity = Some(overlay);
}

fn build_temporary_message_overlay(game_world: &mut GameWorld, world: &mut World) {
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

    game_world.resources.temporary_message_overlay_entity = Some(overlay);
    game_world.resources.temporary_message_text_entity = Some(text_entity);
}

fn build_note_overlay(game_world: &mut GameWorld, world: &mut World) {
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

    game_world.resources.note_overlay_entity = Some(overlay);
    game_world.resources.note_title_text_entity = Some(title_entity);
    game_world.resources.note_content_text_entity = Some(content_entity);
}

fn build_win_overlay(game_world: &mut GameWorld, world: &mut World) {
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

    game_world.resources.win_overlay_entity = Some(overlay);
    game_world.resources.win_text_entity = Some(text_entity);
}

pub fn update_overlays(game_world: &mut GameWorld, world: &mut World) {
    update_death_overlay(game_world, world);
    update_temporary_message_overlay(game_world, world);
    update_note_overlay(game_world, world);
    update_win_overlay(game_world, world);
}

fn update_death_overlay(game_world: &GameWorld, world: &mut World) {
    let Some(entity) = game_world.resources.death_overlay_entity else {
        return;
    };
    let should_show = game_world.resources.monster.active && game_world.resources.game_won;
    world.ui_set_visible(entity, should_show);
}

fn update_temporary_message_overlay(game_world: &mut GameWorld, world: &mut World) {
    let Some(overlay) = game_world.resources.temporary_message_overlay_entity else {
        return;
    };

    if let Some(message) = &game_world.resources.temporary_message {
        world.ui_set_visible(overlay, true);
        if game_world.resources.last_shown_message.as_deref() != Some(message) {
            if let Some(text_entity) = game_world.resources.temporary_message_text_entity {
                world.ui_set_text(text_entity, message);
            }
            game_world.resources.last_shown_message = Some(message.clone());
        }
    } else {
        world.ui_set_visible(overlay, false);
        if game_world.resources.last_shown_message.is_some() {
            game_world.resources.last_shown_message = None;
        }
    }
}

fn update_note_overlay(game_world: &mut GameWorld, world: &mut World) {
    let Some(overlay) = game_world.resources.note_overlay_entity else {
        return;
    };

    if let Some(note_game_entity) = game_world.resources.reading_note {
        world.ui_set_visible(overlay, true);
        if game_world.resources.last_shown_note != Some(note_game_entity) {
            if let Some(note) = game_world.get_note(note_game_entity) {
                let title = note.title.clone();
                let content = note.content.clone();
                if let Some(title_entity) = game_world.resources.note_title_text_entity {
                    world.ui_set_text(title_entity, &title);
                }
                if let Some(content_entity) = game_world.resources.note_content_text_entity {
                    world.ui_set_text(content_entity, &content);
                }
            }
            game_world.resources.last_shown_note = Some(note_game_entity);
        }
    } else {
        world.ui_set_visible(overlay, false);
        if game_world.resources.last_shown_note.is_some() {
            game_world.resources.last_shown_note = None;
        }
    }
}

fn update_win_overlay(game_world: &GameWorld, world: &mut World) {
    let Some(overlay) = game_world.resources.win_overlay_entity else {
        return;
    };

    let should_show = game_world.resources.game_won
        && !game_world.resources.monster.active
        && game_world.resources.fade_amount > 0.01;
    world.ui_set_visible(overlay, should_show);

    if should_show {
        let fade_alpha = game_world.resources.fade_amount;
        if let Some(color) = world.ui.get_ui_node_color_mut(overlay) {
            let black_with_alpha = nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, fade_alpha);
            color.colors[0] = Some(black_with_alpha);
            color.computed_color = black_with_alpha;
        }

        if let Some(text_entity) = game_world.resources.win_text_entity {
            let text_alpha = if game_world.resources.fade_amount > 0.8 {
                ((game_world.resources.fade_amount - 0.8) / 0.2).min(1.0)
            } else {
                0.0
            };
            if let Some(color) = world.ui.get_ui_node_color_mut(text_entity) {
                let white_with_alpha = nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, text_alpha);
                color.colors[0] = Some(white_with_alpha);
                color.computed_color = white_with_alpha;
            }
        }
    }
}

pub fn update_interaction_prompt(game_world: &GameWorld, world: &mut World) {
    let Some(text_index) = game_world.resources.interaction_prompt_text_index else {
        return;
    };
    let Some(prompt_entity) = game_world.resources.interaction_prompt_entity else {
        return;
    };

    if game_world.resources.interaction.grabbed_entity.is_some()
        || game_world.resources.interaction.manipulated_door.is_some()
        || game_world.resources.interaction.manipulated_lever.is_some()
        || game_world
            .resources
            .interaction
            .manipulated_button
            .is_some()
        || game_world.resources.reading_note.is_some()
    {
        world.resources.text_cache.set_text(text_index, "");
        if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
            hud_text.dirty = true;
        }
        return;
    }

    let viewport_size = world
        .resources
        .window
        .cached_viewport_size
        .unwrap_or((800, 600));
    let screen_pos = nalgebra_glm::vec2(viewport_size.0 as f32 / 2.0, viewport_size.1 as f32 / 2.0);

    let options = PickingOptions {
        max_distance: GRAB_RANGE,
        ignore_invisible: true,
    };

    let pick_results = if game_world.resources.input_mode == InputMode::Gamepad {
        pick_entities_cone(world, screen_pos, INTERACT_CONE_RADIUS, options)
    } else {
        pick_entities(world, screen_pos, options)
    };

    let mut prompt_text = "";

    for result in &pick_results {
        let matched = game_world
            .query_entities(INTERACTABLE)
            .find(|&game_entity| {
                game_world
                    .get_interactable(game_entity)
                    .is_some_and(|interactable| {
                        interactable.match_entity == result.entity
                            && (interactable.range == 0.0 || result.distance <= interactable.range)
                    })
            });

        if let Some(game_entity) = matched {
            let kind = game_world
                .get_interactable(game_entity)
                .map(|interactable| interactable.kind)
                .unwrap_or_default();

            prompt_text = match kind {
                InteractionKind::Grab => "Grab",
                InteractionKind::Door => {
                    if game_world
                        .get_door(game_entity)
                        .is_some_and(|door| door.locked)
                    {
                        "Locked"
                    } else {
                        "Open"
                    }
                }
                InteractionKind::Lever => "Interact",
                InteractionKind::Button => "Press",
                InteractionKind::Note => "Read",
            };
            break;
        }
    }

    world.resources.text_cache.set_text(text_index, prompt_text);
    if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
        hud_text.dirty = true;
    }
}

pub fn pick_entities_cone(
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

pub fn update_objective(game_world: &GameWorld, world: &mut World) {
    let Some(text_index) = game_world.resources.objective_text_index else {
        return;
    };
    let Some(objective_entity) = game_world.resources.objective_text_entity else {
        return;
    };

    let objective = if game_world.resources.game_won {
        ""
    } else if game_world.resources.exit_unlocked {
        "Exit through the door"
    } else if game_world.resources.power_restored {
        "Return to main hall and pull the exit lever"
    } else {
        "Find the generator and restore power"
    };

    world.resources.text_cache.set_text(text_index, objective);
    if let Some(hud_text) = world.core.get_text_mut(objective_entity) {
        hud_text.dirty = true;
    }
}

pub fn update_temporary_message(game_world: &mut GameWorld, world: &mut World) {
    if game_world.resources.temporary_message.is_none() {
        return;
    }

    let dt = world.resources.window.timing.delta_time;
    game_world.resources.temporary_message_timer -= dt;

    if game_world.resources.temporary_message_timer <= 0.0 {
        game_world.resources.temporary_message = None;
    }
}

pub fn note_reading_system(game_world: &mut GameWorld, world: &mut World) {
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

    if !game_world.resources.note_close_key_released && !interact_pressed {
        game_world.resources.note_close_key_released = true;
    }

    if game_world.resources.note_close_key_released && interact_pressed {
        game_world.resources.reading_note = None;
    }
}
