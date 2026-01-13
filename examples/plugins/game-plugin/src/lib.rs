#[cfg(target_arch = "wasm32")]
use plugins_shared::{
    damage_enemy, despawn_enemy, drain_engine_events, drain_game_events, give_player_item, log,
    set_player_health, set_player_score, spawn_dragon, spawn_health_potion, spawn_skeleton,
    spawn_slime, trigger_game_event, EngineEvent, EnemyType, GameEvent, ItemType,
};
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
struct Enemy {
    health: u32,
}

#[cfg(target_arch = "wasm32")]
struct GamePluginState {
    initialized: bool,
    wave_number: u32,
    enemies_alive: u32,
    enemies: HashMap<u64, Enemy>,
    player_score: u64,
    spawn_timer: f32,
    wave_active: bool,
}

#[cfg(target_arch = "wasm32")]
impl Default for GamePluginState {
    fn default() -> Self {
        Self {
            initialized: false,
            wave_number: 0,
            enemies_alive: 0,
            enemies: HashMap::new(),
            player_score: 0,
            spawn_timer: 0.0,
            wave_active: false,
        }
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static STATE: RefCell<GamePluginState> = RefCell::new(GamePluginState::default());
}

#[cfg(target_arch = "wasm32")]
fn with_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut GamePluginState) -> R,
{
    STATE.with_borrow_mut(f)
}

#[cfg(target_arch = "wasm32")]
fn get_enemy_max_health(enemy_type: &EnemyType) -> u32 {
    match enemy_type {
        EnemyType::Slime => 20,
        EnemyType::Skeleton => 50,
        EnemyType::Dragon => 200,
    }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    with_state(|state| {
        if state.initialized {
            return;
        }
        state.initialized = true;

        log("[Game Plugin] Initialized - demonstrates app-level plugin pattern");
        log("[Game Plugin] This plugin depends on the shared crate, NOT nightshade directly");
        log("[Game Plugin] It uses game concepts: enemies, items, waves, scores");
        log("[Game Plugin] Press 'W' to start a wave, 'K' to damage enemies, 'H' for health potion");

        set_player_health(100);
        set_player_score(0);
    });
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn on_frame() {
    for event in drain_engine_events() {
        match event {
            EngineEvent::FrameStart { delta_time, .. } => {
                with_state(|state| {
                    if state.wave_active {
                        state.spawn_timer += delta_time;

                        if state.spawn_timer >= 2.0 && state.enemies_alive < 5 {
                            state.spawn_timer = 0.0;

                            let x = (state.enemies_alive as f32) * 2.0 - 4.0;
                            let z = -8.0;

                            match state.wave_number {
                                1 => {
                                    spawn_slime(x, 0.5, z);
                                    log(&format!(
                                        "[Game Plugin] Wave {} - Spawning slime at ({}, 0.5, {})",
                                        state.wave_number, x, z
                                    ));
                                }
                                2 => {
                                    if state.enemies_alive % 2 == 0 {
                                        spawn_slime(x, 0.5, z);
                                    } else {
                                        spawn_skeleton(x, 1.0, z);
                                    }
                                }
                                _ => {
                                    if state.enemies_alive == 0 {
                                        spawn_dragon(0.0, 2.0, -10.0);
                                        log("[Game Plugin] BOSS WAVE - Dragon spawned!");
                                    } else {
                                        spawn_skeleton(x, 1.0, z);
                                    }
                                }
                            }
                        }
                    }
                });
            }
            EngineEvent::KeyPressed { key_code } => match key_code {
                17 => {
                    with_state(|state| {
                        if !state.wave_active {
                            state.wave_number += 1;
                            state.wave_active = true;
                            state.spawn_timer = 0.0;
                            trigger_game_event(&format!("wave_{}_start", state.wave_number));
                            log(&format!(
                                "[Game Plugin] Starting wave {}!",
                                state.wave_number
                            ));
                        }
                    });
                }
                37 => {
                    with_state(|state| {
                        let enemy_ids: Vec<u64> = state.enemies.keys().copied().collect();
                        for enemy_id in enemy_ids {
                            damage_enemy(enemy_id, 25);
                            log(&format!(
                                "[Game Plugin] Damaged enemy {} for 25 damage",
                                enemy_id
                            ));
                        }
                    });
                }
                35 => {
                    give_player_item(ItemType::HealthPotion);
                    log("[Game Plugin] Giving player a health potion");
                }
                31 => {
                    give_player_item(ItemType::Sword);
                    log("[Game Plugin] Giving player a sword");
                }
                23 => {
                    with_state(|state| {
                        let x = (state.enemies_alive as f32) * 2.0;
                        spawn_health_potion(x, 0.5, -3.0);
                        log("[Game Plugin] Spawned health potion");
                    });
                }
                _ => {}
            },
            _ => {}
        }
    }

    for event in drain_game_events() {
        match event {
            GameEvent::EnemySpawned {
                enemy_id,
                enemy_type,
                ..
            } => {
                with_state(|state| {
                    let health = get_enemy_max_health(&enemy_type);
                    state.enemies.insert(enemy_id, Enemy { health });
                    state.enemies_alive += 1;
                    log(&format!(
                        "[Game Plugin] Enemy {:?} spawned (id: {}, health: {})",
                        enemy_type, enemy_id, health
                    ));
                });
            }
            GameEvent::EnemyDamaged {
                enemy_id,
                remaining_health,
            } => {
                with_state(|state| {
                    if let Some(enemy) = state.enemies.get_mut(&enemy_id) {
                        enemy.health = remaining_health;
                        log(&format!(
                            "[Game Plugin] Enemy {} took damage, {} health remaining",
                            enemy_id, remaining_health
                        ));

                        if remaining_health == 0 {
                            despawn_enemy(enemy_id);
                        }
                    }
                });
            }
            GameEvent::EnemyDied {
                enemy_id,
                enemy_type,
            } => {
                with_state(|state| {
                    state.enemies.remove(&enemy_id);
                    state.enemies_alive = state.enemies_alive.saturating_sub(1);

                    let points = match enemy_type {
                        EnemyType::Slime => 10,
                        EnemyType::Skeleton => 25,
                        EnemyType::Dragon => 100,
                    };
                    state.player_score += points;
                    set_player_score(state.player_score);

                    log(&format!(
                        "[Game Plugin] Enemy {:?} died! +{} points (total: {})",
                        enemy_type, points, state.player_score
                    ));

                    if state.enemies.is_empty() && state.wave_active {
                        state.wave_active = false;
                        trigger_game_event(&format!("wave_{}_complete", state.wave_number));
                        log(&format!(
                            "[Game Plugin] Wave {} complete! Press 'W' for next wave.",
                            state.wave_number
                        ));
                    }
                });
            }
            GameEvent::ItemCollected { item_type, .. } => {
                log(&format!("[Game Plugin] Player collected {:?}", item_type));
            }
            GameEvent::PlayerHealthChanged { health, max_health } => {
                log(&format!(
                    "[Game Plugin] Player health: {}/{}",
                    health, max_health
                ));
            }
            GameEvent::PlayerScoreChanged { score } => {
                log(&format!("[Game Plugin] Score updated: {}", score));
            }
            GameEvent::WaveStarted { wave_number } => {
                log(&format!(
                    "[Game Plugin] === WAVE {} STARTED ===",
                    wave_number
                ));
            }
            GameEvent::WaveCompleted { wave_number } => {
                log(&format!(
                    "[Game Plugin] === WAVE {} COMPLETED ===",
                    wave_number
                ));
            }
            _ => {}
        }
    }
}
