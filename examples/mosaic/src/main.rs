use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use nightshade::mosaic::Settings;
use nightshade::mosaic::{
    EventLog, FpsCounter, Mosaic, PendingFileLoad, StatusBar, ThemeState, ToastKind, Toasts,
    ViewportWidget, Widget, WidgetContext, WidgetEntry, apply_theme, render_theme_editor_window,
};
use nightshade::prelude::*;

use std::f32::consts::TAU;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(BasicApp::default())?;
    Ok(())
}

#[derive(Default, Clone, Copy, PartialEq)]
enum LogCategory {
    #[default]
    System,
    Entity,
    Camera,
    Widget,
}

impl LogCategory {
    fn tag(&self) -> &'static str {
        match self {
            LogCategory::System => "SYS",
            LogCategory::Entity => "ENT",
            LogCategory::Camera => "CAM",
            LogCategory::Widget => "WGT",
        }
    }

    fn color(&self) -> egui::Color32 {
        match self {
            LogCategory::System => egui::Color32::from_rgb(180, 180, 180),
            LogCategory::Entity => egui::Color32::from_rgb(100, 200, 100),
            LogCategory::Camera => egui::Color32::from_rgb(100, 150, 255),
            LogCategory::Widget => egui::Color32::from_rgb(255, 180, 80),
        }
    }

    fn from_tag(tag: &str) -> Self {
        match tag {
            "SYS" => LogCategory::System,
            "ENT" => LogCategory::Entity,
            "CAM" => LogCategory::Camera,
            "WGT" => LogCategory::Widget,
            _ => LogCategory::System,
        }
    }
}

enum AppMessage {
    Log {
        category: LogCategory,
        message: String,
    },
    Toast {
        message: String,
        kind: ToastKind,
        duration: f32,
    },
}

struct AppContext {
    cube_counter: u32,
    sphere_counter: u32,
    light_counter: u32,
    scene_view_counter: u32,
    event_log: EventLog,
    fps_counter: FpsCounter,
    toast_counter: u32,
}

