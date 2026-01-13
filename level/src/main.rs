mod constants;
mod state;
mod systems;

use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::ecs::camera::systems::fly_camera_system;
use nightshade::ecs::generational_registry::registry_entry_by_name;
use nightshade::ecs::navmesh::{RecastNavMeshConfig, generate_navmesh_recast};
use nightshade::ecs::physics::{
    ColliderComponent, ColliderShape, RigidBodyComponent, physics_debug_draw_system,
    run_physics_systems, spawn_first_person_player,
};
use nightshade::ecs::prefab::import_gltf_from_bytes;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::prefab::spawn_prefab_with_skins;
use nightshade::ecs::texture_loader::{
    AssetLoadingStatus, process_and_load_textures, set_asset_search_paths,
};
use nightshade::ecs::transform::queries::query_descendants;
use nightshade::ecs::transform::systems::run_systems as run_transform_systems;
use nightshade::ecs::world::commands::WorldCommand;
use nightshade::prelude::*;
use state::{GameScreen, LevelDemo};
use systems::props::PropShape;

const GAMEMAP_MODEL: &[u8] = include_bytes!("../assets/models/gamemap.glb");
const VIEW_MODEL: &[u8] = include_bytes!("../assets/models/view_model/view_model.glb");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_asset_search_paths(vec!["apps/level/".to_string()]);
    launch(LevelDemo::default())?;
    Ok(())
}

impl State for LevelDemo {
    fn title(&self) -> &str {
        "Level Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Space;

        spawn_sun_without_shadows(world);

        self.screen = GameScreen::Title;
    }

    fn run_systems(&mut self, world: &mut World) {
        match self.screen {
            GameScreen::Title => {}
            GameScreen::Loading => {
                run_loading_systems(self, world);
            }
            GameScreen::Gameplay => {
                run_gameplay_systems(self, world);
            }
            GameScreen::Paused => {
                if world
                    .resources
                    .input
                    .keyboard
                    .is_key_pressed(KeyCode::Escape)
                {
                    self.screen = GameScreen::Gameplay;
                }
            }
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        match self.screen {
            GameScreen::Title => {
                systems::ui::title_screen_ui(self, world, ctx);
            }
            GameScreen::Loading => {
                egui::CentralPanel::default()
                    .frame(egui::Frame::default().fill(egui::Color32::BLACK))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(ui.available_height() / 2.0 - 60.0);

                            ui.label(
                                egui::RichText::new("Loading Assets")
                                    .size(32.0)
                                    .color(egui::Color32::WHITE),
                            );

                            ui.add_space(20.0);

                            let total = self.loading.total_textures;
                            let loaded = self.loading.loaded_textures;
                            let failed = self
                                .texture_queue
                                .lock()
                                .map(|q| q.failed_count())
                                .unwrap_or(0);
                            let processed = loaded + failed;

                            if total > 0 {
                                let progress = processed as f32 / total as f32;
                                let bar_width = 400.0;
                                let bar_height = 20.0;

                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(bar_width, bar_height),
                                    egui::Sense::hover(),
                                );

                                ui.painter()
                                    .rect_filled(rect, 4.0, egui::Color32::from_gray(40));

                                let filled_width = bar_width * progress;
                                let filled_rect = egui::Rect::from_min_size(
                                    rect.min,
                                    egui::vec2(filled_width, bar_height),
                                );
                                ui.painter().rect_filled(
                                    filled_rect,
                                    4.0,
                                    egui::Color32::from_rgb(100, 180, 100),
                                );

                                ui.add_space(10.0);

                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} / {} textures",
                                        processed, total
                                    ))
                                    .size(18.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                                );
                            } else if !self.level_spawned {
                                ui.label(
                                    egui::RichText::new("Loading level...")
                                        .size(18.0)
                                        .color(egui::Color32::LIGHT_GRAY),
                                );
                            }
                        });
                    });
            }
            GameScreen::Gameplay => {
                systems::ui::gameplay_ui(self, world, ctx);
            }
            GameScreen::Paused => {
                systems::ui::pause_menu_ui(self, world, ctx);
            }
        }
    }
}

