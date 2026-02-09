use image::GenericImageView;
use nightshade::ecs::animation::systems::{apply_animations, update_animation_players};
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::prefab::mesh_cache_insert;
use nightshade::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

mod pane;
mod scanner;
mod thumbnail;

use pane::{BrowserPane, ModelViewerPane, Pane, PaneAction};

const DEFAULT_KENNEY_ROOT: &str = r"C:\Users\matth\Books\Kenney Game Assets All-in-1 3.2.0";
const DEFAULT_POLYHAVEN_ROOT: &str = r"C:\Users\matth\Documents\Poly Haven";
const THUMBNAIL_MAX_SIZE: u32 = 128;
const MODEL_VIEWER_INITIAL_WIDTH: u32 = 1920;
const MODEL_VIEWER_INITIAL_HEIGHT: u32 = 1080;

static NEXT_VIEWER_WORLD_ID: AtomicU64 = AtomicU64::new(20000);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(AssetViewer::default())?;
    Ok(())
}

struct ModelViewerInstance {
    world: World,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    egui_texture_id: Option<egui::TextureId>,
    model_path: PathBuf,
    width: u32,
    height: u32,
    desired_width: u32,
    desired_height: u32,
    surface_format: wgpu::TextureFormat,
}

#[derive(Default)]
struct AssetViewer {
    sources: Vec<scanner::AssetSource>,
    selected_source: usize,
    pack_filter: String,
    tile_tree: Option<egui_tiles::Tree<Pane>>,
    ortho_camera_entity: Option<Entity>,
    preview_texture: Option<(egui::TextureHandle, u32, u32)>,
    gpu_thumbnails: HashMap<PathBuf, thumbnail::GpuThumbnail>,
    model_viewer: Option<ModelViewerInstance>,
    pending_model_open: Option<(PathBuf, String)>,
    pending_thumbnail_data: Vec<(PathBuf, nightshade::ecs::prefab::GltfLoadResult)>,
}

struct AssetBrowserBehavior<'a> {
    actions: Vec<PaneAction>,
    active_viewer_rect: Option<egui::Rect>,
    gpu_thumbnails: &'a HashMap<PathBuf, thumbnail::GpuThumbnail>,
    model_viewer_texture_id: Option<egui::TextureId>,
    model_viewer_texture_size: Option<(u32, u32)>,
    model_viewer_desired_size: Option<(u32, u32)>,
    pixels_per_point: f32,
}

impl<'a> egui_tiles::Behavior<Pane> for AssetBrowserBehavior<'a> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Browser(_) => "Browser".into(),
            Pane::ModelViewer(viewer) => viewer.display_name.clone().into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match pane {
            Pane::Browser(browser) => {
                browser.draw_grid_ui(ui, &mut self.actions, self.gpu_thumbnails);
            }
            Pane::ModelViewer(viewer) => {
                self.draw_model_viewer_ui(ui, viewer);
            }
        }
        egui_tiles::UiResponse::None
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }

    fn tab_bar_height(&self, _style: &egui::Style) -> f32 {
        24.0
    }

    fn is_tab_closable(
        &self,
        tiles: &egui_tiles::Tiles<Pane>,
        tile_id: egui_tiles::TileId,
    ) -> bool {
        matches!(
            tiles.get(tile_id),
            Some(egui_tiles::Tile::Pane(Pane::ModelViewer(_)))
        )
    }

    fn on_tab_close(
        &mut self,
        tiles: &mut egui_tiles::Tiles<Pane>,
        tile_id: egui_tiles::TileId,
    ) -> bool {
        if let Some(egui_tiles::Tile::Pane(Pane::ModelViewer(_))) = tiles.get(tile_id) {
            self.actions.push(PaneAction::CloseModelViewer);
        }
        true
    }
}

