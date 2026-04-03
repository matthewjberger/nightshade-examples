use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::queries::query_camera_matrices;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::graphics::resources::PbrDebugMode;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::{PassExecutionContext, PassNode, RenderGraph};
use nightshade::run::RenderResources;
use std::sync::{Arc, RwLock};

const HDR_BYTES: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

const LENS_FLARE_SHADER: &str = r#"
struct Uniforms {
    sun_screen_x: f32,
    sun_screen_y: f32,
    intensity: f32,
    _padding: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

fn lens_flare_effect(uv: vec2<f32>, light_pos: vec2<f32>, intensity: f32) -> vec3<f32> {
    let to_light = light_pos - uv;
    let dist = length(to_light);

    let bright_core = exp(-dist * 40.0) * 0.6;
    let core_color = vec3<f32>(1.0, 0.97, 0.9) * bright_core;

    let inner_glow = exp(-dist * 8.0) * 0.25;
    let glow_color = vec3<f32>(1.0, 0.9, 0.65) * inner_glow;

    let outer_glow = exp(-dist * 2.5) * 0.08;
    let outer_color = vec3<f32>(1.0, 0.8, 0.4) * outer_glow;

    let center = vec2<f32>(0.5, 0.5);
    let light_to_center = center - light_pos;

    let ghost1_pos = center + light_to_center * 0.35;
    let ghost1_dist = length(ghost1_pos - uv);
    let ghost1 = exp(-ghost1_dist * 12.0) * 0.12;
    let ghost1_color = vec3<f32>(0.4, 0.6, 1.0) * ghost1;

    let ghost2_pos = center + light_to_center * 0.7;
    let ghost2_dist = length(ghost2_pos - uv);
    let ghost2 = exp(-ghost2_dist * 10.0) * 0.08;
    let ghost2_color = vec3<f32>(0.9, 0.5, 0.8) * ghost2;

    let ghost3_pos = center + light_to_center * 1.2;
    let ghost3_dist = length(ghost3_pos - uv);
    let ghost3 = exp(-ghost3_dist * 14.0) * 0.06;
    let ghost3_color = vec3<f32>(0.4, 0.9, 0.6) * ghost3;

    let ghost4_pos = center + light_to_center * 1.6;
    let ghost4_dist = length(ghost4_pos - uv);
    let ghost4 = exp(-ghost4_dist * 16.0) * 0.04;
    let ghost4_color = vec3<f32>(0.7, 0.5, 1.0) * ghost4;

    let streak_y = exp(-abs(uv.y - light_pos.y) * 30.0);
    let streak_x = exp(-abs(uv.x - light_pos.x) * 0.5);
    let streak = streak_y * streak_x * 0.15;
    let streak_color = vec3<f32>(0.85, 0.9, 1.0) * streak;

    let halo_dist = abs(dist - 0.15);
    let halo = exp(-halo_dist * 20.0) * 0.05;
    let halo_color = vec3<f32>(0.9, 0.85, 1.0) * halo;

    let starburst_angle = atan2(to_light.y, to_light.x);
    let starburst = exp(-dist * 5.0) * pow(abs(sin(starburst_angle * 3.0)), 12.0) * 0.1;
    let starburst_color = vec3<f32>(1.0, 0.95, 0.8) * starburst;

    return (core_color + glow_color + outer_color + ghost1_color + ghost2_color + ghost3_color + ghost4_color + streak_color + halo_color + starburst_color) * intensity;
}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    if uniforms.intensity <= 0.0 {
        return color;
    }
    let sun_pos = vec2<f32>(uniforms.sun_screen_x, uniforms.sun_screen_y);
    let flare = lens_flare_effect(in.uv, sun_pos, uniforms.intensity);
    return vec4<f32>(color.rgb + flare, color.a);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LensFlareUniforms {
    sun_screen_x: f32,
    sun_screen_y: f32,
    intensity: f32,
    _padding: f32,
}

type LensFlareStateHandle = Arc<RwLock<LensFlareUniforms>>;

struct LensFlarePass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    cached_bind_group: Option<wgpu::BindGroup>,
    state: LensFlareStateHandle,
}

impl LensFlarePass {
    fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        state: LensFlareStateHandle,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lens Flare Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(LENS_FLARE_SHADER)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lens Flare Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Lens Flare Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Lens Flare Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Lens Flare Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lens Flare Uniform Buffer"),
            size: std::mem::size_of::<LensFlareUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            uniform_buffer,
            cached_bind_group: None,
            state,
        }
    }
}

