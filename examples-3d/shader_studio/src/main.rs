use std::sync::{Arc, Mutex};

use std::collections::HashMap;

use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::components::Projection;
use nightshade::ecs::camera::queries::query_active_camera_matrices;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::mesh::components::Mesh;
use nightshade::ecs::prefab::commands::{
    GltfLoadResult, import_gltf_from_bytes, import_gltf_from_path,
};
use nightshade::ecs::prefab::components::PrefabNode;
use nightshade::ecs::transform::systems::update_global_transforms_system;
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;

mod geometry;
mod graph;
mod presets;
mod sdf_editor;
mod shader_pass;
mod syntax;

use geometry::{MeshData, PrimitiveType, ShaderVertex};
use shader_pass::{ChannelSource, PendingTexture, RenderMode, ShaderPass, SharedState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(ShaderStudio::default())?;
    Ok(())
}

struct ShaderStudio {
    shared: Arc<Mutex<SharedState>>,
    selected_preset: usize,
    auto_compile: bool,
    compile_timers: [f32; 5],
    source_dirty: [bool; 5],
    common_dirty: bool,
    common_compile_timer: f32,
    show_left_panel: bool,
    custom_sliders: [f32; 16],
    slider_labels: [String; 16],
    slider_ranges: [(f32, f32); 16],
    show_all_sliders: bool,
    accumulated_time: f32,
    frame_count: u32,
    last_fps: f32,
    fps_timer: f32,
    fps_frame_count: u32,
    shuffle: bool,
    shuffle_timer: f32,
    graph: graph::PipelineGraph,
    source_history: Vec<String>,
    save_status: Option<(String, std::time::Instant)>,
    previous_atmosphere: Atmosphere,
    sdf_editor: sdf_editor::SdfEditor,
    preset_filter: String,
    category_filter: Option<String>,
    preset_slider_count: usize,
}

impl Default for ShaderStudio {
    fn default() -> Self {
        let shared = Arc::new(Mutex::new(SharedState::default()));

        {
            let mut locked = shared.lock().unwrap();
            locked.pending_texture_data.push(PendingTexture {
                width: 256,
                height: 256,
                data: generate_builtin_texture_gradient(256, 256),
                slot: 0,
                srgb: true,
            });
            locked.texture_slot_names[0] = Some("gradient noise (built-in)".to_string());
            locked.pending_texture_data.push(PendingTexture {
                width: 256,
                height: 256,
                data: generate_builtin_texture_dots(256, 256),
                slot: 1,
                srgb: true,
            });
            locked.texture_slot_names[1] = Some("dot grid (built-in)".to_string());
            locked.pending_texture_data.push(PendingTexture {
                width: 256,
                height: 256,
                data: generate_builtin_texture_checkerboard(256, 256),
                slot: 2,
                srgb: true,
            });
            locked.texture_slot_names[2] = Some("checkerboard (built-in)".to_string());
        }

        let mut slider_labels: [String; 16] =
            std::array::from_fn(|index| format!("Custom {index}"));
        slider_labels[0] = "Color R".to_string();
        slider_labels[1] = "Color G".to_string();
        slider_labels[2] = "Color B".to_string();
        slider_labels[3] = "Color A".to_string();

        Self {
            shared,
            selected_preset: 0,
            auto_compile: true,
            compile_timers: [0.0; 5],
            source_dirty: [false; 5],
            common_dirty: false,
            common_compile_timer: 0.0,
            show_left_panel: true,
            custom_sliders: [
                0.7, 0.3, 0.2, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ],
            slider_labels,
            slider_ranges: [(0.0, 1.0); 16],
            show_all_sliders: false,
            accumulated_time: 0.0,
            frame_count: 0,
            last_fps: 0.0,
            fps_timer: 0.0,
            fps_frame_count: 0,
            shuffle: false,
            shuffle_timer: 0.0,
            graph: graph::PipelineGraph::new(),
            source_history: Vec::new(),
            save_status: None,
            previous_atmosphere: Atmosphere::Sky,
            sdf_editor: sdf_editor::SdfEditor::default(),
            preset_filter: String::new(),
            category_filter: None,
            preset_slider_count: 4,
        }
    }
}

impl State for ShaderStudio {
    fn title(&self) -> &str {
        "Shader Studio"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.clear_color = [0.05, 0.05, 0.08, 1.0];
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.ssao_enabled = true;
        world.resources.graphics.ssao_radius = 0.5;
        world.resources.graphics.ssao_bias = 0.025;
        world.resources.graphics.ssao_intensity = 1.5;

        const HDR_BYTES: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");
        load_hdr_skybox(world, HDR_BYTES.to_vec());

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
        }

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            3.0,
            0.0,
            0.3,
            "Shader Studio Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);

        if let Some(camera) = world.get_camera_mut(camera_entity)
            && let Projection::Perspective(ref mut perspective) = camera.projection
        {
            perspective.y_fov_rad = 60.0_f32.to_radians();
        }

        const GLTF_DATA: &[u8] = include_bytes!("../../../assets/gltf/DamagedHelmet.glb");
        if let Ok(result) = import_gltf_from_bytes(GLTF_DATA) {
            self.load_gltf_model("DamagedHelmet.glb", result, world);
        }
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

        let shader_pass = ShaderPass::new(device, self.shared.clone());
        graph
            .pass(Box::new(shader_pass))
            .slot("hdr", resources.scene_color);

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

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        let current_atmosphere = world.resources.graphics.atmosphere;
        if current_atmosphere != self.previous_atmosphere {
            if current_atmosphere.is_procedural() {
                capture_procedural_atmosphere_ibl(world, current_atmosphere, 0.0);
            }
            self.previous_atmosphere = current_atmosphere;
        }

        let delta = world.resources.window.timing.delta_time;

        self.fps_timer += delta;
        self.fps_frame_count += 1;
        if self.fps_timer >= 0.5 {
            self.last_fps = self.fps_frame_count as f32 / self.fps_timer;
            self.fps_timer = 0.0;
            self.fps_frame_count = 0;
        }

        let mut shared = self.shared.lock().unwrap();

        if !shared.paused {
            self.accumulated_time += delta * shared.speed;
        }
        self.frame_count += 1;

        shared.uniforms.time = self.accumulated_time + shared.time_offset;
        shared.uniforms.delta_time = delta;
        shared.uniforms.frame = self.frame_count;

        if let Some((width, height)) = world.resources.window.cached_viewport_size {
            shared.uniforms.resolution = [width as f32, height as f32];
        }

        let mouse = &world.resources.input.mouse;
        let res_x = shared.uniforms.resolution[0].max(1.0);
        let res_y = shared.uniforms.resolution[1].max(1.0);
        shared.uniforms.mouse = [mouse.position.x / res_x, 1.0 - mouse.position.y / res_y];