fn load_level(demo: &mut LevelDemo, world: &mut World) {
    demo.fly_mode = false;

    match import_gltf_from_bytes(GAMEMAP_MODEL) {
        Ok(result) => {
            for (name, (rgba_data, width, height)) in result.textures {
                world.queue_command(WorldCommand::LoadTexture {
                    name,
                    rgba_data,
                    width,
                    height,
                });
            }

            for (name, mesh) in &result.meshes {
                mesh_cache_insert(&mut world.resources.mesh_cache, name.clone(), mesh.clone());
            }

            if let Some(prefab) = result.prefabs.into_iter().next() {
                let level_entity = spawn_prefab_with_skins(
                    world,
                    &prefab,
                    &result.animations,
                    &result.skins,
                    Vec3::zeros(),
                );
                demo.level_entity = Some(level_entity);
                demo.spawned_entities.push(level_entity);

                run_transform_systems(world);

                generate_colliders_from_entity(world, level_entity);

                generate_navmesh_from_entity(world, level_entity);
            }

            demo.level_loaded = true;

            let player_position = Vec3::new(0.0, 3.0, 0.0);

            let fly_camera = spawn_camera(world, player_position, "Fly Camera".to_string());
            demo.fly_camera = Some(fly_camera);

            let (player_entity, player_camera) = spawn_first_person_player(world, player_position);
            demo.player_entity = Some(player_entity);
            demo.camera_entity = Some(player_camera);

            world.resources.active_camera = Some(player_camera);

            spawn_test_props(demo, world, player_position);

            spawn_player_hands_and_flashlight(demo, world);

            demo.level_spawned = true;

            tracing::info!("Loaded level with colliders and navmesh");
        }
        Err(error) => {
            tracing::error!("Failed to load level: {}", error);
            demo.level_loaded = false;
        }
    }
}

fn generate_colliders_from_entity(world: &mut World, entity: Entity) {
    let mut entities_to_process = vec![entity];
    entities_to_process.extend(query_descendants(world, entity));

    let mut collider_count = 0;

    for current_entity in entities_to_process {
        let mesh_name = match world.get_render_mesh(current_entity) {
            Some(render_mesh) => render_mesh.name.clone(),
            None => continue,
        };

        let global_transform = world
            .get_global_transform(current_entity)
            .map(|t| t.0)
            .unwrap_or_else(nalgebra_glm::Mat4::identity);

        let mesh = match registry_entry_by_name(&world.resources.mesh_cache.registry, &mesh_name) {
            Some(mesh) => mesh,
            None => continue,
        };

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<[u32; 3]> = Vec::new();

        for vertex in &mesh.vertices {
            let pos = nalgebra_glm::vec4(
                vertex.position[0],
                vertex.position[1],
                vertex.position[2],
                1.0,
            );
            let transformed = global_transform * pos;
            vertices.push([transformed.x, transformed.y, transformed.z]);
        }

        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                indices.push([chunk[0], chunk[1], chunk[2]]);
            }
        }

        if indices.is_empty() {
            continue;
        }

        let collision_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RIGID_BODY
                | COLLIDER,
            1,
        )[0];

        if let Some(name) = world.get_name_mut(collision_entity) {
            name.0 = format!("Mesh Collision {}", collider_count);
        }

        if let Some(rigid_body) = world.get_rigid_body_mut(collision_entity) {
            *rigid_body = RigidBodyComponent::new_static();
        }

        if let Some(collider) = world.get_collider_mut(collision_entity) {
            *collider = ColliderComponent {
                shape: ColliderShape::TriMesh { vertices, indices },
                friction: 0.7,
                restitution: 0.0,
                ..Default::default()
            };
        }

        collider_count += 1;
    }

    tracing::info!("Generated {} colliders from level mesh", collider_count);
}

fn generate_navmesh_from_entity(world: &mut World, entity: Entity) {
    let mut all_vertices: Vec<[f32; 3]> = Vec::new();
    let mut all_indices: Vec<[u32; 3]> = Vec::new();

    let mut entities_to_process = vec![entity];
    entities_to_process.extend(query_descendants(world, entity));

    for current_entity in entities_to_process {
        let mesh_name = match world.get_render_mesh(current_entity) {
            Some(render_mesh) => render_mesh.name.clone(),
            None => continue,
        };

        let global_transform = world
            .get_global_transform(current_entity)
            .map(|t| t.0)
            .unwrap_or_else(nalgebra_glm::Mat4::identity);

        let mesh = match registry_entry_by_name(&world.resources.mesh_cache.registry, &mesh_name) {
            Some(mesh) => mesh,
            None => continue,
        };

        let base_index = all_vertices.len() as u32;

        for vertex in &mesh.vertices {
            let pos = nalgebra_glm::vec4(
                vertex.position[0],
                vertex.position[1],
                vertex.position[2],
                1.0,
            );
            let transformed = global_transform * pos;
            all_vertices.push([transformed.x, transformed.y, transformed.z]);
        }

        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                all_indices.push([
                    base_index + chunk[0],
                    base_index + chunk[1],
                    base_index + chunk[2],
                ]);
            }
        }
    }

    if all_indices.is_empty() {
        tracing::warn!("No geometry found for navmesh generation");
        return;
    }

    tracing::info!(
        "Collected {} vertices, {} triangles for navmesh",
        all_vertices.len(),
        all_indices.len()
    );

    let config = RecastNavMeshConfig::default();

    if let Some(navmesh) = generate_navmesh_recast(&all_vertices, &all_indices, &config) {
        tracing::info!(
            "Built navmesh: {} triangles, {} connections",
            navmesh.triangles.len(),
            navmesh.adjacency.values().map(|v| v.len()).sum::<usize>()
        );
        world.resources.navmesh = navmesh;
    } else {
        tracing::warn!("Failed to generate navmesh from geometry");
    }
}

