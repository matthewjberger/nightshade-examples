mod constants;
mod data;
mod state;
mod systems;

use data::items::ItemType;
use data::skills::SkillType;
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
use nightshade::ecs::ui::ImmediateUi;
use nightshade::ecs::world::commands::WorldCommand;
use nightshade::prelude::*;
use nightshade::shell::shell_immediate_ui;
use state::{GameScreen, ImmersiveSim};
use systems::props::PropShape;

const GAMEMAP_MODEL: &[u8] = include_bytes!("../../../assets/models/gamemap.glb");
const VIEW_MODEL: &[u8] = include_bytes!("../../../assets/models/view_model.glb");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_asset_search_paths(vec!["".to_string()]);
    launch(ImmersiveSim::default())?;
    Ok(())
}

impl State for ImmersiveSim {
    fn title(&self) -> &str {
        "Immersive Sim"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Space;
        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.01;

        spawn_sun_without_shadows(world);

        let fps_text = spawn_hud_text_with_properties(
            world,
            "FPS: 0",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 10.0),
            TextProperties {
                font_size: 48.0,
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        self.fps_hud_text = Some(fps_text);

        self.screen = GameScreen::Loading;
    }

    fn run_systems(&mut self, world: &mut World) {
        update_fps_text(self, world);

        match self.screen {
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
            GameScreen::Loading => {
                loading_ui(self, ctx);
            }
            GameScreen::Gameplay => {
                gameplay_ui(self, world, ctx);
            }
            GameScreen::Paused => {
                pause_menu_ui(self, ctx);
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: ElementState) {
        let pressed = key_state == ElementState::Pressed;

        if pressed {
            let alt_pressed = world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::AltLeft)
                || world
                    .resources
                    .input
                    .keyboard
                    .is_key_pressed(KeyCode::AltRight);

            if key_code == KeyCode::KeyC && alt_pressed {
                self.shell.toggle();
                return;
            }
        }

        if self.shell.visible {
            self.shell.handle_key(key_code, pressed);
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

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 1.0);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.swapchain);
    }

    fn immediate_ui(&mut self, world: &mut World, ui: &mut ImmediateUi) {
        if self.shell.visible {
            let alt_pressed = world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::AltLeft)
                || world
                    .resources
                    .input
                    .keyboard
                    .is_key_pressed(KeyCode::AltRight);

            if !alt_pressed {
                for character in world.resources.input.keyboard.frame_chars.clone() {
                    if !character.is_control() {
                        self.shell.input_buffer.push(character);
                    }
                }
            }
        }

        shell_immediate_ui(&mut self.shell, ui, world);
    }
}

fn update_fps_text(game: &ImmersiveSim, world: &mut World) {
    if let Some(fps_text_entity) = game.fps_hud_text {
        let fps = world.resources.window.timing.frames_per_second;
        let text_index = world.get_hud_text(fps_text_entity).map(|t| t.text_index);
        if let Some(text_index) = text_index {
            world
                .resources
                .text_cache
                .set_text(text_index, format!("FPS: {:.0}", fps));
            if let Some(hud_text) = world.get_hud_text_mut(fps_text_entity) {
                hud_text.dirty = true;
            }
        }
    }
}

fn load_level(game: &mut ImmersiveSim, world: &mut World) {
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
                game.level_entity = Some(level_entity);
                game.spawned_entities.push(level_entity);

                run_transform_systems(world);

                generate_colliders_from_entity(world, level_entity);

                generate_navmesh_from_entity(world, level_entity);
            }

            game.level_loaded = true;

            let player_position = Vec3::new(0.0, 3.0, 0.0);

            let (player_entity, player_camera) = spawn_first_person_player(world, player_position);
            game.player_entity = Some(player_entity);
            game.camera_entity = Some(player_camera);

            world.resources.active_camera = Some(player_camera);

            spawn_test_props(game, world, player_position);

            spawn_player_hands_and_flashlight(game, world);

            systems::npcs::spawn_npcs(game, world);

            game.level_spawned = true;
        }
        Err(error) => {
            tracing::error!("Failed to load level: {}", error);
            game.level_loaded = false;
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
        return;
    }

    let config = RecastNavMeshConfig::default();

    if let Some(navmesh) = generate_navmesh_recast(&all_vertices, &all_indices, &config) {
        world.resources.navmesh = navmesh;
    }
}

