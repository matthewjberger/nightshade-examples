use nightshade::prelude::*;

use crate::player_data::{PlayerProgress, SkillType, get_item_definition, get_skill_definition};

pub fn draw_game_hud(player_progress: &PlayerProgress, ctx: &egui::Context) {
    draw_crosshair(ctx);
    draw_health_mana_bars(player_progress, ctx);
    draw_inventory_bar(player_progress, ctx);
    draw_skill_bar(player_progress, ctx);
    draw_stats_display(player_progress, ctx);
}

fn draw_crosshair(ctx: &egui::Context) {
    #[allow(deprecated)]
    let screen_rect = ctx.screen_rect();
    let center = screen_rect.center();

    egui::Area::new(egui::Id::new("fp_crosshair"))
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

fn draw_health_mana_bars(player_progress: &PlayerProgress, ctx: &egui::Context) {
    let stats = &player_progress.stats;

    egui::Area::new(egui::Id::new("fp_health_mana"))
        .fixed_pos(egui::pos2(20.0, 20.0))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("HP").color(egui::Color32::RED).strong());
                    let health_ratio = stats.health / stats.max_health;
                    let bar = egui::ProgressBar::new(health_ratio)
                        .desired_width(200.0)
                        .fill(egui::Color32::from_rgb(180, 40, 40));
                    ui.add(bar);
                    ui.label(
                        egui::RichText::new(format!("{:.0}/{:.0}", stats.health, stats.max_health))
                            .color(egui::Color32::WHITE),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("MP")
                            .color(egui::Color32::from_rgb(80, 120, 255))
                            .strong(),
                    );
                    let mana_ratio = stats.mana / stats.max_mana;
                    let bar = egui::ProgressBar::new(mana_ratio)
                        .desired_width(200.0)
                        .fill(egui::Color32::from_rgb(60, 100, 200));
                    ui.add(bar);
                    ui.label(
                        egui::RichText::new(format!("{:.0}/{:.0}", stats.mana, stats.max_mana))
                            .color(egui::Color32::WHITE),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("XP")
                            .color(egui::Color32::YELLOW)
                            .strong(),
                    );
                    let exp_ratio = stats.experience as f32 / stats.experience_to_next_level as f32;
                    let bar = egui::ProgressBar::new(exp_ratio)
                        .desired_width(200.0)
                        .fill(egui::Color32::from_rgb(200, 180, 50));
                    ui.add(bar);
                    ui.label(
                        egui::RichText::new(format!("Lv.{}", stats.level))
                            .color(egui::Color32::GOLD),
                    );
                });
            });
        });
}

fn draw_inventory_bar(player_progress: &PlayerProgress, ctx: &egui::Context) {
    let inventory = &player_progress.inventory;

    egui::TopBottomPanel::bottom("fp_inventory_bar")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 40, 200))
                .inner_margin(egui::Margin::same(8)),
        )
        .min_height(60.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(
                    egui::RichText::new(format!("Gold: {}", inventory.gold))
                        .color(egui::Color32::GOLD)
                        .strong(),
                );
                ui.add_space(20.0);

                for (index, slot) in inventory.slots.iter().enumerate() {
                    let is_selected = index == inventory.selected_slot;
                    let frame_color = if is_selected {
                        egui::Color32::GOLD
                    } else {
                        egui::Color32::from_rgb(80, 80, 90)
                    };

                    egui::Frame::default()
                        .fill(egui::Color32::from_rgb(40, 40, 50))
                        .stroke(egui::Stroke::new(2.0, frame_color))
                        .inner_margin(egui::Margin::same(4))
                        .show(ui, |ui| {
                            ui.set_min_size(egui::vec2(40.0, 40.0));

                            if let Some(item_type) = slot.item_type {
                                if let Some(def) = get_item_definition(item_type) {
                                    let color = egui::Color32::from_rgba_unmultiplied(
                                        (def.color[0] * 255.0) as u8,
                                        (def.color[1] * 255.0) as u8,
                                        (def.color[2] * 255.0) as u8,
                                        255,
                                    );
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new(
                                                &def.name[0..def.name.len().min(3)],
                                            )
                                            .color(color)
                                            .strong(),
                                        );
                                        if slot.quantity > 1 {
                                            ui.label(
                                                egui::RichText::new(format!("x{}", slot.quantity))
                                                    .small()
                                                    .color(egui::Color32::LIGHT_GRAY),
                                            );
                                        }
                                    });
                                }
                            } else {
                                ui.label(
                                    egui::RichText::new(format!("{}", index + 1))
                                        .color(egui::Color32::DARK_GRAY),
                                );
                            }
                        });
                }
            });
        });
}

fn draw_skill_bar(player_progress: &PlayerProgress, ctx: &egui::Context) {
    let skills = &player_progress.skills;

    egui::Area::new(egui::Id::new("fp_skill_bar"))
        .fixed_pos(egui::pos2(20.0, 120.0))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("Skills")
                        .color(egui::Color32::WHITE)
                        .strong(),
                );

                let skill_types = [
                    SkillType::Fireball,
                    SkillType::IceBlast,
                    SkillType::LightningBolt,
                    SkillType::Dash,
                    SkillType::MagicShield,
                    SkillType::Heal,
                ];

                for skill_type in skill_types {
                    if let Some(state) = skills.skills.get(&skill_type) {
                        if !state.unlocked {
                            continue;
                        }

                        if let Some(def) = get_skill_definition(skill_type) {
                            let color = egui::Color32::from_rgba_unmultiplied(
                                (def.color[0] * 255.0) as u8,
                                (def.color[1] * 255.0) as u8,
                                (def.color[2] * 255.0) as u8,
                                255,
                            );

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("[{}]", def.key_binding))
                                        .color(egui::Color32::GRAY),
                                );
                                ui.label(egui::RichText::new(def.name).color(color));

                                if state.cooldown_remaining > 0.0 {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "({:.1}s)",
                                            state.cooldown_remaining
                                        ))
                                        .color(egui::Color32::RED),
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new("Ready").color(egui::Color32::GREEN),
                                    );
                                }
                            });
                        }
                    }
                }
            });
        });
}

fn draw_stats_display(player_progress: &PlayerProgress, ctx: &egui::Context) {
    let stats = &player_progress.stats;

    #[allow(deprecated)]
    let screen_rect = ctx.screen_rect();

    egui::Area::new(egui::Id::new("fp_stats"))
        .fixed_pos(egui::pos2(screen_rect.width() - 180.0, 20.0))
        .show(ctx, |ui| {
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 180))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Stats")
                            .color(egui::Color32::WHITE)
                            .strong(),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Damage: {:.0}", stats.get_total_damage()))
                            .color(egui::Color32::from_rgb(255, 100, 100)),
                    );
                    ui.label(
                        egui::RichText::new(format!("Defense: {:.0}%", stats.defense))
                            .color(egui::Color32::from_rgb(100, 150, 255)),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "Speed: {:.0}%",
                            stats.speed_multiplier * 100.0
                        ))
                        .color(egui::Color32::from_rgb(100, 255, 100)),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Kills: {}", player_progress.enemies_killed))
                            .color(egui::Color32::LIGHT_GRAY),
                    );
                    ui.label(
                        egui::RichText::new(format!("Items: {}", player_progress.items_collected))
                            .color(egui::Color32::LIGHT_GRAY),
                    );
                });
        });
}