        if shared.render_mode == RenderMode::Geometry {
            let time = shared.uniforms.time;
            let rotation = nalgebra_glm::quat_angle_axis(time * 0.5, &Vec3::y());
            let rotation_matrix = nalgebra_glm::quat_to_mat4(&rotation);
            shared.uniforms.model = mat4_to_arrays(&rotation_matrix);
        }

        drop(shared);
        pan_orbit_camera_system(world);
        update_global_transforms_system(world);
        let mut shared = self.shared.lock().unwrap();

        if let Some(matrices) = query_active_camera_matrices(world) {
            shared.uniforms.view = mat4_to_arrays(&matrices.view);
            shared.uniforms.projection = mat4_to_arrays(&matrices.projection);
            shared.uniforms.camera_position = [
                matrices.camera_position.x,
                matrices.camera_position.y,
                matrices.camera_position.z,
            ];
        }

        for row in 0..4 {
            for col in 0..4 {
                shared.uniforms.custom[row][col] = self.custom_sliders[row * 4 + col];
            }
        }

        let should_shuffle = self.shuffle && !shared.paused;
        drop(shared);

        if should_shuffle {
            self.shuffle_timer += delta;
            if self.shuffle_timer >= 4.0 {
                self.shuffle_timer = 0.0;
                let next_preset = (self.selected_preset + 1) % presets::PRESETS.len();
                self.apply_preset(next_preset);
            }
        }

        let mut shared = self.shared.lock().unwrap();

        if self.common_dirty && self.auto_compile {
            self.common_compile_timer += delta;
            if self.common_compile_timer > 0.5 {
                self.common_dirty = false;
                self.common_compile_timer = 0.0;
                for pass_index in 0..5 {
                    if shared.pass_enabled[pass_index] {
                        shared.pass_needs_recompile[pass_index] = true;
                    }
                }
            }
        }

        for pass_index in 0..5 {
            if self.source_dirty[pass_index] && self.auto_compile {
                self.compile_timers[pass_index] += delta;
                if self.compile_timers[pass_index] > 0.5 {
                    shared.pass_needs_recompile[pass_index] = true;
                    self.source_dirty[pass_index] = false;
                    self.compile_timers[pass_index] = 0.0;
                }
            }
        }

        let mut compilation_tasks: Vec<(usize, String, u64, bool)> = Vec::new();
        for pass_index in 0..5 {
            if shared.pass_needs_recompile[pass_index] && !shared.pass_is_compiling[pass_index] {
                shared.pass_needs_recompile[pass_index] = false;
                let common = shared.common_source.clone();
                let source = shared.pass_sources[pass_index].clone();
                let generation = shared.compilation_generation;
                let full_source = if common.is_empty() {
                    source
                } else {
                    format!("{common}\n{source}")
                };
                let has_common = !common.is_empty();
                compilation_tasks.push((pass_index, full_source, generation, has_common));
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            for (pass_index, full_source, generation, has_common) in compilation_tasks {
                shared.pass_is_compiling[pass_index] = true;
                let shared_clone = self.shared.clone();
                std::thread::spawn(move || {
                    let result = shader_pass::validate_shader(&full_source);
                    let mut shared = shared_clone.lock().unwrap();
                    shared.pass_is_compiling[pass_index] = false;
                    if shared.compilation_generation != generation {
                        return;
                    }
                    match result {
                        Ok(mode) => {
                            shared.pass_compilation_errors[pass_index] = None;
                            shared.pass_pending_validated[pass_index] = Some((full_source, mode));
                            if has_common {
                                shared.common_error = None;
                            }
                        }
                        Err(error) => {
                            if has_common {
                                shared.common_error = Some(error.clone());
                            }
                            shared.pass_compilation_errors[pass_index] = Some(error);
                        }
                    }
                });
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            drop(shared);
            for (pass_index, full_source, generation, has_common) in compilation_tasks {
                let result = shader_pass::validate_shader(&full_source);
                let mut shared = self.shared.lock().unwrap();
                if shared.compilation_generation != generation {
                    continue;
                }
                match result {
                    Ok(mode) => {
                        shared.pass_compilation_errors[pass_index] = None;
                        shared.pass_pending_validated[pass_index] = Some((full_source, mode));
                        if has_common {
                            shared.common_error = None;
                        }
                    }
                    Err(error) => {
                        if has_common {
                            shared.common_error = Some(error.clone());
                        }
                        shared.pass_compilation_errors[pass_index] = Some(error);
                    }
                }
            }
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        self.handle_keyboard_shortcuts(ui_context);
        self.top_bar(ui_context);
        self.bottom_bar(ui_context);
        if self.show_left_panel {
            self.left_panel(ui_context, world);
        }
        self.graph.show(ui_context, &self.shared);
    }

    fn on_dropped_file(&mut self, world: &mut World, path: &std::path::Path) {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            let extension_lower = extension.to_lowercase();
            if matches!(
                extension_lower.as_str(),
                "png" | "jpg" | "jpeg" | "bmp" | "tga"
            ) && let Ok(image_data) = std::fs::read(path)
            {
                self.load_texture_from_bytes(
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("texture"),
                    &image_data,
                );
            } else if matches!(extension_lower.as_str(), "glb" | "gltf") {
                let file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("model")
                    .to_string();
                match import_gltf_from_path(path) {
                    Ok(result) => self.load_gltf_model(&file_name, result, world),
                    Err(error) => {
                        let mut shared = self.shared.lock().unwrap();
                        shared.pass_compilation_errors[0] =
                            Some(format!("Failed to load GLTF: {error}"));
                    }
                }
            }
        }
    }

    fn on_dropped_file_data(&mut self, world: &mut World, name: &str, data: &[u8]) {
        let extension = name.rsplit('.').next().unwrap_or("").to_lowercase();
        if matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "tga") {
            self.load_texture_from_bytes(name, data);
        } else if matches!(extension.as_str(), "glb" | "gltf") {
            match import_gltf_from_bytes(data) {
                Ok(result) => self.load_gltf_model(name, result, world),
                Err(error) => {
                    let mut shared = self.shared.lock().unwrap();
                    shared.pass_compilation_errors[0] =
                        Some(format!("Failed to load GLTF: {error}"));
                }
            }
        }
    }
}

