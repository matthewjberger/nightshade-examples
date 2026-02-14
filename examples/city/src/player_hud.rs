use nightshade::prelude::*;

pub fn draw_game_hud(camera_forward: Vec3, ctx: &egui::Context) {
    draw_crosshair(ctx);
    draw_compass(camera_forward, ctx);
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

fn draw_compass(camera_forward: Vec3, ctx: &egui::Context) {
    let heading_rad = camera_forward.z.atan2(camera_forward.x);
    let heading_deg = -heading_rad.to_degrees() + 90.0;
    let heading_deg = ((heading_deg % 360.0) + 360.0) % 360.0;

    let cardinal = match heading_deg as i32 {
        338..=360 | 0..=22 => "N",
        23..=67 => "NE",
        68..=112 => "E",
        113..=157 => "SE",
        158..=202 => "S",
        203..=247 => "SW",
        248..=292 => "W",
        293..=337 => "NW",
        _ => "N",
    };

    #[allow(deprecated)]
    let screen_rect = ctx.screen_rect();
    let center_x = screen_rect.center().x;

    egui::Area::new(egui::Id::new("fp_compass"))
        .fixed_pos(egui::pos2(center_x - 40.0, 16.0))
        .show(ctx, |ui| {
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 160))
                .inner_margin(egui::Margin::symmetric(12, 4))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(cardinal)
                                .color(egui::Color32::WHITE)
                                .strong()
                                .size(18.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.0}\u{00b0}", heading_deg))
                                .color(egui::Color32::LIGHT_GRAY)
                                .size(14.0),
                        );
                    });
                });
        });
}
