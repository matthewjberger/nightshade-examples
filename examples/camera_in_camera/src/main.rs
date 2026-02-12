use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::particles::components::{
    ColorGradient, EmitterShape, EmitterType, ParticleEmitter,
};
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use nightshade::run::RenderResources;

const RENDER_TEXTURE_WIDTH: u32 = 1024;
const RENDER_TEXTURE_HEIGHT: u32 = 768;
const RENDER_TEXTURE_NAME: &str = "movie_screen_render";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(DriveInDemo::default())
}

struct SecondaryWorld {
    world: World,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    egui_texture_id: Option<egui::TextureId>,
}

#[derive(Default)]
struct DriveInDemo {
    secondary: Option<SecondaryWorld>,
    initialized: bool,
    total_time: f32,
    secondary_cube_entities: Vec<Entity>,
    campfire_light_entity: Option<Entity>,
}

impl State for DriveInDemo {
    fn title(&self) -> &str {
        "Drive-In Movie Theater"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::Space;
        world.resources.graphics.clear_color = [0.01, 0.01, 0.03, 1.0];
        world.resources.graphics.bloom_enabled = true;

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.color = Vec3::new(0.25, 0.3, 0.45);
            light.intensity = 0.4;
            light.cast_shadows = true;
        }

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 1.5, -1.0),
            18.0,
            0.0,
            0.45,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        spawn_ground(world);
        spawn_movie_screen(world);
        spawn_screen_posts(world);
        spawn_log_benches(world);
        spawn_campfire_logs(world);
        spawn_campfire_particles(world);
        self.campfire_light_entity = Some(spawn_campfire_light(world));
        spawn_trees(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        let delta_time = world.resources.window.timing.delta_time;
        self.total_time += delta_time;

        update_particle_emitters(world, delta_time);
        update_campfire_light(world, self.campfire_light_entity);

        if let Some(secondary) = &mut self.secondary {
            secondary.world.resources.window.timing = world.resources.window.timing.clone();
            animate_secondary_cubes(
                &mut secondary.world,
                &self.secondary_cube_entities,
                self.total_time,
            );
            update_global_transforms_system(&mut secondary.world);
        }
    }

    fn pre_render(&mut self, renderer: &mut dyn Render, _main_world: &mut World) {
        if !self.initialized {
            self.initialized = true;
            self.secondary = Some(create_secondary_world(renderer));

            let secondary = self.secondary.as_mut().unwrap();
            self.secondary_cube_entities = spawn_secondary_scene(&mut secondary.world);
        }

        if let Some(secondary) = &mut self.secondary {
            if secondary.egui_texture_id.is_none() {
                secondary.egui_texture_id = renderer.register_egui_texture(&secondary.texture_view);
            }

            let _ = renderer.render_world_to_texture(
                &mut secondary.world,
                &secondary.texture_view,
                RENDER_TEXTURE_WIDTH,
                RENDER_TEXTURE_HEIGHT,
            );

            let view = secondary
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            renderer.register_render_texture(RENDER_TEXTURE_NAME, view);
        }
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let particle_pass = passes::ParticlePass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(particle_pass))
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
            .write("output", resources.swapchain);
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Drive-In Theater")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.label("Outdoor drive-in movie theater");
                ui.label("with a secondary world rendered to the screen.");
                ui.separator();
                ui.label("Controls:");
                ui.label("  Left-click drag: Orbit");
                ui.label("  Right-click drag: Pan");
                ui.label("  Scroll: Zoom");
                ui.label("  ESC: Exit");
            });

        if let Some(secondary) = &self.secondary
            && let Some(texture_id) = secondary.egui_texture_id
        {
            let margin = 16.0;
            let pip_width = 320.0;
            let pip_height =
                pip_width * (RENDER_TEXTURE_HEIGHT as f32 / RENDER_TEXTURE_WIDTH as f32);

            egui::Area::new(egui::Id::new("pip_overlay"))
                .anchor(egui::Align2::RIGHT_BOTTOM, [-margin, -margin])
                .order(egui::Order::Foreground)
                .show(ui_context, |ui| {
                    egui::Frame::new()
                        .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.image(egui::load::SizedTexture::new(
                                texture_id,
                                [pip_width, pip_height],
                            ));
                        });
                });
        }
    }
}

fn register_material(world: &mut World, name: &str, material: Material) {
    material_registry_insert(
        &mut world.resources.material_registry,
        name.to_string(),
        material,
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
}

struct ShadowEntityDesc<'a> {
    mesh: &'a str,
    position: Vec3,
    scale: Vec3,
    rotation: Quat,
    material_name: &'a str,
    material: Material,
    name: &'a str,
}