impl ShaderStudio {
    fn handle_keyboard_shortcuts(&mut self, ui_context: &egui::Context) {
        let wants_text = ui_context.wants_keyboard_input();

        ui_context.input(|input| {
            if input.key_pressed(egui::Key::F11) {
                self.show_left_panel = !self.show_left_panel;
            }

            if !wants_text {
                if input.modifiers.ctrl
                    && input.key_pressed(egui::Key::Z)
                    && let Some(previous) = self.source_history.pop()
                {
                    let mut shared = self.shared.lock().unwrap();
                    shared.pass_sources[0] = previous;
                    shared.pass_needs_recompile[0] = true;
                    self.source_dirty = [false; 5];
                    self.compile_timers = [0.0; 5];
                }
                if input.key_pressed(egui::Key::Space) {
                    let mut shared = self.shared.lock().unwrap();
                    shared.paused = !shared.paused;
                }
                if input.key_pressed(egui::Key::R) {
                    let mut shared = self.shared.lock().unwrap();
                    self.accumulated_time = 0.0;
                    shared.time_offset = 0.0;
                }
                if input.key_pressed(egui::Key::ArrowRight) {
                    let next = (self.selected_preset + 1) % presets::PRESETS.len();
                    self.apply_preset(next);
                }
                if input.key_pressed(egui::Key::ArrowLeft) {
                    let prev = if self.selected_preset == 0 {
                        presets::PRESETS.len() - 1
                    } else {
                        self.selected_preset - 1
                    };
                    self.apply_preset(prev);
                }
                if input.key_pressed(egui::Key::G) {
                    self.graph.visible = !self.graph.visible;
                }
                if input.modifiers.ctrl && input.key_pressed(egui::Key::S) {
                    let shared = self.shared.lock().unwrap();
                    let source = shared.active_pass_source().to_string();
                    drop(shared);
                    self.save_shader_to_file(&source);
                }
            }
        });
    }

    fn load_texture_from_bytes(&mut self, name: &str, data: &[u8]) {
        if let Ok(image) = image::load_from_memory(data) {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();

            let mut shared = self.shared.lock().unwrap();
            let slot = shared
                .texture_slot_names
                .iter()
                .position(|slot_name| slot_name.is_none())
                .unwrap_or(0);

            shared.texture_slot_names[slot] = Some(name.to_string());
            shared.pending_texture_data.push(PendingTexture {
                width,
                height,
                data: rgba.into_raw(),
                slot,
                srgb: true,
            });
        }
    }

    fn load_gltf_model(&mut self, name: &str, result: GltfLoadResult, world: &mut World) {
        let mut shared = self.shared.lock().unwrap();

        let material = result.materials.first();

        struct TextureBinding {
            texture_name: Option<String>,
            slot: usize,
            label: String,
            srgb: bool,
        }

        let bindings = [
            TextureBinding {
                texture_name: material.and_then(|mat| mat.base_texture.clone()),
                slot: 0,
                label: format!("{name} base_color"),
                srgb: true,
            },
            TextureBinding {
                texture_name: material.and_then(|mat| mat.normal_texture.clone()),
                slot: 1,
                label: format!("{name} normal"),
                srgb: false,
            },
            TextureBinding {
                texture_name: material.and_then(|mat| mat.metallic_roughness_texture.clone()),
                slot: 2,
                label: format!("{name} metallic_roughness"),
                srgb: false,
            },
            TextureBinding {
                texture_name: material.and_then(|mat| mat.emissive_texture.clone()),
                slot: 3,
                label: format!("{name} emissive"),
                srgb: true,
            },
        ];

        for binding in &bindings {
            if let Some(ref tex_name) = binding.texture_name
                && let Some((tex_data, tex_width, tex_height)) = result.textures.get(tex_name)
            {
                shared.texture_slot_names[binding.slot] = Some(binding.label.clone());
                shared.pending_texture_data.push(PendingTexture {
                    width: *tex_width,
                    height: *tex_height,
                    data: tex_data.clone(),
                    slot: binding.slot,
                    srgb: binding.srgb,
                });
            } else {
                let (placeholder_data, placeholder_label) = match binding.slot {
                    1 => (vec![128, 128, 255, 255], format!("{name} flat normal")),
                    2 => (vec![255, 255, 255, 255], format!("{name} default mr")),
                    3 => (vec![0, 0, 0, 255], format!("{name} no emissive")),
                    _ => (vec![255, 255, 255, 255], format!("{name} white")),
                };
                shared.texture_slot_names[binding.slot] = Some(placeholder_label);
                shared.pending_texture_data.push(PendingTexture {
                    width: 1,
                    height: 1,
                    data: placeholder_data,
                    slot: binding.slot,
                    srgb: binding.srgb,
                });
            }
        }

        shared.channel_bindings[0] = [
            ChannelSource::Texture0,
            ChannelSource::Texture1,
            ChannelSource::Texture2,
            ChannelSource::Texture3,
        ];
        shared.channels_dirty = true;

        let mut combined_vertices: Vec<ShaderVertex> = Vec::new();
        let mut combined_indices: Vec<u32> = Vec::new();

        let has_prefab_nodes = result
            .prefabs
            .iter()
            .any(|prefab| !prefab.root_nodes.is_empty());

        if has_prefab_nodes {
            for prefab in &result.prefabs {
                for node in &prefab.root_nodes {
                    collect_meshes_recursive(
                        node,
                        &nalgebra_glm::Mat4::identity(),
                        &result.meshes,
                        &mut combined_vertices,
                        &mut combined_indices,
                    );
                }
            }
        }

        if combined_vertices.is_empty() {
            for mesh in result.meshes.values() {
                let base_index = combined_vertices.len() as u32;

                for vertex in &mesh.vertices {
                    combined_vertices.push(ShaderVertex {
                        position: vertex.position,
                        normal: vertex.normal,
                        uv: vertex.tex_coords,
                    });
                }

                for index in &mesh.indices {
                    combined_indices.push(base_index + index);
                }
            }
        }

        if combined_vertices.is_empty() {
            shared.pass_compilation_errors[0] = Some("GLTF file contains no meshes".to_string());
            return;
        }

        let mesh_data = MeshData {
            vertices: combined_vertices,
            indices: combined_indices,
        };

        let mesh_count = result.meshes.len();
        let display_name = format!("{name} ({mesh_count} meshes)");
        shared.custom_mesh_data = Some(mesh_data);
        shared.custom_mesh_name = Some(display_name);
        shared.primitive_type = PrimitiveType::Custom;
        shared.upload_custom_mesh = true;

        drop(shared);

        world.resources.graphics.atmosphere = Atmosphere::Hdr;
    }

