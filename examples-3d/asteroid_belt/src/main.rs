use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

#[cfg(target_arch = "wasm32")]
const ASTEROID_COUNT: usize = 100_000;
#[cfg(not(target_arch = "wasm32"))]
const ASTEROID_COUNT: usize = 500_000;
const BELT_INNER_RADIUS: f32 = 50.0;
const BELT_OUTER_RADIUS: f32 = 150.0;
const BELT_HEIGHT: f32 = 10.0;
const ORBIT_SPEED: f32 = 0.05;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(AsteroidBeltWorld::default())?;
    Ok(())
}

#[derive(Default)]
struct AsteroidBeltWorld {
    fps_text: Option<Entity>,
    asteroid_entity: Option<Entity>,
}

impl State for AsteroidBeltWorld {
    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.ui_scale = Some(1.0);
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.min_screen_pixel_size = 2.0;
        world.resources.graphics.clear_color = [0.0, 0.0, 0.0, 1.0];
        world.resources.graphics.mesh_lod_chains = vec![MeshLodChain {
            base_mesh: "Sphere".to_string(),
            levels: vec![
                MeshLodLevel {
                    mesh_name: "Sphere".to_string(),
                    min_screen_pixels: 20.0,
                },
                MeshLodLevel {
                    mesh_name: "Sphere_LOD1".to_string(),
                    min_screen_pixels: 6.0,
                },
                MeshLodLevel {
                    mesh_name: "Sphere_LOD2".to_string(),
                    min_screen_pixels: 0.0,
                },
            ],
        }];
        world.resources.user_interface.enabled = true;

        spawn_sun_without_shadows(world);

        let camera = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | CAMERA | PAN_ORBIT_CAMERA,
            1,
        )[0];

        world.set_local_transform(
            camera,
            LocalTransform {
                translation: Vec3::new(0.0, 100.0, 200.0),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(camera, LocalTransformDirty);
        world.set_global_transform(camera, GlobalTransform::default());
        world.set_camera(
            camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 60.0_f32.to_radians(),
                    z_near: 0.1,
                    z_far: Some(1000.0),
                }),
                smoothing: Some(Smoothing::default()),
            },
        );
        world.set_pan_orbit_camera(
            camera,
            PanOrbitCamera {
                focus: Vec3::new(0.0, 0.0, 0.0),
                target_focus: Vec3::new(0.0, 0.0, 0.0),
                radius: 200.0,
                target_radius: 200.0,
                pitch: -0.3,
                target_pitch: -0.3,
                yaw: 0.0,
                target_yaw: 0.0,
                ..Default::default()
            },
        );
        world.resources.active_camera = Some(camera);

        let mut rng = rand::rng();
        let mut instances = Vec::with_capacity(ASTEROID_COUNT);

        for _ in 0..ASTEROID_COUNT {
            let angle = rng.random::<f32>() * std::f32::consts::TAU;
            let radius =
                BELT_INNER_RADIUS + rng.random::<f32>() * (BELT_OUTER_RADIUS - BELT_INNER_RADIUS);
            let height = (rng.random::<f32>() - 0.5) * BELT_HEIGHT;

            let position = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);

            let rotation = nalgebra_glm::quat_angle_axis(
                rng.random::<f32>() * std::f32::consts::TAU,
                &Vec3::new(
                    rng.random::<f32>() - 0.5,
                    rng.random::<f32>() - 0.5,
                    rng.random::<f32>() - 0.5,
                )
                .normalize(),
            );

            let base_scale = 0.1 + rng.random::<f32>() * 0.4;
            let scale = Vec3::new(
                base_scale * (0.5 + rng.random::<f32>()),
                base_scale * (0.5 + rng.random::<f32>()),
                base_scale * (0.5 + rng.random::<f32>()),
            );

            instances.push(InstanceTransform::new(position, rotation, scale));
        }

        material_registry_insert(
            &mut world.resources.material_registry,
            "Asteroid".to_string(),
            Material {
                base_color: [0.5, 0.45, 0.4, 1.0],
                roughness: 0.9,
                metallic: 0.1,
                ..Default::default()
            },
        );

        self.asteroid_entity = Some(spawn_instanced_mesh_with_material(
            world, "Sphere", instances, "Asteroid",
        ));

        material_registry_insert(
            &mut world.resources.material_registry,
            "Planet".to_string(),
            Material {
                base_color: [0.7, 0.5, 0.3, 1.0],
                roughness: 0.95,
                metallic: 0.0,
                ..Default::default()
            },
        );

        let planet = spawn_mesh_at(
            world,
            "Sphere",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(25.0, 25.0, 25.0),
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get("Planet")
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(planet, MaterialRef::new("Planet".to_string()));

        material_registry_insert(
            &mut world.resources.material_registry,
            "Atmosphere".to_string(),
            Material {
                base_color: [0.6, 0.75, 0.9, 0.15],
                alpha_mode: AlphaMode::Blend,
                roughness: 1.0,
                metallic: 0.0,
                unlit: true,
                ..Default::default()
            },
        );

        let atmosphere = spawn_mesh_at(
            world,
            "Sphere",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(26.5, 26.5, 26.5),
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get("Atmosphere")
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(atmosphere, MaterialRef::new("Atmosphere".to_string()));

        self.fps_text = Some(spawn_hud_text_with_properties(
            world,
            "FPS: 0",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 10.0),
            TextProperties {
                font_size: 32.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                ..Default::default()
            },
        ));

        spawn_hud_text_with_properties(
            world,
            format!("Asteroids: {}", format_number(ASTEROID_COUNT)),
            HudAnchor::TopRight,
            Vec2::new(-10.0, 50.0),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(0.8, 0.8, 0.8, 1.0),
                ..Default::default()
            },
        );
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

        if let Some(asteroid_entity) = self.asteroid_entity {
            let rotation = nalgebra_glm::quat_angle_axis(time * ORBIT_SPEED, &Vec3::y_axis());
            if let Some(local_transform) = world.get_local_transform_mut(asteroid_entity) {
                local_transform.rotation = rotation;
            }
            world.set_local_transform_dirty(asteroid_entity, LocalTransformDirty);
        }

        if let Some(fps_entity) = self.fps_text {
            let fps = world.resources.window.timing.frames_per_second;
            if let Some(hud_text) = world.get_hud_text(fps_entity) {
                let text_index = hud_text.text_index;
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("FPS: {:.0}", fps));
                if let Some(hud_text) = world.get_hud_text_mut(fps_entity) {
                    hud_text.dirty = true;
                }
            }
        }
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Asteroid Belt")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.heading("GPU Instancing Demo");
                ui.separator();
                ui.label(format!(
                    "Rendering {} asteroids",
                    format_number(ASTEROID_COUNT)
                ));
                ui.label("with a single draw call per mesh type");
                ui.separator();
                ui.label("Use mouse to orbit camera");
                ui.label("Scroll to zoom");
            });
    }
}

fn format_number(number: usize) -> String {
    let number_str = number.to_string();
    let mut result = String::new();
    let chars: Vec<char> = number_str.chars().collect();

    for (index, character) in chars.iter().enumerate() {
        if index > 0 && (chars.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*character);
    }

    result
}