impl AssetBrowserBehavior<'_> {
    fn draw_model_viewer_ui(&mut self, ui: &mut egui::Ui, viewer: &ModelViewerPane) {
        let rect = ui.available_rect_before_wrap();

        let physical_w = (rect.width() * self.pixels_per_point).round() as u32;
        let physical_h = (rect.height() * self.pixels_per_point).round() as u32;
        if physical_w > 0 && physical_h > 0 {
            self.model_viewer_desired_size = Some((physical_w, physical_h));
        }

        if let Some(texture_id) = self.model_viewer_texture_id
            && let Some((tex_w, tex_h)) = self.model_viewer_texture_size
        {
            let tex_w = tex_w as f32;
            let tex_h = tex_h as f32;
            let tile_w = rect.width();
            let tile_h = rect.height();

            let tex_aspect = tex_w / tex_h;
            let tile_aspect = tile_w / tile_h;

            let uv_rect = if tile_aspect > tex_aspect {
                let uv_height = tex_aspect / tile_aspect;
                let uv_y = (1.0 - uv_height) / 2.0;
                egui::Rect::from_min_max(egui::pos2(0.0, uv_y), egui::pos2(1.0, uv_y + uv_height))
            } else {
                let uv_width = tile_aspect / tex_aspect;
                let uv_x = (1.0 - uv_width) / 2.0;
                egui::Rect::from_min_max(egui::pos2(uv_x, 0.0), egui::pos2(uv_x + uv_width, 1.0))
            };

            let image = egui::Image::new(egui::load::SizedTexture::new(
                texture_id,
                egui::vec2(tile_w, tile_h),
            ))
            .uv(uv_rect);
            image.paint_at(ui, rect);
        }

        let response = ui.allocate_rect(rect, egui::Sense::click());
        if response.hovered() {
            self.active_viewer_rect = Some(rect);

            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 165, 0)),
                egui::StrokeKind::Inside,
            );
        }

        ui.painter().text(
            rect.left_top() + egui::vec2(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            &viewer.display_name,
            egui::FontId::proportional(14.0),
            egui::Color32::WHITE,
        );
    }
}