fn spawn_shadow_entity(world: &mut World, desc: ShadowEntityDesc) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | CASTS_SHADOW,
        1,
    )[0];
    world.set_name(entity, Name(desc.name.to_string()));
    world.set_local_transform(
        entity,
        LocalTransform {
            translation: desc.position,
            rotation: desc.rotation,
            scale: desc.scale,
        },
    );
    world.set_render_mesh(entity, RenderMesh::new(desc.mesh));
    register_material(world, desc.material_name, desc.material);
    world.set_material_ref(entity, MaterialRef::new(desc.material_name.to_string()));
    world.set_casts_shadow(entity, CastsShadow);
    entity
}

fn spawn_ground(world: &mut World) {
    spawn_shadow_entity(
        world,
        ShadowEntityDesc {
            mesh: "Cube",
            position: Vec3::new(0.0, -0.25, 0.0),
            scale: Vec3::new(40.0, 0.5, 40.0),
            rotation: Quat::identity(),
            material_name: "Grass",
            material: Material {
                base_color: [0.08, 0.18, 0.05, 1.0],
                roughness: 0.95,
                metallic: 0.0,
                ..Default::default()
            },
            name: "Ground",
        },
    );
}

fn spawn_movie_screen(world: &mut World) {
    let screen = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF,
        1,
    )[0];
    world.set_name(screen, Name("Movie Screen".to_string()));

    let rotation =
        nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::new(1.0, 0.0, 0.0));

    world.set_local_transform(
        screen,
        LocalTransform {
            translation: Vec3::new(0.0, 4.0, -10.0),
            rotation,
            scale: Vec3::new(3.5, 1.0, 2.625),
        },
    );
    world.set_render_mesh(screen, RenderMesh::new("Plane"));

    register_material(
        world,
        "ScreenMaterial",
        Material {
            base_color: [0.02, 0.02, 0.02, 1.0],
            emissive_factor: [1.0, 1.0, 1.0],
            emissive_texture: Some(RENDER_TEXTURE_NAME.to_string()),
            unlit: true,
            double_sided: true,
            ..Default::default()
        },
    );
    world.set_material_ref(screen, MaterialRef::new("ScreenMaterial".to_string()));
}

fn spawn_screen_posts(world: &mut World) {
    let post_material = Material {
        base_color: [0.25, 0.15, 0.08, 1.0],
        roughness: 0.9,
        metallic: 0.0,
        ..Default::default()
    };

    for (index, x) in [-3.6_f32, 3.6].iter().enumerate() {
        spawn_shadow_entity(
            world,
            ShadowEntityDesc {
                mesh: "Cylinder",
                position: Vec3::new(*x, 3.5, -10.0),
                scale: Vec3::new(0.15, 7.0, 0.15),
                rotation: Quat::identity(),
                material_name: &format!("ScreenPost_{}", index),
                material: post_material.clone(),
                name: &format!("Screen Post {}", index),
            },
        );
    }

    spawn_shadow_entity(
        world,
        ShadowEntityDesc {
            mesh: "Cube",
            position: Vec3::new(0.0, 7.1, -10.0),
            scale: Vec3::new(7.5, 0.2, 0.2),
            rotation: Quat::identity(),
            material_name: "ScreenCrossbar",
            material: post_material,
            name: "Screen Crossbar",
        },
    );
}

fn spawn_log_benches(world: &mut World) {
    let bench_material = Material {
        base_color: [0.35, 0.2, 0.1, 1.0],
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    };

    let stump_material = Material {
        base_color: [0.3, 0.18, 0.08, 1.0],
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    };

    let bench_rows: [(f32, f32); 3] = [(-4.0, 2.5), (-2.0, 3.0), (0.0, 3.5)];

    let roll_rotation =
        nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::new(0.0, 0.0, 1.0));

    let mut bench_index = 0;
    for (row_z, bench_width) in bench_rows {
        for x_offset in [-bench_width, bench_width] {
            spawn_shadow_entity(
                world,
                ShadowEntityDesc {
                    mesh: "Cylinder",
                    position: Vec3::new(x_offset, 0.45, row_z),
                    scale: Vec3::new(0.2, bench_width * 1.3, 0.2),
                    rotation: roll_rotation,
                    material_name: &format!("BenchLog_{}", bench_index),
                    material: bench_material.clone(),
                    name: &format!("Bench Log {}", bench_index),
                },
            );

            for (stump_index, stump_x) in [-bench_width * 0.5, bench_width * 0.5].iter().enumerate()
            {
                spawn_shadow_entity(
                    world,
                    ShadowEntityDesc {
                        mesh: "Cylinder",
                        position: Vec3::new(x_offset + stump_x, 0.2, row_z),
                        scale: Vec3::new(0.12, 0.4, 0.12),
                        rotation: Quat::identity(),
                        material_name: &format!("BenchStump_{}_{}", bench_index, stump_index),
                        material: stump_material.clone(),
                        name: &format!("Bench Stump {}_{}", bench_index, stump_index),
                    },
                );
            }

            bench_index += 1;
        }
    }
}

