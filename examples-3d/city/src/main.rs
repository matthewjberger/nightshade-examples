mod atmosphere;
mod billboard;
mod building;
mod camera;
mod chunk;
mod city;
mod descriptors;
mod districts;
mod first_person;
mod interiors;
mod kenney;
mod materials;
mod minimap;
mod observer;
mod player_hud;
mod player_systems;
mod stroke_font;
mod tube_mesh;
mod waterfront;

use std::collections::HashMap;

use chunk::ChunkStreamer;
use nightshade::ecs::graphics::resources::{DepthOfField, DepthOfFieldQuality};
use nightshade::ecs::physics::{physics_debug_draw_system, run_physics_systems};
use nightshade::ecs::water::Water;
use nightshade::ecs::world::{WATER, WorldCommand};
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;

const MIN_CAMERA_Y: f32 = 2.0;

const FOG_STREET_START: f32 = 20.0;
const FOG_STREET_END: f32 = 180.0;
const FOG_SKY_START: f32 = 200.0;
const FOG_SKY_END: f32 = 700.0;

const FOG_HEIGHT_LOW: f32 = 5.0;
const FOG_HEIGHT_HIGH: f32 = 60.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(CityDemo::default())
}

struct CityDemo {
    camera_entity: Option<Entity>,
    sun_entity: Option<Entity>,
    ocean_entity: Option<Entity>,
    chunk_streamer: Option<ChunkStreamer>,
    minimap_enabled: bool,
    camera_controller: Option<camera::CameraController>,
    current_hour: f32,
    time_speed: f32,
    auto_time: bool,
    next_seed: u32,
    generate_requested: bool,
    city_half: i32,
    pending_city_half: i32,
    depth_of_field: DepthOfField,
    ssgi_enabled: bool,
    ssgi_intensity: f32,
    ssgi_radius: f32,
    ssgi_max_steps: u32,
    observer: Option<observer::ObserverCamera>,
    observer_enabled: bool,
    sun_shadows: bool,
    post_processing: bool,
    leaf_system: atmosphere::LeafSystem,
    district_hud: Option<districts::DistrictHud>,
    billboard_textures: billboard::BillboardTextures,

    first_person_mode: bool,
    player_entity: Option<Entity>,
    player_camera_entity: Option<Entity>,
    hands_entity: Option<Entity>,
    flashlight_entity: Option<Entity>,
    flashlight_on: bool,
    flashlight_key_was_pressed: bool,
    lean_state: player_systems::LeanState,
    show_collision: bool,
    ground_collider_entity: Option<Entity>,
    chunk_collision_entities: HashMap<(i32, i32), Vec<Entity>>,
    input_mode: player_systems::InputMode,
}

impl Default for CityDemo {
    fn default() -> Self {
        Self {
            camera_entity: None,
            sun_entity: None,
            ocean_entity: None,
            chunk_streamer: None,
            minimap_enabled: false,
            camera_controller: None,
            current_hour: 18.0,
            time_speed: 0.5,
            auto_time: true,
            next_seed: 0,
            generate_requested: false,
            city_half: 4,
            pending_city_half: 4,
            depth_of_field: DepthOfField {
                enabled: true,
                focus_distance: 3.5,
                focus_range: 50.0,
                max_blur_radius: 4.5,
                bokeh_threshold: 0.70,
                bokeh_intensity: 0.15,
                quality: DepthOfFieldQuality::High,
                visualize_coc: false,
                tilt_shift_enabled: false,
                tilt_shift_angle: 0.0,
                tilt_shift_center: 0.0,
                tilt_shift_blur_amount: 1.0,
                visualize_tilt_shift: false,
            },
            ssgi_enabled: true,
            ssgi_intensity: 0.5,
            ssgi_radius: 2.0,
            ssgi_max_steps: 16,
            observer: None,
            observer_enabled: false,
            sun_shadows: true,
            post_processing: !cfg!(feature = "openxr"),
            leaf_system: atmosphere::LeafSystem::new(),
            district_hud: None,
            billboard_textures: billboard::BillboardTextures::new(),

            first_person_mode: false,
            player_entity: None,
            player_camera_entity: None,
            hands_entity: None,
            flashlight_entity: None,
            flashlight_on: true,
            flashlight_key_was_pressed: false,
            lean_state: player_systems::LeanState::new(),
            show_collision: false,
            ground_collider_entity: None,
            chunk_collision_entities: HashMap::new(),
            input_mode: player_systems::InputMode::default(),
        }
    }
}

