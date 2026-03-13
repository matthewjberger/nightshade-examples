use image::GenericImageView;
use nightshade::ecs::animation::components::AnimationClip;
use nightshade::ecs::animation::systems::{apply_animations, update_animation_players};
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::prefab::mesh_cache_insert;
use nightshade::ecs::prefab::{GltfSkin, Prefab};
use nightshade::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

mod asset_zoo;
mod glb_export;
mod pane;
mod scanner;
mod thumbnail;

use pane::{AssetZooPane, BrowserPane, ModelViewerPane, Pane, PaneAction};

const DEFAULT_KENNEY_ROOT: &str = r"C:\Users\matth\Books\Kenney Game Assets All-in-1 3.2.0";
const DEFAULT_POLYHAVEN_ROOT: &str = r"C:\Users\matth\Documents\Poly Haven";
const THUMBNAIL_MAX_SIZE: u32 = 128;
const MODEL_VIEWER_WIDTH: u32 = 1920;
const MODEL_VIEWER_HEIGHT: u32 = 1080;

static NEXT_VIEWER_WORLD_ID: AtomicU64 = AtomicU64::new(20000);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(AssetViewer::default())?;
    Ok(())
}

pub struct FbxModelData {
    pub name: String,
    pub prefab: Prefab,
    pub skins: Vec<GltfSkin>,
    pub meshes: HashMap<String, nightshade::ecs::mesh::Mesh>,
    pub textures: HashMap<String, (Vec<u8>, u32, u32)>,
    pub node_count: usize,
    pub all_clips: Vec<AnimationClip>,
}

struct ModelViewerInstance {
    world: World,
    _texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    egui_texture_id: Option<egui::TextureId>,
    model_path: PathBuf,
    loaded_animation_names: Vec<String>,
    loaded_animation_durations: Vec<f32>,
    current_animation_index: Option<usize>,
    root_entity: Option<Entity>,
    fbx_model_data: Option<FbxModelData>,
}

struct PendingZooModel {
    path: PathBuf,
    is_fbx: bool,
    pack_path: Option<PathBuf>,
}

struct AssetZooInstance {
    world: World,
    _texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    egui_texture_id: Option<egui::TextureId>,
    pending_models: VecDeque<PendingZooModel>,
    placed_count: usize,
    current_row_x: f32,
    current_row_z: f32,
    current_row_max_depth: f32,
    row_width_limit: f32,
    grid_min: Vec3,
    grid_max: Vec3,
    camera_entity: Entity,
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
    asset_zoo: Option<AssetZooInstance>,
    pending_model_open: Option<(PathBuf, String)>,
    pending_hdr_open: Option<(PathBuf, String)>,
    pending_fbx_character_open: Option<(PathBuf, String, PathBuf)>,
    pending_asset_zoo: bool,
    pending_thumbnail_data: Vec<(PathBuf, nightshade::ecs::prefab::GltfLoadResult)>,
    pending_fbx_thumbnail_data: Vec<(PathBuf, nightshade::ecs::prefab::FbxLoadResult)>,
    selected_tile: Option<egui_tiles::TileId>,
    model_viewer_viewport_size: Option<(u32, u32)>,
    asset_zoo_viewport_size: Option<(u32, u32)>,
}

struct AssetBrowserBehavior<'a> {
    actions: Vec<PaneAction>,
    active_viewer_rect: Option<egui::Rect>,
    gpu_thumbnails: &'a HashMap<PathBuf, thumbnail::GpuThumbnail>,
    model_viewer_texture_id: Option<egui::TextureId>,
    asset_zoo_texture_id: Option<egui::TextureId>,
    animation_names: Vec<String>,
    animation_durations: Vec<f32>,
    current_animation_index: Option<usize>,
    selected_tile: Option<egui_tiles::TileId>,
    pixels_per_point: f32,
    model_viewer_viewport_size: Option<(u32, u32)>,
    asset_zoo_viewport_size: Option<(u32, u32)>,
}

impl<'a> egui_tiles::Behavior<Pane> for AssetBrowserBehavior<'a> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Browser(_) => "Browser".into(),
            Pane::ModelViewer(viewer) => viewer.display_name.clone().into(),
            Pane::AssetZoo(_) => "Asset Zoo".into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match pane {
            Pane::Browser(browser) => {
                browser.draw_grid_ui(ui, &mut self.actions, self.gpu_thumbnails);
            }
            Pane::ModelViewer(viewer) => {
                self.draw_model_viewer_ui(ui, viewer, tile_id);
            }
            Pane::AssetZoo(zoo) => {
                self.draw_asset_zoo_ui(ui, zoo, tile_id);
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
            Some(egui_tiles::Tile::Pane(
                Pane::ModelViewer(_) | Pane::AssetZoo(_)
            ))
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
        if let Some(egui_tiles::Tile::Pane(Pane::AssetZoo(_))) = tiles.get(tile_id) {
            self.actions.push(PaneAction::CloseAssetZoo);
        }
        true
    }
}

