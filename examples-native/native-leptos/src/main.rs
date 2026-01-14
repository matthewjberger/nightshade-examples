#![windows_subsystem = "windows"]

use include_dir::{Dir, include_dir};
use nightshade::prelude::*;
use std::thread;
use tiny_http::{Header, Response, Server};
use web_host_protocol::{BackendEvent, FrontendCommand};

mod context;

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/site/dist");

fn start_server() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("server");
    let port = server.server_addr().to_ip().unwrap().port();
    thread::spawn(move || {
        for req in server.incoming_requests() {
            let path = req.url().trim_start_matches('/');
            let path = if path.is_empty() { "index.html" } else { path };
            let file = DIST
                .get_file(path)
                .or_else(|| DIST.get_file("index.html"))
                .unwrap();
            let mime = match path.rsplit('.').next() {
                Some("html") => "text/html",
                Some("js") => "application/javascript",
                Some("wasm") => "application/wasm",
                Some("css") => "text/css",
                _ => "application/octet-stream",
            };
            let _ = req.respond(
                Response::from_data(file.contents())
                    .with_header(Header::from_bytes("Content-Type", mime).unwrap()),
            );
        }
    });
    port
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(WebHost {
        port: start_server(),
        ctx: context::WebviewContext::default(),
        connected: false,
    })?;
    Ok(())
}

struct WebHost {
    port: u16,
    ctx: context::WebviewContext,
    connected: bool,
}

impl State for WebHost {
    fn title(&self) -> &str {
        "Nightshade Web Host"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        for cmd in self.ctx.drain_messages() {
            match cmd {
                FrontendCommand::Ready => {
                    self.ctx.send(BackendEvent::Connected);
                    self.connected = true;
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
                let rect = ui.available_rect_before_wrap();
                ui.painter()
                    .rect_filled(rect, 0.0, ui.style().visuals.panel_fill);
                if let Some(handle) = &world.resources.window.handle {
                    self.ctx.ensure_webview(handle.clone(), self.port, rect);
                }
            });

        if !self.connected {
            ctx.request_repaint();
        }
    }
}