impl State for CityDemo {
    fn title(&self) -> &str {
        "Procedural City"
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let dof_texture = graph
            .add_color_texture("dof_output")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(width, height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let ssao_pass = passes::SsaoPass::new(device);
        graph
            .pass(Box::new(ssao_pass))
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .write("ssao_raw", resources.ssao_raw);

        let ssao_blur_pass = passes::SsaoBlurPass::new(device);
        graph
            .pass(Box::new(ssao_blur_pass))
            .read("ssao_raw", resources.ssao_raw)
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .write("ssao", resources.ssao);

        let ssgi_pass = passes::SsgiPass::new(device);
        graph
            .pass(Box::new(ssgi_pass))
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .read("scene_color", resources.scene_color)
            .write("ssgi_raw", resources.ssgi_raw);

        let ssgi_blur_pass = passes::SsgiBlurPass::new(device);
        graph
            .pass(Box::new(ssgi_blur_pass))
            .read("ssgi_raw", resources.ssgi_raw)
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .write("ssgi", resources.ssgi);

        let dof_pass =
            passes::DepthOfFieldPass::new(device, wgpu::TextureFormat::Rgba16Float, width, height);
        graph
            .pass(Box::new(dof_pass))
            .read("hdr", resources.scene_color)
            .read("depth", resources.depth)
            .write("dof_output", dof_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.01);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", dof_texture)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .read("ssgi", resources.ssgi)
            .write("output", resources.swapchain);
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.atmosphere = Atmosphere::DayNight;
        world.resources.graphics.day_night_hour = self.current_hour;
        capture_procedural_atmosphere_ibl(world, Atmosphere::DayNight, self.current_hour);
        capture_ibl_snapshots(
            world,
            Atmosphere::DayNight,
            vec![0.0, 6.0, 8.0, 12.0, 17.0, 18.5, 20.0],
        );
        world.resources.graphics.show_grid = false;
        world.resources.user_interface.enabled = true;
        world.resources.graphics.fog = Some(Fog {
            start: FOG_SKY_START,
            end: FOG_SKY_END,
            color: [0.75, 0.55, 0.45],
        });

        world.resources.graphics.bloom_enabled = self.post_processing;
        world.resources.graphics.bloom_intensity = 0.01;
        world.resources.graphics.ssao_enabled = self.post_processing;
        world.resources.graphics.ssao_radius = 0.5;
        world.resources.graphics.ssao_intensity = 0.5;
        world.resources.graphics.ambient_light = [0.25, 0.22, 0.20, 1.0];
        world.resources.graphics.occlusion_culling_enabled = true;
        if self.post_processing {
            world.resources.graphics.color_grading =
                ColorGradingPreset::Cinematic.to_color_grading();
        }
        world.resources.graphics.depth_of_field = if self.post_processing {
            self.depth_of_field
        } else {
            DepthOfField::default()
        };
        world.resources.graphics.ssgi_enabled = self.post_processing && self.ssgi_enabled;
        world.resources.graphics.ssgi_radius = self.ssgi_radius;
        world.resources.graphics.ssgi_intensity = self.ssgi_intensity;
        world.resources.graphics.ssgi_max_steps = self.ssgi_max_steps;

        self.minimap_enabled = true;

        #[cfg(feature = "openxr")]
        {
            world.resources.xr.locomotion_enabled = true;
            world.resources.xr.locomotion_speed = 15.0;
            world.resources.xr.initial_player_position = Some(Vec3::new(0.0, 2.0, 0.0));
        }

        materials::create_materials(world);
        kenney::load_all(world);
        billboard::register_screen_materials(world);

        self.setup_world(world);
    }

    fn pre_render(&mut self, renderer: &mut dyn Render, world: &mut World) {
        if self.observer.is_none() {
            self.observer = Some(observer::ObserverCamera::new(renderer, world));
        }
        if let Some(observer) = &mut self.observer {
            observer.enabled = self.observer_enabled;
            if observer.enabled
                && let Some(main_camera) = self.camera_entity
            {
                observer.render(renderer, world, main_camera);
            }
        }

        self.billboard_textures.initialize(renderer, world);
        self.billboard_textures.register_textures(renderer);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        let delta_time = world.resources.window.timing.delta_time;

        if self.auto_time {
            self.current_hour += self.time_speed * delta_time;
        }

        if self.current_hour >= 24.0 {
            self.current_hour -= 24.0;
        }

        world.resources.graphics.day_night_hour = self.current_hour;
        self.update_sun_for_hour(world);
        self.update_environment_for_hour(world);
        atmosphere::update_window_emissive(world, self.current_hour);

        if self.post_processing {
            world.resources.graphics.bloom_enabled = true;
            world.resources.graphics.ssao_enabled = true;
            world.resources.graphics.depth_of_field = self.depth_of_field;
            world.resources.graphics.ssgi_enabled = self.ssgi_enabled;
            world.resources.graphics.ssgi_intensity = self.ssgi_intensity;
            world.resources.graphics.ssgi_radius = self.ssgi_radius;
            world.resources.graphics.ssgi_max_steps = self.ssgi_max_steps;
        } else {
            world.resources.graphics.bloom_enabled = false;
            world.resources.graphics.ssao_enabled = false;
            world.resources.graphics.depth_of_field.enabled = false;
            world.resources.graphics.ssgi_enabled = false;
        }

        if self.generate_requested {
            self.generate_requested = false;
            self.city_half = self.pending_city_half;
            self.next_seed += 1;

            if self.first_person_mode {
                first_person::exit_first_person(self, world);
            }

            if let Some(mut streamer) = self.chunk_streamer.take() {
                streamer.despawn_all(world);
            }
            if let Some(entity) = self.camera_entity.take() {
                world.queue_command(WorldCommand::DespawnRecursive { entity });
            }
            if let Some(entity) = self.sun_entity.take() {
                world.queue_command(WorldCommand::DespawnRecursive { entity });
            }
            if let Some(entity) = self.ocean_entity.take() {
                world.queue_command(WorldCommand::DespawnRecursive { entity });
            }
            if let Some(observer) = self.observer.take() {
                observer.despawn(world);
            }
            self.leaf_system.despawn(world);
            self.billboard_textures.reset();
            self.camera_controller = None;
            self.district_hud = None;

            self.setup_world(world);
        }

        #[cfg(feature = "openxr")]
        {
            if let Some(camera) = self.camera_entity
                && let Some(xr_input) = &world.resources.xr.input
            {
                let head_pos = xr_input.head_position;
                let head_rot = xr_input.head_orientation;
                if let Some(transform) = world.get_local_transform_mut(camera) {
                    transform.translation = head_pos;
                    transform.rotation = head_rot;
                }
                mark_local_transform_dirty(world, camera);
            }

            let camera_pos = self.active_camera_position(world);
            let camera_forward = self.active_camera_forward(world);
            if let Some(streamer) = &mut self.chunk_streamer {
                streamer.update(world, camera_pos, camera_forward);
            }
        }

        #[cfg(not(feature = "openxr"))]
        if self.first_person_mode {
            run_first_person_systems(self, world);
        } else {
            run_fly_camera_systems(self, world);
        }

        let uptime = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
        let camera_pos = self.active_camera_position(world);

        atmosphere::update_campfire_lights(world, uptime);
        atmosphere::update_neon_lights(world, uptime);
        atmosphere::update_traffic_lights(world, uptime);

        if let Some(streamer) = &self.chunk_streamer {
            streamer.update_boat_bobbing(world, uptime);
        }

        self.leaf_system.initialize(world);
        self.leaf_system.update(world, camera_pos);

        if self.district_hud.is_none() {
            self.district_hud = Some(districts::DistrictHud::new(self.next_seed));
        }
        if let Some(hud) = &mut self.district_hud {
            hud.update(camera_pos, delta_time);
        }

        update_particle_emitters(world, delta_time);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let camera_pos = self.active_camera_position(world);
        let camera_forward = self.active_camera_forward(world);

        if self.first_person_mode {
            player_hud::draw_game_hud(camera_forward, ui_context);
        }

        egui::Window::new("City Info").show(ui_context, |ui| {
            ui.label(format!(
                "Position: ({:.1}, {:.1}, {:.1})",
                camera_pos.x, camera_pos.y, camera_pos.z
            ));

            if let Some(streamer) = &self.chunk_streamer {
                if streamer.is_ready() {
                    ui.label(format!("Chunks: {}", streamer.loaded_chunk_count()));
                    ui.label(format!("Entities: {}", streamer.entity_count()));
                } else {
                    ui.label("Initializing...");
                }
            }

            let fps = world.resources.window.timing.frames_per_second;
            let fps_color = if fps >= 56.0 {
                egui::Color32::GREEN
            } else {
                egui::Color32::from_rgb(255, 165, 0)
            };
            ui.colored_label(fps_color, format!("FPS: {:.0}", fps));

            ui.checkbox(&mut self.sun_shadows, "Sun Shadows");
            ui.checkbox(&mut self.post_processing, "Post Processing");

            ui.separator();

            let mut fp_toggle = self.first_person_mode;
            if ui.checkbox(&mut fp_toggle, "First Person").changed() {
                if fp_toggle && !self.first_person_mode {
                    first_person::enter_first_person(self, world);
                } else if !fp_toggle && self.first_person_mode {
                    first_person::exit_first_person(self, world);
                }
            }

            if self.first_person_mode {
                ui.checkbox(&mut self.show_collision, "Show Collision");
            }

            ui.checkbox(&mut world.resources.graphics.show_bounding_volumes, "Bounding Volumes");

            ui.separator();
            ui.label("Time of Day");
            ui.add(egui::Slider::new(&mut self.current_hour, 0.0..=24.0).text("Hour"));
            ui.checkbox(&mut self.auto_time, "Auto Time");
            if self.auto_time {
                ui.add(egui::Slider::new(&mut self.time_speed, 0.0..=5.0).text("Speed (h/s)"));
            }

            ui.separator();

            ui.checkbox(&mut self.depth_of_field.enabled, "Depth of Field");
            ui.add_enabled_ui(self.depth_of_field.enabled, |ui| {
                let dof = &mut self.depth_of_field;
                ui.add(
                    egui::Slider::new(&mut dof.focus_distance, 0.5..=200.0)
                        .logarithmic(true)
                        .text("Focus Dist"),
                );
                ui.add(
                    egui::Slider::new(&mut dof.focus_range, 0.1..=100.0)
                        .logarithmic(true)
                        .text("Focus Range"),
                );
                ui.add(egui::Slider::new(&mut dof.max_blur_radius, 1.0..=20.0).text("Blur Radius"));

                ui.separator();

                ui.add(
                    egui::Slider::new(&mut dof.bokeh_threshold, 0.0..=1.0).text("Bokeh Threshold"),
                );
                ui.add(
                    egui::Slider::new(&mut dof.bokeh_intensity, 0.0..=3.0).text("Bokeh Intensity"),
                );

                ui.separator();

                egui::ComboBox::from_id_salt("dof_quality")
                    .selected_text(dof.quality.name())
                    .show_ui(ui, |ui| {
                        for quality in DepthOfFieldQuality::ALL {
                            ui.selectable_value(&mut dof.quality, *quality, quality.name());
                        }
                    });
                ui.checkbox(&mut dof.visualize_coc, "Visualize CoC");

                ui.separator();

                ui.checkbox(&mut dof.tilt_shift_enabled, "Tilt Shift");
                ui.add_enabled_ui(dof.tilt_shift_enabled, |ui| {
                    ui.add(
                        egui::Slider::new(&mut dof.tilt_shift_angle, -90.0..=90.0)
                            .suffix("\u{00b0}")
                            .text("Angle"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dof.tilt_shift_center, -1.0..=1.0).text("Center"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dof.tilt_shift_blur_amount, 0.1..=3.0)
                            .text("Blur Amt"),
                    );
                    ui.checkbox(&mut dof.visualize_tilt_shift, "Visualize Focus Band");
                });
            });

            ui.separator();

            ui.checkbox(&mut self.ssgi_enabled, "Global Illumination");
            if self.ssgi_enabled {
                ui.add(egui::Slider::new(&mut self.ssgi_intensity, 0.0..=2.0).text("Intensity"));
                ui.add(egui::Slider::new(&mut self.ssgi_radius, 0.5..=8.0).text("Radius"));
            }

            let mut minimap_on = self.minimap_enabled;
            if ui.checkbox(&mut minimap_on, "Minimap").changed() {
                self.minimap_enabled = minimap_on;
            }

            if !self.first_person_mode {
                ui.checkbox(&mut self.observer_enabled, "Observer Camera");
            }

            ui.separator();

            let total_size = self.pending_city_half * 2;
            let world_extent = total_size as f32 * city::CHUNK_SIZE;
            ui.label(format!("Map: {total_size}\u{00d7}{total_size} chunks ({world_extent}\u{00d7}{world_extent}m)"));
            ui.add(egui::Slider::new(&mut self.pending_city_half, 4..=256));
            if ui.button("Generate City").clicked() {
                self.generate_requested = true;
            }

            if !self.first_person_mode {
                ui.separator();

                let auto_camera_active = self.camera_controller.is_some();
                let mut auto_cam = auto_camera_active;
                if ui.checkbox(&mut auto_cam, "Auto Camera").changed() {
                    if auto_cam {
                        let city_half_extent = self.city_half as f32 * city::CHUNK_SIZE;
                        self.camera_controller = Some(camera::CameraController::new(
                            camera::CinematicMode::Drive,
                            camera_pos,
                            city_half_extent,
                        ));
                    } else {
                        self.camera_controller = None;
                    }
                }

                if let Some(controller) = &mut self.camera_controller {
                    for &mode in camera::CinematicMode::ALL {
                        if ui.radio(controller.mode() == mode, mode.label()).clicked() {
                            controller.set_mode(mode, camera_pos);
                        }
                    }
                }
            }
        });

        if self.minimap_enabled
            && let Some(streamer) = &self.chunk_streamer
            && streamer.is_ready()
        {
            minimap::draw(
                ui_context,
                streamer.layouts(),
                &minimap::MinimapState {
                    camera_x: camera_pos.x,
                    camera_z: camera_pos.z,
                    camera_forward_x: camera_forward.x,
                    camera_forward_z: camera_forward.z,
                    city_min: streamer.city_min(),
                    city_max: streamer.city_max(),
                },
            );
        }

        if !self.first_person_mode
            && self.observer_enabled
            && let Some(observer) = &self.observer
        {
            observer.draw_ui(ui_context, self.minimap_enabled);
        }

        if let Some(hud) = &self.district_hud {
            hud.draw(ui_context);
        }
    }
}

fn run_first_person_systems(demo: &mut CityDemo, world: &mut World) {
    player_systems::detect_input_mode(demo, world);

    run_physics_systems(world);

    player_systems::camera_look_system(demo, world);
    player_systems::lean_system(demo, world);
    player_systems::crouch_camera_system(demo, world);
    player_systems::update_flashlight(demo, world);

    first_person::update_collider_streaming(demo, world);

    if demo.show_collision {
        world.resources.physics.debug_draw = true;
        physics_debug_draw_system(world);
    } else {
        world.resources.physics.debug_draw = false;
    }

    let camera_pos = demo.active_camera_position(world);
    let camera_forward = demo.active_camera_forward(world);

    if let Some(streamer) = &mut demo.chunk_streamer {
        streamer.update(world, camera_pos, camera_forward);
    }
}

fn run_fly_camera_systems(demo: &mut CityDemo, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time;

    if let Some(camera) = demo.camera_entity {
        if let Some(controller) = &mut demo.camera_controller {
            let (position, rotation) = controller.update(delta_time);
            if let Some(transform) = world.get_local_transform_mut(camera) {
                transform.translation = position;
                transform.rotation = rotation;
            }
        } else if demo.observer_enabled {
            observer::fly_camera_keyboard_mouse_only(world);
            if let Some(transform) = world.get_local_transform_mut(camera)
                && transform.translation.y < MIN_CAMERA_Y
            {
                transform.translation.y = MIN_CAMERA_Y;
            }
        } else {
            fly_camera_system(world);
            if let Some(transform) = world.get_local_transform_mut(camera)
                && transform.translation.y < MIN_CAMERA_Y
            {
                transform.translation.y = MIN_CAMERA_Y;
            }
        }
        mark_local_transform_dirty(world, camera);

        if demo.observer_enabled
            && let Some(observer) = &mut demo.observer
        {
            observer.update(world, delta_time);
        }

        let camera_pos = world
            .get_local_transform(camera)
            .map(|t| t.translation)
            .unwrap_or(Vec3::zeros());

        if let Some(streamer) = &mut demo.chunk_streamer {
            let camera_forward = world
                .get_local_transform(camera)
                .map(|t| nalgebra_glm::quat_rotate_vec3(&t.rotation, &Vec3::new(0.0, 0.0, -1.0)))
                .unwrap_or(Vec3::new(0.0, 0.0, -1.0));
            streamer.update(world, camera_pos, camera_forward);
        }
    }
}

impl CityDemo {
    fn active_camera_position(&self, world: &World) -> Vec3 {
        if self.first_person_mode {
            self.player_camera_entity
                .and_then(|entity| world.get_global_transform(entity))
                .map(|t| t.translation())
                .unwrap_or(Vec3::zeros())
        } else {
            self.camera_entity
                .and_then(|entity| world.get_local_transform(entity))
                .map(|t| t.translation)
                .unwrap_or(Vec3::zeros())
        }
    }

