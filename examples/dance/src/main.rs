use nightshade::ecs::animation::components::AnimationClip;
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::prefab::{GltfSkin, Prefab};
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use nightshade::run::RenderResources;

const DANCE_MODEL: &[u8] = include_bytes!("../../../assets/models/dance.glb");
const HDR_BYTES: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");
const DEFAULT_GRID_SPACING: f32 = 2.0;
const INITIAL_DANCERS: usize = 0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(DanceState::default())
}

#[derive(Default)]
struct DanceState {
    dancer_entities: Vec<Entity>,
    grid_spacing: f32,
    camera_entity: Option<Entity>,
    loaded: bool,
    home_focus: Vec3,
    home_radius: f32,
    home_yaw: f32,
    home_pitch: f32,
    prefab: Option<Prefab>,
    animations: Vec<AnimationClip>,
    skins: Vec<GltfSkin>,
    fps_hud_text: Option<Entity>,
    dancer_count_hud_text: Option<Entity>,
    target_fps_hud_text: Option<Entity>,
    lowest_fps: f32,
    highest_fps: f32,
    frame_times: Vec<f32>,
    frame_time_index: usize,
    auto_spawn_stopped: bool,
    sustained_low_fps: f32,
    sustained_high_fps: f32,
    sustained_low_count: usize,
    sustained_high_count: usize,
    target_fps: f32,
    pending_target_fps: f32,
    frames_since_last_change: usize,
    frames_below_threshold: usize,
    frames_above_threshold: usize,
}

fn calculate_grid_positions(count: usize, spacing: f32) -> Vec<Vec3> {
    if count == 0 {
        return Vec::new();
    }

    let cols = (count as f32).sqrt().ceil() as usize;
    let rows = count.div_ceil(cols);

    let total_width = (cols.saturating_sub(1)) as f32 * spacing;
    let total_depth = (rows.saturating_sub(1)) as f32 * spacing;

    let start_x = -total_width / 2.0;
    let start_z = -total_depth / 2.0;

    (0..count)
        .map(|index| {
            let col = index % cols;
            let row = index / cols;
            Vec3::new(
                start_x + col as f32 * spacing,
                0.0,
                start_z + row as f32 * spacing,
            )
        })
        .collect()
}

impl State for DanceState {
    fn title(&self) -> &str {
        "Skinned Mesh Benchmark"
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

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.08);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .write("output", resources.swapchain);
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.use_fullscreen = true;
        world.resources.graphics.ui_scale = Some(1.0);
        world.resources.graphics.atmosphere = Atmosphere::Hdr;

        self.grid_spacing = DEFAULT_GRID_SPACING;
        self.lowest_fps = 60.0;
        self.highest_fps = 60.0;
        self.frame_times = vec![0.0; 60];
        self.frame_time_index = 0;
        self.auto_spawn_stopped = false;
        self.sustained_low_fps = 60.0;
        self.sustained_high_fps = 60.0;
        self.sustained_low_count = 0;
        self.sustained_high_count = 0;
        self.target_fps = 30.0;
        self.pending_target_fps = 30.0;

