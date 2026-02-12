use std::sync::{Arc, Mutex};

use egui_snarl::ui::{PinInfo, SnarlStyle, SnarlViewer, SnarlWidget, WireStyle};
use egui_snarl::{InPin, OutPin, Snarl};
use nightshade::prelude::egui;

use crate::shader_pass::{ChannelSource, PassId, RenderMode, SharedState};

#[derive(Clone)]
pub enum PipelineNode {
    Uniforms,
    Textures,
    VertexShader,
    FragmentShader,
    Output,
    BufferPass(usize),
}

pub struct PipelineGraph {
    snarl: Snarl<PipelineNode>,
    pub visible: bool,
}

const COLOR_UNIFORM: egui::Color32 = egui::Color32::from_rgb(0x4E, 0xC9, 0xB0);
const COLOR_TEXTURE: egui::Color32 = egui::Color32::from_rgb(0xDC, 0xDC, 0xAA);
const COLOR_VARYING: egui::Color32 = egui::Color32::from_rgb(0x9C, 0xDC, 0xFE);
const COLOR_OUTPUT: egui::Color32 = egui::Color32::from_rgb(0xC5, 0x86, 0xC0);
const COLOR_BUFFER: egui::Color32 = egui::Color32::from_rgb(0x56, 0x9C, 0xD6);

impl PipelineGraph {
    pub fn new() -> Self {
        let mut snarl = Snarl::new();

        let uniforms = snarl.insert_node(egui::pos2(50.0, 80.0), PipelineNode::Uniforms);
        let textures = snarl.insert_node(egui::pos2(50.0, 320.0), PipelineNode::Textures);
        let _buffer_a = snarl.insert_node(egui::pos2(50.0, 520.0), PipelineNode::BufferPass(0));
        let _buffer_b = snarl.insert_node(egui::pos2(250.0, 520.0), PipelineNode::BufferPass(1));
        let _buffer_c = snarl.insert_node(egui::pos2(450.0, 520.0), PipelineNode::BufferPass(2));
        let _buffer_d = snarl.insert_node(egui::pos2(650.0, 520.0), PipelineNode::BufferPass(3));
        let vertex = snarl.insert_node(egui::pos2(380.0, 50.0), PipelineNode::VertexShader);
        let fragment = snarl.insert_node(egui::pos2(380.0, 280.0), PipelineNode::FragmentShader);
        let output = snarl.insert_node(egui::pos2(680.0, 200.0), PipelineNode::Output);

        snarl.connect(
            egui_snarl::OutPinId {
                node: uniforms,
                output: 0,
            },
            egui_snarl::InPinId {
                node: vertex,
                input: 0,
            },
        );
        snarl.connect(
            egui_snarl::OutPinId {
                node: uniforms,
                output: 0,
            },
            egui_snarl::InPinId {
                node: fragment,
                input: 0,
            },
        );
        snarl.connect(
            egui_snarl::OutPinId {
                node: textures,
                output: 0,
            },
            egui_snarl::InPinId {
                node: fragment,
                input: 1,
            },
        );
        snarl.connect(
            egui_snarl::OutPinId {
                node: vertex,
                output: 0,
            },
            egui_snarl::InPinId {
                node: fragment,
                input: 2,
            },
        );
        snarl.connect(
            egui_snarl::OutPinId {
                node: fragment,
                output: 0,
            },
            egui_snarl::InPinId {
                node: output,
                input: 0,
            },
        );

        Self {
            snarl,
            visible: false,
        }
    }

    pub fn show(&mut self, ui_context: &egui::Context, shared: &Arc<Mutex<SharedState>>) {
        if !self.visible {
            return;
        }

        let shared_data = shared.lock().unwrap();
        let viewer_data = ViewerData {
            time: shared_data.uniforms.time,
            resolution: shared_data.uniforms.resolution,
            mouse: shared_data.uniforms.mouse,
            render_mode: shared_data.render_mode,
            has_error: shared_data.pass_compilation_errors[0].is_some(),
            texture_names: shared_data.texture_slot_names.clone(),
            frame: shared_data.uniforms.frame,
            paused: shared_data.paused,
            pass_enabled: shared_data.pass_enabled,
            pass_errors: std::array::from_fn(|index| {
                shared_data.pass_compilation_errors[index].is_some()
            }),
            channel_bindings: shared_data.channel_bindings,
        };
        drop(shared_data);

        let mut viewer = PipelineViewer { data: viewer_data };

        let style = SnarlStyle {
            collapsible: Some(false),
            ..SnarlStyle::new()
        };

        egui::Window::new("Pipeline Graph")
            .default_size([780.0, 480.0])
            .resizable(true)
            .show(ui_context, |ui| {
                SnarlWidget::new()
                    .id(egui::Id::new("pipeline_snarl"))
                    .style(style)
                    .show(&mut self.snarl, &mut viewer, ui);
            });
    }
}

struct ViewerData {
    time: f32,
    resolution: [f32; 2],
    mouse: [f32; 2],
    render_mode: RenderMode,
    has_error: bool,
    texture_names: [Option<String>; 4],
    frame: u32,
    paused: bool,
    pass_enabled: [bool; 5],
    pass_errors: [bool; 5],
    channel_bindings: [[ChannelSource; 4]; 5],
}

struct PipelineViewer {
    data: ViewerData,
}

