use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use nightshade::run::RenderResources;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(InteriorMappingDemo::default())?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum MeshType {
    Box,
    Cylinder,
    Sphere,
}

struct InteriorMappingDemo {
    state_handle: passes::InteriorMappingStateHandle,
    textures: passes::InteriorMappingTextures,
    mesh_type: MeshType,
    grid_size: i32,
    spacing: f32,
}

impl Default for InteriorMappingDemo {
    fn default() -> Self {
        let state_handle = passes::create_interior_mapping_state();
        {
            let mut state = state_handle.write().unwrap();
            state.mesh_data = Some(passes::generate_cube_mesh());
            state.mesh_dirty = true;
            state.instances = build_instances(1, 1.5);
        }
        Self {
            state_handle,
            textures: load_textures(),
            mesh_type: MeshType::Box,
            grid_size: 1,
            spacing: 1.5,
        }
    }
}

fn load_tga_rgba(bytes: &[u8]) -> (Vec<u8>, u32, u32) {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Tga)
        .expect("Failed to load TGA");
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    (rgba.into_raw(), width, height)
}

fn load_textures() -> passes::InteriorMappingTextures {
    let (ceiling, ceiling_width, ceiling_height) =
        load_tga_rgba(include_bytes!("../assets/Ceiling256.tga"));
    let (floor, floor_width, floor_height) =
        load_tga_rgba(include_bytes!("../assets/Floor256.tga"));
    let (wall_xy, wall_xy_width, wall_xy_height) =
        load_tga_rgba(include_bytes!("../assets/Stones256.tga"));
    let (wall_zy, wall_zy_width, wall_zy_height) =
        load_tga_rgba(include_bytes!("../assets/StonesLight256.tga"));
    let (noise, noise_width, noise_height) = load_tga_rgba(include_bytes!("../assets/Noise.tga"));
    let (furniture, furniture_width, furniture_height) =
        load_tga_rgba(include_bytes!("../assets/FurniturePlane.tga"));
    let (exterior, exterior_width, exterior_height) =
        load_tga_rgba(include_bytes!("../assets/Windows.tga"));

    let (face_rt, _, _) = load_tga_rgba(include_bytes!("../assets/WindowsCubeMap_rt.tga"));
    let (face_lf, _, _) = load_tga_rgba(include_bytes!("../assets/WindowsCubeMap_lf.tga"));
    let (face_up, _, _) = load_tga_rgba(include_bytes!("../assets/WindowsCubeMap_up.tga"));
    let (face_dn, _, _) = load_tga_rgba(include_bytes!("../assets/WindowsCubeMap_dn.tga"));
    let (face_fr, _, _) = load_tga_rgba(include_bytes!("../assets/WindowsCubeMap_fr.tga"));
    let (face_bk, cubemap_face_size, _) =
        load_tga_rgba(include_bytes!("../assets/WindowsCubeMap_bk.tga"));

    passes::InteriorMappingTextures {
        ceiling,
        ceiling_width,
        ceiling_height,
        floor,
        floor_width,
        floor_height,
        wall_xy,
        wall_xy_width,
        wall_xy_height,
        wall_zy,
        wall_zy_width,
        wall_zy_height,
        noise,
        noise_width,
        noise_height,
        furniture,
        furniture_width,
        furniture_height,
        exterior,
        exterior_width,
        exterior_height,
        cubemap_faces: [face_rt, face_lf, face_up, face_dn, face_fr, face_bk],
        cubemap_face_size,
    }
}

fn build_instances(grid_size: i32, spacing: f32) -> Vec<passes::InteriorMappingInstance> {
    let mut instances = Vec::new();
    let offset = (grid_size - 1) as f32 * spacing * 0.5;
    for row in 0..grid_size {
        for col in 0..grid_size {
            let x = col as f32 * spacing - offset;
            let z = row as f32 * spacing - offset;
            let model_matrix = nalgebra_glm::translation(&nalgebra_glm::Vec3::new(x, 0.0, z));
            instances.push(passes::InteriorMappingInstance { model_matrix });
        }
    }
    instances
}

