use image::GenericImageView;
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::prefab::mesh_cache_insert;
use nightshade::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

mod scanner;

const DEFAULT_KENNEY_ROOT: &str = r"C:\Users\matth\Books\Kenney Game Assets All-in-1 3.2.0";
const DEFAULT_POLYHAVEN_ROOT: &str = r"C:\Users\matth\Documents\Poly Haven";
const THUMBNAIL_MAX_SIZE: u32 = 128;
const THUMBNAILS_PER_FRAME: usize = 3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(AssetViewer::default())?;
    Ok(())
}

#[derive(Default, PartialEq, Eq)]
enum ViewMode {
    #[default]
    Browse,
    Model3d,
    Skybox,
}

struct AssetViewer {
    sources: Vec<scanner::AssetSource>,
    selected_source: usize,
    selected_category: Option<usize>,
    selected_pack: Option<usize>,
    pack_assets: Vec<scanner::AssetFile>,
    selected_asset: Option<usize>,
    thumbnail_cache: HashMap<PathBuf, egui::TextureHandle>,
    stale_textures: Vec<egui::TextureHandle>,
    preview_texture: Option<(egui::TextureHandle, u32, u32)>,
    load_queue: VecDeque<usize>,
    total_to_load: usize,
    pack_filter: String,
    thumbnail_size: f32,
    ortho_camera_entity: Option<Entity>,
    view_mode: ViewMode,
    orbit_camera_entity: Option<Entity>,
    sun_entity: Option<Entity>,
    model_entity: Option<Entity>,
    active_model_path: Option<PathBuf>,
    scene_initialized: bool,
}

impl Default for AssetViewer {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            selected_source: 0,
            selected_category: None,
            selected_pack: None,
            pack_assets: Vec::new(),
            selected_asset: None,
            thumbnail_cache: HashMap::new(),
            stale_textures: Vec::new(),
            preview_texture: None,
            load_queue: VecDeque::new(),
            total_to_load: 0,
            pack_filter: String::new(),
            thumbnail_size: 96.0,
            ortho_camera_entity: None,
            view_mode: ViewMode::Browse,
            orbit_camera_entity: None,
            sun_entity: None,
            model_entity: None,
            active_model_path: None,
            scene_initialized: false,
        }
    }
}

impl AssetViewer {
    fn select_pack(&mut self, category_index: usize, pack_index: usize) {
        if self.selected_category == Some(category_index) && self.selected_pack == Some(pack_index)
        {
            return;
        }
        self.selected_category = Some(category_index);
        self.selected_pack = Some(pack_index);
        self.selected_asset = None;
        self.preview_texture = None;
        self.stale_textures
            .extend(self.thumbnail_cache.drain().map(|(_, handle)| handle));
        self.load_queue.clear();

        let source = &self.sources[self.selected_source];
        let pack = &source.categories[category_index].packs[pack_index];
        self.pack_assets = scanner::scan_pack_assets(&pack.path);

        self.total_to_load = self.pack_assets.len();
        for index in 0..self.pack_assets.len() {
            self.load_queue.push_back(index);
        }
    }

    fn select_asset(&mut self, index: usize, ctx: &egui::Context, world: &mut World) {
        self.selected_asset = Some(index);
        let Some(asset_file) = self.pack_assets.get(index) else {
            return;
        };

        let kind = asset_file.kind;
        let path = asset_file.path.clone();

        match kind {
            scanner::AssetFileKind::Image => {
                if self.view_mode != ViewMode::Browse {
                    self.enter_browse_mode(world);
                }
                if let Some((handle, width, height)) = load_full_image(ctx, &path) {
                    self.preview_texture = Some((handle, width, height));
                }
            }
            scanner::AssetFileKind::Model => {
                self.preview_texture = None;
                self.load_model(world, &path);
            }
            scanner::AssetFileKind::Hdr => {
                if let Some((handle, width, height)) = load_full_image(ctx, &path) {
                    self.preview_texture = Some((handle, width, height));
                }
                self.load_skybox(world, &path);
            }
        }
    }