fn spawn_campfire_logs(world: &mut World) {
    let log_material = Material {
        base_color: [0.35, 0.2, 0.1, 1.0],
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    };

    let campfire = Vec3::new(0.0, 0.0, -1.0);

    let logs: [(f32, f32, f32, f32, f32); 6] = [
        (0.0, 0.15, 0.5, 0.0, std::f32::consts::FRAC_PI_2),
        (0.0, 0.15, -0.5, 0.0, std::f32::consts::FRAC_PI_2),
        (-0.4, 0.3, 0.0, std::f32::consts::FRAC_PI_4, 0.45),
        (0.4, 0.3, 0.0, -std::f32::consts::FRAC_PI_4, 0.45),
        (-0.2, 0.45, 0.2, std::f32::consts::FRAC_PI_6, 0.6),
        (0.2, 0.45, -0.2, -std::f32::consts::FRAC_PI_6, 0.6),
    ];

    for (index, (offset_x, offset_y, offset_z, yaw, pitch)) in logs.iter().enumerate() {
        let pitch_rot = nalgebra_glm::quat_angle_axis(*pitch, &Vec3::new(1.0, 0.0, 0.0));
        let yaw_rot = nalgebra_glm::quat_angle_axis(*yaw, &Vec3::new(0.0, 1.0, 0.0));
        let roll_rot =
            nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::new(0.0, 0.0, 1.0));

        spawn_shadow_entity(
            world,
            ShadowEntityDesc {
                mesh: "Cylinder",
                position: Vec3::new(
                    campfire.x + offset_x,
                    campfire.y + offset_y,
                    campfire.z + offset_z,
                ),
                scale: Vec3::new(0.08, 0.7, 0.08),
                rotation: yaw_rot * pitch_rot * roll_rot,
                material_name: &format!("CampfireLog_{}", index),
                material: log_material.clone(),
                name: &format!("Campfire Log {}", index),
            },
        );
    }

    let stone_material = Material {
        base_color: [0.3, 0.3, 0.3, 1.0],
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    };

    for index in 0..10 {
        let angle = (index as f32 / 10.0) * std::f32::consts::TAU;
        let radius = 0.6;
        let size = 0.1 + (index as f32 * 0.3).sin().abs() * 0.05;

        spawn_shadow_entity(
            world,
            ShadowEntityDesc {
                mesh: "Sphere",
                position: Vec3::new(
                    campfire.x + angle.cos() * radius,
                    campfire.y + size * 0.4,
                    campfire.z + angle.sin() * radius,
                ),
                scale: Vec3::new(size, size * 0.7, size),
                rotation: Quat::identity(),
                material_name: &format!("FireStone_{}", index),
                material: stone_material.clone(),
                name: &format!("Fire Stone {}", index),
            },
        );
    }
}