impl AssetViewer {
    fn create_model_viewer(
        &mut self,
        renderer: &mut dyn Render,
        main_world: &mut World,
        path: PathBuf,
        filename: String,
    ) {
        self.model_viewer = None;

        let mut viewer_world = World::default();
        renderer.copy_fonts_to_world(&mut viewer_world);

        viewer_world.resources.world_id = NEXT_VIEWER_WORLD_ID.fetch_add(1, Ordering::Relaxed);
        viewer_world.resources.graphics.atmosphere = Atmosphere::CloudySky;
        viewer_world.resources.graphics.show_grid = true;
        viewer_world.resources.window.cached_viewport_size =
            Some((MODEL_VIEWER_INITIAL_WIDTH, MODEL_VIEWER_INITIAL_HEIGHT));

        let mut focus = Vec3::new(0.0, 1.0, 0.0);
        let mut orbit_radius = 5.0;

        if let Ok(mut result) = nightshade::ecs::prefab::import_gltf_from_path(&path) {
            let textures = std::mem::take(&mut result.textures);
            for (name, (rgba_data, width, height)) in textures {
                main_world.queue_command(WorldCommand::LoadTexture {
                    name,
                    rgba_data,
                    width,
                    height,
                });
            }

            if let Some((aabb_min, aabb_max)) = thumbnail::compute_mesh_aabb(&result.meshes) {
                let center = (aabb_min + aabb_max) * 0.5;
                let extent = aabb_max - aabb_min;
                let diagonal = nalgebra_glm::length(&extent);
                if diagonal > 1e-6 {
                    focus = center;
                    orbit_radius = diagonal * 1.5;
                }
            }

            for (name, mesh) in result.meshes {
                mesh_cache_insert(&mut viewer_world.resources.mesh_cache, name, mesh);
            }

            if let Some(prefab) = result.prefabs.first() {
                let entity = if !result.skins.is_empty() {
                    nightshade::ecs::prefab::spawn_prefab_with_skins(
                        &mut viewer_world,
                        prefab,
                        &result.animations,
                        &result.skins,
                        Vec3::zeros(),
                    )
                } else {
                    nightshade::ecs::prefab::spawn_prefab(&mut viewer_world, prefab, Vec3::zeros())
                };

                if let Some(player) = viewer_world.get_animation_player_mut(entity)
                    && !player.clips.is_empty()
                {
                    player.play(0);
                    player.looping = true;
                }
            }
        }

        let orbit_camera = spawn_pan_orbit_camera(
            &mut viewer_world,
            focus,
            orbit_radius,
            0.0,
            0.3,
            format!("Orbit Camera - {}", filename),
        );
        viewer_world.resources.active_camera = Some(orbit_camera);

        let sun_entity = spawn_sun(&mut viewer_world);
        if let Some(transform) = viewer_world.get_local_transform_mut(sun_entity) {
            transform.translation = Vec3::new(10.0, 20.0, 10.0);
        }

        nightshade::ecs::transform::systems::run_systems(&mut viewer_world);

        let surface_format = renderer.surface_format();
        let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("model_viewer_texture"),
            size: wgpu::Extent3d {
                width: MODEL_VIEWER_INITIAL_WIDTH,
                height: MODEL_VIEWER_INITIAL_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.model_viewer = Some(ModelViewerInstance {
            world: viewer_world,
            texture,
            texture_view,
            egui_texture_id: None,
            model_path: path,
            width: MODEL_VIEWER_INITIAL_WIDTH,
            height: MODEL_VIEWER_INITIAL_HEIGHT,
            desired_width: MODEL_VIEWER_INITIAL_WIDTH,
            desired_height: MODEL_VIEWER_INITIAL_HEIGHT,
            surface_format,
        });

        let browser = self.tile_tree.as_mut().and_then(take_browser_pane);
        if let Some(browser) = browser {
            let mut tiles = egui_tiles::Tiles::default();
            let browser_id = tiles.insert_pane(Pane::Browser(browser));
            let viewer_id = tiles.insert_pane(Pane::ModelViewer(ModelViewerPane {
                display_name: filename,
            }));
            let root = tiles.insert_horizontal_tile(vec![browser_id, viewer_id]);
            self.tile_tree = Some(egui_tiles::Tree::new("asset_tiles", root, tiles));
        }
    }

    fn close_model_viewer(&mut self) {
        self.model_viewer = None;
        if let Some(browser) = self.tile_tree.as_mut().and_then(take_browser_pane) {
            let mut tiles = egui_tiles::Tiles::default();
            let browser_id = tiles.insert_pane(Pane::Browser(browser));
            self.tile_tree = Some(egui_tiles::Tree::new("asset_tiles", browser_id, tiles));
        }
    }

    fn handle_select_asset(&mut self, index: usize, ctx: &egui::Context, world: &mut World) {
        let (kind, path) = {
            let tree = self.tile_tree.as_ref().unwrap();
            let Some(browser) = get_browser_pane(tree) else {
                return;
            };
            let Some(asset_file) = browser.pack_assets.get(index) else {
                return;
            };
            (asset_file.kind, asset_file.path.clone())
        };

        match kind {
            scanner::AssetFileKind::Image => {
                if let Some((handle, width, height)) = load_full_image(ctx, &path) {
                    self.preview_texture = Some((handle, width, height));
                }
            }
            scanner::AssetFileKind::Hdr => {
                if let Some((handle, width, height)) = load_full_image(ctx, &path) {
                    self.preview_texture = Some((handle, width, height));
                }
                load_hdr_skybox_from_path(world, path);
                world.resources.graphics.atmosphere = Atmosphere::Hdr;
            }
            scanner::AssetFileKind::Model => {}
        }
    }
}

fn take_browser_pane(tree: &mut egui_tiles::Tree<Pane>) -> Option<Box<BrowserPane>> {
    for tile in tree.tiles.tiles_mut() {
        if let egui_tiles::Tile::Pane(Pane::Browser(browser)) = tile {
            let mut taken = Box::new(BrowserPane::new());
            std::mem::swap(&mut taken, browser);
            return Some(taken);
        }
    }
    None
}

fn get_browser_pane(tree: &egui_tiles::Tree<Pane>) -> Option<&BrowserPane> {
    tree.tiles.tiles().find_map(|tile| {
        if let egui_tiles::Tile::Pane(Pane::Browser(browser)) = tile {
            Some(browser.as_ref())
        } else {
            None
        }
    })
}

fn get_browser_pane_mut(tree: &mut egui_tiles::Tree<Pane>) -> Option<&mut BrowserPane> {
    tree.tiles.tiles_mut().find_map(|tile| {
        if let egui_tiles::Tile::Pane(Pane::Browser(browser)) = tile {
            Some(browser.as_mut())
        } else {
            None
        }
    })
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

pub fn load_thumbnail(ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let thumbnail = img.thumbnail(128, 128);
    let rgba = to_displayable_rgba(thumbnail);
    let (width, height) = rgba.dimensions();

    let color_image =
        egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], rgba.as_raw());