fn run_loading_systems(game: &mut ImmersiveSim, world: &mut World) {
    if !game.level_spawned {
        load_level(game, world);
        return;
    }

    let status = process_and_load_textures(&game.texture_queue, world, &mut game.loading, 8);

    if status == AssetLoadingStatus::Complete {
        game.screen = GameScreen::Gameplay;
    }
}

fn spawn_test_props(game: &mut ImmersiveSim, world: &mut World, near_position: Vec3) {
    let box_size = 0.25;
    let box_offset = Vec3::new(1.5, 0.0, 2.0);
    let box_xz = near_position + box_offset;
    let box_floor_y = systems::npcs::sample_lowest_navmesh_height(world, box_xz.x, box_xz.z)
        .unwrap_or(near_position.y);
    let box_material = Material {
        base_color: [0.6, 0.5, 0.35, 1.0],
        roughness: 0.7,
        metallic: 0.0,
        ..Default::default()
    };
    systems::props::spawn_grabbable_prop(
        world,
        &mut game.physics_objects,
        &mut game.props,
        Vec3::new(box_xz.x, box_floor_y + box_size / 2.0, box_xz.z),
        PropShape::Cube(box_size),
        box_material.clone(),
        2.0,
    );

    let sphere_radius = 0.15;
    let sphere_offset = Vec3::new(-1.5, 0.0, 2.0);
    let sphere_xz = near_position + sphere_offset;
    let sphere_floor_y =
        systems::npcs::sample_lowest_navmesh_height(world, sphere_xz.x, sphere_xz.z)
            .unwrap_or(near_position.y);
    let sphere_material = Material {
        base_color: [0.7, 0.2, 0.2, 1.0],
        roughness: 0.5,
        metallic: 0.3,
        ..Default::default()
    };
    systems::props::spawn_grabbable_prop(
        world,
        &mut game.physics_objects,
        &mut game.props,
        Vec3::new(sphere_xz.x, sphere_floor_y + sphere_radius, sphere_xz.z),
        PropShape::Sphere(sphere_radius),
        sphere_material,
        1.5,
    );

    let cylinder_radius = 0.12;
    let cylinder_height = 0.4;
    let cylinder_offset = Vec3::new(0.0, 0.0, 3.0);
    let cylinder_xz = near_position + cylinder_offset;
    let cylinder_floor_y =
        systems::npcs::sample_lowest_navmesh_height(world, cylinder_xz.x, cylinder_xz.z)
            .unwrap_or(near_position.y);
    let cylinder_material = Material {
        base_color: [0.5, 0.5, 0.55, 1.0],
        roughness: 0.3,
        metallic: 0.8,
        ..Default::default()
    };
    systems::props::spawn_grabbable_prop(
        world,
        &mut game.physics_objects,
        &mut game.props,
        Vec3::new(
            cylinder_xz.x,
            cylinder_floor_y + cylinder_height / 2.0,
            cylinder_xz.z,
        ),
        PropShape::Cylinder {
            radius: cylinder_radius,
            height: cylinder_height,
        },
        cylinder_material,
        3.0,
    );
}

fn spawn_player_hands_and_flashlight(game: &mut ImmersiveSim, world: &mut World) {
    let Some(camera_entity) = game.camera_entity else {
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

                game.hands_entity = Some(hands_entity);
            }
        }
        Err(error) => {
            tracing::error!("Failed to load view model: {}", error);
        }
    }

    let flashlight_entity = systems::flashlight::spawn_flashlight(world);
    game.flashlight_entity = Some(flashlight_entity);
}

