use nightshade::ecs::animation::components::AnimationClip;
use nightshade::ecs::animation::systems::{apply_animations, update_animation_players};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::mesh::Mesh;
use nightshade::ecs::prefab::resources::{mesh_cache_insert, mesh_cache_lookup_id};
use nightshade::ecs::prefab::{GltfSkin, Prefab};
use nightshade::ecs::text::systems::sync_text_meshes_system;
use nightshade::prelude::*;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashSet;

#[cfg(not(target_arch = "wasm32"))]
mod webview_manager;

const DANCE_MODEL: &[u8] = include_bytes!("../../../assets/models/dance.glb");
const HELMET_MODEL: &[u8] = include_bytes!("../../../assets/gltf/DamagedHelmet.glb");
const HDR_SKYBOX: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

#[cfg(not(target_arch = "wasm32"))]
fn open_directory(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(path).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(MultiWorldDemo::default())
}

struct LoadedModel {
    prefab: Prefab,
    meshes: HashMap<String, Mesh>,
    textures: Vec<(String, Vec<u8>, u32, u32)>,
    animations: Vec<AnimationClip>,
    skins: Vec<GltfSkin>,
}

#[derive(Clone)]
enum PaneKind {
    World(u32),
    #[cfg(not(target_arch = "wasm32"))]
    Web(WebWidget),
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
struct WebWidget {
    url: String,
    id: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for WebWidget {
    fn default() -> Self {
        Self {
            url: "https://matthewberger.dev/nightshade".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ViewportSize {
    Small,  // 512x512 - prefabs, icons, small objects
    Medium, // 1024x768 - medium scenes, UI layouts
    Large,  // 1920x1080 - full levels, complex scenes
}

impl ViewportSize {
    fn dimensions(self) -> (u32, u32) {
        match self {
            ViewportSize::Small => (512, 512),
            ViewportSize::Medium => (1024, 768),
            ViewportSize::Large => (1920, 1080),
        }
    }

    fn name(self) -> &'static str {
        match self {
            ViewportSize::Small => "Small (512x512)",
            ViewportSize::Medium => "Medium (1024x768)",
            ViewportSize::Large => "Large (1920x1080)",
        }
    }
}

struct WorldInstance {
    world: World,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    viewport_size: ViewportSize,
    dirty: bool,
    last_render_frame: u64,
    egui_texture_id: Option<egui::TextureId>,
    name: String,
    selected_entities: Vec<Entity>,
    viewport_rect: Option<egui::Rect>,
    popped_out_window_index: Option<usize>,
}

impl WorldInstance {
    fn is_popped_out(&self) -> bool {
        self.popped_out_window_index.is_some()
    }
}

struct AnimatedEntity {
    entity: Entity,
    original_translation: Vec3,
    original_scale: Vec3,
    animation_type: u32,
    speed: f32,
    phase: f32,
}

struct TileBehavior<'a> {
    worlds: &'a mut HashMap<u32, WorldInstance>,
    selected_world: &'a mut Option<u32>,
    selected_tile: &'a mut Option<egui_tiles::TileId>,
    world_to_tile: &'a mut HashMap<u32, egui_tiles::TileId>,
    active_window: &'a mut Option<Option<usize>>,
    window_id: Option<usize>,
    pixels_per_point: f32,
    texture_id_overrides: Option<&'a HashMap<u32, egui::TextureId>>,
    #[cfg(not(target_arch = "wasm32"))]
    web_widget_rects: &'a mut Vec<(String, String, egui::Rect)>,
}

impl<'a> egui_tiles::Behavior<PaneKind> for TileBehavior<'a> {
    fn tab_title_for_pane(&mut self, pane: &PaneKind) -> egui::WidgetText {
        match pane {
            PaneKind::World(id) => {
                let is_selected = *self.selected_world == Some(*id);
                if let Some(instance) = self.worlds.get(id) {
                    if is_selected {
                        egui::RichText::new(&instance.name).strong().into()
                    } else {
                        instance.name.clone().into()
                    }
                } else {
                    format!("World {}", id).into()
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            PaneKind::Web(_) => "Web".into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut PaneKind,
    ) -> egui_tiles::UiResponse {
        match pane {
            PaneKind::World(world_id) => {
                self.world_to_tile.insert(*world_id, tile_id);
                self.render_world_pane(ui, *world_id, tile_id);
            }
            #[cfg(not(target_arch = "wasm32"))]
            PaneKind::Web(widget) => {
                let rect = ui.available_rect_before_wrap();
                ui.painter()
                    .rect_filled(rect, 0.0, ui.style().visuals.panel_fill);
                self.web_widget_rects
                    .push((widget.id.clone(), widget.url.clone(), rect));
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
}

impl<'a> TileBehavior<'a> {
    fn render_world_pane(&mut self, ui: &mut egui::Ui, world_id: u32, tile_id: egui_tiles::TileId) {
        let rect = ui.available_rect_before_wrap();

        if let Some(instance) = self.worlds.get_mut(&world_id) {
            instance.viewport_rect = Some(rect);

            if instance.is_popped_out() {
                ui.allocate_rect(rect, egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 30));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} (Popped Out)", instance.name),
                    egui::FontId::default(),
                    egui::Color32::GRAY,
                );
                return;
            }

            let texture_id = self
                .texture_id_overrides
                .and_then(|overrides| overrides.get(&world_id).copied())
                .or(instance.egui_texture_id);

            let clicked = if let Some(texture_id) = texture_id {
                let (tex_w, tex_h) = instance.viewport_size.dimensions();
                let tex_w = tex_w as f32;
                let tex_h = tex_h as f32;
                let tile_w = rect.width();
                let tile_h = rect.height();

                let tex_aspect = tex_w / tex_h;
                let tile_aspect = tile_w / tile_h;

                let uv_rect = if tile_aspect > tex_aspect {
                    let uv_height = tex_aspect / tile_aspect;
                    let uv_y = (1.0 - uv_height) / 2.0;
                    egui::Rect::from_min_max(
                        egui::pos2(0.0, uv_y),
                        egui::pos2(1.0, uv_y + uv_height),
                    )
                } else {
                    let uv_width = tile_aspect / tex_aspect;
                    let uv_x = (1.0 - uv_width) / 2.0;
                    egui::Rect::from_min_max(
                        egui::pos2(uv_x, 0.0),
                        egui::pos2(uv_x + uv_width, 1.0),
                    )
                };

                let response = ui.allocate_rect(rect, egui::Sense::click());
                let image = egui::Image::new(egui::load::SizedTexture::new(
                    texture_id,
                    egui::vec2(tile_w, tile_h),
                ))
                .uv(uv_rect);
                image.paint_at(ui, rect);
                response.clicked()
            } else {
                let response = ui.allocate_rect(rect, egui::Sense::click());
                response.clicked()
            };

            if self.selected_world.is_none() {
                *self.selected_world = Some(world_id);
                *self.selected_tile = Some(tile_id);
                *self.active_window = Some(self.window_id);
            }

            if clicked {
                *self.selected_world = Some(world_id);
                *self.selected_tile = Some(tile_id);
                *self.active_window = Some(self.window_id);
            }

            let is_active = *self.active_window == Some(self.window_id);
            let is_selected = is_active && *self.selected_tile == Some(tile_id);
            if is_selected {
                ui.painter().rect_stroke(
                    rect,
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 165, 0)),
                    egui::StrokeKind::Inside,
                );

                let scaled_rect = ViewportRect {
                    x: rect.min.x * self.pixels_per_point,
                    y: rect.min.y * self.pixels_per_point,
                    width: rect.width() * self.pixels_per_point,
                    height: rect.height() * self.pixels_per_point,
                };
                instance.world.resources.window.active_viewport_rect = Some(scaled_rect);
            }

            if !instance.selected_entities.is_empty() {
                let selection_text = format!("Selected: {}", instance.selected_entities.len());
                let text_pos = egui::pos2(rect.min.x + 8.0, rect.max.y - 24.0);
                ui.painter().text(
                    text_pos,
                    egui::Align2::LEFT_CENTER,
                    selection_text,
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("World not found");
            });
        }
    }
}

#[derive(Default)]
struct PerWindowState {
    tile_tree: Option<egui_tiles::Tree<PaneKind>>,
    selected_tile: Option<egui_tiles::TileId>,
    world_to_tile: HashMap<u32, egui_tiles::TileId>,
    world_texture_ids: HashMap<u32, egui::TextureId>,
    world_ids: Vec<u32>,
}

struct MultiWorldDemo {
    worlds: HashMap<u32, WorldInstance>,
    tile_tree: Option<egui_tiles::Tree<PaneKind>>,
    selected_world: Option<u32>,
    selected_tile: Option<egui_tiles::TileId>,
    world_to_tile: HashMap<u32, egui_tiles::TileId>,
    active_window: Option<Option<usize>>,
    next_world_id: u32,
    primary_world_ids: Vec<u32>,
    window_states: HashMap<usize, PerWindowState>,
    initialized: bool,
    frame_count: u64,
    pending_spawns: Vec<Option<usize>>,
    bumper_cooldown: f32,
    x_button_cooldown: f32,
    total_time: f32,
    dancer_model: Option<LoadedModel>,
    helmet_model: Option<LoadedModel>,
    models_loaded: bool,
    dancer_spawn_counter: u32,
    helmet_spawn_counter: u32,
    pendingtexture_loads: Vec<(String, Vec<u8>, u32, u32)>,
    animated_entities: HashMap<u64, Vec<AnimatedEntity>>,
    light_spawn_counter: u32,
    text_spawn_counter: u32,
    cube_spawn_counter: u32,
    sphere_spawn_counter: u32,
    pending_viewport_resizes: Vec<(u32, ViewportSize)>,
    global_quality: ViewportSize,
    #[cfg(not(target_arch = "wasm32"))]
    pending_screenshot: Option<u32>,
    #[cfg(not(target_arch = "wasm32"))]
    pending_hq_screenshot: Option<u32>,
    tv_screen_spawn_counter: u32,
    fallback_texture_created: bool,
    #[cfg(not(target_arch = "wasm32"))]
    webview_manager: webview_manager::WebviewManager,
    #[cfg(not(target_arch = "wasm32"))]
    web_widget_rects: Vec<(String, String, egui::Rect)>,
    popped_out_worlds: HashMap<usize, u32>,
    pending_popout_world_ids: Vec<u32>,
}

impl Default for MultiWorldDemo {
    fn default() -> Self {
        Self {
            worlds: HashMap::new(),
            tile_tree: None,
            selected_world: None,
            selected_tile: None,
            world_to_tile: HashMap::new(),
            active_window: None,
            next_world_id: 1,
            primary_world_ids: Vec::new(),
            initialized: false,
            frame_count: 0,
            pending_spawns: Vec::new(),
            bumper_cooldown: 0.0,
            x_button_cooldown: 0.0,
            total_time: 0.0,
            dancer_model: None,
            helmet_model: None,
            models_loaded: false,
            dancer_spawn_counter: 0,
            helmet_spawn_counter: 0,
            pendingtexture_loads: Vec::new(),
            animated_entities: HashMap::new(),
            light_spawn_counter: 0,
            text_spawn_counter: 0,
            cube_spawn_counter: 0,
            sphere_spawn_counter: 0,
            pending_viewport_resizes: Vec::new(),
            global_quality: ViewportSize::Medium,
            #[cfg(not(target_arch = "wasm32"))]
            pending_screenshot: None,
            #[cfg(not(target_arch = "wasm32"))]
            pending_hq_screenshot: None,
            tv_screen_spawn_counter: 0,
            fallback_texture_created: false,
            #[cfg(not(target_arch = "wasm32"))]
            webview_manager: webview_manager::WebviewManager::default(),
            #[cfg(not(target_arch = "wasm32"))]
            web_widget_rects: Vec::new(),
            popped_out_worlds: HashMap::new(),
            pending_popout_world_ids: Vec::new(),
            window_states: HashMap::new(),
        }
    }
}

impl State for MultiWorldDemo {
    fn initialize(&mut self, world: &mut World) {
        world.resources.world_id = 1;
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.show_grid = false;

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            10.0,
            0.0,
            0.5,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);

        load_hdr_skybox(world, HDR_SKYBOX.to_vec());

        if let Ok(result) = nightshade::ecs::prefab::import_gltf_from_bytes(DANCE_MODEL) {
            tracing::info!(
                "Loaded dancer model: {} meshes, {} animations",
                result.meshes.len(),
                result.animations.len()
            );
            for (name, (rgba_data, width, height)) in &result.textures {
                world.queue_command(WorldCommand::LoadTexture {
                    name: name.clone(),
                    rgba_data: rgba_data.clone(),
                    width: *width,
                    height: *height,
                });
            }
            if let Some(prefab) = result.prefabs.into_iter().next() {
                self.dancer_model = Some(LoadedModel {
                    prefab,
                    meshes: result.meshes,
                    textures: result
                        .textures
                        .into_iter()
                        .map(|(n, (d, w, h))| (n, d, w, h))
                        .collect(),
                    animations: result.animations,
                    skins: result.skins,
                });
            }
        }

        if let Ok(result) = nightshade::ecs::prefab::import_gltf_from_bytes(HELMET_MODEL) {
            tracing::info!("Loaded helmet model: {} meshes", result.meshes.len());
            for (name, (rgba_data, width, height)) in &result.textures {
                world.queue_command(WorldCommand::LoadTexture {
                    name: name.clone(),
                    rgba_data: rgba_data.clone(),
                    width: *width,
                    height: *height,
                });
            }
            if let Some(prefab) = result.prefabs.into_iter().next() {
                self.helmet_model = Some(LoadedModel {
                    prefab,
                    meshes: result.meshes,
                    textures: result
                        .textures
                        .into_iter()
                        .map(|(n, (d, w, h))| (n, d, w, h))
                        .collect(),
                    animations: result.animations,
                    skins: result.skins,
                });
            }
        }

        self.models_loaded = true;
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        let delta_time = world.resources.window.timing.delta_time;
        self.total_time += delta_time;

        for (world_id, instance) in self.worlds.iter_mut() {
            instance.world.resources.window.timing = world.resources.window.timing.clone();

            if let Some(animated) = self.animated_entities.get(&(*world_id as u64)) {
                animate_objects_fixed(&mut instance.world, self.total_time, animated);
            }

            update_animation_players(&mut instance.world);
            apply_animations(&mut instance.world);
            sync_text_meshes_system(&mut instance.world);
            update_global_transforms_system(&mut instance.world);
        }

        self.bumper_cooldown = (self.bumper_cooldown - delta_time).max(0.0);
        self.x_button_cooldown = (self.x_button_cooldown - delta_time).max(0.0);

        if self.x_button_cooldown <= 0.0
            && world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::ControlLeft)
            && world.resources.input.keyboard.is_key_pressed(KeyCode::KeyN)
        {
            world
                .resources
                .secondary_windows
                .pending_spawns
                .push(WindowSpawnRequest {
                    title: "Multi-World Demo".to_string(),
                    width: 1280,
                    height: 720,
                    egui_enabled: true,
                });
            self.x_button_cooldown = 0.5;
        }

        if let Some(ref gilrs) = world.resources.input.gamepad.gilrs
            && let Some(gamepad_id) = world.resources.input.gamepad.gamepad
        {
            let gamepad = gilrs.gamepad(gamepad_id);

            if self.bumper_cooldown <= 0.0 && !self.worlds.is_empty() {
                let mut world_ids: Vec<u32> = self.worlds.keys().copied().collect();
                world_ids.sort();

                let current_index = self
                    .selected_world
                    .and_then(|id| world_ids.iter().position(|&wid| wid == id))
                    .unwrap_or(0);

                if gamepad.is_pressed(gilrs::Button::RightTrigger) {
                    let next_index = (current_index + 1) % world_ids.len();
                    let new_world_id = world_ids[next_index];
                    self.selected_world = Some(new_world_id);
                    self.selected_tile = self.world_to_tile.get(&new_world_id).copied();
                    self.active_window = Some(None);
                    self.bumper_cooldown = 0.2;
                } else if gamepad.is_pressed(gilrs::Button::LeftTrigger) {
                    let prev_index = if current_index == 0 {
                        world_ids.len() - 1
                    } else {
                        current_index - 1
                    };
                    let new_world_id = world_ids[prev_index];
                    self.selected_world = Some(new_world_id);
                    self.selected_tile = self.world_to_tile.get(&new_world_id).copied();
                    self.active_window = Some(None);
                    self.bumper_cooldown = 0.2;
                }
            }

            if self.x_button_cooldown <= 0.0 && gamepad.is_pressed(gilrs::Button::West) {
                self.rebuild_tile_tree();
                self.x_button_cooldown = 0.3;
            }

            if self.x_button_cooldown <= 0.0
                && gamepad.is_pressed(gilrs::Button::South)
                && let Some(selected_id) = self.selected_world
            {
                self.spawn_cube_in_world(selected_id);
                self.x_button_cooldown = 0.15;
            }

            if self.x_button_cooldown <= 0.0
                && gamepad.is_pressed(gilrs::Button::DPadUp)
                && let Some(selected_id) = self.selected_world
            {
                self.spawn_dancer_in_world(selected_id);
                self.x_button_cooldown = 0.3;
            }

            if self.x_button_cooldown <= 0.0
                && gamepad.is_pressed(gilrs::Button::DPadDown)
                && let Some(selected_id) = self.selected_world
            {
                self.spawn_helmet_in_world(selected_id);
                self.x_button_cooldown = 0.3;
            }

            if self.x_button_cooldown <= 0.0
                && gamepad.is_pressed(gilrs::Button::DPadLeft)
                && let Some(selected_id) = self.selected_world
            {
                self.spawn_point_light_in_world(selected_id);
                self.x_button_cooldown = 0.3;
            }

            if self.x_button_cooldown <= 0.0
                && gamepad.is_pressed(gilrs::Button::DPadRight)
                && let Some(selected_id) = self.selected_world
            {
                self.spawn_text_in_world(selected_id);
                self.x_button_cooldown = 0.3;
            }
        }

        for secondary_window in &world.resources.secondary_windows.states {
            if let Some(&world_id) = self.popped_out_worlds.get(&secondary_window.index)
                && let Some(instance) = self.worlds.get_mut(&world_id)
            {
                instance.world.resources.input.mouse.position =
                    secondary_window.input.mouse_position;
                instance.world.resources.input.mouse.state = secondary_window.input.mouse_state;
                instance.world.resources.input.mouse.position_delta =
                    secondary_window.input.mouse_position_delta;
                instance.world.resources.input.mouse.raw_mouse_delta =
                    secondary_window.input.raw_mouse_delta;
                instance.world.resources.input.mouse.wheel_delta =
                    secondary_window.input.mouse_wheel_delta;
                instance.world.resources.input.keyboard.keystates = secondary_window
                    .input
                    .keyboard_keystates
                    .iter()
                    .map(|(k, v)| (*k, *v))
                    .collect();
                instance.world.resources.user_interface.hud_wants_pointer = false;
                pan_orbit_camera_system(&mut instance.world);
            }
        }

        if let Some(selected_id) = self.selected_world {
            match self.active_window {
                Some(None) => {
                    let gamepad_input = read_gamepad_input(world);
                    if let Some(instance) = self.worlds.get_mut(&selected_id) {
                        instance.world.resources.input.mouse = world.resources.input.mouse;
                        instance.world.resources.input.keyboard.keystates = world
                            .resources
                            .input
                            .keyboard
                            .keystates
                            .iter()
                            .map(|(k, v)| (*k, *v))
                            .collect();
                        instance.world.resources.user_interface.hud_wants_pointer =
                            world.resources.user_interface.hud_wants_pointer;
                        pan_orbit_camera_system(&mut instance.world);
                        if let Some(input) = gamepad_input {
                            apply_gamepad_to_pan_orbit(&mut instance.world, &input, delta_time);
                        }
                        update_picking(&mut instance.world, &mut instance.selected_entities);
                    }
                }
                Some(Some(window_index)) => {
                    if let Some(secondary_window) = world
                        .resources
                        .secondary_windows
                        .states
                        .iter()
                        .find(|w| w.index == window_index)
                        && !self.popped_out_worlds.contains_key(&window_index)
                        && let Some(instance) = self.worlds.get_mut(&selected_id)
                    {
                        instance.world.resources.input.mouse.position =
                            secondary_window.input.mouse_position;
                        instance.world.resources.input.mouse.state =
                            secondary_window.input.mouse_state;
                        instance.world.resources.input.mouse.position_delta =
                            secondary_window.input.mouse_position_delta;
                        instance.world.resources.input.mouse.raw_mouse_delta =
                            secondary_window.input.raw_mouse_delta;
                        instance.world.resources.input.mouse.wheel_delta =
                            secondary_window.input.mouse_wheel_delta;
                        instance.world.resources.input.keyboard.keystates = secondary_window
                            .input
                            .keyboard_keystates
                            .iter()
                            .map(|(k, v)| (*k, *v))
                            .collect();
                        instance.world.resources.user_interface.hud_wants_pointer = false;
                        pan_orbit_camera_system(&mut instance.world);
                        update_picking(&mut instance.world, &mut instance.selected_entities);
                    }
                }
                None => {}
            }
        }

        for secondary_window in &mut world.resources.secondary_windows.states {
            secondary_window.input.mouse_state.remove(
                MouseState::LEFT_JUST_PRESSED
                    | MouseState::LEFT_JUST_RELEASED
                    | MouseState::MIDDLE_JUST_PRESSED
                    | MouseState::MIDDLE_JUST_RELEASED
                    | MouseState::RIGHT_JUST_PRESSED
                    | MouseState::RIGHT_JUST_RELEASED
                    | MouseState::MOVED
                    | MouseState::SCROLLED,
            );
            secondary_window.input.raw_mouse_delta = nalgebra_glm::Vec2::zeros();
            secondary_window.input.mouse_wheel_delta = nalgebra_glm::Vec2::zeros();
            secondary_window.input.mouse_position_delta = nalgebra_glm::Vec2::zeros();
        }
    }

    fn pre_render(&mut self, renderer: &mut dyn Render, main_world: &mut World) {
        self.frame_count += 1;

        if !self.initialized {
            self.initialized = true;
        }

        for (world_id, new_size) in self.pending_viewport_resizes.drain(..) {
            if let Some(instance) = self.worlds.get_mut(&world_id) {
                let (width, height) = new_size.dimensions();
                let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
                    label: Some(&format!("{}texture", instance.name)),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: renderer.surface_format(),
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

                tracing::info!(
                    "Resized {} from {} to {}",
                    instance.name,
                    instance.viewport_size.name(),
                    new_size.name()
                );

                instance.texture = texture;
                instance.texture_view = texture_view;
                instance.viewport_size = new_size;
                instance.egui_texture_id = None;
                instance.dirty = true;
            }
        }

        let pending_spawns: Vec<Option<usize>> = self.pending_spawns.drain(..).collect();
        for target_window in pending_spawns {
            let instance = self.create_world_instance(renderer);
            let world_id = self.next_world_id - 1;
            self.worlds.insert(world_id, instance);

            match target_window {
                None => {
                    self.primary_world_ids.push(world_id);
                    self.selected_world = Some(world_id);
                    self.active_window = Some(None);
                    self.rebuild_tile_tree();
                }
                Some(window_index) => {
                    if let Some(window_state) = self.window_states.get_mut(&window_index) {
                        window_state.world_ids.push(world_id);
                        self.selected_world = Some(world_id);
                        self.active_window = Some(Some(window_index));
                        Self::rebuild_window_tile_tree(window_state);
                    }
                }
            }
        }

        if !self.pending_popout_world_ids.is_empty() {
            for secondary_window in &main_world.resources.secondary_windows.states {
                if !self.popped_out_worlds.contains_key(&secondary_window.index)
                    && !self.pending_popout_world_ids.is_empty()
                {
                    let world_id = self.pending_popout_world_ids.remove(0);
                    self.popped_out_worlds
                        .insert(secondary_window.index, world_id);
                    if let Some(instance) = self.worlds.get_mut(&world_id) {
                        instance.popped_out_window_index = Some(secondary_window.index);
                    }
                }
            }
        }

        let closed_popout_indices: Vec<usize> = self
            .popped_out_worlds
            .keys()
            .filter(|index| {
                !main_world
                    .resources
                    .secondary_windows
                    .states
                    .iter()
                    .any(|w| w.index == **index)
            })
            .copied()
            .collect();
        for index in closed_popout_indices {
            if let Some(world_id) = self.popped_out_worlds.remove(&index)
                && let Some(instance) = self.worlds.get_mut(&world_id)
            {
                instance.popped_out_window_index = None;
            }
        }

        let active_indices: Vec<usize> = main_world
            .resources
            .secondary_windows
            .states
            .iter()
            .map(|w| w.index)
            .collect();
        self.window_states
            .retain(|index, _| active_indices.contains(index));

        for (name, rgba_data, width, height) in self.pendingtexture_loads.drain(..) {
            main_world.queue_command(WorldCommand::LoadTexture {
                name,
                rgba_data,
                width,
                height,
            });
        }

        for instance in self.worlds.values_mut() {
            if instance.egui_texture_id.is_none() {
                instance.egui_texture_id = renderer.register_egui_texture(&instance.texture_view);
            }
        }

        let window_indices: Vec<usize> = self.window_states.keys().copied().collect();
        for window_index in window_indices {
            let world_ids: Vec<u32> = self.worlds.keys().copied().collect();
            for world_id in world_ids {
                let already_registered = self
                    .window_states
                    .get(&window_index)
                    .map(|state| state.world_texture_ids.contains_key(&world_id))
                    .unwrap_or(true);
                if !already_registered
                    && let Some(instance) = self.worlds.get(&world_id)
                    && let Some(texture_id) = renderer
                        .register_secondary_egui_texture(window_index, &instance.texture_view)
                    && let Some(window_state) = self.window_states.get_mut(&window_index)
                {
                    window_state.world_texture_ids.insert(world_id, texture_id);
                }
            }
        }

        for instance in self.worlds.values_mut() {
            instance
                .world
                .resources
                .command_queue
                .retain(|cmd| !matches!(cmd, WorldCommand::LoadTexture { .. }));

            instance
                .world
                .resources
                .graphics
                .bounding_volume_selected_entity = instance.selected_entities.first().copied();
        }

        let mut worlds_to_render: Vec<(u32, ViewportSize)> = self
            .worlds
            .iter()
            .filter_map(|(&world_id, instance)| {
                let should_render = true;

                if should_render {
                    Some((world_id, instance.viewport_size))
                } else {
                    None
                }
            })
            .collect();

        worlds_to_render.sort_by_key(|(_, size)| match size {
            ViewportSize::Small => 0,
            ViewportSize::Medium => 1,
            ViewportSize::Large => 2,
        });

        if !self.fallback_texture_created {
            self.fallback_texture_created = true;
            let size = 64u32;
            let mut pixels = Vec::with_capacity((size * size * 4) as usize);
            for y in 0..size {
                for x in 0..size {
                    let checker = ((x / 8) + (y / 8)) % 2 == 0;
                    if checker {
                        pixels.extend_from_slice(&[128, 0, 128, 255]);
                    } else {
                        pixels.extend_from_slice(&[64, 0, 64, 255]);
                    }
                }
            }
            let fallback_texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("Fallback Checkerboard"),
                size: wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            renderer.queue().write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &fallback_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size * 4),
                    rows_per_image: Some(size),
                },
                wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
            );
            let fallback_view =
                fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());
            renderer.register_render_texture("fallback_checkerboard", fallback_view);
        }

        for (world_id, _viewport_size) in worlds_to_render {
            if let Some(instance) = self.worlds.get_mut(&world_id) {
                if instance.is_popped_out() {
                    continue;
                }
                let (width, height) = instance.viewport_size.dimensions();
                let _ = renderer.render_world_to_texture(
                    &mut instance.world,
                    None,
                    &instance.texture_view,
                    width,
                    height,
                );
                instance.last_render_frame = self.frame_count;
                instance.dirty = false;
            }
        }

        for (&window_index, &world_id) in &self.popped_out_worlds {
            if let Some(instance) = self.worlds.get_mut(&world_id) {
                let _ =
                    renderer.render_world_to_secondary_surface(window_index, &mut instance.world);
                instance.last_render_frame = self.frame_count;
                instance.dirty = false;
            }
        }

        for (&world_id, instance) in &self.worlds {
            let view = instance
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            renderer.register_render_texture(&format!("world_{}_render", world_id), view);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(world_id) = self.pending_screenshot.take()
            && let Some(instance) = self.worlds.get(&world_id)
        {
            let (width, height) = instance.viewport_size.dimensions();
            let path = Self::screenshot_path(&instance.name);
            renderer.save_texture_to_file(&instance.texture, width, height, &path);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(world_id) = self.pending_hq_screenshot.take()
            && let Some(instance) = self.worlds.get_mut(&world_id)
        {
            let (width, height) = renderer.surface_size();
            let hq_texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("HQ Screenshot Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: renderer.surface_format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let hq_view = hq_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let _ = renderer.render_world_to_texture(
                &mut instance.world,
                None,
                &hq_view,
                width,
                height,
            );

            let path = Self::screenshot_path(&format!("{}_hq", instance.name));
            renderer.save_texture_to_file(&hq_texture, width, height, &path);
        }
    }

    fn ui(&mut self, _world: &mut World, ctx: &egui::Context) {
        let mut pending_popout_world_id: Option<u32> = None;
        let mut pending_popin_world_id: Option<u32> = None;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Multi-World Demo");
                ui.separator();

                if ui.button("+ World").clicked() {
                    self.pending_spawns.push(None);
                }

                if ui.button("New Window").clicked() {
                    _world
                        .resources
                        .secondary_windows
                        .pending_spawns
                        .push(WindowSpawnRequest {
                            title: "Multi-World Demo".to_string(),
                            width: 1280,
                            height: 720,
                            egui_enabled: true,
                        });
                }

                if ui.button("Arrange").clicked() {
                    self.rebuild_tile_tree();
                }

                ui.separator();

                if self.dancer_model.is_some()
                    && ui.button("+ Dancer").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_dancer_in_world(selected_id);
                }

                if self.helmet_model.is_some()
                    && ui.button("+ Helmet").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_helmet_in_world(selected_id);
                }

                if ui.button("+ Cube").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_cube_in_world(selected_id);
                }

                if ui.button("+ Sphere").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_sphere_in_world(selected_id);
                }

                if ui.button("+ Point Light").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_point_light_in_world(selected_id);
                }

                if ui.button("+ Spot Light").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_spot_light_in_world(selected_id);
                }

                if ui.button("+ Text").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_text_in_world(selected_id);
                }

                if ui.button("+ TV Screen").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    let other_world_id = self.worlds.keys().find(|&&id| id != selected_id).copied();
                    self.spawn_tv_screen_in_world(selected_id, other_world_id);
                }

                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("+ Web Tile").clicked() {
                    self.add_web_tile();
                }

                ui.separator();
                ui.label(format!("Worlds: {}", self.worlds.len()));

                ui.separator();
                egui::ComboBox::from_id_salt("global_quality")
                    .selected_text(self.global_quality.name())
                    .show_ui(ui, |ui| {
                        for size in [
                            ViewportSize::Small,
                            ViewportSize::Medium,
                            ViewportSize::Large,
                        ] {
                            if ui
                                .selectable_label(self.global_quality == size, size.name())
                                .clicked()
                            {
                                self.global_quality = size;
                            }
                        }
                    });
                if ui.button("Apply All").clicked() {
                    let world_ids: Vec<u32> = self.worlds.keys().copied().collect();
                    for world_id in world_ids {
                        self.change_world_viewport_size(world_id, self.global_quality);
                    }
                }

                if self.active_window == Some(None)
                    && let Some(selected_id) = self.selected_world
                    && let Some(instance) = self.worlds.get(&selected_id)
                {
                    ui.separator();
                    ui.label(format!("Selected: {}", instance.name));
                    ui.separator();

                    let current_size = instance.viewport_size;
                    let is_popped_out = instance.is_popped_out();

                    egui::ComboBox::from_label("")
                        .selected_text(current_size.name())
                        .show_ui(ui, |ui| {
                            for size in [
                                ViewportSize::Small,
                                ViewportSize::Medium,
                                ViewportSize::Large,
                            ] {
                                if ui
                                    .selectable_label(current_size == size, size.name())
                                    .clicked()
                                {
                                    self.change_world_viewport_size(selected_id, size);
                                }
                            }
                        });

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();
                        if ui.button("Screenshot").clicked() {
                            self.pending_screenshot = Some(selected_id);
                        }
                        if ui.button("HQ Screenshot").clicked() {
                            self.pending_hq_screenshot = Some(selected_id);
                        }
                        if ui.button("Open Folder").clicked() {
                            let screenshots_dir = Self::screenshots_dir();
                            if !screenshots_dir.exists() {
                                let _ = std::fs::create_dir_all(&screenshots_dir);
                            }
                            let _ = open_directory(&screenshots_dir);
                        }
                    }

                    ui.separator();
                    if is_popped_out {
                        if ui.button("Pop In").clicked() {
                            pending_popin_world_id = Some(selected_id);
                        }
                    } else if ui.button("Pop Out").clicked() {
                        pending_popout_world_id = Some(selected_id);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("ESC to exit");
                });
            });
        });

        let pixels_per_point = ctx.pixels_per_point();

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.primary_world_ids.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("No worlds yet");
                        ui.add_space(10.0);
                        if ui.button("Create First World").clicked() {
                            self.pending_spawns.push(None);
                        }
                    });
                });
            } else if let Some(tree) = &mut self.tile_tree {
                #[cfg(not(target_arch = "wasm32"))]
                self.web_widget_rects.clear();

                let mut behavior = TileBehavior {
                    worlds: &mut self.worlds,
                    selected_world: &mut self.selected_world,
                    selected_tile: &mut self.selected_tile,
                    world_to_tile: &mut self.world_to_tile,
                    active_window: &mut self.active_window,
                    window_id: None,
                    pixels_per_point,
                    texture_id_overrides: None,
                    #[cfg(not(target_arch = "wasm32"))]
                    web_widget_rects: &mut self.web_widget_rects,
                };
                tree.ui(&mut behavior, ui);
            }
        });

        if let Some(world_id) = pending_popout_world_id
            && let Some(instance) = self.worlds.get(&world_id)
        {
            _world
                .resources
                .secondary_windows
                .pending_spawns
                .push(WindowSpawnRequest {
                    title: instance.name.clone(),
                    width: 1280,
                    height: 720,
                    egui_enabled: false,
                });
            self.pending_popout_world_ids.push(world_id);
        }

        if let Some(world_id) = pending_popin_world_id
            && let Some(instance) = self.worlds.get_mut(&world_id)
            && let Some(window_index) = instance.popped_out_window_index
        {
            self.popped_out_worlds.remove(&window_index);
            instance.popped_out_window_index = None;
            if let Some(window_state) = _world
                .resources
                .secondary_windows
                .states
                .iter_mut()
                .find(|w| w.index == window_index)
            {
                window_state.close_requested = true;
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(window_handle) = &_world.resources.window.handle {
                self.webview_manager.set_window(window_handle.clone());
            }

            let active_ids: HashSet<String> = self
                .web_widget_rects
                .iter()
                .map(|(id, _, _)| id.clone())
                .collect();

            self.webview_manager.retain_only(&active_ids);

            for (id, url, rect) in &self.web_widget_rects {
                if !self.webview_manager.has_webview(id) {
                    self.webview_manager.create_webview(id.clone(), url, *rect);
                }
                self.webview_manager.update_position(id, *rect);
            }

            self.webview_manager.ensure_all_visible();
        }
    }

    fn secondary_ui(&mut self, _world: &mut World, window_index: usize, ctx: &egui::Context) {
        self.window_states.entry(window_index).or_default();

        egui::TopBottomPanel::top("secondary_top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Multi-World Demo");
                ui.separator();

                if ui.button("+ World").clicked() {
                    self.pending_spawns.push(Some(window_index));
                }

                if ui.button("Arrange").clicked()
                    && let Some(window_state) = self.window_states.get_mut(&window_index)
                {
                    Self::rebuild_window_tile_tree(window_state);
                }

                ui.separator();

                if self.dancer_model.is_some()
                    && ui.button("+ Dancer").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_dancer_in_world(selected_id);
                }

                if self.helmet_model.is_some()
                    && ui.button("+ Helmet").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_helmet_in_world(selected_id);
                }

                if ui.button("+ Cube").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_cube_in_world(selected_id);
                }

                if ui.button("+ Sphere").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_sphere_in_world(selected_id);
                }

                if ui.button("+ Point Light").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_point_light_in_world(selected_id);
                }

                if ui.button("+ Spot Light").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_spot_light_in_world(selected_id);
                }

                if ui.button("+ Text").clicked()
                    && let Some(selected_id) = self.selected_world
                {
                    self.spawn_text_in_world(selected_id);
                }

                ui.separator();
                ui.label(format!("Worlds: {}", self.worlds.len()));

                if self.active_window == Some(Some(window_index))
                    && let Some(selected_id) = self.selected_world
                    && let Some(instance) = self.worlds.get(&selected_id)
                {
                    ui.separator();
                    ui.label(format!("Selected: {}", instance.name));
                }
            });
        });

        let pixels_per_point = ctx.pixels_per_point();
        let mut window_state = self.window_states.remove(&window_index).unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            if window_state.world_ids.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("No worlds yet");
                        ui.add_space(10.0);
                        if ui.button("Create First World").clicked() {
                            self.pending_spawns.push(Some(window_index));
                        }
                    });
                });
            } else if let Some(tree) = &mut window_state.tile_tree {
                let mut behavior = TileBehavior {
                    worlds: &mut self.worlds,
                    selected_world: &mut self.selected_world,
                    selected_tile: &mut window_state.selected_tile,
                    world_to_tile: &mut window_state.world_to_tile,
                    active_window: &mut self.active_window,
                    window_id: Some(window_index),
                    pixels_per_point,
                    texture_id_overrides: Some(&window_state.world_texture_ids),
                    #[cfg(not(target_arch = "wasm32"))]
                    web_widget_rects: &mut Vec::new(),
                };
                tree.ui(&mut behavior, ui);
            }
        });

        self.window_states.insert(window_index, window_state);
    }
}