fn spawn_campfire_particles(world: &mut World) {
    let campfire = Vec3::new(0.0, 0.0, -1.0);

    let fire_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
    let mut fire = ParticleEmitter::fire(campfire + Vec3::new(0.0, 0.3, 0.0));
    fire.size_start = 0.08;
    fire.size_end = 0.02;
    fire.spawn_rate = 60.0;
    fire.initial_velocity_min = 0.3;
    fire.initial_velocity_max = 0.8;
    fire.gravity = Vec3::new(0.0, 0.4, 0.0);
    fire.drag = 1.5;
    fire.emissive_strength = 6.0;
    fire.turbulence_strength = 0.4;
    fire.turbulence_frequency = 1.5;
    world.set_particle_emitter(fire_entity, fire);

    let smoke_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
    let mut smoke = ParticleEmitter::smoke(campfire + Vec3::new(0.0, 0.6, 0.0));
    smoke.initial_velocity_min = 0.1;
    smoke.initial_velocity_max = 0.3;
    smoke.spawn_rate = 8.0;
    smoke.size_start = 0.1;
    smoke.size_end = 0.8;
    world.set_particle_emitter(smoke_entity, smoke);

    let ember_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
    world.set_particle_emitter(
        ember_entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.06 },
            position: campfire + Vec3::new(0.0, 0.35, 0.0),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 2.0,
            burst_count: 0,
            particle_lifetime_min: 1.5,
            particle_lifetime_max: 3.0,
            initial_velocity_min: 0.1,
            initial_velocity_max: 0.4,
            velocity_spread: 0.4,
            gravity: Vec3::new(0.01, 0.05, 0.005),
            drag: 0.3,
            size_start: 0.015,
            size_end: 0.004,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(1.0, 0.7, 0.2, 0.0)),
                    (0.1, Vec4::new(1.0, 0.6, 0.15, 1.0)),
                    (0.4, Vec4::new(1.0, 0.4, 0.08, 0.9)),
                    (0.7, Vec4::new(0.9, 0.25, 0.03, 0.6)),
                    (0.9, Vec4::new(0.6, 0.1, 0.01, 0.2)),
                    (1.0, Vec4::new(0.3, 0.03, 0.0, 0.0)),
                ],
            },
            emissive_strength: 8.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.3,
            turbulence_frequency: 0.8,

            ..Default::default()
        },
    );
}

fn spawn_campfire_light(world: &mut World) -> Entity {
    let campfire = Vec3::new(0.0, 0.0, -1.0);

    let light_entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];
    world.set_local_transform(
        light_entity,
        LocalTransform {
            translation: campfire + Vec3::new(0.0, 0.6, 0.0),
            ..Default::default()
        },
    );
    world.set_light(
        light_entity,
        Light {
            light_type: LightType::Point,
            color: Vec3::new(1.0, 0.7, 0.3),
            intensity: 5.0,
            range: 18.0,
            cast_shadows: true,
            ..Default::default()
        },
    );
    light_entity
}

fn update_campfire_light(world: &mut World, light_entity: Option<Entity>) {
    let Some(entity) = light_entity else {
        return;
    };

    let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
    if let Some(light) = world.get_light_mut(entity) {
        let flicker1 = (time * 8.0).sin() * 0.2;
        let flicker2 = (time * 12.5).sin() * 0.15;
        let flicker3 = (time * 23.0).sin() * 0.1;
        let flicker4 = (time * 3.5).sin() * 0.08;
        light.intensity = 5.0 + flicker1 + flicker2 + flicker3 + flicker4;
    }
}

fn spawn_trees(world: &mut World) {
    let tree_positions: [(f32, f32); 12] = [
        (-8.0, -8.0),
        (-10.0, -3.0),
        (-9.0, 3.0),
        (-7.0, 7.0),
        (-3.0, 8.0),
        (3.0, 8.0),
        (7.0, 7.0),
        (9.0, 3.0),
        (10.0, -3.0),
        (8.0, -8.0),
        (-5.0, -10.0),
        (5.0, -10.0),
    ];

    let trunk_material = Material {
        base_color: [0.3, 0.18, 0.08, 1.0],
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    };

    for (tree_index, (tree_x, tree_z)) in tree_positions.iter().enumerate() {
        let tree_scale = 0.8 + (tree_index as f32 * 0.13) % 0.5;
        let trunk_height = 0.8 + (tree_index as f32 * 0.07) % 0.4;
        let trunk_radius = 0.12 + (tree_index as f32 * 0.01) % 0.06;

        spawn_shadow_entity(
            world,
            ShadowEntityDesc {
                mesh: "Cylinder",
                position: Vec3::new(*tree_x, trunk_height / 2.0, *tree_z),
                scale: Vec3::new(trunk_radius * 2.0, trunk_height, trunk_radius * 2.0),
                rotation: Quat::identity(),
                material_name: &format!("TreeTrunk_{}", tree_index),
                material: trunk_material.clone(),
                name: &format!("Tree Trunk {}", tree_index),
            },
        );

        let tier_radii = [2.2 * tree_scale, 1.6 * tree_scale, 1.0 * tree_scale];
        let tier_heights = [1.5 * tree_scale, 1.3 * tree_scale, 1.1 * tree_scale];
        let tier_y_offsets = [0.0, 0.9 * tree_scale, 1.7 * tree_scale];
        let green_variation = (tree_index as f32 * 0.05) % 0.12;

        for tier in 0..3 {
            let foliage_y = trunk_height + tier_y_offsets[tier] + tier_heights[tier] / 2.0;

            spawn_shadow_entity(
                world,
                ShadowEntityDesc {
                    mesh: "Cone",
                    position: Vec3::new(*tree_x, foliage_y, *tree_z),
                    scale: Vec3::new(tier_radii[tier], tier_heights[tier], tier_radii[tier]),
                    rotation: Quat::identity(),
                    material_name: &format!("TreeCone_{}_{}", tree_index, tier),
                    material: Material {
                        base_color: [0.05, 0.28 + green_variation, 0.04, 1.0],
                        roughness: 0.95,
                        metallic: 0.0,
                        ..Default::default()
                    },
                    name: &format!("Tree {} Foliage {}", tree_index, tier),
                },
            );
        }
    }
}