fn run_gameplay_systems(game: &mut ImmersiveSim, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time;
    game.shell.update_animation(delta_time);
    game.game_time += delta_time;

    if let Some(result) = world.resources.gpu_picking.take_result() {
        game.last_pick_result = Some(result);
    }

    if !game.shell.visible {
        let viewport_size = world
            .resources
            .window
            .cached_viewport_size
            .unwrap_or((800, 600));
        let center_x = viewport_size.0 / 2;
        let center_y = viewport_size.1 / 2;
        world.resources.gpu_picking.request_pick(center_x, center_y);
    }

    if let Some(target_level) = game.shell.context.pending_level.take() {
        transition_to_level(game, world, target_level);
    }

    if game.is_dead {
        return;
    }

    if !game.shell.visible
        && !game.shell.dragging_resize
        && world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Escape)
    {
        game.screen = GameScreen::Paused;
        return;
    }

    let noclip_just_disabled = game.noclip_was_active && !game.shell.context.noclip;
    game.noclip_was_active = game.shell.context.noclip;

    if noclip_just_disabled {
        teleport_player_to_camera(game, world);
    }

    systems::input::detect_input_mode(game, world);

    if !game.shell.visible && !game.shell.dragging_resize {
        if game.shell.context.noclip {
            fly_camera_system(world);
        } else {
            run_physics_systems(world);

            if game.dialogue.active {
                systems::camera::lean_system(game, world);
                systems::camera::crouch_camera_system(game, world);
            } else {
                systems::camera::camera_look_system(game, world);
                systems::camera::lean_system(game, world);
                systems::camera::crouch_camera_system(game, world);
                systems::interaction::interaction_system(game, world);
            }
        }
    } else if !game.shell.context.noclip {
        run_physics_systems(world);
    }

    game.player_progress.stats.update(delta_time);
    game.player_progress.play_time += delta_time;

    for state in game.player_progress.skills.skills.values_mut() {
        state.cooldown_remaining = (state.cooldown_remaining - delta_time).max(0.0);
    }

    game.player_progress
        .skills
        .active_effects
        .retain_mut(|effect| {
            effect.duration_remaining -= delta_time;
            effect.duration_remaining > 0.0
        });

    if !game.shell.visible && !game.dialogue.active {
        handle_skill_input(game, world);
    }

    let player_pos = get_player_position(game, world);

    systems::combat::update_projectiles(
        &mut game.combat_state,
        &mut game.loaded_level.enemies,
        &mut game.player_progress,
        &mut game.particle_system,
        world,
        player_pos,
        delta_time,
    );

    systems::combat::update_enemy_ai(
        &mut game.loaded_level.enemies,
        &mut game.combat_state,
        &mut game.particle_system,
        world,
        player_pos,
        delta_time,
    );

    systems::combat::check_melee_combat(
        &mut game.loaded_level.enemies,
        &mut game.player_progress,
        &mut game.particle_system,
        world,
        player_pos,
        delta_time,
    );

    handle_enemy_deaths(game, world);

    if let Some((item_type, quantity)) =
        systems::level_loader::check_item_pickup(&mut game.loaded_level, world, player_pos)
    {
        handle_item_pickup(game, item_type, quantity, player_pos);
    }

    if ((game.game_time * 10.0) as u32).is_multiple_of(2) {
        systems::level_loader::update_item_bobbing(&mut game.loaded_level, world, game.game_time);
    }

    if let Some(target_level) = systems::level_loader::check_portal_collision(
        &game.loaded_level,
        world,
        player_pos,
        game.player_progress.inventory.has_item(ItemType::Key, 1),
    ) {
        transition_to_level(game, world, target_level);
    }

    systems::particles::update_particles(&mut game.particle_system, world, delta_time);

    run_transform_systems(world);

    if game.player_progress.stats.is_dead() {
        game.is_dead = true;
        game.player_progress.deaths += 1;
    }

    systems::flashlight::update_flashlight(game, world);

    systems::dialogue::dialogue_system(game, world);

    run_navmesh_systems(world);

    systems::audio::audio_system(game, world);

    physics_debug_draw_system(world);
}

fn get_player_position(game: &ImmersiveSim, world: &World) -> Vec3 {
    game.player_entity
        .and_then(|entity| world.get_global_transform(entity))
        .map(|t| t.translation())
        .unwrap_or(Vec3::zeros())
}