fn update_picking(world: &mut World, selected_entities: &mut Vec<Entity>) {
    let mouse = &world.resources.input.mouse;

    if mouse.state.contains(MouseState::LEFT_JUST_PRESSED)
        && !world.resources.user_interface.hud_wants_pointer
    {
        let viewport_rect = world.resources.window.active_viewport_rect.as_ref();
        let mouse_pos = mouse.position;

        let pick_pos = if let Some(rect) = viewport_rect {
            if !rect.contains(mouse_pos) {
                return;
            }
            let local = rect.to_local(mouse_pos);
            if let Some((width, height)) = world.resources.window.cached_viewport_size {
                let scale_x = width as f32 / rect.width;
                let scale_y = height as f32 / rect.height;
                ((local.x * scale_x) as u32, (local.y * scale_y) as u32)
            } else {
                (local.x as u32, local.y as u32)
            }
        } else {
            (mouse_pos.x as u32, mouse_pos.y as u32)
        };

        world
            .resources
            .gpu_picking
            .request_pick(pick_pos.0, pick_pos.1);
    }

    if let Some(result) = world.resources.gpu_picking.take_result() {
        let shift_pressed = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::ShiftLeft)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::ShiftRight);

        let ctrl_pressed = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::ControlLeft)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::ControlRight);

        if let Some(entity_id) = result.entity_id {
            let camera_entity = world.resources.active_camera;

            let found_entity = world
                .core
                .query_entities(nightshade::ecs::RENDER_MESH | nightshade::ecs::GLOBAL_TRANSFORM)
                .find(|entity| {
                    if entity.id != entity_id {
                        return false;
                    }
                    if Some(*entity) == camera_entity {
                        return false;
                    }
                    true
                });

            if let Some(entity) = found_entity {
                if ctrl_pressed {
                    if let Some(pos) = selected_entities.iter().position(|e| *e == entity) {
                        selected_entities.remove(pos);
                    } else {
                        selected_entities.push(entity);
                    }
                } else if shift_pressed {
                    if !selected_entities.contains(&entity) {
                        selected_entities.push(entity);
                    }
                } else {
                    selected_entities.clear();
                    selected_entities.push(entity);
                }

                if let Some(name) = world.core.get_name(entity) {
                    tracing::info!("Selected: {}", name.0);
                }
            } else if !shift_pressed && !ctrl_pressed {
                selected_entities.clear();
            }
        } else if !shift_pressed && !ctrl_pressed {
            selected_entities.clear();
        }
    }
}

