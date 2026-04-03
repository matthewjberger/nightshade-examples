#![windows_subsystem = "windows"]

use std::sync::mpsc;

use nightshade::claude::{
    ClaudeConfig, CliCommand, CliEvent, McpConfig, create_cli_channels, spawn_cli_worker,
};
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (command_sender, command_receiver, event_sender, event_receiver) = create_cli_channels();

    let config = ClaudeConfig {
        mcp_config: McpConfig::None,
        ..Default::default()
    };

    spawn_cli_worker(command_receiver, event_sender, config);

    launch(ChatApp {
        command_sender,
        event_receiver,
        messages: Vec::new(),
        input_text: String::new(),
        streaming_text: String::new(),
        thinking_text: String::new(),
        active_tools: Vec::new(),
        status: Status::Idle,
        session_id: None,
        total_cost_usd: 0.0,
    })?;

    Ok(())
}

#[derive(Clone)]
enum MessageRole {
    User,
    Assistant,
}

#[derive(Clone)]
struct ChatMessage {
    role: MessageRole,
    content: String,
    thinking: String,
    tool_uses: Vec<ToolUse>,
}

#[derive(Clone)]
struct ToolUse {
    tool_name: String,
    input_json: String,
    finished: bool,
}

#[derive(Clone, PartialEq)]
enum Status {
    Idle,
    Thinking,
    Streaming,
    UsingTool(String),
}

struct ChatApp {
    command_sender: mpsc::Sender<CliCommand>,
    event_receiver: mpsc::Receiver<CliEvent>,
    messages: Vec<ChatMessage>,
    input_text: String,
    streaming_text: String,
    thinking_text: String,
    active_tools: Vec<ToolUse>,
    status: Status,
    session_id: Option<String>,
    total_cost_usd: f64,
}

impl ChatApp {
    fn finalize_streaming_message(&mut self) {
        if !self.streaming_text.is_empty()
            || !self.thinking_text.is_empty()
            || !self.active_tools.is_empty()
        {
            self.messages.push(ChatMessage {
                role: MessageRole::Assistant,
                content: std::mem::take(&mut self.streaming_text),
                thinking: std::mem::take(&mut self.thinking_text),
                tool_uses: std::mem::take(&mut self.active_tools),
            });
        }
        self.status = Status::Idle;
    }

    fn is_busy(&self) -> bool {
        self.status != Status::Idle
    }

    fn drain_events(&mut self) {
        let events: Vec<CliEvent> = self.event_receiver.try_iter().collect();
        for event in events {
            match event {
                CliEvent::SessionStarted { session_id } => {
                    self.session_id = Some(session_id);
                    self.streaming_text.clear();
                    self.thinking_text.clear();
                    self.active_tools.clear();
                    self.status = Status::Streaming;
                }
                CliEvent::TextDelta { text } => {
                    self.streaming_text.push_str(&text);
                    if self.status == Status::Thinking {
                        self.status = Status::Streaming;
                    }
                }
                CliEvent::ThinkingDelta { text } => {
                    self.thinking_text.push_str(&text);
                    self.status = Status::Thinking;
                }
                CliEvent::ToolUseStarted { tool_name, .. } => {
                    self.status = Status::UsingTool(tool_name.clone());
                    self.active_tools.push(ToolUse {
                        tool_name,
                        input_json: String::new(),
                        finished: false,
                    });
                }
                CliEvent::ToolUseInputDelta { partial_json, .. } => {
                    if let Some(tool) = self.active_tools.last_mut() {
                        tool.input_json.push_str(&partial_json);
                    }
                }
                CliEvent::ToolUseFinished { .. } => {
                    if let Some(tool) = self.active_tools.last_mut() {
                        tool.finished = true;
                    }
                    self.status = Status::Streaming;
                }
                CliEvent::TurnComplete { .. } => {}
                CliEvent::Complete { total_cost_usd, .. } => {
                    if let Some(cost) = total_cost_usd {
                        self.total_cost_usd = cost;
                    }
                    self.finalize_streaming_message();
                }
                CliEvent::Error { message } => {
                    self.finalize_streaming_message();
                    self.messages.push(ChatMessage {
                        role: MessageRole::Assistant,
                        content: format!("Error: {message}"),
                        thinking: String::new(),
                        tool_uses: Vec::new(),
                    });
                }
            }
        }
    }

