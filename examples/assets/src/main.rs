use image::GenericImageView;
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::prefab::mesh_cache_insert;
use nightshade::prelude::*;
use std::path::{Path, PathBuf};

mod pane;
mod scanner;
mod thumbnail;

use pane::{BrowserPane, ModelViewerPane, Pane, PaneAction};

const DEFAULT_KENNEY_ROOT: &str = r"C:\Users\matth\Books\Kenney Game Assets All-in-1 3.2.0";
const DEFAULT_POLYHAVEN_ROOT: &str = r"C:\Users\matth\Documents\Poly Haven";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(AssetViewer::default())?;
    Ok(())
}

#[derive(Default)]
struct AssetViewer {
    sources: Vec<scanner::AssetSource>,
    selected_source: usize,
    pack_filter: String,
    tile_tree: Option<egui_tiles::Tree<Pane>>,
    ortho_camera_entity: Option<Entity>,
    preview_texture: Option<(egui::TextureHandle, u32, u32)>,
}

struct AssetBrowserBehavior<'a> {
    viewport_textures: &'a [egui::TextureId],
    required_cameras: &'a [Entity],
    actions: Vec<PaneAction>,
    active_viewer_rect: Option<egui::Rect>,
    active_viewer_camera: Option<Entity>,
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
                browser.draw_grid_ui(ui, &mut self.actions);
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
        if let Some(egui_tiles::Tile::Pane(Pane::ModelViewer(viewer))) = tiles.get(tile_id) {
            self.actions.push(PaneAction::CloseModelViewer {
                orbit_camera: viewer.orbit_camera_entity,
                sun: viewer.sun_entity,
                model: viewer.model_entity,
            });
        }
        true
    }
}

impl AssetBrowserBehavior<'_> {
    fn draw_model_viewer_ui(&mut self, ui: &mut egui::Ui, viewer: &ModelViewerPane) {
        let rect = ui.available_rect_before_wrap();

        let camera_index = self
            .required_cameras
            .iter()
            .position(|&camera| camera == viewer.orbit_camera_entity);

        if let Some(index) = camera_index
            && let Some(&texture_id) = self.viewport_textures.get(index)
        {
            ui.painter().image(
                texture_id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }

        let response = ui.allocate_rect(rect, egui::Sense::hover());
        if response.hovered() {
            self.active_viewer_rect = Some(rect);
            self.active_viewer_camera = Some(viewer.orbit_camera_entity);
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
    fn has_model_viewer_open(&self) -> bool {
        self.tile_tree.as_ref().is_some_and(|tree| {
            tree.tiles
                .tiles()
                .any(|tile| matches!(tile, egui_tiles::Tile::Pane(Pane::ModelViewer(_))))
        })
    }

    fn open_model_viewer(&mut self, world: &mut World, path: PathBuf, filename: String) {
        let orbit_camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 1.0, 0.0),
            5.0,
            0.0,
            0.3,
            format!("Orbit Camera - {}", filename),
        );

        let sun = spawn_sun(world);

        let mut model_entity = None;
        if let Ok(result) = nightshade::ecs::prefab::import_gltf_from_path(&path) {
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

                model_entity = Some(entity);
            }
        }

        world.resources.graphics.atmosphere = Atmosphere::Hdr;
        world.resources.graphics.show_grid = true;

        let viewer = ModelViewerPane {
            display_name: filename,
            orbit_camera_entity: orbit_camera,
            sun_entity: sun,
            model_entity,
        };

        let tree = self.tile_tree.as_mut().unwrap();
        let new_tile_id = tree.tiles.insert_pane(Pane::ModelViewer(viewer));

        if let Some(root_id) = tree.root {
            if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
                tree.tiles.get_mut(root_id)
            {
                tabs.add_child(new_tile_id);
                tabs.set_active(new_tile_id);
            } else {
                let tab_id = tree.tiles.insert_tab_tile(vec![root_id, new_tile_id]);
                tree.root = Some(tab_id);
            }
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

        let mut tiles = egui_tiles::Tiles::default();
        let browser = tiles.insert_pane(Pane::Browser(Box::new(BrowserPane::new())));
        self.tile_tree = Some(egui_tiles::Tree::new("asset_tiles", browser, tiles));
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);

        if !self.has_model_viewer_open() {
            escape_key_exit_system(world);
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        let mut tree = self.tile_tree.take().unwrap();

        if let Some(browser) = get_browser_pane_mut(&mut tree) {
            browser.process_thumbnail_queue(ctx);
            browser.process_model_thumbnail_queue(ctx);
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
                                            browser_selected_category == Some(cat_index)
                                                && browser_selected_pack == Some(pack_index);

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
            if let Some(browser) = get_browser_pane_mut(&mut tree) {
                browser.select_pack(cat, pack, &self.sources[self.selected_source]);
            }
            self.preview_texture = None;
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

        let required_cameras: Vec<Entity> = tree
            .tiles
            .tiles()
            .filter_map(|tile| {
                if let egui_tiles::Tile::Pane(Pane::ModelViewer(viewer)) = tile {
                    Some(viewer.orbit_camera_entity)
                } else {
                    None
                }
            })
            .collect();

        world.resources.user_interface.required_cameras = required_cameras.clone();
        let viewport_textures = world.resources.user_interface.viewport_textures.clone();

        let mut behavior = AssetBrowserBehavior {
            viewport_textures: &viewport_textures,
            required_cameras: &required_cameras,
            actions: Vec::new(),
            active_viewer_rect: None,
            active_viewer_camera: None,
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            tree.ui(&mut behavior, ui);
        });

        let actions = behavior.actions;
        let active_viewer_rect = behavior.active_viewer_rect;
        let active_viewer_camera = behavior.active_viewer_camera;

        let pixels_per_point = ctx.pixels_per_point();
        if let Some(rect) = active_viewer_rect {
            world.resources.window.active_viewport_rect = Some(ViewportRect {
                x: rect.min.x * pixels_per_point,
                y: rect.min.y * pixels_per_point,
                width: rect.width() * pixels_per_point,
                height: rect.height() * pixels_per_point,
            });
            world.resources.active_camera = active_viewer_camera;
        } else {
            world.resources.window.active_viewport_rect = None;
            if let Some(ortho) = self.ortho_camera_entity {
                world.resources.active_camera = Some(ortho);
            }
        }

        self.tile_tree = Some(tree);

        for action in actions {
            match action {
                PaneAction::OpenModelViewer { path, filename } => {
                    self.open_model_viewer(world, path, filename);
                }
                PaneAction::CloseModelViewer {
                    orbit_camera,
                    sun,
                    model,
                } => {
                    despawn_recursive_immediate(world, orbit_camera);
                    despawn_recursive_immediate(world, sun);
                    if let Some(model_entity) = model {
                        despawn_recursive_immediate(world, model_entity);
                    }
                    if !self.has_model_viewer_open() {
                        world.resources.graphics.atmosphere = Atmosphere::None;
                        world.resources.graphics.show_grid = false;
                    }
                }
                PaneAction::SelectAsset { index } => {
                    self.handle_select_asset(index, ctx, world);
                }
            }
        }
    }
}