impl Default for AppContext {
    fn default() -> Self {
        Self {
            cube_counter: 0,
            sphere_counter: 0,
            light_counter: 0,
            scene_view_counter: 0,
            event_log: EventLog::new(500),
            fps_counter: FpsCounter::default(),
            toast_counter: 0,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct BasicAppSettings {
    theme_name: Option<String>,
}

struct BasicApp {
    primary: Mosaic<AppWidget, AppContext, AppMessage>,
    secondary: HashMap<usize, Mosaic<AppWidget, AppContext, AppMessage>>,
    context: AppContext,
    toasts: Toasts,
    theme_state: ThemeState,
    status_bar: StatusBar,
    #[cfg(not(target_arch = "wasm32"))]
    settings: Option<Settings<BasicAppSettings>>,
    pending_file_load: Option<PendingFileLoad>,
    pending_messages: Vec<AppMessage>,
    new_window_cooldown: f32,
    active_window: Option<usize>,
    window_counter: u32,
}

impl Default for BasicApp {
    fn default() -> Self {
        Self {
            primary: Mosaic::new(),
            secondary: HashMap::new(),
            context: AppContext::default(),
            toasts: Toasts::new(),
            theme_state: ThemeState::default(),
            status_bar: StatusBar::new(),
            #[cfg(not(target_arch = "wasm32"))]
            settings: None,
            pending_file_load: None,
            pending_messages: Vec::new(),
            new_window_cooldown: 0.0,
            active_window: None,
            window_counter: 1,
        }
    }
}

impl State for BasicApp {
    fn title(&self) -> &str {
        "Nightshade Mosaic"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let settings: Settings<BasicAppSettings> = Settings::load("nightshade-mosaic-basic");
            if let Some(theme_name) = &settings.data.theme_name {
                self.theme_state.select_preset_by_name(theme_name);
            }
            self.settings = Some(settings);
        }

        spawn_sun(world);

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            15.0,
            0.0,
            std::f32::consts::FRAC_PI_4,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);

        self.pending_messages.push(AppMessage::Log {
            category: LogCategory::System,
            message: "Application started".to_string(),
        });

        self.primary = Mosaic::with_panes(vec![
            AppWidget::Viewport(ViewportWidget { camera_index: 0 }),
            AppWidget::SceneGraph(SceneGraphWidget),
            AppWidget::Properties(PropertiesWidget),
        ])
        .with_title("Primary Window");
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        apply_theme(ui_context, &self.theme_state);
        Mosaic::<AppWidget, AppContext, AppMessage>::clear_required_cameras(world);

        if ui_context.is_pointer_over_area() {
            self.active_window = None;
        }

        self.primary.is_active_window = self.active_window.is_none();
        self.render_toolbar(world, ui_context, None);
        self.update_status_bar(world);
        self.status_bar.render(ui_context);
        self.primary.show(world, ui_context, &mut self.context);

        if render_theme_editor_window(ui_context, &mut self.theme_state) {
            self.save_theme_to_settings();
        }

        self.toasts.render(ui_context);
    }

    fn secondary_ui(&mut self, world: &mut World, window_index: usize, ui_context: &egui::Context) {
        apply_theme(ui_context, &self.theme_state);

        self.secondary.entry(window_index).or_insert_with(|| {
            self.window_counter += 1;
            let window_title = format!("Window {}", self.window_counter);
            self.pending_messages.push(AppMessage::Log {
                category: LogCategory::Widget,
                message: format!("Opened {}", window_title),
            });
            let mut mosaic = Mosaic::with_panes(vec![
                AppWidget::Viewport(ViewportWidget { camera_index: 0 }),
                AppWidget::SceneGraph(SceneGraphWidget),
                AppWidget::Properties(PropertiesWidget),
            ])
            .with_title(window_title);
            mosaic.set_viewport_textures(vec![]);
            mosaic.window_index = Some(window_index);
            mosaic
        });

        if ui_context.is_pointer_over_area() {
            self.active_window = Some(window_index);
        }

        self.render_toolbar(world, ui_context, Some(window_index));

        if let Some(mosaic) = self.secondary.get_mut(&window_index) {
            mosaic.is_active_window = self.active_window == Some(window_index);
            mosaic.show(world, ui_context, &mut self.context);
        }

        self.toasts.render(ui_context);
    }

    fn pre_render(&mut self, renderer: &mut dyn Render, world: &mut World) {
        let cameras = world.resources.user_interface.required_cameras.clone();
        for (&window_index, mosaic) in &mut self.secondary {
            let textures = renderer.register_camera_viewports_for_secondary(window_index, &cameras);
            mosaic.set_viewport_textures(textures);
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let original_mouse = world.resources.input.mouse;
        let original_hud_wants_pointer = world.resources.user_interface.hud_wants_pointer;

        if let Some(window_index) = self.active_window
            && let Some(secondary_window) = world
                .resources
                .secondary_windows
                .states
                .iter()
                .find(|w| w.index == window_index)
        {
            world.resources.input.mouse.position = secondary_window.input.mouse_position;
            world.resources.input.mouse.position_delta =
                secondary_window.input.mouse_position_delta;
            world.resources.input.mouse.raw_mouse_delta = secondary_window.input.raw_mouse_delta;
            world.resources.input.mouse.wheel_delta = secondary_window.input.mouse_wheel_delta;
            world.resources.input.mouse.state = secondary_window.input.mouse_state;
            world.resources.user_interface.hud_wants_pointer = false;
        }

        pan_orbit_camera_system(world);

        world.resources.input.mouse = original_mouse;
        world.resources.user_interface.hud_wants_pointer = original_hud_wants_pointer;

        let delta_time = world.resources.window.timing.delta_time;
        self.context.event_log.tick(delta_time);
        self.context.fps_counter.tick(delta_time);
        self.toasts.tick(delta_time);
        self.new_window_cooldown = (self.new_window_cooldown - delta_time).max(0.0);

        if self.new_window_cooldown <= 0.0
            && world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::ControlLeft)
            && world.resources.input.keyboard.is_key_pressed(KeyCode::KeyN)
        {
            self.pending_messages.push(AppMessage::Log {
                category: LogCategory::Widget,
                message: "Opening new window".to_string(),
            });
            let next_window = self.window_counter + 1;
            world
                .resources
                .secondary_windows
                .pending_spawns
                .push(WindowSpawnRequest {
                    title: format!("Nightshade Mosaic - Window {}", next_window),
                    width: 1280,
                    height: 720,
                    egui_enabled: true,
                });
            self.new_window_cooldown = 0.5;
        }

        let active_indices: Vec<usize> = world
            .resources
            .secondary_windows
            .states
            .iter()
            .map(|window| window.index)
            .collect();
        self.secondary
            .retain(|index, _| active_indices.contains(index));

        self.process_pending_file_load();
        self.process_messages();
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::Escape, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}

impl BasicApp {
    fn process_messages(&mut self) {
        let mut messages = self.primary.drain_messages();
        for mosaic in self.secondary.values_mut() {
            messages.extend(mosaic.drain_messages());
        }
        messages.append(&mut self.pending_messages);

        for message in messages {
            match message {
                AppMessage::Log { category, message } => {
                    self.context.event_log.log(category.tag(), message);
                }
                AppMessage::Toast {
                    message,
                    kind,
                    duration,
                } => {
                    self.toasts.push(kind, message, duration);
                }
            }
        }
    }

    fn update_status_bar(&mut self, world: &World) {
        self.status_bar.clear();

        self.status_bar
            .add_left(format!("FPS: {}", self.context.fps_counter.fps_rounded()));

        let entity_count = world.query_entities(RENDER_MESH | GLOBAL_TRANSFORM).count();
        self.status_bar
            .add_left(format!("Entities: {}", entity_count));

        self.status_bar
            .add_left(format!("Widgets: {}", self.primary.widget_count()));

        self.status_bar
            .add_right(format!("Theme: {}", self.theme_state.current_config.name));

        let window_count = 1 + self.secondary.len();
        if window_count > 1 {
            self.status_bar
                .add_right(format!("Windows: {}", window_count));
        }
    }

    fn save_theme_to_settings(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(settings) = &mut self.settings {
            settings.data.theme_name = Some(self.theme_state.current_config.name.clone());
            let _ = settings.save();
        }
    }

    fn render_toolbar(
        &mut self,
        world: &mut World,
        ui_context: &egui::Context,
        window_index: Option<usize>,
    ) {
        egui::TopBottomPanel::top(egui::Id::new("toolbar").with(window_index)).show(
            ui_context,
            |ui| {
                ui.horizontal(|ui| {
                    let mosaic_title = if let Some(index) = window_index {
                        self.secondary
                            .get(&index)
                            .map(|mosaic| mosaic.title.clone())
                            .unwrap_or_else(|| "Window".to_string())
                    } else {
                        self.primary.title.clone()
                    };
                    ui.label(&mosaic_title);
                    ui.separator();

                    if window_index.is_none() && ui.button("New Window").clicked() {
                        let next_window = self.window_counter + 1;
                        world
                            .resources
                            .secondary_windows
                            .pending_spawns
                            .push(WindowSpawnRequest {
                                title: format!("Nightshade Mosaic - Window {}", next_window),
                                width: 1280,
                                height: 720,
                                egui_enabled: true,
                            });
                    }

                    ui.separator();

                    if ui.button("+ Cube").clicked() {
                        let counter = spawn_cube(world, &mut self.context);
                        self.pending_messages.push(AppMessage::Log {
                            category: LogCategory::Entity,
                            message: format!("Spawned Cube {}", counter),
                        });
                    }

                    if ui.button("+ Sphere").clicked() {
                        let counter = spawn_sphere(world, &mut self.context);
                        self.pending_messages.push(AppMessage::Log {
                            category: LogCategory::Entity,
                            message: format!("Spawned Sphere {}", counter),
                        });
                    }

                    if ui.button("+ Light").clicked() {
                        let counter = spawn_colored_point_light(world, &mut self.context);
                        self.pending_messages.push(AppMessage::Log {
                            category: LogCategory::Entity,
                            message: format!("Spawned Light {}", counter),
                        });
                    }

                    ui.separator();

                    if ui.button("Toast").clicked() {
                        self.context.toast_counter += 1;
                        let kinds = [
                            ToastKind::Info,
                            ToastKind::Success,
                            ToastKind::Warning,
                            ToastKind::Error,
                        ];
                        let kind = kinds[self.context.toast_counter as usize % kinds.len()];
                        self.pending_messages.push(AppMessage::Toast {
                            message: format!("Notification #{}", self.context.toast_counter),
                            kind,
                            duration: 3.0,
                        });
                    }

                    if ui.button("Theme").clicked() {
                        self.theme_state.show_theme_editor = !self.theme_state.show_theme_editor;
                    }

                    ui.separator();

                    if ui.button("Save Project").clicked() {
                        self.save_project_dialog();
                    }

                    if ui.button("Load Project").clicked() {
                        self.load_project_dialog();
                    }

                    let entity_count = world.query_entities(RENDER_MESH | GLOBAL_TRANSFORM).count();
                    ui.separator();
                    ui.label(format!("Entities: {}", entity_count));

                    let mosaic = if let Some(index) = window_index {
                        self.secondary.get(&index)
                    } else {
                        Some(&self.primary)
                    };

                    if let Some(mosaic) = mosaic {
                        ui.separator();
                        ui.label(format!("Widgets: {}", mosaic.widget_count()));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("ESC to exit | Ctrl+N new window");
                    });
                });
            },
        );
    }

    fn save_project_dialog(&mut self) {
        match self.primary.save_project_to_file("project.json") {
            Ok(()) => {
                self.pending_messages.push(AppMessage::Toast {
                    message: "Project saved".to_string(),
                    kind: ToastKind::Success,
                    duration: 3.0,
                });
            }
            Err(error) => {
                self.pending_messages.push(AppMessage::Toast {
                    message: format!("Failed to save: {}", error),
                    kind: ToastKind::Error,
                    duration: 4.0,
                });
            }
        }
    }

    fn load_project_dialog(&mut self) {
        self.pending_file_load = Some(self.primary.request_project_load());
    }

    fn process_pending_file_load(&mut self) {
        if let Some(pending) = &self.pending_file_load
            && let Some(loaded) = pending.take()
        {
            self.pending_file_load = None;
            match self.primary.load_project_from_bytes(&loaded.bytes) {
                Ok(()) => {
                    self.pending_messages.push(AppMessage::Toast {
                        message: format!("Loaded project: {}", loaded.name),
                        kind: ToastKind::Success,
                        duration: 3.0,
                    });
                }
                Err(error) => {
                    self.pending_messages.push(AppMessage::Toast {
                        message: format!("Failed to load: {}", error),
                        kind: ToastKind::Error,
                        duration: 4.0,
                    });
                }
            }
        }
    }
}

enum SpawnAction {
    Cube,
    Sphere,
    Light,
}

fn spawn_cube(world: &mut World, context: &mut AppContext) -> u32 {
    let counter = context.cube_counter;
    context.cube_counter += 1;

    let angle = (counter as f32 * 0.6) % std::f32::consts::TAU;
    let radius = 2.0 + (counter % 4) as f32 * 1.5;
    let height = 0.5 + (counter % 3) as f32 * 0.5;
    let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

    let scale = 0.5 + (counter % 5) as f32 * 0.2;
    let entity = spawn_mesh(world, "Cube", position, Vec3::new(scale, scale, scale));
    world.set_name(entity, Name(format!("Cube {}", counter)));

    let colors = [
        "Red", "Green", "Blue", "Yellow", "Cyan", "Magenta", "Orange", "White",
    ];
    let color = colors[counter as usize % colors.len()];
    world.set_material_ref(entity, MaterialRef::new(color.to_string()));

    counter
}

fn spawn_sphere(world: &mut World, context: &mut AppContext) -> u32 {
    let counter = context.sphere_counter;
    context.sphere_counter += 1;

    let angle = (counter as f32 * 0.9 + 0.5) % std::f32::consts::TAU;
    let radius = 3.0 + (counter % 3) as f32;
    let height = 1.0 + (counter % 4) as f32 * 0.3;
    let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

    let entity = spawn_mesh(world, "Sphere", position, Vec3::new(0.5, 0.5, 0.5));
    world.set_name(entity, Name(format!("Sphere {}", counter)));

    let colors = ["Cyan", "Magenta", "Yellow", "White", "Red", "Green", "Blue"];
    let color = colors[counter as usize % colors.len()];
    world.set_material_ref(entity, MaterialRef::new(color.to_string()));

    counter
}

fn spawn_colored_point_light(world: &mut World, context: &mut AppContext) -> u32 {
    let counter = context.light_counter;
    context.light_counter += 1;

    let palette: [[f32; 3]; 7] = [
        [1.0, 0.3, 0.3],
        [0.3, 1.0, 0.3],
        [0.3, 0.3, 1.0],
        [1.0, 1.0, 0.3],
        [1.0, 0.3, 1.0],
        [0.3, 1.0, 1.0],
        [1.0, 0.6, 0.2],
    ];
    let color = palette[counter as usize % palette.len()];

    let angle = (counter as f32 * 1.3) % std::f32::consts::TAU;
    let radius = 4.0 + (counter % 3) as f32 * 2.0;
    let height = 3.0 + (counter % 4) as f32;
    let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];

