use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::texture_loader::{
    AssetLoadingState, AssetLoadingStatus, SharedTextureQueue, create_shared_queue,
    process_and_load_textures, queue_texture_from_path,
};
use nightshade::filesystem::open_directory;
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::texture_cache_add_reference;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(HotReloadDemo::default())?;
    Ok(())
}

struct HotReloadDemo {
    texture_queue: SharedTextureQueue,
    loading_state: AssetLoadingState,
    loaded: bool,
    texture_path: PathBuf,
    material_path: PathBuf,
    texture_reload_count: u32,
    material_reload_count: u32,
    last_texture_reload_time: Option<u64>,
    last_material_reload_time: Option<u64>,
    last_texture_modified: Option<std::time::SystemTime>,
    last_material_modified: Option<std::time::SystemTime>,
    shader_path: PathBuf,
    shader_pass_enabled: bool,
    shader_reload_count: u32,
    last_shader_reload_time: Option<u64>,
    last_shader_modified: Option<std::time::SystemTime>,
    pending_shader_source: Arc<Mutex<Option<String>>>,
}

impl Default for HotReloadDemo {
    fn default() -> Self {
        Self {
            texture_queue: create_shared_queue(),
            loading_state: AssetLoadingState::new(1),
            loaded: false,
            texture_path: PathBuf::new(),
            material_path: PathBuf::new(),
            texture_reload_count: 0,
            material_reload_count: 0,
            last_texture_reload_time: None,
            last_material_reload_time: None,
            last_texture_modified: None,
            last_material_modified: None,
            shader_path: PathBuf::new(),
            shader_pass_enabled: false,
            shader_reload_count: 0,
            last_shader_reload_time: None,
            last_shader_modified: None,
            pending_shader_source: Arc::new(Mutex::new(None)),
        }
    }
}

const TEXTURE_NAME: &str = "hot_reload_test.png";
const MATERIAL_NAME: &str = "hot_reload_material";
const MATERIAL_FILE_NAME: &str = "hot_reload_material.json";
const SHADER_FILE_NAME: &str = "hot_reload_effect.wgsl";

fn generate_checkerboard(size: u32, tile_size: u32, color_a: [u8; 4], color_b: [u8; 4]) -> Vec<u8> {
    let mut data = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let tile_x = x / tile_size;
            let tile_y = y / tile_size;
            let color = if (tile_x + tile_y).is_multiple_of(2) {
                color_a
            } else {
                color_b
            };
            let offset = ((y * size + x) * 4) as usize;
            data[offset..offset + 4].copy_from_slice(&color);
        }
    }
    data
}

fn default_material() -> Material {
    Material {
        base_color: [1.0, 1.0, 1.0, 1.0],
        roughness: 0.5,
        metallic: 0.0,
        ..Default::default()
    }
}

fn write_material_json(path: &std::path::Path, material: &Material) {
    if let Ok(json) = serde_json::to_string_pretty(material) {
        let _ = std::fs::write(path, json);
    }
}

fn grayscale_shader() -> String {
    format!(
        "{FULLSCREEN_VERTEX}\n{}",
        r#"@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    let luma = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    return vec4<f32>(vec3<f32>(luma), color.a);
}"#
    )
}

fn invert_shader() -> String {
    format!(
        "{FULLSCREEN_VERTEX}\n{}",
        r#"@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    return vec4<f32>(vec3<f32>(1.0) - color.rgb, color.a);
}"#
    )
}

fn sepia_shader() -> String {
    format!(
        "{FULLSCREEN_VERTEX}\n{}",
        r#"@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    let r = dot(color.rgb, vec3<f32>(0.393, 0.769, 0.189));
    let g = dot(color.rgb, vec3<f32>(0.349, 0.686, 0.168));
    let b = dot(color.rgb, vec3<f32>(0.272, 0.534, 0.131));
    return vec4<f32>(min(r, 1.0), min(g, 1.0), min(b, 1.0), color.a);
}"#
    )
}