impl MultiWorldDemo {
    fn change_world_viewport_size(&mut self, world_id: u32, new_size: ViewportSize) {
        if let Some(instance) = self.worlds.get(&world_id) {
            if instance.viewport_size == new_size {
                return;
            }
            self.pending_viewport_resizes.push((world_id, new_size));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn screenshots_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("screenshots")
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn screenshot_path(name: &str) -> std::path::PathBuf {
        let screenshots_dir = Self::screenshots_dir();
        if !screenshots_dir.exists() {
            let _ = std::fs::create_dir_all(&screenshots_dir);
        }
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        screenshots_dir.join(format!("{}_{}.png", name, timestamp))
    }

    fn rebuild_window_tile_tree(window_state: &mut PerWindowState) {
        let mut tiles = egui_tiles::Tiles::default();
        let mut world_panes = Vec::new();

        let mut sorted_ids = window_state.world_ids.clone();
        sorted_ids.sort();

        for world_id in sorted_ids {
            world_panes.push(tiles.insert_pane(PaneKind::World(world_id)));
        }

        if world_panes.is_empty() {
            window_state.tile_tree = None;
            return;
        }

        let count = world_panes.len();
        let root = if count == 1 {
            world_panes[0]
        } else {
            let cols = (count as f32).sqrt().ceil() as usize;
            let mut rows_tiles = Vec::new();
            for row_panes in world_panes.chunks(cols) {
                if row_panes.len() == 1 {
                    rows_tiles.push(row_panes[0]);
                } else {
                    rows_tiles.push(tiles.insert_horizontal_tile(row_panes.to_vec()));
                }
            }
            if rows_tiles.len() == 1 {
                rows_tiles[0]
            } else {
                tiles.insert_vertical_tile(rows_tiles)
            }
        };

        window_state.tile_tree = Some(egui_tiles::Tree::new("secondary_world_tiles", root, tiles));
        window_state.world_to_tile.clear();
    }

    fn rebuild_tile_tree(&mut self) {
        let mut tiles = egui_tiles::Tiles::default();
        let mut world_panes = Vec::new();

        let mut world_ids = self.primary_world_ids.clone();
        world_ids.sort();

        for world_id in world_ids {
            world_panes.push(tiles.insert_pane(PaneKind::World(world_id)));
        }

        if world_panes.is_empty() {
            self.tile_tree = None;
            return;
        }

        let count = world_panes.len();
        let root = if count == 1 {
            world_panes[0]
        } else {
            let cols = (count as f32).sqrt().ceil() as usize;
            let mut rows_tiles = Vec::new();

            for row_panes in world_panes.chunks(cols) {
                if row_panes.len() == 1 {
                    rows_tiles.push(row_panes[0]);
                } else {
                    rows_tiles.push(tiles.insert_horizontal_tile(row_panes.to_vec()));
                }
            }

            if rows_tiles.len() == 1 {
                rows_tiles[0]
            } else {
                tiles.insert_vertical_tile(rows_tiles)
            }
        };

        self.tile_tree = Some(egui_tiles::Tree::new("multi_world_tiles", root, tiles));
        self.world_to_tile.clear();
    }

    fn create_world_instance(&mut self, renderer: &dyn Render) -> WorldInstance {
        self.create_world_instance_with_size(renderer, ViewportSize::Medium)
    }

    fn create_world_instance_with_size(
        &mut self,
        renderer: &dyn Render,
        viewport_size: ViewportSize,
    ) -> WorldInstance {
        let mut play_world = World::default();
        renderer.copy_fonts_to_world(&mut play_world);

        let world_id = self.next_world_id;
        play_world.resources.world_id = world_id as u64 + 1000;

        let atmospheres = [
            Atmosphere::Sunset,
            Atmosphere::Space,
            Atmosphere::Nebula,
            Atmosphere::CloudySky,
        ];
        let atmosphere = atmospheres[world_id as usize % atmospheres.len()];
        play_world.resources.graphics.atmosphere = atmosphere;
        play_world.resources.graphics.show_grid = false;
        play_world.resources.graphics.show_bounding_volumes = true;

        capture_procedural_atmosphere_ibl(&mut play_world, atmosphere, 0.0);

        let name = format!("World {}", world_id);
        self.next_world_id += 1;

        let focus = Vec3::new(0.0, 2.0, 0.0);
        let radius = 15.0 + (world_id as f32 * 0.5).sin() * 3.0;
        let yaw = (world_id as f32 * 0.7) % std::f32::consts::TAU;
        let pitch = 0.4;

        let camera_entity = spawn_pan_orbit_camera(
            &mut play_world,
            focus,
            radius,
            yaw,
            pitch,
            format!("{} Camera", name),
        );
        play_world.resources.active_camera = Some(camera_entity);

        spawn_floor(&mut play_world);

        let sun_entity = spawn_sun(&mut play_world);
        if let Some(transform) = play_world.core.get_local_transform_mut(sun_entity) {
            transform.translation = Vec3::new(10.0, 20.0, 10.0);
        }

        let animated = generate_scene_content(&mut play_world, world_id);
        self.animated_entities.insert(world_id as u64, animated);

        let device = renderer.device();
        let (width, height) = viewport_size.dimensions();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{}texture", name)),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: renderer.surface_format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        tracing::info!("Created {} with {} viewport", name, viewport_size.name());

        WorldInstance {
            world: play_world,
            texture,
            texture_view,
            viewport_size,
            dirty: true,
            last_render_frame: 0,
            egui_texture_id: None,
            name,
            selected_entities: Vec::new(),
            viewport_rect: None,
            popped_out_window_index: None,
        }
    }

    fn ensure_model_meshes_in_world(world: &mut World, model: &LoadedModel) {
        for (name, mesh) in &model.meshes {
            if mesh_cache_lookup_id(&world.resources.mesh_cache, name).is_none() {
                mesh_cache_insert(&mut world.resources.mesh_cache, name.clone(), mesh.clone());
            }
        }
    }

    fn spawn_dancer_in_world(&mut self, world_id: u32) {
        let Some(model) = &self.dancer_model else {
            return;
        };

        for (name, rgba_data, width, height) in &model.textures {
            self.pendingtexture_loads
                .push((name.clone(), rgba_data.clone(), *width, *height));
        }

        let Some(model) = &self.dancer_model else {
            return;
        };

        let Some(instance) = self.worlds.get_mut(&world_id) else {
            return;
        };

        Self::ensure_model_meshes_in_world(&mut instance.world, model);

        let counter = self.dancer_spawn_counter;
        self.dancer_spawn_counter += 1;

        let angle = (counter as f32 * 0.7) % std::f32::consts::TAU;
        let radius = 3.0 + (counter % 5) as f32;
        let position = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);

        let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
            &mut instance.world,
            &model.prefab,
            &model.animations,
            &model.skins,
            position,
        );

        instance
            .world
            .core
            .set_name(entity, Name(format!("Dancer_{}", counter)));

        if let Some(player) = instance.world.core.get_animation_player_mut(entity) {
            player.play(0);
            player.looping = true;
            player.speed = 0.8 + (counter % 5) as f32 * 0.1;
        }

        instance.dirty = true;
        tracing::info!("Spawned dancer {} in {}", counter, instance.name);
    }