    world.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.set_local_transform_dirty(entity, LocalTransformDirty);
    world.set_global_transform(entity, GlobalTransform::default());
    world.set_light(
        entity,
        Light {
            light_type: LightType::Point,
            color: Vec3::new(color[0], color[1], color[2]),
            intensity: 500.0 + (counter % 5) as f32 * 200.0,
            range: 20.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: false,
            shadow_bias: 0.007,
        },
    );
    world.set_name(entity, Name(format!("Light {}", counter)));

    let bulb_entity = spawn_mesh(world, "Sphere", position, Vec3::new(0.15, 0.15, 0.15));
    world.set_name(bulb_entity, Name(format!("LightBulb {}", counter)));

    counter
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct SceneViewWidget {
    scene_index: u32,
    #[serde(skip)]
    camera_entity: Option<Entity>,
}

impl SceneViewWidget {
    fn new(scene_index: u32) -> Self {
        Self {
            scene_index,
            camera_entity: None,
        }
    }

    fn title(&self) -> String {
        format!("Scene View {}", self.scene_index + 1)
    }

    fn on_add(&mut self, context: &mut WidgetContext<AppContext, AppMessage>) {
        let yaw = (self.scene_index as f32 * 0.7) % TAU;
        let radius = 10.0 + (self.scene_index as f32 * 0.5).sin() * 5.0;
        let name = format!("Scene Camera {}", self.scene_index + 1);

        context.send(AppMessage::Log {
            category: LogCategory::Camera,
            message: format!("Created {}", name),
        });

        let entity = spawn_pan_orbit_camera(
            context.world_mut(),
            Vec3::new(0.0, 2.0, 0.0),
            radius,
            yaw,
            std::f32::consts::FRAC_PI_4,
            name,
        );
        self.camera_entity = Some(entity);
    }

