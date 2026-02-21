use nightshade::prelude::egui;
use nightshade::prelude::egui::Color32;

use crate::simulation::Simulation;

pub fn draw_ui(
    simulation: &mut Simulation,
    is_night: bool,
    campfire_count: usize,
    ui_context: &egui::Context,
) {
    egui::SidePanel::left("village_panel")
        .min_width(260.0)
        .show(ui_context, |ui| {
            ui.heading("Village Survival");
            ui.separator();

            ui.label(format!("Generation: {}", simulation.generation));
            ui.label(format!(
                "Alive: {} / {}",
                simulation.alive_count(),
                simulation.agents.len()
            ));
            ui.label(format!(
                "Timer: {:.1}s / {:.0}s",
                simulation.generation_timer, simulation.generation_length
            ));
            let day_night_label = if is_night { "Night" } else { "Day" };
            ui.label(format!("Time: {day_night_label}"));
            ui.label(format!("Campfires: {campfire_count}"));
            ui.separator();

            ui.label("Speed:");
            ui.add(egui::Slider::new(
                &mut simulation.speed_multiplier,
                1.0..=5.0,
            ));

            if simulation.paused {
                if ui.button("Resume").clicked() {
                    simulation.paused = false;
                }
            } else if ui.button("Pause").clicked() {
                simulation.paused = true;
            }

            ui.separator();

            if let Some(selected_index) = simulation.selected_agent {
                if selected_index < simulation.agents.len() {
                    let agent = &simulation.agents[selected_index];

                    ui.label(format!("{} (Agent #{})", agent.name, selected_index));
                    let status = if agent.alive { "Alive" } else { "Dead" };
                    ui.label(format!("Status: {status}"));
                    ui.label(format!("Action: {}", agent.current_action.label()));
                    ui.label(format!("Survival: {:.1}s", agent.survival_time));
                    ui.label(format!("Home Lv: {}", agent.home_level));
                    ui.label(format!("Nearby: {}", agent.nearby_agent_count));
                    if agent.build_progress > 0.0 {
                        ui.label(format!("Building: {:.0}%", agent.build_progress * 100.0));
                    }
                    ui.separator();

                    ui.label("Personality:");
                    draw_labeled_bar(
                        ui,
                        "Boldness",
                        agent.genome.boldness,
                        Color32::from_rgb(200, 60, 60),
                    );
                    draw_labeled_bar(
                        ui,
                        "Social",
                        agent.genome.sociability,
                        Color32::from_rgb(200, 180, 50),
                    );
                    draw_labeled_bar(
                        ui,
                        "Metabolism",
                        agent.genome.metabolism,
                        Color32::from_rgb(60, 170, 60),
                    );
                    draw_labeled_bar(
                        ui,
                        "Wander",
                        agent.genome.wander_range,
                        Color32::from_rgb(60, 100, 200),
                    );
                    draw_labeled_bar(
                        ui,
                        "Home",
                        agent.genome.home_investment,
                        Color32::from_rgb(160, 80, 200),
                    );
                    ui.separator();

                    ui.label("Needs:");
                    draw_labeled_bar(
                        ui,
                        "Hunger",
                        agent.needs.hunger,
                        Color32::from_rgb(220, 140, 40),
                    );
                    draw_labeled_bar(
                        ui,
                        "Energy",
                        agent.needs.energy,
                        Color32::from_rgb(140, 60, 200),
                    );
                    draw_labeled_bar(
                        ui,
                        "Loneliness",
                        agent.needs.loneliness,
                        Color32::from_rgb(40, 180, 200),
                    );
                } else {
                    simulation.selected_agent = None;
                }
            } else {
                ui.label("Click an agent to inspect");
            }

            ui.separator();

            draw_survival_graph(ui, &simulation.history);

            ui.add_space(8.0);

            draw_trait_graph(ui, &simulation.history);

            ui.add_space(8.0);
            ui.separator();
            ui.label("Event Log");

            let log_height = ui.available_height().max(120.0);
            egui::ScrollArea::vertical()
                .max_height(log_height)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &simulation.event_log {
                        let color = Color32::from_rgb(
                            (entry.color[0] * 255.0) as u8,
                            (entry.color[1] * 255.0) as u8,
                            (entry.color[2] * 255.0) as u8,
                        );
                        ui.colored_label(color, &entry.message);
                    }
                });
        });
}

fn draw_labeled_bar(ui: &mut egui::Ui, label: &str, value: f32, fill_color: Color32) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        ui.add(
            egui::ProgressBar::new(value)
                .desired_width(140.0)
                .fill(fill_color),
        );
    });
}