fn vignette_shader() -> String {
    format!(
        "{FULLSCREEN_VERTEX}\n{}",
        r#"@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center);
    let vignette = smoothstep(0.8, 0.3, dist);
    return vec4<f32>(color.rgb * vignette, color.a);
}"#
    )
}

fn edge_detection_shader() -> String {
    format!(
        "{FULLSCREEN_VERTEX}\n{}",
        r#"@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = textureDimensions(input_texture);
    let texel_size = vec2<f32>(1.0 / f32(texture_size.x), 1.0 / f32(texture_size.y));

    let tl = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-texel_size.x, -texel_size.y)).rgb);
    let tm = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, -texel_size.y)).rgb);
    let tr = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(texel_size.x, -texel_size.y)).rgb);

    let ml = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-texel_size.x, 0.0)).rgb);
    let mr = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(texel_size.x, 0.0)).rgb);

    let bl = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(-texel_size.x, texel_size.y)).rgb);
    let bm = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, texel_size.y)).rgb);
    let br = luminance(textureSample(input_texture, input_sampler, in.uv + vec2<f32>(texel_size.x, texel_size.y)).rgb);

    let gx = -tl - 2.0 * ml - bl + tr + 2.0 * mr + br;
    let gy = -tl - 2.0 * tm - tr + bl + 2.0 * bm + br;

    let edge_strength = sqrt(gx * gx + gy * gy);

    let original = textureSample(input_texture, input_sampler, in.uv).rgb;
    let result = original + vec3<f32>(edge_strength);

    return vec4<f32>(result, 1.0);
}"#
    )
}

const FULLSCREEN_VERTEX: &str = r#"struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}"#;

struct HotReloadShaderPass {
    pipeline: wgpu::RenderPipeline,
    blit_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    cached_bind_group: Option<wgpu::BindGroup>,
    pending_shader_source: Arc<Mutex<Option<String>>>,
    surface_format: wgpu::TextureFormat,
}

impl HotReloadShaderPass {
    fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        initial_shader_source: &str,
        pending_shader_source: Arc<Mutex<Option<String>>>,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hot Reload Effect Bind Group Layout"),
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
            ],
        });

        let pipeline = Self::create_effect_pipeline(
            device,
            surface_format,
            &bind_group_layout,
            initial_shader_source,
        );

        let blit_pipeline = passes::BlitPass::create_pipeline(device, surface_format);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Hot Reload Effect Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            blit_pipeline,
            bind_group_layout,
            sampler,
            cached_bind_group: None,
            pending_shader_source,
            surface_format,
        }
    }

    fn create_effect_pipeline(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        bind_group_layout: &wgpu::BindGroupLayout,
        shader_source: &str,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hot Reload Effect Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader_source)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hot Reload Effect Pipeline Layout"),
            bind_group_layouts: &[Some(bind_group_layout)],
            immediate_size: 0,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hot Reload Effect Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
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
        })
    }
}

impl PassNode<World> for HotReloadShaderPass {
    fn name(&self) -> &str {
        "hot_reload_effect"
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

    fn prepare(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, _configs: &World) {
        let new_source = self
            .pending_shader_source
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());

        if let Some(source) = new_source {
            let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
            let new_pipeline = Self::create_effect_pipeline(
                device,
                self.surface_format,
                &self.bind_group_layout,
                &source,
            );
            let error = pollster::block_on(error_scope.pop());
            if error.is_some() {
                tracing::warn!("Shader compilation failed, keeping old pipeline");
            } else {
                self.pipeline = new_pipeline;
                tracing::info!("Shader hot-reloaded successfully");
            }
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, World>,
    ) -> Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        nightshade::render::wgpu::rendergraph::RenderGraphError,
    > {
        if self.cached_bind_group.is_none() {
            let input_view = context.get_texture_view("input")?;

            self.cached_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Hot Reload Effect Bind Group"),
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
                    ],
                },
            ));
        }

        let pipeline = if context.is_pass_enabled() {
            &self.pipeline
        } else {
            &self.blit_pipeline
        };

        let (color_view, color_load_op, color_store_op) = context.get_color_attachment("output")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Hot Reload Effect Render Pass"),
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

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, self.cached_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}

