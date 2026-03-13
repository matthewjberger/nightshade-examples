use nightshade::ecs::world::Entity;
use nightshade::plugin_runtime::{
    Caller, Linker, PluginRuntime, PluginRuntimeConfig, PluginState, read_plugin_memory,
};
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::texture_cache_add_reference;
use nightshade::ecs::material::resources::material_registry_insert;
use plugins_shared::{EnemyType, GameCommand, GameEvent, ItemType};
use std::collections::HashMap;
use std::path::PathBuf;

struct EnemyData {
    entity: Entity,
    enemy_type: EnemyType,
    health: u32,
}

struct ItemData;

struct GameState {
    enemies: HashMap<u64, EnemyData>,
    items: HashMap<u64, ItemData>,
    player_health: u32,
    player_max_health: u32,
    player_score: u64,
    next_enemy_id: u64,
    next_item_id: u64,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            enemies: HashMap::new(),
            items: HashMap::new(),
            player_health: 100,
            player_max_health: 100,
            player_score: 0,
            next_enemy_id: 1,
            next_item_id: 1,
        }
    }
}

#[derive(Default)]
struct PluginsDemo {
    runtime: Option<PluginRuntime>,
    game_state: GameState,
}

impl State for PluginsDemo {
    fn title(&self) -> &str {
        "Plugins Demo - Engine & Game Level"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = true;
        spawn_sun(world);
        spawn_camera(world);

        let ground = spawn_plane_at(world, Vec3::new(0.0, 0.0, 0.0));
        if let Some(transform) = world.mutate_local_transform(ground) {
            transform.scale = Vec3::new(20.0, 1.0, 20.0);
        }
        let ground_material = format!("Ground_{}", ground.id);
        texture_cache_add_reference(&mut world.resources.texture_cache, "checkerboard");
        material_registry_insert(
            &mut world.resources.material_registry,
            ground_material.clone(),
            Material {
                base_color: [0.2, 0.5, 0.2, 1.0],
                base_texture: Some("checkerboard".to_string()),
                ..Default::default()
            },
        );
        if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&ground_material) {
            world.resources.material_registry.registry.add_reference(index);
        };
        world.core.set_material_ref(ground, MaterialRef::new(ground_material));

        let plugins_dir = PathBuf::from("plugins");
        let config = PluginRuntimeConfig {
            plugins_base_path: plugins_dir.clone(),
            ..Default::default()
        };

        match PluginRuntime::new(config) {
            Ok(mut runtime) => {
                runtime.with_custom_linker(|linker: &mut Linker<PluginState>, _engine| {
                    linker.func_wrap(
                        "env",
                        "host_send_game_command",
                        |mut caller: Caller<'_, PluginState>, ptr: u32, len: u32| {
                            if let Some(bytes) = read_plugin_memory(&mut caller, ptr, len) {
                                caller.data_mut().push_custom_command(bytes);
                            } else {
                                tracing::error!("Plugin game command buffer out of bounds");
                            }
                        },
                    )?;
                    Ok(())
                });

                tracing::info!(
                    "Engine-level plugins: plugins/engine (depend on nightshade directly)"
                );
                tracing::info!(
                    "App-level plugins: plugins/app (depend on shared crate with game concepts)"
                );

                let engine_plugins_dir = plugins_dir.join("engine");
                let app_plugins_dir = plugins_dir.join("app");

                if let Err(error) = runtime.load_plugins_from_directory(&engine_plugins_dir) {
                    tracing::error!("Failed to load engine plugins: {}", error);
                }
                if let Err(error) = runtime.load_plugins_from_directory(&app_plugins_dir) {
                    tracing::error!("Failed to load app plugins: {}", error);
                }

                runtime.call_on_init(world);

                let game_commands: Vec<_> = runtime
                    .drain_custom_commands()
                    .into_iter()
                    .filter_map(|(plugin_id, bytes)| {
                        GameCommand::from_bytes(&bytes).map(|cmd| (plugin_id, cmd))
                    })
                    .collect();

                process_game_commands(world, &mut runtime, &mut self.game_state, game_commands);

                self.runtime = Some(runtime);
            }
            Err(error) => {
                tracing::error!("Failed to create plugin runtime: {}", error);
            }
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        fly_camera_system(world);

        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };

        runtime.run_frame(world);

        let game_commands: Vec<_> = runtime
            .drain_custom_commands()
            .into_iter()
            .filter_map(|(plugin_id, bytes)| {
                GameCommand::from_bytes(&bytes).map(|cmd| (plugin_id, cmd))
            })
            .collect();

        process_game_commands(
            world,
            runtime,
            &mut self.game_state,
            game_commands,
        );
    }

    fn on_keyboard_input(&mut self, _world: &mut World, key_code: KeyCode, state: KeyState) {
        if let Some(runtime) = &mut self.runtime {
            runtime.queue_keyboard_event(key_code, state);
        }
    }

    fn on_mouse_input(&mut self, _world: &mut World, state: ElementState, button: MouseButton) {
        if let Some(runtime) = &mut self.runtime {
            runtime.queue_mouse_event(state, button);
        }
    }
}