fn draw_survival_graph(ui: &mut egui::Ui, history: &[crate::simulation::GenerationStats]) {
    ui.label("Survival Time");
    let available_width = ui.available_width();
    let (response, painter) =
        ui.allocate_painter(egui::vec2(available_width, 150.0), egui::Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, 2.0, Color32::from_rgb(20, 20, 25));

    if history.len() < 2 {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Waiting for data...",
            egui::FontId::proportional(12.0),
            Color32::GRAY,
        );
        return;
    }

    let display_count = history.len().min(50);
    let start_index = history.len() - display_count;
    let data = &history[start_index..];

    let max_value = data
        .iter()
        .map(|stats| stats.best_survival)
        .fold(1.0f32, f32::max);

    let margin = 4.0;
    let plot_left = rect.left() + margin;
    let plot_right = rect.right() - margin;
    let plot_top = rect.top() + margin;
    let plot_bottom = rect.bottom() - margin;
    let plot_width = plot_right - plot_left;
    let plot_height = plot_bottom - plot_top;

    let avg_color = Color32::from_rgb(80, 200, 80);
    let best_color = Color32::WHITE;

    for line_index in 0..(data.len() - 1) {
        let x0 = plot_left + (line_index as f32 / (data.len() - 1) as f32) * plot_width;
        let x1 = plot_left + ((line_index + 1) as f32 / (data.len() - 1) as f32) * plot_width;

        let y0_avg = plot_bottom - (data[line_index].avg_survival / max_value) * plot_height;
        let y1_avg = plot_bottom - (data[line_index + 1].avg_survival / max_value) * plot_height;
        painter.line_segment(
            [egui::pos2(x0, y0_avg), egui::pos2(x1, y1_avg)],
            egui::Stroke::new(1.5, avg_color),
        );

        let y0_best = plot_bottom - (data[line_index].best_survival / max_value) * plot_height;
        let y1_best = plot_bottom - (data[line_index + 1].best_survival / max_value) * plot_height;
        painter.line_segment(
            [egui::pos2(x0, y0_best), egui::pos2(x1, y1_best)],
            egui::Stroke::new(1.5, best_color),
        );
    }

    painter.text(
        egui::pos2(plot_left + 2.0, plot_top + 2.0),
        egui::Align2::LEFT_TOP,
        format!("{max_value:.0}s"),
        egui::FontId::proportional(10.0),
        Color32::GRAY,
    );

    let legend_y = plot_bottom - 12.0;
    painter.line_segment(
        [
            egui::pos2(plot_left + 2.0, legend_y),
            egui::pos2(plot_left + 14.0, legend_y),
        ],
        egui::Stroke::new(1.5, avg_color),
    );
    painter.text(
        egui::pos2(plot_left + 17.0, legend_y),
        egui::Align2::LEFT_CENTER,
        "Avg",
        egui::FontId::proportional(9.0),
        avg_color,
    );
    painter.line_segment(
        [
            egui::pos2(plot_left + 42.0, legend_y),
            egui::pos2(plot_left + 54.0, legend_y),
        ],
        egui::Stroke::new(1.5, best_color),
    );
    painter.text(
        egui::pos2(plot_left + 57.0, legend_y),
        egui::Align2::LEFT_CENTER,
        "Best",
        egui::FontId::proportional(9.0),
        best_color,
    );
}

fn draw_trait_graph(ui: &mut egui::Ui, history: &[crate::simulation::GenerationStats]) {
    ui.label("Trait Averages");
    let available_width = ui.available_width();
    let (response, painter) =
        ui.allocate_painter(egui::vec2(available_width, 150.0), egui::Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, 2.0, Color32::from_rgb(20, 20, 25));

    if history.len() < 2 {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Waiting for data...",
            egui::FontId::proportional(12.0),
            Color32::GRAY,
        );
        return;
    }

    let display_count = history.len().min(50);
    let start_index = history.len() - display_count;
    let data = &history[start_index..];

    let margin = 4.0;
    let plot_left = rect.left() + margin;
    let plot_right = rect.right() - margin;
    let plot_top = rect.top() + margin;
    let plot_bottom = rect.bottom() - margin;
    let plot_width = plot_right - plot_left;
    let plot_height = plot_bottom - plot_top;

    let trait_colors = [
        Color32::from_rgb(200, 60, 60),
        Color32::from_rgb(200, 180, 50),
        Color32::from_rgb(60, 170, 60),
        Color32::from_rgb(60, 100, 200),
        Color32::from_rgb(160, 80, 200),
    ];
    let trait_names = ["Bold", "Soc", "Meta", "Wand", "Home"];

    for (trait_index, color) in trait_colors.iter().enumerate() {
        for line_index in 0..(data.len() - 1) {
            let x0 = plot_left + (line_index as f32 / (data.len() - 1) as f32) * plot_width;
            let x1 = plot_left + ((line_index + 1) as f32 / (data.len() - 1) as f32) * plot_width;

            let y0 = plot_bottom - data[line_index].trait_averages[trait_index] * plot_height;
            let y1 = plot_bottom - data[line_index + 1].trait_averages[trait_index] * plot_height;

            painter.line_segment(
                [egui::pos2(x0, y0), egui::pos2(x1, y1)],
                egui::Stroke::new(1.5, *color),
            );
        }
    }

    let legend_y = plot_bottom - 12.0;
    let mut legend_x = plot_left + 2.0;
    for (trait_index, name) in trait_names.iter().enumerate() {
        painter.line_segment(
            [
                egui::pos2(legend_x, legend_y),
                egui::pos2(legend_x + 10.0, legend_y),
            ],
            egui::Stroke::new(1.5, trait_colors[trait_index]),
        );
        painter.text(
            egui::pos2(legend_x + 13.0, legend_y),
            egui::Align2::LEFT_CENTER,
            *name,
            egui::FontId::proportional(9.0),
            trait_colors[trait_index],
        );
        legend_x += 45.0;
    }
}
