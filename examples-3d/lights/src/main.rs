use nightshade::ecs::material::material_registry_insert;
use nightshade::prelude::*;

const NUM_LIGHTS: usize = 1024;
const ARENA_SIZE: f32 = 100.0;
const NUM_PILLARS_PER_SIDE: usize = 8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(ManyLights::default())
}

#[derive(Default)]
struct ManyLights {
    light_entities: Vec<Entity>,
    light_spheres: Vec<Entity>,
    light_phases: Vec<f32>,
}

impl State for ManyLights {
    fn title(&self) -> &str {
        "Many Lights - Clustered Forward Rendering"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Space;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.3;

        let position = Vec3::new(60.0, 40.0, 60.0);
        let camera_entity = spawn_camera(world, position, "Main Camera".to_string());
        world.resources.active_camera = Some(camera_entity);

        spawn_ground(world);
        spawn_pillars(world);
        self.spawn_lights(world);

        spawn_ui_text_with_properties(
            world,
            format!(
                "{} point lights with clustered forward rendering\nWASD to move, Mouse to look, Escape to exit",
                NUM_LIGHTS
            ),
            Vec2::zeros(),
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
            "Created many lights demo with {} point lights using clustered forward rendering",
            NUM_LIGHTS
        );
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);
        self.update_lights(world);
    }
}

