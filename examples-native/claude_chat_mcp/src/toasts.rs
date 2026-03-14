use std::collections::VecDeque;

use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
}

impl ToastKind {
    fn accent_color(self) -> egui::Color32 {
        match self {
            ToastKind::Error => egui::Color32::from_rgb(255, 80, 80),
            ToastKind::Success => egui::Color32::from_rgb(80, 200, 120),
        }
    }
}

struct ToastEntry {
    message: String,
    kind: ToastKind,
    spawn_time: f32,
    duration: f32,
}

pub struct Toasts {
    entries: VecDeque<ToastEntry>,
    elapsed_time: f32,
}

impl Toasts {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            elapsed_time: 0.0,
        }
    }

    pub fn push(&mut self, kind: ToastKind, message: impl Into<String>, duration: f32) {
        self.entries.push_back(ToastEntry {
            message: message.into(),
            kind,
            spawn_time: self.elapsed_time,
            duration,
        });
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.elapsed_time += delta_time;
        while let Some(front) = self.entries.front() {
            if self.elapsed_time - front.spawn_time >= front.duration {
                self.entries.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn render(&self, ui_context: &egui::Context) {
        if self.entries.is_empty() {
            return;
        }

        egui::Area::new(egui::Id::new("toast_area"))
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -16.0))
            .order(egui::Order::Foreground)
            .show(ui_context, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    for (index, entry) in self.entries.iter().enumerate() {
                        let age = self.elapsed_time - entry.spawn_time;
                        let fade_start = entry.duration - 0.5;
                        let alpha = if age > fade_start {
                            ((entry.duration - age) / 0.5).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };

                        let bg_alpha = (200.0 * alpha) as u8;
                        let text_alpha = (255.0 * alpha) as u8;
                        let accent = entry.kind.accent_color();

                        egui::Frame::NONE
                            .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 50, bg_alpha))
                            .inner_margin(egui::Margin::symmetric(12, 8))
                            .corner_radius(egui::CornerRadius::same(6))
                            .stroke(egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_unmultiplied(
                                    accent.r(),
                                    accent.g(),
                                    accent.b(),
                                    bg_alpha,
                                ),
                            ))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&entry.message)
                                        .color(egui::Color32::from_rgba_unmultiplied(
                                            230, 230, 240, text_alpha,
                                        ))
                                        .size(14.0),
                                );
                            });

                        if index < self.entries.len() - 1 {
                            ui.add_space(4.0);
                        }
                    }
                });
            });

        ui_context.request_repaint();
    }
}
