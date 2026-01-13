pub use nightshade::plugin::{EngineCommand, EngineEvent, Primitive};

#[cfg(target_arch = "wasm32")]
pub use nightshade::plugin::{
    drain_engine_events, log,
    spawn_cone, spawn_cube, spawn_cylinder, spawn_plane, spawn_primitive, spawn_sphere,
    despawn_entity, set_entity_position, set_entity_scale, set_entity_rotation,
    get_entity_position, get_entity_scale, get_entity_rotation,
    read_file, load_texture, load_prefab, set_entity_material,
    next_request_id, send_engine_command,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EnemyType {
    Slime,
    Skeleton,
    Dragon,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ItemType {
    HealthPotion,
    ManaPotion,
    Sword,
    Shield,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameCommand {
    SpawnEnemy {
        enemy_type: EnemyType,
        x: f32,
        y: f32,
        z: f32,
        request_id: u64,
    },
    DespawnEnemy {
        enemy_id: u64,
    },
    DamageEnemy {
        enemy_id: u64,
        damage: u32,
    },
    SpawnItem {
        item_type: ItemType,
        x: f32,
        y: f32,
        z: f32,
        request_id: u64,
    },
    GivePlayerItem {
        item_type: ItemType,
    },
    SetPlayerHealth {
        health: u32,
    },
    SetPlayerScore {
        score: u64,
    },
    TriggerGameEvent {
        event_name: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEvent {
    EnemySpawned {
        request_id: u64,
        enemy_id: u64,
        enemy_type: EnemyType,
    },
    EnemyDied {
        enemy_id: u64,
        enemy_type: EnemyType,
    },
    EnemyDamaged {
        enemy_id: u64,
        remaining_health: u32,
    },
    ItemSpawned {
        request_id: u64,
        item_id: u64,
        item_type: ItemType,
    },
    ItemCollected {
        item_id: u64,
        item_type: ItemType,
    },
    PlayerHealthChanged {
        health: u32,
        max_health: u32,
    },
    PlayerScoreChanged {
        score: u64,
    },
    PlayerDied,
    GameEventTriggered {
        event_name: String,
    },
    WaveStarted {
        wave_number: u32,
    },
    WaveCompleted {
        wave_number: u32,
    },
}

impl GameCommand {
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        postcard::from_bytes(bytes).ok()
    }
}

impl GameEvent {
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        postcard::from_bytes(bytes).ok()
    }
}

#[cfg(target_arch = "wasm32")]
mod guest {
    use super::{EnemyType, GameCommand, GameEvent, ItemType};
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicU64, Ordering};

    unsafe extern "C" {
        fn host_send_game_command(ptr: *const u8, len: u32);
    }

    static GAME_REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

    thread_local! {
        static PENDING_GAME_EVENTS: RefCell<Vec<GameEvent>> = const { RefCell::new(Vec::new()) };
        static GAME_EVENT_BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    }

    pub fn next_game_request_id() -> u64 {
        GAME_REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub fn send_game_command(command: GameCommand) {
        let Ok(bytes) = command.to_bytes() else {
            return;
        };
        unsafe {
            host_send_game_command(bytes.as_ptr(), bytes.len() as u32);
        }
    }

    pub fn spawn_enemy(enemy_type: EnemyType, x: f32, y: f32, z: f32) -> u64 {
        let request_id = next_game_request_id();
        send_game_command(GameCommand::SpawnEnemy {
            enemy_type,
            x,
            y,
            z,
            request_id,
        });
        request_id
    }

    pub fn spawn_slime(x: f32, y: f32, z: f32) -> u64 {
        spawn_enemy(EnemyType::Slime, x, y, z)
    }

    pub fn spawn_skeleton(x: f32, y: f32, z: f32) -> u64 {
        spawn_enemy(EnemyType::Skeleton, x, y, z)
    }

    pub fn spawn_dragon(x: f32, y: f32, z: f32) -> u64 {
        spawn_enemy(EnemyType::Dragon, x, y, z)
    }

    pub fn despawn_enemy(enemy_id: u64) {
        send_game_command(GameCommand::DespawnEnemy { enemy_id });
    }

    pub fn damage_enemy(enemy_id: u64, damage: u32) {
        send_game_command(GameCommand::DamageEnemy { enemy_id, damage });
    }

    pub fn spawn_item(item_type: ItemType, x: f32, y: f32, z: f32) -> u64 {
        let request_id = next_game_request_id();
        send_game_command(GameCommand::SpawnItem {
            item_type,
            x,
            y,
            z,
            request_id,
        });
        request_id
    }

    pub fn spawn_health_potion(x: f32, y: f32, z: f32) -> u64 {
        spawn_item(ItemType::HealthPotion, x, y, z)
    }

    pub fn spawn_sword(x: f32, y: f32, z: f32) -> u64 {
        spawn_item(ItemType::Sword, x, y, z)
    }

    pub fn give_player_item(item_type: ItemType) {
        send_game_command(GameCommand::GivePlayerItem { item_type });
    }

    pub fn set_player_health(health: u32) {
        send_game_command(GameCommand::SetPlayerHealth { health });
    }

    pub fn set_player_score(score: u64) {
        send_game_command(GameCommand::SetPlayerScore { score });
    }

    pub fn trigger_game_event(event_name: &str) {
        send_game_command(GameCommand::TriggerGameEvent {
            event_name: event_name.to_string(),
        });
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn game_plugin_alloc(size: u32) -> *mut u8 {
        GAME_EVENT_BUFFER.with_borrow_mut(|buffer| {
            buffer.resize(size as usize, 0);
            buffer.as_mut_ptr()
        })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn game_plugin_receive_event(ptr: *const u8, len: u32) {
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
        if let Some(event) = GameEvent::from_bytes(bytes) {
            PENDING_GAME_EVENTS.with_borrow_mut(|events| {
                events.push(event);
            });
        }
    }

    pub fn drain_game_events() -> Vec<GameEvent> {
        PENDING_GAME_EVENTS.with_borrow_mut(|events| std::mem::take(events))
    }
}

#[cfg(target_arch = "wasm32")]
pub use guest::*;