impl State for HotReloadDemo {
    fn title(&self) -> &str {
        "Hot Reload"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;

        load_procedural_textures(world);
        capture_procedural_atmosphere_ibl(world, Atmosphere::Nebula, 0.0);
        spawn_sun(world);

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 1.5, 0.0),
            8.0,
            0.0,
            0.3,
            "Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let texture_path = exe_dir.join(TEXTURE_NAME);
        let material_path = exe_dir.join(MATERIAL_FILE_NAME);
        let shader_path = exe_dir.join(SHADER_FILE_NAME);

        let size = 256u32;
        let rgba = generate_checkerboard(size, 32, [60, 120, 220, 255], [240, 240, 240, 255]);
        let img =
            image::RgbaImage::from_raw(size, size, rgba).expect("failed to create image buffer");
        img.save(&texture_path)
            .expect("failed to write test texture");

        let material = default_material();
        write_material_json(&material_path, &material);

        let shader_source = grayscale_shader();
        let _ = std::fs::write(&shader_path, &shader_source);

        tracing::info!("Wrote test texture to: {}", texture_path.display());
        tracing::info!("Wrote material JSON to: {}", material_path.display());
        tracing::info!("Wrote shader to: {}", shader_path.display());

        self.texture_path = texture_path.clone();
        self.material_path = material_path;
        self.shader_path = shader_path.clone();

        world
            .resources
            .file_watcher
            .watch("shader:effect".to_string(), shader_path);

        let path_str = texture_path.to_string_lossy().replace('\\', "/");
        queue_texture_from_path(&self.texture_queue, &path_str);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        if !self.loaded {
            let status =
                process_and_load_textures(&self.texture_queue, world, &mut self.loading_state, 4);
            if status == AssetLoadingStatus::Complete {
                self.loaded = true;
                tracing::info!("Initial texture loaded, spawning meshes");
                self.last_texture_modified = std::fs::metadata(&self.texture_path)
                    .ok()
                    .and_then(|m| m.modified().ok());
                self.last_material_modified = std::fs::metadata(&self.material_path)
                    .ok()
                    .and_then(|m| m.modified().ok());
                self.last_shader_modified = std::fs::metadata(&self.shader_path)
                    .ok()
                    .and_then(|m| m.modified().ok());

                let texture_name = self.texture_path.to_string_lossy().replace('\\', "/");
                spawn_demo_meshes(world, &texture_name, &self.material_path);

                world
                    .resources
                    .asset_watcher
                    .track_texture(texture_name, self.texture_path.clone());
                world
                    .resources
                    .asset_watcher
                    .track_material(MATERIAL_NAME.to_string(), self.material_path.clone());
            }
        }

        if self.loaded {
            let current_tex_modified = std::fs::metadata(&self.texture_path)
                .ok()
                .and_then(|m| m.modified().ok());
            if current_tex_modified != self.last_texture_modified {
                self.last_texture_modified = current_tex_modified;
                self.texture_reload_count += 1;
                self.last_texture_reload_time =
                    Some(world.resources.window.timing.uptime_milliseconds);
            }

            let current_mat_modified = std::fs::metadata(&self.material_path)
                .ok()
                .and_then(|m| m.modified().ok());
            if current_mat_modified != self.last_material_modified {
                self.last_material_modified = current_mat_modified;
                self.material_reload_count += 1;
                self.last_material_reload_time =
                    Some(world.resources.window.timing.uptime_milliseconds);
            }
        }

