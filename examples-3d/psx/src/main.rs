use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::{look_camera_system, spawn_first_person_player};
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::world::commands::load_procedural_textures;
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::texture_cache_add_reference;

const CAMERA_HEIGHT: f32 = 0.8;
const DANCE_MODEL: &[u8] = include_bytes!("../../../assets/models/dance.glb");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(PsxDemo::default())
}

struct PsxDemo {
    player_entity: Option<Entity>,
    camera_entity: Option<Entity>,
    dance_entity: Option<Entity>,
    rotation_speed: f32,
}

impl Default for PsxDemo {
    fn default() -> Self {
        Self {
            player_entity: None,
            camera_entity: None,
            dance_entity: None,
            rotation_speed: 0.5,
        }
    }
}

impl State for PsxDemo {
    fn title(&self) -> &str {
        "PS1 Style Rendering Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.vertex_snap = Some(VertexSnap::default());
        world.resources.graphics.affine_texture_mapping = true;
        world.resources.graphics.fog = Some(Fog::default());

        load_procedural_textures(world);

        let sun = spawn_sun(world);
        if let Some(light) = world.core.get_light_mut(sun) {
            light.intensity = 1.5;
        }

        let player_position = Vec3::new(0.0, 1.2, 6.0);
        let (player_entity, camera_entity) = spawn_first_person_player(world, player_position);

        if let Some(transform) = world.core.get_local_transform_mut(camera_entity) {
            transform.translation.y = CAMERA_HEIGHT;
        }

        if let Some(camera) = world.core.get_camera_mut(camera_entity) {
            camera.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 75.0_f32.to_radians(),
                z_far: None,
                z_near: 0.1,
            });
        }

        self.player_entity = Some(player_entity);
        self.camera_entity = Some(camera_entity);
        world.resources.active_camera = Some(camera_entity);

        let ground_collider = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | COLLIDER
                | RIGID_BODY,
            1,
        )[0];
        if let Some(name) = world.core.get_name_mut(ground_collider) {
            name.0 = "Ground Collider".to_string();
        }
        if let Some(transform) = world.core.get_local_transform_mut(ground_collider) {
            transform.translation = Vec3::new(0.0, -0.5, 0.0);
        }
        if let Some(rigid_body) = world.core.get_rigid_body_mut(ground_collider) {
            *rigid_body = RigidBodyComponent::new_static().with_translation(0.0, -0.5, 0.0);
        }
        if let Some(collider) = world.core.get_collider_mut(ground_collider) {
            *collider = ColliderComponent::new_cuboid(20.0, 0.1, 20.0)
                .with_friction(0.8)
                .with_restitution(0.1);
        }

        let ground = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(20.0, 0.1, 20.0),
        );
        let ground_material = format!("Ground_{}", ground.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            ground_material.clone(),
            Material {
                base_color: [0.0, 1.0, 0.0, 1.0],
                base_texture: None,
                roughness: 1.0,
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
        world.core.set_material_ref(ground, MaterialRef::new(ground_material));

        let cube = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        world.core.add_components(cube, RIGID_BODY | COLLIDER);
        if let Some(rigid_body) = world.core.get_rigid_body_mut(cube) {
            *rigid_body = RigidBodyComponent::new_static().with_translation(0.0, 0.5, 0.0);
        }
        if let Some(collider) = world.core.get_collider_mut(cube) {
            *collider = ColliderComponent::new_cuboid(0.5, 0.5, 0.5);
        }
        let cube_material = format!("Cube_{}", cube.id);
        texture_cache_add_reference(&mut world.resources.texture_cache, "checkerboard");
        material_registry_insert(
            &mut world.resources.material_registry,
            cube_material.clone(),
            Material {
                base_color: [1.0, 1.0, 1.0, 1.0],
                base_texture: Some("checkerboard".to_string()),
                roughness: 0.5,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&cube_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(cube, MaterialRef::new(cube_material));

        let sphere = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(-2.5, 0.5, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );
        world.core.add_components(sphere, RIGID_BODY | COLLIDER);
        if let Some(rigid_body) = world.core.get_rigid_body_mut(sphere) {
            *rigid_body = RigidBodyComponent::new_static().with_translation(-2.5, 0.5, 0.0);
        }
        if let Some(collider) = world.core.get_collider_mut(sphere) {
            *collider = ColliderComponent::new_ball(0.5);
        }
        let sphere_material = format!("Sphere_{}", sphere.id);
        texture_cache_add_reference(&mut world.resources.texture_cache, "gradient");
        material_registry_insert(
            &mut world.resources.material_registry,
            sphere_material.clone(),
            Material {
                base_color: [1.0, 1.0, 1.0, 1.0],
                base_texture: Some("gradient".to_string()),
                roughness: 0.3,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&sphere_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(sphere, MaterialRef::new(sphere_material));

        let torus = spawn_mesh(
            world,
            "Torus",
            Vec3::new(2.5, 0.5, 0.0),
            Vec3::new(0.8, 0.8, 0.8),
        );
        world.core.add_components(torus, RIGID_BODY | COLLIDER);
        if let Some(rigid_body) = world.core.get_rigid_body_mut(torus) {
            *rigid_body = RigidBodyComponent::new_static().with_translation(2.5, 0.5, 0.0);
        }
        if let Some(collider) = world.core.get_collider_mut(torus) {
            *collider = ColliderComponent::new_cylinder(0.2, 0.4);
        }
        let torus_material = format!("Torus_{}", torus.id);
        texture_cache_add_reference(&mut world.resources.texture_cache, "uv_test");
        material_registry_insert(
            &mut world.resources.material_registry,
            torus_material.clone(),
            Material {
                base_color: [1.0, 1.0, 1.0, 1.0],
                base_texture: Some("uv_test".to_string()),
                roughness: 0.4,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&torus_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(torus, MaterialRef::new(torus_material));

        for index in 0..8 {
            let angle = (index as f32 / 8.0) * std::f32::consts::TAU;
            let radius = 8.0;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            let pillar = spawn_mesh(
                world,
                "Cylinder",
                Vec3::new(x, 0.5, z),
                Vec3::new(0.5, 2.0, 0.5),
            );
            world.core.add_components(pillar, RIGID_BODY | COLLIDER);
            if let Some(rigid_body) = world.core.get_rigid_body_mut(pillar) {
                *rigid_body = RigidBodyComponent::new_static().with_translation(x, 0.5, z);
            }
            if let Some(collider) = world.core.get_collider_mut(pillar) {
                *collider = ColliderComponent::new_cylinder(1.0, 0.25);
            }
            let pillar_material = format!("Pillar_{}", pillar.id);
            texture_cache_add_reference(&mut world.resources.texture_cache, "checkerboard");
            material_registry_insert(
                &mut world.resources.material_registry,
                pillar_material.clone(),
                Material {
                    base_color: [0.8, 0.7, 0.6, 1.0],
                    base_texture: Some("checkerboard".to_string()),
                    roughness: 0.8,
                    metallic: 0.0,
                    uv_scale: [2.0, 4.0],
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&pillar_material)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(mat_index);
            }
            world.core.set_material_ref(pillar, MaterialRef::new(pillar_material));
        }

        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(DANCE_MODEL);
        match load_result {
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
                    let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                        world,
                        &prefab,
                        &result.animations,
                        &result.skins,
                        Vec3::new(0.0, -0.4, -3.0),
                    );
                    self.dance_entity = Some(entity);

                    if let Some(player) = world.core.get_animation_player_mut(entity) {
                        player.looping = true;
                        if !player.clips.is_empty() {
                            player.play(0);
                        }
                    }
                }
            }
            Err(error) => {
                tracing::error!("Failed to load dance model: {}", error);
            }
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        look_camera_system(world);

        let time = world.resources.window.timing.uptime_milliseconds as f32 * 0.001;
        let angle = time * self.rotation_speed;
        let rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::y());

        let entities: Vec<_> = world
            .core.query_entities(RENDER_MESH | LOCAL_TRANSFORM)
            .collect();
        for entity in entities {
            if Some(entity) == self.player_entity
                || Some(entity) == self.camera_entity
                || Some(entity) == self.dance_entity
            {
                continue;
            }
            if let Some(transform) = world.core.get_local_transform_mut(entity)
                && transform.translation.y > 0.3
                && transform.translation.y < 1.5
            {
                transform.rotation = rotation;
            }
            mark_local_transform_dirty(world, entity);
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Render Effects")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.heading("Vertex Snapping");
                let mut snap_enabled = world.resources.graphics.vertex_snap.is_some();
                if ui
                    .checkbox(&mut snap_enabled, "Enable Vertex Snapping")
                    .changed()
                {
                    if snap_enabled {
                        world.resources.graphics.vertex_snap = Some(VertexSnap::default());
                    } else {
                        world.resources.graphics.vertex_snap = None;
                    }
                }
                if let Some(ref mut vertex_snap) = world.resources.graphics.vertex_snap {
                    ui.horizontal(|ui| {
                        ui.label("Resolution X:");
                        ui.add(egui::Slider::new(
                            &mut vertex_snap.resolution[0],
                            80.0..=640.0,
                        ));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Resolution Y:");
                        ui.add(egui::Slider::new(
                            &mut vertex_snap.resolution[1],
                            60.0..=480.0,
                        ));
                    });
                }

                ui.add_space(10.0);
                ui.heading("Texture Mapping");
                ui.checkbox(
                    &mut world.resources.graphics.affine_texture_mapping,
                    "Enable Affine Texture Mapping",
                );

                ui.add_space(10.0);
                ui.heading("Distance Fog");
                let mut fog_enabled = world.resources.graphics.fog.is_some();
                if ui.checkbox(&mut fog_enabled, "Enable Fog").changed() {
                    if fog_enabled {
                        world.resources.graphics.fog = Some(Fog::default());
                    } else {
                        world.resources.graphics.fog = None;
                    }
                }
                if let Some(ref mut fog) = world.resources.graphics.fog {
                    ui.horizontal(|ui| {
                        ui.label("Start Distance:");
                        ui.add(egui::Slider::new(&mut fog.start, 0.5..=10.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("End Distance:");
                        ui.add(egui::Slider::new(&mut fog.end, 5.0..=50.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Fog Color:");
                        ui.color_edit_button_rgb(&mut fog.color);
                    });
                }

                ui.add_space(10.0);
                ui.heading("Animation");
                ui.horizontal(|ui| {
                    ui.label("Rotation Speed:");
                    ui.add(egui::Slider::new(&mut self.rotation_speed, 0.0..=2.0));
                });

                ui.add_space(10.0);
                ui.separator();
                ui.label("Controls:");
                ui.label("  WASD - Move");
                ui.label("  Right-click + Mouse - Look around");
                ui.label("  Space - Jump");
                ui.label("  Ctrl - Crouch");
                ui.label("  ESC - Exit");
            });
    }
}