    fn ensure_3d_scene(&mut self, world: &mut World) {
        if !self.scene_initialized {
            let orbit_camera = spawn_pan_orbit_camera(
                world,
                Vec3::new(0.0, 1.0, 0.0),
                5.0,
                0.0,
                0.3,
                "Orbit Camera".to_string(),
            );
            self.orbit_camera_entity = Some(orbit_camera);

            let sun = spawn_sun(world);
            self.sun_entity = Some(sun);

            self.scene_initialized = true;
        }

        if let Some(orbit_camera) = self.orbit_camera_entity {
            world.resources.active_camera = Some(orbit_camera);
        }

        world.resources.graphics.atmosphere = Atmosphere::Hdr;
        world.resources.graphics.show_grid = true;
    }

    fn enter_browse_mode(&mut self, world: &mut World) {
        if let Some(model) = self.model_entity.take() {
            despawn_recursive_immediate(world, model);
        }
        self.active_model_path = None;

        if let Some(ortho_camera) = self.ortho_camera_entity {
            world.resources.active_camera = Some(ortho_camera);
        }

        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.show_grid = false;
        self.view_mode = ViewMode::Browse;
    }

    fn load_model(&mut self, world: &mut World, path: &Path) {
        if self.active_model_path.as_deref() == Some(path) {
            return;
        }

        if let Some(model) = self.model_entity.take() {
            despawn_recursive_immediate(world, model);
        }

        self.ensure_3d_scene(world);

        if let Ok(result) = nightshade::ecs::prefab::import_gltf_from_path(path) {
            for (name, (rgba_data, width, height)) in result.textures {
                world.queue_command(WorldCommand::LoadTexture {
                    name,
                    rgba_data,
                    width,
                    height,
                });
            }

            for (name, mesh) in result.meshes {
                mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
            }

            if let Some(prefab) = result.prefabs.first() {
                let entity = if !result.skins.is_empty() {
                    nightshade::ecs::prefab::spawn_prefab_with_skins(
                        world,
                        prefab,
                        &result.animations,
                        &result.skins,
                        Vec3::zeros(),
                    )
                } else {
                    nightshade::ecs::prefab::spawn_prefab(world, prefab, Vec3::zeros())
                };

                if let Some(player) = world.get_animation_player_mut(entity)
                    && !player.clips.is_empty()
                {
                    player.play(0);
                    player.looping = true;
                }

                self.model_entity = Some(entity);
            }

            self.active_model_path = Some(path.to_path_buf());
            self.view_mode = ViewMode::Model3d;
        }
    }

    fn load_skybox(&mut self, world: &mut World, path: &Path) {
        if let Some(model) = self.model_entity.take() {
            despawn_recursive_immediate(world, model);
        }
        self.active_model_path = None;

        self.ensure_3d_scene(world);
        load_hdr_skybox_from_path(world, path.to_path_buf());
        self.view_mode = ViewMode::Skybox;
    }