fn handle_skill_input(game: &mut ImmersiveSim, world: &mut World) {
    let camera_entity = match game.camera_entity {
        Some(e) => e,
        None => return,
    };

    let skill_keys = [
        (KeyCode::Digit1, SkillType::Fireball, 0),
        (KeyCode::Digit2, SkillType::IceBlast, 1),
        (KeyCode::Digit3, SkillType::LightningBolt, 2),
        (KeyCode::Digit4, SkillType::Dash, 3),
        (KeyCode::Digit5, SkillType::Shield, 4),
        (KeyCode::Digit6, SkillType::Heal, 5),
        (KeyCode::Digit7, SkillType::Blink, 6),
        (KeyCode::Digit8, SkillType::Explosion, 7),
    ];

    for (key, skill_type, index) in skill_keys {
        let pressed = world.resources.input.keyboard.is_key_pressed(key);
        let was_pressed = game.skill_keys_pressed[index];

        if pressed && !was_pressed {
            systems::combat::use_skill(
                skill_type,
                &mut game.player_progress,
                &mut game.combat_state,
                &mut game.particle_system,
                world,
                camera_entity,
            );
        }

        game.skill_keys_pressed[index] = pressed;
    }
}

fn handle_enemy_deaths(game: &mut ImmersiveSim, world: &mut World) {
    let mut loot_drops: Vec<(Vec3, ItemType, usize)> = Vec::new();
    let mut experience_gained: Vec<u32> = Vec::new();
    let mut enemies_to_despawn: Vec<(Entity, Vec3)> = Vec::new();

    for enemy in &mut game.loaded_level.enemies {
        if enemy.is_dead() && enemy.entity.id != 0 {
            let enemy_pos = world
                .get_local_transform(enemy.entity)
                .map(|t| t.translation)
                .unwrap_or(enemy.home_position);

            let def = data::enemies::get_enemy_definition(enemy.enemy_type);
            if let Some(def) = def {
                experience_gained.push(def.experience_value);
            }

            if let Some((item_type, quantity)) = systems::combat::get_loot_drop(enemy.enemy_type) {
                loot_drops.push((enemy_pos, item_type, quantity));
            }

            enemies_to_despawn.push((enemy.entity, enemy_pos));
            enemy.entity = Entity {
                id: 0,
                generation: 0,
            };
        }
    }

    let player_pos = get_player_position(game, world);

    for exp in experience_gained {
        let leveled = game.player_progress.stats.add_experience(exp);
        game.player_progress.enemies_killed += 1;
        if leveled {
            systems::particles::spawn_level_up_effect(&mut game.particle_system, player_pos);
        }
    }

    for (entity, enemy_pos) in enemies_to_despawn {
        systems::particles::spawn_explosion_effect(&mut game.particle_system, enemy_pos);
        world.despawn_entities(&[entity]);
    }

    for (position, item_type, quantity) in loot_drops {
        systems::level_loader::spawn_loot_item(
            &mut game.loaded_level,
            world,
            position,
            item_type,
            quantity,
        );
    }
}

fn handle_item_pickup(
    game: &mut ImmersiveSim,
    item_type: ItemType,
    quantity: usize,
    player_pos: Vec3,
) {
    game.player_progress.items_collected += quantity as u32;

    match item_type {
        ItemType::HealthPotion => {
            game.player_progress.stats.heal(50.0);
        }
        ItemType::ManaPotion => {
            game.player_progress.stats.restore_mana(50.0);
        }
        ItemType::SpeedPotion => {
            game.player_progress
                .skills
                .active_effects
                .push(data::skills::ActiveEffect {
                    effect_type: data::skills::EffectType::SpeedBoost,
                    duration_remaining: 10.0,
                    strength: 1.5,
                });
        }
        ItemType::Coin => {
            game.player_progress.inventory.gold += quantity as u32;
        }
        ItemType::Gem => {
            game.player_progress.inventory.gold += (quantity as u32) * 100;
        }
        ItemType::Scroll => {
            let locked_skills = [
                SkillType::IceBlast,
                SkillType::LightningBolt,
                SkillType::Blink,
                SkillType::Explosion,
            ];
            for skill in locked_skills {
                if let Some(state) = game.player_progress.skills.skills.get_mut(&skill) {
                    if !state.unlocked {
                        state.unlocked = true;
                        break;
                    }
                } else {
                    game.player_progress.skills.skills.insert(
                        skill,
                        data::skills::SkillState {
                            unlocked: true,
                            cooldown_remaining: 0.0,
                            level: 1,
                        },
                    );
                    break;
                }
            }
        }
        _ => {
            game.player_progress.inventory.add_item(item_type, quantity);
        }
    }

    systems::particles::spawn_pickup_effect(&mut game.particle_system, player_pos);
}