        load_hdr_skybox(world, HDR_BYTES.to_vec());

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 2.0;
        }

        let ground = world.spawn_entities(
            LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | CASTS_SHADOW,
            1,
        )[0];
        world.set_local_transform(
            ground,
            LocalTransform {
                translation: Vec3::new(0.0, -0.05, 0.0),
                rotation: Quat::identity(),
                scale: Vec3::new(100.0, 0.1, 100.0),
            },
        );
        world.set_render_mesh(ground, RenderMesh::new("Cube"));
        let ground_material = format!("Ground_{}", ground.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            ground_material.clone(),
            Material {
                base_color: [0.3, 0.3, 0.35, 1.0],
                roughness: 0.9,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&ground_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(ground, MaterialRef::new(ground_material));
        world.set_casts_shadow(ground, CastsShadow);

        self.home_focus = Vec3::new(0.0, 1.0, 0.0);
        self.home_radius = 8.0;
        self.home_yaw = 0.0;
        self.home_pitch = 0.3;

        let camera_entity = spawn_pan_orbit_camera(
            world,
            self.home_focus,
            self.home_radius,
            self.home_yaw,
            self.home_pitch,
            "Dance Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);
        self.camera_entity = Some(camera_entity);

        if let Some(pan_orbit) = world.get_pan_orbit_camera_mut(camera_entity) {
            pan_orbit.zoom_lower_limit = 2.0;
            pan_orbit.zoom_upper_limit = Some(200.0);
            pan_orbit.pitch_lower_limit = -0.5;
            pan_orbit.pitch_upper_limit = std::f32::consts::FRAC_PI_2 - 0.1;
        }

        let fps_text = spawn_hud_text_with_properties(
            world,
            "FPS: 0",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 10.0),
            TextProperties {
                font_size: 48.0,
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        self.fps_hud_text = Some(fps_text);

        let dancer_count_text = spawn_hud_text_with_properties(
            world,
            "Dancers: 0",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 70.0),
            TextProperties {
                font_size: 32.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                ..Default::default()
            },
        );
        self.dancer_count_hud_text = Some(dancer_count_text);

        let target_fps_text = spawn_hud_text_with_properties(
            world,
            "Target FPS: 30",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 115.0),
            TextProperties {
                font_size: 28.0,
                color: Vec4::new(0.8, 0.8, 0.8, 1.0),
                ..Default::default()
            },
        );
        self.target_fps_hud_text = Some(target_fps_text);

        tracing::info!("Loading dance model");
        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(DANCE_MODEL);

        match load_result {
            Ok(result) => {
                tracing::info!("Successfully loaded dance model");
                tracing::info!("Loaded {} meshes", result.meshes.len());
                tracing::info!("Loaded {} materials", result.materials.len());
                tracing::info!("Loaded {} textures", result.textures.len());
                tracing::info!("Loaded {} prefabs", result.prefabs.len());
                tracing::info!("Loaded {} animations", result.animations.len());

                for (name, (rgba_data, width, height)) in result.textures {
                    tracing::info!("Loading texture '{}': {}x{}", name, width, height);
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

                self.animations = result.animations.clone();
                self.skins = result.skins.clone();

                if let Some(prefab) = result.prefabs.into_iter().next() {
                    self.prefab = Some(prefab.clone());
                    self.spawn_dancers(world, INITIAL_DANCERS);
                    tracing::info!("Spawned {} dancer(s)", self.dancer_entities.len());
                }

                self.loaded = true;
            }
            Err(e) => {
                tracing::error!("Failed to load dance model: {}", e);
            }
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        if world.resources.input.keyboard.is_key_pressed(KeyCode::KeyC)
            || world.resources.input.keyboard.is_key_pressed(KeyCode::Home)
        {
            self.reset_camera_to_home(world);
        }

        let fps = world.resources.window.timing.frames_per_second;
        let target_fps = self.target_fps;
        let lower_threshold = target_fps - 4.0;
        let upper_threshold = target_fps + 4.0;

        if let Some(fps_text_entity) = self.fps_hud_text {
            let fps_color = if fps >= lower_threshold && fps <= upper_threshold {
                Vec4::new(0.0, 1.0, 0.0, 1.0)
            } else if fps > upper_threshold {
                Vec4::new(1.0, 1.0, 1.0, 1.0)
            } else {
                Vec4::new(1.0, 0.65, 0.0, 1.0)
            };

            let text_index = world.get_hud_text(fps_text_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("FPS: {:.0}", fps));
                if let Some(hud_text) = world.get_hud_text_mut(fps_text_entity) {
                    hud_text.properties.color = fps_color;
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(dancer_count_entity) = self.dancer_count_hud_text {
            let text_index = world
                .get_hud_text(dancer_count_entity)
                .map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world.resources.text_cache.set_text(
                    text_index,
                    format!(
                        "Dancers: {}",
                        format_number_with_commas(self.dancer_entities.len())
                    ),
                );
                if let Some(hud_text) = world.get_hud_text_mut(dancer_count_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(target_fps_entity) = self.target_fps_hud_text {
            let text_index = world.get_hud_text(target_fps_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("Target FPS: {:.0}", self.target_fps));
                if let Some(hud_text) = world.get_hud_text_mut(target_fps_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        self.update_sustained_fps_tracking(world);
        self.auto_spawn_system(world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let dancer_count = self.dancer_entities.len();
        let fps = world.resources.window.timing.frames_per_second;

        let avg_frame_time = if !self.frame_times.is_empty() {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        } else {
            0.0
        };

        let resolution = if let Some(window_handle) = &world.resources.window.handle {
            let size = window_handle.inner_size();
            format!("{}x{}", size.width, size.height)
        } else {
            "Unknown".to_string()
        };

        egui::Window::new("Skinned Mesh Benchmark")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.heading("GPU Skinning Benchmark");
                ui.separator();

                ui.label(format!("Resolution: {}", resolution));
                ui.label(format!(
                    "Dancers: {}",
                    format_number_with_commas(dancer_count)
                ));
                ui.label(format!(
                    "FPS: {:.0} (Low: {:.0} High: {:.0})",
                    fps, self.lowest_fps, self.highest_fps
                ));
                ui.label(format!("Frame Time: {:.1}ms", avg_frame_time));

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Target FPS:");
                    ui.add(
                        egui::Slider::new(&mut self.pending_target_fps, 30.0..=144.0).step_by(1.0),
                    );

                    let apply_enabled = (self.pending_target_fps - self.target_fps).abs() > 0.1;
                    if ui
                        .add_enabled(apply_enabled, egui::Button::new("Apply"))
                        .clicked()
                    {
                        self.target_fps = self.pending_target_fps;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Set FPS:");
                    if ui.button("30").clicked() {
                        self.target_fps = 30.0;
                        self.pending_target_fps = 30.0;
                    }
                    if ui.button("60").clicked() {
                        self.target_fps = 60.0;
                        self.pending_target_fps = 60.0;
                    }
                    if ui.button("75").clicked() {
                        self.target_fps = 75.0;
                        self.pending_target_fps = 75.0;
                    }
                    if ui.button("90").clicked() {
                        self.target_fps = 90.0;
                        self.pending_target_fps = 90.0;
                    }
                    if ui.button("120").clicked() {
                        self.target_fps = 120.0;
                        self.pending_target_fps = 120.0;
                    }
                    if ui.button("144").clicked() {
                        self.target_fps = 144.0;
                        self.pending_target_fps = 144.0;
                    }
                });

                ui.separator();

                const FRAMES_REQUIRED_BELOW: usize = 45;
                const FRAMES_REQUIRED_ABOVE: usize = 60;

                if !self.auto_spawn_stopped && dancer_count >= 10 {
                    if self.frames_below_threshold > 0 {
                        let progress =
                            self.frames_below_threshold as f32 / FRAMES_REQUIRED_BELOW as f32;
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::YELLOW, "Despawning");
                            ui.add(egui::ProgressBar::new(progress.min(1.0)).desired_width(80.0));
                        });
                    } else if self.frames_above_threshold > 0 {
                        let progress =
                            self.frames_above_threshold as f32 / FRAMES_REQUIRED_ABOVE as f32;
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Spawning");
                            ui.add(egui::ProgressBar::new(progress.min(1.0)).desired_width(80.0));
                        });
                    } else {
                        ui.colored_label(egui::Color32::GREEN, "Stable");
                    }
                } else if dancer_count < 10 {
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Spawning dancers");
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Grid Spacing:");
                    if ui
                        .add(egui::Slider::new(&mut self.grid_spacing, 1.0..=5.0).suffix("m"))
                        .changed()
                    {
                        self.reposition_dancers(world);
                    }
                });

                ui.separator();
                ui.collapsing("Animation Controls", |ui| {
                    let first_entity = self.dancer_entities.first().copied();
                    let Some(entity) = first_entity else {
                        ui.label("No dancers spawned");
                        return;
                    };

                    let Some(player) = world.get_animation_player_mut(entity) else {
                        ui.label("No animation player found");
                        return;
                    };

                    if player.clips.is_empty() {
                        ui.label("No animation clips available");
                        return;
                    }

                    let mut clip_to_play = None;
                    let current_clip = player.current_clip;
                    let clips_clone: Vec<_> = player
                        .clips
                        .iter()
                        .map(|c| (c.name.clone(), c.duration))
                        .collect();

                    ui.horizontal(|ui| {
                        ui.label("Animation:");
                        egui::ComboBox::from_id_salt("dance_animation_selector")
                            .width(150.0)
                            .selected_text(
                                current_clip
                                    .and_then(|index| clips_clone.get(index))
                                    .map(|(name, _)| name.as_str())
                                    .unwrap_or("None"),
                            )
                            .show_ui(ui, |ui| {
                                for (index, (name, duration)) in clips_clone.iter().enumerate() {
                                    let is_selected = current_clip == Some(index);
                                    let label = format!("{} ({:.2}s)", name, duration);
                                    if ui.selectable_label(is_selected, label).clicked() {
                                        clip_to_play = Some(index);
                                    }
                                }
                            });
                    });

                    if let Some(index) = clip_to_play {
                        self.play_animation_all(world, index);
                    }

                    let (mut speed, mut looping, playing) =
                        if let Some(player) = world.get_animation_player(entity) {
                            (player.speed, player.looping, player.playing)
                        } else {
                            (1.0, true, false)
                        };

                    let old_speed = speed;
                    let old_looping = looping;

                    ui.horizontal(|ui| {
                        ui.label("Speed:");
                        ui.add(egui::Slider::new(&mut speed, 0.0..=3.0).suffix("x"));
                    });

                    ui.checkbox(&mut looping, "Loop");

                    if (speed - old_speed).abs() > f32::EPSILON {
                        self.set_speed_all(world, speed);
                    }

                    if looping != old_looping {
                        self.set_looping_all(world, looping);
                    }

                    ui.horizontal(|ui| {
                        if ui.button(if playing { "Pause" } else { "Play" }).clicked() {
                            if playing {
                                self.pause_all(world);
                            } else {
                                self.resume_all(world);
                            }
                        }
                        if ui.button("Stop").clicked() {
                            self.stop_all(world);
                        }
                    });
                });

                ui.collapsing("Visual Effects", |ui| {
                    ui.add_space(4.0);
                    ui.label("Vertex Snapping");
                    let mut snap_enabled = world.resources.graphics.vertex_snap.is_some();
                    if ui
                        .checkbox(&mut snap_enabled, "Enable Vertex Snapping")
                        .changed()
                    {
                        if snap_enabled {
                            world.resources.graphics.vertex_snap = Some(VertexSnap::default());
                        } else {
                            world.resources.graphics.vertex_snap = None;
                        }
                    }
                    if let Some(ref mut vertex_snap) = world.resources.graphics.vertex_snap {
                        ui.horizontal(|ui| {
                            ui.label("Resolution X:");
                            ui.add(egui::Slider::new(
                                &mut vertex_snap.resolution[0],
                                80.0..=640.0,
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Resolution Y:");
                            ui.add(egui::Slider::new(
                                &mut vertex_snap.resolution[1],
                                60.0..=480.0,
                            ));
                        });
                    }

                    ui.add_space(8.0);
                    ui.label("Texture Mapping");
                    ui.checkbox(
                        &mut world.resources.graphics.affine_texture_mapping,
                        "Enable Affine Texture Mapping",
                    );

                    ui.add_space(8.0);
                    ui.label("Distance Fog");
                    let mut fog_enabled = world.resources.graphics.fog.is_some();
                    if ui.checkbox(&mut fog_enabled, "Enable Fog").changed() {
                        if fog_enabled {
                            world.resources.graphics.fog = Some(Fog::default());
                        } else {
                            world.resources.graphics.fog = None;
                        }
                    }
                    if let Some(ref mut fog) = world.resources.graphics.fog {
                        ui.horizontal(|ui| {
                            ui.label("Start Distance:");
                            ui.add(egui::Slider::new(&mut fog.start, 0.5..=10.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("End Distance:");
                            ui.add(egui::Slider::new(&mut fog.end, 5.0..=50.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Fog Color:");
                            ui.color_edit_button_rgb(&mut fog.color);
                        });
                    }
                });

                ui.separator();
                ui.label("Controls:");
                ui.label("  Mouse drag - Orbit camera");
                ui.label("  Scroll - Zoom");
                ui.label("  C / Home - Reset camera");
                ui.label("  Escape - Exit");
            });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        if matches!(key, KeyCode::KeyC | KeyCode::Home) {
            self.reset_camera_to_home(world);
        }
    }
}

impl DanceState {
    fn reset_camera_to_home(&self, world: &mut World) {
        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let Some(pan_orbit) = world.get_pan_orbit_camera_mut(camera_entity) else {
            return;
        };

        pan_orbit.target_focus = self.home_focus;
        pan_orbit.target_radius = self.home_radius;
        pan_orbit.target_yaw = self.home_yaw;
        pan_orbit.target_pitch = self.home_pitch;
    }

    fn update_sustained_fps_tracking(&mut self, world: &World) {
        if self.frame_times.is_empty() {
            return;
        }

        let fps = world.resources.window.timing.frames_per_second;
        let frame_time = world.resources.window.timing.raw_delta_time * 1000.0;
        let dancer_count = self.dancer_entities.len();

        if dancer_count > 10 {
            if fps < self.sustained_low_fps {
                self.sustained_low_count += 1;
                if self.sustained_low_count >= 20 {
                    self.lowest_fps = fps.min(self.lowest_fps);
                    self.sustained_low_fps = fps;
                }
            } else {
                self.sustained_low_count = 0;
                self.sustained_low_fps = fps;
            }

            if fps > self.sustained_high_fps {
                self.sustained_high_count += 1;
                if self.sustained_high_count >= 20 {
                    self.highest_fps = fps.max(self.highest_fps);
                    self.sustained_high_fps = fps;
                }
            } else {
                self.sustained_high_count = 0;
                self.sustained_high_fps = fps;
            }
        }

        self.frame_times[self.frame_time_index] = frame_time;
        self.frame_time_index = (self.frame_time_index + 1) % self.frame_times.len();
    }

    fn auto_spawn_system(&mut self, world: &mut World) {
        if self.auto_spawn_stopped || self.prefab.is_none() {
            return;
        }

        self.frames_since_last_change += 1;

        let current_count = self.dancer_entities.len();

        if current_count < 10 {
            self.spawn_dancers(world, 10);
            self.frames_since_last_change = 0;
            self.frames_below_threshold = 0;
            self.frames_above_threshold = 0;
            return;
        }

        if self.frame_times.is_empty() {
            return;
        }

        let avg_frame_time: f32 =
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;

        if avg_frame_time < 0.001 {
            return;
        }

        let avg_fps = 1000.0 / avg_frame_time;
        let target_fps = self.target_fps;
        let lower_threshold = target_fps - 4.0;
        let upper_threshold = target_fps + 4.0;

        const MIN_FRAMES_BETWEEN_CHANGES: usize = 30;
        const FRAMES_REQUIRED_BELOW: usize = 45;
        const FRAMES_REQUIRED_ABOVE: usize = 60;

        if avg_fps < lower_threshold {
            self.frames_below_threshold += 1;
            self.frames_above_threshold = 0;
        } else if avg_fps > upper_threshold {
            self.frames_above_threshold += 1;
            self.frames_below_threshold = 0;
        } else {
            self.frames_below_threshold = 0;
            self.frames_above_threshold = 0;
        }

        if self.frames_since_last_change < MIN_FRAMES_BETWEEN_CHANGES {
            return;
        }

        if self.frames_below_threshold >= FRAMES_REQUIRED_BELOW {
            let fps_deficit = lower_threshold - avg_fps;

            let despawn_percentage = if fps_deficit > 15.0 {
                0.10
            } else if fps_deficit > 10.0 {
                0.05
            } else if fps_deficit > 5.0 {
                0.02
            } else if fps_deficit > 2.0 {
                0.01
            } else {
                0.005
            };

            let min_despawn = if fps_deficit > 10.0 {
                5
            } else if fps_deficit > 5.0 {
                3
            } else if fps_deficit > 2.0 {
                2
            } else {
                1
            };

            let despawn_count =
                ((current_count as f32 * despawn_percentage).max(min_despawn as f32)) as usize;
            let despawn_count = despawn_count.min(current_count.saturating_sub(1));
            self.despawn_dancers(world, despawn_count);
            self.frames_since_last_change = 0;
            self.frames_below_threshold = 0;
        } else if self.frames_above_threshold >= FRAMES_REQUIRED_ABOVE {
            let fps_surplus = avg_fps - upper_threshold;

            let spawn_count = if fps_surplus > 30.0 {
                20
            } else if fps_surplus > 20.0 {
                10
            } else if fps_surplus > 10.0 {
                5
            } else if fps_surplus > 5.0 {
                3
            } else if fps_surplus > 2.0 {
                2
            } else {
                1
            };

            self.spawn_dancers(world, spawn_count);
            self.frames_since_last_change = 0;
            self.frames_above_threshold = 0;
        }
    }

    fn spawn_dancers(&mut self, world: &mut World, count: usize) {
        let Some(prefab) = &self.prefab.clone() else {
            return;
        };

        let new_total = self.dancer_entities.len() + count;
        let positions = calculate_grid_positions(new_total, self.grid_spacing);

        for position in positions.iter().skip(self.dancer_entities.len()) {
            let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                world,
                prefab,
                &self.animations,
                &self.skins,
                *position,
            );
            self.dancer_entities.push(entity);
        }

        self.reposition_dancers(world);
    }

    fn despawn_dancers(&mut self, world: &mut World, count: usize) {
        let to_remove = count.min(self.dancer_entities.len().saturating_sub(1));

        for _ in 0..to_remove {
            if let Some(entity) = self.dancer_entities.pop() {
                world.queue_command(WorldCommand::DespawnRecursive { entity });
            }
        }

        self.reposition_dancers(world);
    }

    fn reposition_dancers(&mut self, world: &mut World) {
        let positions = calculate_grid_positions(self.dancer_entities.len(), self.grid_spacing);

        for (entity, position) in self.dancer_entities.iter().zip(positions.iter()) {
            if let Some(transform) = world.get_local_transform_mut(*entity) {
                transform.translation = *position;
            }
            world.set_local_transform_dirty(*entity, LocalTransformDirty);
        }
    }

    fn play_animation_all(&self, world: &mut World, clip_index: usize) {
        for &entity in &self.dancer_entities {
            if let Some(player) = world.get_animation_player_mut(entity) {
                player.play(clip_index);
            }
        }
    }

    fn set_speed_all(&self, world: &mut World, speed: f32) {
        for &entity in &self.dancer_entities {
            if let Some(player) = world.get_animation_player_mut(entity) {
                player.speed = speed;
            }
        }
    }

    fn set_looping_all(&self, world: &mut World, looping: bool) {
        for &entity in &self.dancer_entities {
            if let Some(player) = world.get_animation_player_mut(entity) {
                player.looping = looping;
            }
        }
    }

    fn pause_all(&self, world: &mut World) {
        for &entity in &self.dancer_entities {
            if let Some(player) = world.get_animation_player_mut(entity) {
                player.pause();
            }
        }
    }

    fn resume_all(&self, world: &mut World) {
        for &entity in &self.dancer_entities {
            if let Some(player) = world.get_animation_player_mut(entity) {
                player.resume();
            }
        }
    }

    fn stop_all(&self, world: &mut World) {
        for &entity in &self.dancer_entities {
            if let Some(player) = world.get_animation_player_mut(entity) {
                player.stop();
            }
        }
    }
}

fn format_number_with_commas(number: usize) -> String {
    let number_str = number.to_string();
    let mut result = String::new();
    let chars: Vec<char> = number_str.chars().collect();

    for (index, character) in chars.iter().enumerate() {
        if index > 0 && (chars.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*character);
    }

    result
}