impl ManyLights {
    fn spawn_lights(&mut self, world: &mut World) {
        let colors = [
            Vec3::new(1.0, 0.3, 0.1),
            Vec3::new(0.1, 1.0, 0.3),
            Vec3::new(0.3, 0.1, 1.0),
            Vec3::new(1.0, 1.0, 0.1),
            Vec3::new(0.1, 1.0, 1.0),
            Vec3::new(1.0, 0.1, 1.0),
            Vec3::new(1.0, 0.5, 0.0),
            Vec3::new(0.0, 0.5, 1.0),
        ];

        self.light_entities.clear();
        self.light_spheres.clear();
        self.light_phases.clear();

        for index in 0..NUM_LIGHTS {
            let phase = (index as f32 / NUM_LIGHTS as f32) * std::f32::consts::TAU;
            let color_index = index % colors.len();
            let color = colors[color_index];

            let radius = (index as f32 / NUM_LIGHTS as f32) * (ARENA_SIZE * 0.8) + 5.0;
            let x = radius * phase.cos();
            let z = radius * phase.sin();
            let y = 2.0 + (index as f32 * 0.1) % 3.0;

            let initial_position = Vec3::new(x, y, z);

            let light_entity = world.spawn_entities(
                nightshade::ecs::world::NAME
                    | nightshade::ecs::world::LOCAL_TRANSFORM
                    | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
                    | nightshade::ecs::world::GLOBAL_TRANSFORM
                    | nightshade::ecs::world::LIGHT,
                1,
            )[0];

            world
                .core
                .set_name(light_entity, Name(format!("Light_{}", index)));
            world.core.set_local_transform(
                light_entity,
                LocalTransform {
                    translation: initial_position,
                    rotation: Quat::identity(),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
            );
            world
                .core
                .set_local_transform_dirty(light_entity, LocalTransformDirty);
            world
                .core
                .set_global_transform(light_entity, GlobalTransform::default());
            world.core.set_light(
                light_entity,
                Light {
                    light_type: LightType::Point,
                    color,
                    intensity: 3.0,
                    range: 10.0,
                    inner_cone_angle: 0.0,
                    outer_cone_angle: 0.0,
                    cast_shadows: false,
                    shadow_bias: 0.007,
                },
            );

            let sphere_material = Material {
                base_color: [0.0, 0.0, 0.0, 1.0],
                emissive_factor: [color.x * 3.0, color.y * 3.0, color.z * 3.0],
                unlit: true,
                ..Default::default()
            };

            let sphere_entity = spawn_mesh_with_material(
                world,
                "Sphere",
                initial_position,
                Vec3::new(0.2, 0.2, 0.2),
                sphere_material,
                index,
            );

            self.light_entities.push(light_entity);
            self.light_spheres.push(sphere_entity);
            self.light_phases.push(phase);
        }
    }

    fn update_lights(&mut self, world: &mut World) {
        let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

        for (index, &light_entity) in self.light_entities.iter().enumerate() {
            let base_phase = self.light_phases[index];
            let speed = 0.3 + (index as f32 * 0.01) % 0.2;
            let current_phase = base_phase + time * speed;

            let base_radius = (index as f32 / NUM_LIGHTS as f32) * (ARENA_SIZE * 0.8) + 5.0;
            let radius_variation = (time * 0.5 + base_phase).sin() * 3.0;
            let radius = base_radius + radius_variation;

            let x = radius * current_phase.cos();
            let z = radius * current_phase.sin();
            let base_y = 2.0 + (index as f32 * 0.1) % 3.0;
            let y = base_y + (time * 2.0 + base_phase * 2.0).sin() * 1.5;

            let new_position = Vec3::new(x, y, z);

            if let Some(transform) = world.core.get_local_transform(light_entity) {
                let mut new_transform = *transform;
                new_transform.translation = new_position;
                world.core.set_local_transform(light_entity, new_transform);
                world
                    .core
                    .set_local_transform_dirty(light_entity, LocalTransformDirty);
            }

            if let Some(&sphere_entity) = self.light_spheres.get(index)
                && let Some(transform) = world.core.get_local_transform(sphere_entity)
            {
                let mut new_transform = *transform;
                new_transform.translation = new_position;
                world.core.set_local_transform(sphere_entity, new_transform);
                world
                    .core
                    .set_local_transform_dirty(sphere_entity, LocalTransformDirty);
            }
        }
    }
}

fn spawn_ground(world: &mut World) {
    let ground_material = Material {
        base_color: [0.15, 0.15, 0.18, 1.0],
        roughness: 0.8,
        metallic: 0.0,
        ..Default::default()
    };

    spawn_mesh_with_material(
        world,
        "Cube",
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(ARENA_SIZE * 2.0, 1.0, ARENA_SIZE * 2.0),
        ground_material,
        1000,
    );
}

fn spawn_pillars(world: &mut World) {
    let spacing = (ARENA_SIZE * 2.0) / (NUM_PILLARS_PER_SIDE as f32 + 1.0);
    let start = -ARENA_SIZE + spacing;

    for row in 0..NUM_PILLARS_PER_SIDE {
        for col in 0..NUM_PILLARS_PER_SIDE {
            let x = start + col as f32 * spacing;
            let z = start + row as f32 * spacing;

            let height = 4.0 + ((row + col) as f32 * 0.5) % 4.0;

            let pillar_material = Material {
                base_color: [0.4, 0.35, 0.3, 1.0],
                roughness: 0.6,
                metallic: 0.1,
                ..Default::default()
            };

            spawn_mesh_with_material(
                world,
                "Cube",
                Vec3::new(x, height / 2.0, z),
                Vec3::new(1.0, height, 1.0),
                pillar_material,
                2000 + row * NUM_PILLARS_PER_SIDE + col,
            );
        }
    }
}

static MATERIAL_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn spawn_mesh_with_material(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material: Material,
    _hint: usize,
) -> Entity {
    let entity = spawn_mesh(world, mesh_name, position, scale);
    let material_index = MATERIAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let material_name = format!("ManyLightsMaterial_{}", material_index);
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

    if let Some(camera_name) = world.core.get_name_mut(camera) {
        *camera_name = Name(name);
    }

    if let Some(local_transform) = world.core.get_local_transform_mut(camera) {
        local_transform.translation = position;
        let pitch = nalgebra_glm::quat_angle_axis(-0.4, &Vec3::x_axis());
        let yaw = nalgebra_glm::quat_angle_axis(0.8, &Vec3::y_axis());
        local_transform.rotation = yaw * pitch;
    }

    if let Some(camera_component) = world.core.get_camera_mut(camera) {
        *camera_component = Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 45.0_f32.to_radians(),
                z_far: Some(500.0),
                z_near: 0.1,
            }),
            smoothing: Some(Smoothing::default()),
        };
    }

    camera
}
