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
    rotation_speed: f32,
    loaded: bool,
    left_arrow_was_pressed: bool,
    right_arrow_was_pressed: bool,
    previous_atmosphere: Atmosphere,
}

impl Default for PrefabsState {
    fn default() -> Self {
        Self {
            model_entities: Vec::new(),
            camera_entity: None,
            rotation_speed: 0.0,
            loaded: false,
            left_arrow_was_pressed: false,
            right_arrow_was_pressed: false,
            previous_atmosphere: Atmosphere::Hdr,
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
        world.resources.graphics.atmosphere = Atmosphere::Hdr;
        world.resources.graphics.use_fullscreen = true;
        world.resources.graphics.ssao_enabled = true;
        world.resources.graphics.ssao_radius = 0.5;
        world.resources.graphics.ssao_bias = 0.025;
        world.resources.graphics.ssao_intensity = 1.5;

        load_hdr_skybox(world, HDR_BYTES.to_vec());

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
        }

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

        if right_pressed && !self.right_arrow_was_pressed {
            world.resources.graphics.atmosphere = world.resources.graphics.atmosphere.next();
        }
        if left_pressed && !self.left_arrow_was_pressed {
            world.resources.graphics.atmosphere = world.resources.graphics.atmosphere.previous();
        }

        self.right_arrow_was_pressed = right_pressed;
        self.left_arrow_was_pressed = left_pressed;

        let current_atmosphere = world.resources.graphics.atmosphere;
        if current_atmosphere != self.previous_atmosphere {
            if current_atmosphere.is_procedural() {
                capture_procedural_atmosphere_ibl(world, current_atmosphere, 0.0);
            }
            self.previous_atmosphere = current_atmosphere;
        }
    }
}
