use nightshade::ecs::material::material_registry_insert;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(BloomDemo::default())
}

#[derive(Default)]
struct BloomDemo {
    moving_lights: Vec<Entity>,
    light_spheres: Vec<Entity>,
}

impl State for BloomDemo {
    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.3;

        let position = Vec3::new(10.0, 10.0, 10.0);
        let camera_entity = spawn_camera(world, position, "Main Camera".to_string());
        world.resources.active_camera = Some(camera_entity);

        spawn_mesh_with_material(
            world,
            "Cube",
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(50.0, 0.1, 50.0),
            create_pastel_material(Vec3::new(0.8, 0.8, 0.8)),
        );

        spawn_mesh_with_material(
            world,
            "Cube",
            Vec3::new(0.0, 12.0, 0.0),
            Vec3::new(50.0, 0.1, 50.0),
            create_pastel_material(Vec3::new(0.7, 0.7, 0.7)),
        );

        spawn_mesh_with_material(
            world,
            "Cube",
            Vec3::new(0.0, 5.5, -25.0),
            Vec3::new(50.0, 13.0, 0.1),
            create_pastel_material(Vec3::new(0.3, 0.3, 0.3)),
        );

        spawn_mesh_with_material(
            world,
            "Cube",
            Vec3::new(0.0, 5.5, 25.0),
            Vec3::new(50.0, 13.0, 0.1),
            create_pastel_material(Vec3::new(0.75, 0.75, 0.8)),
        );

        spawn_mesh_with_material(
            world,
            "Cube",
            Vec3::new(25.0, 5.5, 0.0),
            Vec3::new(0.1, 13.0, 50.0),
            create_pastel_material(Vec3::new(0.8, 0.75, 0.75)),
        );

        spawn_mesh_with_material(
            world,
            "Cube",
            Vec3::new(-25.0, 5.5, 0.0),
            Vec3::new(0.1, 13.0, 50.0),
            create_pastel_material(Vec3::new(0.8, 0.75, 0.75)),
        );

        spawn_mesh_with_material(
            world,
            "Sphere",
            Vec3::new(-8.0, 3.0, -5.0),
            Vec3::new(1.5, 1.5, 1.5),
            create_pastel_material(Vec3::new(1.0, 0.8, 0.8)),
        );

        spawn_mesh_with_material(
            world,
            "Sphere",
            Vec3::new(6.0, 4.0, 2.0),
            Vec3::new(2.0, 2.0, 2.0),
            create_pastel_material(Vec3::new(0.8, 1.0, 0.8)),
        );

        spawn_mesh_with_material(
            world,
            "Sphere",
            Vec3::new(-2.0, 5.0, 8.0),
            Vec3::new(1.0, 1.0, 1.0),
            create_pastel_material(Vec3::new(0.8, 0.8, 1.0)),
        );

        spawn_mesh_with_material(
            world,
            "Sphere",
            Vec3::new(10.0, 2.5, -8.0),
            Vec3::new(1.8, 1.8, 1.8),
            create_pastel_material(Vec3::new(1.0, 1.0, 0.8)),
        );

        spawn_3d_text_with_properties(
            world,
            "Bloom",
            Vec3::new(0.0, 8.0, 0.0),
            TextProperties {
                font_size: 120.0,
                color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                outline_width: 0.02,
                outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                smoothing: 0.01,
                line_height: 1.0,
                letter_spacing: 0.0,
                vertical_alignment: VerticalAlignment::Middle,
                monospace_width: None,
                ..Default::default()
            },
        );

        self.moving_lights.clear();
        self.light_spheres.clear();

        let light_colors = [
            Vec3::new(1.0, 0.2, 0.2),
            Vec3::new(0.2, 1.0, 0.2),
            Vec3::new(0.2, 0.2, 1.0),
            Vec3::new(1.0, 1.0, 0.2),
            Vec3::new(1.0, 0.2, 1.0),
            Vec3::new(0.2, 1.0, 1.0),
        ];

        let num_lights = 30;
        let num_rings = 10;
        let lights_per_ring = num_lights / num_rings;
        let base_radius = 5.0;
        let max_radius = 20.0;
        let min_height = 0.5;
        let max_height = 8.0;

