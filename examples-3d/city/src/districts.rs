use nightshade::prelude::*;
use noise::{NoiseFn, Perlin};

use crate::city::CHUNK_SIZE;

const DISPLAY_DURATION: f32 = 3.0;
const FADE_DURATION: f32 = 0.8;

const DOWNTOWN_NAMES: &[&[&str]] = &[
    &["Financial District", "Midtown"],
    &["City Center", "Commerce Row"],
    &["Downtown Core", "Capital Square"],
    &["Tower District", "Exchange Plaza"],
];

const MIXED_NAMES: &[&[&str]] = &[
    &["Eastside", "Riverside"],
    &["Westgate", "Harbor View"],
    &["Midtown West", "Canal Street"],
    &["Union District", "Market Quarter"],
];

const RESIDENTIAL_NAMES: &[&[&str]] = &[
    &["Greenfield", "Oak Park"],
    &["Maple Heights", "Willowbrook"],
    &["Cedar Lane", "Pinewood"],
    &["Birch Hill", "Elm Gardens"],
];

fn quadrant_index(chunk_x: i32, chunk_z: i32) -> usize {
    let col = if chunk_x >= 0 { 1 } else { 0 };
    let row = if chunk_z >= 0 { 1 } else { 0 };
    row * 2 + col
}

fn sample_district_noise(noise: &Perlin, chunk_x: i32, chunk_z: i32) -> f32 {
    let scale = 0.05;
    let value = noise.get([chunk_x as f64 * scale, chunk_z as f64 * scale]);
    ((value + 1.0) / 2.0).clamp(0.0, 1.0) as f32
}

pub fn district_name(chunk_x: i32, chunk_z: i32, noise: &Perlin) -> &'static str {
    let height_influence = sample_district_noise(noise, chunk_x, chunk_z);
    let quadrant = quadrant_index(chunk_x, chunk_z);

    let pool = if height_influence > 0.7 {
        DOWNTOWN_NAMES
    } else if height_influence > 0.4 {
        MIXED_NAMES
    } else {
        RESIDENTIAL_NAMES
    };

    let group = &pool[quadrant % pool.len()];
    let sub_index = ((chunk_x.wrapping_abs() + chunk_z.wrapping_abs()) as usize / 3) % group.len();
    group[sub_index]
}

enum FadeState {
    Hidden,
    FadingIn,
    Visible,
    FadingOut,
}

pub struct DistrictHud {
    current_district: Option<&'static str>,
    fade_alpha: f32,
    fade_state: FadeState,
    display_timer: f32,
    noise: Perlin,
    last_chunk: Option<(i32, i32)>,
}

impl DistrictHud {
    pub fn new(noise_seed: u32) -> Self {
        Self {
            current_district: None,
            fade_alpha: 0.0,
            fade_state: FadeState::Hidden,
            display_timer: 0.0,
            noise: Perlin::new(noise_seed),
            last_chunk: None,
        }
    }

    pub fn update(&mut self, camera_position: Vec3, delta_time: f32) {
        let chunk_x = (camera_position.x / CHUNK_SIZE).floor() as i32;
        let chunk_z = (camera_position.z / CHUNK_SIZE).floor() as i32;
        let current_chunk = (chunk_x, chunk_z);

        if self.last_chunk != Some(current_chunk) {
            self.last_chunk = Some(current_chunk);
            let name = district_name(chunk_x, chunk_z, &self.noise);

            if self.current_district != Some(name) {
                self.current_district = Some(name);
                self.fade_state = FadeState::FadingIn;
                self.fade_alpha = 0.0;
                self.display_timer = 0.0;
            }
        }

        match self.fade_state {
            FadeState::Hidden => {}
            FadeState::FadingIn => {
                self.fade_alpha = (self.fade_alpha + delta_time / FADE_DURATION).min(1.0);
                if self.fade_alpha >= 1.0 {
                    self.fade_state = FadeState::Visible;
                    self.display_timer = 0.0;
                }
            }
            FadeState::Visible => {
                self.display_timer += delta_time;
                if self.display_timer >= DISPLAY_DURATION {
                    self.fade_state = FadeState::FadingOut;
                }
            }
            FadeState::FadingOut => {
                self.fade_alpha = (self.fade_alpha - delta_time / FADE_DURATION).max(0.0);
                if self.fade_alpha <= 0.0 {
                    self.fade_state = FadeState::Hidden;
                }
            }
        }
    }

    pub fn draw(&self, ui_context: &egui::Context) {
        if self.fade_alpha <= 0.0 {
            return;
        }

        let Some(name) = self.current_district else {
            return;
        };

        let alpha = (self.fade_alpha * 255.0) as u8;

        egui::Area::new(egui::Id::new("district_hud"))
            .anchor(egui::Align2::CENTER_TOP, [0.0, 60.0])
            .show(ui_context, |ui| {
                ui.set_min_width(400.0);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(name)
                            .size(32.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha))
                            .strong(),
                    );
                });
            });
    }
}
