use crate::state::{GameScreen, LevelDemo};
use crate::systems::dialogue::select_dialogue_choice;
use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::prelude::*;

pub fn title_screen_ui(demo: &mut LevelDemo, world: &mut World, ctx: &egui::Context) {
    let gamepad_start = query_active_gamepad(world)
        .map(|g| g.is_pressed(gilrs::Button::South) || g.is_pressed(gilrs::Button::Start))
        .unwrap_or(false);

    let keyboard_start = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Enter)
        || world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Space);

    if gamepad_start || keyboard_start {
        demo.screen = GameScreen::Loading;
        return;
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_rgb(20, 20, 25)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.3);

                ui.heading(
                    egui::RichText::new("LEVEL DEMO")
                        .size(48.0)
                        .color(egui::Color32::WHITE),
                );

                ui.add_space(20.0);

                ui.label(
                    egui::RichText::new("A Nightshade Engine Demo")
                        .size(18.0)
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(60.0);

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Start Game").size(24.0))
                            .min_size(egui::vec2(200.0, 50.0)),
                    )
                    .clicked()
                {
                    demo.screen = GameScreen::Loading;
                }

                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Press A/Enter/Space to start")
                        .size(14.0)
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(20.0);

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Quit").size(20.0))
                            .min_size(egui::vec2(150.0, 40.0)),
                    )
                    .clicked()
                {
                    std::process::exit(0);
                }
            });
        });
}

pub fn pause_menu_ui(demo: &mut LevelDemo, world: &mut World, ctx: &egui::Context) {
    let gamepad_resume = query_active_gamepad(world)
        .map(|g| g.is_pressed(gilrs::Button::East) || g.is_pressed(gilrs::Button::Start))
        .unwrap_or(false);

    if gamepad_resume {
        demo.screen = GameScreen::Gameplay;
        return;
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.3);

                ui.heading(
                    egui::RichText::new("PAUSED")
                        .size(36.0)
                        .color(egui::Color32::WHITE),
                );

                ui.add_space(40.0);

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Resume").size(20.0))
                            .min_size(egui::vec2(150.0, 40.0)),
                    )
                    .clicked()
                {
                    demo.screen = GameScreen::Gameplay;
                }

                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("Press B/Start to resume")
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(15.0);

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Main Menu").size(20.0))
                            .min_size(egui::vec2(150.0, 40.0)),
                    )
                    .clicked()
                {
                    demo.screen = GameScreen::Title;
                }

                ui.add_space(15.0);

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Quit").size(20.0))
                            .min_size(egui::vec2(150.0, 40.0)),
                    )
                    .clicked()
                {
                    std::process::exit(0);
                }
            });
        });
}

pub fn gameplay_ui(demo: &mut LevelDemo, world: &mut World, ctx: &egui::Context) {
    if demo.dialogue.active {
        dialogue_ui(demo, ctx);
        return;
    }

    crosshair_ui(ctx);

    egui::Window::new("Debug")
        .default_pos([10.0, 10.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "FPS: {:.1}",
                world.resources.window.timing.frames_per_second
            ));
            ui.label(format!("Entities: {}", demo.spawned_entities.len()));

            ui.separator();

            if demo.fly_mode {
                ui.label("Fly Mode: WASD + right-click to look");
            } else {
                ui.label("Walk Mode: WASD, Space=jump, C=crouch");
                ui.label("Q/E=lean, F=flashlight, E=interact");
            }

            ui.separator();

            if ui.checkbox(&mut demo.fly_mode, "Fly Mode").changed() {
                toggle_fly_mode(demo, world);
            }

            if ui
                .checkbox(&mut demo.show_collision, "Show Collision")
                .changed()
            {
                world.resources.physics.debug_draw = demo.show_collision;
            }

            if ui
                .checkbox(&mut demo.show_navmesh, "Show NavMesh")
                .changed()
            {
                set_navmesh_debug_draw(world, demo.show_navmesh);
            }

            if ui.checkbox(&mut demo.unlit, "Unlit").changed() {
                for material in world
                    .resources
                    .material_registry
                    .registry
                    .entries
                    .iter_mut()
                    .flatten()
                {
                    material.unlit = demo.unlit;
                }
            }
        });
}