    fn send_prompt(&mut self) {
        let text = self.input_text.trim().to_string();
        if text.is_empty() {
            return;
        }

        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: text.clone(),
            thinking: String::new(),
            tool_uses: Vec::new(),
        });

        self.status = Status::Thinking;

        let _ = self.command_sender.send(CliCommand::StartQuery {
            prompt: text,
            session_id: self.session_id.clone(),
            model: None,
        });

        self.input_text.clear();
    }
}

const BG_DARK: egui::Color32 = egui::Color32::from_rgb(0x0d, 0x11, 0x17);
const BG_MID: egui::Color32 = egui::Color32::from_rgb(0x16, 0x1b, 0x22);
const BORDER: egui::Color32 = egui::Color32::from_rgb(0x30, 0x36, 0x3d);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xc9, 0xd1, 0xd9);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(0x8b, 0x94, 0x9e);
const TEXT_FAINT: egui::Color32 = egui::Color32::from_rgb(0x48, 0x4f, 0x58);
const BLUE: egui::Color32 = egui::Color32::from_rgb(0x1f, 0x6f, 0xeb);
const GREEN: egui::Color32 = egui::Color32::from_rgb(0x23, 0x86, 0x36);
const RED: egui::Color32 = egui::Color32::from_rgb(0xda, 0x36, 0x33);
const YELLOW: egui::Color32 = egui::Color32::from_rgb(0xe3, 0xb3, 0x41);
const PURPLE: egui::Color32 = egui::Color32::from_rgb(0xa5, 0x71, 0xf5);

impl State for ChatApp {
    fn title(&self) -> &str {
        "Claude Chat"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
    }

    fn ui(&mut self, _world: &mut World, ctx: &egui::Context) {
        self.drain_events();

        if self.is_busy() {
            ctx.request_repaint();
        }

        let mut root_ui = egui::Ui::new(
            ctx.clone(),
            egui::Id::new("root_ui"),
            egui::UiBuilder::new()
                .layer_id(egui::LayerId::background())
                .max_rect(ctx.content_rect()),
        );
        root_ui.set_clip_rect(ctx.content_rect());

        egui::Panel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(BG_MID)
                    .inner_margin(egui::Margin::symmetric(12, 6)),
            )
            .show_inside(&mut root_ui, |ui| {
                ui.visuals_mut().override_text_color = Some(TEXT_PRIMARY);
                self.render_toolbar(ui);
            });

        egui::Panel::bottom("input")
            .frame(
                egui::Frame::new()
                    .fill(BG_MID)
                    .stroke(egui::Stroke::new(1.0, BORDER))
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show_inside(&mut root_ui, |ui| {
                ui.visuals_mut().override_text_color = Some(TEXT_PRIMARY);
                self.render_input(ui);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG_DARK)
                    .inner_margin(egui::Margin::symmetric(16, 12)),
            )
            .show_inside(&mut root_ui, |ui| {
                ui.visuals_mut().override_text_color = Some(TEXT_PRIMARY);
                self.render_messages(ui);
            });
    }
}

