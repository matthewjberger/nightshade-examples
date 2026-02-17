use nightshade::prelude::*;

use crate::agent::{AgentState, CarriedItem};
use crate::ecs::GameWorld;

const COLOR_IDLE: egui::Color32 = egui::Color32::from_rgb(120, 120, 120);
const COLOR_MOVING: egui::Color32 = egui::Color32::from_rgb(80, 140, 220);
const COLOR_PERFORMING: egui::Color32 = egui::Color32::from_rgb(60, 190, 80);
const COLOR_REPLANNING: egui::Color32 = egui::Color32::from_rgb(220, 50, 220);
const COLOR_STEP_DONE: egui::Color32 = egui::Color32::from_rgb(90, 90, 90);
const COLOR_STEP_CURRENT: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
const COLOR_STEP_PENDING: egui::Color32 = egui::Color32::from_rgb(160, 160, 160);
const COLOR_DESTROYED: egui::Color32 = egui::Color32::from_rgb(200, 50, 50);

pub fn draw_ui(game: &mut GameWorld, _world: &mut World, ui_context: &egui::Context) {
    draw_status_panel(game, ui_context);
    draw_agent_panel(game, ui_context);
    draw_controls(game, ui_context);

    let failure_triggered = game.resources.failure_triggered;
    if failure_triggered {
        draw_failure_overlay(game, ui_context);
    }
}