impl AssetBrowserBehavior<'_> {
    fn draw_model_viewer_ui(
        &mut self,
        ui: &mut egui::Ui,
        viewer: &mut ModelViewerPane,
        tile_id: egui_tiles::TileId,
    ) {
        let full_rect = ui.available_rect_before_wrap();

        if viewer.is_animated_character && !self.animation_names.is_empty() {
            let animation_panel_width = 280.0;
            let viewer_rect = egui::Rect::from_min_max(
                full_rect.min,
                egui::pos2(full_rect.max.x - animation_panel_width, full_rect.max.y),
            );
            let panel_rect = egui::Rect::from_min_max(
                egui::pos2(full_rect.max.x - animation_panel_width, full_rect.min.y),
                full_rect.max,
            );

            self.draw_3d_viewport(ui, viewer, viewer_rect, tile_id);
            self.draw_animation_panel(ui, viewer, panel_rect);
        } else {
            self.draw_3d_viewport(ui, viewer, full_rect, tile_id);
        }
    }

    fn draw_3d_viewport(
        &mut self,
        ui: &mut egui::Ui,
        viewer: &ModelViewerPane,
        rect: egui::Rect,
        tile_id: egui_tiles::TileId,
    ) {
        if let Some(texture_id) = self.model_viewer_texture_id {
            let (tex_w, tex_h) = self
                .model_viewer_viewport_size
                .map(|(w, h)| (w as f32, h as f32))
                .unwrap_or((MODEL_VIEWER_WIDTH as f32, MODEL_VIEWER_HEIGHT as f32));
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

        let pixel_w = (rect.width() * self.pixels_per_point) as u32;
        let pixel_h = (rect.height() * self.pixels_per_point) as u32;
        if pixel_w > 0 && pixel_h > 0 {
            self.model_viewer_viewport_size = Some((pixel_w, pixel_h));
        }

        let response = ui.allocate_rect(rect, egui::Sense::click());

        if self.selected_tile.is_none() {
            self.selected_tile = Some(tile_id);
        }

        if response.clicked() {
            self.selected_tile = Some(tile_id);
        }

        let is_selected = self.selected_tile == Some(tile_id);

        if is_selected {
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

    fn draw_animation_panel(
        &mut self,
        ui: &mut egui::Ui,
        viewer: &mut ModelViewerPane,
        rect: egui::Rect,
    ) {
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));

        child_ui.painter().rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_premultiplied(20, 20, 25, 230),
        );

        child_ui.scope_builder(egui::UiBuilder::new().max_rect(rect.shrink(8.0)), |ui| {
            ui.vertical(|ui| {
                ui.heading("Animations");
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut viewer.animation_filter);
                });
                ui.add_space(4.0);

                ui.label(format!("{} clips", self.animation_names.len()));
                ui.separator();

                let filter_lower = viewer.animation_filter.to_lowercase();

                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .max_height(rect.height() - 140.0)
                    .show(ui, |ui| {
                        for (clip_index, name) in self.animation_names.iter().enumerate() {
                            if !filter_lower.is_empty()
                                && !name.to_lowercase().contains(&filter_lower)
                            {
                                continue;
                            }

                            let is_current = self.current_animation_index == Some(clip_index);
                            let duration = self
                                .animation_durations
                                .get(clip_index)
                                .copied()
                                .unwrap_or(0.0);

                            let label = format!("{} ({:.1}s)", name, duration);

                            let response = ui.selectable_label(is_current, &label);
                            if response.clicked() {
                                self.actions.push(PaneAction::PlayAnimation { clip_index });
                            }
                        }
                    });

                ui.separator();
                if ui.button("Export GLB").clicked() {
                    self.actions.push(PaneAction::ExportGlb);
                }
            });
        });
    }

    fn draw_asset_zoo_ui(
        &mut self,
        ui: &mut egui::Ui,
        zoo: &mut AssetZooPane,
        tile_id: egui_tiles::TileId,
    ) {
        egui::TopBottomPanel::top(egui::Id::new("asset_zoo_toolbar")).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} - {}/{} models",
                    zoo.source_name, zoo.placed_count, zoo.total_count
                ));

                ui.separator();

                ui.label("Export:");
                ui.add(egui::TextEdit::singleline(&mut zoo.output_path).desired_width(300.0));
                if ui.button("Browse...").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .set_title("Select output folder for GLB files")
                        .pick_folder()
                {
                    zoo.output_path = path.to_string_lossy().to_string();
                }

                ui.add_enabled_ui(!zoo.output_path.is_empty(), |ui| {
                    if ui.button("Export All GLB").clicked() {
                        self.actions.push(PaneAction::GenerateAssetZoo);
                    }
                });

                if !zoo.log_messages.is_empty() {
                    ui.separator();
                    if let Some(last) = zoo.log_messages.last() {
                        ui.label(last);
                    }
                }
            });
        });

        let viewport_rect = ui.available_rect_before_wrap();

        if let Some(texture_id) = self.asset_zoo_texture_id {
            let (tex_w, tex_h) = self
                .asset_zoo_viewport_size
                .map(|(w, h)| (w as f32, h as f32))
                .unwrap_or((MODEL_VIEWER_WIDTH as f32, MODEL_VIEWER_HEIGHT as f32));
            let tile_w = viewport_rect.width();
            let tile_h = viewport_rect.height();

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
            image.paint_at(ui, viewport_rect);
        }

        let pixel_w = (viewport_rect.width() * self.pixels_per_point) as u32;
        let pixel_h = (viewport_rect.height() * self.pixels_per_point) as u32;
        if pixel_w > 0 && pixel_h > 0 {
            self.asset_zoo_viewport_size = Some((pixel_w, pixel_h));
        }

        let response = ui.allocate_rect(viewport_rect, egui::Sense::click());

        if self.selected_tile.is_none() {
            self.selected_tile = Some(tile_id);
        }

        if response.clicked() {
            self.selected_tile = Some(tile_id);
        }

        let is_selected = self.selected_tile == Some(tile_id);

        if is_selected {
            self.active_viewer_rect = Some(viewport_rect);

            ui.painter().rect_stroke(
                viewport_rect,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 165, 0)),
                egui::StrokeKind::Inside,
            );
        }
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
            Some((MODEL_VIEWER_WIDTH, MODEL_VIEWER_HEIGHT));

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

                if let Some(player) = viewer_world.core.get_animation_player_mut(entity)
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
        if let Some(transform) = viewer_world.core.get_local_transform_mut(sun_entity) {
            transform.translation = Vec3::new(10.0, 20.0, 10.0);
        }

        nightshade::ecs::transform::systems::run_systems(&mut viewer_world);

        let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("model_viewer_texture"),
            size: wgpu::Extent3d {
                width: MODEL_VIEWER_WIDTH,
                height: MODEL_VIEWER_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer.surface_format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.model_viewer = Some(ModelViewerInstance {
            world: viewer_world,
            _texture: texture,
            texture_view,
            egui_texture_id: None,
            model_path: path,
            loaded_animation_names: Vec::new(),
            loaded_animation_durations: Vec::new(),
            current_animation_index: None,
            root_entity: None,
            fbx_model_data: None,
        });

        let browser = self.tile_tree.as_mut().and_then(take_browser_pane);
        if let Some(browser) = browser {
            let mut tiles = egui_tiles::Tiles::default();
            let browser_id = tiles.insert_pane(Pane::Browser(browser));
            let viewer_id = tiles.insert_pane(Pane::ModelViewer(ModelViewerPane {
                display_name: filename,
                is_animated_character: false,
                animation_filter: String::new(),
            }));
            let root = tiles.insert_horizontal_tile(vec![browser_id, viewer_id]);
            self.tile_tree = Some(egui_tiles::Tree::new("asset_tiles", root, tiles));
        }
    }

    fn create_fbx_character_viewer(
        &mut self,
        renderer: &mut dyn Render,
        main_world: &mut World,
        model_path: PathBuf,
        filename: String,
        pack_path: PathBuf,
    ) {
        self.model_viewer = None;

        let mut viewer_world = World::default();
        renderer.copy_fonts_to_world(&mut viewer_world);

        viewer_world.resources.world_id = NEXT_VIEWER_WORLD_ID.fetch_add(1, Ordering::Relaxed);
        viewer_world.resources.graphics.atmosphere = Atmosphere::CloudySky;
        viewer_world.resources.graphics.show_grid = true;
        viewer_world.resources.window.cached_viewport_size =
            Some((MODEL_VIEWER_WIDTH, MODEL_VIEWER_HEIGHT));

        let mut focus = Vec3::new(0.0, 1.0, 0.0);
        let mut orbit_radius = 5.0;
        let mut animation_names = Vec::new();
        let mut animation_durations = Vec::new();
        let mut root_entity = None;
        let mut fbx_model_data = None;

        if let Ok(mut result) = nightshade::ecs::prefab::import_fbx_from_path(&model_path) {
            let textures_for_gpu = std::mem::take(&mut result.textures);
            let textures_for_export = textures_for_gpu.clone();
            for (name, (rgba_data, width, height)) in textures_for_gpu {
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

            let meshes_for_export = result.meshes.clone();
            for (name, mesh) in result.meshes {
                mesh_cache_insert(&mut viewer_world.resources.mesh_cache, name, mesh);
            }

            let mut all_clips: Vec<AnimationClip> = Vec::new();

            if let Some(pack_info) = scanner::detect_animated_character_pack(&pack_path) {
                for anim_path in &pack_info.animation_files {
                    if let Ok(clips) =
                        nightshade::ecs::prefab::import_fbx_animations_from_path(anim_path)
                    {
                        let anim_name = anim_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        for mut clip in clips {
                            clip.name = anim_name.clone();
                            all_clips.push(clip);
                        }
                    }
                }
            }

            for clip in &result.animations {
                if !all_clips.iter().any(|existing| existing.name == clip.name) {
                    all_clips.push(clip.clone());
                }
            }

            for clip in &all_clips {
                animation_names.push(clip.name.clone());
                animation_durations.push(clip.duration);
            }

            if let Some(prefab) = result.prefabs.first() {
                let entity = if !result.skins.is_empty() {
                    nightshade::ecs::prefab::spawn_prefab_with_skins(
                        &mut viewer_world,
                        prefab,
                        &all_clips,
                        &result.skins,
                        Vec3::zeros(),
                    )
                } else {
                    nightshade::ecs::prefab::spawn_prefab(&mut viewer_world, prefab, Vec3::zeros())
                };

                if let Some(player) = viewer_world.core.get_animation_player_mut(entity)
                    && !player.clips.is_empty()
                {
                    player.play(0);
                    player.looping = true;
                }

                root_entity = Some(entity);

                fbx_model_data = Some(FbxModelData {
                    name: filename.clone(),
                    prefab: prefab.clone(),
                    skins: result.skins.clone(),
                    meshes: meshes_for_export,
                    textures: textures_for_export,
                    node_count: result.node_count,
                    all_clips,
                });
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
        if let Some(transform) = viewer_world.core.get_local_transform_mut(sun_entity) {
            transform.translation = Vec3::new(10.0, 20.0, 10.0);
        }

        nightshade::ecs::transform::systems::run_systems(&mut viewer_world);

        let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("model_viewer_texture"),
            size: wgpu::Extent3d {
                width: MODEL_VIEWER_WIDTH,
                height: MODEL_VIEWER_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer.surface_format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let current_animation_index = if !animation_names.is_empty() {
            Some(0)
        } else {
            None
        };

        self.model_viewer = Some(ModelViewerInstance {
            world: viewer_world,
            _texture: texture,
            texture_view,
            egui_texture_id: None,
            model_path,
            loaded_animation_names: animation_names,
            loaded_animation_durations: animation_durations,
            current_animation_index,
            root_entity,
            fbx_model_data,
        });

        let browser = self.tile_tree.as_mut().and_then(take_browser_pane);
        if let Some(browser) = browser {
            let mut tiles = egui_tiles::Tiles::default();
            let browser_id = tiles.insert_pane(Pane::Browser(browser));
            let viewer_id = tiles.insert_pane(Pane::ModelViewer(ModelViewerPane {
                display_name: filename,
                is_animated_character: true,
                animation_filter: String::new(),
            }));
            let root = tiles.insert_horizontal_tile(vec![browser_id, viewer_id]);
            self.tile_tree = Some(egui_tiles::Tree::new("asset_tiles", root, tiles));
        }
    }

    fn create_skybox_viewer(&mut self, renderer: &mut dyn Render, path: PathBuf, filename: String) {
        self.model_viewer = None;

        let mut viewer_world = World::default();
        renderer.copy_fonts_to_world(&mut viewer_world);

        viewer_world.resources.world_id = NEXT_VIEWER_WORLD_ID.fetch_add(1, Ordering::Relaxed);
        viewer_world.resources.graphics.atmosphere = Atmosphere::Hdr;
        viewer_world.resources.graphics.show_grid = false;
        viewer_world.resources.window.cached_viewport_size =
            Some((MODEL_VIEWER_WIDTH, MODEL_VIEWER_HEIGHT));

        let orbit_camera = spawn_pan_orbit_camera(
            &mut viewer_world,
            Vec3::new(0.0, 1.0, 0.0),
            5.0,
            0.0,
            0.3,
            format!("Orbit Camera - {}", filename),
        );
        viewer_world.resources.active_camera = Some(orbit_camera);

        nightshade::ecs::transform::systems::run_systems(&mut viewer_world);

        let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("model_viewer_texture"),
            size: wgpu::Extent3d {
                width: MODEL_VIEWER_WIDTH,
                height: MODEL_VIEWER_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer.surface_format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.model_viewer = Some(ModelViewerInstance {
            world: viewer_world,
            _texture: texture,
            texture_view,
            egui_texture_id: None,
            model_path: path,
            loaded_animation_names: Vec::new(),
            loaded_animation_durations: Vec::new(),
            current_animation_index: None,
            root_entity: None,
            fbx_model_data: None,
        });

        let browser = self.tile_tree.as_mut().and_then(take_browser_pane);
        if let Some(browser) = browser {
            let mut tiles = egui_tiles::Tiles::default();
            let browser_id = tiles.insert_pane(Pane::Browser(browser));
            let viewer_id = tiles.insert_pane(Pane::ModelViewer(ModelViewerPane {
                display_name: filename,
                is_animated_character: false,
                animation_filter: String::new(),
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

    fn create_asset_zoo(&mut self, renderer: &mut dyn Render, _main_world: &mut World) {
        self.model_viewer = None;
        self.asset_zoo = None;

        if self.selected_source >= self.sources.len() {
            return;
        }

        let source = &self.sources[self.selected_source];
        let source_name = source.name.clone();

        let mut zoo_world = World::default();
        renderer.copy_fonts_to_world(&mut zoo_world);

        zoo_world.resources.world_id = NEXT_VIEWER_WORLD_ID.fetch_add(1, Ordering::Relaxed);
        zoo_world.resources.graphics.atmosphere = Atmosphere::CloudySky;
        zoo_world.resources.graphics.show_grid = true;
        zoo_world.resources.window.cached_viewport_size =
            Some((MODEL_VIEWER_WIDTH, MODEL_VIEWER_HEIGHT));

        let mut pending_models = VecDeque::new();

        for category in &source.categories {
            for pack in &category.packs {
                let glb_dir = pack.path.join("Models").join("GLB format");
                if glb_dir.exists()
                    && let Ok(entries) = std::fs::read_dir(&glb_dir)
                {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("glb"))
                        {
                            pending_models.push_back(PendingZooModel {
                                path,
                                is_fbx: false,
                                pack_path: None,
                            });
                        }
                    }
                }

                if let Some(pack_info) = scanner::detect_animated_character_pack(&pack.path) {
                    for model_path in pack_info.model_files {
                        pending_models.push_back(PendingZooModel {
                            path: model_path,
                            is_fbx: true,
                            pack_path: Some(pack.path.clone()),
                        });
                    }
                }
            }
        }

        let total_model_count = pending_models.len();

        let camera_entity = spawn_pan_orbit_camera(
            &mut zoo_world,
            Vec3::new(0.0, 1.0, 0.0),
            10.0,
            0.5,
            0.5,
            "Asset Zoo Camera".to_string(),
        );
        zoo_world.resources.active_camera = Some(camera_entity);

        let sun_entity = spawn_sun(&mut zoo_world);
        if let Some(transform) = zoo_world.core.get_local_transform_mut(sun_entity) {
            transform.translation = Vec3::new(10.0, 20.0, 10.0);
        }

        nightshade::ecs::transform::systems::run_systems(&mut zoo_world);

        let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("asset_zoo_texture"),
            size: wgpu::Extent3d {
                width: MODEL_VIEWER_WIDTH,
                height: MODEL_VIEWER_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer.surface_format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.asset_zoo = Some(AssetZooInstance {
            world: zoo_world,
            _texture: texture,
            texture_view,
            egui_texture_id: None,
            pending_models,
            placed_count: 0,
            current_row_x: 0.0,
            current_row_z: 0.0,
            current_row_max_depth: 0.0,
            row_width_limit: 30.0,
            grid_min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            grid_max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
            camera_entity,
        });

        let zoo_pane = AssetZooPane {
            source_name,
            output_path: String::new(),
            placed_count: 0,
            total_count: total_model_count,
            log_messages: Vec::new(),
        };

        let browser = self.tile_tree.as_mut().and_then(take_browser_pane);
        if let Some(browser) = browser {
            let mut tiles = egui_tiles::Tiles::default();
            let browser_id = tiles.insert_pane(Pane::Browser(browser));
            let zoo_id = tiles.insert_pane(Pane::AssetZoo(zoo_pane));
            let root = tiles.insert_horizontal_tile(vec![browser_id, zoo_id]);
            self.tile_tree = Some(egui_tiles::Tree::new("asset_tiles", root, tiles));
        }
    }

    fn handle_select_asset(&mut self, index: usize, ctx: &egui::Context, _world: &mut World) {
        let path = {
            let tree = self.tile_tree.as_ref().unwrap();
            let Some(browser) = get_browser_pane(tree) else {
                return;
            };
            let Some(asset_file) = browser.pack_assets.get(index) else {
                return;
            };
            asset_file.path.clone()
        };

        if let Some((handle, width, height)) = load_full_image(ctx, &path) {
            self.preview_texture = Some((handle, width, height));
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

fn get_asset_zoo_pane_mut(tree: &mut egui_tiles::Tree<Pane>) -> Option<&mut AssetZooPane> {
    tree.tiles.tiles_mut().find_map(|tile| {
        if let egui_tiles::Tile::Pane(Pane::AssetZoo(zoo)) = tile {
            Some(zoo)
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
            update_animation_players(&mut viewer.world);
            apply_animations(&mut viewer.world);
            update_global_transforms_system(&mut viewer.world);
        }

        if let Some(ref mut zoo) = self.asset_zoo {
            zoo.world.resources.window.timing = world.resources.window.timing.clone();

            zoo.world.resources.input.mouse = world.resources.input.mouse;
            zoo.world.resources.input.keyboard.keystates = world
                .resources
                .input
                .keyboard
                .keystates
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect();
            zoo.world.resources.user_interface.hud_wants_pointer = false;

            pan_orbit_camera_system(&mut zoo.world);
            update_animation_players(&mut zoo.world);
            apply_animations(&mut zoo.world);
            update_global_transforms_system(&mut zoo.world);
        }

        if self.model_viewer.is_none() && self.asset_zoo.is_none() {
            escape_key_exit_system(world);
        }
    }

    fn pre_render(&mut self, renderer: &mut dyn Render, main_world: &mut World) {
        if let Some((path, filename)) = self.pending_model_open.take() {
            self.create_model_viewer(renderer, main_world, path, filename);
        }

        if let Some((path, filename)) = self.pending_hdr_open.take() {
            self.create_skybox_viewer(renderer, path, filename);
        }

        if let Some((model_path, filename, pack_path)) = self.pending_fbx_character_open.take() {
            self.create_fbx_character_viewer(renderer, main_world, model_path, filename, pack_path);
        }

        if self.pending_asset_zoo {
            self.pending_asset_zoo = false;
            self.create_asset_zoo(renderer, main_world);
        }

        if let Some(ref mut viewer) = self.model_viewer {
            let (render_width, render_height) = self
                .model_viewer_viewport_size
                .unwrap_or((MODEL_VIEWER_WIDTH, MODEL_VIEWER_HEIGHT));

            let current_size = viewer._texture.size();
            if current_size.width != render_width || current_size.height != render_height {
                let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
                    label: Some("model_viewer_texture"),
                    size: wgpu::Extent3d {
                        width: render_width,
                        height: render_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: renderer.surface_format(),
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                viewer._texture = texture;
                viewer.texture_view = texture_view;
                viewer.egui_texture_id = None;
                viewer.world.resources.window.cached_viewport_size =
                    Some((render_width, render_height));
            }

            if viewer.egui_texture_id.is_none() {
                viewer.egui_texture_id = renderer.register_egui_texture(&viewer.texture_view);
            }

            let _ = renderer.render_world_to_texture(
                &mut viewer.world,
                None,
                &viewer.texture_view,
                render_width,
                render_height,
            );
        }

        if let Some(ref mut zoo) = self.asset_zoo
            && let Some(pending) = zoo.pending_models.pop_front()
        {
            let gap = 0.3_f32;

            let imported = if pending.is_fbx {
                nightshade::ecs::prefab::import_fbx_from_path(&pending.path)
                    .ok()
                    .map(|mut result| {
                        let textures = std::mem::take(&mut result.textures);
                        let aabb = thumbnail::compute_mesh_aabb(&result.meshes);

                        let mut all_clips = result.animations.clone();
                        if let Some(ref pack_path) = pending.pack_path
                            && let Some(pack_info) =
                                scanner::detect_animated_character_pack(pack_path)
                        {
                            for anim_path in &pack_info.animation_files {
                                if let Ok(clips) =
                                    nightshade::ecs::prefab::import_fbx_animations_from_path(
                                        anim_path,
                                    )
                                {
                                    let anim_name = anim_path
                                        .file_stem()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();
                                    for mut clip in clips {
                                        clip.name = anim_name.clone();
                                        all_clips.push(clip);
                                    }
                                }
                            }
                        }

                        (
                            textures,
                            result.meshes,
                            result.prefabs,
                            result.skins,
                            all_clips,
                            aabb,
                        )
                    })
            } else {
                nightshade::ecs::prefab::import_gltf_from_path(&pending.path)
                    .ok()
                    .map(|mut result| {
                        let textures = std::mem::take(&mut result.textures);
                        let aabb = thumbnail::compute_mesh_aabb(&result.meshes);
                        (
                            textures,
                            result.meshes,
                            result.prefabs,
                            result.skins,
                            result.animations,
                            aabb,
                        )
                    })
            };

            if let Some((textures, meshes, prefabs, skins, animations, aabb_opt)) = imported {
                for (name, (rgba_data, width, height)) in textures {
                    main_world.queue_command(WorldCommand::LoadTexture {
                        name,
                        rgba_data,
                        width,
                        height,
                    });
                }

                if let Some((aabb_min, aabb_max)) = aabb_opt {
                    for (name, mesh) in meshes {
                        mesh_cache_insert(&mut zoo.world.resources.mesh_cache, name, mesh);
                    }

                    let extent = aabb_max - aabb_min;
                    let model_width = extent.x.max(0.01);
                    let model_depth = extent.z.max(0.01);

                    if zoo.current_row_x + model_width > zoo.row_width_limit
                        && zoo.current_row_x > 0.0
                    {
                        zoo.current_row_z += zoo.current_row_max_depth + gap;
                        zoo.current_row_x = 0.0;
                        zoo.current_row_max_depth = 0.0;
                    }

                    let position = Vec3::new(
                        zoo.current_row_x + model_width / 2.0,
                        0.0,
                        zoo.current_row_z + model_depth / 2.0,
                    );

                    zoo.current_row_x += model_width + gap;
                    zoo.current_row_max_depth = zoo.current_row_max_depth.max(model_depth);

                    let model_world_min = Vec3::new(
                        position.x - model_width / 2.0,
                        aabb_min.y,
                        position.z - model_depth / 2.0,
                    );
                    let model_world_max = Vec3::new(
                        position.x + model_width / 2.0,
                        aabb_max.y,
                        position.z + model_depth / 2.0,
                    );
                    zoo.grid_min = Vec3::new(
                        zoo.grid_min.x.min(model_world_min.x),
                        zoo.grid_min.y.min(model_world_min.y),
                        zoo.grid_min.z.min(model_world_min.z),
                    );
                    zoo.grid_max = Vec3::new(
                        zoo.grid_max.x.max(model_world_max.x),
                        zoo.grid_max.y.max(model_world_max.y),
                        zoo.grid_max.z.max(model_world_max.z),
                    );

                    if let Some(prefab) = prefabs.into_iter().next() {
                        let entity = if !skins.is_empty() {
                            nightshade::ecs::prefab::spawn_prefab_with_skins(
                                &mut zoo.world,
                                &prefab,
                                &animations,
                                &skins,
                                position,
                            )
                        } else {
                            nightshade::ecs::prefab::spawn_prefab(&mut zoo.world, &prefab, position)
                        };

                        if let Some(player) = zoo.world.core.get_animation_player_mut(entity)
                            && !player.clips.is_empty()
                        {
                            player.play(0);
                            player.looping = true;
                        }
                    }

                    let grid_center = (zoo.grid_min + zoo.grid_max) / 2.0;
                    let grid_extent = zoo.grid_max - zoo.grid_min;
                    let target_radius = nalgebra_glm::length(&grid_extent) * 0.8;

                    if let Some(cam) = zoo.world.core.get_pan_orbit_camera_mut(zoo.camera_entity) {
                        cam.target_focus = grid_center;
                        cam.target_radius = target_radius.max(5.0);
                        cam.focus = cam.target_focus;
                        cam.radius = cam.target_radius;
                    }

                    zoo.placed_count += 1;
                }
            }

            nightshade::ecs::transform::systems::run_systems(&mut zoo.world);
        }

        if let Some(ref zoo) = self.asset_zoo {
            let placed = zoo.placed_count;
            if let Some(ref mut tree) = self.tile_tree
                && let Some(pane) = get_asset_zoo_pane_mut(tree)
            {
                pane.placed_count = placed;
            }
        }

        if let Some(ref mut zoo) = self.asset_zoo {
            let (render_width, render_height) = self
                .asset_zoo_viewport_size
                .unwrap_or((MODEL_VIEWER_WIDTH, MODEL_VIEWER_HEIGHT));

            let current_size = zoo._texture.size();
            if current_size.width != render_width || current_size.height != render_height {
                let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
                    label: Some("asset_zoo_texture"),
                    size: wgpu::Extent3d {
                        width: render_width,
                        height: render_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: renderer.surface_format(),
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                zoo._texture = texture;
                zoo.texture_view = texture_view;
                zoo.egui_texture_id = None;
                zoo.world.resources.window.cached_viewport_size =
                    Some((render_width, render_height));
            }

            if zoo.egui_texture_id.is_none() {
                zoo.egui_texture_id = renderer.register_egui_texture(&zoo.texture_view);
            }

            let _ = renderer.render_world_to_texture(
                &mut zoo.world,
                None,
                &zoo.texture_view,
                render_width,
                render_height,
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

        for (path, result) in self.pending_fbx_thumbnail_data.drain(..) {
            if let std::collections::hash_map::Entry::Vacant(entry) =
                self.gpu_thumbnails.entry(path)
            {
                let gltf_result = nightshade::ecs::prefab::GltfLoadResult {
                    prefabs: result.prefabs,
                    meshes: result.meshes,
                    materials: Vec::new(),
                    textures: HashMap::new(),
                    animations: result.animations,
                    skins: result.skins,
                    node_to_skin: result.node_to_skin,
                    node_to_morph_target_count: HashMap::new(),
                    node_count: result.node_count,
                };
                if let Some(gpu_thumb) =
                    thumbnail::generate_gpu_thumbnail(renderer, gltf_result, THUMBNAIL_MAX_SIZE)
                {
                    entry.insert(gpu_thumb);
                }
            }
        }

        let thumbnail_info = self.tile_tree.as_mut().and_then(|tree| {
            get_browser_pane_mut(tree).and_then(|browser| {
                let index = browser.model_thumbnail_queue.pop_front()?;
                let asset = browser.pack_assets.get(index)?;
                Some((asset.path.clone(), asset.kind))
            })
        });

        if let Some((path, kind)) = thumbnail_info
            && !self.gpu_thumbnails.contains_key(&path)
        {
            match kind {
                scanner::AssetFileKind::Fbx => {
                    if let Ok(mut result) = nightshade::ecs::prefab::import_fbx_from_path(&path) {
                        let textures = std::mem::take(&mut result.textures);
                        for (name, (rgba_data, width, height)) in textures {
                            main_world.queue_command(WorldCommand::LoadTexture {
                                name,
                                rgba_data,
                                width,
                                height,
                            });
                        }
                        self.pending_fbx_thumbnail_data.push((path, result));
                    }
                }
                _ => {
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
                        self.pending_thumbnail_data.push((path, result));
                    }
                }
            }
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        let mut tree = self.tile_tree.take().unwrap();

        if let Some(browser) = get_browser_pane_mut(&mut tree) {
            browser.process_thumbnail_queue(ctx);
        }

        let mut source_changed = false;
        let mut open_asset_zoo = false;
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

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Asset Zoo").clicked() {
                        open_asset_zoo = true;
                    }
                });
            });
        });

        if source_changed {
            if let Some(browser) = get_browser_pane_mut(&mut tree) {
                browser.clear_selection();
            }
            self.preview_texture = None;
            self.gpu_thumbnails.clear();
            self.pending_thumbnail_data.clear();
            self.pending_fbx_thumbnail_data.clear();
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
            self.pending_fbx_thumbnail_data.clear();
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

        let animation_names = self
            .model_viewer
            .as_ref()
            .map(|viewer| viewer.loaded_animation_names.clone())
            .unwrap_or_default();
        let animation_durations = self
            .model_viewer
            .as_ref()
            .map(|viewer| viewer.loaded_animation_durations.clone())
            .unwrap_or_default();
        let current_animation_index = self
            .model_viewer
            .as_ref()
            .and_then(|viewer| viewer.current_animation_index);

        let asset_zoo_texture_id = self.asset_zoo.as_ref().and_then(|zoo| zoo.egui_texture_id);

        let pixels_per_point = ctx.pixels_per_point();

        let mut behavior = AssetBrowserBehavior {
            actions: Vec::new(),
            active_viewer_rect: None,
            gpu_thumbnails: &self.gpu_thumbnails,
            model_viewer_texture_id,
            asset_zoo_texture_id,
            animation_names,
            animation_durations,
            current_animation_index,
            selected_tile: self.selected_tile,
            pixels_per_point,
            model_viewer_viewport_size: self.model_viewer_viewport_size,
            asset_zoo_viewport_size: self.asset_zoo_viewport_size,
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            tree.ui(&mut behavior, ui);
        });

        let actions = behavior.actions;
        let active_viewer_rect = behavior.active_viewer_rect;
        self.selected_tile = behavior.selected_tile;
        self.model_viewer_viewport_size = behavior.model_viewer_viewport_size;
        self.asset_zoo_viewport_size = behavior.asset_zoo_viewport_size;

        let inactive_rect = ViewportRect {
            x: -1.0,
            y: -1.0,
            width: 0.0,
            height: 0.0,
        };

        if let Some(rect) = active_viewer_rect {
            let viewport = ViewportRect {
                x: rect.min.x * pixels_per_point,
                y: rect.min.y * pixels_per_point,
                width: rect.width() * pixels_per_point,
                height: rect.height() * pixels_per_point,
            };
            if let Some(ref mut viewer) = self.model_viewer {
                viewer.world.resources.window.active_viewport_rect = Some(viewport);
            }
            if let Some(ref mut zoo) = self.asset_zoo {
                zoo.world.resources.window.active_viewport_rect = Some(viewport);
            }
        } else {
            if let Some(ref mut viewer) = self.model_viewer {
                viewer.world.resources.window.active_viewport_rect = Some(inactive_rect);
            }
            if let Some(ref mut zoo) = self.asset_zoo {
                zoo.world.resources.window.active_viewport_rect = Some(inactive_rect);
            }
        }

        if let Some(ortho) = self.ortho_camera_entity {
            world.resources.active_camera = Some(ortho);
        }

        self.tile_tree = Some(tree);

        if self
            .asset_zoo
            .as_ref()
            .is_some_and(|zoo| !zoo.pending_models.is_empty())
        {
            ctx.request_repaint();
        }

        if open_asset_zoo {
            self.pending_asset_zoo = true;
        }

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
                PaneAction::OpenFbxCharacter {
                    path,
                    filename,
                    pack_path,
                } => {
                    let already_open = self
                        .model_viewer
                        .as_ref()
                        .is_some_and(|viewer| viewer.model_path == path);
                    if !already_open {
                        self.preview_texture = None;
                        self.pending_fbx_character_open = Some((path, filename, pack_path));
                    }
                }
                PaneAction::CloseModelViewer => {
                    self.close_model_viewer();
                }
                PaneAction::SelectAsset { index } => {
                    self.handle_select_asset(index, ctx, world);
                }
                PaneAction::LoadHdrSkybox { path, filename } => {
                    load_hdr_skybox_from_path(world, path.clone());
                    if let Some(ref mut viewer) = self.model_viewer {
                        viewer.world.resources.graphics.atmosphere = Atmosphere::Hdr;
                    } else {
                        self.pending_hdr_open = Some((path, filename));
                    }
                }
                PaneAction::PlayAnimation { clip_index } => {
                    if let Some(ref mut viewer) = self.model_viewer {
                        if let Some(entity) = viewer.root_entity
                            && let Some(player) = viewer.world.core.get_animation_player_mut(entity)
                        {
                            player.blend_to(clip_index, 0.3);
                            player.looping = true;
                        }
                        viewer.current_animation_index = Some(clip_index);
                    }
                }
                PaneAction::ExportGlb => {
                    if let Some(ref viewer) = self.model_viewer
                        && let Some(ref model_data) = viewer.fbx_model_data
                    {
                        let export_model = glb_export::GlbExportModel {
                            prefab: model_data.prefab.clone(),
                            skins: model_data.skins.clone(),
                            meshes: model_data.meshes.clone(),
                            textures: model_data.textures.clone(),
                        };

                        let default_name =
                            format!("{}.glb", model_data.name.trim_end_matches(".fbx"));

                        if let Some(save_path) = rfd::FileDialog::new()
                            .add_filter("GLB", &["glb"])
                            .set_file_name(&default_name)
                            .save_file()
                        {
                            match glb_export::build_glb(&export_model, &model_data.all_clips, 0.01)
                            {
                                Ok(glb_bytes) => {
                                    if let Err(error) = std::fs::write(&save_path, &glb_bytes) {
                                        eprintln!("Failed to write GLB: {}", error);
                                    }
                                }
                                Err(error) => {
                                    eprintln!("Failed to build GLB: {}", error);
                                }
                            }
                        }
                    }
                }
                PaneAction::CloseAssetZoo => {
                    self.asset_zoo = None;
                    if let Some(browser) = self.tile_tree.as_mut().and_then(take_browser_pane) {
                        let mut tiles = egui_tiles::Tiles::default();
                        let browser_id = tiles.insert_pane(Pane::Browser(browser));
                        self.tile_tree =
                            Some(egui_tiles::Tree::new("asset_tiles", browser_id, tiles));
                    }
                }
                PaneAction::GenerateAssetZoo => {
                    if self.selected_source < self.sources.len() {
                        let source = &self.sources[self.selected_source];
                        let zoo_pane = self.tile_tree.as_mut().and_then(get_asset_zoo_pane_mut);

                        if let Some(zoo) = zoo_pane {
                            let output_path = PathBuf::from(&zoo.output_path);
                            if output_path.as_os_str().is_empty() {
                                zoo.log_messages
                                    .push("Please set an output path".to_string());
                            } else if let Err(error) = std::fs::create_dir_all(&output_path) {
                                zoo.log_messages
                                    .push(format!("Failed to create output directory: {}", error));
                            } else {
                                zoo.log_messages.clear();

                                for category in &source.categories {
                                    for pack in &category.packs {
                                        if let Some(pack_info) =
                                            scanner::detect_animated_character_pack(&pack.path)
                                        {
                                            zoo.log_messages
                                                .push(format!("Processing {}...", pack.name));

                                            match asset_zoo::generate_pack_glb(
                                                &pack_info.model_files,
                                                &pack_info.animation_files,
                                                &output_path,
                                                0.01,
                                            ) {
                                                Ok(result) => {
                                                    zoo.log_messages.push(format!(
                                                        "  {} ({} models, {} KB)",
                                                        pack.name,
                                                        result.model_count,
                                                        result.total_bytes / 1024,
                                                    ));
                                                }
                                                Err(error) => {
                                                    zoo.log_messages
                                                        .push(format!("  Failed: {}", error));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