impl ChatApp {
    fn render_toolbar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("CLAUDE CHAT")
                    .strong()
                    .size(12.0)
                    .color(TEXT_PRIMARY),
            );

            ui.add_space(16.0);

            let (dot_color, label) = match &self.status {
                Status::Idle => (GREEN, "Ready"),
                Status::Thinking => (YELLOW, "Thinking..."),
                Status::Streaming => (BLUE, "Streaming..."),
                Status::UsingTool(_) => (PURPLE, "Using tool..."),
            };

            let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.0, dot_color);

            ui.label(egui::RichText::new(label).size(11.0).color(TEXT_DIM));

            if let Status::UsingTool(ref name) = self.status {
                ui.label(
                    egui::RichText::new(format!("({name})"))
                        .size(11.0)
                        .color(PURPLE),
                );
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.total_cost_usd > 0.0 {
                    ui.label(
                        egui::RichText::new(format!("${:.4}", self.total_cost_usd))
                            .size(11.0)
                            .color(TEXT_FAINT),
                    );
                }

                if let Some(ref session) = self.session_id {
                    let truncated = if session.len() > 12 {
                        &session[..12]
                    } else {
                        session
                    };
                    ui.label(egui::RichText::new(truncated).size(11.0).color(TEXT_FAINT));
                }
            });
        });
    }

    fn render_messages(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if self.messages.is_empty()
                    && self.streaming_text.is_empty()
                    && self.thinking_text.is_empty()
                    && self.status == Status::Idle
                {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 3.0);
                        ui.label(
                            egui::RichText::new("Send a prompt to get started")
                                .size(13.0)
                                .color(TEXT_FAINT),
                        );
                    });
                    return;
                }

                for message in &self.messages {
                    render_message_bubble(ui, message);
                    ui.add_space(8.0);
                }

                let has_streaming = !self.streaming_text.is_empty()
                    || !self.thinking_text.is_empty()
                    || !self.active_tools.is_empty()
                    || self.status == Status::Thinking;

                if has_streaming {
                    render_streaming_bubble(
                        ui,
                        &self.streaming_text,
                        &self.thinking_text,
                        &self.active_tools,
                        &self.status,
                    );
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                }
            });
    }

    fn render_input(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            let available_width = ui.available_width() - 80.0;

            let text_edit = egui::TextEdit::multiline(&mut self.input_text)
                .desired_width(available_width)
                .desired_rows(3)
                .hint_text(
                    egui::RichText::new("Type a prompt... (Ctrl+Enter to send)").color(TEXT_FAINT),
                )
                .text_color(TEXT_PRIMARY);

            let response = ui.add(text_edit);

            if response.has_focus()
                && ui.input(|input| input.key_pressed(egui::Key::Enter) && input.modifiers.ctrl)
                && !self.input_text.trim().is_empty()
                && !self.is_busy()
            {
                self.send_prompt();
            }

            ui.vertical(|ui| {
                let send_enabled = !self.input_text.trim().is_empty() && !self.is_busy();
                let send_button = egui::Button::new(egui::RichText::new("Send").size(12.0).color(
                    if send_enabled {
                        egui::Color32::WHITE
                    } else {
                        TEXT_FAINT
                    },
                ))
                .fill(if send_enabled {
                    GREEN
                } else {
                    egui::Color32::from_rgb(0x21, 0x26, 0x2d)
                })
                .min_size(egui::vec2(64.0, 28.0));

                if ui.add_enabled(send_enabled, send_button).clicked() {
                    self.send_prompt();
                }

                ui.add_space(4.0);

                let cancel_enabled = self.is_busy();
                let cancel_button =
                    egui::Button::new(egui::RichText::new("Cancel").size(12.0).color(
                        if cancel_enabled {
                            egui::Color32::WHITE
                        } else {
                            TEXT_FAINT
                        },
                    ))
                    .fill(if cancel_enabled {
                        RED
                    } else {
                        egui::Color32::from_rgb(0x21, 0x26, 0x2d)
                    })
                    .min_size(egui::vec2(64.0, 28.0));

                if ui.add_enabled(cancel_enabled, cancel_button).clicked() {
                    let _ = self.command_sender.send(CliCommand::Cancel);
                }
            });
        });
    }
}

