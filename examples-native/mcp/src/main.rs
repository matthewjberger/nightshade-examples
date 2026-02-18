#![windows_subsystem = "windows"]

use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(McpViewer)?;
    Ok(())
}

struct McpViewer;

const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xc9, 0xd1, 0xd9);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(0x8b, 0x94, 0x9e);
const TEXT_FAINT: egui::Color32 = egui::Color32::from_rgb(0x48, 0x4f, 0x58);
const BG_MID: egui::Color32 = egui::Color32::from_rgb(0x16, 0x1b, 0x22);

impl State for McpViewer {
    fn title(&self) -> &str {
        "Nightshade MCP"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = true;

        let focus = Vec3::zeros();
        let camera = nightshade::ecs::camera::spawn_pan_orbit_camera(
            world,
            focus,
            10.0,
            0.5,
            0.4,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);
    }

    fn ui(&mut self, _world: &mut World, ctx: &egui::Context) {
        egui::TopBottomPanel::top("mcp_info")
            .frame(
                egui::Frame::new()
                    .fill(BG_MID)
                    .inner_margin(egui::Margin::symmetric(12, 6)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("NIGHTSHADE MCP")
                            .strong()
                            .size(12.0)
                            .color(TEXT_PRIMARY),
                    );

                    ui.separator();

                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 4.0, egui::Color32::from_rgb(0x23, 0x86, 0x36));

                    ui.label(
                        egui::RichText::new("http://127.0.0.1:3333/mcp")
                            .size(11.0)
                            .color(TEXT_DIM),
                    );

                    ui.separator();

                    ui.label(
                        egui::RichText::new(
                            "Connect Claude or any MCP client to spawn entities and manipulate the scene",
                        )
                        .size(11.0)
                        .color(TEXT_FAINT),
                    );
                });
            });
    }
}