impl PassNode<World> for LensFlarePass {
    fn name(&self) -> &str {
        "lens_flare_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["input"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["output"]
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, _world: &World) {
        if let Ok(uniforms) = self.state.read() {
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&*uniforms));
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, World>,
    ) -> nightshade::render::wgpu::rendergraph::Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
    > {
        let input_view = context.get_texture_view("input")?;

        if self.cached_bind_group.is_none() {
            self.cached_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Lens Flare Bind Group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(input_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.uniform_buffer.as_entire_binding(),
                        },
                    ],
                },
            ));
        }

        let (color_view, color_load_op, color_store_op) = context.get_color_attachment("output")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Lens Flare Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_load_op,
                        store: color_store_op,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.cached_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}

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
    lens_flare_state: LensFlareStateHandle,
    lens_flare_enabled: bool,
    lens_flare_intensity: f32,
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
            lens_flare_state: Arc::new(RwLock::new(LensFlareUniforms {
                sun_screen_x: 0.5,
                sun_screen_y: 0.5,
                intensity: 0.0,
                _padding: 0.0,
            })),
            lens_flare_enabled: true,
            lens_flare_intensity: 0.5,
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

        let ssr_pass = passes::SsrPass::new(device);
        graph
            .pass(Box::new(ssr_pass))
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .read("scene_color", resources.scene_color)
            .write("ssr_raw", resources.ssr_raw);

        let ssr_blur_pass = passes::SsrBlurPass::new(device);
        graph
            .pass(Box::new(ssr_blur_pass))
            .read("ssr_raw", resources.ssr_raw)
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .write("ssr", resources.ssr);

        let postprocess_texture = graph
            .add_color_texture("postprocess")
            .format(wgpu::TextureFormat::Rgba8Unorm)
            .size(width, height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let postprocess_pass =
            passes::PostProcessPass::new(device, wgpu::TextureFormat::Rgba8Unorm, 0.005);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .read("ssr", resources.ssr)
            .write("output", postprocess_texture);

        let lens_flare_pass =
            LensFlarePass::new(device, surface_format, self.lens_flare_state.clone());
        graph
            .pass(Box::new(lens_flare_pass))
            .read("input", postprocess_texture)
            .write("output", resources.compute_output);

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        graph
            .pass(Box::new(fxaa_pass))
            .read("input", resources.compute_output)
            .write("output", fxaa_output);

        let swapchain_blit_pass =
            passes::BlitPass::new(device, surface_format).with_name("default_swapchain_blit");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", fxaa_output)
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

        world.resources.graphics.ssr_enabled = false;
        world.resources.graphics.ssr_max_steps = 64;
        world.resources.graphics.ssr_thickness = 0.25;
        world.resources.graphics.ssr_max_distance = 50.0;
        world.resources.graphics.ssr_stride = 1.0;
        world.resources.graphics.ssr_fade_start = 0.7;
        world.resources.graphics.ssr_fade_end = 1.0;
        world.resources.graphics.ssr_intensity = 1.0;

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
        if let Some(light) = world.core.get_light_mut(sun) {
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
        world.core.set_local_transform(
            ground,
            LocalTransform {
                translation: Vec3::new(0.0, -2.0, 0.0),
                rotation: Quat::identity(),
                scale: Vec3::new(10.0, 0.1, 10.0),
            },
        );
        world.core.set_render_mesh(ground, RenderMesh::new("Cube"));
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
        world
            .core
            .set_material_ref(ground, MaterialRef::new(ground_material));
        world.core.set_casts_shadow(ground, CastsShadow);

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
        self.update_lens_flare(world);

        if self.loaded {
            for entity in &self.model_entities {
                if let Some(transform) = world.core.get_local_transform_mut(*entity) {
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

                ui.horizontal(|ui| {
                    ui.label("SSR:");
                    ui.checkbox(&mut world.resources.graphics.ssr_enabled, "Enabled");
                });

                if world.resources.graphics.ssr_enabled {
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_max_steps, 8..=128)
                            .text("Max Steps"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_thickness, 0.01..=2.0)
                            .text("Thickness"),
                    );
                    ui.add(
                        egui::Slider::new(
                            &mut world.resources.graphics.ssr_max_distance,
                            1.0..=200.0,
                        )
                        .text("Max Distance"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_stride, 0.1..=4.0)
                            .text("Stride"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_fade_start, 0.0..=1.0)
                            .text("Fade Start"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_fade_end, 0.0..=1.0)
                            .text("Fade End"),
                    );
                    ui.add(
                        egui::Slider::new(&mut world.resources.graphics.ssr_intensity, 0.0..=2.0)
                            .text("Intensity"),
                    );
                }

                ui.horizontal(|ui| {
                    ui.label("Lens Flare:");
                    ui.checkbox(&mut self.lens_flare_enabled, "Enabled");
                });

                if self.lens_flare_enabled {
                    ui.add(
                        egui::Slider::new(&mut self.lens_flare_intensity, 0.0..=1.0)
                            .text("Flare Intensity"),
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
    fn get_day_night_sun_direction(hour: f32) -> Vec3 {
        let pi = std::f32::consts::PI;
        if !(6.0..=18.0).contains(&hour) {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            let sun_angle = (hour - 6.0) / 12.0 * pi;
            nalgebra_glm::normalize(&Vec3::new(-sun_angle.cos(), sun_angle.sin(), -0.3))
        }
    }

    fn get_atmosphere_sun_direction(atmosphere: Atmosphere, hour: f32) -> Option<Vec3> {
        match atmosphere {
            Atmosphere::Sky | Atmosphere::CloudySky => {
                Some(nalgebra_glm::normalize(&Vec3::new(0.0, 0.5, -1.0)))
            }
            Atmosphere::Sunset => Some(nalgebra_glm::normalize(&Vec3::new(0.5, 0.12, -0.8))),
            Atmosphere::DayNight => {
                let dir = Self::get_day_night_sun_direction(hour);
                if dir.y > 0.0 { Some(dir) } else { None }
            }
            _ => None,
        }
    }

    fn update_sun_for_hour(&self, world: &mut World) {
        let sun = match self.sun_entity {
            Some(entity) => entity,
            None => return,
        };

        let sun_dir = Self::get_day_night_sun_direction(self.day_night_hour);
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

        if let Some(light) = world.core.get_light_mut(sun) {
            light.intensity = sun_intensity;
            light.color = sun_color;
        }

        let sun_position = sun_dir * 100.0;
        if let Some(transform) = world.core.get_local_transform_mut(sun) {
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

    fn update_lens_flare(&self, world: &World) {
        if !self.lens_flare_enabled {
            if let Ok(mut state) = self.lens_flare_state.write() {
                state.intensity = 0.0;
            }
            return;
        }

        let atmosphere = world.resources.graphics.atmosphere;
        if matches!(atmosphere, Atmosphere::Space | Atmosphere::Nebula) {
            if let Ok(mut state) = self.lens_flare_state.write() {
                state.intensity = 0.0;
            }
            return;
        }
        let sun_dir = match Self::get_atmosphere_sun_direction(atmosphere, self.day_night_hour) {
            Some(dir) => dir,
            None => Self::get_day_night_sun_direction(self.day_night_hour),
        };

        let camera_entity = match self.camera_entity {
            Some(entity) => entity,
            None => return,
        };

        let matrices = match query_camera_matrices(world, camera_entity) {
            Some(matrices) => matrices,
            None => return,
        };

        let sun_world_pos = sun_dir * 100.0;
        let view_projection = matrices.projection * matrices.view;
        let clip = view_projection
            * nalgebra_glm::vec4(sun_world_pos.x, sun_world_pos.y, sun_world_pos.z, 1.0);

        if clip.w <= 0.0 {
            if let Ok(mut state) = self.lens_flare_state.write() {
                state.intensity = 0.0;
            }
            return;
        }

        let ndc_x = clip.x / clip.w;
        let ndc_y = clip.y / clip.w;

        let screen_x = ndc_x * 0.5 + 0.5;
        let screen_y = -ndc_y * 0.5 + 0.5;

        let elevation = sun_dir.y.max(0.0);
        let elevation_factor = if atmosphere == Atmosphere::DayNight {
            elevation
        } else {
            1.0
        };

        let flare_intensity = self.lens_flare_intensity * elevation_factor;

        if let Ok(mut state) = self.lens_flare_state.write() {
            state.sun_screen_x = screen_x;
            state.sun_screen_y = screen_y;
            state.intensity = flare_intensity;
        }
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
