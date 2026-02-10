use image::GenericImageView;
use nightshade::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

mod scanner;

const DEFAULT_KENNEY_ROOT: &str = r"C:\Users\matth\Books\Kenney Game Assets All-in-1 3.2.0";
const DEFAULT_POLYHAVEN_ROOT: &str = r"C:\Users\matth\Documents\Poly Haven";
const THUMBNAIL_MAX_SIZE: u32 = 128;
const THUMBNAILS_PER_FRAME: usize = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(AssetViewer::default())?;
    Ok(())
}

#[derive(Default)]
struct AssetViewer {
    sources: Vec<scanner::AssetSource>,
    selected_source: usize,
    selected_category: Option<usize>,
    selected_pack: Option<usize>,
    pack_images: Vec<scanner::ImageFile>,
    selected_image: Option<usize>,
    thumbnail_cache: HashMap<PathBuf, egui::TextureHandle>,
    preview_texture: Option<(egui::TextureHandle, u32, u32)>,
    load_queue: VecDeque<usize>,
    pack_filter: String,
    thumbnail_size: f32,
    camera_entity: Option<Entity>,
}

impl AssetViewer {
    fn select_pack(&mut self, category_index: usize, pack_index: usize) {
        if self.selected_category == Some(category_index) && self.selected_pack == Some(pack_index)
        {
            return;
        }
        self.selected_category = Some(category_index);
        self.selected_pack = Some(pack_index);
        self.selected_image = None;
        self.preview_texture = None;
        self.thumbnail_cache.clear();
        self.load_queue.clear();

        let source = &self.sources[self.selected_source];
        let pack = &source.categories[category_index].packs[pack_index];
        self.pack_images = scanner::scan_pack_images(&pack.path);

        for index in 0..self.pack_images.len() {
            self.load_queue.push_back(index);
        }
    }

    fn select_image(&mut self, index: usize, ctx: &egui::Context) {
        self.selected_image = Some(index);
        let Some(image_file) = self.pack_images.get(index) else {
            return;
        };

        if let Some((handle, width, height)) = load_full_image(ctx, &image_file.path) {
            self.preview_texture = Some((handle, width, height));
        }
    }

    fn process_thumbnail_queue(&mut self, ctx: &egui::Context) {
        let count = THUMBNAILS_PER_FRAME.min(self.load_queue.len());
        for _ in 0..count {
            let Some(index) = self.load_queue.pop_front() else {
                break;
            };
            let Some(image_file) = self.pack_images.get(index) else {
                continue;
            };
            if self.thumbnail_cache.contains_key(&image_file.path) {
                continue;
            }
            if let Some(handle) = load_thumbnail(ctx, &image_file.path) {
                self.thumbnail_cache.insert(image_file.path.clone(), handle);
            }
        }
    }

    fn clear_selection(&mut self) {
        self.selected_category = None;
        self.selected_pack = None;
        self.selected_image = None;
        self.pack_images.clear();
        self.thumbnail_cache.clear();
        self.preview_texture = None;
        self.load_queue.clear();
    }
}

fn to_displayable_rgba(img: image::DynamicImage) -> image::RgbaImage {
    if let image::DynamicImage::ImageRgb32F(ref hdr) = img {
        let (width, height) = hdr.dimensions();
        let mut rgba = image::RgbaImage::new(width, height);
        for (x, y, pixel) in hdr.enumerate_pixels() {
            let r = pixel[0] / (1.0 + pixel[0]);
            let g = pixel[1] / (1.0 + pixel[1]);
            let b = pixel[2] / (1.0 + pixel[2]);
            let gamma = 1.0 / 2.2;
            rgba.put_pixel(
                x,
                y,
                image::Rgba([
                    (r.powf(gamma) * 255.0).clamp(0.0, 255.0) as u8,
                    (g.powf(gamma) * 255.0).clamp(0.0, 255.0) as u8,
                    (b.powf(gamma) * 255.0).clamp(0.0, 255.0) as u8,
                    255,
                ]),
            );
        }
        rgba
    } else {
        img.to_rgba8()
    }
}

