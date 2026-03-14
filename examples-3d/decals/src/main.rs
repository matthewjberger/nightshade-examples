use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::decal::Decal;
use nightshade::ecs::gpu_picking::GpuPickResult;
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::ecs::material::material_registry_insert;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::world::commands::{WorldCommand, despawn_recursive_immediate};
use nightshade::prelude::*;
use rand::Rng;

const HELMET_GLTF: &[u8] = include_bytes!("../../../assets/gltf/DamagedHelmet.glb");
const HDR_SKYBOX: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(DecalsDemo::default())
}

struct DecalsDemo {
    decal_entities: Vec<Entity>,
    current_decal_type: DecalType,
    emissive_strength: f32,
    decal_size: f32,
    show_controls: bool,
    show_preview: bool,
    use_awesomeface: bool,
    preview_lines_entity: Option<Entity>,
    last_pick_result: Option<GpuPickResult>,
    last_mouse_pos: (u32, u32),
}

impl Default for DecalsDemo {
    fn default() -> Self {
        Self {
            decal_entities: Vec::new(),
            current_decal_type: DecalType::default(),
            emissive_strength: 1.5,
            decal_size: 1.0,
            show_controls: true,
            show_preview: true,
            use_awesomeface: false,
            preview_lines_entity: None,
            last_pick_result: None,
            last_mouse_pos: (0, 0),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
enum DecalType {
    #[default]
    Spray,
    BulletHole,
    BloodSplatter,
    NeonSign,
    Graffiti,
    AwesomeFace,
}

impl DecalType {
    fn name(&self) -> &str {
        match self {
            DecalType::Spray => "Spray Paint",
            DecalType::BulletHole => "Bullet Hole",
            DecalType::BloodSplatter => "Blood Splatter",
            DecalType::NeonSign => "Neon Sign (Emissive)",
            DecalType::Graffiti => "Graffiti (Emissive)",
            DecalType::AwesomeFace => "Awesome Face",
        }
    }

    fn texture(&self) -> &str {
        match self {
            DecalType::AwesomeFace => "awesomeface",
            _ => "checkerboard",
        }
    }

    fn color(&self) -> [f32; 4] {
        match self {
            DecalType::Spray => [1.0, 0.2, 0.2, 0.9],
            DecalType::BulletHole => [0.2, 0.2, 0.2, 1.0],
            DecalType::BloodSplatter => [0.6, 0.0, 0.0, 0.8],
            DecalType::NeonSign => [0.0, 1.0, 1.0, 1.0],
            DecalType::Graffiti => [1.0, 0.0, 1.0, 1.0],
            DecalType::AwesomeFace => [1.0, 1.0, 1.0, 1.0],
        }
    }

    fn is_emissive(&self) -> bool {
        matches!(self, DecalType::NeonSign | DecalType::Graffiti)
    }

    fn next(&self) -> Self {
        match self {
            DecalType::Spray => DecalType::BulletHole,
            DecalType::BulletHole => DecalType::BloodSplatter,
            DecalType::BloodSplatter => DecalType::NeonSign,
            DecalType::NeonSign => DecalType::Graffiti,
            DecalType::Graffiti => DecalType::AwesomeFace,
            DecalType::AwesomeFace => DecalType::Spray,
        }
    }
}

impl State for DecalsDemo {
    fn title(&self) -> &str {
        "Decals Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::Hdr;
        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.01;

        load_hdr_skybox(world, HDR_SKYBOX.to_vec());

        self.emissive_strength = 1.5;
        self.decal_size = 1.0;
        self.show_controls = true;
        self.show_preview = true;

        load_procedural_textures(world);
        load_awesomeface_texture(world);

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 3.0, 0.0),
            6.0,
            0.5,
            0.3,
            "Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        let sun = spawn_sun(world);
        if let Some(light) = world.core.get_light_mut(sun) {
            light.cast_shadows = false;
            light.intensity = 3.0;
        }

        spawn_room(world);
        spawn_damaged_helmet(world);

        spawn_initial_decals(world, &mut self.decal_entities, self.emissive_strength);

        let preview_entity = world.spawn_entities(
            nightshade::ecs::LINES
                | nightshade::ecs::VISIBILITY
                | nightshade::ecs::GLOBAL_TRANSFORM,
            1,
        )[0];
        world.core.set_lines(preview_entity, Lines::default());
        world.core.set_visibility(
            preview_entity,
            nightshade::ecs::world::components::Visibility { visible: true },
        );
        world
            .core
            .set_global_transform(preview_entity, GlobalTransform::default());
        self.preview_lines_entity = Some(preview_entity);

        spawn_ui_text_with_properties(
            world,
            "Left Click: Place Decal | Tab: Cycle Type | R: Random | C: Clear | H: Help",
            Vec2::zeros(),
            TextProperties {
                font_size: 18.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                outline_width: 0.02,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        if !self.show_controls {
            return;
        }

        egui::Window::new("Decals Controls")
            .default_pos([10.0, 10.0])
            .default_width(250.0)
            .show(ui_context, |ui| {
                ui.heading("Current Decal Type");
                ui.label(self.current_decal_type.name());
                ui.separator();

                ui.label("Decal Size");
                ui.add(egui::Slider::new(&mut self.decal_size, 0.2..=5.0));

                if self.current_decal_type.is_emissive() {
                    ui.label("Emissive Strength");
                    ui.add(egui::Slider::new(&mut self.emissive_strength, 0.0..=10.0));
                }

                ui.separator();

                ui.checkbox(&mut self.show_preview, "Show Preview");
                ui.checkbox(&mut self.use_awesomeface, "Use Awesomeface Texture");

                ui.separator();

                ui.label(format!("Active Decals: {}", self.decal_entities.len()));

                if ui.button("Clear All Decals").clicked() {
                    for entity in self.decal_entities.drain(..) {
                        despawn_recursive_immediate(world, entity);
                    }
                }

                ui.separator();

                ui.heading("Bloom Settings");
                let mut bloom_enabled = world.resources.graphics.bloom_enabled;
                if ui.checkbox(&mut bloom_enabled, "Enable Bloom").changed() {
                    world.resources.graphics.bloom_enabled = bloom_enabled;
                }

                let mut bloom_intensity = world.resources.graphics.bloom_intensity;
                if ui
                    .add(egui::Slider::new(&mut bloom_intensity, 0.0..=2.0).text("Intensity"))
                    .changed()
                {
                    world.resources.graphics.bloom_intensity = bloom_intensity;
                }
            });
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);
        sync_text_meshes_system(world);

        if let Some(result) = world.resources.gpu_picking.take_result() {
            self.last_pick_result = Some(result);
        }

        let mouse_pos = world.resources.input.mouse.position;
        let current_mouse_pos = (mouse_pos.x as u32, mouse_pos.y as u32);
        if !world.resources.user_interface.hud_wants_pointer
            && current_mouse_pos != self.last_mouse_pos
        {
            world
                .resources
                .gpu_picking
                .request_pick(current_mouse_pos.0, current_mouse_pos.1);
            self.last_mouse_pos = current_mouse_pos;
        }

        if let Some(preview_entity) = self.preview_lines_entity {
            if self.show_preview {
                if let Some(ref pick) = self.last_pick_result {
                    let lines = create_decal_preview_lines(
                        pick.world_position,
                        pick.world_normal,
                        self.decal_size,
                    );
                    world.core.set_lines(preview_entity, lines);
                } else {
                    world.core.set_lines(preview_entity, Lines::new(vec![]));
                }
            } else {
                world.core.set_lines(preview_entity, Lines::new(vec![]));
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if key_state != KeyState::Pressed {
            return;
        }

        match key_code {
            KeyCode::Escape | KeyCode::KeyQ => {
                world.resources.window.should_exit = true;
            }
            KeyCode::Tab => {
                self.current_decal_type = self.current_decal_type.next();
            }
            KeyCode::KeyR => {
                spawn_random_decals(world, &mut self.decal_entities, 20, self.emissive_strength);
            }
            KeyCode::KeyC => {
                for entity in self.decal_entities.drain(..) {
                    despawn_recursive_immediate(world, entity);
                }
            }
            KeyCode::KeyH => {
                self.show_controls = !self.show_controls;
            }
            _ => {}
        }
    }

    fn on_mouse_input(&mut self, world: &mut World, state: ElementState, button: MouseButton) {
        if state != ElementState::Pressed || button != MouseButton::Left {
            return;
        }

        if world.resources.user_interface.hud_wants_pointer {
            return;
        }

        if let Some(ref pick) = self.last_pick_result {
            let texture_override = if self.use_awesomeface {
                Some("awesomeface")
            } else {
                None
            };
            let entity = spawn_decal_at(
                world,
                pick.world_position,
                pick.world_normal,
                self.current_decal_type,
                self.decal_size,
                self.emissive_strength,
                texture_override,
            );
            self.decal_entities.push(entity);
        }
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let decal_pass = passes::DecalPass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(decal_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

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

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.01);
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
}

fn load_awesomeface_texture(world: &mut World) {
    let png_data = include_bytes!("../../../assets/textures/awesomeface.png");
    if let Ok(img) = image::load_from_memory(png_data) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        world.queue_command(WorldCommand::LoadTexture {
            name: "awesomeface".to_string(),
            rgba_data: rgba.into_raw(),
            width,
            height,
        });
    }
}

fn create_decal_preview_lines(position: Vec3, normal: Vec3, size: f32) -> Lines {
    let half_size = size / 2.0;
    let depth = size;

    let rotation = rotation_from_normal(normal);
    let rot_matrix = nalgebra_glm::quat_to_mat3(&rotation);

    let local_corners = [
        Vec3::new(-half_size, -half_size, 0.0),
        Vec3::new(half_size, -half_size, 0.0),
        Vec3::new(half_size, half_size, 0.0),
        Vec3::new(-half_size, half_size, 0.0),
    ];

    let local_corners_back = [
        Vec3::new(-half_size, -half_size, depth),
        Vec3::new(half_size, -half_size, depth),
        Vec3::new(half_size, half_size, depth),
        Vec3::new(-half_size, half_size, depth),
    ];

    let transform_point = |local: Vec3| -> Vec3 {
        let rotated = rot_matrix * local;
        position + rotated
    };

    let front: Vec<Vec3> = local_corners.iter().map(|&p| transform_point(p)).collect();
    let back: Vec<Vec3> = local_corners_back
        .iter()
        .map(|&p| transform_point(p))
        .collect();

    let color = Vec4::new(0.0, 1.0, 0.0, 1.0);

    let mut lines_vec = Vec::new();

    for index in 0..4 {
        let next = (index + 1) % 4;
        lines_vec.push(Line {
            start: front[index],
            end: front[next],
            color,
        });
    }

    for index in 0..4 {
        let next = (index + 1) % 4;
        lines_vec.push(Line {
            start: back[index],
            end: back[next],
            color,
        });
    }

    for index in 0..4 {
        lines_vec.push(Line {
            start: front[index],
            end: back[index],
            color,
        });
    }

    let center_front = position;
    let center_back = transform_point(Vec3::new(0.0, 0.0, depth * 2.0));
    lines_vec.push(Line {
        start: center_front,
        end: center_back,
        color: Vec4::new(1.0, 1.0, 0.0, 1.0),
    });

    Lines::new(lines_vec)
}

fn spawn_damaged_helmet(world: &mut World) {
    match nightshade::ecs::prefab::import_gltf_from_bytes(HELMET_GLTF) {
        Ok(result) => {
            for (name, (rgba_data, width, height)) in result.textures {
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

            for prefab in result.prefabs {
                nightshade::ecs::prefab::spawn_prefab(world, &prefab, Vec3::new(0.0, 2.0, 0.0));
            }
        }
        Err(e) => {
            tracing::error!("Failed to load DamagedHelmet: {}", e);
        }
    }
}

fn spawn_room(world: &mut World) {
    let floor_material = Material {
        base_color: [0.15, 0.12, 0.1, 1.0],
        roughness: 0.85,
        metallic: 0.0,
        ..Default::default()
    };
    spawn_mesh_with_material(
        world,
        "Cube",
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(20.0, 1.0, 20.0),
        floor_material,
    );

    let back_wall_material = Material {
        base_color: [0.85, 0.75, 0.65, 1.0],
        roughness: 0.7,
        metallic: 0.0,
        ..Default::default()
    };
    spawn_mesh_with_material(
        world,
        "Cube",
        Vec3::new(0.0, 5.0, -10.0),
        Vec3::new(20.0, 10.0, 0.5),
        back_wall_material,
    );

    let front_wall_material = Material {
        base_color: [0.7, 0.8, 0.75, 1.0],
        roughness: 0.7,
        metallic: 0.0,
        ..Default::default()
    };
    spawn_mesh_with_material(
        world,
        "Cube",
        Vec3::new(0.0, 5.0, 10.0),
        Vec3::new(20.0, 10.0, 0.5),
        front_wall_material,
    );

    let left_wall_material = Material {
        base_color: [0.75, 0.55, 0.55, 1.0],
        roughness: 0.7,
        metallic: 0.0,
        ..Default::default()
    };
    spawn_mesh_with_material(
        world,
        "Cube",
        Vec3::new(-10.0, 5.0, 0.0),
        Vec3::new(0.5, 10.0, 20.0),
        left_wall_material,
    );

    let right_wall_material = Material {
        base_color: [0.55, 0.65, 0.75, 1.0],
        roughness: 0.7,
        metallic: 0.0,
        ..Default::default()
    };
    spawn_mesh_with_material(
        world,
        "Cube",
        Vec3::new(10.0, 5.0, 0.0),
        Vec3::new(0.5, 10.0, 20.0),
        right_wall_material,
    );

    let pillar_material = Material {
        base_color: [0.6, 0.55, 0.5, 1.0],
        roughness: 0.5,
        metallic: 0.3,
        ..Default::default()
    };

    let pillar_positions = [
        Vec3::new(-5.0, 3.0, -5.0),
        Vec3::new(5.0, 3.0, -5.0),
        Vec3::new(-5.0, 3.0, 5.0),
        Vec3::new(5.0, 3.0, 5.0),
    ];

    for pos in pillar_positions {
        spawn_mesh_with_material(
            world,
            "Cube",
            pos,
            Vec3::new(1.0, 6.0, 1.0),
            pillar_material.clone(),
        );
    }

    let sphere_material = Material {
        base_color: [0.8, 0.3, 0.3, 1.0],
        roughness: 0.4,
        metallic: 0.1,
        ..Default::default()
    };
    spawn_mesh_with_material(
        world,
        "Sphere",
        Vec3::new(-3.0, 2.0, 0.0),
        Vec3::new(2.0, 2.0, 2.0),
        sphere_material,
    );

    let cylinder_material = Material {
        base_color: [0.3, 0.5, 0.8, 1.0],
        roughness: 0.3,
        metallic: 0.2,
        ..Default::default()
    };
    spawn_mesh_with_material(
        world,
        "Cylinder",
        Vec3::new(3.0, 2.0, 0.0),
        Vec3::new(1.5, 3.0, 1.5),
        cylinder_material,
    );
}

fn spawn_initial_decals(
    world: &mut World,
    decal_entities: &mut Vec<Entity>,
    emissive_strength: f32,
) {
    let decal = spawn_decal_at(
        world,
        Vec3::new(0.0, 3.0, -9.7),
        Vec3::new(0.0, 0.0, 1.0),
        DecalType::NeonSign,
        2.0,
        emissive_strength,
        None,
    );
    decal_entities.push(decal);

    let decal = spawn_decal_at(
        world,
        Vec3::new(-3.0, 2.0, -9.7),
        Vec3::new(0.0, 0.0, 1.0),
        DecalType::Spray,
        1.5,
        emissive_strength,
        None,
    );
    decal_entities.push(decal);

    let decal = spawn_decal_at(
        world,
        Vec3::new(3.0, 4.0, -9.7),
        Vec3::new(0.0, 0.0, 1.0),
        DecalType::BulletHole,
        0.5,
        emissive_strength,
        None,
    );
    decal_entities.push(decal);

    let decal = spawn_decal_at(
        world,
        Vec3::new(-9.7, 2.5, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        DecalType::Graffiti,
        3.0,
        emissive_strength,
        None,
    );
    decal_entities.push(decal);

    let decal = spawn_decal_at(
        world,
        Vec3::new(2.0, 0.1, 2.0),
        Vec3::new(0.0, 1.0, 0.0),
        DecalType::BloodSplatter,
        1.2,
        emissive_strength,
        None,
    );
    decal_entities.push(decal);

    let decal = spawn_decal_at(
        world,
        Vec3::new(5.0, 3.0, -9.7),
        Vec3::new(0.0, 0.0, 1.0),
        DecalType::AwesomeFace,
        2.5,
        emissive_strength,
        Some("awesomeface"),
    );
    decal_entities.push(decal);
}

fn spawn_random_decals(
    world: &mut World,
    decal_entities: &mut Vec<Entity>,
    count: usize,
    emissive_strength: f32,
) {
    let mut rng = rand::rng();

    for _ in 0..count {
        let wall = rng.random_range(0..5);
        let (position, normal) = match wall {
            0 => {
                let x: f32 = rng.random_range(-9.0..9.0);
                let z: f32 = rng.random_range(-9.0..9.0);
                (Vec3::new(x, 0.1, z), Vec3::new(0.0, 1.0, 0.0))
            }
            1 => {
                let x: f32 = rng.random_range(-9.0..9.0);
                let y: f32 = rng.random_range(1.0..8.0);
                (Vec3::new(x, y, -9.7), Vec3::new(0.0, 0.0, 1.0))
            }
            2 => {
                let x: f32 = rng.random_range(-9.0..9.0);
                let y: f32 = rng.random_range(1.0..8.0);
                (Vec3::new(x, y, 9.7), Vec3::new(0.0, 0.0, -1.0))
            }
            3 => {
                let z: f32 = rng.random_range(-9.0..9.0);
                let y: f32 = rng.random_range(1.0..8.0);
                (Vec3::new(-9.7, y, z), Vec3::new(1.0, 0.0, 0.0))
            }
            _ => {
                let z: f32 = rng.random_range(-9.0..9.0);
                let y: f32 = rng.random_range(1.0..8.0);
                (Vec3::new(9.7, y, z), Vec3::new(-1.0, 0.0, 0.0))
            }
        };

        let decal_type = match rng.random_range(0..6) {
            0 => DecalType::Spray,
            1 => DecalType::BulletHole,
            2 => DecalType::BloodSplatter,
            3 => DecalType::NeonSign,
            4 => DecalType::Graffiti,
            _ => DecalType::AwesomeFace,
        };

        let size: f32 = rng.random_range(0.5..2.0);

        let texture_override = if matches!(decal_type, DecalType::AwesomeFace) {
            Some("awesomeface")
        } else {
            None
        };
        let entity = spawn_decal_at(
            world,
            position,
            normal,
            decal_type,
            size,
            emissive_strength,
            texture_override,
        );
        decal_entities.push(entity);
    }
}

fn spawn_decal_at(
    world: &mut World,
    position: Vec3,
    normal: Vec3,
    decal_type: DecalType,
    size: f32,
    emissive_strength: f32,
    texture_override: Option<&str>,
) -> Entity {
    let entity = world.spawn_entities(
        nightshade::ecs::DECAL
            | nightshade::ecs::LOCAL_TRANSFORM
            | nightshade::ecs::GLOBAL_TRANSFORM
            | nightshade::ecs::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::NAME,
        1,
    )[0];

    let rotation = rotation_from_normal(normal);

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
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
    world
        .core
        .set_name(entity, Name(format!("Decal_{}", entity.id)));

    let color = if texture_override.is_some() {
        [1.0, 1.0, 1.0, 1.0]
    } else {
        decal_type.color()
    };
    let emissive = if decal_type.is_emissive() {
        emissive_strength
    } else {
        0.0
    };
    let texture = texture_override.unwrap_or_else(|| decal_type.texture());

    world.core.set_decal(
        entity,
        Decal::new(texture)
            .with_size(size, size)
            .with_color(color)
            .with_emissive_strength(emissive)
            .with_depth(size),
    );

    entity
}

fn rotation_from_normal(normal: Vec3) -> Quat {
    let forward = Vec3::new(0.0, 0.0, -1.0);
    let normal = nalgebra_glm::normalize(&normal);

    if (normal - forward).norm() < 0.001 {
        return Quat::identity();
    }

    if (normal + forward).norm() < 0.001 {
        return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::new(0.0, 1.0, 0.0));
    }

    let axis = nalgebra_glm::cross(&forward, &normal);
    let axis = nalgebra_glm::normalize(&axis);
    let angle = nalgebra_glm::dot(&forward, &normal).acos();

    nalgebra_glm::quat_angle_axis(angle, &axis)
}

fn spawn_mesh_with_material(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material: Material,
) -> Entity {
    let entity = spawn_mesh(world, mesh_name, position, scale);
    let material_index = world.resources.material_registry.registry.entries.len();
    let material_name = format!("DecalDemoMaterial_{}", material_index);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        material,
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
    };
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name));
    entity
}