    fn spawn_helmet_in_world(&mut self, world_id: u32) {
        let Some(model) = &self.helmet_model else {
            return;
        };

        for (name, rgba_data, width, height) in &model.textures {
            self.pendingtexture_loads
                .push((name.clone(), rgba_data.clone(), *width, *height));
        }

        let Some(model) = &self.helmet_model else {
            return;
        };

        let Some(instance) = self.worlds.get_mut(&world_id) else {
            return;
        };

        Self::ensure_model_meshes_in_world(&mut instance.world, model);

        let counter = self.helmet_spawn_counter;
        self.helmet_spawn_counter += 1;

        let angle = (counter as f32 * 1.1 + 0.5) % std::f32::consts::TAU;
        let radius = 2.0 + (counter % 4) as f32 * 1.5;
        let height = 1.0 + (counter % 3) as f32 * 0.5;
        let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

        let entity =
            nightshade::ecs::prefab::spawn_prefab(&mut instance.world, &model.prefab, position);

        instance
            .world
            .core
            .set_name(entity, Name(format!("Helmet_{}", counter)));

        instance.dirty = true;
        tracing::info!("Spawned helmet {} in {}", counter, instance.name);
    }

    fn spawn_point_light_in_world(&mut self, world_id: u32) {
        let Some(instance) = self.worlds.get_mut(&world_id) else {
            return;
        };

        let counter = self.light_spawn_counter;
        self.light_spawn_counter += 1;

        let colors = [
            [1.0, 0.3, 0.3],
            [0.3, 1.0, 0.3],
            [0.3, 0.3, 1.0],
            [1.0, 1.0, 0.3],
            [1.0, 0.3, 1.0],
            [0.3, 1.0, 1.0],
            [1.0, 0.6, 0.2],
        ];
        let color = colors[counter as usize % colors.len()];

        let angle = (counter as f32 * 1.3) % std::f32::consts::TAU;
        let radius = 4.0 + (counter % 3) as f32 * 2.0;
        let height = 3.0 + (counter % 4) as f32;
        let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

        let entity = spawn_point_light(
            &mut instance.world,
            position,
            color,
            500.0 + (counter % 5) as f32 * 200.0,
            20.0,
        );

        instance
            .world
            .core
            .set_name(entity, Name(format!("PointLight_{}", counter)));

        let sphere_entity = spawn_mesh(
            &mut instance.world,
            "Sphere",
            position,
            Vec3::new(0.2, 0.2, 0.2),
        );
        instance
            .world
            .core
            .set_name(sphere_entity, Name(format!("LightBulb_{}", counter)));

        let mat_name = format!("EmissiveLight_{}", counter);
        let mat = Material {
            base_color: [color[0], color[1], color[2], 1.0],
            emissive_factor: color,
            unlit: true,
            ..Default::default()
        };
        nightshade::ecs::material::resources::material_registry_insert(
            &mut instance.world.resources.material_registry,
            mat_name.clone(),
            mat,
        );
        instance
            .world
            .core
            .set_material_ref(sphere_entity, MaterialRef::new(mat_name));

        instance.dirty = true;
        tracing::info!("Spawned point light {} in {}", counter, instance.name);
    }