fn render_message_bubble(ui: &mut egui::Ui, message: &ChatMessage) {
    let is_user = matches!(message.role, MessageRole::User);

    let bubble_fill = if is_user { BLUE } else { BG_MID };
    let bubble_stroke = if is_user {
        egui::Stroke::NONE
    } else {
        egui::Stroke::new(1.0, BORDER)
    };

    let max_width = ui.available_width() * 0.8;

    if is_user {
        ui.add_space(ui.available_width() - max_width);
    }

    let bubble_frame = egui::Frame::new()
        .fill(bubble_fill)
        .stroke(bubble_stroke)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(12, 8));

    bubble_frame.show(ui, |ui| {
        ui.set_max_width(max_width);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        if !message.thinking.is_empty() && !is_user {
            let header_id = ui.id().with("thinking_header");
            let thinking_state = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                header_id,
                false,
            );
            let is_open = thinking_state.is_open();

            let header_response = ui.horizontal(|ui| {
                let arrow = if is_open { "v" } else { ">" };
                ui.label(egui::RichText::new(arrow).size(9.0).color(YELLOW));
                ui.label(egui::RichText::new("Thinking").size(11.0).color(YELLOW));
            });

            if header_response
                .response
                .interact(egui::Sense::click())
                .clicked()
            {
                let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    header_id,
                    false,
                );
                state.toggle(ui);
                state.store(ui.ctx());
            }

            if is_open {
                let thinking_frame = egui::Frame::new()
                    .stroke(egui::Stroke::new(1.0, BORDER))
                    .inner_margin(egui::Margin {
                        left: 8,
                        ..egui::Margin::symmetric(4, 4)
                    });
                thinking_frame.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&message.thinking)
                            .size(11.0)
                            .color(TEXT_DIM)
                            .family(egui::FontFamily::Monospace),
                    );
                });
            }

            ui.add_space(4.0);
        }

        ui.label(
            egui::RichText::new(&message.content)
                .size(13.0)
                .family(egui::FontFamily::Monospace),
        );

        if !message.tool_uses.is_empty() {
            ui.add_space(4.0);
            for tool in &message.tool_uses {
                render_tool_use(ui, tool);
            }
        }
    });
}

fn render_streaming_bubble(
    ui: &mut egui::Ui,
    streaming_text: &str,
    thinking_text: &str,
    active_tools: &[ToolUse],
    status: &Status,
) {
    let max_width = ui.available_width() * 0.8;

    let bubble_frame = egui::Frame::new()
        .fill(BG_MID)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(12, 8));

    bubble_frame.show(ui, |ui| {
        ui.set_max_width(max_width);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        if !thinking_text.is_empty() {
            ui.label(egui::RichText::new("Thinking").size(11.0).color(YELLOW));
            ui.label(
                egui::RichText::new(thinking_text)
                    .size(11.0)
                    .color(TEXT_DIM)
                    .family(egui::FontFamily::Monospace),
            );
            ui.add_space(4.0);
            ui.add(egui::Separator::default());
            ui.add_space(4.0);
        } else if *status == Status::Thinking && streaming_text.is_empty() {
            ui.label(egui::RichText::new("Thinking...").size(11.0).color(YELLOW));
        }

        if !streaming_text.is_empty() {
            let time = ui.input(|input| input.time);
            let blink = (time * 2.0).sin() > 0.0;
            let cursor = if blink { "_" } else { " " };
            let display_text = format!("{streaming_text}{cursor}");
            ui.label(
                egui::RichText::new(display_text)
                    .size(13.0)
                    .family(egui::FontFamily::Monospace),
            );
        } else {
            let time = ui.input(|input| input.time);
            let blink = (time * 2.0).sin() > 0.0;
            if blink {
                ui.label(egui::RichText::new("_").size(13.0).color(TEXT_PRIMARY));
            }
        }

        if !active_tools.is_empty() {
            ui.add_space(4.0);
            for tool in active_tools {
                render_tool_use(ui, tool);
            }
        }
    });
}

fn render_tool_use(ui: &mut egui::Ui, tool: &ToolUse) {
    let tool_frame = egui::Frame::new()
        .fill(BG_DARK)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(8, 4));

    tool_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&tool.tool_name)
                    .size(11.0)
                    .color(PURPLE)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if tool.finished {
                    ui.label(egui::RichText::new("done").size(10.0).color(GREEN));
                } else {
                    ui.label(egui::RichText::new("running...").size(10.0).color(YELLOW));
                }
            });
        });

        if !tool.input_json.is_empty() {
            ui.label(
                egui::RichText::new(&tool.input_json)
                    .size(10.0)
                    .color(TEXT_DIM)
                    .family(egui::FontFamily::Monospace),
            );
        }
    });
}