    fn save_shader_to_file(&mut self, source: &str) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let filename = format!("shader_{timestamp}.wgsl");
        match std::fs::write(&filename, source) {
            Ok(()) => {
                self.save_status = Some((format!("Saved: {filename}"), std::time::Instant::now()));
            }
            Err(error) => {
                self.save_status =
                    Some((format!("Save failed: {error}"), std::time::Instant::now()));
            }
        }
    }

    fn top_bar(&mut self, ui_context: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ui_context, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Shader Studio");
                ui.separator();

                let current_preset = &presets::PRESETS[self.selected_preset];
                ui.label(
                    egui::RichText::new(current_preset.name)
                        .strong()
                        .color(egui::Color32::from_rgb(0xCE, 0x91, 0x78)),
                )
                .on_hover_text(current_preset.description);

                let mut shared = self.shared.lock().unwrap();
                let mode_label = match shared.render_mode {
                    RenderMode::Fullscreen => "2D",
                    RenderMode::Geometry => "3D",
                };
                ui.label(
                    egui::RichText::new(mode_label)
                        .strong()
                        .color(egui::Color32::from_rgb(0x4E, 0xC9, 0xB0)),
                );

                ui.separator();

                if ui.button("Compile").clicked() {
                    let active = shared.active_tab;
                    if active == 5 {
                        for pass_index in 0..5 {
                            if shared.pass_enabled[pass_index] {
                                shared.pass_needs_recompile[pass_index] = true;
                            }
                        }
                    } else {
                        shared.pass_needs_recompile[active] = true;
                    }
                    self.source_dirty = [false; 5];
                    self.compile_timers = [0.0; 5];
                    self.common_dirty = false;
                    self.common_compile_timer = 0.0;
                }
                ui.checkbox(&mut self.auto_compile, "Auto");

                if ui.button("Copy").clicked() {
                    let source = shared.active_pass_source().to_string();
                    ui_context.copy_text(source);
                }

                if ui.button("Save").clicked() {
                    let source = shared.active_pass_source().to_string();
                    drop(shared);
                    self.save_shader_to_file(&source);
                    shared = self.shared.lock().unwrap();
                }

                ui.separator();

                let pause_icon = if shared.paused { ">" } else { "||" };
                if ui.button(pause_icon).clicked() {
                    shared.paused = !shared.paused;
                }
                if ui.button("Reset").clicked() {
                    self.accumulated_time = 0.0;
                    shared.time_offset = 0.0;
                }
                ui.add(
                    egui::DragValue::new(&mut shared.speed)
                        .range(0.0..=5.0)
                        .speed(0.01)
                        .prefix("Speed: ")
                        .max_decimals(2),
                );

                ui.separator();

                if ui.checkbox(&mut self.shuffle, "Shuffle").changed() && self.shuffle {
                    self.shuffle_timer = 0.0;
                }
                ui.toggle_value(&mut self.graph.visible, "Graph");

                ui.separator();
                ui.button(egui::RichText::new("?").strong())
                    .on_hover_ui(|ui| {
                        ui.heading("Keyboard Shortcuts");
                        ui.add_space(4.0);
                        egui::Grid::new("shortcuts_grid")
                            .min_col_width(100.0)
                            .show(ui, |ui| {
                                ui.monospace("Ctrl+Enter");
                                ui.label("Recompile active tab");
                                ui.end_row();
                                ui.monospace("Ctrl+S");
                                ui.label("Save active shader to file");
                                ui.end_row();
                                ui.monospace("Ctrl+Z");
                                ui.label("Undo preset change");
                                ui.end_row();
                                ui.monospace("Space");
                                ui.label("Pause / Resume time");
                                ui.end_row();
                                ui.monospace("R");
                                ui.label("Reset time to 0");
                                ui.end_row();
                                ui.monospace("Left/Right");
                                ui.label("Previous / Next preset");
                                ui.end_row();
                                ui.monospace("G");
                                ui.label("Toggle pipeline graph");
                                ui.end_row();
                                ui.monospace("F11");
                                ui.label("Toggle editor panel");
                                ui.end_row();
                            });
                        ui.add_space(4.0);
                        ui.heading("Multipass");
                        ui.label("Use tabs (Image, Buf A-D, Common) for multipass rendering.");
                        ui.label("Right-click buffer tabs to enable/disable them.");
                        ui.label("Configure channel inputs in the Channels section.");
                        ui.add_space(4.0);
                        ui.label("Drag & drop images (PNG, JPG, BMP, TGA) to load textures.");
                        ui.label("Drag & drop .glb / .gltf files to load 3D models.");
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let panel_label = if self.show_left_panel {
                        "Hide Editor"
                    } else {
                        "Show Editor"
                    };
                    if ui.button(panel_label).clicked() {
                        self.show_left_panel = !self.show_left_panel;
                    }
                });
            });
        });
    }

    fn bottom_bar(&self, ui_context: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_bar").show(ui_context, |ui| {
            ui.horizontal(|ui| {
                let shared = self.shared.lock().unwrap();

                let active = shared.active_tab;
                let (is_compiling, error) = if active == 5 {
                    let any_compiling = shared.pass_is_compiling.iter().any(|compiling| *compiling);
                    (any_compiling, shared.common_error.clone())
                } else {
                    (
                        shared.pass_is_compiling[active],
                        shared.pass_compilation_errors[active].clone(),
                    )
                };

                if is_compiling {
                    ui.colored_label(egui::Color32::YELLOW, "Compiling...");
                } else {
                    match &error {
                        Some(error) => {
                            ui.colored_label(egui::Color32::RED, "Error:");
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 150, 150),
                                truncate_error(error, 120),
                            );
                        }
                        None => {
                            ui.colored_label(egui::Color32::GREEN, "OK");
                        }
                    }
                }

                if let Some((message, instant)) = &self.save_status
                    && instant.elapsed().as_secs() < 3
                {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(0x4E, 0xC9, 0xB0), message);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Frame: {}", self.frame_count));
                    ui.separator();
                    ui.label(format!(
                        "{}x{}",
                        shared.uniforms.resolution[0] as u32, shared.uniforms.resolution[1] as u32
                    ));
                    ui.separator();
                    ui.label(format!("FPS: {:.0}", self.last_fps));
                });
            });
        });
    }

    fn left_panel(&mut self, ui_context: &egui::Context, world: &mut World) {
        egui::SidePanel::left("left_panel")
            .default_width(500.0)
            .min_width(300.0)
            .show(ui_context, |ui| {
                self.preset_tree(ui);
                ui.separator();

                self.tab_bar(ui);
                ui.separator();

                let editor_height = (ui.available_height() * 0.55).max(200.0);
                egui::ScrollArea::vertical()
                    .id_salt("code_editor")
                    .max_height(editor_height)
                    .show(ui, |ui| {
                        let mut shared = self.shared.lock().unwrap();
                        let active_tab = shared.active_tab;

                        let source = shared.active_pass_source_mut();

                        let line_count = source.lines().count().max(1);
                        let trailing_newline = source.ends_with('\n');
                        let display_lines = if trailing_newline {
                            line_count + 1
                        } else {
                            line_count
                        };
                        let digits = format!("{display_lines}").len();
                        let mut line_nums = String::with_capacity(display_lines * (digits + 1));
                        for line_num in 1..=display_lines {
                            if line_num > 1 {
                                line_nums.push('\n');
                            }
                            use std::fmt::Write;
                            write!(line_nums, "{line_num:>width$}", width = digits).ok();
                        }

                        let mut layouter =
                            |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                                let layout_job =
                                    syntax::highlight_wgsl(ui, text.as_str(), wrap_width);
                                ui.painter().layout_job(layout_job)
                            };

                        let changed = ui
                            .horizontal_top(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&line_nums)
                                            .monospace()
                                            .color(egui::Color32::from_rgb(0x60, 0x60, 0x60)),
                                    )
                                    .selectable(false),
                                );

                                ui.add_space(4.0);

                                let response = ui.add(
                                    egui::TextEdit::multiline(source)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(30)
                                        .layouter(&mut layouter),
                                );

                                response.changed()
                            })
                            .inner;

                        if changed {
                            if active_tab == 5 {
                                self.common_dirty = true;
                                self.common_compile_timer = 0.0;
                            } else {
                                self.source_dirty[active_tab] = true;
                                self.compile_timers[active_tab] = 0.0;
                            }
                        }

                        if ui.input(|input| {
                            input.key_pressed(egui::Key::Enter) && input.modifiers.ctrl
                        }) {
                            if active_tab == 5 {
                                for pass_index in 0..5 {
                                    if shared.pass_enabled[pass_index] {
                                        shared.pass_needs_recompile[pass_index] = true;
                                    }
                                }
                                self.common_dirty = false;
                                self.common_compile_timer = 0.0;
                            } else {
                                shared.pass_needs_recompile[active_tab] = true;
                                self.source_dirty[active_tab] = false;
                                self.compile_timers[active_tab] = 0.0;
                            }
                        }
                    });

                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("controls")
                    .show(ui, |ui| {
                        {
                            let shared = self.shared.lock().unwrap();
                            let active = shared.active_tab;
                            if active < 5 {
                                drop(shared);
                                self.channels_section(ui);
                                ui.add_space(8.0);
                            }
                        }
                        self.sdf_editor_section(ui);
                        ui.add_space(8.0);
                        self.uniforms_section(ui);
                        ui.add_space(8.0);
                        self.textures_section(ui);
                        ui.add_space(8.0);
                        self.geometry_section(ui, world);
                        ui.add_space(8.0);
                        self.error_details_section(ui);
                    });
            });
    }

    fn tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let mut shared = self.shared.lock().unwrap();
            let active = shared.active_tab;

            let tab_labels = ["Image", "Buf A", "Buf B", "Buf C", "Buf D", "Common"];

            for (tab_index, label) in tab_labels.iter().enumerate() {
                let is_active = active == tab_index;
                let is_buffer = (1..=4).contains(&tab_index);

                let has_error = if tab_index < 5 {
                    shared.pass_compilation_errors[tab_index].is_some()
                } else {
                    shared.common_error.is_some()
                };

                let is_enabled = if tab_index == 0 || tab_index == 5 {
                    true
                } else {
                    shared.pass_enabled[tab_index]
                };

                let is_compiling = if tab_index < 5 {
                    shared.pass_is_compiling[tab_index]
                } else {
                    false
                };

                let mut text = egui::RichText::new(*label).strong();
                if !is_enabled {
                    text = text.color(egui::Color32::from_rgb(0x60, 0x60, 0x60));
                } else if has_error {
                    text = text.color(egui::Color32::from_rgb(0xFF, 0x66, 0x66));
                } else if is_compiling {
                    text = text.color(egui::Color32::YELLOW);
                } else if is_active {
                    text = text.color(egui::Color32::WHITE);
                }

                let button = ui.selectable_label(is_active, text);

                if button.clicked() {
                    shared.active_tab = tab_index;
                }

                if is_buffer {
                    button.context_menu(|ui| {
                        let mut enabled = shared.pass_enabled[tab_index];
                        if ui.checkbox(&mut enabled, "Enabled").changed() {
                            shared.pass_enabled[tab_index] = enabled;
                            if enabled && !shared.pass_sources[tab_index].is_empty() {
                                shared.pass_needs_recompile[tab_index] = true;
                            }
                        }
                    });
                }
            }
        });
    }

    fn channels_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Channels")
            .default_open(false)
            .show(ui, |ui| {
                let mut shared = self.shared.lock().unwrap();
                let active = shared.active_tab;
                if active >= 5 {
                    return;
                }

                for channel in 0..4 {
                    ui.horizontal(|ui| {
                        ui.label(format!("Channel {channel}:"));
                        let current = shared.channel_bindings[active][channel];
                        egui::ComboBox::from_id_salt(format!("channel_{active}_{channel}"))
                            .selected_text(current.label())
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                for source in &ChannelSource::ALL {
                                    if ui
                                        .selectable_value(
                                            &mut shared.channel_bindings[active][channel],
                                            *source,
                                            source.label(),
                                        )
                                        .changed()
                                    {
                                        shared.channels_dirty = true;
                                    }
                                }
                            });
                    });
                }
            });
    }

    fn uniforms_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Uniforms")
            .default_open(true)
            .show(ui, |ui| {
                let shared = self.shared.lock().unwrap();
                ui.horizontal(|ui| {
                    ui.label(format!("Time: {:.2}s", shared.uniforms.time));
                    ui.label(format!("  Frame: {}", shared.uniforms.frame));
                });
                drop(shared);

                ui.add_space(4.0);
                let visible_count = if self.show_all_sliders {
                    16
                } else {
                    self.preset_slider_count
                };
                for index in 0..visible_count {
                    let (range_min, range_max) = self.slider_ranges[index];
                    ui.horizontal(|ui| {
                        ui.label(&self.slider_labels[index]);
                        let slider_response = ui.add(
                            egui::Slider::new(
                                &mut self.custom_sliders[index],
                                range_min..=range_max,
                            )
                            .max_decimals(3),
                        );
                        slider_response.context_menu(|ui| {
                            ui.label("Slider Range");
                            ui.separator();
                            for (label, min, max) in SLIDER_RANGE_PRESETS {
                                if ui.button(*label).clicked() {
                                    self.slider_ranges[index] = (*min, *max);
                                    self.custom_sliders[index] =
                                        self.custom_sliders[index].clamp(*min, *max);
                                    ui.close();
                                }
                            }
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Min:");
                                ui.add(
                                    egui::DragValue::new(&mut self.slider_ranges[index].0)
                                        .speed(0.1)
                                        .max_decimals(2),
                                );
                                ui.label("Max:");
                                ui.add(
                                    egui::DragValue::new(&mut self.slider_ranges[index].1)
                                        .speed(0.1)
                                        .max_decimals(2),
                                );
                            });
                        });
                    });
                }
                ui.horizontal(|ui| {
                    let toggle_label = if self.show_all_sliders {
                        "Show fewer"
                    } else {
                        "Show all 16 sliders"
                    };
                    if ui.small_button(toggle_label).clicked() {
                        self.show_all_sliders = !self.show_all_sliders;
                    }
                    if ui.small_button("Reset to defaults").clicked() {
                        let preset = &presets::PRESETS[self.selected_preset];
                        self.custom_sliders = [
                            0.7, 0.3, 0.2, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                            0.0, 0.0,
                        ];
                        self.slider_ranges = [(0.0, 1.0); 16];
                        for slider_index in 0..16 {
                            self.slider_labels[slider_index] = format!("Custom {slider_index}");
                        }
                        for &(slider_index, label) in preset.slider_labels {
                            if slider_index < 16 {
                                self.slider_labels[slider_index] = label.to_string();
                            }
                        }
                        for &(slider_index, value) in preset.slider_defaults {
                            if slider_index < 16 {
                                self.custom_sliders[slider_index] = value;
                            }
                        }
                    }
                });
            });
    }

    fn textures_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Textures")
            .default_open(true)
            .show(ui, |ui| {
                let mut shared = self.shared.lock().unwrap();
                for slot in 0..4 {
                    ui.horizontal(|ui| {
                        ui.label(format!("[{slot}]"));
                        match &shared.texture_slot_names[slot] {
                            Some(name) => {
                                ui.label(name.as_str());
                                if ui.small_button("X").clicked() {
                                    shared.texture_slot_names[slot] = None;
                                    shared.clear_texture_slot = Some(slot);
                                }
                            }
                            None => {
                                ui.colored_label(egui::Color32::GRAY, "Drop image here");
                            }
                        }
                    });
                }
                ui.label("Access as texture_0..texture_3 / sampler_0..sampler_3 in WGSL");
            });
    }

    fn geometry_section(&mut self, ui: &mut egui::Ui, world: &mut World) {
        egui::CollapsingHeader::new("Geometry")
            .default_open(true)
            .show(ui, |ui| {
                let mut shared = self.shared.lock().unwrap();
                ui.horizontal_wrapped(|ui| {
                    for primitive in PrimitiveType::ALL {
                        let selected = shared.primitive_type == *primitive;
                        if ui.selectable_label(selected, primitive.label()).clicked() && !selected {
                            shared.primitive_type = *primitive;
                            shared.geometry_dirty = true;
                        }
                    }
                    if let Some(name) = &shared.custom_mesh_name {
                        let selected = shared.primitive_type == PrimitiveType::Custom;
                        if ui.selectable_label(selected, name.as_str()).clicked() && !selected {
                            shared.primitive_type = PrimitiveType::Custom;
                            shared.upload_custom_mesh = true;
                        }
                    }
                });

                drop(shared);

                ui.horizontal(|ui| {
                    ui.label("Atmosphere:");
                    egui::ComboBox::from_id_salt("atmosphere_selector")
                        .selected_text(format!("{:?}", world.resources.graphics.atmosphere))
                        .width(120.0)
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

                ui.horizontal(|ui| {
                    ui.checkbox(&mut world.resources.graphics.bloom_enabled, "Bloom");
                    ui.checkbox(&mut world.resources.graphics.ssao_enabled, "SSAO");
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

                ui.label(
                    egui::RichText::new("Drop a .glb / .gltf file to load custom geometry")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            });
    }

    fn sdf_editor_section(&mut self, ui: &mut egui::Ui) {
        if presets::PRESETS[self.selected_preset].name != "SDF Editor" {
            return;
        }

        egui::CollapsingHeader::new(
            egui::RichText::new("SDF Scene Editor")
                .strong()
                .color(egui::Color32::from_rgb(0x4E, 0xC9, 0xB0)),
        )
        .default_open(true)
        .show(ui, |ui| {
            if self.sdf_editor.ui(ui) {
                let generated = self.sdf_editor.generate_full_shader().to_string();
                let mut shared = self.shared.lock().unwrap();
                shared.pass_sources[0] = generated;
                shared.pass_needs_recompile[0] = true;
            }
        });
    }

    fn error_details_section(&self, ui: &mut egui::Ui) {
        let shared = self.shared.lock().unwrap();
        let active = shared.active_tab;
        let error = if active == 5 {
            shared.common_error.as_ref()
        } else {
            shared.pass_compilation_errors[active].as_ref()
        };
        if let Some(error) = error {
            egui::CollapsingHeader::new("Compilation Error")
                .default_open(true)
                .show(ui, |ui| {
                    if let Some((line, col)) = parse_error_location(error) {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(0xC5, 0x86, 0xC0),
                                format!("Line {line}, Column {col}"),
                            );
                        });
                        ui.add_space(4.0);
                    }

                    let first_line = error.lines().next().unwrap_or(error);
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100),
                        egui::RichText::new(first_line).strong(),
                    );

                    let remaining: String = error.lines().skip(1).collect::<Vec<_>>().join("\n");
                    if !remaining.is_empty() {
                        ui.add_space(2.0);
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(&remaining)
                                    .monospace()
                                    .color(egui::Color32::from_rgb(200, 150, 150))
                                    .size(11.0),
                            )
                            .wrap(),
                        );
                    }
                });
        }
    }

    fn preset_tree(&mut self, ui: &mut egui::Ui) {
        let selected_category = presets::PRESETS[self.selected_preset].category;
        let filter_active = !self.preset_filter.is_empty();
        let filter_lower = self.preset_filter.to_lowercase();

        egui::CollapsingHeader::new(egui::RichText::new("Presets").strong())
            .default_open(true)
            .show(ui, |ui| {
                let all_categories: Vec<&str> = {
                    let mut cats = Vec::new();
                    for preset in presets::PRESETS.iter() {
                        if !cats.contains(&preset.category) {
                            cats.push(preset.category);
                        }
                    }
                    cats
                };

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.spacing_mut().button_padding = egui::vec2(6.0, 2.0);

                    let all_selected = self.category_filter.is_none();
                    let all_button =
                        ui.selectable_label(all_selected, egui::RichText::new("All").small());
                    if all_button.clicked() {
                        self.category_filter = None;
                    }

                    for category in &all_categories {
                        let is_active = self
                            .category_filter
                            .as_deref()
                            .is_some_and(|filter| filter == *category);
                        let count = presets::PRESETS
                            .iter()
                            .filter(|preset| preset.category == *category)
                            .count();
                        let label = format!("{category} ({count})");
                        let button =
                            ui.selectable_label(is_active, egui::RichText::new(label).small());
                        if button.clicked() {
                            if is_active {
                                self.category_filter = None;
                            } else {
                                self.category_filter = Some(category.to_string());
                            }
                        }
                    }
                });

                ui.add_space(2.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.preset_filter)
                        .hint_text("Search...")
                        .desired_width(f32::INFINITY),
                );

                egui::ScrollArea::vertical()
                    .id_salt("preset_tree_scroll")
                    .max_height(300.0)
                    .show(ui, |ui| {
                        let mut categories: Vec<(&str, Vec<(usize, &presets::ShaderPreset)>)> =
                            Vec::new();

                        for (index, preset) in presets::PRESETS.iter().enumerate() {
                            if let Some(ref cat_filter) = self.category_filter
                                && preset.category != cat_filter.as_str()
                            {
                                continue;
                            }

                            if filter_active
                                && !preset.name.to_lowercase().contains(&filter_lower)
                                && !preset.description.to_lowercase().contains(&filter_lower)
                            {
                                continue;
                            }

                            if let Some(entry) = categories
                                .iter_mut()
                                .find(|(cat, _)| *cat == preset.category)
                            {
                                entry.1.push((index, preset));
                            } else {
                                categories.push((preset.category, vec![(index, preset)]));
                            }
                        }

                        for (category, category_presets) in &categories {
                            let single_category = self.category_filter.is_some();
                            let should_open = if filter_active || single_category {
                                true
                            } else {
                                *category == selected_category
                            };
                            egui::CollapsingHeader::new(
                                egui::RichText::new(*category)
                                    .color(egui::Color32::from_rgb(0x9C, 0xDC, 0xFE)),
                            )
                            .default_open(should_open)
                            .open(if filter_active || single_category {
                                Some(true)
                            } else {
                                None
                            })
                            .show(ui, |ui| {
                                for &(index, preset) in category_presets {
                                    let is_selected = index == self.selected_preset;
                                    let response = ui.selectable_label(is_selected, preset.name);
                                    if response.clicked() && !is_selected {
                                        self.apply_preset(index);
                                    }
                                    response.on_hover_text(preset.description);
                                }
                            });
                        }
                    });
            });
    }

    fn apply_preset(&mut self, index: usize) {
        self.selected_preset = index;
        let preset = &presets::PRESETS[index];
        let mut shared = self.shared.lock().unwrap();

        shared.compilation_generation += 1;

        self.source_history.push(shared.pass_sources[0].clone());
        if self.source_history.len() > 32 {
            self.source_history.remove(0);
        }

        shared.pass_sources[0] = preset.source.to_string();
        shared.pass_needs_recompile[0] = true;

        if let Some(buffer_a) = preset.buffer_a_source {
            shared.pass_sources[1] = buffer_a.to_string();
            shared.pass_enabled[1] = true;
            shared.pass_needs_recompile[1] = true;
        } else {
            shared.pass_sources[1].clear();
            shared.pass_enabled[1] = false;
        }
        if let Some(buffer_b) = preset.buffer_b_source {
            shared.pass_sources[2] = buffer_b.to_string();
            shared.pass_enabled[2] = true;
            shared.pass_needs_recompile[2] = true;
        } else {
            shared.pass_sources[2].clear();
            shared.pass_enabled[2] = false;
        }
        if let Some(buffer_c) = preset.buffer_c_source {
            shared.pass_sources[3] = buffer_c.to_string();
            shared.pass_enabled[3] = true;
            shared.pass_needs_recompile[3] = true;
        } else {
            shared.pass_sources[3].clear();
            shared.pass_enabled[3] = false;
        }
        if let Some(buffer_d) = preset.buffer_d_source {
            shared.pass_sources[4] = buffer_d.to_string();
            shared.pass_enabled[4] = true;
            shared.pass_needs_recompile[4] = true;
        } else {
            shared.pass_sources[4].clear();
            shared.pass_enabled[4] = false;
        }

        if let Some(common) = preset.common_source {
            shared.common_source = common.to_string();
        } else {
            shared.common_source.clear();
        }

        if let Some(bindings) = &preset.channel_bindings {
            shared.channel_bindings = *bindings;
            shared.channels_dirty = true;
        } else {
            shared.channel_bindings = [[ChannelSource::None; 4]; 5];
            shared.channels_dirty = true;
        }

        if preset.is_geometry && shared.render_mode != RenderMode::Geometry {
            shared.render_mode = RenderMode::Geometry;
        } else if !preset.is_geometry && shared.render_mode != RenderMode::Fullscreen {
            shared.render_mode = RenderMode::Fullscreen;
        }

        for slider_index in 0..16 {
            self.slider_labels[slider_index] = format!("Custom {slider_index}");
        }
        let mut max_labeled = 0;
        for &(slider_index, label) in preset.slider_labels {
            if slider_index < 16 {
                self.slider_labels[slider_index] = label.to_string();
                if slider_index + 1 > max_labeled {
                    max_labeled = slider_index + 1;
                }
            }
        }
        self.preset_slider_count = max_labeled;
        self.show_all_sliders = false;

        shared.active_tab = 0;
        shared.time_offset = 0.0;
        self.accumulated_time = 0.0;
        self.frame_count = 0;
        self.source_dirty = [false; 5];
        self.compile_timers = [0.0; 5];
        self.common_dirty = false;
        self.common_compile_timer = 0.0;

        self.custom_sliders = [
            0.7, 0.3, 0.2, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        self.slider_ranges = [(0.0, 1.0); 16];
        for &(slider_index, value) in preset.slider_defaults {
            if slider_index < 16 {
                self.custom_sliders[slider_index] = value;
            }
        }

        if shared.custom_mesh_name.is_none() {
            shared.texture_slot_names = [
                Some("gradient noise (built-in)".to_string()),
                Some("dot grid (built-in)".to_string()),
                Some("checkerboard (built-in)".to_string()),
                None,
            ];
            shared.pending_texture_data.push(PendingTexture {
                width: 256,
                height: 256,
                data: generate_builtin_texture_gradient(256, 256),
                slot: 0,
                srgb: true,
            });
            shared.pending_texture_data.push(PendingTexture {
                width: 256,
                height: 256,
                data: generate_builtin_texture_dots(256, 256),
                slot: 1,
                srgb: true,
            });
            shared.pending_texture_data.push(PendingTexture {
                width: 256,
                height: 256,
                data: generate_builtin_texture_checkerboard(256, 256),
                slot: 2,
                srgb: true,
            });
        }

        shared.common_error = None;
        for index in 0..5 {
            shared.pass_compilation_errors[index] = None;
        }

        if preset.name == "SDF Editor" {
            self.sdf_editor.dirty = true;
            let generated = self.sdf_editor.generate_full_shader().to_string();
            shared.pass_sources[0] = generated;
        }
    }
}