    fn on_remove(&mut self, context: &mut WidgetContext<AppContext, AppMessage>) {
        context.send(AppMessage::Log {
            category: LogCategory::Camera,
            message: format!("Removed Scene Camera {}", self.scene_index + 1),
        });
        if let Some(entity) = self.camera_entity.take() {
            despawn_recursive_immediate(context.world_mut(), entity);
        }
    }

    fn required_camera(&self, _cached_cameras: &[Entity]) -> Option<Entity> {
        self.camera_entity
    }

    fn ensure_camera(&mut self, context: &mut WidgetContext<AppContext, AppMessage>) {
        if self.camera_entity.is_none() {
            self.on_add(context);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut WidgetContext<AppContext, AppMessage>) {
        self.ensure_camera(context);

        let rect = ui.available_rect_before_wrap();
        let tile_id = context.current_tile_id;

        let camera_entity = self.camera_entity;

        let texture_index = camera_entity.and_then(|camera| {
            context
                .world()
                .resources
                .user_interface
                .required_cameras
                .iter()
                .position(|&entity| entity == camera)
        });

        let clicked = if let Some(index) = texture_index
            && let Some(texture_id) = context.viewport_textures.get(index)
        {
            let image = egui::Image::new(egui::load::SizedTexture::new(
                *texture_id,
                egui::vec2(rect.width(), rect.height()),
            ))
            .fit_to_exact_size(egui::vec2(rect.width(), rect.height()))
            .sense(egui::Sense::click());

            ui.put(rect, image).clicked()
        } else {
            ui.allocate_rect(rect, egui::Sense::click()).clicked()
        };

        let pixels_per_point = ui.ctx().pixels_per_point();

        if context.selected_viewport_tile.is_none() {
            *context.selected_viewport_tile = Some(tile_id);
            if let Some(camera) = camera_entity {
                context.world_mut().resources.active_camera = Some(camera);
            }
        }

        if clicked {
            *context.selected_viewport_tile = Some(tile_id);
            if let Some(camera) = camera_entity {
                context.world_mut().resources.active_camera = Some(camera);
            }
        }

        let is_selected = *context.selected_viewport_tile == Some(tile_id);
        if is_selected {
            if context.is_active_window {
                context.world_mut().resources.window.active_viewport_rect = Some(ViewportRect {
                    x: rect.min.x * pixels_per_point,
                    y: rect.min.y * pixels_per_point,
                    width: rect.width() * pixels_per_point,
                    height: rect.height() * pixels_per_point,
                });
                if let Some(camera) = camera_entity {
                    context.world_mut().resources.active_camera = Some(camera);
                }
            }

            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 165, 0)),
                egui::StrokeKind::Inside,
            );
        }
    }
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
struct LogWidget;