impl State for InteriorMappingDemo {
    fn title(&self) -> &str {
        "Interior Mapping"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            3.0,
            0.5,
            0.3,
            "Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        _device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let textures = self.textures.clone();
        let state_handle = self.state_handle.clone();

        let interior_pass = passes::InteriorMappingPass::new(
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Depth32Float,
            textures,
            state_handle,
        );

        let _ = graph.add_pass(
            Box::new(interior_pass),
            &[("color", resources.scene_color), ("depth", resources.depth)],
        );

        let postprocess_pass = passes::PostProcessPass::new(_device, surface_format, 0.0);
        let _ = graph.add_pass(
            Box::new(postprocess_pass),
            &[
                ("hdr", resources.scene_color),
                ("bloom", resources.scene_color),
                ("ssao", resources.ssao),
                ("output", resources.compute_output),
            ],
        );

        let swapchain_blit_pass =
            passes::BlitPass::new(_device, surface_format).with_name("swapchain_blit");
        let _ = graph.add_pass(
            Box::new(swapchain_blit_pass),
            &[
                ("input", resources.compute_output),
                ("output", resources.swapchain),
            ],
        );
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        let keyboard = &world.resources.input.keyboard;
        let delta = world.resources.window.timing.delta_time;
        let speed = 2.0 * delta;

        let mut state = self.state_handle.write().unwrap();

        if keyboard.is_key_pressed(KeyCode::KeyU) {
            state.wall_frequencies[0] += speed;
        }
        if keyboard.is_key_pressed(KeyCode::KeyJ) {
            state.wall_frequencies[0] = (state.wall_frequencies[0] - speed).max(0.1);
        }
        if keyboard.is_key_pressed(KeyCode::KeyI) {
            state.wall_frequencies[1] += speed;
        }
        if keyboard.is_key_pressed(KeyCode::KeyK) {
            state.wall_frequencies[1] = (state.wall_frequencies[1] - speed).max(0.1);
        }
        if keyboard.is_key_pressed(KeyCode::KeyO) {
            state.wall_frequencies[2] += speed;
        }
        if keyboard.is_key_pressed(KeyCode::KeyL) {
            state.wall_frequencies[2] = (state.wall_frequencies[2] - speed).max(0.1);
        }
        if keyboard.is_key_pressed(KeyCode::KeyY) {
            state.light_threshold = (state.light_threshold + speed * 0.5).min(1.0);
        }
        if keyboard.is_key_pressed(KeyCode::KeyH) {
            state.light_threshold = (state.light_threshold - speed * 0.5).max(0.0);
        }
        if keyboard.is_key_pressed(KeyCode::KeyT) {
            state.alpha_plane_distance += speed * 0.1;
        }
        if keyboard.is_key_pressed(KeyCode::KeyG) {
            state.alpha_plane_distance -= speed * 0.1;
        }
        if keyboard.is_key_pressed(KeyCode::Digit1) {
            state.displacement_strengths[0] += speed;
            state.displacement_strengths[2] += speed;
        }
        if keyboard.is_key_pressed(KeyCode::Digit2) {
            state.displacement_strengths[0] = (state.displacement_strengths[0] - speed).max(0.0);
            state.displacement_strengths[2] = (state.displacement_strengths[2] - speed).max(0.0);
        }

        let mut mesh_changed = false;
        for &(key_code, pressed) in &keyboard.frame_keys {
            if !pressed {
                continue;
            }
            match key_code {
                KeyCode::KeyN => {
                    self.mesh_type = match self.mesh_type {
                        MeshType::Box => MeshType::Cylinder,
                        MeshType::Cylinder => MeshType::Sphere,
                        MeshType::Sphere => MeshType::Box,
                    };
                    mesh_changed = true;
                }
                KeyCode::KeyM => {
                    self.mesh_type = match self.mesh_type {
                        MeshType::Box => MeshType::Sphere,
                        MeshType::Cylinder => MeshType::Box,
                        MeshType::Sphere => MeshType::Cylinder,
                    };
                    mesh_changed = true;
                }
                KeyCode::KeyV => {
                    self.grid_size = (self.grid_size + 1).min(10);
                    state.instances = build_instances(self.grid_size, self.spacing);
                }
                KeyCode::KeyB => {
                    self.grid_size = (self.grid_size - 1).max(1);
                    state.instances = build_instances(self.grid_size, self.spacing);
                }
                _ => {}
            }
        }

        if mesh_changed {
            state.mesh_data = Some(match self.mesh_type {
                MeshType::Box => passes::generate_cube_mesh(),
                MeshType::Cylinder => passes::generate_cylinder_mesh(32, 16),
                MeshType::Sphere => passes::generate_sphere_mesh(32, 16),
            });
            state.mesh_dirty = true;
        }
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Interior Mapping").show(ui_context, |ui| {
            let mut state = self.state_handle.write().unwrap();

            ui.heading("Wall Frequencies");
            ui.add(egui::Slider::new(&mut state.wall_frequencies[0], 0.1..=20.0).text("X (U/J)"));
            ui.add(egui::Slider::new(&mut state.wall_frequencies[1], 0.1..=20.0).text("Y (I/K)"));
            ui.add(egui::Slider::new(&mut state.wall_frequencies[2], 0.1..=20.0).text("Z (O/L)"));

            ui.separator();
            ui.add(
                egui::Slider::new(&mut state.light_threshold, 0.0..=1.0)
                    .text("Light Threshold (Y/H)"),
            );
            ui.add(
                egui::Slider::new(&mut state.alpha_plane_distance, -0.5..=0.5)
                    .text("Alpha Plane Distance (T/G)"),
            );
            ui.add(egui::Slider::new(&mut state.uv_multiplier, 0.1..=10.0).text("UV Multiplier"));

            ui.separator();
            ui.heading("Displacement");
            ui.add(
                egui::Slider::new(&mut state.displacement_strengths[0], 0.0..=5.0).text("X (1/2)"),
            );
            ui.add(
                egui::Slider::new(&mut state.displacement_strengths[2], 0.0..=5.0).text("Z (1/2)"),
            );

            ui.separator();
            let mesh_label = match self.mesh_type {
                MeshType::Box => "Box",
                MeshType::Cylinder => "Cylinder",
                MeshType::Sphere => "Sphere",
            };
            ui.label(format!("Mesh: {} (N/M to switch)", mesh_label));
            ui.label(format!(
                "Grid: {}x{} = {} instances (V/B)",
                self.grid_size,
                self.grid_size,
                self.grid_size * self.grid_size
            ));
        });
    }
}