fn mat4_to_arrays(matrix: &nalgebra_glm::Mat4) -> [[f32; 4]; 4] {
    let slice = matrix.as_slice();
    [
        [slice[0], slice[1], slice[2], slice[3]],
        [slice[4], slice[5], slice[6], slice[7]],
        [slice[8], slice[9], slice[10], slice[11]],
        [slice[12], slice[13], slice[14], slice[15]],
    ]
}

fn truncate_error(error: &str, max_len: usize) -> &str {
    if error.len() <= max_len {
        error
    } else {
        let boundary = error
            .char_indices()
            .take_while(|(index, _)| *index < max_len)
            .last()
            .map(|(index, _)| index + 1)
            .unwrap_or(0);
        &error[..boundary]
    }
}

fn parse_error_location(error: &str) -> Option<(usize, usize)> {
    for line in error.lines() {
        let trimmed = line.trim();
        if let Some(wgsl_pos) = trimmed.find("wgsl:") {
            let after = &trimmed[wgsl_pos + 5..];
            let parts: Vec<&str> = after.splitn(3, ':').collect();
            if parts.len() >= 2
                && let (Ok(line_num), Ok(col_num)) =
                    (parts[0].parse::<usize>(), parts[1].trim().parse::<usize>())
            {
                return Some((line_num, col_num));
            }
        }
    }
    None
}