    fn spawn_spot_light_in_world(&mut self, world_id: u32) {
        let Some(instance) = self.worlds.get_mut(&world_id) else {
            return;
        };

        let counter = self.light_spawn_counter;
        self.light_spawn_counter += 1;

        let colors = [
            [1.0, 0.9, 0.8],
            [0.8, 0.9, 1.0],
            [1.0, 0.5, 0.2],
            [0.5, 1.0, 0.5],
        ];
        let color = colors[counter as usize % colors.len()];

        let angle = (counter as f32 * 0.9 + 0.3) % std::f32::consts::TAU;
        let radius = 5.0;
        let height = 6.0;
        let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

        let target = Vec3::new(0.0, 0.0, 0.0);
        let direction = nalgebra_glm::normalize(&(target - position));

        let entity = spawn_spotlight(
            &mut instance.world,
            SpotlightParams {
                position,
                direction,
                color,
                intensity: 800.0,
                range: 25.0,
                inner_cone_angle: 0.4,
                outer_cone_angle: 0.5,
            },
        );

        instance
            .world
            .core
            .set_name(entity, Name(format!("SpotLight_{}", counter)));

        let cone_entity = spawn_mesh(
            &mut instance.world,
            "Cone",
            position,
            Vec3::new(0.15, 0.3, 0.15),
        );
        instance
            .world
            .core
            .set_name(cone_entity, Name(format!("SpotLightCone_{}", counter)));

        let mat_name = format!("EmissiveSpot_{}", counter);
        let mat = Material {
            base_color: [color[0], color[1], color[2], 1.0],
            emissive_factor: color,
            unlit: true,
            ..Default::default()
        };
        nightshade::ecs::material::resources::material_registry_insert(
            &mut instance.world.resources.material_registry,
            mat_name.clone(),
            mat,
        );
        instance
            .world
            .core
            .set_material_ref(cone_entity, MaterialRef::new(mat_name));

        instance.dirty = true;
        tracing::info!("Spawned spot light {} in {}", counter, instance.name);
    }