fn dispatch_game_event_to_plugin(runtime: &mut PluginRuntime, plugin_id: u64, event: &GameEvent) {
    if let Ok(bytes) = event.to_bytes() {
        runtime.dispatch_custom_event_to_plugin(
            plugin_id,
            &bytes,
            "game_plugin_alloc",
            "game_plugin_receive_event",
        );
    }
}

fn dispatch_game_event_to_all(runtime: &mut PluginRuntime, event: &GameEvent) {
    if let Ok(bytes) = event.to_bytes() {
        runtime.dispatch_custom_event_to_all_unfiltered(
            &bytes,
            "game_plugin_alloc",
            "game_plugin_receive_event",
        );
    }
}

fn process_game_commands(
    world: &mut World,
    runtime: &mut PluginRuntime,
    game_state: &mut GameState,
    commands: Vec<(u64, GameCommand)>,
) {
    for (plugin_id, command) in commands {
        match command {
            GameCommand::SpawnEnemy {
                enemy_type,
                x,
                y,
                z,
                request_id,
            } => {
                let position = Vec3::new(x, y, z);
                let entity = spawn_enemy_entity(world, &enemy_type, position);
                let enemy_id = game_state.next_enemy_id;
                game_state.next_enemy_id += 1;
                let max_health = get_enemy_max_health(&enemy_type);

                game_state.enemies.insert(
                    enemy_id,
                    EnemyData {
                        entity,
                        enemy_type: enemy_type.clone(),
                        health: max_health,
                    },
                );

                let event = GameEvent::EnemySpawned {
                    request_id,
                    enemy_id,
                    enemy_type,
                };
                dispatch_game_event_to_plugin(runtime, plugin_id, &event);
            }
            GameCommand::DespawnEnemy { enemy_id } => {
                if let Some(enemy_data) = game_state.enemies.remove(&enemy_id) {
                    world.queue_command(WorldCommand::DespawnRecursive {
                        entity: enemy_data.entity,
                    });
                    let event = GameEvent::EnemyDied {
                        enemy_id,
                        enemy_type: enemy_data.enemy_type,
                    };
                    dispatch_game_event_to_all(runtime, &event);
                }
            }
            GameCommand::DamageEnemy { enemy_id, damage } => {
                if let Some(enemy_data) = game_state.enemies.get_mut(&enemy_id) {
                    enemy_data.health = enemy_data.health.saturating_sub(damage);
                    let remaining_health = enemy_data.health;
                    let event = GameEvent::EnemyDamaged {
                        enemy_id,
                        remaining_health,
                    };
                    dispatch_game_event_to_all(runtime, &event);
                }
            }
            GameCommand::SpawnItem {
                item_type,
                x,
                y,
                z,
                request_id,
            } => {
                let position = Vec3::new(x, y, z);
                let _entity = spawn_item_entity(world, &item_type, position);
                let item_id = game_state.next_item_id;
                game_state.next_item_id += 1;

                game_state.items.insert(item_id, ItemData);

                let event = GameEvent::ItemSpawned {
                    request_id,
                    item_id,
                    item_type,
                };
                dispatch_game_event_to_plugin(runtime, plugin_id, &event);
            }
            GameCommand::GivePlayerItem { item_type } => {
                let event = GameEvent::ItemCollected {
                    item_id: 0,
                    item_type,
                };
                dispatch_game_event_to_all(runtime, &event);
            }
            GameCommand::SetPlayerHealth { health } => {
                game_state.player_health = health.min(game_state.player_max_health);
                let event = GameEvent::PlayerHealthChanged {
                    health: game_state.player_health,
                    max_health: game_state.player_max_health,
                };
                dispatch_game_event_to_all(runtime, &event);
            }
            GameCommand::SetPlayerScore { score } => {
                game_state.player_score = score;
                let event = GameEvent::PlayerScoreChanged { score };
                dispatch_game_event_to_all(runtime, &event);
            }
            GameCommand::TriggerGameEvent { event_name } => {
                if let Some(wave_str) = event_name
                    .strip_prefix("wave_")
                    .and_then(|s| s.strip_suffix("_start"))
                {
                    if let Ok(wave_number) = wave_str.parse::<u32>() {
                        let event = GameEvent::WaveStarted { wave_number };
                        dispatch_game_event_to_all(runtime, &event);
                    }
                } else if let Some(wave_str) = event_name
                    .strip_prefix("wave_")
                    .and_then(|s| s.strip_suffix("_complete"))
                {
                    if let Ok(wave_number) = wave_str.parse::<u32>() {
                        let event = GameEvent::WaveCompleted { wave_number };
                        dispatch_game_event_to_all(runtime, &event);
                    }
                } else {
                    let event = GameEvent::GameEventTriggered { event_name };
                    dispatch_game_event_to_all(runtime, &event);
                }
            }
        }
    }
}