const SLIDER_RANGE_PRESETS: &[(&str, f32, f32)] = &[
    ("0 .. 1", 0.0, 1.0),
    ("-1 .. 1", -1.0, 1.0),
    ("0 .. 10", 0.0, 10.0),
    ("0 .. 100", 0.0, 100.0),
    ("-10 .. 10", -10.0, 10.0),
    ("0 .. 360", 0.0, 360.0),
];

fn hash_2d(x: f32, y: f32) -> f32 {
    let val = (x * 127.1 + y * 311.7).sin() * 43_758.547;
    val - val.floor()
}

fn noise_2d(x: f32, y: f32) -> f32 {
    let integer_x = x.floor();
    let integer_y = y.floor();
    let frac_x = x - integer_x;
    let frac_y = y - integer_y;

    let smooth_x = frac_x * frac_x * (3.0 - 2.0 * frac_x);
    let smooth_y = frac_y * frac_y * (3.0 - 2.0 * frac_y);

    let bottom_left = hash_2d(integer_x, integer_y);
    let bottom_right = hash_2d(integer_x + 1.0, integer_y);
    let top_left = hash_2d(integer_x, integer_y + 1.0);
    let top_right = hash_2d(integer_x + 1.0, integer_y + 1.0);

    let bottom = bottom_left + (bottom_right - bottom_left) * smooth_x;
    let top = top_left + (top_right - top_left) * smooth_x;

    bottom + (top - bottom) * smooth_y
}