        for light_index in 0..num_lights {
            let ring_index = light_index / lights_per_ring;
            let light_in_ring = light_index % lights_per_ring;

            let ring_progress = ring_index as f32 / (num_rings - 1) as f32;
            let circle_radius = base_radius + (max_radius - base_radius) * ring_progress;
            let light_height = min_height
                + (max_height - min_height)
                    * ((ring_index as f32 / num_rings as f32) * 2.0 * std::f32::consts::PI)
                        .sin()
                        .abs();

            let angle =
                (light_in_ring as f32 / lights_per_ring as f32) * 2.0 * std::f32::consts::PI;

            let initial_position = Vec3::new(
                angle.cos() * circle_radius,
                light_height,
                angle.sin() * circle_radius,
            );

            let light_color = light_colors[light_index % light_colors.len()];

            let light_entity = world.spawn_entities(
                nightshade::ecs::world::NAME
                    | nightshade::ecs::world::LOCAL_TRANSFORM
                    | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
                    | nightshade::ecs::world::GLOBAL_TRANSFORM
                    | nightshade::ecs::world::LIGHT,
                1,
            )[0];

            world.set_name(light_entity, Name(format!("Moving Light {}", light_index)));
            world.set_local_transform(
                light_entity,
                LocalTransform {
                    translation: initial_position,
                    rotation: Quat::identity(),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
            );
            world.set_local_transform_dirty(light_entity, LocalTransformDirty);
            world.set_global_transform(light_entity, GlobalTransform::default());
            world.set_light(
                light_entity,
                Light {
                    light_type: LightType::Point,
                    color: light_color,
                    intensity: 3.0,
                    range: 20.0,
                    inner_cone_angle: 0.0,
                    outer_cone_angle: 0.0,
                    cast_shadows: false,
                    shadow_bias: 0.007,
                },
            );

            let sphere_material = Material {
                base_color: [0.0, 0.0, 0.0, 1.0],
                emissive_factor: [
                    light_color.x * 2.0,
                    light_color.y * 2.0,
                    light_color.z * 2.0,
                ],
                unlit: true,
                ..Default::default()
            };
            let sphere_entity = spawn_mesh_with_material(
                world,
                "Sphere",
                initial_position,
                Vec3::new(0.15, 0.15, 0.15),
                sphere_material,
            );

            self.moving_lights.push(light_entity);
            self.light_spheres.push(sphere_entity);
        }

        spawn_hud_text_with_properties(
            world,
            "WASD to move, Mouse to look, Escape to exit",
            HudAnchor::BottomCenter,
            Vec2::new(0.0, -20.0),
            TextProperties {
                font_size: 20.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                outline_width: 0.01,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        tracing::info!(
            "Created lighting demo scene with {} moving point lights",
            num_lights
        );
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);
        sync_text_meshes_system(world);

        let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
        let num_lights = self.moving_lights.len();
        let num_rings = 10;
        let lights_per_ring = num_lights / num_rings;
        let base_radius = 5.0;
        let max_radius = 20.0;
        let min_height = 0.5;
        let max_height = 8.0;
        let rotation_speed = 0.3;

        for (light_index, &light_entity) in self.moving_lights.iter().enumerate() {
            let ring_index = light_index / lights_per_ring;
            let light_in_ring = light_index % lights_per_ring;

            let ring_progress = ring_index as f32 / (num_rings - 1) as f32;
            let circle_radius = base_radius + (max_radius - base_radius) * ring_progress;
            let light_height = min_height
                + (max_height - min_height)
                    * ((ring_index as f32 / num_rings as f32) * 2.0 * std::f32::consts::PI)
                        .sin()
                        .abs();

            let ring_speed = rotation_speed * (1.0 + ring_progress * 0.5);
            let base_angle =
                (light_in_ring as f32 / lights_per_ring as f32) * 2.0 * std::f32::consts::PI;
            let current_angle = base_angle
                + time
                    * ring_speed
                    * if ring_index.is_multiple_of(2) {
                        1.0
                    } else {
                        -1.0
                    };

            let new_position = Vec3::new(
                current_angle.cos() * circle_radius,
                light_height,
                current_angle.sin() * circle_radius,
            );

            if let Some(transform) = world.get_local_transform(light_entity) {
                let mut new_transform = *transform;
                new_transform.translation = new_position;
                world.set_local_transform(light_entity, new_transform);
                world.set_local_transform_dirty(light_entity, LocalTransformDirty);
            }

            if let Some(&sphere_entity) = self.light_spheres.get(light_index)
                && let Some(transform) = world.get_local_transform(sphere_entity)
            {
                let mut new_transform = *transform;
                new_transform.translation = new_position;
                world.set_local_transform(sphere_entity, new_transform);
                world.set_local_transform_dirty(sphere_entity, LocalTransformDirty);
            }
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

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.3);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.swapchain);
    }
}

fn create_pastel_material(base_color: Vec3) -> Material {
    Material {
        base_color: [base_color.x, base_color.y, base_color.z, 1.0],
        emissive_factor: [0.0, 0.0, 0.0],
        alpha_mode: AlphaMode::Opaque,
        alpha_cutoff: 0.5,
        base_texture: None,
        emissive_texture: None,
        normal_texture: None,
        normal_scale: 1.0,
        normal_map_flip_y: false,
        normal_map_two_component: false,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        occlusion_strength: 1.0,
        roughness: 0.5,
        metallic: 0.0,
        unlit: false,
        uv_scale: [1.0, 1.0],
        transmission_factor: 0.0,
        transmission_texture: None,
        thickness: 0.0,
        thickness_texture: None,
        attenuation_color: [1.0, 1.0, 1.0],
        attenuation_distance: f32::INFINITY,
        ior: 1.5,
        specular_factor: 1.0,
        specular_color_factor: [1.0, 1.0, 1.0],
        specular_texture: None,
        specular_color_texture: None,
        emissive_strength: 1.0,
    }
}

static BLOOM_MATERIAL_COUNTER: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

fn spawn_mesh_with_material(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material: Material,
) -> Entity {
    let entity = spawn_mesh(world, mesh_name, position, scale);
    let material_index = BLOOM_MATERIAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let material_name = format!("BloomMaterial_{}", material_index);
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
    world.set_material_ref(entity, MaterialRef::new(material_name));
    entity
}

fn spawn_camera(world: &mut World, position: Vec3, name: String) -> Entity {
    let cameras = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::CAMERA,
        1,
    );

    let camera = cameras[0];

    if let Some(camera_name) = world.get_name_mut(camera) {
        *camera_name = Name(name);
    }

    if let Some(local_transform) = world.get_local_transform_mut(camera) {
        local_transform.translation = position;
    }

    if let Some(camera_component) = world.get_camera_mut(camera) {
        *camera_component = Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 45.0_f32.to_radians(),
                z_far: None,
                z_near: 0.01,
            }),
            smoothing: Some(Smoothing::default()),
        };
    }

    camera
}