fn draw_status_panel(game: &GameWorld, ui_context: &egui::Context) {
    egui::SidePanel::left("castle_status")
        .default_width(240.0)
        .show(ui_context, |ui| {
            ui.heading("Castle Siege");
            ui.separator();

            let minutes = (game.resources.elapsed_time / 60.0) as u32;
            let seconds = (game.resources.elapsed_time % 60.0) as u32;
            let act_color = match game.resources.bombardment.current_act {
                crate::bombardment::Act::Warmup => egui::Color32::from_rgb(100, 180, 100),
                crate::bombardment::Act::Escalation => egui::Color32::from_rgb(220, 180, 50),
                crate::bombardment::Act::Crescendo => egui::Color32::from_rgb(220, 60, 60),
            };
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{:02}:{:02}", minutes, seconds))
                        .strong()
                        .size(16.0),
                );
                ui.label(
                    egui::RichText::new(format!("{}", game.resources.bombardment.current_act))
                        .color(act_color)
                        .strong(),
                );
            });
            ui.label(format!(
                "Boulders fired: {}",
                game.resources.bombardment.total_boulders_fired
            ));

            ui.add_space(6.0);
            ui.separator();
            ui.label(egui::RichText::new("Defenses").strong());
            ui.add_space(2.0);

            let wall_names = ["North", "South", "East", "West"];
            for (wall_index, wall) in game.resources.castle.walls.iter().enumerate() {
                let total_hp: f32 = wall.segments.iter().map(|segment| segment.health).sum();
                let max_hp: f32 = wall.segments.iter().map(|segment| segment.max_health).sum();
                let ratio = if max_hp > 0.0 { total_hp / max_hp } else { 0.0 };
                let breached = wall.segments.iter().any(|segment| segment.breached);

                let color = lerp_color(
                    egui::Color32::from_rgb(200, 50, 50),
                    egui::Color32::from_rgb(50, 200, 50),
                    ratio,
                );

                ui.horizontal(|ui| {
                    let label = if breached {
                        format!("{} BREACH", wall_names[wall_index])
                    } else {
                        wall_names[wall_index].to_string()
                    };
                    ui.label(
                        egui::RichText::new(format!("{:6}", label))
                            .color(if breached {
                                COLOR_DESTROYED
                            } else {
                                egui::Color32::WHITE
                            })
                            .monospace(),
                    );
                    ui.add(
                        egui::ProgressBar::new(ratio)
                            .fill(color)
                            .desired_width(100.0),
                    );
                });
            }

            let gate_ratio =
                game.resources.castle.gate_health / game.resources.castle.gate_max_health;
            let gate_color = lerp_color(
                egui::Color32::from_rgb(200, 50, 50),
                egui::Color32::from_rgb(180, 140, 60),
                gate_ratio,
            );
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Gate  ").monospace());
                ui.add(
                    egui::ProgressBar::new(gate_ratio)
                        .fill(gate_color)
                        .desired_width(100.0),
                );
            });

            ui.add_space(6.0);
            ui.separator();
            ui.label(egui::RichText::new("Threats").strong());
            ui.add_space(2.0);

            let fire_count = game.resources.fires.len();
            let rubble_count = game.resources.rubble_list.len();
            let boulder_count = game.resources.boulders.len();
            if fire_count > 0 {
                ui.label(
                    egui::RichText::new(format!(
                        "  {} active fire{}",
                        fire_count,
                        if fire_count != 1 { "s" } else { "" }
                    ))
                    .color(egui::Color32::from_rgb(255, 140, 40)),
                );
            }
            if rubble_count > 0 {
                ui.label(format!(
                    "  {} rubble pile{}",
                    rubble_count,
                    if rubble_count != 1 { "s" } else { "" }
                ));
            }
            if boulder_count > 0 {
                ui.label(
                    egui::RichText::new(format!(
                        "  {} incoming boulder{}",
                        boulder_count,
                        if boulder_count != 1 { "s" } else { "" }
                    ))
                    .color(egui::Color32::from_rgb(200, 200, 60)),
                );
            }
            if fire_count == 0 && rubble_count == 0 && boulder_count == 0 {
                ui.label(
                    egui::RichText::new("  None").color(egui::Color32::from_rgb(100, 100, 100)),
                );
            }

            ui.add_space(6.0);
            ui.separator();
            ui.label(egui::RichText::new("Supplies").strong());
            ui.add_space(2.0);

            draw_supply_row(
                ui,
                "Well",
                &format!("{:.0}%", game.resources.castle.well_water_remaining),
                game.resources.castle.well_destroyed,
                egui::Color32::from_rgb(80, 140, 220),
            );
            draw_supply_row(
                ui,
                "Armory",
                &format!("{} arrows", game.resources.castle.armory_stock),
                !game.resources.castle.armory_exists,
                egui::Color32::from_rgb(160, 160, 170),
            );
            draw_supply_row(
                ui,
                "Repairs",
                &format!("{} materials", game.resources.castle.repair_pile_count),
                false,
                egui::Color32::from_rgb(180, 160, 100),
            );
            draw_supply_row(
                ui,
                "Healing",
                if game.resources.castle.healing_station_exists {
                    "Active"
                } else {
                    "---"
                },
                !game.resources.castle.healing_station_exists,
                egui::Color32::from_rgb(80, 200, 120),
            );

            let stocked = game
                .resources
                .castle
                .archer_posts
                .iter()
                .filter(|post| post.arrows_remaining > 0)
                .count();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("  Archers").monospace());
                ui.label(format!("{}/4 stocked", stocked));
            });

            ui.add_space(6.0);
            ui.separator();
            ui.label(egui::RichText::new("GOAP World State").strong());
            ui.add_space(2.0);
            draw_world_state_flags(game, ui);

            ui.add_space(6.0);
            ui.separator();
            let alive = game.resources.agents.len();
            let wounded = game
                .resources
                .agents
                .iter()
                .filter(|&&entity| game.get_agent(entity).is_some_and(|agent| agent.wounded))
                .count();
            ui.label(format!("{} defenders, {} wounded", alive, wounded));
        });
}

fn draw_supply_row(
    ui: &mut egui::Ui,
    name: &str,
    value: &str,
    destroyed: bool,
    color: egui::Color32,
) {
    ui.horizontal(|ui| {
        if destroyed {
            ui.label(
                egui::RichText::new(format!("  {} DESTROYED", name))
                    .color(COLOR_DESTROYED)
                    .monospace(),
            );
        } else {
            ui.label(egui::RichText::new(format!("  {}", name)).monospace());
            ui.label(egui::RichText::new(value).color(color));
        }
    });
}