        if world.resources.file_watcher.take_change("shader:effect")
            && let Ok(source) = std::fs::read_to_string(&self.shader_path)
        {
            if let Ok(mut guard) = self.pending_shader_source.lock() {
                *guard = Some(source);
            }
            self.shader_reload_count += 1;
            self.last_shader_reload_time = Some(world.resources.window.timing.uptime_milliseconds);
            self.last_shader_modified = std::fs::metadata(&self.shader_path)
                .ok()
                .and_then(|m| m.modified().ok());
        }
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let bloom_width = resources.surface_width / 2;
        let bloom_height = resources.surface_height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass =
            passes::BloomPass::new(device, resources.surface_width, resources.surface_height);
        let _ = graph.add_pass(
            Box::new(bloom_pass),
            &[("hdr", resources.scene_color), ("bloom", bloom_texture)],
        );

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 1.0);
        let _ = graph.add_pass(
            Box::new(postprocess_pass),
            &[
                ("hdr", resources.scene_color),
                ("bloom", bloom_texture),
                ("ssao", resources.ssao),
                ("output", resources.compute_output),
            ],
        );

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        let _ = graph.add_pass(
            Box::new(fxaa_pass),
            &[("input", resources.compute_output), ("output", fxaa_output)],
        );

        let initial_source =
            std::fs::read_to_string(&self.shader_path).unwrap_or_else(|_| grayscale_shader());

        let effect_pass = HotReloadShaderPass::new(
            device,
            surface_format,
            &initial_source,
            self.pending_shader_source.clone(),
        );
        let _ = graph.add_pass(
            Box::new(effect_pass),
            &[("input", fxaa_output), ("output", resources.swapchain)],
        );
    }

    fn update_render_graph(&mut self, graph: &mut RenderGraph<World>, _world: &World) {
        let _ = graph.set_pass_enabled("hot_reload_effect", self.shader_pass_enabled);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Hot Reload Test")
            .default_pos([10.0, 10.0])
            .default_width(380.0)
            .show(ui_context, |ui| {
                ui.heading("Texture Hot-Reload");
                ui.separator();

                ui.label("Test texture path:");
                let tex_path_str = self.texture_path.to_string_lossy();
                ui.monospace(tex_path_str.as_ref());

                ui.horizontal(|ui| {
                    if ui.button("Copy path").clicked() {
                        ui.ctx().copy_text(tex_path_str.to_string());
                    }
                    if ui.button("Open directory").clicked() {
                        open_directory(&self.texture_path);
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Status:");
                    if self.loaded {
                        ui.colored_label(egui::Color32::GREEN, "Watching for changes");
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "Loading...");
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Texture reloads:");
                    ui.monospace(format!("{}", self.texture_reload_count));
                });

                if let Some(reload_time) = self.last_texture_reload_time {
                    let elapsed_ms = world
                        .resources
                        .window
                        .timing
                        .uptime_milliseconds
                        .saturating_sub(reload_time);
                    let elapsed_secs = elapsed_ms as f64 / 1000.0;
                    ui.horizontal(|ui| {
                        ui.label("Last texture reload:");
                        ui.monospace(format!("{elapsed_secs:.1}s ago"));
                    });
                }

                ui.separator();

                if ui.button("Regenerate as red/white checkerboard").clicked() {
                    let rgba =
                        generate_checkerboard(256, 32, [220, 50, 50, 255], [255, 255, 255, 255]);
                    let img = image::RgbaImage::from_raw(256, 256, rgba).unwrap();
                    let _ = img.save(&self.texture_path);
                }

                if ui
                    .button("Regenerate as green/black checkerboard")
                    .clicked()
                {
                    let rgba =
                        generate_checkerboard(256, 16, [30, 200, 60, 255], [20, 20, 20, 255]);
                    let img = image::RgbaImage::from_raw(256, 256, rgba).unwrap();
                    let _ = img.save(&self.texture_path);
                }

                if ui.button("Regenerate as gradient").clicked() {
                    let size = 256u32;
                    let mut data = vec![0u8; (size * size * 4) as usize];
                    for y in 0..size {
                        for x in 0..size {
                            let offset = ((y * size + x) * 4) as usize;
                            data[offset] = (x * 255 / size) as u8;
                            data[offset + 1] = (y * 255 / size) as u8;
                            data[offset + 2] = 128;
                            data[offset + 3] = 255;
                        }
                    }
                    let img = image::RgbaImage::from_raw(size, size, data).unwrap();
                    let _ = img.save(&self.texture_path);
                }

                ui.add_space(16.0);
                ui.heading("Material Hot-Reload");
                ui.separator();

                ui.label("Material JSON path:");
                let mat_path_str = self.material_path.to_string_lossy();
                ui.monospace(mat_path_str.as_ref());

                ui.horizontal(|ui| {
                    if ui.button("Copy path").clicked() {
                        ui.ctx().copy_text(mat_path_str.to_string());
                    }
                    if ui.button("Open directory").clicked() {
                        open_directory(&self.material_path);
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Material reloads:");
                    ui.monospace(format!("{}", self.material_reload_count));
                });

                if let Some(reload_time) = self.last_material_reload_time {
                    let elapsed_ms = world
                        .resources
                        .window
                        .timing
                        .uptime_milliseconds
                        .saturating_sub(reload_time);
                    let elapsed_secs = elapsed_ms as f64 / 1000.0;
                    ui.horizontal(|ui| {
                        ui.label("Last material reload:");
                        ui.monospace(format!("{elapsed_secs:.1}s ago"));
                    });
                }

                ui.separator();
                ui.label("Quick material presets:");

                if ui.button("Make metallic (gold)").clicked() {
                    let material = Material {
                        base_color: [1.0, 0.84, 0.0, 1.0],
                        roughness: 0.2,
                        metallic: 1.0,
                        ..Default::default()
                    };
                    write_material_json(&self.material_path, &material);
                }

                if ui.button("Make rough (matte red)").clicked() {
                    let material = Material {
                        base_color: [0.8, 0.15, 0.15, 1.0],
                        roughness: 0.95,
                        metallic: 0.0,
                        ..Default::default()
                    };
                    write_material_json(&self.material_path, &material);
                }

                if ui.button("Make emissive (neon green)").clicked() {
                    let material = Material {
                        base_color: [0.1, 0.1, 0.1, 1.0],
                        emissive_factor: [0.0, 5.0, 0.0],
                        roughness: 0.5,
                        metallic: 0.0,
                        emissive_strength: 2.0,
                        ..Default::default()
                    };
                    write_material_json(&self.material_path, &material);
                }

                if ui.button("Make glossy (chrome)").clicked() {
                    let material = Material {
                        base_color: [0.9, 0.9, 0.95, 1.0],
                        roughness: 0.05,
                        metallic: 1.0,
                        ..Default::default()
                    };
                    write_material_json(&self.material_path, &material);
                }

                if ui.button("Reset to default").clicked() {
                    write_material_json(&self.material_path, &default_material());
                }

                ui.add_space(16.0);
                ui.heading("Shader Hot-Reload");
                ui.separator();

                ui.label("Shader path:");
                let shader_path_str = self.shader_path.to_string_lossy();
                ui.monospace(shader_path_str.as_ref());

                ui.horizontal(|ui| {
                    if ui.button("Copy path").clicked() {
                        ui.ctx().copy_text(shader_path_str.to_string());
                    }
                    if ui.button("Open directory").clicked() {
                        open_directory(&self.shader_path);
                    }
                });

                ui.separator();

                ui.checkbox(&mut self.shader_pass_enabled, "Enable shader effect");

                ui.horizontal(|ui| {
                    ui.label("Shader reloads:");
                    ui.monospace(format!("{}", self.shader_reload_count));
                });

                if let Some(reload_time) = self.last_shader_reload_time {
                    let elapsed_ms = world
                        .resources
                        .window
                        .timing
                        .uptime_milliseconds
                        .saturating_sub(reload_time);
                    let elapsed_secs = elapsed_ms as f64 / 1000.0;
                    ui.horizontal(|ui| {
                        ui.label("Last shader reload:");
                        ui.monospace(format!("{elapsed_secs:.1}s ago"));
                    });
                }

                ui.separator();
                ui.label("Shader presets:");

                if ui.button("Grayscale").clicked() {
                    let _ = std::fs::write(&self.shader_path, grayscale_shader());
                }
                if ui.button("Invert Colors").clicked() {
                    let _ = std::fs::write(&self.shader_path, invert_shader());
                }
                if ui.button("Sepia").clicked() {
                    let _ = std::fs::write(&self.shader_path, sepia_shader());
                }
                if ui.button("Vignette").clicked() {
                    let _ = std::fs::write(&self.shader_path, vignette_shader());
                }
                if ui.button("Edge Detection").clicked() {
                    let _ = std::fs::write(&self.shader_path, edge_detection_shader());
                }
            });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}

fn spawn_demo_meshes(world: &mut World, texture_name: &str, material_path: &std::path::Path) {
    let material = match std::fs::read_to_string(material_path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_else(|_| default_material()),
        Err(_) => default_material(),
    };

    material_registry_insert(
        &mut world.resources.material_registry,
        MATERIAL_NAME.to_string(),
        material,
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(MATERIAL_NAME)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }

    spawn_material_mesh(world, "Torus", Vec3::new(-3.0, 1.5, 0.0), "Torus");
    spawn_material_mesh(world, "Sphere", Vec3::new(0.0, 1.5, 0.0), "Sphere");
    spawn_material_mesh(world, "Cone", Vec3::new(3.0, 1.5, 0.0), "Cone");

    spawn_textured_mesh(
        world,
        "Cube",
        texture_name,
        Vec3::new(-3.0, 1.5, -3.0),
        "Cube",
    );
    spawn_textured_mesh(
        world,
        "Sphere",
        texture_name,
        Vec3::new(0.0, 1.5, -3.0),
        "Sphere (Textured)",
    );
    spawn_textured_mesh(
        world,
        "Cylinder",
        texture_name,
        Vec3::new(3.0, 1.5, -3.0),
        "Cylinder",
    );
}

fn spawn_material_mesh(world: &mut World, mesh_name: &str, position: Vec3, label: &str) {
    let entity = world.spawn_entities(
        RENDER_MESH
            | MATERIAL_REF
            | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | BOUNDING_VOLUME
            | NAME
            | VISIBILITY,
        1,
    )[0];

    world
        .core
        .set_render_mesh(entity, RenderMesh::new(mesh_name));

    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(MATERIAL_NAME)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world
        .core
        .set_material_ref(entity, MaterialRef::new(MATERIAL_NAME));

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    if let Some(name) = world.core.get_name_mut(entity) {
        *name = Name(format!("Hot Reload {label}"));
    }

    world.resources.mesh_render_state.mark_entity_added(entity);
}

fn spawn_textured_mesh(
    world: &mut World,
    mesh_name: &str,
    texture_name: &str,
    position: Vec3,
    label: &str,
) {
    let entity = world.spawn_entities(
        RENDER_MESH
            | MATERIAL_REF
            | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | BOUNDING_VOLUME
            | NAME
            | VISIBILITY,
        1,
    )[0];

    world
        .core
        .set_render_mesh(entity, RenderMesh::new(mesh_name));

    let material_name = format!("HotReload_{}_{}", label, entity.id);
    texture_cache_add_reference(&mut world.resources.texture_cache, texture_name);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(texture_name.to_string()),
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    if let Some(name) = world.core.get_name_mut(entity) {
        *name = Name(format!("Hot Reload {label}"));
    }

    world.resources.mesh_render_state.mark_entity_added(entity);
}
