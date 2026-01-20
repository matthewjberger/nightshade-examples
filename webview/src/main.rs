#![windows_subsystem = "windows"]

use include_dir::{Dir, include_dir};
use nightshade::prelude::*;
use nightshade::webview::{WebviewContext, serve_embedded_dir};
use web_host_protocol::{BackendEvent, FrontendCommand};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/site/dist");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(WebHost {
        port: serve_embedded_dir(&DIST),
        ctx: WebviewContext::default(),
        connected: false,
    })?;
    Ok(())
}

struct WebHost {
    port: u16,
    ctx: WebviewContext<FrontendCommand, BackendEvent>,
    connected: bool,
}

impl State for WebHost {
    fn title(&self) -> &str {
        "Nightshade WebView Example"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        for cmd in self.ctx.drain_messages() {
            match cmd {
                FrontendCommand::Ready => {
                    if !self.connected {
                        self.ctx.send(BackendEvent::Connected);
                        self.connected = true;
                    }
                }
                FrontendCommand::RequestRandomNumber { request_id } => {
                    self.ctx.send(BackendEvent::RandomNumber {
                        request_id,
                        value: rand::random(),
                    });
                }
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                if let Some(handle) = &world.resources.window.handle {
                    self.ctx.ensure_webview(
                        handle.clone(),
                        self.port,
                        ui.available_rect_before_wrap(),
                    );
                    handle.request_redraw();
                }
            });
    }
}