fn draw_world_state_flags(game: &GameWorld, ui: &mut egui::Ui) {
    let flags: &[(&str, bool)] = &[
        (
            "Well has water",
            !game.resources.castle.well_destroyed
                && game.resources.castle.well_water_remaining > 0.0,
        ),
        ("Armory exists", game.resources.castle.armory_exists),
        (
            "Healing exists",
            game.resources.castle.healing_station_exists,
        ),
        ("Repair pile", game.resources.castle.repair_pile_count > 0),
        ("Back gate intact", game.resources.castle.back_gate_intact),
        ("River accessible", game.resources.castle.river_accessible),
        ("Rubble present", !game.resources.rubble_list.is_empty()),
    ];

    for (name, value) in flags {
        let (icon, color) = if *value {
            ("T", egui::Color32::from_rgb(80, 180, 80))
        } else {
            ("F", egui::Color32::from_rgb(180, 60, 60))
        };
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("  [{}]", icon))
                    .color(color)
                    .monospace(),
            );
            ui.label(egui::RichText::new(*name).color(if *value {
                egui::Color32::from_rgb(180, 180, 180)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            }));
        });
    }
}

fn draw_agent_panel(game: &mut GameWorld, ui_context: &egui::Context) {
    egui::SidePanel::right("agent_panel")
        .default_width(320.0)
        .show(ui_context, |ui| {
            ui.heading("GOAP Agent Plans");
            ui.label(
                egui::RichText::new("Backwards-chaining A* planner")
                    .color(egui::Color32::from_rgb(140, 140, 140))
                    .size(11.0),
            );
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (agent_index, &entity) in game.resources.agents.iter().enumerate() {
                    if let Some(agent) = game.get_agent(entity) {
                        draw_agent_card(ui, agent, agent_index);
                        ui.add_space(4.0);
                    }
                }
            });
        });
}