fn fbm_2d(x: f32, y: f32, octaves: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut current_x = x;
    let mut current_y = y;

    for _ in 0..octaves {
        value += amplitude * noise_2d(current_x, current_y);
        current_x *= 2.0;
        current_y *= 2.0;
        amplitude *= 0.5;
    }

    value
}

fn generate_builtin_texture_gradient(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32;
            let v = y as f32 / height as f32;

            let noise_value = fbm_2d(u * 6.0, v * 6.0, 5);

            let warm = [0.95_f32, 0.55, 0.2];
            let cool = [0.15_f32, 0.35, 0.75];

            let r = (cool[0] + (warm[0] - cool[0]) * noise_value).clamp(0.0, 1.0);
            let g = (cool[1] + (warm[1] - cool[1]) * noise_value).clamp(0.0, 1.0);
            let b = (cool[2] + (warm[2] - cool[2]) * noise_value).clamp(0.0, 1.0);

            data.extend_from_slice(&[(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]);
        }
    }

    data
}

fn generate_builtin_texture_dots(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    let grid_size = 32.0_f32;

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32;
            let v = y as f32 / height as f32;

            let cell_x = (u * grid_size).floor();
            let cell_y = (v * grid_size).floor();
            let frac_x = (u * grid_size) - cell_x;
            let frac_y = (v * grid_size) - cell_y;

            let center_dist =
                ((frac_x - 0.5) * (frac_x - 0.5) + (frac_y - 0.5) * (frac_y - 0.5)).sqrt();
            let radius = 0.25 + 0.1 * hash_2d(cell_x, cell_y);
            let dot = if center_dist < radius { 1.0 } else { 0.0 };

            let hue = hash_2d(cell_x * 7.3, cell_y * 13.1);
            let (dot_r, dot_g, dot_b) = hue_to_rgb(hue);

            let bg_r = 0.12_f32;
            let bg_g = 0.12;
            let bg_b = 0.15;

            let r = (bg_r + (dot_r - bg_r) * dot).clamp(0.0, 1.0);
            let g = (bg_g + (dot_g - bg_g) * dot).clamp(0.0, 1.0);
            let b = (bg_b + (dot_b - bg_b) * dot).clamp(0.0, 1.0);

            data.extend_from_slice(&[(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]);
        }
    }

    data
}