    Some(ctx.load_texture(
        path.to_string_lossy().to_string(),
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}

fn load_full_image(ctx: &egui::Context, path: &Path) -> Option<(egui::TextureHandle, u32, u32)> {
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

        let mut tiles = egui_tiles::Tiles::default();
        let browser = tiles.insert_pane(Pane::Browser(Box::new(BrowserPane::new())));
        self.tile_tree = Some(egui_tiles::Tree::new("asset_tiles", browser, tiles));
    }

    fn run_systems(&mut self, world: &mut World) {
        let escape_just_pressed = world
            .resources
            .input
            .keyboard
            .frame_keys
            .iter()
            .any(|(key, pressed)| *key == winit::keyboard::KeyCode::Escape && *pressed);

        if self.model_viewer.is_some() && escape_just_pressed {
            self.close_model_viewer();
            world
                .resources
                .input
                .keyboard
                .keystates
                .remove(&winit::keyboard::KeyCode::Escape);
            return;
        }

        if let Some(ref mut viewer) = self.model_viewer {
            let delta_time = world.resources.window.timing.delta_time;
            viewer.world.resources.window.timing = world.resources.window.timing.clone();

            viewer.world.resources.input.mouse = world.resources.input.mouse;
            viewer.world.resources.input.keyboard.keystates = world
                .resources
                .input
                .keyboard
                .keystates
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect();
            viewer.world.resources.user_interface.hud_wants_pointer = false;

            pan_orbit_camera_system(&mut viewer.world);
            update_animation_players(&mut viewer.world, delta_time);
            apply_animations(&mut viewer.world);
            update_global_transforms_system(&mut viewer.world);
        } else {
            escape_key_exit_system(world);
        }
    }

    fn pre_render(&mut self, renderer: &mut dyn Render, main_world: &mut World) {
        if let Some((path, filename)) = self.pending_model_open.take() {
            self.create_model_viewer(renderer, main_world, path, filename);
        }

        if let Some(ref mut viewer) = self.model_viewer {
            if viewer.desired_width != viewer.width || viewer.desired_height != viewer.height {
                let new_width = viewer.desired_width.max(1);
                let new_height = viewer.desired_height.max(1);

                let new_texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
                    label: Some("model_viewer_texture"),
                    size: wgpu::Extent3d {
                        width: new_width,
                        height: new_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: viewer.surface_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let new_view = new_texture.create_view(&wgpu::TextureViewDescriptor::default());

                viewer.texture = new_texture;
                viewer.texture_view = new_view;
                viewer.width = new_width;
                viewer.height = new_height;
                viewer.world.resources.window.cached_viewport_size = Some((new_width, new_height));
                viewer.egui_texture_id = renderer.register_egui_texture(&viewer.texture_view);
            }

            if viewer.egui_texture_id.is_none() {
                viewer.egui_texture_id = renderer.register_egui_texture(&viewer.texture_view);
            }

            let _ = renderer.render_world_to_texture(
                &mut viewer.world,
                &viewer.texture_view,
                viewer.width,
                viewer.height,
            );
        }

        for (path, result) in self.pending_thumbnail_data.drain(..) {
            if let std::collections::hash_map::Entry::Vacant(entry) =
                self.gpu_thumbnails.entry(path)
                && let Some(gpu_thumb) =
                    thumbnail::generate_gpu_thumbnail(renderer, result, THUMBNAIL_MAX_SIZE)
            {
                entry.insert(gpu_thumb);
            }
        }

        let path_to_prepare = self.tile_tree.as_mut().and_then(|tree| {
            get_browser_pane_mut(tree).and_then(|browser| {
                let index = browser.model_thumbnail_queue.pop_front()?;
                browser
                    .pack_assets
                    .get(index)
                    .map(|asset| asset.path.clone())
            })
        });

        if let Some(path) = path_to_prepare
            && !self.gpu_thumbnails.contains_key(&path)
            && let Ok(mut result) = nightshade::ecs::prefab::import_gltf_from_path(&path)
        {
            let textures = std::mem::take(&mut result.textures);
            for (name, (rgba_data, width, height)) in textures {
                main_world.queue_command(WorldCommand::LoadTexture {
                    name,
                    rgba_data,
                    width,
                    height,
                });
            }
            self.pending_thumbnail_data.push((path, result));
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        let mut tree = self.tile_tree.take().unwrap();

        if let Some(browser) = get_browser_pane_mut(&mut tree) {
            browser.process_thumbnail_queue(ctx);
        }

        let mut source_changed = false;
        egui::TopBottomPanel::top("source_tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Asset Browser");
                ui.separator();
                for (index, source) in self.sources.iter().enumerate() {
                    let label = format!(
                        "{} ({})",
                        source.name,
                        source
                            .categories
                            .iter()
                            .map(|category| category.packs.len())
                            .sum::<usize>()
                    );
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
            if let Some(browser) = get_browser_pane_mut(&mut tree) {
                browser.clear_selection();
            }
            self.preview_texture = None;
            self.gpu_thumbnails.clear();
            self.pending_thumbnail_data.clear();
        }

        let mut pack_action: Option<(usize, usize)> = None;

        let browser_selected_category =
            get_browser_pane(&tree).and_then(|browser| browser.selected_category);
        let browser_selected_pack =
            get_browser_pane(&tree).and_then(|browser| browser.selected_pack);

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

                                let header_text =
                                    format!("{} ({})", category.name, matching_packs.len());
                                egui::CollapsingHeader::new(
                                    egui::RichText::new(header_text).strong(),
                                )
                                .default_open(cat_index == 0)
                                .show(ui, |ui| {
                                    for &pack_index in &matching_packs {
                                        let pack = &category.packs[pack_index];
                                        let is_selected = browser_selected_category
                                            == Some(cat_index)
                                            && browser_selected_pack == Some(pack_index);

                                        if ui.selectable_label(is_selected, &pack.name).clicked() {
                                            pack_action = Some((cat_index, pack_index));
                                        }
                                    }
                                });
                            }
                        });
                }
            });

        if let Some((cat, pack)) = pack_action {
            if let Some(browser) = get_browser_pane_mut(&mut tree) {
                browser.select_pack(cat, pack, &self.sources[self.selected_source]);
            }
            self.preview_texture = None;
            self.gpu_thumbnails.clear();
            self.pending_thumbnail_data.clear();
        }

        let preview_info = get_browser_pane(&tree).and_then(|browser| {
            browser.selected_asset.and_then(|index| {
                browser
                    .pack_assets
                    .get(index)
                    .map(|asset_file| (asset_file.filename.clone(), asset_file.path.clone()))
            })
        });

        if self.preview_texture.is_some() {
            egui::SidePanel::right("preview")
                .default_width(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Preview");
                    ui.separator();

                    if let Some((ref filename, ref path)) = preview_info {
                        ui.label(egui::RichText::new(filename).strong());
                        if let Some((_, width, height)) = &self.preview_texture {
                            ui.label(format!("Dimensions: {} x {}", width, height));
                        }
                        if let Ok(metadata) = std::fs::metadata(path) {
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

        world.resources.user_interface.required_cameras.clear();

        let model_viewer_texture_id = self
            .model_viewer
            .as_ref()
            .and_then(|viewer| viewer.egui_texture_id);

        let pixels_per_point = ctx.pixels_per_point();
        let model_viewer_texture_size = self
            .model_viewer
            .as_ref()
            .map(|viewer| (viewer.width, viewer.height));

        let mut behavior = AssetBrowserBehavior {
            actions: Vec::new(),
            active_viewer_rect: None,
            gpu_thumbnails: &self.gpu_thumbnails,
            model_viewer_texture_id,
            model_viewer_texture_size,
            model_viewer_desired_size: None,
            pixels_per_point,
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            tree.ui(&mut behavior, ui);
        });

        let actions = behavior.actions;
        let active_viewer_rect = behavior.active_viewer_rect;

        if let Some((desired_w, desired_h)) = behavior.model_viewer_desired_size
            && let Some(ref mut viewer) = self.model_viewer
        {
            viewer.desired_width = desired_w;
            viewer.desired_height = desired_h;
        }
        if let Some(rect) = active_viewer_rect {
            if let Some(ref mut viewer) = self.model_viewer {
                viewer.world.resources.window.active_viewport_rect = Some(ViewportRect {
                    x: rect.min.x * pixels_per_point,
                    y: rect.min.y * pixels_per_point,
                    width: rect.width() * pixels_per_point,
                    height: rect.height() * pixels_per_point,
                });
            }
        } else if let Some(ref mut viewer) = self.model_viewer {
            viewer.world.resources.window.active_viewport_rect = None;
        }

        if let Some(ortho) = self.ortho_camera_entity {
            world.resources.active_camera = Some(ortho);
        }

        self.tile_tree = Some(tree);

        for action in actions {
            match action {
                PaneAction::OpenModelViewer { path, filename } => {
                    let already_open = self
                        .model_viewer
                        .as_ref()
                        .is_some_and(|viewer| viewer.model_path == path);
                    if !already_open {
                        self.preview_texture = None;
                        self.pending_model_open = Some((path, filename));
                    }
                }
                PaneAction::CloseModelViewer => {
                    self.close_model_viewer();
                }
                PaneAction::SelectAsset { index } => {
                    self.handle_select_asset(index, ctx, world);
                }
            }
        }
    }
}