fn draw_agent_card(ui: &mut egui::Ui, agent: &crate::agent::Agent, agent_index: usize) {
    let state_color = match agent.state {
        AgentState::Idle => COLOR_IDLE,
        AgentState::Moving => COLOR_MOVING,
        AgentState::Performing => COLOR_PERFORMING,
        AgentState::Replanning => COLOR_REPLANNING,
    };

    let bg_color = if agent.state == AgentState::Replanning {
        egui::Color32::from_rgba_premultiplied(80, 0, 80, 40)
    } else {
        egui::Color32::from_rgba_premultiplied(40, 40, 50, 80)
    };

    egui::Frame::default()
        .fill(bg_color)
        .inner_margin(6.0)
        .corner_radius(4.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 70)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&agent.name).strong().size(13.0));

                let state_text = match agent.state {
                    AgentState::Idle => "IDLE",
                    AgentState::Moving => "MOVING",
                    AgentState::Performing => "ACTING",
                    AgentState::Replanning => "REPLAN",
                };
                ui.label(
                    egui::RichText::new(state_text)
                        .color(state_color)
                        .strong()
                        .size(11.0),
                );

                if agent.wounded {
                    ui.label(
                        egui::RichText::new("WOUNDED")
                            .color(egui::Color32::from_rgb(220, 80, 80))
                            .size(10.0),
                    );
                }

                if let Some(item) = &agent.carrying {
                    let item_color = match item {
                        CarriedItem::Water => egui::Color32::from_rgb(80, 140, 220),
                        CarriedItem::RepairMaterials => egui::Color32::from_rgb(180, 160, 100),
                        CarriedItem::Arrows => egui::Color32::from_rgb(160, 110, 60),
                    };
                    ui.label(
                        egui::RichText::new(format!("[{}]", item))
                            .color(item_color)
                            .size(10.0),
                    );
                }
            });

            if let Some(goal) = agent.current_goal {
                let [red, green, blue] = goal.color();
                let goal_color = egui::Color32::from_rgb(red, green, blue);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("GOAL:")
                            .color(egui::Color32::from_rgb(140, 140, 140))
                            .size(10.0),
                    );
                    ui.label(
                        egui::RichText::new(format!("{}", goal))
                            .color(goal_color)
                            .strong()
                            .size(12.0),
                    );
                });
            } else {
                ui.label(
                    egui::RichText::new("No goal — waiting for threats")
                        .color(egui::Color32::from_rgb(100, 100, 100))
                        .italics()
                        .size(10.0),
                );
            }

            if !agent.current_plan.is_empty() {
                ui.add_space(2.0);

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    ui.label(
                        egui::RichText::new("Plan:")
                            .color(egui::Color32::from_rgb(140, 140, 140))
                            .size(10.0),
                    );

                    for (step_index, step) in agent.current_plan.iter().enumerate() {
                        if step_index > 0 {
                            ui.label(
                                egui::RichText::new("->")
                                    .color(egui::Color32::from_rgb(80, 80, 80))
                                    .size(10.0),
                            );
                        }

                        let (color, prefix) = if step_index < agent.current_step {
                            (COLOR_STEP_DONE, "")
                        } else if step_index == agent.current_step {
                            (COLOR_STEP_CURRENT, "")
                        } else {
                            (COLOR_STEP_PENDING, "")
                        };

                        let action_label = format_action_name(step.action.name);

                        let text = if step_index == agent.current_step {
                            egui::RichText::new(format!("{}{}", prefix, action_label))
                                .color(color)
                                .strong()
                                .size(11.0)
                                .underline()
                        } else if step_index < agent.current_step {
                            egui::RichText::new(format!("{}{}", prefix, action_label))
                                .color(color)
                                .strikethrough()
                                .size(10.0)
                        } else {
                            egui::RichText::new(format!("{}{}", prefix, action_label))
                                .color(color)
                                .size(10.0)
                        };

                        ui.label(text);
                    }
                });

                if agent.state == AgentState::Performing
                    && agent.current_step < agent.current_plan.len()
                {
                    let current_action = &agent.current_plan[agent.current_step];
                    let progress = agent.action_progress / current_action.action.duration;
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "  {} {:.1}s/{:.1}s",
                                format_action_name(current_action.action.name),
                                agent.action_progress,
                                current_action.action.duration,
                            ))
                            .color(COLOR_PERFORMING)
                            .size(10.0),
                        );
                    });
                    ui.add(
                        egui::ProgressBar::new(progress.clamp(0.0, 1.0))
                            .fill(COLOR_PERFORMING)
                            .desired_height(4.0),
                    );
                }

                if agent.state == AgentState::Moving
                    && let Some(target_pos) = agent.target_position
                {
                    let dist = nalgebra_glm::distance(&agent.position, &target_pos);
                    let target_name = if agent.current_step < agent.current_plan.len() {
                        format_action_target(agent.current_plan[agent.current_step].action.target)
                    } else {
                        "?".to_string()
                    };
                    ui.label(
                        egui::RichText::new(format!(
                            "  Walking to {} ({:.1}m away)",
                            target_name, dist
                        ))
                        .color(COLOR_MOVING)
                        .size(10.0),
                    );
                }
            }

            if agent.state == AgentState::Replanning && !agent.replan_reason.is_empty() {
                ui.label(
                    egui::RichText::new(format!("  REPLAN: {}", agent.replan_reason))
                        .color(COLOR_REPLANNING)
                        .strong()
                        .size(11.0),
                );
            }

            if agent.plan_generation > 0 {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("#{} plans generated", agent.plan_generation))
                            .color(egui::Color32::from_rgb(80, 80, 80))
                            .size(9.0),
                    );
                    if agent_index < 6 {
                        ui.label(
                            egui::RichText::new(format!("HP:{:.0}", agent.health))
                                .color(if agent.wounded {
                                    egui::Color32::from_rgb(200, 80, 80)
                                } else {
                                    egui::Color32::from_rgb(80, 80, 80)
                                })
                                .size(9.0),
                        );
                    }
                });
            }
        });
}

fn draw_controls(game: &mut GameWorld, ui_context: &egui::Context) {
    egui::TopBottomPanel::bottom("controls")
        .min_height(44.0)
        .show(ui_context, |ui| {
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(
                    egui::Slider::new(&mut game.resources.game_speed, 0.25..=4.0)
                        .step_by(0.25)
                        .suffix("x"),
                );

                ui.separator();

                if ui.button("Fire Boulder").clicked() {
                    game.resources.manual_boulder_requested = true;
                }

                if ui
                    .button(
                        egui::RichText::new("Burn Armory")
                            .color(egui::Color32::from_rgb(220, 120, 50)),
                    )
                    .clicked()
                {
                    game.resources.manual_burn_armory = true;
                }

                if ui
                    .button(
                        egui::RichText::new("Drain Well")
                            .color(egui::Color32::from_rgb(60, 120, 220)),
                    )
                    .clicked()
                {
                    game.resources.manual_drain_well = true;
                }

                ui.separator();

                if game.resources.paused {
                    if ui.button("Resume").clicked() {
                        game.resources.paused = false;
                    }
                } else if ui.button("Pause").clicked() {
                    game.resources.paused = true;
                }
            });
        });
}