fn create_secondary_world(renderer: &dyn Render) -> SecondaryWorld {
    let mut world = World::default();
    renderer.copy_fonts_to_world(&mut world);

    world.resources.world_id = 2000;
    world.resources.graphics.atmosphere = Atmosphere::Sunset;
    world.resources.graphics.show_grid = false;
    capture_procedural_atmosphere_ibl(&mut world, Atmosphere::Sunset, 0.0);

    let camera = spawn_pan_orbit_camera(
        &mut world,
        Vec3::new(0.0, 1.0, 0.0),
        8.0,
        0.0,
        0.3,
        "Secondary Camera".to_string(),
    );
    world.resources.active_camera = Some(camera);

    let sun = spawn_sun_without_shadows(&mut world);
    if let Some(transform) = world.get_local_transform_mut(sun) {
        transform.translation = Vec3::new(5.0, 10.0, 5.0);
    }

    let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("Movie Screen Texture"),
        size: wgpu::Extent3d {
            width: RENDER_TEXTURE_WIDTH,
            height: RENDER_TEXTURE_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: renderer.surface_format(),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    SecondaryWorld {
        world,
        texture,
        texture_view,
        egui_texture_id: None,
    }
}

fn spawn_secondary_scene(world: &mut World) -> Vec<Entity> {
    let floor = spawn_mesh(
        world,
        "Cube",
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(12.0, 1.0, 12.0),
    );
    world.set_name(floor, Name("Secondary Floor".to_string()));
    world.set_material_ref(floor, MaterialRef::new("White".to_string()));

    let mut cube_entities = Vec::new();

    let colors = [
        "Red", "Green", "Blue", "Yellow", "Cyan", "Magenta", "Orange",
    ];
    for (index, color) in colors.iter().enumerate() {
        let angle = (index as f32 / colors.len() as f32) * std::f32::consts::TAU;
        let radius = 3.0;
        let position = Vec3::new(angle.cos() * radius, 0.5, angle.sin() * radius);

        let entity = spawn_mesh(world, "Cube", position, Vec3::new(0.8, 0.8, 0.8));
        world.set_name(entity, Name(format!("Orbiting Cube {}", index)));
        world.set_material_ref(entity, MaterialRef::new(color.to_string()));
        cube_entities.push(entity);
    }

    let center_sphere = spawn_mesh(
        world,
        "Sphere",
        Vec3::new(0.0, 1.5, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
    );
    world.set_name(center_sphere, Name("Center Sphere".to_string()));

    material_registry_insert(
        &mut world.resources.material_registry,
        "GlowingSphere".to_string(),
        Material {
            base_color: [1.0, 0.8, 0.2, 1.0],
            emissive_factor: [1.0, 0.6, 0.1],
            unlit: true,
            ..Default::default()
        },
    );
    world.set_material_ref(center_sphere, MaterialRef::new("GlowingSphere".to_string()));

    cube_entities
}

fn animate_secondary_cubes(world: &mut World, cube_entities: &[Entity], total_time: f32) {
    for (index, &entity) in cube_entities.iter().enumerate() {
        let base_angle = (index as f32 / cube_entities.len() as f32) * std::f32::consts::TAU;
        let angle = base_angle + total_time * 0.5;
        let radius = 3.0;
        let bob_height = 0.5 + (total_time * 1.5 + index as f32).sin() * 0.4;

        if let Some(transform) = world.get_local_transform_mut(entity) {
            transform.translation =
                Vec3::new(angle.cos() * radius, bob_height, angle.sin() * radius);
            transform.rotation =
                nalgebra_glm::quat_angle_axis(total_time * 2.0 + index as f32, &Vec3::y());
        }
        world.set_local_transform_dirty(entity, LocalTransformDirty);
    }
}
