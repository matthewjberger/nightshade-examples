use nightshade::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

use crate::scanner;
use crate::thumbnail;

const THUMBNAIL_MAX_SIZE: u32 = 128;
const THUMBNAILS_PER_FRAME: usize = 3;

pub enum PaneAction {
    OpenModelViewer {
        path: PathBuf,
        filename: String,
    },
    CloseModelViewer {
        orbit_camera: Entity,
        sun: Entity,
        model: Option<Entity>,
    },
    SelectAsset {
        index: usize,
    },
}

pub enum Pane {
    Browser(Box<BrowserPane>),
    ModelViewer(ModelViewerPane),
}

pub struct ModelViewerPane {
    pub display_name: String,
    pub orbit_camera_entity: Entity,
    pub sun_entity: Entity,
    pub model_entity: Option<Entity>,
}

pub struct BrowserPane {
    pub selected_category: Option<usize>,
    pub selected_pack: Option<usize>,
    pub current_pack_name: String,
    pub pack_assets: Vec<scanner::AssetFile>,
    pub selected_asset: Option<usize>,
    pub thumbnail_cache: HashMap<PathBuf, egui::TextureHandle>,
    pub stale_textures: Vec<egui::TextureHandle>,
    pub load_queue: VecDeque<usize>,
    pub total_to_load: usize,
    pub thumbnail_size: f32,
    pub model_thumbnail_queue: VecDeque<usize>,
    pub model_thumbnail_cache: HashMap<PathBuf, egui::TextureHandle>,
}

impl BrowserPane {
    pub fn new() -> Self {
        Self {
            selected_category: None,
            selected_pack: None,
            current_pack_name: String::new(),
            pack_assets: Vec::new(),
            selected_asset: None,
            thumbnail_cache: HashMap::new(),
            stale_textures: Vec::new(),
            load_queue: VecDeque::new(),
            total_to_load: 0,
            thumbnail_size: 96.0,
            model_thumbnail_queue: VecDeque::new(),
            model_thumbnail_cache: HashMap::new(),
        }
    }

    pub fn select_pack(
        &mut self,
        category_index: usize,
        pack_index: usize,
        source: &scanner::AssetSource,
    ) {
        if self.selected_category == Some(category_index)
            && self.selected_pack == Some(pack_index)
        {
            return;
        }
        self.selected_category = Some(category_index);
        self.selected_pack = Some(pack_index);
        self.selected_asset = None;
        self.stale_textures
            .extend(self.thumbnail_cache.drain().map(|(_, handle)| handle));
        self.stale_textures
            .extend(self.model_thumbnail_cache.drain().map(|(_, handle)| handle));
        self.load_queue.clear();
        self.model_thumbnail_queue.clear();

        let pack = &source.categories[category_index].packs[pack_index];
        self.current_pack_name = pack.name.clone();
        self.pack_assets = scanner::scan_pack_assets(&pack.path);

        self.total_to_load = self.pack_assets.len();
        for index in 0..self.pack_assets.len() {
            self.load_queue.push_back(index);
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_category = None;
        self.selected_pack = None;
        self.selected_asset = None;
        self.current_pack_name.clear();
        self.pack_assets.clear();
        self.stale_textures
            .extend(self.thumbnail_cache.drain().map(|(_, handle)| handle));
        self.stale_textures
            .extend(self.model_thumbnail_cache.drain().map(|(_, handle)| handle));
        self.load_queue.clear();
        self.model_thumbnail_queue.clear();
        self.total_to_load = 0;
    }

    pub fn process_thumbnail_queue(&mut self, ctx: &egui::Context) {
        self.stale_textures.clear();

        let count = THUMBNAILS_PER_FRAME.min(self.load_queue.len());
        for _ in 0..count {
            let Some(index) = self.load_queue.pop_front() else {
                break;
            };
            let Some(asset_file) = self.pack_assets.get(index) else {
                continue;
            };
            if asset_file.kind == scanner::AssetFileKind::Model {
                self.model_thumbnail_queue.push_back(index);
                continue;
            }
            if self.thumbnail_cache.contains_key(&asset_file.path) {
                continue;
            }
            if let Some(handle) = crate::load_thumbnail(ctx, &asset_file.path) {
                self.thumbnail_cache.insert(asset_file.path.clone(), handle);
            }
        }

        if !self.load_queue.is_empty() || !self.model_thumbnail_queue.is_empty() {
            ctx.request_repaint();
        }
    }

    pub fn process_model_thumbnail_queue(&mut self, ctx: &egui::Context) {
        if let Some(index) = self.model_thumbnail_queue.pop_front()
            && let Some(asset_file) = self.pack_assets.get(index)
            && !self.model_thumbnail_cache.contains_key(&asset_file.path)
            && let Some(color_image) =
                thumbnail::generate_model_thumbnail(&asset_file.path, THUMBNAIL_MAX_SIZE)
        {
            let handle = ctx.load_texture(
                format!("model_thumb_{}", asset_file.path.to_string_lossy()),
                color_image,
                egui::TextureOptions::LINEAR,
            );
            self.model_thumbnail_cache
                .insert(asset_file.path.clone(), handle);
        }

        if !self.model_thumbnail_queue.is_empty() {
            ctx.request_repaint();
        }
    }

    pub fn draw_grid_ui(&mut self, ui: &mut egui::Ui, actions: &mut Vec<PaneAction>) {
        ui.horizontal(|ui| {
            if !self.current_pack_name.is_empty() {
                ui.heading(&self.current_pack_name);
                ui.separator();
                ui.label(format!("{} assets", self.pack_assets.len()));
            } else {
                ui.heading("Select a pack to browse");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::Slider::new(&mut self.thumbnail_size, 48.0..=256.0).text("Size"),
                );
            });
        });
        ui.separator();