fn crosshair_ui(ctx: &egui::Context) {
    #[allow(deprecated)]
    let screen_rect = ctx.screen_rect();
    let center = screen_rect.center();

    egui::Area::new(egui::Id::new("crosshair"))
        .fixed_pos(center - egui::vec2(10.0, 10.0))
        .show(ctx, |ui| {
            let painter = ui.painter();
            let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180);
            let stroke = egui::Stroke::new(2.0, color);

            painter.line_segment(
                [
                    egui::pos2(center.x - 8.0, center.y),
                    egui::pos2(center.x - 3.0, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(center.x + 3.0, center.y),
                    egui::pos2(center.x + 8.0, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(center.x, center.y - 8.0),
                    egui::pos2(center.x, center.y - 3.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(center.x, center.y + 3.0),
                    egui::pos2(center.x, center.y + 8.0),
                ],
                stroke,
            );
        });
}

fn dialogue_ui(demo: &mut LevelDemo, ctx: &egui::Context) {
    if demo.dialogue.current_node >= demo.dialogue.nodes.len() {
        return;
    }

    let current_line = demo.dialogue.current_line;
    let lines: Vec<_> = demo.dialogue.nodes[demo.dialogue.current_node]
        .lines
        .iter()
        .map(|line| (line.speaker.clone(), line.text.clone()))
        .collect();
    let choices: Vec<_> = demo.dialogue.nodes[demo.dialogue.current_node]
        .choices
        .iter()
        .map(|choice| choice.text.clone())
        .collect();

    let mut selected_choice: Option<usize> = None;

    egui::TopBottomPanel::bottom("dialogue_panel")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 230))
                .inner_margin(egui::Margin::same(20)),
        )
        .min_height(150.0)
        .show(ctx, |ui| {
            if current_line < lines.len() {
                let (speaker, text) = &lines[current_line];

                ui.label(
                    egui::RichText::new(speaker)
                        .size(16.0)
                        .color(egui::Color32::YELLOW)
                        .strong(),
                );

                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new(text)
                        .size(18.0)
                        .color(egui::Color32::WHITE),
                );

                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("[Space/Click to continue]")
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
            } else if !choices.is_empty() {
                ui.label(
                    egui::RichText::new("Choose a response:")
                        .size(16.0)
                        .color(egui::Color32::YELLOW)
                        .strong(),
                );

                ui.add_space(10.0);

                for (index, choice_text) in choices.iter().enumerate() {
                    let button_text = format!("{}. {}", index + 1, choice_text);
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(&button_text).size(16.0))
                                .min_size(egui::vec2(300.0, 30.0)),
                        )
                        .clicked()
                    {
                        selected_choice = Some(index);
                    }
                }
            }
        });

    if let Some(choice_index) = selected_choice {
        select_dialogue_choice(demo, choice_index);
    }
}

fn toggle_fly_mode(demo: &mut LevelDemo, world: &mut World) {
    if demo.fly_mode {
        if let Some(fly_camera) = demo.fly_camera {
            if let Some(player_camera) = demo.camera_entity {
                let pos = world
                    .get_global_transform(player_camera)
                    .map(|t| t.translation())
                    .unwrap_or(Vec3::zeros());
                if let Some(fly_transform) = world.get_local_transform_mut(fly_camera) {
                    fly_transform.translation = pos;
                }
            }
            world.resources.active_camera = Some(fly_camera);
        }
    } else if let Some(player_camera) = demo.camera_entity {
        if let Some(fly_camera) = demo.fly_camera {
            let pos = world
                .get_global_transform(fly_camera)
                .map(|t| t.translation())
                .unwrap_or(Vec3::zeros());
            if let Some(player_entity) = demo.player_entity {
                if let Some(player_transform) = world.get_local_transform_mut(player_entity) {
                    player_transform.translation = pos - Vec3::new(0.0, 1.3, 0.0);
                }
                if let Some(controller) = world.get_character_controller_mut(player_entity) {
                    controller.velocity = Vec3::zeros();
                }
            }
        }
        world.resources.active_camera = Some(player_camera);
    }
}