fn generate_builtin_texture_checkerboard(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    let cells = 8u32;

    for y in 0..height {
        for x in 0..width {
            let cell_x = x * cells / width;
            let cell_y = y * cells / height;
            let is_white = (cell_x + cell_y).is_multiple_of(2);
            let value = if is_white { 255u8 } else { 0u8 };
            data.extend_from_slice(&[value, value, value, 255]);
        }
    }

    data
}

fn hue_to_rgb(hue: f32) -> (f32, f32, f32) {
    let r = (hue * 6.0 - 3.0).abs().clamp(0.0, 1.0);
    let g = (2.0 - (hue * 6.0 - 2.0).abs()).clamp(0.0, 1.0);
    let b = (2.0 - (hue * 6.0 - 4.0).abs()).clamp(0.0, 1.0);
    let saturation = 0.7;
    (
        1.0 - saturation + saturation * r,
        1.0 - saturation + saturation * g,
        1.0 - saturation + saturation * b,
    )
}

fn local_transform_to_mat4(transform: &LocalTransform) -> nalgebra_glm::Mat4 {
    let translation = nalgebra_glm::translation(&transform.translation);
    let rotation = nalgebra_glm::quat_to_mat4(&transform.rotation);
    let scale = nalgebra_glm::scaling(&transform.scale);
    translation * rotation * scale
}

fn collect_meshes_recursive(
    node: &PrefabNode,
    parent_transform: &nalgebra_glm::Mat4,
    meshes: &HashMap<String, Mesh>,
    combined_vertices: &mut Vec<ShaderVertex>,
    combined_indices: &mut Vec<u32>,
) {
    let world_transform = parent_transform * local_transform_to_mat4(&node.local_transform);

    if let Some(render_mesh) = &node.components.render_mesh {
        let mesh_name = &render_mesh.name;
        if let Some(mesh) = meshes.get(mesh_name.as_str()) {
            let base_index = combined_vertices.len() as u32;

            let upper_3x3: nalgebra_glm::Mat3 =
                world_transform.fixed_view::<3, 3>(0, 0).clone_owned();
            let normal_matrix = upper_3x3
                .try_inverse()
                .map(|inverse| inverse.transpose())
                .unwrap_or_else(nalgebra_glm::Mat3::identity);

            for vertex in &mesh.vertices {
                let position =
                    nalgebra_glm::vec3(vertex.position[0], vertex.position[1], vertex.position[2]);
                let transformed_position =
                    world_transform * nalgebra_glm::vec4(position.x, position.y, position.z, 1.0);

                let normal =
                    nalgebra_glm::vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]);
                let transformed_normal = normal_matrix * normal;
                let normal_length = nalgebra_glm::length(&transformed_normal);
                let final_normal = if normal_length > 1e-6 {
                    transformed_normal / normal_length
                } else {
                    nalgebra_glm::vec3(0.0, 1.0, 0.0)
                };

                combined_vertices.push(ShaderVertex {
                    position: [
                        transformed_position.x,
                        transformed_position.y,
                        transformed_position.z,
                    ],
                    normal: [final_normal.x, final_normal.y, final_normal.z],
                    uv: vertex.tex_coords,
                });
            }

            for index in &mesh.indices {
                combined_indices.push(base_index + index);
            }
        }
    }

    for child in &node.children {
        collect_meshes_recursive(
            child,
            &world_transform,
            meshes,
            combined_vertices,
            combined_indices,
        );
    }
}