    fn spawn_text_in_world(&mut self, world_id: u32) {
        let Some(instance) = self.worlds.get_mut(&world_id) else {
            return;
        };

        let counter = self.text_spawn_counter;
        self.text_spawn_counter += 1;

        let texts = [
            "Hello!",
            "World",
            "Nightshade",
            "Multi-World",
            "Test",
            "Lights!",
            "3D Text",
            "Demo",
        ];
        let text = texts[counter as usize % texts.len()];

        let angle = (counter as f32 * 0.8 + 1.0) % std::f32::consts::TAU;
        let radius = 3.5 + (counter % 3) as f32;
        let height = 1.5 + (counter % 4) as f32 * 0.5;
        let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

        let entity = spawn_3d_text_at(&mut instance.world, text, position, 0.4);
        instance
            .world
            .core
            .set_name(entity, Name(format!("Text_{}", counter)));

        let colors = ["Red", "Green", "Blue", "Yellow", "Cyan", "Magenta", "White"];
        let color = colors[counter as usize % colors.len()];
        instance
            .world
            .core
            .set_material_ref(entity, MaterialRef::new(color.to_string()));

        instance.dirty = true;
        tracing::info!("Spawned text '{}' ({}) in {}", text, counter, instance.name);
    }

    fn spawn_cube_in_world(&mut self, world_id: u32) {
        let Some(instance) = self.worlds.get_mut(&world_id) else {
            return;
        };

        let counter = self.cube_spawn_counter;
        self.cube_spawn_counter += 1;

        let angle = (counter as f32 * 0.6) % std::f32::consts::TAU;
        let radius = 2.0 + (counter % 4) as f32 * 1.5;
        let height = 0.5 + (counter % 3) as f32 * 0.5;
        let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

        let scale = 0.5 + (counter % 5) as f32 * 0.2;
        let entity = spawn_mesh(
            &mut instance.world,
            "Cube",
            position,
            Vec3::new(scale, scale, scale),
        );
        instance
            .world
            .core
            .set_name(entity, Name(format!("Cube_{}", counter)));

        let colors = [
            "Red", "Green", "Blue", "Yellow", "Cyan", "Magenta", "Orange", "White",
        ];
        let color = colors[counter as usize % colors.len()];
        instance
            .world
            .core
            .set_material_ref(entity, MaterialRef::new(color.to_string()));

        let animated = AnimatedEntity {
            entity,
            original_translation: position,
            original_scale: Vec3::new(scale, scale, scale),
            animation_type: counter % 6,
            speed: 0.3 + (counter % 10) as f32 * 0.1,
            phase: (counter as f32) * 0.7,
        };

        self.animated_entities
            .entry(world_id as u64)
            .or_default()
            .push(animated);

        instance.dirty = true;
        tracing::info!("Spawned cube {} in {}", counter, instance.name);
    }

