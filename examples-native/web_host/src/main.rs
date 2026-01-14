#![windows_subsystem = "windows"]

use nightshade::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashSet;

#[cfg(not(target_arch = "wasm32"))]
mod static_server;
#[cfg(not(target_arch = "wasm32"))]
mod webview_manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(WebHost::default())?;
    Ok(())
}

struct WebHost {
    web_widget_id: String,
    #[cfg(not(target_arch = "wasm32"))]
    server_port: u16,
    #[cfg(not(target_arch = "wasm32"))]
    webview_manager: webview_manager::WebviewManager,
}

impl Default for WebHost {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let server_port = static_server::start_server();

        Self {
            web_widget_id: uuid::Uuid::new_v4().to_string(),
            #[cfg(not(target_arch = "wasm32"))]
            server_port,
            #[cfg(not(target_arch = "wasm32"))]
            webview_manager: webview_manager::WebviewManager::default(),
        }
    }
}

impl State for WebHost {
    fn title(&self) -> &str {
        "Nightshade Web Host"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui_context, |ui| {
                let rect = ui.available_rect_before_wrap();

                ui.painter()
                    .rect_filled(rect, 0.0, ui.style().visuals.panel_fill);

                #[cfg(target_arch = "wasm32")]
                {
                    ui.centered_and_justified(|ui| {
                        ui.label("Web widget is unavailable on wasm");
                    });
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(window_handle) = &world.resources.window.handle {
                        self.webview_manager.set_window(window_handle.clone());
                    }

                    let scale_factor = ui_context.pixels_per_point();
                    let mut active_ids: HashSet<String> = HashSet::new();
                    active_ids.insert(self.web_widget_id.clone());

                    self.webview_manager.retain_only(&active_ids);

                    let url = format!("http://127.0.0.1:{}", self.server_port);

                    if !self.webview_manager.has_webview(&self.web_widget_id) {
                        self.webview_manager.create_webview(
                            self.web_widget_id.clone(),
                            &url,
                            rect,
                            scale_factor,
                        );
                    }

                    self.webview_manager
                        .update_position(&self.web_widget_id, rect, scale_factor);

                    self.webview_manager.ensure_all_visible();
                }
            });
    }
}