    fn active_camera_forward(&self, world: &World) -> Vec3 {
        if self.first_person_mode {
            self.player_camera_entity
                .and_then(|entity| world.get_global_transform(entity))
                .map(|t| t.forward_vector())
                .unwrap_or(Vec3::new(0.0, 0.0, -1.0))
        } else {
            self.camera_entity
                .and_then(|entity| world.get_local_transform(entity))
                .map(|t| nalgebra_glm::quat_rotate_vec3(&t.rotation, &Vec3::new(0.0, 0.0, -1.0)))
                .unwrap_or(Vec3::new(0.0, 0.0, -1.0))
        }
    }

    fn setup_world(&mut self, world: &mut World) {
        let camera = spawn_camera(world, Vec3::new(0.0, 30.0, 0.0), "City Camera".to_string());
        if let Some(camera_component) = world.get_camera_mut(camera) {
            camera_component.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_far: Some(2000.0),
                z_near: 1.0,
            });
        }
        world.resources.active_camera = Some(camera);
        self.camera_entity = Some(camera);

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 3.5;
            light.shadow_bias = 0.008;
        }
        self.sun_entity = Some(sun);
        self.update_sun_for_hour(world);

        let ocean = world.spawn_entities(WATER | NAME, 1)[0];
        world.set_name(ocean, Name("Ocean".to_string()));
        world.set_water(
            ocean,
            Water {
                base_height: -2.0,
                wave_height: 0.3,
                choppy: 2.0,
                speed: 0.5,
                frequency: 0.12,
                ..Default::default()
            },
        );
        self.ocean_entity = Some(ocean);

        let streamer = ChunkStreamer::new(self.city_half, self.next_seed);
        self.chunk_streamer = Some(streamer);
    }

    fn get_sun_direction(hour: f32) -> Vec3 {
        let pi = std::f32::consts::PI;
        if !(6.0..=18.0).contains(&hour) {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            let sun_angle = (hour - 6.0) / 12.0 * pi;
            nalgebra_glm::normalize(&Vec3::new(-sun_angle.cos(), sun_angle.sin(), -0.3))
        }
    }

    fn update_sun_for_hour(&self, world: &mut World) {
        let sun = match self.sun_entity {
            Some(entity) => entity,
            None => return,
        };

        let sun_dir = Self::get_sun_direction(self.current_hour);
        let is_night = !(6.0..=18.0).contains(&self.current_hour);

        let sun_intensity = if is_night {
            0.0
        } else {
            let elevation = sun_dir.y.max(0.0);
            3.5 * elevation.sqrt()
        };

        let warm = Vec3::new(1.0, 0.7, 0.4);
        let white = Vec3::new(1.0, 0.95, 0.8);
        let sun_color = if is_night {
            Vec3::new(0.0, 0.0, 0.0)
        } else if self.current_hour < 7.5 {
            let t = ((self.current_hour - 6.0) / 1.5).clamp(0.0, 1.0);
            nalgebra_glm::lerp(&warm, &white, t)
        } else if self.current_hour > 16.5 {
            let t = ((18.0 - self.current_hour) / 1.5).clamp(0.0, 1.0);
            nalgebra_glm::lerp(&warm, &white, t)
        } else {
            white
        };

        if let Some(light) = world.get_light_mut(sun) {
            light.intensity = sun_intensity;
            light.color = sun_color;
            light.cast_shadows = self.sun_shadows;
        }

        let sun_position = sun_dir * 100.0;
        if let Some(transform) = world.get_local_transform_mut(sun) {
            transform.translation = sun_position;
            let direction = -sun_dir;
            let up = Vec3::y();
            let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &up));
            if right.norm() > 0.001 {
                let corrected_up = nalgebra_glm::cross(&right, &direction);
                transform.rotation =
                    nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[
                        right,
                        corrected_up,
                        -direction,
                    ]));
            }
        }
        mark_local_transform_dirty(world, sun);
    }

    fn update_environment_for_hour(&self, world: &mut World) {
        let hour = self.current_hour;
        let is_night = !(6.0..=18.0).contains(&hour);

        let (ambient_intensity, ambient_color) = if is_night {
            (0.05, [0.05, 0.05, 0.1, 1.0])
        } else if !(7.5..=16.5).contains(&hour) {
            let t = if hour < 7.5 {
                (hour - 6.0) / 1.5
            } else {
                (18.0 - hour) / 1.5
            };
            let intensity = 0.05 + 0.20 * t;
            (
                intensity,
                [0.05 + 0.20 * t, 0.05 + 0.17 * t, 0.10 + 0.10 * t, 1.0],
            )
        } else {
            (0.25, [0.25, 0.22, 0.20, 1.0])
        };
        let _ = ambient_intensity;
        world.resources.graphics.ambient_light = ambient_color;

        let night_fog = [0.02_f32, 0.02, 0.05];
        let dawn_fog = [0.65_f32, 0.45, 0.35];
        let day_fog = [0.7_f32, 0.75, 0.8];
        let (fog_color, fog_density_mult) = if !(5.5..19.5).contains(&hour) {
            (night_fog, 0.5)
        } else if hour < 7.5 {
            let t = ((hour - 5.5) / 2.0).clamp(0.0, 1.0);
            (
                [
                    night_fog[0] + (dawn_fog[0] - night_fog[0]) * t,
                    night_fog[1] + (dawn_fog[1] - night_fog[1]) * t,
                    night_fog[2] + (dawn_fog[2] - night_fog[2]) * t,
                ],
                0.5 + 0.5 * t,
            )
        } else if hour < 8.5 {
            let t = ((hour - 7.5) / 1.0).clamp(0.0, 1.0);
            (
                [
                    dawn_fog[0] + (day_fog[0] - dawn_fog[0]) * t,
                    dawn_fog[1] + (day_fog[1] - dawn_fog[1]) * t,
                    dawn_fog[2] + (day_fog[2] - dawn_fog[2]) * t,
                ],
                1.0 + 0.2 * t,
            )
        } else if hour < 16.0 {
            (day_fog, 1.2)
        } else if hour < 17.0 {
            let t = ((hour - 16.0) / 1.0).clamp(0.0, 1.0);
            (
                [
                    day_fog[0] + (dawn_fog[0] - day_fog[0]) * t,
                    day_fog[1] + (dawn_fog[1] - day_fog[1]) * t,
                    day_fog[2] + (dawn_fog[2] - day_fog[2]) * t,
                ],
                1.2 - 0.2 * t,
            )
        } else if hour < 19.5 {
            let t = ((hour - 17.0) / 2.5).clamp(0.0, 1.0);
            (
                [
                    dawn_fog[0] + (night_fog[0] - dawn_fog[0]) * t,
                    dawn_fog[1] + (night_fog[1] - dawn_fog[1]) * t,
                    dawn_fog[2] + (night_fog[2] - dawn_fog[2]) * t,
                ],
                1.0 - 0.5 * t,
            )
        } else {
            (night_fog, 0.5)
        };

        let camera_y = self.active_camera_position(world).y;

        let height_factor =
            ((camera_y - FOG_HEIGHT_LOW) / (FOG_HEIGHT_HIGH - FOG_HEIGHT_LOW)).clamp(0.0, 1.0);
        let base_start = FOG_STREET_START + (FOG_SKY_START - FOG_STREET_START) * height_factor;
        let base_end = FOG_STREET_END + (FOG_SKY_END - FOG_STREET_END) * height_factor;

        world.resources.graphics.fog = Some(Fog {
            start: base_start * fog_density_mult,
            end: base_end * fog_density_mult,
            color: fog_color,
        });
    }
}