fn transition_to_level(
    game: &mut ImmersiveSim,
    world: &mut World,
    target_level: data::levels::LevelId,
) {
    systems::level_loader::unload_level(world, &mut game.loaded_level);

    game.loaded_level = systems::level_loader::load_level(world, target_level);
    game.current_level = target_level;

    let level_def = data::levels::get_level(target_level);
    if let Some(player_entity) = game.player_entity {
        if let Some(transform) = world.get_local_transform_mut(player_entity) {
            transform.translation = level_def.player_spawn;
        }
        world.mark_local_transform_dirty(player_entity);

        let rigid_body_handle = world.get_rigid_body(player_entity).and_then(|rb| rb.handle);
        if let Some(handle) = rigid_body_handle
            && let Some(rigid_body) = world
                .resources
                .physics
                .rigid_body_set
                .get_mut(handle.into())
        {
            rigid_body.set_translation(
                rapier3d::prelude::Vector::new(
                    level_def.player_spawn.x,
                    level_def.player_spawn.y,
                    level_def.player_spawn.z,
                ),
                true,
            );
            rigid_body.set_linvel(rapier3d::prelude::Vector::new(0.0, 0.0, 0.0), true);
        }
    }
}

fn teleport_player_to_camera(game: &ImmersiveSim, world: &mut World) {
    let Some(camera_entity) = game.camera_entity else {
        return;
    };
    let Some(player_entity) = game.player_entity else {
        return;
    };

    let camera_global_pos = world
        .get_global_transform(camera_entity)
        .map(|t| t.translation())
        .unwrap_or(Vec3::zeros());

    if let Some(transform) = world.get_local_transform_mut(player_entity) {
        transform.translation = camera_global_pos;
    }
    world.mark_local_transform_dirty(player_entity);

    let rigid_body_handle = world.get_rigid_body(player_entity).and_then(|rb| rb.handle);

    if let Some(handle) = rigid_body_handle
        && let Some(rigid_body) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(handle.into())
    {
        rigid_body.set_translation(
            rapier3d::prelude::Vector::new(
                camera_global_pos.x,
                camera_global_pos.y,
                camera_global_pos.z,
            ),
            true,
        );
        rigid_body.set_linvel(rapier3d::prelude::Vector::new(0.0, 0.0, 0.0), true);
    }
}

fn gameplay_ui(game: &mut ImmersiveSim, world: &mut World, ctx: &egui::Context) {
    if game.is_dead {
        if systems::game_hud::draw_death_screen(ctx) {
            respawn_player(game, world);
        }
        return;
    }

    if game.dialogue.active {
        dialogue_ui(game, ctx);
        return;
    }

    systems::game_hud::draw_game_hud(&game.player_progress, ctx);
    crosshair_ui(ctx);
}

fn respawn_player(game: &mut ImmersiveSim, world: &mut World) {
    game.is_dead = false;
    game.player_progress.stats.health = game.player_progress.stats.max_health;
    game.player_progress.stats.mana = game.player_progress.stats.max_mana;

    let spawn_pos = data::levels::get_level(game.current_level).player_spawn;

    if let Some(player_entity) = game.player_entity {
        if let Some(transform) = world.get_local_transform_mut(player_entity) {
            transform.translation = spawn_pos;
        }
        world.mark_local_transform_dirty(player_entity);

        let rigid_body_handle = world.get_rigid_body(player_entity).and_then(|rb| rb.handle);
        if let Some(handle) = rigid_body_handle
            && let Some(rigid_body) = world
                .resources
                .physics
                .rigid_body_set
                .get_mut(handle.into())
        {
            rigid_body.set_translation(
                rapier3d::prelude::Vector::new(spawn_pos.x, spawn_pos.y, spawn_pos.z),
                true,
            );
            rigid_body.set_linvel(rapier3d::prelude::Vector::new(0.0, 0.0, 0.0), true);
        }
    }
}