fn get_enemy_max_health(enemy_type: &EnemyType) -> u32 {
    match enemy_type {
        EnemyType::Slime => 20,
        EnemyType::Skeleton => 50,
        EnemyType::Dragon => 200,
    }
}

fn spawn_enemy_entity(world: &mut World, enemy_type: &EnemyType, position: Vec3) -> Entity {
    match enemy_type {
        EnemyType::Slime => {
            let entity = spawn_sphere_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.5, 0.5, 0.5);
            }
            let material_name = format!("Slime_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.2, 0.8, 0.2, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&material_name) {
            world.resources.material_registry.registry.add_reference(index);
        };
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            entity
        }
        EnemyType::Skeleton => {
            let entity = spawn_cylinder_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.3, 1.0, 0.3);
            }
            let material_name = format!("Skeleton_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.9, 0.9, 0.85, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&material_name) {
            world.resources.material_registry.registry.add_reference(index);
        };
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            entity
        }
        EnemyType::Dragon => {
            let entity = spawn_cube_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(2.0, 2.0, 3.0);
            }
            let material_name = format!("Dragon_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.8, 0.2, 0.1, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&material_name) {
            world.resources.material_registry.registry.add_reference(index);
        };
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            entity
        }
    }
}

fn spawn_item_entity(world: &mut World, item_type: &ItemType, position: Vec3) -> Entity {
    match item_type {
        ItemType::HealthPotion => {
            let entity = spawn_sphere_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.3, 0.3, 0.3);
            }
            let material_name = format!("HealthPotion_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [1.0, 0.2, 0.2, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&material_name) {
            world.resources.material_registry.registry.add_reference(index);
        };
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            entity
        }
        ItemType::ManaPotion => {
            let entity = spawn_sphere_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.3, 0.3, 0.3);
            }
            let material_name = format!("ManaPotion_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.2, 0.2, 1.0, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&material_name) {
            world.resources.material_registry.registry.add_reference(index);
        };
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            entity
        }
        ItemType::Sword => {
            let entity = spawn_cube_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.1, 0.8, 0.1);
            }
            let material_name = format!("Sword_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.7, 0.7, 0.8, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&material_name) {
            world.resources.material_registry.registry.add_reference(index);
        };
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            entity
        }
        ItemType::Shield => {
            let entity = spawn_cube_at(world, position);
            if let Some(transform) = world.mutate_local_transform(entity) {
                transform.scale = Vec3::new(0.6, 0.6, 0.1);
            }
            let material_name = format!("Shield_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.6, 0.4, 0.2, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world.resources.material_registry.registry.name_to_index.get(&material_name) {
            world.resources.material_registry.registry.add_reference(index);
        };
            world.core.set_material_ref(entity, MaterialRef::new(material_name));
            entity
        }
    }
}

fn spawn_camera(world: &mut World) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | CAMERA,
        1,
    )[0];

    world.core.set_name(entity, Name("Camera".to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::new(0.0, 5.0, 10.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.core.set_local_transform_dirty(entity, LocalTransformDirty);
    world.core.set_global_transform(entity, GlobalTransform::default());
    world.core.set_camera(
        entity,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                y_fov_rad: 60.0_f32.to_radians(),
                aspect_ratio: None,
                z_near: 0.1,
                z_far: Some(1000.0),
            }),
            smoothing: Some(Smoothing::default()),
        },
    );

    world.resources.active_camera = Some(entity);
    entity
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(PluginsDemo::default())
}