impl LogWidget {
    fn ui(&mut self, ui: &mut egui::Ui, context: &mut WidgetContext<AppContext, AppMessage>) {
        let rect = ui.available_rect_before_wrap();
        ui.painter()
            .rect_filled(rect, 0.0, ui.style().visuals.panel_fill);

        context
            .app
            .event_log
            .render(ui, |tag| LogCategory::from_tag(tag).color());
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
enum AppWidget {
    Viewport(ViewportWidget),
    SceneGraph(SceneGraphWidget),
    Properties(PropertiesWidget),
    SceneView(SceneViewWidget),
    Log(LogWidget),
}

impl Widget<AppContext, AppMessage> for AppWidget {
    fn title(&self) -> String {
        match self {
            AppWidget::Viewport(widget) => widget.title(),
            AppWidget::SceneGraph(_) => "Scene".to_string(),
            AppWidget::Properties(_) => "Properties".to_string(),
            AppWidget::SceneView(widget) => widget.title(),
            AppWidget::Log(_) => "Event Log".to_string(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut WidgetContext<AppContext, AppMessage>) {
        match self {
            AppWidget::Viewport(widget) => widget.ui(ui, context),
            AppWidget::SceneGraph(widget) => widget.ui(ui, context),
            AppWidget::Properties(widget) => widget.ui(ui, context),
            AppWidget::SceneView(widget) => widget.ui(ui, context),
            AppWidget::Log(widget) => widget.ui(ui, context),
        }
    }

    fn on_add(&mut self, context: &mut WidgetContext<AppContext, AppMessage>) {
        if let AppWidget::SceneView(widget) = self {
            let scene_index = context.app.scene_view_counter;
            context.app.scene_view_counter += 1;
            widget.scene_index = scene_index;
            widget.on_add(context);
        }
    }

    fn on_remove(&mut self, context: &mut WidgetContext<AppContext, AppMessage>) {
        if let AppWidget::SceneView(widget) = self {
            widget.on_remove(context);
        }
    }

    fn required_camera(&self, cached_cameras: &[Entity]) -> Option<Entity> {
        match self {
            AppWidget::Viewport(widget) => widget.required_camera(cached_cameras),
            AppWidget::SceneView(widget) => widget.required_camera(cached_cameras),
            AppWidget::SceneGraph(_) | AppWidget::Properties(_) | AppWidget::Log(_) => None,
        }
    }

    fn catalog() -> Vec<WidgetEntry<Self>> {
        vec![
            WidgetEntry {
                name: "Viewport".to_string(),
                create: || AppWidget::Viewport(ViewportWidget::default()),
            },
            WidgetEntry {
                name: "Scene View".to_string(),
                create: || AppWidget::SceneView(SceneViewWidget::new(0)),
            },
            WidgetEntry {
                name: "Scene Graph".to_string(),
                create: || AppWidget::SceneGraph(SceneGraphWidget),
            },
            WidgetEntry {
                name: "Properties".to_string(),
                create: || AppWidget::Properties(PropertiesWidget),
            },
            WidgetEntry {
                name: "Event Log".to_string(),
                create: || AppWidget::Log(LogWidget),
            },
        ]
    }
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
struct SceneGraphWidget;

impl SceneGraphWidget {
    fn ui(&mut self, ui: &mut egui::Ui, context: &mut WidgetContext<AppContext, AppMessage>) {
        let rect = ui.available_rect_before_wrap();
        ui.painter()
            .rect_filled(rect, 0.0, ui.style().visuals.panel_fill);

        ui.vertical(|ui| {
            ui.add_space(4.0);

            let mut spawn_action = None;

            ui.horizontal(|ui| {
                ui.strong("Scene Graph");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("+ Cube").clicked() {
                        spawn_action = Some(SpawnAction::Cube);
                    }
                    if ui.small_button("+ Sphere").clicked() {
                        spawn_action = Some(SpawnAction::Sphere);
                    }
                    if ui.small_button("+ Light").clicked() {
                        spawn_action = Some(SpawnAction::Light);
                    }
                });
            });

            if let Some(action) = spawn_action {
                let (world, app) = context.world_and_app();
                let (counter, entity_type) = match action {
                    SpawnAction::Cube => (spawn_cube(world, app), "Cube"),
                    SpawnAction::Sphere => (spawn_sphere(world, app), "Sphere"),
                    SpawnAction::Light => (spawn_colored_point_light(world, app), "Light"),
                };
                context.send(AppMessage::Log {
                    category: LogCategory::Entity,
                    message: format!("Spawned {} {}", entity_type, counter),
                });
            }

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let entities: Vec<Entity> = context.world().query_entities(NAME).collect();

                if entities.is_empty() {
                    ui.label("No named entities");
                    return;
                }

                for entity in &entities {
                    let name = context
                        .world()
                        .get_name(*entity)
                        .map(|name| name.0.clone())
                        .unwrap_or_else(|| format!("Entity {}", entity.id));

                    let has_mesh = context.world().get_render_mesh(*entity).is_some();
                    let has_light = context.world().get_light(*entity).is_some();
                    let has_camera = context.world().get_camera(*entity).is_some();

                    let icon = if has_camera {
                        "cam"
                    } else if has_light {
                        "lit"
                    } else if has_mesh {
                        "msh"
                    } else {
                        "   "
                    };

                    ui.horizontal(|ui| {
                        ui.monospace(format!("[{}]", icon));
                        ui.label(&name);
                    });
                }
            });
        });
    }
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
struct PropertiesWidget;

impl PropertiesWidget {
    fn ui(&mut self, ui: &mut egui::Ui, context: &mut WidgetContext<AppContext, AppMessage>) {
        let rect = ui.available_rect_before_wrap();
        ui.painter()
            .rect_filled(rect, 0.0, ui.style().visuals.panel_fill);

        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.strong("Properties");
            ui.separator();

            let timing = &context.world().resources.window.timing;

            egui::Grid::new("properties_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("FPS:");
                    ui.label(format!("{}", context.app.fps_counter.fps_rounded()));
                    ui.end_row();

                    ui.label("Delta:");
                    ui.label(format!("{:.1}ms", timing.delta_time * 1000.0));
                    ui.end_row();

                    let atmosphere = context.world().resources.graphics.atmosphere;
                    ui.label("Atmosphere:");
                    ui.label(format!("{:?}", atmosphere));
                    ui.end_row();

                    let grid_on = context.world().resources.graphics.show_grid;
                    ui.label("Grid:");
                    ui.label(if grid_on { "On" } else { "Off" });
                    ui.end_row();

                    let mesh_count = context.world().query_entities(RENDER_MESH).count();
                    ui.label("Meshes:");
                    ui.label(format!("{}", mesh_count));
                    ui.end_row();

                    let light_count = context.world().query_entities(LIGHT).count();
                    ui.label("Lights:");
                    ui.label(format!("{}", light_count));
                    ui.end_row();

                    let camera_count = context.world().query_entities(CAMERA).count();
                    ui.label("Cameras:");
                    ui.label(format!("{}", camera_count));
                    ui.end_row();
                });
        });
    }
}