fn crosshair_ui(ctx: &egui::Context) {
    #[allow(deprecated)]
    let screen_rect = ctx.screen_rect();
    let center = screen_rect.center();

    egui::Area::new(egui::Id::new("crosshair"))
        .fixed_pos(center - egui::vec2(10.0, 10.0))
        .show(ctx, |ui| {
            let painter = ui.painter();
            let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180);
            let stroke = egui::Stroke::new(2.0, color);

            painter.line_segment(
                [
                    egui::pos2(center.x - 8.0, center.y),
                    egui::pos2(center.x - 3.0, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(center.x + 3.0, center.y),
                    egui::pos2(center.x + 8.0, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(center.x, center.y - 8.0),
                    egui::pos2(center.x, center.y - 3.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(center.x, center.y + 3.0),
                    egui::pos2(center.x, center.y + 8.0),
                ],
                stroke,
            );
        });
}

fn dialogue_ui(game: &mut ImmersiveSim, ctx: &egui::Context) {
    if game.dialogue.current_node >= game.dialogue.nodes.len() {
        return;
    }

    let current_line = game.dialogue.current_line;
    let lines: Vec<_> = game.dialogue.nodes[game.dialogue.current_node]
        .lines
        .iter()
        .map(|line| (line.speaker.clone(), line.text.clone()))
        .collect();
    let choices: Vec<_> = game.dialogue.nodes[game.dialogue.current_node]
        .choices
        .iter()
        .map(|choice| choice.text.clone())
        .collect();

    let mut selected_choice: Option<usize> = None;

    egui::TopBottomPanel::bottom("dialogue_panel")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 230))
                .inner_margin(egui::Margin::same(20)),
        )
        .min_height(150.0)
        .show(ctx, |ui| {
            if current_line < lines.len() {
                let (speaker, text) = &lines[current_line];

                ui.label(
                    egui::RichText::new(speaker)
                        .size(16.0)
                        .color(egui::Color32::YELLOW)
                        .strong(),
                );

                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new(text)
                        .size(18.0)
                        .color(egui::Color32::WHITE),
                );

                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("[Space/Click to continue]")
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
            } else if !choices.is_empty() {
                ui.label(
                    egui::RichText::new("Choose a response:")
                        .size(16.0)
                        .color(egui::Color32::YELLOW)
                        .strong(),
                );

                ui.add_space(10.0);

                for (index, choice_text) in choices.iter().enumerate() {
                    let button_text = format!("{}. {}", index + 1, choice_text);
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(&button_text).size(16.0))
                                .min_size(egui::vec2(300.0, 30.0)),
                        )
                        .clicked()
                    {
                        selected_choice = Some(index);
                    }
                }
            }
        });

    if let Some(choice_index) = selected_choice {
        systems::dialogue::select_dialogue_choice(game, choice_index);
    }
}

fn pause_menu_ui(game: &mut ImmersiveSim, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.3);

                ui.heading(
                    egui::RichText::new("PAUSED")
                        .size(36.0)
                        .color(egui::Color32::WHITE),
                );

                ui.add_space(40.0);

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Resume").size(20.0))
                            .min_size(egui::vec2(150.0, 40.0)),
                    )
                    .clicked()
                {
                    game.screen = GameScreen::Gameplay;
                }

                ui.add_space(15.0);

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Quit").size(20.0))
                            .min_size(egui::vec2(150.0, 40.0)),
                    )
                    .clicked()
                {
                    std::process::exit(0);
                }
            });
        });
}

fn loading_ui(game: &ImmersiveSim, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 255)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.4);

                ui.heading(
                    egui::RichText::new("LOADING")
                        .size(36.0)
                        .color(egui::Color32::WHITE),
                );

                ui.add_space(30.0);

                let (status_text, progress) = if !game.level_spawned {
                    ("Loading level...", 0.0)
                } else {
                    let total = game.loading.total_textures.max(1) as f32;
                    let loaded = game.loading.loaded_textures as f32;
                    let progress = loaded / total;
                    ("Loading textures...", progress)
                };

                ui.label(
                    egui::RichText::new(status_text)
                        .size(18.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );

                ui.add_space(20.0);

                let progress_bar = egui::ProgressBar::new(progress)
                    .desired_width(300.0)
                    .animate(true);
                ui.add(progress_bar);

                if game.level_spawned && game.loading.total_textures > 0 {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{} / {}",
                            game.loading.loaded_textures, game.loading.total_textures
                        ))
                        .size(14.0)
                        .color(egui::Color32::GRAY),
                    );
                }
            });
        });
}
