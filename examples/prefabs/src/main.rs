use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::graphics::resources::PbrDebugMode;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use nightshade::run::RenderResources;

const HDR_BYTES: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(PrefabsState::default())
}

struct PrefabsState {
    model_entities: Vec<Entity>,
    camera_entity: Option<Entity>,
    sun_entity: Option<Entity>,
    rotation_speed: f32,
    loaded: bool,
    left_arrow_was_pressed: bool,
    right_arrow_was_pressed: bool,
    previous_atmosphere: Atmosphere,
    day_night_hour: f32,
    last_ibl_hour: f32,
    a_button_was_pressed: bool,
    b_button_was_pressed: bool,
}

impl Default for PrefabsState {
    fn default() -> Self {
        Self {
            model_entities: Vec::new(),
            camera_entity: None,
            sun_entity: None,
            rotation_speed: 0.0,
            loaded: false,
            left_arrow_was_pressed: false,
            right_arrow_was_pressed: false,
            previous_atmosphere: if cfg!(feature = "openxr") {
                Atmosphere::DayNight
            } else {
                Atmosphere::Hdr
            },
            day_night_hour: 12.0,
            last_ibl_hour: 12.0,
            a_button_was_pressed: false,
            b_button_was_pressed: false,
        }
    }
}

impl State for PrefabsState {
    fn title(&self) -> &str {
        "Prefabs"
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

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.08);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.swapchain);
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        #[cfg(feature = "openxr")]
        {
            world.resources.graphics.atmosphere = Atmosphere::DayNight;
            world.resources.graphics.day_night_hour = self.day_night_hour;
        }
        #[cfg(not(feature = "openxr"))]
        {
            world.resources.graphics.atmosphere = Atmosphere::Hdr;
        }
        world.resources.graphics.use_fullscreen = true;
        world.resources.graphics.ssao_enabled = true;
        world.resources.graphics.ssao_radius = 0.5;
        world.resources.graphics.ssao_bias = 0.025;
        world.resources.graphics.ssao_intensity = 1.5;

        load_hdr_skybox(world, HDR_BYTES.to_vec());

        #[cfg(feature = "openxr")]
        {
            capture_procedural_atmosphere_ibl(world, Atmosphere::DayNight, self.day_night_hour);
            capture_ibl_snapshots(
                world,
                Atmosphere::DayNight,
                vec![0.0, 4.0, 7.0, 10.0, 14.0, 17.0, 20.0],
            );
        }

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 3.5;
            light.shadow_bias = 0.008;
        }
        self.sun_entity = Some(sun);
        self.update_sun_for_hour(world);

        self.rotation_speed = 0.5;

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            5.0,
            0.0,
            0.3,
            "Main Camera".to_string(),
        );

        self.camera_entity = Some(camera_entity);
        world.resources.active_camera = Some(camera_entity);

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
                translation: Vec3::new(0.0, -2.0, 0.0),
                rotation: Quat::identity(),
                scale: Vec3::new(10.0, 0.1, 10.0),
            },
        );
        world.set_render_mesh(ground, RenderMesh::new("Cube"));
        let ground_material = format!("Ground_{}", ground.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            ground_material.clone(),
            Material {
                base_color: [0.5, 0.5, 0.5, 1.0],
                roughness: 0.8,
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

        tracing::info!("Loading embedded GLTF model");
        const GLTF_DATA: &[u8] = include_bytes!("../../../assets/gltf/DamagedHelmet.glb");
        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(GLTF_DATA);

        match load_result {
            Ok(result) => {
                tracing::info!("Successfully loaded GLTF file");
                tracing::info!("Loaded {} meshes", result.meshes.len());
                tracing::info!("Loaded {} materials", result.materials.len());
                tracing::info!("Loaded {} textures", result.textures.len());
                tracing::info!("Loaded {} prefabs", result.prefabs.len());

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
                    tracing::info!(
                        "Mesh '{}': {} vertices, {} indices",
                        name,
                        mesh.vertices.len(),
                        mesh.indices.len()
                    );
                    mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                }

                for prefab in result.prefabs {
                    tracing::info!("Spawning prefab '{}'", prefab.name);
                    let entity = nightshade::ecs::prefab::spawn_prefab(
                        world,
                        &prefab,
                        nalgebra_glm::vec3(0.0, 0.0, 0.0),
                    );

                    self.model_entities.push(entity);
                    tracing::info!("Spawned prefab with root entity {:?}", entity);
                }

                self.loaded = true;
            }
            Err(e) => {
                tracing::error!("Failed to load GLTF file: {}", e);
            }
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);
        self.atmosphere_switch_system(world);

        if world.resources.graphics.atmosphere == Atmosphere::DayNight {
            let delta = world.resources.window.timing.delta_time;
            self.day_night_hour += delta * 0.5;
            if self.day_night_hour >= 24.0 {
                self.day_night_hour -= 24.0;
            }

            let hour_diff = (self.day_night_hour - self.last_ibl_hour).abs();
            if hour_diff > 1.0 || (self.day_night_hour < self.last_ibl_hour) {
                capture_procedural_atmosphere_ibl(world, Atmosphere::DayNight, self.day_night_hour);
                self.last_ibl_hour = self.day_night_hour;
            }
        }

        world.resources.graphics.day_night_hour = self.day_night_hour;
        self.update_sun_for_hour(world);

        if self.loaded {
            for entity in &self.model_entities {
                if let Some(transform) = world.get_local_transform_mut(*entity) {
                    let rotation = nalgebra_glm::quat_angle_axis(
                        self.rotation_speed * 0.016,
                        &nalgebra_glm::vec3(0.0, 1.0, 0.0),
                    );
                    transform.rotation = rotation * transform.rotation;
                }
                world.mark_local_transform_dirty(*entity);
            }
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Settings")
            .default_pos(egui::pos2(10.0, 10.0))
            .default_width(280.0)
            .show(ui_context, |ui| {
                ui.label("Color Grading");

                let color_grading = &mut world.resources.graphics.color_grading;

                ui.horizontal(|ui| {
                    ui.label("Preset:");
                    ui.label(color_grading.preset.name());
                });

                ui.horizontal_wrapped(|ui| {
                    for preset in ColorGradingPreset::ALL {
                        if *preset == ColorGradingPreset::Custom {
                            continue;
                        }
                        let is_selected = color_grading.preset == *preset;
                        if ui.selectable_label(is_selected, preset.name()).clicked() {
                            *color_grading = preset.to_color_grading();
                        }
                    }
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Tonemap:");
                    egui::ComboBox::from_id_salt("tonemap_algorithm")
                        .selected_text(color_grading.tonemap_algorithm.name())
                        .show_ui(ui, |ui| {
                            for algorithm in TonemapAlgorithm::ALL {
                                if ui
                                    .selectable_value(
                                        &mut color_grading.tonemap_algorithm,
                                        *algorithm,
                                        algorithm.name(),
                                    )
                                    .changed()
                                {
                                    color_grading.preset = ColorGradingPreset::Custom;
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Gamma:");
                    if ui
                        .add(
                            egui::Slider::new(&mut color_grading.gamma, 1.0..=3.0)
                                .fixed_decimals(2),
                        )
                        .changed()
                    {
                        color_grading.preset = ColorGradingPreset::Custom;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Saturation:");
                    if ui
                        .add(
                            egui::Slider::new(&mut color_grading.saturation, 0.0..=2.0)
                                .fixed_decimals(2),
                        )
                        .changed()
                    {
                        color_grading.preset = ColorGradingPreset::Custom;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Brightness:");
                    if ui
                        .add(
                            egui::Slider::new(&mut color_grading.brightness, -0.5..=0.5)
                                .fixed_decimals(2),
                        )
                        .changed()
                    {
                        color_grading.preset = ColorGradingPreset::Custom;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Contrast:");
                    if ui
                        .add(
                            egui::Slider::new(&mut color_grading.contrast, 0.5..=2.0)
                                .fixed_decimals(2),
                        )
                        .changed()
                    {
                        color_grading.preset = ColorGradingPreset::Custom;
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Rotation Speed:");
                    ui.add(
                        egui::Slider::new(&mut self.rotation_speed, 0.0..=2.0).fixed_decimals(2),
                    );
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Bloom:");
                    ui.checkbox(&mut world.resources.graphics.bloom_enabled, "Enabled");
                });

                ui.horizontal(|ui| {
                    ui.label("SSAO:");
                    ui.checkbox(&mut world.resources.graphics.ssao_enabled, "Enabled");
                });

                if world.resources.graphics.ssao_enabled {
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssao_radius, 0.1..=2.0)
                            .text("Radius"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssao_bias, 0.001..=0.1)
                            .text("Bias"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssao_intensity, 0.5..=3.0)
                            .text("Intensity"),
                    );
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Atmosphere:");
                    egui::ComboBox::from_id_salt("atmosphere")
                        .selected_text(format!("{:?}", world.resources.graphics.atmosphere))
                        .show_ui(ui, |ui| {
                            for atmosphere in Atmosphere::ALL {
                                ui.selectable_value(
                                    &mut world.resources.graphics.atmosphere,
                                    *atmosphere,
                                    format!("{:?}", atmosphere),
                                );
                            }
                        });
                });

                if world.resources.graphics.atmosphere == Atmosphere::DayNight {
                    ui.horizontal(|ui| {
                        ui.label("Hour:");
                        ui.add(
                            egui::Slider::new(&mut self.day_night_hour, 0.0..=24.0)
                                .fixed_decimals(1),
                        );
                    });
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("PBR Debug:");
                    egui::ComboBox::from_id_salt("pbr_debug")
                        .selected_text(world.resources.graphics.pbr_debug_mode.name())
                        .show_ui(ui, |ui| {
                            for mode in PbrDebugMode::ALL {
                                ui.selectable_value(
                                    &mut world.resources.graphics.pbr_debug_mode,
                                    *mode,
                                    mode.name(),
                                );
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Texture Stripes:");
                    ui.checkbox(
                        &mut world.resources.graphics.texture_debug_stripes,
                        "Show texture map stripes",
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Stripe Speed:");
                    ui.add(
                        egui::Slider::new(
                            &mut world.resources.graphics.texture_debug_stripes_speed,
                            0.0..=500.0,
                        )
                        .suffix(" px/s"),
                    );
                });
            });
    }
}

impl PrefabsState {
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

        let sun_dir = Self::get_sun_direction(self.day_night_hour);
        let is_night = !(6.0..=18.0).contains(&self.day_night_hour);

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
        } else if self.day_night_hour < 7.5 {
            let t = ((self.day_night_hour - 6.0) / 1.5).clamp(0.0, 1.0);
            nalgebra_glm::lerp(&warm, &white, t)
        } else if self.day_night_hour > 16.5 {
            let t = ((18.0 - self.day_night_hour) / 1.5).clamp(0.0, 1.0);
            nalgebra_glm::lerp(&warm, &white, t)
        } else {
            white
        };

        if let Some(light) = world.get_light_mut(sun) {
            light.intensity = sun_intensity;
            light.color = sun_color;
        }

        let sun_position = sun_dir * 100.0;
        if let Some(transform) = world.get_local_transform_mut(sun) {
            transform.translation = sun_position;
            let forward = -sun_dir;
            let up = Vec3::y();
            let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&forward, &up));
            if right.norm() > 0.001 {
                let corrected_up = nalgebra_glm::normalize(&nalgebra_glm::cross(&right, &forward));
                transform.rotation =
                    nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[
                        right,
                        corrected_up,
                        -forward,
                    ]));
            }
        }
        mark_local_transform_dirty(world, sun);
    }

    fn atmosphere_switch_system(&mut self, world: &mut World) {
        let right_pressed = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::ArrowRight);
        let left_pressed = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::ArrowLeft);

        #[cfg(feature = "openxr")]
        let (a_pressed, b_pressed) = if let Some(ref xr_input) = world.resources.xr.input {
            (xr_input.a_button_pressed(), xr_input.b_button_pressed())
        } else {
            (false, false)
        };
        #[cfg(not(feature = "openxr"))]
        let (a_pressed, b_pressed) = (false, false);

        if (right_pressed && !self.right_arrow_was_pressed)
            || (a_pressed && !self.a_button_was_pressed)
        {
            world.resources.graphics.atmosphere = world.resources.graphics.atmosphere.next();
        }
        if (left_pressed && !self.left_arrow_was_pressed)
            || (b_pressed && !self.b_button_was_pressed)
        {
            world.resources.graphics.atmosphere = world.resources.graphics.atmosphere.previous();
        }

        self.right_arrow_was_pressed = right_pressed;
        self.left_arrow_was_pressed = left_pressed;
        self.a_button_was_pressed = a_pressed;
        self.b_button_was_pressed = b_pressed;

        let current_atmosphere = world.resources.graphics.atmosphere;
        if current_atmosphere != self.previous_atmosphere {
            if current_atmosphere == Atmosphere::DayNight {
                self.day_night_hour = 12.0;
                self.last_ibl_hour = 12.0;
                world.resources.graphics.day_night_hour = self.day_night_hour;
                capture_procedural_atmosphere_ibl(world, Atmosphere::DayNight, self.day_night_hour);
                capture_ibl_snapshots(
                    world,
                    Atmosphere::DayNight,
                    vec![0.0, 4.0, 7.0, 10.0, 14.0, 17.0, 20.0],
                );
            } else if current_atmosphere.is_procedural() {
                capture_procedural_atmosphere_ibl(world, current_atmosphere, 0.0);
            } else if current_atmosphere == Atmosphere::Hdr {
                load_hdr_skybox(world, HDR_BYTES.to_vec());
            }
            self.previous_atmosphere = current_atmosphere;
        }
    }
}
