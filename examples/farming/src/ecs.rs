use nightshade::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::types::{
    ActiveBuff, CameraMode, CharacterMovementState, EnemyMaterials, EnemyType,
    FarmingAnimationIndices, GameState, HighScoreType, LineEffect, LobBomb, PlayerStats,
    TreeState, UpgradeType, ZoneType,
};

freecs::ecs! {
    GameWorld {
        entity_handle: EntityHandle => ENTITY_HANDLE,
        position: Position => POSITION,
        velocity: Velocity => VELOCITY,
        enemy: Enemy => ENEMY,
        projectile: Projectile => PROJECTILE,
        gem: Gem => GEM,
        popup: Popup => POPUP,
        shield: Shield => SHIELD,
        health_crystal: HealthCrystal => HEALTH_CRYSTAL,
        health_gem: HealthGem => HEALTH_GEM,
        tree: Tree => TREE,
        log: Log => LOG,
    }
    GameResources {
        enemy_list: Vec<freecs::Entity>,
        projectile_list: Vec<freecs::Entity>,
        gem_list: Vec<freecs::Entity>,
        popup_list: Vec<freecs::Entity>,
        health_crystal_list: Vec<freecs::Entity>,
        health_gem_list: Vec<freecs::Entity>,
        tree_list: Vec<freecs::Entity>,
        log_list: Vec<freecs::Entity>,
        spawn_timer: f32,
        enemies_spawned: u32,
        enemies_killed: u32,
        current_wave: u32,
        wave_timer: f32,
        wave_enemies_remaining: u32,
        boss_alive: bool,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EntityHandle(pub Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct Position(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Enemy {
    pub speed: f32,
    pub health: f32,
    pub enemy_type: EnemyType,
    pub xp_value: u32,
    pub shield_hits: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Projectile {
    pub damage: f32,
    pub particle_emitter: Option<Entity>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Gem {
    pub xp_value: u32,
    pub particle_emitter: Option<Entity>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HealthCrystal {
    pub health_value: f32,
    pub current_hp: f32,
    pub particle_emitter: Option<Entity>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HealthGem {
    pub health_value: f32,
    pub particle_emitter: Option<Entity>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tree {
    pub trunk_entity: Entity,
    pub foliage_entities: [Entity; 3],
    pub health: f32,
    pub max_health: f32,
    pub trunk_height: f32,
    pub state: TreeState,
    pub fall_progress: f32,
    pub fall_direction: Vec3,
    pub shrink_progress: f32,
    pub original_trunk_scale: Vec3,
    pub original_foliage_scales: [Vec3; 3],
    pub trunk_y_offset: f32,
    pub foliage_y_offsets: [f32; 3],
    pub chunk: (i32, i32),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Log {
    pub base_height: f32,
    pub rotation_offset: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Popup {
    pub text_entity: Entity,
    pub lifetime: f32,
    pub base_position: Vec3,
}

impl Default for Popup {
    fn default() -> Self {
        Self {
            text_entity: Entity {
                id: 0,
                generation: 0,
            },
            lifetime: 0.0,
            base_position: Vec3::zeros(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Shield;

#[derive(Debug, Clone)]
pub struct TreasureZone {
    pub center: Vec3,
    pub radius: f32,
    pub power_up_entity: Option<Entity>,
    pub power_up_emitter: Option<Entity>,
    pub zone_type: ZoneType,
    pub cleared: bool,
    pub activated: bool,
    pub zone_enemies: Vec<freecs::Entity>,
}

pub struct PlayerState {
    pub entity: Option<Entity>,
    pub position: Vec3,
    pub health: f32,
    pub xp: u32,
    pub level: u32,
    pub stats: PlayerStats,
    pub facing: Vec3,
    pub vertical_velocity: f32,
    pub height: f32,
    pub is_grounded: bool,
    pub damage_cooldown: f32,
    pub attack_cooldown: f32,
    pub invincibility_timer: f32,
    pub pulse_cooldown: f32,
    pub whip_cooldown: f32,
    pub whip_angle: f32,
    pub lightning_cooldown: f32,
    pub garlic_timer: f32,
    pub bomb_cooldown: f32,
    pub orb_angle: f32,
    pub regen_timer: f32,
    pub combo_count: u32,
    pub combo_timer: f32,
    pub combo_max: u32,
    pub speed_boost_timer: f32,
    pub dust_timer: f32,
    pub is_chopping: bool,
    pub chopping_tree: Option<freecs::Entity>,
    pub axe_swing_angle: f32,
    pub nearest_tree_entity: Option<freecs::Entity>,
    pub log_inventory: u32,
    pub character_movement_state: CharacterMovementState,
    pub animation_indices: FarmingAnimationIndices,
    pub current_animation: Option<usize>,
    pub was_moving: bool,
    pub character_loaded: bool,
    pub current_animation_name: String,
    pub shield_layers: Vec<(Entity, f32, f32, u32)>,
    pub shield_regen_timer: f32,
    pub upgrade_choices: Vec<UpgradeType>,
    pub selected_upgrade_index: usize,
    pub active_buffs: Vec<ActiveBuff>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            entity: None,
            position: Vec3::zeros(),
            health: crate::types::PLAYER_MAX_HEALTH,
            xp: 0,
            level: 1,
            stats: PlayerStats::default(),
            facing: Vec3::new(1.0, 0.0, 0.0),
            vertical_velocity: 0.0,
            height: 0.0,
            is_grounded: true,
            damage_cooldown: 0.0,
            attack_cooldown: 0.0,
            invincibility_timer: 0.0,
            pulse_cooldown: 0.0,
            whip_cooldown: 0.0,
            whip_angle: 0.0,
            lightning_cooldown: 0.0,
            garlic_timer: 0.0,
            bomb_cooldown: 0.0,
            orb_angle: 0.0,
            regen_timer: 0.0,
            combo_count: 0,
            combo_timer: 0.0,
            combo_max: 0,
            speed_boost_timer: 0.0,
            dust_timer: 0.0,
            is_chopping: false,
            chopping_tree: None,
            axe_swing_angle: 0.0,
            nearest_tree_entity: None,
            log_inventory: 0,
            character_movement_state: CharacterMovementState::Idle,
            animation_indices: FarmingAnimationIndices::default(),
            current_animation: None,
            was_moving: false,
            character_loaded: false,
            current_animation_name: String::from("None"),
            shield_layers: Vec::new(),
            shield_regen_timer: 0.0,
            upgrade_choices: Vec::new(),
            selected_upgrade_index: 0,
            active_buffs: Vec::new(),
        }
    }
}

pub struct CameraState {
    pub entity: Option<Entity>,
    pub mode: CameraMode,
    pub yaw: f32,
    pub transition: f32,
    pub shake: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            entity: None,
            mode: CameraMode::TopDown,
            yaw: 0.0,
            transition: 1.0,
            shake: 0.0,
        }
    }
}

pub struct GameStateData {
    pub state: GameState,
    pub time: f32,
    pub speed: f32,
    pub kill_flash_timer: f32,
    pub level_up_flash: f32,
    pub boss_kill_flash: f32,
    pub last_wave_announced: u32,
    pub high_score_kills: u32,
    pub high_score_wave: u32,
    pub high_score_time: f32,
    pub high_score_combo: u32,
    pub new_high_score_timer: f32,
    pub new_high_score_type: HighScoreType,
    pub score_popup_scale: f32,
}

impl Default for GameStateData {
    fn default() -> Self {
        Self {
            state: GameState::MainMenu,
            time: 0.0,
            speed: 1.0,
            kill_flash_timer: 0.0,
            level_up_flash: 0.0,
            boss_kill_flash: 0.0,
            last_wave_announced: 0,
            high_score_kills: 0,
            high_score_wave: 0,
            high_score_time: 0.0,
            high_score_combo: 0,
            new_high_score_timer: 0.0,
            new_high_score_type: HighScoreType::None,
            score_popup_scale: 1.0,
        }
    }
}

#[derive(Default)]
pub struct VisualEntities {
    pub ground: Option<Entity>,
    pub grass_region: Option<Entity>,
    pub grass_plane: Option<Entity>,
    pub axe: Option<Entity>,
    pub target_indicator: Option<Entity>,
    pub garlic_emitter: Option<Entity>,
    pub ambient_emitter: Option<Entity>,
    pub combo_emitter: Option<Entity>,
    pub orb_entities: Vec<Entity>,
    pub enemy_shield_entities: Vec<(freecs::Entity, Entity, f32)>,
    pub line_effects: Vec<LineEffect>,
    pub lob_bombs: Vec<LobBomb>,
}

pub struct ChunkState {
    pub loaded_chunks: HashSet<(i32, i32)>,
    pub chunk_entities: HashMap<(i32, i32), Vec<Entity>>,
    pub max_distance_traveled: f32,
    pub treasure_zones: Vec<TreasureZone>,
    pub next_zone_distance: f32,
    pub health_crystal_spawn_timer: f32,
    pub health_crystal_spawn_interval: f32,
}

impl Default for ChunkState {
    fn default() -> Self {
        Self {
            loaded_chunks: HashSet::new(),
            chunk_entities: HashMap::new(),
            max_distance_traveled: 0.0,
            treasure_zones: Vec::new(),
            next_zone_distance: 50.0,
            health_crystal_spawn_timer: 0.0,
            health_crystal_spawn_interval: 60.0,
        }
    }
}

#[derive(Default)]
pub struct MaterialCache {
    pub enemy_materials: EnemyMaterials,
    pub projectile_material_name: Option<String>,
    pub gem_material_name: Option<String>,
    pub orb_material_name: Option<String>,
}