fn draw_failure_overlay(game: &mut GameWorld, ui_context: &egui::Context) {
    let area = egui::Area::new(egui::Id::new("failure_overlay"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);

    area.show(ui_context, |ui| {
        egui::Frame::default()
            .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180))
            .inner_margin(32.0)
            .corner_radius(8.0)
            .show(ui, |ui| {
                let minutes = (game.resources.survival_time / 60.0) as u32;
                let seconds = (game.resources.survival_time % 60.0) as u32;

                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("CASTLE FALLEN")
                            .size(48.0)
                            .color(egui::Color32::from_rgb(220, 180, 50))
                            .strong(),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!("Survived {:02}:{:02}", minutes, seconds))
                            .size(28.0)
                            .color(egui::Color32::from_rgb(200, 200, 200)),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "Boulders faced: {}",
                            game.resources.bombardment.total_boulders_fired
                        ))
                        .size(16.0)
                        .color(egui::Color32::from_rgb(160, 160, 160)),
                    );

                    let total_replans: u32 = game
                        .resources
                        .agents
                        .iter()
                        .filter_map(|&entity| game.get_agent(entity))
                        .map(|agent| agent.plan_generation)
                        .sum();
                    ui.label(
                        egui::RichText::new(format!("Plans generated: {}", total_replans))
                            .size(16.0)
                            .color(egui::Color32::from_rgb(160, 160, 160)),
                    );

                    ui.add_space(16.0);
                    if ui
                        .button(egui::RichText::new("Restart").size(20.0).strong())
                        .clicked()
                    {
                        game.resources.restart_requested = true;
                    }
                });
            });
    });
}

fn format_action_name(name: &str) -> String {
    match name {
        "FetchWaterWell" => "Fetch Water (Well)".to_string(),
        "FetchWaterRiver" => "Fetch Water (River)".to_string(),
        "DouseFire" => "Douse Fire".to_string(),
        "FetchRepairMaterials" => "Get Repair Mat.".to_string(),
        "SalvageRubble" => "Salvage Rubble".to_string(),
        "RepairWall" => "Repair Wall".to_string(),
        "ClearRubble" => "Clear Rubble".to_string(),
        "FetchArrows" => "Fetch Arrows".to_string(),
        "ResupplyArcher" => "Resupply Archer".to_string(),
        "ReinforceGate" => "Reinforce Gate".to_string(),
        "RepairBackGate" => "Repair Back Gate".to_string(),
        "TendWounded" => "Tend Wounded".to_string(),
        _ => name.to_string(),
    }
}

fn format_action_target(target: crate::goap::ActionTarget) -> String {
    match target {
        crate::goap::ActionTarget::Well => "Well".to_string(),
        crate::goap::ActionTarget::River => "River".to_string(),
        crate::goap::ActionTarget::Fire => "Fire".to_string(),
        crate::goap::ActionTarget::RepairPile => "Repair Pile".to_string(),
        crate::goap::ActionTarget::RubbleNearest => "Rubble".to_string(),
        crate::goap::ActionTarget::Breach => "Wall Breach".to_string(),
        crate::goap::ActionTarget::Armory => "Armory".to_string(),
        crate::goap::ActionTarget::ArcherPost => "Archer Post".to_string(),
        crate::goap::ActionTarget::Gate => "Gate".to_string(),
        crate::goap::ActionTarget::HealStation => "Healing Station".to_string(),
        crate::goap::ActionTarget::BackGate => "Back Gate".to_string(),
    }
}

fn lerp_color(bad: egui::Color32, good: egui::Color32, ratio: f32) -> egui::Color32 {
    let ratio = ratio.clamp(0.0, 1.0);
    egui::Color32::from_rgb(
        (bad.r() as f32 * (1.0 - ratio) + good.r() as f32 * ratio) as u8,
        (bad.g() as f32 * (1.0 - ratio) + good.g() as f32 * ratio) as u8,
        (bad.b() as f32 * (1.0 - ratio) + good.b() as f32 * ratio) as u8,
    )
}