fn load_thumbnail(ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let thumbnail = img.thumbnail(THUMBNAIL_MAX_SIZE, THUMBNAIL_MAX_SIZE);
    let rgba = to_displayable_rgba(thumbnail);
    let (width, height) = rgba.dimensions();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        rgba.as_raw(),
    );

    Some(ctx.load_texture(
        path.to_string_lossy().to_string(),
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}

fn load_full_image(
    ctx: &egui::Context,
    path: &Path,
) -> Option<(egui::TextureHandle, u32, u32)> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let (orig_width, orig_height) = img.dimensions();

    let max_preview = 2048;
    let display_img = if orig_width > max_preview || orig_height > max_preview {
        img.thumbnail(max_preview, max_preview)
    } else {
        img
    };
    let rgba = to_displayable_rgba(display_img);
    let (rw, rh) = rgba.dimensions();

    let color_image =
        egui::ColorImage::from_rgba_unmultiplied([rw as usize, rh as usize], rgba.as_raw());

    let handle = ctx.load_texture(
        format!("preview_{}", path.to_string_lossy()),
        color_image,
        egui::TextureOptions::LINEAR,
    );

    Some((handle, orig_width, orig_height))
}

impl State for AssetViewer {
    fn title(&self) -> &str {
        "Asset Browser"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.1, 0.1, 0.12, 1.0];
        world.resources.user_interface.enabled = true;
        self.thumbnail_size = 96.0;

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        self.camera_entity = Some(camera);

        if let Some(source) = scanner::scan_kenney(DEFAULT_KENNEY_ROOT) {
            self.sources.push(source);
        }
        if let Some(source) = scanner::scan_polyhaven(DEFAULT_POLYHAVEN_ROOT) {
            self.sources.push(source);
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
    }

    fn ui(&mut self, _world: &mut World, ctx: &egui::Context) {
        self.process_thumbnail_queue(ctx);

        let mut source_changed = false;

        egui::TopBottomPanel::top("source_tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Asset Browser");
                ui.separator();
                for (index, source) in self.sources.iter().enumerate() {
                    let label = format!("{} ({})", source.name, source.categories.iter().map(|c| c.packs.len()).sum::<usize>());
                    if ui
                        .selectable_label(self.selected_source == index, label)
                        .clicked()
                        && self.selected_source != index
                    {
                        self.selected_source = index;
                        source_changed = true;
                    }
                }
                if self.sources.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 100, 100),
                        "No asset sources found",
                    );
                }
            });
        });

        if source_changed {
            self.clear_selection();
        }

        let mut pack_action: Option<(usize, usize)> = None;

        egui::SidePanel::left("browser")
            .default_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.pack_filter);
                });
                ui.separator();

                if self.selected_source < self.sources.len() {
                    let filter_lower = self.pack_filter.to_lowercase();
                    let source = &self.sources[self.selected_source];

                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            for (cat_index, category) in source.categories.iter().enumerate() {
                                let matching_packs: Vec<usize> = category
                                    .packs
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, pack)| {
                                        filter_lower.is_empty()
                                            || pack.name.to_lowercase().contains(&filter_lower)
                                    })
                                    .map(|(index, _)| index)
                                    .collect();

                                if matching_packs.is_empty() && !filter_lower.is_empty() {
                                    continue;
                                }

                                let header_text = format!(
                                    "{} ({})",
                                    category.name,
                                    matching_packs.len()
                                );
                                egui::CollapsingHeader::new(
                                    egui::RichText::new(header_text).strong(),
                                )
                                .default_open(cat_index == 0)
                                .show(ui, |ui| {
                                    for &pack_index in &matching_packs {
                                        let pack = &category.packs[pack_index];
                                        let is_selected =
                                            self.selected_category == Some(cat_index)
                                                && self.selected_pack == Some(pack_index);

                                        if ui
                                            .selectable_label(is_selected, &pack.name)
                                            .clicked()
                                        {
                                            pack_action = Some((cat_index, pack_index));
                                        }
                                    }
                                });
                            }
                        });
                }
            });

        if let Some((cat, pack)) = pack_action {
            self.select_pack(cat, pack);
        }

        if self.preview_texture.is_some() {
            egui::SidePanel::right("preview")
                .default_width(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Preview");
                    ui.separator();

                    if let Some(index) = self.selected_image
                        && let Some(image_file) = self.pack_images.get(index)
                    {
                        ui.label(egui::RichText::new(&image_file.filename).strong());
                        if let Some((_, width, height)) = &self.preview_texture {
                            ui.label(format!("Dimensions: {} x {}", width, height));
                        }
                        if let Ok(metadata) = std::fs::metadata(&image_file.path) {
                            let size_kb = metadata.len() as f64 / 1024.0;
                            if size_kb > 1024.0 {
                                ui.label(format!("Size: {:.1} MB", size_kb / 1024.0));
                            } else {
                                ui.label(format!("Size: {:.1} KB", size_kb));
                            }
                        }
                        ui.separator();
                    }

                    if let Some((ref handle, width, height)) = self.preview_texture {
                        egui::ScrollArea::both().show(ui, |ui| {
                            let available_width = ui.available_width();
                            let scale = (available_width / width as f32).min(1.0);
                            let display_w = width as f32 * scale;
                            let display_h = height as f32 * scale;

                            ui.image(egui::load::SizedTexture::new(
                                handle.id(),
                                [display_w, display_h],
                            ));
                        });
                    }
                });
        }

        let mut image_action: Option<usize> = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(cat_idx) = self.selected_category
                    && let Some(pack_idx) = self.selected_pack
                    && self.selected_source < self.sources.len()
                {
                    let source = &self.sources[self.selected_source];
                    let pack = &source.categories[cat_idx].packs[pack_idx];
                    ui.heading(&pack.name);
                    ui.separator();
                    ui.label(format!("{} images", self.pack_images.len()));
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

            if !self.load_queue.is_empty() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!(
                        "Loading thumbnails... ({}/{})",
                        self.thumbnail_cache.len(),
                        self.pack_images.len()
                    ));
                });
            }

            if self.pack_images.is_empty() {
                if self.selected_pack.is_some() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No images found in this pack");
                    });
                }
            } else {
                draw_thumbnail_grid(
                    ui,
                    &self.pack_images,
                    &self.thumbnail_cache,
                    self.selected_image,
                    self.thumbnail_size,
                    &mut image_action,
                );
            }
        });

        if let Some(index) = image_action {
            self.select_image(index, ctx);
        }
    }
}