    fn spawn_sphere_in_world(&mut self, world_id: u32) {
        let Some(instance) = self.worlds.get_mut(&world_id) else {
            return;
        };

        let counter = self.sphere_spawn_counter;
        self.sphere_spawn_counter += 1;

        let angle = (counter as f32 * 0.5 + 0.5) % std::f32::consts::TAU;
        let radius = 2.5 + (counter % 3) as f32 * 1.2;
        let height = 1.0 + (counter % 4) as f32 * 0.4;
        let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

        let scale = 0.4 + (counter % 4) as f32 * 0.15;
        let entity = spawn_mesh(
            &mut instance.world,
            "Sphere",
            position,
            Vec3::new(scale, scale, scale),
        );
        instance
            .world
            .core
            .set_name(entity, Name(format!("Sphere_{}", counter)));

        let colors = [
            "Red", "Green", "Blue", "Yellow", "Cyan", "Magenta", "Orange", "White",
        ];
        let color = colors[counter as usize % colors.len()];
        instance
            .world
            .core
            .set_material_ref(entity, MaterialRef::new(color.to_string()));

        let animated = AnimatedEntity {
            entity,
            original_translation: position,
            original_scale: Vec3::new(scale, scale, scale),
            animation_type: (counter + 3) % 6,
            speed: 0.4 + (counter % 8) as f32 * 0.08,
            phase: (counter as f32) * 1.1,
        };

        self.animated_entities
            .entry(world_id as u64)
            .or_default()
            .push(animated);

        instance.dirty = true;
        tracing::info!("Spawned sphere {} in {}", counter, instance.name);
    }

    fn spawn_tv_screen_in_world(&mut self, world_id: u32, target_world_id: Option<u32>) {
        let Some(instance) = self.worlds.get_mut(&world_id) else {
            return;
        };

        let counter = self.tv_screen_spawn_counter;
        self.tv_screen_spawn_counter += 1;

        let angle = (counter as f32 * 1.2) % std::f32::consts::TAU;
        let radius = 3.0 + (counter % 3) as f32 * 1.5;
        let height = 1.5 + (counter % 4) as f32 * 0.5;
        let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

        let entity = spawn_mesh(
            &mut instance.world,
            "Plane",
            position,
            Vec3::new(2.0, 2.0, 1.0),
        );

        let direction = -position.normalize();
        let rotation = nalgebra_glm::quat_look_at(&direction, &Vec3::new(0.0, 1.0, 0.0));

        instance.world.core.set_local_transform(
            entity,
            LocalTransform {
                translation: position,
                scale: Vec3::new(2.0, 2.0, 1.0),
                rotation,
            },
        );

        instance
            .world
            .core
            .set_name(entity, Name(format!("TVScreen_{}", counter)));

        let texture_name = if let Some(target_id) = target_world_id {
            format!("world_{}_render", target_id)
        } else {
            "fallback_checkerboard".to_string()
        };

        let material_name = format!("TVScreen_{}_{}", world_id, counter);
        let material = Material {
            base_color: [0.1, 0.1, 0.1, 1.0],
            emissive_factor: [1.0, 1.0, 1.0],
            emissive_texture: Some(texture_name.clone()),
            unlit: true,
            ..Default::default()
        };

        material_registry_insert(
            &mut instance.world.resources.material_registry,
            material_name.clone(),
            material,
        );

        instance
            .world
            .core
            .set_material_ref(entity, MaterialRef::new(material_name));

        instance.dirty = true;
        if let Some(target_id) = target_world_id {
            tracing::info!(
                "Spawned TV screen {} showing World {} in {}",
                counter,
                target_id,
                instance.name
            );
        } else {
            tracing::info!(
                "Spawned TV screen {} with fallback in {}",
                counter,
                instance.name
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn add_web_tile(&mut self) {
        let web_widget = WebWidget::default();
        let pane = PaneKind::Web(web_widget);

        if let Some(tree) = &mut self.tile_tree {
            let new_tile_id = tree.tiles.insert_pane(pane);
            if let Some(root_id) = tree.root()
                && let Some(egui_tiles::Tile::Container(container)) = tree.tiles.get_mut(root_id)
            {
                match container {
                    egui_tiles::Container::Tabs(tabs) => {
                        tabs.add_child(new_tile_id);
                    }
                    egui_tiles::Container::Linear(linear) => {
                        linear.add_child(new_tile_id);
                    }
                    egui_tiles::Container::Grid(grid) => {
                        grid.add_child(new_tile_id);
                    }
                }
            }
        } else {
            let mut tiles = egui_tiles::Tiles::default();
            let tile_id = tiles.insert_pane(pane);
            self.tile_tree = Some(egui_tiles::Tree::new("multi_world_tiles", tile_id, tiles));
        }
        tracing::info!("Added web tile");
    }
}

fn generate_scene_content(world: &mut World, seed: u32) -> Vec<AnimatedEntity> {
    let scene_type = seed % 4;

    match scene_type {
        0 => generate_tower_scene(world, seed),
        1 => generate_scatter_scene(world, seed),
        2 => generate_grid_scene(world, seed),
        _ => generate_spiral_scene(world, seed),
    }
}

fn generate_tower_scene(world: &mut World, seed: u32) -> Vec<AnimatedEntity> {
    let mut animated = Vec::new();
    let tower_count = 3 + (seed % 3) as i32;

    for tower_index in 0..tower_count {
        let tower_x = (tower_index as f32 - tower_count as f32 / 2.0) * 4.0;
        let tower_height = 3 + ((seed + tower_index as u32) % 5) as i32;

        for level in 0..tower_height {
            let y = level as f32 + 0.5;
            let scale = 1.0 - (level as f32 * 0.1);
            let position = Vec3::new(tower_x, y, 0.0);
            let scale_vec = Vec3::new(scale, 1.0, scale);

            let entity = spawn_scaled_cube(
                world,
                position,
                scale_vec,
                format!("Tower{}_{}", tower_index, level),
            );

            animated.push(AnimatedEntity {
                entity,
                original_translation: position,
                original_scale: scale_vec,
                animation_type: ((seed + tower_index as u32 + level as u32) % 6),
                speed: 0.3 + ((seed + level as u32) % 10) as f32 * 0.1,
                phase: (tower_index as f32 * 0.5 + level as f32 * 0.3),
            });
        }
    }

    animated
}

fn generate_scatter_scene(world: &mut World, seed: u32) -> Vec<AnimatedEntity> {
    let mut animated = Vec::new();
    let object_count = 8 + (seed % 8) as i32;

    for index in 0..object_count {
        let angle = (index as f32 / object_count as f32) * std::f32::consts::TAU;
        let radius = 2.0 + ((seed + index as u32) % 5) as f32;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        let y = 0.5 + ((seed * (index as u32 + 1)) % 3) as f32 * 0.5;

        let scale = 0.5 + ((seed + index as u32) % 3) as f32 * 0.3;
        let position = Vec3::new(x, y, z);
        let scale_vec = Vec3::new(scale, scale, scale);

        let entity = spawn_scaled_cube(world, position, scale_vec, format!("Scatter_{}", index));

        animated.push(AnimatedEntity {
            entity,
            original_translation: position,
            original_scale: scale_vec,
            animation_type: ((seed + index as u32) % 6),
            speed: 0.3 + (index % 10) as f32 * 0.1,
            phase: (index as f32) * 0.7,
        });
    }

    animated
}

fn generate_grid_scene(world: &mut World, seed: u32) -> Vec<AnimatedEntity> {
    let mut animated = Vec::new();
    let grid_size = 2 + (seed % 3) as i32;
    let spacing = 2.5;

    for x_index in -grid_size..=grid_size {
        for z_index in -grid_size..=grid_size {
            if x_index == 0 && z_index == 0 {
                continue;
            }

            let height = ((seed as i32 + x_index.abs() + z_index.abs()) % 4) as f32 * 0.5 + 0.5;
            let x = x_index as f32 * spacing;
            let z = z_index as f32 * spacing;
            let position = Vec3::new(x, height / 2.0, z);
            let scale_vec = Vec3::new(0.8, height, 0.8);

            let entity = spawn_scaled_cube(
                world,
                position,
                scale_vec,
                format!("Grid_{}_{}", x_index + grid_size, z_index + grid_size),
            );

            let entity_seed = (x_index.abs() + z_index.abs()) as u32;
            animated.push(AnimatedEntity {
                entity,
                original_translation: position,
                original_scale: scale_vec,
                animation_type: ((seed + entity_seed) % 6),
                speed: 0.3 + (entity_seed % 10) as f32 * 0.1,
                phase: (entity_seed as f32) * 0.5,
            });
        }
    }

    animated
}

fn generate_spiral_scene(world: &mut World, seed: u32) -> Vec<AnimatedEntity> {
    let mut animated = Vec::new();
    let steps = 12 + (seed % 8) as i32;
    let height_per_step = 0.3;
    let radius_growth = 0.2;

    for step in 0..steps {
        let angle = (step as f32 / 4.0) * std::f32::consts::TAU;
        let radius = 1.5 + step as f32 * radius_growth;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        let y = step as f32 * height_per_step + 0.5;

        let position = Vec3::new(x, y, z);
        let scale_vec = Vec3::new(0.6, 0.6, 0.6);

        let entity = spawn_scaled_cube(world, position, scale_vec, format!("Spiral_{}", step));

        animated.push(AnimatedEntity {
            entity,
            original_translation: position,
            original_scale: scale_vec,
            animation_type: ((seed + step as u32) % 6),
            speed: 0.3 + (step % 10) as f32 * 0.1,
            phase: (step as f32) * 0.7,
        });
    }

    animated
}

fn spawn_floor(world: &mut World) {
    let entity = spawn_mesh(
        world,
        "Cube",
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(20.0, 1.0, 20.0),
    );
    world.core.set_name(entity, Name("Floor".to_string()));
    world
        .core
        .set_material_ref(entity, MaterialRef::new("White".to_string()));
}

fn spawn_scaled_cube(world: &mut World, position: Vec3, scale: Vec3, name: String) -> Entity {
    let entity = spawn_mesh(world, "Cube", position, scale);
    let name_len = name.len();
    world.core.set_name(entity, Name(name));

    let colors = [
        "Red", "Green", "Blue", "Yellow", "Cyan", "Magenta", "Orange",
    ];
    let color_index = (entity.id as usize + name_len) % colors.len();
    world
        .core
        .set_material_ref(entity, MaterialRef::new(colors[color_index].to_string()));

    entity
}

fn spawn_point_light(
    world: &mut World,
    position: Vec3,
    color: [f32; 3],
    intensity: f32,
    range: f32,
) -> Entity {
    let entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::LIGHT,
        1,
    )[0];

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Point,
            color: Vec3::new(color[0], color[1], color[2]),
            intensity,
            range,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: false,
            shadow_bias: 0.007,
        },
    );