fn run_loading_systems(demo: &mut LevelDemo, world: &mut World) {
    if !demo.level_spawned {
        load_level(demo, world);
        return;
    }

    let status = process_and_load_textures(&demo.texture_queue, world, &mut demo.loading, 8);

    if status == AssetLoadingStatus::Complete {
        demo.screen = GameScreen::Gameplay;
    }
}

fn spawn_test_props(demo: &mut LevelDemo, world: &mut World, near_position: Vec3) {
    let box_material = Material {
        base_color: [0.6, 0.5, 0.35, 1.0],
        roughness: 0.7,
        metallic: 0.0,
        ..Default::default()
    };

    systems::props::spawn_grabbable_prop(
        world,
        &mut demo.physics_objects,
        &mut demo.props,
        near_position + Vec3::new(1.0, 1.0, 1.0),
        PropShape::Cube(0.25),
        box_material.clone(),
        2.0,
    );

    let sphere_material = Material {
        base_color: [0.7, 0.2, 0.2, 1.0],
        roughness: 0.5,
        metallic: 0.3,
        ..Default::default()
    };

    systems::props::spawn_grabbable_prop(
        world,
        &mut demo.physics_objects,
        &mut demo.props,
        near_position + Vec3::new(-1.0, 1.0, 1.0),
        PropShape::Sphere(0.15),
        sphere_material,
        1.5,
    );

    let cylinder_material = Material {
        base_color: [0.5, 0.5, 0.55, 1.0],
        roughness: 0.3,
        metallic: 0.8,
        ..Default::default()
    };

    systems::props::spawn_grabbable_prop(
        world,
        &mut demo.physics_objects,
        &mut demo.props,
        near_position + Vec3::new(0.0, 1.0, 2.0),
        PropShape::Cylinder {
            radius: 0.12,
            height: 0.4,
        },
        cylinder_material,
        3.0,
    );
}

fn spawn_player_hands_and_flashlight(demo: &mut LevelDemo, world: &mut World) {
    let Some(camera_entity) = demo.camera_entity else {
        return;
    };

    let load_result = import_gltf_from_bytes(VIEW_MODEL);

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

            if let Some(prefab) = result.prefabs.into_iter().next() {
                let hands_entity = spawn_prefab_with_skins(
                    world,
                    &prefab,
                    &result.animations,
                    &result.skins,
                    Vec3::zeros(),
                );

                world.update_parent(hands_entity, Some(Parent(Some(camera_entity))));

                if let Some(transform) = world.get_local_transform_mut(hands_entity) {
                    let view_model_scale = 0.4;
                    transform.translation = Vec3::new(0.0, -0.02, -0.06);
                    transform.rotation = nalgebra_glm::quat_angle_axis(
                        std::f32::consts::PI,
                        &Vec3::new(0.0, 1.0, 0.0),
                    );
                    transform.scale =
                        Vec3::new(view_model_scale, view_model_scale, view_model_scale);
                }
                world.mark_local_transform_dirty(hands_entity);

                if let Some(player) = world.get_animation_player_mut(hands_entity)
                    && player.clips.len() > 9
                {
                    player.blend_to(9, 0.0);
                    player.looping = true;
                }

                demo.hands_entity = Some(hands_entity);
                tracing::info!(
                    "Spawned player hands with {} animation clips",
                    world
                        .get_animation_player(hands_entity)
                        .map(|p| p.clips.len())
                        .unwrap_or(0)
                );
            }
        }
        Err(error) => {
            tracing::error!("Failed to load view model: {}", error);
        }
    }

    let flashlight_entity = systems::flashlight::spawn_flashlight(world);
    demo.flashlight_entity = Some(flashlight_entity);
    tracing::info!("Spawned flashlight");
}

fn run_gameplay_systems(demo: &mut LevelDemo, world: &mut World) {
    if world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Escape)
    {
        demo.screen = GameScreen::Paused;
        return;
    }

    systems::input::detect_input_mode(demo, world);

    if demo.fly_mode {
        fly_camera_system(world);
    } else {
        run_physics_systems(world);

        systems::camera::camera_look_system(demo, world);
        systems::camera::lean_system(demo, world);
        systems::camera::crouch_camera_system(demo, world);
    }

    systems::interaction::interaction_system(demo, world);
    systems::flashlight::update_flashlight(demo, world);

    systems::dialogue::check_dialogue_triggers(demo, world);
    systems::dialogue::dialogue_system(demo, world);

    run_navmesh_systems(world);

    systems::audio::audio_system(demo, world);

    physics_debug_draw_system(world);
}