fn draw_thumbnail_grid(
    ui: &mut egui::Ui,
    pack_images: &[scanner::ImageFile],
    thumbnail_cache: &HashMap<PathBuf, egui::TextureHandle>,
    selected_image: Option<usize>,
    thumbnail_size: f32,
    image_action: &mut Option<usize>,
) {
    let spacing = 8.0;
    let cell_width = thumbnail_size;
    let cell_height = thumbnail_size + 18.0;

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            let available_width = ui.available_width();
            let columns = ((available_width + spacing) / (cell_width + spacing))
                .floor()
                .max(1.0) as usize;
            let rows = pack_images.len().div_ceil(columns);

            for row in 0..rows {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                    for col in 0..columns {
                        let index = row * columns + col;
                        if index >= pack_images.len() {
                            break;
                        }

                        let image_file = &pack_images[index];
                        let is_selected = selected_image == Some(index);

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

                        if let Some(handle) = thumbnail_cache.get(&image_file.path) {
                            let [tw, th] = handle.size();
                            let scale = (thumbnail_size / tw as f32)
                                .min(thumbnail_size / th as f32)
                                .min(1.0);
                            let img_size = egui::vec2(tw as f32 * scale, th as f32 * scale);
                            let img_rect =
                                egui::Rect::from_center_size(thumb_rect.center(), img_size);
                            ui.painter().image(
                                handle.id(),
                                img_rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                egui::Color32::WHITE,
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

                        let label_center = egui::pos2(rect.center().x, thumb_rect.max.y + 9.0);
                        let max_chars = (thumbnail_size / 7.0) as usize;
                        let display_name = if image_file.filename.len() > max_chars {
                            format!(
                                "{}...",
                                &image_file.filename[..max_chars.saturating_sub(3)]
                            )
                        } else {
                            image_file.filename.clone()
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
                            *image_action = Some(index);
                        }
                    }
                });
            }
        });
}