        if !self.load_queue.is_empty() && self.total_to_load > 0 {
            let loaded = self.total_to_load - self.load_queue.len();
            let progress = loaded as f32 / self.total_to_load as f32;

            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(format!(
                    "Loading thumbnails... {}/{}",
                    loaded, self.total_to_load
                ));
            });
            ui.add(
                egui::ProgressBar::new(progress)
                    .show_percentage()
                    .animate(true),
            );
            ui.add_space(4.0);
        }

        if self.pack_assets.is_empty() {
            if self.selected_pack.is_some() {
                ui.centered_and_justified(|ui| {
                    ui.label("No assets found in this pack");
                });
            }
        } else {
            self.draw_thumbnail_grid(ui, actions);
        }
    }

    fn draw_thumbnail_grid(&mut self, ui: &mut egui::Ui, actions: &mut Vec<PaneAction>) {
        let spacing = 8.0;
        let cell_width = self.thumbnail_size;
        let cell_height = self.thumbnail_size + 18.0;
        let thumbnail_size = self.thumbnail_size;

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let available_width = ui.available_width();
                let columns = ((available_width + spacing) / (cell_width + spacing))
                    .floor()
                    .max(1.0) as usize;
                let rows = self.pack_assets.len().div_ceil(columns);

                for row in 0..rows {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                        for col in 0..columns {
                            let index = row * columns + col;
                            if index >= self.pack_assets.len() {
                                break;
                            }

                            let is_selected = self.selected_asset == Some(index);

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(cell_width, cell_height),
                                egui::Sense::click(),
                            );

                            let thumb_rect = egui::Rect::from_min_size(
                                rect.min,
                                egui::vec2(thumbnail_size, thumbnail_size),
                            );

                            let bg_color = if is_selected {
                                egui::Color32::from_rgba_premultiplied(50, 70, 110, 200)
                            } else if response.hovered() {
                                egui::Color32::from_gray(45)
                            } else {
                                egui::Color32::from_gray(30)
                            };
                            ui.painter().rect_filled(thumb_rect, 4.0, bg_color);

                            let asset_kind = self.pack_assets[index].kind;
                            let asset_path = &self.pack_assets[index].path;

                            if asset_kind == scanner::AssetFileKind::Model {
                                if let Some(handle) =
                                    self.model_thumbnail_cache.get(asset_path)
                                {
                                    Self::paint_thumbnail_image(
                                        ui,
                                        handle,
                                        thumb_rect,
                                        thumbnail_size,
                                    );
                                } else {
                                    ui.painter().text(
                                        thumb_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "[3D]",
                                        egui::FontId::proportional(16.0),
                                        egui::Color32::from_rgb(150, 200, 255),
                                    );
                                }
                            } else if let Some(handle) =
                                self.thumbnail_cache.get(asset_path)
                            {
                                Self::paint_thumbnail_image(
                                    ui,
                                    handle,
                                    thumb_rect,
                                    thumbnail_size,
                                );
                            }

                            if is_selected {
                                ui.painter().rect_stroke(
                                    thumb_rect,
                                    4.0,
                                    egui::Stroke::new(
                                        2.0,
                                        egui::Color32::from_rgb(100, 150, 255),
                                    ),
                                    egui::StrokeKind::Outside,
                                );
                            }

                            let label_center =
                                egui::pos2(rect.center().x, thumb_rect.max.y + 9.0);
                            let max_chars = (thumbnail_size / 7.0) as usize;
                            let filename = &self.pack_assets[index].filename;
                            let display_name = if filename.len() > max_chars {
                                format!(
                                    "{}...",
                                    &filename[..max_chars.saturating_sub(3)]
                                )
                            } else {
                                filename.clone()
                            };
                            ui.painter().text(
                                label_center,
                                egui::Align2::CENTER_CENTER,
                                &display_name,
                                egui::FontId::proportional(11.0),
                                if is_selected {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::GRAY
                                },
                            );

                            if response.clicked() {
                                self.selected_asset = Some(index);
                                if asset_kind == scanner::AssetFileKind::Model {
                                    actions.push(PaneAction::OpenModelViewer {
                                        path: self.pack_assets[index].path.clone(),
                                        filename: self.pack_assets[index]
                                            .filename
                                            .clone(),
                                    });
                                } else {
                                    actions.push(PaneAction::SelectAsset { index });
                                }
                            }
                        }
                    });
                }
            });
    }

    fn paint_thumbnail_image(
        ui: &egui::Ui,
        handle: &egui::TextureHandle,
        thumb_rect: egui::Rect,
        thumbnail_size: f32,
    ) {
        let [texture_width, texture_height] = handle.size();
        let scale = (thumbnail_size / texture_width as f32)
            .min(thumbnail_size / texture_height as f32)
            .min(1.0);
        let img_size = egui::vec2(texture_width as f32 * scale, texture_height as f32 * scale);
        let img_rect = egui::Rect::from_center_size(thumb_rect.center(), img_size);
        ui.painter().image(
            handle.id(),
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}
