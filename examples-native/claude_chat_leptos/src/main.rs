#![windows_subsystem = "windows"]

use std::sync::mpsc;

use claude_chat_protocol::{AgentStatus, BackendEvent, FrontendCommand};
use include_dir::{Dir, include_dir};
use nightshade::claude::{ClaudeConfig, CliCommand, CliEvent, McpConfig, spawn_cli_worker};
use nightshade::prelude::*;
use nightshade::webview::{WebviewContext, serve_embedded_dir};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/site/dist");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cli_cmd_tx, cli_cmd_rx) = mpsc::channel::<CliCommand>();
    let (cli_event_tx, cli_event_rx) = mpsc::channel::<CliEvent>();

    let config = ClaudeConfig {
        mcp_config: McpConfig::None,
        ..Default::default()
    };

    spawn_cli_worker(cli_cmd_rx, cli_event_tx, config);

    launch(ChatApp {
        port: serve_embedded_dir(&DIST),
        ctx: WebviewContext::default(),
        connected: false,
        cli_cmd_tx,
        cli_event_rx,
    })?;

    Ok(())
}

struct ChatApp {
    port: u16,
    ctx: WebviewContext<FrontendCommand, BackendEvent>,
    connected: bool,
    cli_cmd_tx: mpsc::Sender<CliCommand>,
    cli_event_rx: mpsc::Receiver<CliEvent>,
}

impl State for ChatApp {
    fn title(&self) -> &str {
        "Claude Chat"
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

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        let commands: Vec<FrontendCommand> = self.ctx.drain_messages().collect();
        for cmd in commands {
            match cmd {
                FrontendCommand::Ready => {
                    if !self.connected {
                        self.ctx.send(BackendEvent::Connected);
                        self.ctx.send(BackendEvent::StatusUpdate {
                            status: AgentStatus::Idle,
                        });
                        self.connected = true;
                    }
                }
                FrontendCommand::SendPrompt {
                    prompt,
                    session_id,
                    model,
                } => {
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Thinking,
                    });
                    let _ = self.cli_cmd_tx.send(CliCommand::StartQuery {
                        prompt,
                        session_id,
                        model,
                    });
                }
                FrontendCommand::CancelRequest => {
                    let _ = self.cli_cmd_tx.send(CliCommand::Cancel);
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
            }
        }

        for event in self.cli_event_rx.try_iter() {
            match event {
                CliEvent::SessionStarted { session_id } => {
                    self.ctx.send(BackendEvent::StreamingStarted { session_id });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Streaming,
                    });
                }
                CliEvent::TextDelta { text } => {
                    self.ctx.send(BackendEvent::TextDelta { text });
                }
                CliEvent::ThinkingDelta { text } => {
                    self.ctx.send(BackendEvent::ThinkingDelta { text });
                }
                CliEvent::ToolUseStarted { tool_name, tool_id } => {
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::UsingTool {
                            tool_name: tool_name.clone(),
                        },
                    });
                    self.ctx.send(BackendEvent::ToolUseStarted {
                        tool_name,
                        tool_id,
                    });
                }
                CliEvent::ToolUseInputDelta {
                    tool_id,
                    partial_json,
                } => {
                    self.ctx.send(BackendEvent::ToolUseInputDelta {
                        tool_id,
                        partial_json,
                    });
                }
                CliEvent::ToolUseFinished { tool_id } => {
                    self.ctx.send(BackendEvent::ToolUseFinished { tool_id });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Streaming,
                    });
                }
                CliEvent::TurnComplete { session_id } => {
                    self.ctx.send(BackendEvent::TurnComplete { session_id });
                }
                CliEvent::Complete {
                    session_id,
                    total_cost_usd,
                    num_turns,
                } => {
                    self.ctx.send(BackendEvent::RequestComplete {
                        session_id,
                        total_cost_usd,
                        num_turns,
                    });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
                CliEvent::Error { message } => {
                    self.ctx.send(BackendEvent::Error { message });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
            }
        }

        let panel_width = (ctx.content_rect().width() * 0.4).max(360.0);

        egui::SidePanel::right("chat_panel")
            .exact_width(panel_width)
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