    fn process_thumbnail_queue(&mut self, ctx: &egui::Context) {
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
                continue;
            }
            if self.thumbnail_cache.contains_key(&asset_file.path) {
                continue;
            }
            if let Some(handle) = load_thumbnail(ctx, &asset_file.path) {
                self.thumbnail_cache.insert(asset_file.path.clone(), handle);
            }
        }

        if !self.load_queue.is_empty() {
            ctx.request_repaint();
        }
    }

    fn clear_selection(&mut self) {
        self.selected_category = None;
        self.selected_pack = None;
        self.selected_asset = None;
        self.pack_assets.clear();
        self.stale_textures
            .extend(self.thumbnail_cache.drain().map(|(_, handle)| handle));
        self.preview_texture = None;
        self.load_queue.clear();
        self.total_to_load = 0;
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

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        self.ortho_camera_entity = Some(camera);

        if let Some(source) = scanner::scan_kenney(DEFAULT_KENNEY_ROOT) {
            self.sources.push(source);
        }
        if let Some(source) = scanner::scan_polyhaven(DEFAULT_POLYHAVEN_ROOT) {
            self.sources.push(source);
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        match self.view_mode {
            ViewMode::Browse => {
                escape_key_exit_system(world);
            }
            ViewMode::Model3d | ViewMode::Skybox => {
                pan_orbit_camera_system(world);

                let escape_pressed = world
                    .resources
                    .input
                    .keyboard
                    .frame_keys
                    .iter()
                    .any(|(key, pressed)| *key == KeyCode::Escape && *pressed);

                if escape_pressed {
                    self.enter_browse_mode(world);
                }
            }
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        self.process_thumbnail_queue(ctx);

        let mut source_changed = false;

        egui::TopBottomPanel::top("source_tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Asset Browser");
                ui.separator();
                for (index, source) in self.sources.iter().enumerate() {
                    let label = format!("{} ({})", source.name, source.categories.iter().map(|category| category.packs.len()).sum::<usize>());
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
            if self.view_mode != ViewMode::Browse {
                self.enter_browse_mode(world);
            }
        }

        match self.view_mode {
            ViewMode::Browse => {
                self.draw_browse_ui(world, ctx);
            }
            ViewMode::Model3d | ViewMode::Skybox => {
                self.draw_3d_overlay_ui(ctx);
            }
        }
    }
}

impl AssetViewer {
    fn draw_browse_ui(&mut self, world: &mut World, ctx: &egui::Context) {
        if self.preview_texture.is_some() {
            egui::SidePanel::right("preview")
                .default_width(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Preview");
                    ui.separator();

                    if let Some(index) = self.selected_asset
                        && let Some(asset_file) = self.pack_assets.get(index)
                    {
                        ui.label(egui::RichText::new(&asset_file.filename).strong());
                        if let Some((_, width, height)) = &self.preview_texture {
                            ui.label(format!("Dimensions: {} x {}", width, height));
                        }
                        if let Ok(metadata) = std::fs::metadata(&asset_file.path) {
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

        let mut asset_action: Option<usize> = None;

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
                draw_thumbnail_grid(
                    ui,
                    &self.pack_assets,
                    &self.thumbnail_cache,
                    self.selected_asset,
                    self.thumbnail_size,
                    &mut asset_action,
                );
            }
        });

        if let Some(index) = asset_action {
            self.select_asset(index, ctx, world);
        }
    }

    fn draw_3d_overlay_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(index) = self.selected_asset
                        && let Some(asset_file) = self.pack_assets.get(index)
                    {
                        ui.label(
                            egui::RichText::new(&asset_file.filename)
                                .strong()
                                .color(egui::Color32::WHITE)
                                .size(16.0),
                        );
                    }

                    if ui.button("Back (Esc)").clicked() {
                        self.view_mode = ViewMode::Browse;
                    }
                });
            });
    }
}

fn draw_thumbnail_grid(
    ui: &mut egui::Ui,
    pack_assets: &[scanner::AssetFile],
    thumbnail_cache: &HashMap<PathBuf, egui::TextureHandle>,
    selected_asset: Option<usize>,
    thumbnail_size: f32,
    asset_action: &mut Option<usize>,
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
            let rows = pack_assets.len().div_ceil(columns);

            for row in 0..rows {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                    for col in 0..columns {
                        let index = row * columns + col;
                        if index >= pack_assets.len() {
                            break;
                        }

                        let asset_file = &pack_assets[index];
                        let is_selected = selected_asset == Some(index);

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

                        if asset_file.kind == scanner::AssetFileKind::Model {
                            ui.painter().text(
                                thumb_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "[3D]",
                                egui::FontId::proportional(16.0),
                                egui::Color32::from_rgb(150, 200, 255),
                            );
                        } else if let Some(handle) = thumbnail_cache.get(&asset_file.path) {
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
                        let display_name = if asset_file.filename.len() > max_chars {
                            format!(
                                "{}...",
                                &asset_file.filename[..max_chars.saturating_sub(3)]
                            )
                        } else {
                            asset_file.filename.clone()
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
                            *asset_action = Some(index);
                        }
                    }
                });
            }
        });
}