    entity
}

struct SpotlightParams {
    position: Vec3,
    direction: Vec3,
    color: [f32; 3],
    intensity: f32,
    range: f32,
    inner_cone_angle: f32,
    outer_cone_angle: f32,
}

fn spawn_spotlight(world: &mut World, params: SpotlightParams) -> Entity {
    let entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::LIGHT,
        1,
    )[0];

    let rotation = nalgebra_glm::quat_rotation(&Vec3::new(0.0, -1.0, 0.0), &params.direction);

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: params.position,
            rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Spot,
            color: Vec3::new(params.color[0], params.color[1], params.color[2]),
            intensity: params.intensity,
            range: params.range,
            inner_cone_angle: params.inner_cone_angle,
            outer_cone_angle: params.outer_cone_angle,
            cast_shadows: false,
            shadow_bias: 0.007,
        },
    );

    entity
}

fn spawn_3d_text_at(world: &mut World, text_str: &str, position: Vec3, size: f32) -> Entity {
    let text_index = world.resources.text_cache.add_text(text_str);

    let entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::TEXT
            | nightshade::ecs::world::MATERIAL_REF
            | nightshade::ecs::world::VISIBILITY,
        1,
    )[0];

    world.core.set_name(entity, Name(text_str.to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(size, size, size),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_text(entity, Text::new(text_index));
    world
        .core
        .set_visibility(entity, Visibility { visible: true });

    entity
}

fn animate_objects_fixed(world: &mut World, total_time: f32, animated_entities: &[AnimatedEntity]) {
    for anim in animated_entities {
        let Some(transform) = world.core.get_local_transform_mut(anim.entity) else {
            continue;
        };

        let time = total_time * anim.speed + anim.phase;

        match anim.animation_type {
            0 => {
                let bob_amount = 0.3;
                transform.translation = anim.original_translation;
                transform.translation.y += (time * 2.0).sin() * bob_amount;
            }
            1 => {
                transform.translation = anim.original_translation;
                transform.scale = anim.original_scale;
                let angle = time * 1.5;
                transform.rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::y());
            }
            2 => {
                let orbit_radius = 0.3;
                transform.translation = anim.original_translation;
                transform.translation.x += time.cos() * orbit_radius;
                transform.translation.z += time.sin() * orbit_radius;
            }
            3 => {
                let pulse_amount = 0.15;
                let scale_factor = 1.0 + (time * 3.0).sin() * pulse_amount;
                transform.translation = anim.original_translation;
                transform.scale = anim.original_scale * scale_factor;
            }
            4 => {
                transform.translation = anim.original_translation;
                transform.scale = anim.original_scale;
                let wobble_amount = 0.25;
                let wobble_x = (time * 2.5).sin() * wobble_amount;
                let wobble_z = (time * 2.5 + 1.0).cos() * wobble_amount;
                transform.rotation = nalgebra_glm::quat_angle_axis(wobble_x, &Vec3::x())
                    * nalgebra_glm::quat_angle_axis(wobble_z, &Vec3::z());
            }
            _ => {
                let float_amount = 0.5;
                transform.translation = anim.original_translation;
                transform.translation.y += (time * 0.8).sin() * float_amount;
                transform.scale = anim.original_scale;
                let gentle_spin = time * 0.5;
                transform.rotation = nalgebra_glm::quat_angle_axis(gentle_spin, &Vec3::y());
            }
        }

        world
            .core
            .set_local_transform_dirty(anim.entity, LocalTransformDirty);
    }
}

struct GamepadInput {
    left_stick_x: f32,
    left_stick_y: f32,
    right_stick_x: f32,
    right_stick_y: f32,
    left_trigger: f32,
    right_trigger: f32,
}

fn read_gamepad_input(world: &World) -> Option<GamepadInput> {
    let gilrs = world.resources.input.gamepad.gilrs.as_ref()?;
    let gamepad_id = world.resources.input.gamepad.gamepad?;
    let gamepad = gilrs.gamepad(gamepad_id);

    Some(GamepadInput {
        left_stick_x: gamepad
            .axis_data(gilrs::Axis::LeftStickX)
            .map(|a| a.value())
            .unwrap_or(0.0),
        left_stick_y: gamepad
            .axis_data(gilrs::Axis::LeftStickY)
            .map(|a| a.value())
            .unwrap_or(0.0),
        right_stick_x: gamepad
            .axis_data(gilrs::Axis::RightStickX)
            .map(|a| a.value())
            .unwrap_or(0.0),
        right_stick_y: gamepad
            .axis_data(gilrs::Axis::RightStickY)
            .map(|a| a.value())
            .unwrap_or(0.0),
        left_trigger: if gamepad.is_pressed(gilrs::Button::LeftTrigger2) {
            1.0
        } else {
            0.0
        },
        right_trigger: if gamepad.is_pressed(gilrs::Button::RightTrigger2) {
            1.0
        } else {
            0.0
        },
    })
}

fn apply_gamepad_to_pan_orbit(world: &mut World, input: &GamepadInput, delta_time: f32) {
    let camera_entity = world.resources.active_camera;
    let Some(camera_entity) = camera_entity else {
        return;
    };

    let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera_entity) else {
        return;
    };

    let deadzone = pan_orbit.gamepad_deadzone;

    let apply_deadzone = |value: f32| -> f32 {
        if value.abs() > deadzone {
            let sign = value.signum();
            (value.abs() - deadzone) / (1.0 - deadzone) * sign
        } else {
            0.0
        }
    };

    let orbit_x = apply_deadzone(input.right_stick_x);
    let orbit_y = apply_deadzone(input.right_stick_y);
    let pan_x = apply_deadzone(input.left_stick_x);
    let pan_y = apply_deadzone(input.left_stick_y);
    let zoom_input = input.left_trigger - input.right_trigger;

    if orbit_x.abs() > 0.001 || orbit_y.abs() > 0.001 {
        pan_orbit.target_yaw -= orbit_x * pan_orbit.gamepad_orbit_sensitivity * delta_time;
        pan_orbit.target_pitch += orbit_y * pan_orbit.gamepad_orbit_sensitivity * delta_time;
        pan_orbit.target_pitch = pan_orbit
            .target_pitch
            .clamp(pan_orbit.pitch_lower_limit, pan_orbit.pitch_upper_limit);
    }

    if pan_x.abs() > 0.001 || pan_y.abs() > 0.001 {
        let yaw = pan_orbit.target_yaw;
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let forward_flat = Vec3::new(yaw.sin(), 0.0, yaw.cos());
        let pan_scale =
            pan_orbit.target_radius * pan_orbit.gamepad_pan_sensitivity * delta_time * 0.1;
        pan_orbit.target_focus += right * pan_x * pan_scale;
        pan_orbit.target_focus += forward_flat * pan_y * pan_scale;
    }

    if zoom_input.abs() > 0.1 {
        let zoom_delta = zoom_input
            * pan_orbit.target_radius
            * pan_orbit.gamepad_zoom_sensitivity
            * delta_time
            * 0.1;
        let max_radius = pan_orbit.zoom_upper_limit.unwrap_or(f32::MAX);
        pan_orbit.target_radius =
            (pan_orbit.target_radius + zoom_delta).clamp(pan_orbit.zoom_lower_limit, max_radius);
    }
}