impl SnarlViewer<PipelineNode> for PipelineViewer {
    fn title(&mut self, node: &PipelineNode) -> String {
        match node {
            PipelineNode::Uniforms => "Uniforms (group 0)".to_string(),
            PipelineNode::Textures => "Textures (group 1)".to_string(),
            PipelineNode::VertexShader => {
                let mode = match self.data.render_mode {
                    RenderMode::Fullscreen => "Fullscreen",
                    RenderMode::Geometry => "Geometry",
                };
                format!("Vertex Shader [{mode}]")
            }
            PipelineNode::FragmentShader => "Fragment Shader (Image)".to_string(),
            PipelineNode::Output => "Render Output".to_string(),
            PipelineNode::BufferPass(index) => {
                let label = PassId::BUFFERS[*index].label();
                let enabled = self.data.pass_enabled[index + 1];
                if enabled {
                    format!("{label} Pass")
                } else {
                    format!("{label} (disabled)")
                }
            }
        }
    }

    fn inputs(&mut self, node: &PipelineNode) -> usize {
        match node {
            PipelineNode::Uniforms | PipelineNode::Textures => 0,
            PipelineNode::VertexShader => 1,
            PipelineNode::FragmentShader => 3,
            PipelineNode::Output => 1,
            PipelineNode::BufferPass(_) => 0,
        }
    }

    fn outputs(&mut self, node: &PipelineNode) -> usize {
        match node {
            PipelineNode::Output => 0,
            _ => 1,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<PipelineNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        match &snarl[pin.id.node] {
            PipelineNode::VertexShader => {
                ui.label("@group(0) uniforms");
                PinInfo::circle()
                    .with_fill(COLOR_UNIFORM)
                    .with_wire_style(WireStyle::Bezier5)
            }
            PipelineNode::FragmentShader => match pin.id.input {
                0 => {
                    ui.label("@group(0) uniforms");
                    PinInfo::circle()
                        .with_fill(COLOR_UNIFORM)
                        .with_wire_style(WireStyle::Bezier5)
                }
                1 => {
                    ui.label("@group(1) textures");
                    PinInfo::square()
                        .with_fill(COLOR_TEXTURE)
                        .with_wire_style(WireStyle::Bezier5)
                }
                _ => {
                    ui.label("varyings (inter-stage)");
                    PinInfo::triangle()
                        .with_fill(COLOR_VARYING)
                        .with_wire_style(WireStyle::Bezier5)
                }
            },
            PipelineNode::Output => {
                ui.label("@location(0) vec4<f32>");
                PinInfo::circle()
                    .with_fill(COLOR_OUTPUT)
                    .with_wire_style(WireStyle::Bezier5)
            }
            _ => PinInfo::circle(),
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<PipelineNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        match &snarl[pin.id.node] {
            PipelineNode::Uniforms => {
                let paused_indicator = if self.data.paused { " (paused)" } else { "" };
                ui.monospace(format!("time: {:.2}s{paused_indicator}", self.data.time));
                ui.monospace(format!(
                    "resolution: {}x{}",
                    self.data.resolution[0] as u32, self.data.resolution[1] as u32
                ));
                ui.monospace(format!(
                    "mouse: ({:.2}, {:.2})",
                    self.data.mouse[0], self.data.mouse[1]
                ));
                ui.monospace(format!("frame: {}", self.data.frame));
                PinInfo::circle()
                    .with_fill(COLOR_UNIFORM)
                    .with_wire_style(WireStyle::Bezier5)
            }
            PipelineNode::Textures => {
                for (slot, name) in self.data.texture_names.iter().enumerate() {
                    match name {
                        Some(name) => {
                            ui.monospace(format!("[{slot}] {name}"));
                        }
                        None => {
                            ui.colored_label(
                                egui::Color32::from_rgb(0x60, 0x60, 0x60),
                                format!("[{slot}] empty"),
                            );
                        }
                    }
                }
                PinInfo::square()
                    .with_fill(COLOR_TEXTURE)
                    .with_wire_style(WireStyle::Bezier5)
            }
            PipelineNode::VertexShader => {
                let description = match self.data.render_mode {
                    RenderMode::Fullscreen => {
                        "generates fullscreen triangle\nfrom @builtin(vertex_index)"
                    }
                    RenderMode::Geometry => "transforms mesh vertices\nvia model/view/projection",
                };
                ui.monospace(description);
                PinInfo::triangle()
                    .with_fill(COLOR_VARYING)
                    .with_wire_style(WireStyle::Bezier5)
            }
            PipelineNode::FragmentShader => {
                if self.data.has_error {
                    ui.colored_label(egui::Color32::RED, "COMPILE ERROR");
                } else {
                    ui.colored_label(egui::Color32::GREEN, "compiled OK");
                }
                PinInfo::circle()
                    .with_fill(COLOR_OUTPUT)
                    .with_wire_style(WireStyle::Bezier5)
            }
            PipelineNode::BufferPass(index) => {
                let pass_index = index + 1;
                let enabled = self.data.pass_enabled[pass_index];
                let has_error = self.data.pass_errors[pass_index];

                if !enabled {
                    ui.colored_label(egui::Color32::GRAY, "disabled");
                } else if has_error {
                    ui.colored_label(egui::Color32::RED, "COMPILE ERROR");
                } else {
                    ui.colored_label(egui::Color32::GREEN, "compiled OK");
                }

                let channels = &self.data.channel_bindings[pass_index];
                for (channel_index, source) in channels.iter().enumerate() {
                    if *source != ChannelSource::None {
                        ui.monospace(format!("ch{channel_index}: {}", source.label()));
                    }
                }

                PinInfo::circle()
                    .with_fill(COLOR_BUFFER)
                    .with_wire_style(WireStyle::Bezier5)
            }
            _ => PinInfo::circle(),
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<PipelineNode>) -> bool {
        false
    }

    fn has_node_menu(&mut self, _node: &PipelineNode) -> bool {
        false
    }

    fn connect(&mut self, _from: &OutPin, _to: &InPin, _snarl: &mut Snarl<PipelineNode>) {}
}
