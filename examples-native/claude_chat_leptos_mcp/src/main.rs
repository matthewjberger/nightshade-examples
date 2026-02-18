#![windows_subsystem = "windows"]

use std::sync::mpsc;

use claude_chat_mcp_protocol::{AgentStatus, BackendEvent, FrontendCommand};
use include_dir::{Dir, include_dir};
use nightshade::claude::{CliCommand, CliEvent, ClaudeConfig, spawn_cli_worker};
use nightshade::mosaic::{Mosaic, Pane, ViewportWidget, Widget, WidgetContext, WidgetEntry};
use nightshade::prelude::*;
use nightshade::webview::{WebviewContext, serve_embedded_dir};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/site/dist");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cli_cmd_tx, cli_cmd_rx) = mpsc::channel::<CliCommand>();
    let (cli_event_tx, cli_event_rx) = mpsc::channel::<CliEvent>();

    let config = ClaudeConfig {
        system_prompt: Some(
            "You have access to nightshade's MCP scene tools. You can spawn entities (cube, sphere, cylinder, cone, torus, plane, capsule), set positions, rotations, scales, colors, materials, emissive values, add lights (point, spot, directional), change the camera, set atmosphere, add water, particles, HUD text, 3D text, and more. The 3D scene is visible to the left of the chat window. Use these tools when asked to create or manipulate 3D scenes. Toast notifications will appear in the chat when you use tools.".to_string(),
        ),
        ..Default::default()
    };

    spawn_cli_worker(cli_cmd_rx, cli_event_tx, config);

    let port = serve_embedded_dir(&DIST);

    launch(ChatApp {
        port,
        ctx: WebviewContext::default(),
        connected: false,
        cli_cmd_tx,
        cli_event_rx,
        mosaic: create_default_mosaic(),
        chat_context: ChatContext {
            webview_rect: None,
        },
    })?;

    Ok(())
}

fn create_default_mosaic() -> Mosaic<ChatWidget, ChatContext, ()> {
    let mut tiles = egui_tiles::Tiles::default();
    let viewport = tiles.insert_pane(Pane::new(ChatWidget::Viewport(ViewportWidget {
        camera_index: 0,
    })));
    let chat = tiles.insert_pane(Pane::new(ChatWidget::Chat(ChatWebWidget)));
    let root = tiles.insert_horizontal_tile(vec![viewport, chat]);
    let tree = egui_tiles::Tree::new("chat_mosaic", root, tiles);
    Mosaic::with_tree(tree)
}

struct ChatContext {
    webview_rect: Option<egui::Rect>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ChatWebWidget;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
enum ChatWidget {
    Viewport(ViewportWidget),
    Chat(ChatWebWidget),
}

impl Widget<ChatContext, ()> for ChatWidget {
    fn title(&self) -> String {
        match self {
            ChatWidget::Viewport(viewport) => viewport.title(),
            ChatWidget::Chat(_) => "Chat".to_string(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut WidgetContext<ChatContext, ()>) {
        match self {
            ChatWidget::Viewport(viewport) => viewport.ui(ui, context),
            ChatWidget::Chat(_) => {
                let rect = ui.available_rect_before_wrap();
                ui.painter()
                    .rect_filled(rect, 0.0, ui.style().visuals.panel_fill);
                ui.allocate_rect(rect, egui::Sense::hover());
                context.app.webview_rect = Some(rect);
            }
        }
    }

    fn closable(&self) -> bool {
        false
    }

    fn required_camera(&self, cached_cameras: &[Entity]) -> Option<Entity> {
        match self {
            ChatWidget::Viewport(viewport) => viewport.required_camera(cached_cameras),
            ChatWidget::Chat(_) => None,
        }
    }

    fn catalog() -> Vec<WidgetEntry<Self>> {
        vec![
            WidgetEntry {
                name: "Viewport".to_string(),
                create: || ChatWidget::Viewport(ViewportWidget::default()),
            },
            WidgetEntry {
                name: "Chat".to_string(),
                create: || ChatWidget::Chat(ChatWebWidget),
            },
        ]
    }
}

struct ChatApp {
    port: u16,
    ctx: WebviewContext<FrontendCommand, BackendEvent>,
    connected: bool,
    cli_cmd_tx: mpsc::Sender<CliCommand>,
    cli_event_rx: mpsc::Receiver<CliEvent>,
    mosaic: Mosaic<ChatWidget, ChatContext, ()>,
    chat_context: ChatContext,
}

impl State for ChatApp {
    fn title(&self) -> &str {
        "Claude Chat MCP"
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

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);
    }

    fn after_mcp_command(
        &mut self,
        _world: &mut World,
        command: &nightshade::mcp::McpCommand,
        response: &nightshade::mcp::McpResponse,
    ) {
        let message = command.to_string();
        let kind = match response {
            nightshade::mcp::McpResponse::Error(_) => "error",
            _ => match command {
                nightshade::mcp::McpCommand::ClearScene(_)
                | nightshade::mcp::McpCommand::DespawnEntity { .. } => "warning",
                _ => "success",
            },
        };
        self.ctx.send(BackendEvent::ShowNotification {
            message,
            kind: kind.to_string(),
        });
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        self.process_frontend_commands();
        self.forward_cli_events();

        self.chat_context.webview_rect = None;
        Mosaic::<ChatWidget, ChatContext, ()>::clear_required_cameras(world);

        self.mosaic.show(world, ui_context, &mut self.chat_context);

        if let Some(rect) = self.chat_context.webview_rect
            && let Some(handle) = &world.resources.window.handle
        {
            self.ctx.ensure_webview(handle.clone(), self.port, rect);
            handle.request_redraw();
        }
    }
}

impl ChatApp {
    fn process_frontend_commands(&mut self) {
        let commands: Vec<FrontendCommand> = self.ctx.drain_messages().collect();
        for command in commands {
            match command {
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
    }

    fn forward_cli_events(&mut self) {
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
    }
}
