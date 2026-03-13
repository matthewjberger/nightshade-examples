use nightshade::ecs::input::queries::query_active_gamepad;
use nightshade::ecs::input::resources::TouchGesture;
use nightshade::ecs::lines::components::Line;
use nightshade::ecs::material::components::AlphaMode;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::particles::components::{
    ColorGradient, EmitterShape, EmitterType, ParticleEmitter,
};
use nightshade::ecs::text::TextProperties;
use nightshade::ecs::ui::state::UiStateTrait;
use nightshade::prelude::*;
use nightshade::render::wgpu::passes::geometry::UiRect;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h * 6.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + m, g + m, b + m)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Survivors::default())
}

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
    }
    GameResources {
        enemy_list: Vec<freecs::Entity>,
        projectile_list: Vec<freecs::Entity>,
        gem_list: Vec<freecs::Entity>,
        popup_list: Vec<freecs::Entity>,
        health_crystal_list: Vec<freecs::Entity>,
        health_gem_list: Vec<freecs::Entity>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyType {
    #[default]
    Normal,
    Fast,
    Tank,
    Exploder,
    Boss,
}

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
    pub speed: f32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    MaxHealth,
    Damage,
    Berserk,
    Haste,
    Invincible,
    HealthCache,
    BombCache,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffType {
    Berserk,
    Haste,
    Invincible,
}

#[derive(Debug, Clone)]
pub struct TreasureZone {
    pub center: Vec3,
    pub radius: f32,
    pub fence_entities: Vec<Entity>,
    pub power_up_entity: Option<Entity>,
    pub power_up_emitter: Option<Entity>,
    pub zone_type: ZoneType,
    pub cleared: bool,
    pub activated: bool,
    pub zone_enemies: Vec<freecs::Entity>,
}

#[derive(Debug, Clone, Copy)]
pub struct ActiveBuff {
    pub buff_type: BuffType,
    pub remaining_time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupType {
    Damage,
    CriticalDamage,
    Xp,
    Combo,
    LevelUp,
    Wave,
    BossDamage,
    PowerUp,
}

#[derive(Debug, Clone, Copy)]
pub struct Popup {
    pub text_entity: Entity,
    pub lifetime: f32,
    pub popup_type: PopupType,
    pub start_scale: f32,
    pub max_scale: f32,
    pub base_position: Vec3,
    pub velocity: Vec3,
}

impl Default for Popup {
    fn default() -> Self {
        Self {
            text_entity: Entity {
                id: 0,
                generation: 0,
            },
            lifetime: 0.0,
            popup_type: PopupType::Damage,
            start_scale: 0.0,
            velocity: Vec3::zeros(),
            max_scale: 1.0,
            base_position: Vec3::zeros(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Shield {
    pub hits_remaining: u32,
    pub max_hits: u32,
    pub regen_timer: f32,
    pub visual_entity: Option<Entity>,
}

const ARENA_SIZE: f32 = 40.0;
const GROUND_SIZE: f32 = 200.0;
const CHUNK_SIZE: f32 = 20.0;
const RENDER_DISTANCE: i32 = 3;
const PLAYER_RADIUS: f32 = 0.5;
const PLAYER_SPEED: f32 = 8.0;
const CAMERA_HEIGHT: f32 = 25.0;
const CAMERA_DISTANCE: f32 = 15.0;

const ENEMY_RADIUS: f32 = 0.4;
const ENEMY_SPEED: f32 = 3.0;
const SPAWN_INTERVAL: f32 = 0.5;
const COLLISION_DISTANCE: f32 = PLAYER_RADIUS + ENEMY_RADIUS;

const PLAYER_MAX_HEALTH: f32 = 100.0;
const ENEMY_DAMAGE: f32 = 10.0;
const DAMAGE_COOLDOWN: f32 = 0.5;

const PROJECTILE_RADIUS: f32 = 0.15;
const PROJECTILE_SPEED: f32 = 15.0;
const PROJECTILE_COOLDOWN: f32 = 0.3;
const PROJECTILE_RANGE: f32 = 20.0;
const PROJECTILE_HIT_DISTANCE: f32 = PROJECTILE_RADIUS + ENEMY_RADIUS;

const GEM_RADIUS: f32 = 0.2;
const GEM_MAGNET_RANGE: f32 = 3.0;
const GEM_MAGNET_SPEED: f32 = 12.0;
const GEM_COLLECT_DISTANCE: f32 = PLAYER_RADIUS + GEM_RADIUS;

const XP_PER_LEVEL: u32 = 100;

const ORB_RADIUS: f32 = 0.25;
const ORB_ORBIT_RADIUS: f32 = 2.0;
const ORB_ORBIT_SPEED: f32 = 3.0;
const ORB_DAMAGE: f32 = 25.0;
const ORB_HIT_DISTANCE: f32 = ORB_RADIUS + ENEMY_RADIUS;

const PULSE_COOLDOWN: f32 = 2.0;
const PULSE_RADIUS: f32 = 5.0;
const PULSE_BASE_DAMAGE: f32 = 30.0;

const REGEN_INTERVAL: f32 = 1.0;
const REGEN_AMOUNT: f32 = 2.0;

const WHIP_COOLDOWN: f32 = 1.2;
const WHIP_RANGE: f32 = 4.0;
const WHIP_ARC: f32 = 2.5;
const WHIP_DAMAGE: f32 = 20.0;

const LIGHTNING_COOLDOWN: f32 = 1.5;
const LIGHTNING_RANGE: f32 = 8.0;
const LIGHTNING_CHAIN_COUNT: u32 = 3;
const LIGHTNING_CHAIN_RANGE: f32 = 4.0;
const LIGHTNING_DAMAGE: f32 = 15.0;

const GARLIC_RADIUS: f32 = 2.5;
const GARLIC_TICK_RATE: f32 = 0.5;
const GARLIC_DAMAGE: f32 = 5.0;

const BOMB_RADIUS: f32 = 12.0;
const BOMB_DAMAGE: f32 = 100.0;
const BOMB_COOLDOWN: f32 = 8.0;

const INVINCIBILITY_DURATION: f32 = 0.5;
const INVINCIBILITY_FLASH_RATE: f32 = 10.0;

const DUST_SPAWN_INTERVAL: f32 = 0.08;
const COMBO_DECAY_TIME: f32 = 2.0;
const SPEED_BOOST_DURATION: f32 = 0.3;
const SPEED_BOOST_MULTIPLIER: f32 = 1.3;

const WAVE_ENEMIES_BASE: u32 = 20;
const BOSS_WAVE_INTERVAL: u32 = 5;
const BOSS_HEALTH: f32 = 50.0;
const BOSS_SPEED: f32 = 1.5;
const BOSS_RADIUS: f32 = 1.2;
const BOSS_XP: u32 = 100;

const SHIELD_BASE_DURATION: f32 = 8.0;
const SHIELD_DURATION_PER_LAYER: f32 = 6.0;
const SHIELD_REGEN_DELAY: f32 = 5.0;
const SHIELD_RADIUS_BASE: f32 = 1.3;
const SHIELD_RADIUS_STEP: f32 = 0.2;

const MAX_UPGRADE_LEVEL: u32 = 5;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    LevelUp,
    GameOver,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum UpgradeType {
    Damage,
    FireRate,
    ProjectileCount,
    Range,
    Speed,
    MaxHealth,
    OrbitingOrbs,
    AreaPulse,
    Magnetism,
    Regeneration,
    Whip,
    Lightning,
    Garlic,
    Bomb,
    Shield,
}

impl UpgradeType {
    fn base_name(&self) -> &'static str {
        match self {
            UpgradeType::Damage => "Damage",
            UpgradeType::FireRate => "Fire Rate",
            UpgradeType::ProjectileCount => "Projectiles",
            UpgradeType::Range => "Range",
            UpgradeType::Speed => "Speed",
            UpgradeType::MaxHealth => "Health",
            UpgradeType::OrbitingOrbs => "Orbs",
            UpgradeType::AreaPulse => "Pulse",
            UpgradeType::Magnetism => "Magnet",
            UpgradeType::Regeneration => "Regen",
            UpgradeType::Whip => "Whip",
            UpgradeType::Lightning => "Lightning",
            UpgradeType::Garlic => "Garlic",
            UpgradeType::Bomb => "Bomb",
            UpgradeType::Shield => "Shield",
        }
    }

    fn tier_name(&self, level: u32) -> String {
        let tier = level + 1;
        let tier_suffix = match tier {
            1 => "I",
            2 => "II",
            3 => "III",
            4 => "IV",
            5 => "V",
            _ => "MAX",
        };
        format!("{} {}", self.base_name(), tier_suffix)
    }

    fn description(&self, level: u32) -> String {
        match self {
            UpgradeType::Damage => format!("Damage +{}%", 25 * (level + 1)),
            UpgradeType::FireRate => format!("Fire rate +{}%", 20 * (level + 1)),
            UpgradeType::ProjectileCount => format!("+{} projectile(s)", level + 1),
            UpgradeType::Range => format!("Range +{}%", 25 * (level + 1)),
            UpgradeType::Speed => format!("Speed +{}%", 15 * (level + 1)),
            UpgradeType::MaxHealth => format!("+{} max health", 25 * (level + 1)),
            UpgradeType::OrbitingOrbs => format!("{} orbs orbit you", 2 * (level + 1)),
            UpgradeType::AreaPulse => format!("Pulse Lv{}: {}dmg", level + 1, 30 + 10 * level),
            UpgradeType::Magnetism => format!("Magnet range +{}%", 50 * (level + 1)),
            UpgradeType::Regeneration => format!("Heal {} HP/sec", 2 * (level + 1)),
            UpgradeType::Whip => format!("Whip Lv{}: {}dmg", level + 1, 20 + 10 * level),
            UpgradeType::Lightning => format!("Chain {} targets", 3 + level),
            UpgradeType::Garlic => format!("Aura Lv{}: {}dmg/tick", level + 1, 5 + 3 * level),
            UpgradeType::Bomb => {
                format!("Auto-bomb every {:.1}s", BOMB_COOLDOWN / (level + 1) as f32)
            }
            UpgradeType::Shield => format!("{} shield layer(s)", level + 1),
        }
    }

    fn max_level(&self) -> u32 {
        match self {
            UpgradeType::ProjectileCount => 4,
            UpgradeType::OrbitingOrbs => 3,
            UpgradeType::Bomb => 3,
            UpgradeType::Shield => 5,
            _ => MAX_UPGRADE_LEVEL,
        }
    }

    fn tier_color(&self, level: u32) -> Vec4 {
        match level {
            0 => Vec4::new(0.7, 0.7, 0.7, 1.0),
            1 => Vec4::new(0.4, 0.8, 0.4, 1.0),
            2 => Vec4::new(0.3, 0.5, 1.0, 1.0),
            3 => Vec4::new(0.8, 0.3, 0.9, 1.0),
            4 => Vec4::new(1.0, 0.8, 0.2, 1.0),
            _ => Vec4::new(1.0, 0.5, 0.2, 1.0),
        }
    }
}

#[derive(Clone)]
struct PlayerStats {
    damage_multiplier: f32,
    cooldown_multiplier: f32,
    projectile_count: u32,
    range_multiplier: f32,
    speed_multiplier: f32,
    max_health: f32,
    orb_count: u32,
    area_pulse_level: u32,
    magnet_multiplier: f32,
    regen_level: u32,
    whip_level: u32,
    lightning_level: u32,
    garlic_level: u32,
    shield_level: u32,
    damage_level: u32,
    fire_rate_level: u32,
    projectile_level: u32,
    range_level: u32,
    speed_level: u32,
    health_level: u32,
    magnetism_level: u32,
    bomb_level: u32,
    buff_damage_multiplier: f32,
    buff_speed_multiplier: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            cooldown_multiplier: 1.0,
            projectile_count: 1,
            range_multiplier: 1.0,
            speed_multiplier: 1.0,
            max_health: PLAYER_MAX_HEALTH,
            orb_count: 0,
            area_pulse_level: 0,
            magnet_multiplier: 1.0,
            regen_level: 0,
            whip_level: 0,
            lightning_level: 0,
            garlic_level: 0,
            shield_level: 0,
            damage_level: 0,
            fire_rate_level: 0,
            projectile_level: 0,
            range_level: 0,
            speed_level: 0,
            health_level: 0,
            magnetism_level: 0,
            bomb_level: 0,
            buff_damage_multiplier: 1.0,
            buff_speed_multiplier: 1.0,
        }
    }
}

impl PlayerStats {
    fn get_upgrade_level(&self, upgrade: UpgradeType) -> u32 {
        match upgrade {
            UpgradeType::Damage => self.damage_level,
            UpgradeType::FireRate => self.fire_rate_level,
            UpgradeType::ProjectileCount => self.projectile_level,
            UpgradeType::Range => self.range_level,
            UpgradeType::Speed => self.speed_level,
            UpgradeType::MaxHealth => self.health_level,
            UpgradeType::OrbitingOrbs => self.orb_count / 2,
            UpgradeType::AreaPulse => self.area_pulse_level,
            UpgradeType::Magnetism => self.magnetism_level,
            UpgradeType::Regeneration => self.regen_level,
            UpgradeType::Whip => self.whip_level,
            UpgradeType::Lightning => self.lightning_level,
            UpgradeType::Garlic => self.garlic_level,
            UpgradeType::Bomb => self.bomb_level,
            UpgradeType::Shield => self.shield_level,
        }
    }

    fn is_maxed(&self, upgrade: UpgradeType) -> bool {
        self.get_upgrade_level(upgrade) >= upgrade.max_level()
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
enum HighScoreType {
    #[default]
    None,
    Kills,
    Wave,
    Time,
    Combo,
}

struct Survivors {
    player_entity: Option<Entity>,
    player_position: Vec3,
    camera_entity: Option<Entity>,
    game_world: GameWorld,
    game_state: GameState,
    player_health: f32,
    damage_cooldown: f32,
    attack_cooldown: f32,
    enemy_materials: EnemyMaterials,
    projectile_material_name: Option<String>,
    gem_material_name: Option<String>,
    orb_material_name: Option<String>,
    player_xp: u32,
    player_level: u32,
    stats: PlayerStats,
    upgrade_choices: Vec<UpgradeType>,
    selected_upgrade_index: usize,
    camera_shake: f32,
    game_time: f32,
    orb_entities: Vec<Entity>,
    orb_angle: f32,
    pulse_cooldown: f32,
    regen_timer: f32,
    invincibility_timer: f32,
    whip_cooldown: f32,
    whip_angle: f32,
    lightning_cooldown: f32,
    garlic_timer: f32,
    garlic_emitter: Option<Entity>,
    bomb_cooldown: f32,
    player_facing: Vec3,
    dust_timer: f32,
    combo_count: u32,
    combo_timer: f32,
    combo_max: u32,
    speed_boost_timer: f32,
    last_wave_announced: u32,
    ambient_emitter: Option<Entity>,
    kill_flash_timer: f32,
    line_effects: Vec<LineEffect>,
    game_speed: f32,
    lob_bombs: Vec<LobBomb>,
    level_up_flash: f32,
    boss_kill_flash: f32,
    combo_emitter: Option<Entity>,
    player_shield_layers: Vec<(Entity, f32, f32, u32)>,
    player_shield_regen_timer: f32,
    enemy_shield_entities: Vec<(freecs::Entity, Entity, f32)>,
    high_score_kills: u32,
    high_score_wave: u32,
    high_score_time: f32,
    high_score_combo: u32,
    new_high_score_timer: f32,
    new_high_score_type: HighScoreType,
    score_popup_scale: f32,
    ground_entity: Option<Entity>,
    loaded_chunks: HashSet<(i32, i32)>,
    chunk_entities: HashMap<(i32, i32), Vec<Entity>>,
    max_distance_traveled: f32,
    health_crystal_spawn_timer: f32,
    health_crystal_spawn_interval: f32,
    treasure_zones: Vec<TreasureZone>,
    next_zone_distance: f32,
    active_buffs: Vec<ActiveBuff>,
    ui: SurvivorsUi,
}

struct LineEffect {
    entity: Entity,
    timer: f32,
    max_time: f32,
    center: Vec3,
    start_radius: f32,
    end_radius: f32,
    segments: u32,
    color_start: Vec4,
    color_end: Vec4,
}

struct LobBomb {
    entity: Entity,
    start_position: Vec3,
    target_position: Vec3,
    flight_time: f32,
    elapsed: f32,
    arc_height: f32,
    trail_emitter: Option<Entity>,
    fuse_emitter: Option<Entity>,
}

#[derive(Default, Clone)]
struct EnemyMaterials {
    normal: Option<String>,
    fast: Option<String>,
    tank: Option<String>,
    exploder: Option<String>,
    boss: Option<String>,
}

struct SurvivorsUi {
    main_menu_screen: Entity,
    start_button: Entity,
    menu_high_scores_container: Entity,
    menu_high_scores_slot: usize,
    menu_high_scores_time_slot: usize,

    paused_screen: Entity,
    resume_button: Entity,

    hud_screen: Entity,
    health_bar: Entity,
    health_bar_fill: Entity,
    xp_bar: Entity,
    level_label_slot: usize,
    wave_bar: Entity,
    wave_label_slot: usize,
    kills_time_slot: usize,
    kills_time_entity: Entity,
    combo_entity: Entity,
    combo_slot: usize,
    combo_best_entity: Entity,
    combo_best_slot: usize,
    bomb_entity: Entity,
    bomb_slot: usize,
    boss_entity: Entity,
    speed_entity: Entity,
    speed_slot: usize,
    buffs_container: Entity,
    buff_slots: Vec<(Entity, usize)>,

    levelup_screen: Entity,
    levelup_title_slot: usize,
    upgrade_buttons: [Entity; 3],
    upgrade_desc_entity: Entity,
    upgrade_desc_slot: usize,

    gameover_screen: Entity,
    high_score_banner_entity: Entity,
    stats_level_slot: usize,
    stats_wave_entity: Entity,
    stats_wave_slot: usize,
    stats_kills_entity: Entity,
    stats_kills_slot: usize,
    stats_time_entity: Entity,
    stats_time_slot: usize,
    stats_combo_entity: Entity,
    stats_combo_slot: usize,
    best_scores_slot: usize,
    best_scores_time_slot: usize,
}

impl Default for SurvivorsUi {
    fn default() -> Self {
        let placeholder = Entity {
            id: 0,
            generation: 0,
        };
        Self {
            main_menu_screen: placeholder,
            start_button: placeholder,
            menu_high_scores_container: placeholder,
            menu_high_scores_slot: 0,
            menu_high_scores_time_slot: 0,
            paused_screen: placeholder,
            resume_button: placeholder,
            hud_screen: placeholder,
            health_bar: placeholder,
            health_bar_fill: placeholder,
            xp_bar: placeholder,
            level_label_slot: 0,
            wave_bar: placeholder,
            wave_label_slot: 0,
            kills_time_slot: 0,
            kills_time_entity: placeholder,
            combo_entity: placeholder,
            combo_slot: 0,
            combo_best_entity: placeholder,
            combo_best_slot: 0,
            bomb_entity: placeholder,
            bomb_slot: 0,
            boss_entity: placeholder,
            speed_entity: placeholder,
            speed_slot: 0,
            buffs_container: placeholder,
            buff_slots: Vec::new(),
            levelup_screen: placeholder,
            levelup_title_slot: 0,
            upgrade_buttons: [placeholder; 3],
            upgrade_desc_entity: placeholder,
            upgrade_desc_slot: 0,
            gameover_screen: placeholder,
            high_score_banner_entity: placeholder,
            stats_level_slot: 0,
            stats_wave_entity: placeholder,
            stats_wave_slot: 0,
            stats_kills_entity: placeholder,
            stats_kills_slot: 0,
            stats_time_entity: placeholder,
            stats_time_slot: 0,
            stats_combo_entity: placeholder,
            stats_combo_slot: 0,
            best_scores_slot: 0,
            best_scores_time_slot: 0,
        }
    }
}

impl Default for Survivors {
    fn default() -> Self {
        Self {
            player_entity: None,
            player_position: Vec3::zeros(),
            camera_entity: None,
            game_world: GameWorld::default(),
            game_state: GameState::MainMenu,
            player_health: PLAYER_MAX_HEALTH,
            damage_cooldown: 0.0,
            attack_cooldown: 0.0,
            enemy_materials: EnemyMaterials::default(),
            projectile_material_name: None,
            gem_material_name: None,
            orb_material_name: None,
            player_xp: 0,
            player_level: 1,
            stats: PlayerStats::default(),
            upgrade_choices: Vec::new(),
            selected_upgrade_index: 0,
            camera_shake: 0.0,
            game_time: 0.0,
            orb_entities: Vec::new(),
            orb_angle: 0.0,
            pulse_cooldown: 0.0,
            regen_timer: 0.0,
            invincibility_timer: 0.0,
            whip_cooldown: 0.0,
            whip_angle: 0.0,
            lightning_cooldown: 0.0,
            garlic_timer: 0.0,
            garlic_emitter: None,
            bomb_cooldown: 0.0,
            player_facing: Vec3::new(1.0, 0.0, 0.0),
            dust_timer: 0.0,
            combo_count: 0,
            combo_timer: 0.0,
            combo_max: 0,
            speed_boost_timer: 0.0,
            last_wave_announced: 0,
            ambient_emitter: None,
            kill_flash_timer: 0.0,
            line_effects: Vec::new(),
            game_speed: 1.0,
            lob_bombs: Vec::new(),
            level_up_flash: 0.0,
            boss_kill_flash: 0.0,
            combo_emitter: None,
            player_shield_layers: Vec::new(),
            player_shield_regen_timer: 0.0,
            enemy_shield_entities: Vec::new(),
            high_score_kills: 0,
            high_score_wave: 0,
            high_score_time: 0.0,
            high_score_combo: 0,
            new_high_score_timer: 0.0,
            new_high_score_type: HighScoreType::None,
            score_popup_scale: 1.0,
            ground_entity: None,
            loaded_chunks: HashSet::new(),
            chunk_entities: HashMap::new(),
            max_distance_traveled: 0.0,
            health_crystal_spawn_timer: 0.0,
            health_crystal_spawn_interval: 60.0,
            treasure_zones: Vec::new(),
            next_zone_distance: 50.0,
            active_buffs: Vec::new(),
            ui: SurvivorsUi::default(),
        }
    }
}

impl State for Survivors {
    fn title(&self) -> &str {
        "Survivors"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = false;
        world.resources.user_interface.enabled = false;
        world.resources.retained_ui.enabled = true;

        self.spawn_arena(world);
        self.spawn_player(world);
        self.spawn_camera(world);
        self.spawn_lighting(world);
        self.create_materials(world);
        self.build_ui(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.window.timing.delta_time;
        let game_delta = delta * self.game_speed;

        self.update_popups(world, game_delta);
        self.update_invincibility(world, game_delta);
        self.update_combo(game_delta);
        self.update_speed_boost(game_delta);
        self.update_kill_flash(game_delta);
        self.update_flashes(delta);
        self.update_combo_fire(world);

        match self.game_state {
            GameState::MainMenu => {
                self.camera_follow_system(world, delta);
                self.update_ambient_particles(world);
            }
            GameState::Playing => {
                self.game_time += game_delta;
                self.player_movement_system(world, game_delta);
                self.camera_follow_system(world, game_delta);
                self.update_ground_position(world);
                self.update_chunks(world);
                self.enemy_spawn_system(world, game_delta);
                self.enemy_chase_system(world, game_delta);
                self.attack_system(world, game_delta);
                self.projectile_movement_system(world, game_delta);
                self.projectile_collision_system(world);
                self.gem_system(world, game_delta);
                self.health_crystal_spawn_system(world, game_delta);
                self.health_crystal_system(world);
                self.health_gem_system(world, game_delta);
                self.player_collision_system(world, game_delta);
                self.update_player_shield_system(world, game_delta);
                self.update_enemy_shields(world);
                self.orb_system(world, game_delta);
                self.pulse_system(world, game_delta);
                self.regen_system(world, game_delta);
                self.whip_system(world, game_delta);
                self.lightning_system(world, game_delta);
                self.garlic_system(world, game_delta);
                self.bomb_system(world, game_delta);
                self.check_wave_announcement(world);
                self.update_ambient_particles(world);
                self.update_line_effects(world, game_delta);
                self.update_lob_bombs(world, game_delta);
                self.treasure_zone_system(world, game_delta);
                self.update_active_buffs(game_delta);

                if self.player_health <= 0.0 {
                    self.check_high_scores();
                    self.game_state = GameState::GameOver;
                }

                update_particle_emitters(world, game_delta);
                nightshade::ecs::text::systems::sync_text_meshes_system(world);
            }
            GameState::Paused => {
                self.camera_follow_system(world, delta);
            }
            GameState::LevelUp => {}
            GameState::GameOver => {}
        }

        self.draw_ui(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            KeyCode::KeyR => {
                if self.game_state == GameState::GameOver {
                    self.restart_game(world);
                }
            }
            KeyCode::Escape => match self.game_state {
                GameState::Playing => {
                    self.game_state = GameState::Paused;
                }
                GameState::Paused => {
                    self.game_state = GameState::Playing;
                }
                _ => {}
            },
            KeyCode::Enter | KeyCode::Space => match self.game_state {
                GameState::MainMenu => {
                    self.start_game(world);
                }
                GameState::LevelUp => {
                    if let Some(upgrade) = self.upgrade_choices.get(self.selected_upgrade_index) {
                        self.apply_upgrade(*upgrade, world);
                        self.game_state = GameState::Playing;
                    }
                }
                _ => {}
            },
            KeyCode::ArrowLeft | KeyCode::KeyA => {
                if self.game_state == GameState::LevelUp && self.selected_upgrade_index > 0 {
                    self.selected_upgrade_index -= 1;
                }
            }
            KeyCode::ArrowRight | KeyCode::KeyD => {
                if self.game_state == GameState::LevelUp
                    && self.selected_upgrade_index < self.upgrade_choices.len().saturating_sub(1)
                {
                    self.selected_upgrade_index += 1;
                }
            }
            KeyCode::BracketRight | KeyCode::Equal => {
                if self.game_state == GameState::Playing {
                    self.game_speed = (self.game_speed * 2.0).min(8.0);
                }
            }
            KeyCode::BracketLeft | KeyCode::Minus => {
                if self.game_state == GameState::Playing {
                    self.game_speed = (self.game_speed / 2.0).max(0.25);
                }
            }
            _ => {}
        }
    }

    fn on_gamepad_event(&mut self, world: &mut World, event: gilrs::Event) {
        if let gilrs::EventType::ButtonPressed(button, _) = event.event {
            match button {
                gilrs::Button::Start => match self.game_state {
                    GameState::MainMenu => {
                        self.start_game(world);
                    }
                    GameState::Playing => {
                        self.game_state = GameState::Paused;
                    }
                    GameState::Paused => {
                        self.game_state = GameState::Playing;
                    }
                    GameState::GameOver => {
                        self.restart_game(world);
                    }
                    _ => {}
                },
                gilrs::Button::South => match self.game_state {
                    GameState::MainMenu => {
                        self.start_game(world);
                    }
                    GameState::Paused => {
                        self.game_state = GameState::Playing;
                    }
                    GameState::GameOver => {
                        self.restart_game(world);
                    }
                    GameState::LevelUp => {
                        if let Some(upgrade) = self.upgrade_choices.get(self.selected_upgrade_index)
                        {
                            self.apply_upgrade(*upgrade, world);
                            self.game_state = GameState::Playing;
                        }
                    }
                    _ => {}
                },
                gilrs::Button::DPadLeft | gilrs::Button::DPadUp => {
                    if self.game_state == GameState::LevelUp && self.selected_upgrade_index > 0 {
                        self.selected_upgrade_index -= 1;
                    }
                }
                gilrs::Button::DPadRight | gilrs::Button::DPadDown => {
                    if self.game_state == GameState::LevelUp
                        && self.selected_upgrade_index
                            < self.upgrade_choices.len().saturating_sub(1)
                    {
                        self.selected_upgrade_index += 1;
                    }
                }
                gilrs::Button::RightTrigger => {
                    self.game_speed = (self.game_speed + 0.25).min(3.0);
                }
                gilrs::Button::LeftTrigger => {
                    self.game_speed = (self.game_speed - 0.25).max(0.25);
                }
                gilrs::Button::RightTrigger2 => {
                    self.game_speed = (self.game_speed + 0.5).min(3.0);
                }
                gilrs::Button::LeftTrigger2 => {
                    self.game_speed = (self.game_speed - 0.5).max(0.25);
                }
                _ => {}
            }
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

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.02);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.compute_output);

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        graph
            .pass(Box::new(fxaa_pass))
            .read("input", resources.compute_output)
            .write("output", fxaa_output);

        let swapchain_blit_pass =
            passes::BlitPass::new(device, surface_format).with_name("default_swapchain_blit");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", fxaa_output)
            .write("output", resources.swapchain);
    }
}

impl Survivors {
    fn build_ui(&mut self, world: &mut World) {
        let font_size = 14.0;
        let small_font = 12.0;
        let title_font = 24.0;
        let bar_width = 150.0;
        let bar_height = 12.0;
        let dim_text = Vec4::new(0.5, 0.5, 0.5, 1.0);
        let white = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let cyan = Vec4::new(0.39, 0.78, 1.0, 1.0);
        let gold = Vec4::new(1.0, 0.84, 0.0, 1.0);

        let tc = &mut world.resources.text_cache;
        let title_slot = tc.add_text("SURVIVORS");
        let subtitle_slot = tc.add_text("A Vampire Survivors-style Arena Game");
        let hs_header_slot = tc.add_text("--- HIGH SCORES ---");
        let menu_high_scores_slot = tc.add_text("");
        let menu_high_scores_time_slot = tc.add_text("");
        let begin_hint_slot = tc.add_text("Press Enter, Space, A, or Start to begin");
        let paused_title_slot = tc.add_text("PAUSED");
        let paused_hint_slot = tc.add_text("Press ESC, Start, or A to resume");
        let health_label_slot = tc.add_text("Health:");
        let level_label_slot = tc.add_text("Lv.1:");
        let wave_label_slot = tc.add_text("Wave 1:");
        let kills_time_slot = tc.add_text("Kills: 0 | Time: 0s");
        let combo_slot = tc.add_text("");
        let combo_best_slot = tc.add_text("");
        let bomb_slot = tc.add_text("");
        let boss_slot = tc.add_text("BOSS!");
        let speed_slot = tc.add_text("");
        let levelup_title_slot = tc.add_text("LEVEL UP! (Lv.1)");
        let choose_upgrade_slot = tc.add_text("Choose an upgrade:");
        let upgrade_desc_slot = tc.add_text("");
        let levelup_hint_slot = tc.add_text("Left/Right: Select | A/Enter: Confirm");
        let gameover_title_slot = tc.add_text("GAME OVER");
        let new_hs_banner_slot = tc.add_text("NEW HIGH SCORE!");
        let stats_level_slot = tc.add_text("Level: 1");
        let stats_wave_slot = tc.add_text("Wave: 1");
        let stats_kills_slot = tc.add_text("Kills: 0");
        let stats_time_slot = tc.add_text("Time: 0s");
        let stats_combo_slot = tc.add_text("");
        let go_hs_header_slot = tc.add_text("--- HIGH SCORES ---");
        let best_scores_slot = tc.add_text("");
        let best_scores_time_slot = tc.add_text("");
        let restart_hint_slot = tc.add_text("Press R, Start, or A to restart");
        let buff_text_slots: Vec<usize> = (0..5).map(|_| tc.add_text("")).collect();

        let mut tree = UiTreeBuilder::new(world);

        let placeholder = Entity {
            id: 0,
            generation: 0,
        };

        let mut start_button = placeholder;
        let mut menu_hs_container = placeholder;

        self.ui.main_menu_screen = tree
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.7))
            .with_layer(UiLayer::FloatingPanels)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(Vec2::new(50.0, 50.0)),
                        Ab(Vec2::new(400.0, 500.0)),
                        Anchor::Center,
                    )
                    .flow(FlowDirection::Vertical, 0.0, 4.0)
                    .with_children(|tree| {
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 1.5)),
                            )
                            .with_text_slot(title_slot, title_font)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(cyan)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(6.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(subtitle_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(dim_text)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(30.0);

                        start_button =
                            tree.add_button_colored("START GAME", Vec4::new(0.24, 0.47, 0.31, 1.0));

                        tree.add_spacing(20.0);

                        tree.add_label("Controls:");
                        tree.add_label_colored("WASD / Arrow Keys / Left Stick - Move", dim_text);
                        tree.add_label_colored("Space / X Button - Use Bomb", dim_text);
                        tree.add_label_colored("ESC / Start - Pause", dim_text);
                        tree.add_label_colored("]/= or RB/LB - Speed Up/Down", dim_text);

                        menu_hs_container = tree
                            .add_node()
                            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
                            .auto_size(AutoSizeMode::Height)
                            .flow(FlowDirection::Vertical, 0.0, 4.0)
                            .with_visible(false)
                            .with_children(|tree| {
                                tree.add_spacing(16.0);

                                tree.add_node()
                                    .flow_child(
                                        Rl(Vec2::new(100.0, 0.0))
                                            + Ab(Vec2::new(0.0, font_size * 1.5)),
                                    )
                                    .with_text_slot(hs_header_slot, font_size)
                                    .with_text_alignment(
                                        TextAlignment::Center,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(gold)
                                    .without_pointer_events()
                                    .done();

                                tree.add_node()
                                    .flow_child(
                                        Rl(Vec2::new(100.0, 0.0))
                                            + Ab(Vec2::new(0.0, font_size * 1.5)),
                                    )
                                    .with_text_slot(menu_high_scores_slot, font_size)
                                    .with_text_alignment(
                                        TextAlignment::Center,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(Vec4::new(0.8, 0.8, 0.8, 1.0))
                                    .without_pointer_events()
                                    .done();

                                tree.add_node()
                                    .flow_child(
                                        Rl(Vec2::new(100.0, 0.0))
                                            + Ab(Vec2::new(0.0, font_size * 1.5)),
                                    )
                                    .with_text_slot(menu_high_scores_time_slot, font_size)
                                    .with_text_alignment(
                                        TextAlignment::Center,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(Vec4::new(0.8, 0.8, 0.8, 1.0))
                                    .without_pointer_events()
                                    .done();
                            })
                            .done();

                        tree.add_spacing(10.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(begin_hint_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(0.59, 0.59, 0.59, 1.0))
                            .without_pointer_events()
                            .done();
                    })
                    .done();
            })
            .done();

        self.ui.start_button = start_button;
        self.ui.menu_high_scores_container = menu_hs_container;
        self.ui.menu_high_scores_slot = menu_high_scores_slot;
        self.ui.menu_high_scores_time_slot = menu_high_scores_time_slot;

        // --- Paused Screen ---
        let mut resume_button = placeholder;

        self.ui.paused_screen = tree
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.78))
            .with_layer(UiLayer::FloatingPanels)
            .with_visible(false)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(Vec2::new(50.0, 50.0)),
                        Ab(Vec2::new(300.0, 200.0)),
                        Anchor::Center,
                    )
                    .flow(FlowDirection::Vertical, 0.0, 4.0)
                    .with_children(|tree| {
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 1.5)),
                            )
                            .with_text_slot(paused_title_slot, title_font)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(30.0);

                        resume_button =
                            tree.add_button_colored("Resume", Vec4::new(0.24, 0.39, 0.24, 1.0));

                        tree.add_spacing(16.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(paused_hint_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(dim_text)
                            .without_pointer_events()
                            .done();
                    })
                    .done();
            })
            .done();

        self.ui.resume_button = resume_button;

        // --- Playing HUD ---
        let mut health_bar = placeholder;
        let mut xp_bar = placeholder;
        let mut wave_bar = placeholder;
        let mut kills_time_entity = placeholder;
        let mut combo_entity = placeholder;
        let mut combo_best_entity = placeholder;
        let mut bomb_entity = placeholder;
        let mut boss_entity = placeholder;
        let mut speed_entity = placeholder;
        let mut buffs_container = placeholder;

        self.ui.hud_screen = tree
            .add_node()
            .window(
                Ab(Vec2::new(10.0, 10.0)),
                Ab(Vec2::new(280.0, 300.0)),
                Anchor::TopLeft,
            )
            .with_rect(4.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.6))
            .with_visible(false)
            .without_pointer_events()
            .auto_size(AutoSizeMode::Height)
            .with_children(|tree| {
                tree.add_node()
                    .boundary(
                        Ab(Vec2::new(10.0, 10.0)),
                        Rl(Vec2::new(100.0, 100.0)) + Ab(Vec2::new(-10.0, -10.0)),
                    )
                    .flow(FlowDirection::Vertical, 0.0, 2.0)
                    .auto_size(AutoSizeMode::Height)
                    .with_children(|tree| {
                        // Health bar row
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, bar_height + 6.0)),
                            )
                            .flow(FlowDirection::Horizontal, 0.0, 6.0)
                            .with_children(|tree| {
                                tree.add_node()
                                    .flow_child(Ab(Vec2::new(55.0, bar_height + 6.0)))
                                    .with_text_slot(health_label_slot, small_font)
                                    .with_text_alignment(
                                        TextAlignment::Left,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(white)
                                    .without_pointer_events()
                                    .done();

                                health_bar = tree.add_progress_bar(1.0);
                                if let Some(node) =
                                    tree.world_mut().ui.get_ui_layout_node_mut(health_bar)
                                {
                                    node.flow_child_size =
                                        Some(Ab(Vec2::new(bar_width, bar_height)).into());
                                }
                            })
                            .done();

                        // XP bar row
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, bar_height + 6.0)),
                            )
                            .flow(FlowDirection::Horizontal, 0.0, 6.0)
                            .with_children(|tree| {
                                tree.add_node()
                                    .flow_child(Ab(Vec2::new(55.0, bar_height + 6.0)))
                                    .with_text_slot(level_label_slot, small_font)
                                    .with_text_alignment(
                                        TextAlignment::Left,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(white)
                                    .without_pointer_events()
                                    .done();

                                xp_bar = tree.add_progress_bar(0.0);
                                if let Some(node) = tree.world_mut().ui.get_ui_layout_node_mut(xp_bar)
                                {
                                    node.flow_child_size =
                                        Some(Ab(Vec2::new(bar_width, bar_height)).into());
                                }
                            })
                            .done();

                        // Wave bar row
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, bar_height + 6.0)),
                            )
                            .flow(FlowDirection::Horizontal, 0.0, 6.0)
                            .with_children(|tree| {
                                tree.add_node()
                                    .flow_child(Ab(Vec2::new(55.0, bar_height + 6.0)))
                                    .with_text_slot(wave_label_slot, small_font)
                                    .with_text_alignment(
                                        TextAlignment::Left,
                                        VerticalAlignment::Middle,
                                    )
                                    .with_color::<UiBase>(white)
                                    .without_pointer_events()
                                    .done();

                                wave_bar = tree.add_progress_bar(0.0);
                                if let Some(node) =
                                    tree.world_mut().ui.get_ui_layout_node_mut(wave_bar)
                                {
                                    node.flow_child_size =
                                        Some(Ab(Vec2::new(bar_width, bar_height)).into());
                                }
                            })
                            .done();

                        // Kills/time
                        kills_time_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(kills_time_slot, small_font)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        // Combo
                        combo_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(combo_slot, small_font)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(1.0, 1.0, 0.39, 1.0))
                            .with_visible(false)
                            .without_pointer_events()
                            .done();

                        // Combo best
                        combo_best_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(combo_best_slot, small_font)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(gold)
                            .with_visible(false)
                            .without_pointer_events()
                            .done();

                        // Bomb status
                        bomb_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(bomb_slot, small_font)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(0.39, 1.0, 0.39, 1.0))
                            .with_visible(false)
                            .without_pointer_events()
                            .done();

                        // Boss indicator
                        boss_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(boss_slot, small_font)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(1.0, 0.0, 0.0, 1.0))
                            .with_visible(false)
                            .without_pointer_events()
                            .done();

                        // Speed indicator
                        speed_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(speed_slot, small_font)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(cyan)
                            .with_visible(false)
                            .without_pointer_events()
                            .done();

                        // Buffs container
                        buffs_container = tree
                            .add_node()
                            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
                            .auto_size(AutoSizeMode::Height)
                            .flow(FlowDirection::Vertical, 0.0, 2.0)
                            .without_pointer_events()
                            .done();
                    })
                    .done();
            })
            .done();

        self.ui.health_bar = health_bar;
        self.ui.xp_bar = xp_bar;
        self.ui.level_label_slot = level_label_slot;
        self.ui.wave_bar = wave_bar;
        self.ui.wave_label_slot = wave_label_slot;
        self.ui.kills_time_slot = kills_time_slot;
        self.ui.kills_time_entity = kills_time_entity;
        self.ui.combo_entity = combo_entity;
        self.ui.combo_slot = combo_slot;
        self.ui.combo_best_entity = combo_best_entity;
        self.ui.combo_best_slot = combo_best_slot;
        self.ui.bomb_entity = bomb_entity;
        self.ui.bomb_slot = bomb_slot;
        self.ui.boss_entity = boss_entity;
        self.ui.speed_entity = speed_entity;
        self.ui.speed_slot = speed_slot;
        self.ui.buffs_container = buffs_container;

        if let Some(UiWidgetState::ProgressBar(data)) =
            tree.world_mut().ui.get_ui_widget_state(health_bar)
        {
            self.ui.health_bar_fill = data.fill_entity;
        }

        // Pre-allocate buff label slots
        for _ in 0..5 {
            let slot = buff_text_slots[self.ui.buff_slots.len()];
            let entity = {
                let parent = buffs_container;
                tree.push_parent(parent);
                let entity = tree
                    .add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)))
                    .with_text_slot(slot, small_font)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_color::<UiBase>(white)
                    .with_visible(false)
                    .without_pointer_events()
                    .done();
                tree.pop_parent();
                entity
            };
            self.ui.buff_slots.push((entity, slot));
        }

        // --- Level Up Screen ---
        let mut upgrade_buttons = [placeholder; 3];
        let mut upgrade_desc_entity = placeholder;

        self.ui.levelup_screen = tree
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.78))
            .with_layer(UiLayer::FloatingPanels)
            .with_visible(false)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(Vec2::new(50.0, 50.0)),
                        Ab(Vec2::new(500.0, 300.0)),
                        Anchor::Center,
                    )
                    .flow(FlowDirection::Vertical, 0.0, 4.0)
                    .with_children(|tree| {
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 1.5)),
                            )
                            .with_text_slot(levelup_title_slot, title_font)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(gold)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(12.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(choose_upgrade_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(12.0);

                        // Upgrade buttons row
                        tree.add_node()
                            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 40.0)))
                            .flow(FlowDirection::Horizontal, 0.0, 8.0)
                            .with_children(|tree| {
                                for button in &mut upgrade_buttons {
                                    *button = tree
                                        .add_button_colored("---", Vec4::new(0.3, 0.3, 0.3, 1.0));
                                    if let Some(node) =
                                        tree.world_mut().ui.get_ui_layout_node_mut(*button)
                                    {
                                        node.flex_grow = Some(1.0);
                                    }
                                }
                            })
                            .done();

                        tree.add_spacing(10.0);

                        upgrade_desc_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(upgrade_desc_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(0.83, 0.83, 0.83, 1.0))
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(10.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(levelup_hint_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(dim_text)
                            .without_pointer_events()
                            .done();
                    })
                    .done();
            })
            .done();

        self.ui.levelup_title_slot = levelup_title_slot;
        self.ui.upgrade_buttons = upgrade_buttons;
        self.ui.upgrade_desc_entity = upgrade_desc_entity;
        self.ui.upgrade_desc_slot = upgrade_desc_slot;

        // --- Game Over Screen ---
        let mut high_score_banner_entity = placeholder;
        let mut stats_wave_entity = placeholder;
        let mut stats_kills_entity = placeholder;
        let mut stats_time_entity = placeholder;
        let mut stats_combo_entity = placeholder;

        self.ui.gameover_screen = tree
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.78))
            .with_layer(UiLayer::FloatingPanels)
            .with_visible(false)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(Vec2::new(50.0, 50.0)),
                        Ab(Vec2::new(400.0, 450.0)),
                        Anchor::Center,
                    )
                    .flow(FlowDirection::Vertical, 0.0, 4.0)
                    .with_children(|tree| {
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 1.5)),
                            )
                            .with_text_slot(gameover_title_slot, title_font)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(1.0, 0.0, 0.0, 1.0))
                            .without_pointer_events()
                            .done();

                        high_score_banner_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(new_hs_banner_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(gold)
                            .with_visible(false)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(12.0);

                        // Stats labels
                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(stats_level_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        stats_wave_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(stats_wave_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        stats_kills_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(stats_kills_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        stats_time_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(stats_time_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        stats_combo_entity = tree
                            .add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(stats_combo_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .with_visible(false)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(12.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(go_hs_header_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(Vec4::new(0.7, 0.7, 0.7, 1.0))
                            .without_pointer_events()
                            .done();

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(best_scores_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(best_scores_time_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(white)
                            .without_pointer_events()
                            .done();

                        tree.add_spacing(12.0);

                        tree.add_node()
                            .flow_child(
                                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)),
                            )
                            .with_text_slot(restart_hint_slot, font_size)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_color::<UiBase>(dim_text)
                            .without_pointer_events()
                            .done();
                    })
                    .done();
            })
            .done();

        self.ui.high_score_banner_entity = high_score_banner_entity;
        self.ui.stats_level_slot = stats_level_slot;
        self.ui.stats_wave_entity = stats_wave_entity;
        self.ui.stats_wave_slot = stats_wave_slot;
        self.ui.stats_kills_entity = stats_kills_entity;
        self.ui.stats_kills_slot = stats_kills_slot;
        self.ui.stats_time_entity = stats_time_entity;
        self.ui.stats_time_slot = stats_time_slot;
        self.ui.stats_combo_entity = stats_combo_entity;
        self.ui.stats_combo_slot = stats_combo_slot;
        self.ui.best_scores_slot = best_scores_slot;
        self.ui.best_scores_time_slot = best_scores_time_slot;

        tree.finish();
    }

    fn draw_ui(&mut self, world: &mut World) {
        let show_main_menu = self.game_state == GameState::MainMenu;
        let show_paused = self.game_state == GameState::Paused;
        let show_hud =
            self.game_state == GameState::Playing || self.game_state == GameState::LevelUp;
        let show_levelup = self.game_state == GameState::LevelUp;
        let show_gameover = self.game_state == GameState::GameOver;

        world.ui_set_visible(self.ui.main_menu_screen, show_main_menu);
        world.ui_set_visible(self.ui.paused_screen, show_paused);
        world.ui_set_visible(self.ui.hud_screen, show_hud);
        world.ui_set_visible(self.ui.levelup_screen, show_levelup);
        world.ui_set_visible(self.ui.gameover_screen, show_gameover);

        if show_main_menu {
            if world
                .widget::<UiButtonData>(self.ui.start_button)
                .is_some_and(|d| d.clicked)
            {
                self.start_game(world);
                return;
            }

            let has_scores = self.high_score_kills > 0 || self.high_score_wave > 0;
            world.ui_set_visible(self.ui.menu_high_scores_container, has_scores);
            if has_scores {
                world.resources.text_cache.set_text(
                    self.ui.menu_high_scores_slot,
                    format!(
                        "Wave: {} | Kills: {}",
                        self.high_score_wave, self.high_score_kills
                    ),
                );
                world.resources.text_cache.set_text(
                    self.ui.menu_high_scores_time_slot,
                    format!(
                        "Time: {:.0}s | Combo: {}x",
                        self.high_score_time, self.high_score_combo
                    ),
                );
            }
        }

        if show_paused
            && world
                .widget::<UiButtonData>(self.ui.resume_button)
                .is_some_and(|d| d.clicked)
        {
            self.game_state = GameState::Playing;
            return;
        }

        if show_hud {
            let health_pct = self.player_health / self.stats.max_health;
            world.ui_progress_bar_set_value(self.ui.health_bar, health_pct);

            let health_color = if health_pct > 0.5 {
                Vec4::new(0.0, 0.78, 0.0, 1.0)
            } else if health_pct > 0.25 {
                Vec4::new(0.78, 0.78, 0.0, 1.0)
            } else {
                Vec4::new(0.78, 0.0, 0.0, 1.0)
            };
            if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.health_bar_fill) {
                color.colors[UiBase::INDEX] = Some(health_color);
            }

            let xp_for_next = XP_PER_LEVEL * self.player_level;
            let xp_pct = self.player_xp as f32 / xp_for_next as f32;
            world.ui_progress_bar_set_value(self.ui.xp_bar, xp_pct);

            world.resources.text_cache.set_text(
                self.ui.level_label_slot,
                format!("Lv.{}:", self.player_level),
            );

            let wave = self.game_world.resources.current_wave;
            let total_wave_enemies = WAVE_ENEMIES_BASE + wave * 5;
            let remaining = self.game_world.resources.wave_enemies_remaining
                + self.game_world.resources.enemy_list.len() as u32;
            let killed = total_wave_enemies.saturating_sub(remaining);
            let wave_pct = if total_wave_enemies > 0 {
                killed as f32 / total_wave_enemies as f32
            } else {
                0.0
            };
            world.ui_progress_bar_set_value(self.ui.wave_bar, wave_pct);
            world
                .resources
                .text_cache
                .set_text(self.ui.wave_label_slot, format!("Wave {}:", wave));

            let current_kills = self.game_world.resources.enemies_killed;
            let is_kills_record =
                current_kills > self.high_score_kills && self.high_score_kills > 0;
            let is_time_record =
                self.game_time > self.high_score_time && self.high_score_time > 0.0;

            if is_kills_record || is_time_record {
                let pulse = (self.game_time * 6.0).sin() * 0.3 + 0.7;
                let record_color = Vec4::new(1.0, 0.84 * pulse + 0.16, 0.0, 1.0);
                world.resources.text_cache.set_text(
                    self.ui.kills_time_slot,
                    format!("Kills: {} | Time: {:.0}s", current_kills, self.game_time),
                );
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.kills_time_entity) {
                    color.colors[UiBase::INDEX] = Some(record_color);
                }
            } else {
                world.resources.text_cache.set_text(
                    self.ui.kills_time_slot,
                    format!("Kills: {} | Time: {:.0}s", current_kills, self.game_time),
                );
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.kills_time_entity) {
                    color.colors[UiBase::INDEX] = Some(Vec4::new(1.0, 1.0, 1.0, 1.0));
                }
            }

            let show_combo = self.combo_count > 1;
            world.ui_set_visible(self.ui.combo_entity, show_combo);
            if show_combo {
                let combo_color = if self.combo_count >= 50 {
                    Vec4::new(1.0, 0.39, 1.0, 1.0)
                } else if self.combo_count >= 25 {
                    Vec4::new(1.0, 0.78, 0.2, 1.0)
                } else if self.combo_count >= 10 {
                    Vec4::new(1.0, 0.59, 0.2, 1.0)
                } else {
                    Vec4::new(1.0, 1.0, 0.39, 1.0)
                };
                world
                    .resources
                    .text_cache
                    .set_text(self.ui.combo_slot, format!("{}x COMBO!", self.combo_count));
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.combo_entity) {
                    color.colors[UiBase::INDEX] = Some(combo_color);
                }
            }

            let show_combo_best = show_combo && self.combo_count > self.combo_max;
            world.ui_set_visible(self.ui.combo_best_entity, show_combo_best);
            if show_combo_best {
                let best_pulse = (self.game_time * 10.0).sin() * 0.5 + 0.5;
                let best_color = Vec4::new(1.0, 0.84 + best_pulse * 0.16, best_pulse * 0.3, 1.0);
                world
                    .resources
                    .text_cache
                    .set_text(self.ui.combo_best_slot, "NEW BEST COMBO!");
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.combo_best_entity) {
                    color.colors[UiBase::INDEX] = Some(best_color);
                }
            }

            let show_bomb = self.stats.bomb_level > 0;
            world.ui_set_visible(self.ui.bomb_entity, show_bomb);
            if show_bomb {
                let ready = self.bomb_cooldown <= 0.0;
                let bomb_text = if ready {
                    "Bomb: READY".to_string()
                } else {
                    format!("Bomb: {:.1}s", self.bomb_cooldown)
                };
                let cooldown_percent =
                    self.bomb_cooldown / (BOMB_COOLDOWN / self.stats.bomb_level as f32);
                let bomb_color = if ready {
                    Vec4::new(0.39, 1.0, 0.39, 1.0)
                } else {
                    let r = 0.39 + 0.61 * cooldown_percent;
                    Vec4::new(r, 0.39, 0.39, 1.0)
                };
                world
                    .resources
                    .text_cache
                    .set_text(self.ui.bomb_slot, &bomb_text);
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.bomb_entity) {
                    color.colors[UiBase::INDEX] = Some(bomb_color);
                }
            }

            world.ui_set_visible(self.ui.boss_entity, self.game_world.resources.boss_alive);

            let show_speed = (self.game_speed - 1.0).abs() > 0.01;
            world.ui_set_visible(self.ui.speed_entity, show_speed);
            if show_speed {
                let speed_text = if self.game_speed >= 1.0 {
                    format!("{}x Speed", self.game_speed as i32)
                } else {
                    format!("{:.2}x Speed", self.game_speed)
                };
                let speed_color = if self.game_speed > 1.0 {
                    Vec4::new(0.39, 0.78, 1.0, 1.0)
                } else {
                    Vec4::new(1.0, 0.78, 0.39, 1.0)
                };
                world
                    .resources
                    .text_cache
                    .set_text(self.ui.speed_slot, &speed_text);
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.speed_entity) {
                    color.colors[UiBase::INDEX] = Some(speed_color);
                }
            }

            for (buff_index, &(entity, slot)) in self.ui.buff_slots.iter().enumerate() {
                if buff_index < self.active_buffs.len() {
                    let buff = &self.active_buffs[buff_index];
                    let (buff_name, buff_color) = match buff.buff_type {
                        BuffType::Berserk => ("BERSERK", Vec4::new(0.8, 0.0, 0.0, 1.0)),
                        BuffType::Haste => ("HASTE", Vec4::new(0.0, 0.8, 1.0, 1.0)),
                        BuffType::Invincible => ("INVINCIBLE", Vec4::new(1.0, 1.0, 0.0, 1.0)),
                    };
                    let pulse = (self.game_time * 6.0).sin() * 0.2 + 0.8;
                    let pulsing_color = Vec4::new(
                        buff_color.x * pulse,
                        buff_color.y * pulse,
                        buff_color.z * pulse,
                        1.0,
                    );
                    world
                        .resources
                        .text_cache
                        .set_text(slot, format!("{}: {:.1}s", buff_name, buff.remaining_time));
                    if let Some(color) = world.ui.get_ui_node_color_mut(entity) {
                        color.colors[UiBase::INDEX] = Some(pulsing_color);
                    }
                    world.ui_set_visible(entity, true);
                } else {
                    world.ui_set_visible(entity, false);
                }
            }
        }

        if show_levelup {
            world.resources.text_cache.set_text(
                self.ui.levelup_title_slot,
                format!("LEVEL UP! (Lv.{})", self.player_level),
            );

            for (index, button) in self.ui.upgrade_buttons.iter().enumerate() {
                if index < self.upgrade_choices.len() {
                    let upgrade = self.upgrade_choices[index];
                    let current_level = self.stats.get_upgrade_level(upgrade);
                    let tier_color = upgrade.tier_color(current_level);
                    let is_selected = index == self.selected_upgrade_index;

                    let fill_color = if is_selected {
                        Vec4::new(
                            (tier_color.x * 0.6 + 0.4).min(1.0),
                            (tier_color.y * 0.6 + 0.4).min(1.0),
                            (tier_color.z * 0.6 + 0.4).min(1.0),
                            1.0,
                        )
                    } else {
                        Vec4::new(
                            tier_color.x * 0.4 + 0.1,
                            tier_color.y * 0.4 + 0.1,
                            tier_color.z * 0.4 + 0.1,
                            1.0,
                        )
                    };

                    world.ui_button_set_text(*button, &upgrade.tier_name(current_level));
                    if let Some(color) = world.ui.get_ui_node_color_mut(*button) {
                        color.colors[UiBase::INDEX] = Some(fill_color);
                    }
                    world.ui_set_visible(*button, true);

                    if world
                        .widget::<UiButtonData>(*button)
                        .is_some_and(|d| d.clicked)
                    {
                        self.apply_upgrade(upgrade, world);
                        self.game_state = GameState::Playing;
                        return;
                    }
                } else {
                    world.ui_set_visible(*button, false);
                }
            }

            if let Some(upgrade) = self.upgrade_choices.get(self.selected_upgrade_index) {
                let current_level = self.stats.get_upgrade_level(*upgrade);
                world.resources.text_cache.set_text(
                    self.ui.upgrade_desc_slot,
                    upgrade.description(current_level),
                );
            }
        }

        if show_gameover {
            world.ui_set_visible(
                self.ui.high_score_banner_entity,
                self.new_high_score_timer > 0.0,
            );

            if self.new_high_score_timer > 0.0 {
                let rainbow_phase = self.new_high_score_timer * 5.0;
                let r = (rainbow_phase.sin() * 0.5 + 0.5).max(0.3);
                let g = ((rainbow_phase + 2.094).sin() * 0.5 + 0.5).max(0.3);
                let b = ((rainbow_phase + 4.189).sin() * 0.5 + 0.5).max(0.3);
                let glow = (self.new_high_score_timer * 8.0).sin().abs() * 0.5 + 0.5;
                let high_score_color = Vec4::new(
                    (r + glow * 0.5).min(1.0),
                    (g + glow * 0.5).min(1.0),
                    (b + glow * 0.3).min(1.0),
                    1.0,
                );
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.high_score_banner_entity) {
                    color.colors[UiBase::INDEX] = Some(high_score_color);
                }
            }

            let kills = self.game_world.resources.enemies_killed;
            let wave = self.game_world.resources.current_wave;
            let is_kills_record = kills == self.high_score_kills && kills > 0;
            let is_wave_record = wave == self.high_score_wave && wave > 0;
            let is_time_record =
                (self.game_time - self.high_score_time).abs() < 0.1 && self.game_time > 0.0;
            let is_combo_record = self.combo_max == self.high_score_combo && self.combo_max > 0;

            let gold_color = Vec4::new(1.0, 0.84, 0.0, 1.0);
            let normal_color = Vec4::new(1.0, 1.0, 1.0, 1.0);

            world.resources.text_cache.set_text(
                self.ui.stats_level_slot,
                format!("Level: {}", self.player_level),
            );

            if is_wave_record && self.new_high_score_timer > 0.0 {
                world.resources.text_cache.set_text(
                    self.ui.stats_wave_slot,
                    format!("Wave: {} - NEW BEST!", wave),
                );
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.stats_wave_entity) {
                    color.colors[UiBase::INDEX] = Some(gold_color);
                }
            } else {
                world
                    .resources
                    .text_cache
                    .set_text(self.ui.stats_wave_slot, format!("Wave: {}", wave));
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.stats_wave_entity) {
                    color.colors[UiBase::INDEX] = Some(normal_color);
                }
            }

            if is_kills_record && self.new_high_score_timer > 0.0 {
                world.resources.text_cache.set_text(
                    self.ui.stats_kills_slot,
                    format!("Kills: {} - NEW BEST!", kills),
                );
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.stats_kills_entity) {
                    color.colors[UiBase::INDEX] = Some(gold_color);
                }
            } else {
                world
                    .resources
                    .text_cache
                    .set_text(self.ui.stats_kills_slot, format!("Kills: {}", kills));
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.stats_kills_entity) {
                    color.colors[UiBase::INDEX] = Some(normal_color);
                }
            }

            if is_time_record && self.new_high_score_timer > 0.0 {
                world.resources.text_cache.set_text(
                    self.ui.stats_time_slot,
                    format!("Time: {:.0}s - NEW BEST!", self.game_time),
                );
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.stats_time_entity) {
                    color.colors[UiBase::INDEX] = Some(gold_color);
                }
            } else {
                world.resources.text_cache.set_text(
                    self.ui.stats_time_slot,
                    format!("Time: {:.0}s", self.game_time),
                );
                if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.stats_time_entity) {
                    color.colors[UiBase::INDEX] = Some(normal_color);
                }
            }

            let show_combo_stat = self.combo_max > 1;
            world.ui_set_visible(self.ui.stats_combo_entity, show_combo_stat);
            if show_combo_stat {
                if is_combo_record && self.new_high_score_timer > 0.0 {
                    world.resources.text_cache.set_text(
                        self.ui.stats_combo_slot,
                        format!("Best Combo: {}x - NEW BEST!", self.combo_max),
                    );
                    if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.stats_combo_entity) {
                        color.colors[UiBase::INDEX] = Some(gold_color);
                    }
                } else {
                    world.resources.text_cache.set_text(
                        self.ui.stats_combo_slot,
                        format!("Best Combo: {}x", self.combo_max),
                    );
                    if let Some(color) = world.ui.get_ui_node_color_mut(self.ui.stats_combo_entity) {
                        color.colors[UiBase::INDEX] = Some(normal_color);
                    }
                }
            }

            world.resources.text_cache.set_text(
                self.ui.best_scores_slot,
                format!(
                    "Best Wave: {} | Best Kills: {}",
                    self.high_score_wave, self.high_score_kills
                ),
            );
            world.resources.text_cache.set_text(
                self.ui.best_scores_time_slot,
                format!(
                    "Best Time: {:.0}s | Best Combo: {}x",
                    self.high_score_time, self.high_score_combo
                ),
            );
        }

        // Flash effects via overlay
        let screen_size = world
            .resources
            .window
            .handle
            .as_ref()
            .map(|handle| {
                let size = handle.inner_size();
                Vec2::new(size.width as f32, size.height as f32)
            })
            .unwrap_or(Vec2::new(1920.0, 1080.0));

        let health_pct = self.player_health / self.stats.max_health;
        if health_pct < 0.3 && self.game_state == GameState::Playing {
            let pulse = ((self.game_time * 4.0).sin() * 0.5 + 0.5) * (0.3 - health_pct) / 0.3;
            let alpha = pulse * 0.31;
            world.resources.retained_ui.draw_overlay_rect(UiRect {
                position: Vec2::new(0.0, 0.0),
                size: screen_size,
                color: Vec4::new(1.0, 0.0, 0.0, alpha),
                layer: UiLayer::Tooltips,
                ..Default::default()
            });
        }

        if self.boss_kill_flash > 0.0 {
            let alpha = self.boss_kill_flash * 0.2;
            world.resources.retained_ui.draw_overlay_rect(UiRect {
                position: Vec2::new(0.0, 0.0),
                size: screen_size,
                color: Vec4::new(1.0, 0.5, 0.2, alpha),
                layer: UiLayer::Tooltips,
                ..Default::default()
            });
        }

        if self.level_up_flash > 0.0 {
            let alpha = self.level_up_flash * 0.15;
            world.resources.retained_ui.draw_overlay_rect(UiRect {
                position: Vec2::new(0.0, 0.0),
                size: screen_size,
                color: Vec4::new(1.0, 0.85, 0.3, alpha),
                layer: UiLayer::Tooltips,
                ..Default::default()
            });
        }
    }

    fn spawn_arena(&mut self, world: &mut World) {
        self.ground_entity = Some(spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(GROUND_SIZE, 1.0, GROUND_SIZE),
        ));

        let ground_material = format!("Ground_{}", self.ground_entity.unwrap().id);
        material_registry_insert(
            &mut world.resources.material_registry,
            ground_material.clone(),
            Material {
                base_color: [0.15, 0.35, 0.12, 1.0],
                roughness: 0.95,
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
        world.core.set_material_ref(
            self.ground_entity.unwrap(),
            MaterialRef::new(ground_material),
        );
    }

    fn spawn_player(&mut self, world: &mut World) {
        self.player_position = Vec3::new(0.0, PLAYER_RADIUS, 0.0);

        let player = spawn_mesh(
            world,
            "Sphere",
            self.player_position,
            Vec3::new(
                PLAYER_RADIUS * 2.0,
                PLAYER_RADIUS * 2.0,
                PLAYER_RADIUS * 2.0,
            ),
        );

        let body_material = format!("PlayerBody_{}", player.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            body_material.clone(),
            Material {
                base_color: [0.1, 0.7, 0.95, 1.0],
                roughness: 0.2,
                metallic: 0.5,
                emissive_factor: [0.1, 0.4, 0.6],
                ..Default::default()
            },
        );
        self.apply_material(world, player, &body_material);

        let stripe = spawn_mesh(
            world,
            "Torus",
            Vec3::new(0.0, 0.3, 0.0),
            Vec3::new(
                PLAYER_RADIUS * 1.4,
                PLAYER_RADIUS * 0.2,
                PLAYER_RADIUS * 1.4,
            ),
        );
        world.core.set_parent(stripe, Parent(Some(player)));

        let stripe_material = format!("PlayerStripe_{}", stripe.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            stripe_material.clone(),
            Material {
                base_color: [1.0, 0.85, 0.2, 1.0],
                roughness: 0.2,
                metallic: 0.7,
                emissive_factor: [0.5, 0.4, 0.0],
                ..Default::default()
            },
        );
        self.apply_material(world, stripe, &stripe_material);

        self.player_entity = Some(player);
    }

    fn apply_material(&self, world: &mut World, entity: Entity, material_name: &str) {
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(entity, MaterialRef::new(material_name.to_string()));
    }

    fn spawn_camera(&mut self, world: &mut World) {
        let camera_position = self.player_position + Vec3::new(0.0, CAMERA_HEIGHT, CAMERA_DISTANCE);
        let camera = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | CAMERA,
            1,
        )[0];

        if let Some(transform) = world.core.get_local_transform_mut(camera) {
            transform.translation = camera_position;
            let direction = nalgebra_glm::normalize(&(self.player_position - camera_position));
            let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &Vec3::y()));
            let up = nalgebra_glm::cross(&right, &direction);
            let rotation_matrix = nalgebra_glm::Mat3::from_columns(&[right, up, -direction]);
            transform.rotation = nalgebra_glm::mat3_to_quat(&rotation_matrix);
        }
        mark_local_transform_dirty(world, camera);

        world.core.set_camera(
            camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 45.0_f32.to_radians(),
                    z_far: Some(500.0),
                    z_near: 0.1,
                }),
                smoothing: Some(Smoothing::default()),
            },
        );

        world.resources.active_camera = Some(camera);
        self.camera_entity = Some(camera);
    }

    fn spawn_lighting(&mut self, world: &mut World) {
        spawn_sun(world);
    }

    fn create_materials(&mut self, world: &mut World) {
        self.create_enemy_material(world, "EnemyNormal", [0.9, 0.2, 0.2, 1.0], [0.0, 0.0, 0.0]);
        self.enemy_materials.normal = Some("EnemyNormal".to_string());

        self.create_enemy_material(world, "EnemyFast", [1.0, 0.6, 0.1, 1.0], [0.3, 0.2, 0.0]);
        self.enemy_materials.fast = Some("EnemyFast".to_string());

        self.create_enemy_material(world, "EnemyTank", [0.4, 0.2, 0.5, 1.0], [0.1, 0.0, 0.2]);
        self.enemy_materials.tank = Some("EnemyTank".to_string());

        self.create_enemy_material(
            world,
            "EnemyExploder",
            [0.2, 0.8, 0.2, 1.0],
            [0.1, 0.4, 0.1],
        );
        self.enemy_materials.exploder = Some("EnemyExploder".to_string());

        self.create_enemy_material(world, "EnemyBoss", [0.8, 0.1, 0.1, 1.0], [0.6, 0.1, 0.1]);
        self.enemy_materials.boss = Some("EnemyBoss".to_string());

        let projectile_material_name = "ProjectileMaterial".to_string();
        material_registry_insert(
            &mut world.resources.material_registry,
            projectile_material_name.clone(),
            Material {
                base_color: [0.6, 0.2, 0.9, 1.0],
                roughness: 0.2,
                metallic: 0.3,
                emissive_factor: [0.5, 0.1, 0.8],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&projectile_material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        self.projectile_material_name = Some(projectile_material_name);

        let gem_material_name = "GemMaterial".to_string();
        material_registry_insert(
            &mut world.resources.material_registry,
            gem_material_name.clone(),
            Material {
                base_color: [0.2, 0.9, 0.3, 1.0],
                roughness: 0.2,
                metallic: 0.4,
                emissive_factor: [0.1, 0.6, 0.2],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&gem_material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        self.gem_material_name = Some(gem_material_name);

        let orb_material_name = "OrbMaterial".to_string();
        material_registry_insert(
            &mut world.resources.material_registry,
            orb_material_name.clone(),
            Material {
                base_color: [0.3, 0.6, 1.0, 1.0],
                roughness: 0.1,
                metallic: 0.8,
                emissive_factor: [0.2, 0.5, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&orb_material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        self.orb_material_name = Some(orb_material_name);
    }

    fn create_enemy_material(
        &self,
        world: &mut World,
        name: &str,
        base_color: [f32; 4],
        emissive: [f32; 3],
    ) {
        material_registry_insert(
            &mut world.resources.material_registry,
            name.to_string(),
            Material {
                base_color,
                roughness: 0.5,
                metallic: 0.1,
                emissive_factor: emissive,
                ..Default::default()
            },
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

    fn player_movement_system(&mut self, world: &mut World, delta: f32) {
        let keyboard = &world.resources.input.keyboard;

        let mut movement = Vec3::zeros();

        if keyboard.is_key_pressed(KeyCode::KeyW) || keyboard.is_key_pressed(KeyCode::ArrowUp) {
            movement.z -= 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::KeyS) || keyboard.is_key_pressed(KeyCode::ArrowDown) {
            movement.z += 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::KeyA) || keyboard.is_key_pressed(KeyCode::ArrowLeft) {
            movement.x -= 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::KeyD) || keyboard.is_key_pressed(KeyCode::ArrowRight) {
            movement.x += 1.0;
        }

        if let Some(gamepad) = query_active_gamepad(world) {
            let left_stick_x = gamepad.value(gilrs::Axis::LeftStickX);
            let left_stick_y = gamepad.value(gilrs::Axis::LeftStickY);

            const DEADZONE: f32 = 0.15;

            if left_stick_x.abs() > DEADZONE {
                movement.x += left_stick_x;
            }
            if left_stick_y.abs() > DEADZONE {
                movement.z -= left_stick_y;
            }

            let dpad_up = gamepad.is_pressed(gilrs::Button::DPadUp);
            let dpad_down = gamepad.is_pressed(gilrs::Button::DPadDown);
            let dpad_left = gamepad.is_pressed(gilrs::Button::DPadLeft);
            let dpad_right = gamepad.is_pressed(gilrs::Button::DPadRight);

            if dpad_up {
                movement.z -= 1.0;
            }
            if dpad_down {
                movement.z += 1.0;
            }
            if dpad_left {
                movement.x -= 1.0;
            }
            if dpad_right {
                movement.x += 1.0;
            }
        }

        let gesture = world.resources.input.touch.gesture;
        if let TouchGesture::SingleDrag { delta: touch_delta } = gesture {
            let sensitivity = 0.02;
            movement.x += touch_delta.x * sensitivity;
            movement.z += touch_delta.y * sensitivity;
        }

        let is_moving = nalgebra_glm::length(&movement) > 0.0;

        if is_moving {
            if nalgebra_glm::length(&movement) > 1.0 {
                movement = nalgebra_glm::normalize(&movement);
            }

            self.player_facing = movement;

            let mut speed =
                PLAYER_SPEED * self.stats.speed_multiplier * self.stats.buff_speed_multiplier;
            if self.speed_boost_timer > 0.0 {
                speed *= SPEED_BOOST_MULTIPLIER;
            }

            self.player_position += movement * speed * delta;

            self.dust_timer += delta;
            if self.dust_timer >= DUST_SPAWN_INTERVAL {
                self.dust_timer = 0.0;
                self.spawn_dust_particle(world, self.player_position);
            }
        }

        if let Some(entity) = self.player_entity {
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.translation = self.player_position;
            }
            mark_local_transform_dirty(world, entity);
        }

        let distance_from_origin =
            nalgebra_glm::length(&Vec2::new(self.player_position.x, self.player_position.z));
        if distance_from_origin > self.max_distance_traveled {
            self.max_distance_traveled = distance_from_origin;
            self.try_spawn_treasure_zone(world);
        }
    }

    fn camera_follow_system(&mut self, world: &mut World, delta: f32) {
        if self.camera_shake > 0.0 {
            self.camera_shake = (self.camera_shake - delta * 8.0).max(0.0);
        }

        if let Some(camera) = self.camera_entity {
            let mut target_position =
                self.player_position + Vec3::new(0.0, CAMERA_HEIGHT, CAMERA_DISTANCE);

            if self.camera_shake > 0.0 {
                let mut rng = rand::rng();
                let shake_amount = self.camera_shake * 0.5;
                target_position.x += rng.random_range(-shake_amount..shake_amount);
                target_position.y += rng.random_range(-shake_amount..shake_amount);
                target_position.z += rng.random_range(-shake_amount..shake_amount);
            }

            if let Some(transform) = world.core.get_local_transform_mut(camera) {
                transform.translation = target_position;
            }
            mark_local_transform_dirty(world, camera);
        }
    }

    fn update_ground_position(&mut self, world: &mut World) {
        if let Some(ground) = self.ground_entity {
            if let Some(transform) = world.core.get_local_transform_mut(ground) {
                transform.translation.x = self.player_position.x;
                transform.translation.z = self.player_position.z;
            }
            mark_local_transform_dirty(world, ground);
        }
    }

    fn update_chunks(&mut self, world: &mut World) {
        let player_chunk_x = (self.player_position.x / CHUNK_SIZE).floor() as i32;
        let player_chunk_z = (self.player_position.z / CHUNK_SIZE).floor() as i32;

        let current_distance =
            (self.player_position.x.powi(2) + self.player_position.z.powi(2)).sqrt();
        if current_distance > self.max_distance_traveled {
            self.max_distance_traveled = current_distance;
        }

        let mut chunks_to_load = Vec::new();
        let mut chunks_to_unload = Vec::new();

        for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
                let chunk = (player_chunk_x + dx, player_chunk_z + dz);
                if !self.loaded_chunks.contains(&chunk) {
                    chunks_to_load.push(chunk);
                }
            }
        }

        for &chunk in &self.loaded_chunks {
            let (chunk_x, chunk_z) = chunk;
            if (chunk_x - player_chunk_x).abs() > RENDER_DISTANCE + 1
                || (chunk_z - player_chunk_z).abs() > RENDER_DISTANCE + 1
            {
                chunks_to_unload.push(chunk);
            }
        }

        for chunk in chunks_to_unload {
            self.unload_chunk(world, chunk);
        }

        for chunk in chunks_to_load {
            self.load_chunk(world, chunk);
        }
    }

    fn load_chunk(&mut self, world: &mut World, chunk: (i32, i32)) {
        let (chunk_x, chunk_z) = chunk;
        let mut entities = Vec::new();
        let seed =
            (chunk_x as u64).wrapping_mul(73856093) ^ (chunk_z as u64).wrapping_mul(19349663);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let chunk_base_x = chunk_x as f32 * CHUNK_SIZE;
        let chunk_base_z = chunk_z as f32 * CHUNK_SIZE;

        let tree_count: u32 = rng.random_range(2..5);
        for tree_index in 0..tree_count {
            let x = chunk_base_x + rng.random_range(1.0..CHUNK_SIZE - 1.0);
            let z = chunk_base_z + rng.random_range(1.0..CHUNK_SIZE - 1.0);

            let trunk_height = 2.0 + rng.random_range(0.0..1.5);
            let trunk_radius = 0.2 + rng.random_range(0.0..0.1);
            let tree_scale = 0.8 + rng.random_range(0.0..0.5);

            let trunk = world.spawn_entities(
                LOCAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | GLOBAL_TRANSFORM
                    | RENDER_MESH
                    | MATERIAL_REF
                    | CASTS_SHADOW,
                1,
            )[0];
            world.core.set_local_transform(
                trunk,
                LocalTransform {
                    translation: Vec3::new(x, trunk_height / 2.0, z),
                    rotation: Quat::identity(),
                    scale: Vec3::new(trunk_radius, trunk_height, trunk_radius),
                },
            );
            world.core.set_render_mesh(trunk, RenderMesh::new("Cylinder"));

            let trunk_material_name =
                format!("TreeTrunk_{}_{}", chunk_x.wrapping_abs(), tree_index);
            material_registry_insert(
                &mut world.resources.material_registry,
                trunk_material_name.clone(),
                Material {
                    base_color: [0.35, 0.22, 0.12, 1.0],
                    roughness: 0.95,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&trunk_material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(mat_index);
            }
            world.core.set_material_ref(trunk, MaterialRef::new(trunk_material_name));
            world.core.set_casts_shadow(trunk, CastsShadow);
            entities.push(trunk);

            let foliage_tiers = 3;
            let tier_heights = [1.8 * tree_scale, 1.5 * tree_scale, 1.2 * tree_scale];
            let tier_radii = [2.0 * tree_scale, 1.5 * tree_scale, 1.0 * tree_scale];
            let tier_offsets = [0.0, 1.0 * tree_scale, 1.8 * tree_scale];

            for tier in 0..foliage_tiers {
                let cone = world.spawn_entities(
                    LOCAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | GLOBAL_TRANSFORM
                        | RENDER_MESH
                        | MATERIAL_REF
                        | CASTS_SHADOW,
                    1,
                )[0];

                let tier_y = trunk_height + tier_offsets[tier] + tier_heights[tier] / 2.0;
                world.core.set_local_transform(
                    cone,
                    LocalTransform {
                        translation: Vec3::new(x, tier_y, z),
                        rotation: Quat::identity(),
                        scale: Vec3::new(tier_radii[tier], tier_heights[tier], tier_radii[tier]),
                    },
                );
                world.core.set_render_mesh(cone, RenderMesh::new("Cone"));

                let green_variation = rng.random_range(0.0..0.15);
                let cone_material_name = format!(
                    "TreeCone_{}_{}_{}",
                    chunk_x.wrapping_abs(),
                    tree_index,
                    tier
                );
                material_registry_insert(
                    &mut world.resources.material_registry,
                    cone_material_name.clone(),
                    Material {
                        base_color: [0.1, 0.4 + green_variation, 0.08, 1.0],
                        roughness: 0.9,
                        metallic: 0.0,
                        ..Default::default()
                    },
                );
                if let Some(&mat_index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&cone_material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(mat_index);
                }
                world.core.set_material_ref(cone, MaterialRef::new(cone_material_name));
                world.core.set_casts_shadow(cone, CastsShadow);
                entities.push(cone);
            }
        }

        let rock_count: u32 = rng.random_range(1..4);
        for rock_index in 0..rock_count {
            let x = chunk_base_x + rng.random_range(1.0..CHUNK_SIZE - 1.0);
            let z = chunk_base_z + rng.random_range(1.0..CHUNK_SIZE - 1.0);
            let size = 0.3 + rng.random_range(0.0..0.5);

            let rock = world.spawn_entities(
                LOCAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | GLOBAL_TRANSFORM
                    | RENDER_MESH
                    | MATERIAL_REF
                    | CASTS_SHADOW,
                1,
            )[0];
            world.core.set_local_transform(
                rock,
                LocalTransform {
                    translation: Vec3::new(x, size * 0.4, z),
                    rotation: Quat::identity(),
                    scale: Vec3::new(size, size * 0.7, size),
                },
            );
            world.core.set_render_mesh(rock, RenderMesh::new("Sphere"));

            let gray = 0.35 + rng.random_range(0.0..0.2);
            let rock_material_name = format!("Rock_{}_{}", chunk_x.wrapping_abs(), rock_index);
            material_registry_insert(
                &mut world.resources.material_registry,
                rock_material_name.clone(),
                Material {
                    base_color: [gray, gray - 0.02, gray - 0.05, 1.0],
                    roughness: 0.95,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&rock_material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(mat_index);
            }
            world.core.set_material_ref(rock, MaterialRef::new(rock_material_name));
            world.core.set_casts_shadow(rock, CastsShadow);
            entities.push(rock);
        }

        self.loaded_chunks.insert(chunk);
        self.chunk_entities.insert(chunk, entities);
    }

    fn unload_chunk(&mut self, world: &mut World, chunk: (i32, i32)) {
        if let Some(entities) = self.chunk_entities.remove(&chunk) {
            for entity in entities {
                world.queue_despawn_entity(entity);
            }
        }
        self.loaded_chunks.remove(&chunk);
    }

    fn enemy_spawn_system(&mut self, world: &mut World, delta: f32) {
        if self.game_world.resources.current_wave == 0 {
            self.game_world.resources.current_wave = 1;
            self.game_world.resources.wave_enemies_remaining = WAVE_ENEMIES_BASE;
        }

        if self.game_world.resources.wave_enemies_remaining == 0
            && self.game_world.resources.enemy_list.is_empty()
        {
            self.spawn_wave_complete_effect(world);
            self.advance_wave(world);
        }

        self.game_world.resources.spawn_timer += delta;

        let wave = self.game_world.resources.current_wave;
        let difficulty_multiplier =
            1.0 + (self.player_level as f32 - 1.0) * 0.15 + (wave as f32 - 1.0) * 0.1;
        let spawn_interval = (SPAWN_INTERVAL / difficulty_multiplier).max(0.08);

        if self.game_world.resources.spawn_timer >= spawn_interval
            && self.game_world.resources.wave_enemies_remaining > 0
        {
            self.game_world.resources.spawn_timer = 0.0;

            let enemies_to_spawn = if self.player_level >= 5 && rand::rng().random::<f32>() < 0.3 {
                2.min(self.game_world.resources.wave_enemies_remaining)
            } else {
                1.min(self.game_world.resources.wave_enemies_remaining)
            };

            for _ in 0..enemies_to_spawn {
                self.spawn_enemy(world);
                self.game_world.resources.wave_enemies_remaining -= 1;
            }
        }
    }

    fn advance_wave(&mut self, world: &mut World) {
        self.game_world.resources.current_wave += 1;
        let wave = self.game_world.resources.current_wave;
        self.game_world.resources.wave_enemies_remaining = WAVE_ENEMIES_BASE + wave * 5;

        if wave.is_multiple_of(BOSS_WAVE_INTERVAL) && !self.game_world.resources.boss_alive {
            self.spawn_boss(world);
        }
    }

    fn spawn_boss(&mut self, world: &mut World) {
        let mut rng = rand::rng();

        let spawn_distance = rng.random_range(30.0..40.0);
        let spawn_angle = rng.random_range(0.0..std::f32::consts::TAU);
        let spawn_position = Vec3::new(
            self.player_position.x + spawn_angle.cos() * spawn_distance,
            BOSS_RADIUS,
            self.player_position.z + spawn_angle.sin() * spawn_distance,
        );

        let engine_entity =
            self.spawn_enemy_mesh(world, spawn_position, BOSS_RADIUS, EnemyType::Boss);

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | POSITION | VELOCITY | ENEMY, 1)[0];

        let wave = self.game_world.resources.current_wave;
        let boss_health = BOSS_HEALTH * (1.0 + wave as f32 * 0.5);

        let shield_hits = 3;

        self.game_world
            .set_entity_handle(game_entity, EntityHandle(engine_entity));
        self.game_world
            .set_position(game_entity, Position(spawn_position));
        self.game_world
            .set_velocity(game_entity, Velocity(Vec3::zeros()));
        self.game_world.set_enemy(
            game_entity,
            Enemy {
                speed: BOSS_SPEED,
                health: boss_health,
                enemy_type: EnemyType::Boss,
                xp_value: BOSS_XP,
                shield_hits,
            },
        );

        if shield_hits > 0 {
            self.spawn_enemy_shield(world, game_entity, engine_entity, BOSS_RADIUS);
        }

        self.game_world.resources.enemy_list.push(game_entity);
        self.game_world.resources.boss_alive = true;

        self.spawn_enemy_spawn_effect(world, spawn_position, EnemyType::Boss);
        self.spawn_boss_entrance_effect(world, spawn_position);
        self.camera_shake = 1.5;
    }

    fn spawn_boss_entrance_effect(&mut self, world: &mut World, position: Vec3) {
        for ring_index in 0..4 {
            let line_entity = world.spawn_entities(
                LINES | VISIBILITY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
                1,
            )[0];

            let delay = ring_index as f32 * 0.1;
            let effect = LineEffect {
                entity: line_entity,
                timer: -delay,
                max_time: 0.6,
                center: Vec3::new(position.x, 0.1, position.z),
                start_radius: 0.0,
                end_radius: 4.0 + ring_index as f32 * 1.5,
                segments: 32,
                color_start: Vec4::new(1.0, 0.2, 0.2, 1.0),
                color_end: Vec4::new(0.8, 0.0, 0.0, 0.0),
            };
            self.line_effects.push(effect);
        }

        let pillar_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
        let pillar_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.3, 0.3, 1.0)),
                (0.3, Vec4::new(1.0, 0.1, 0.1, 0.8)),
                (0.6, Vec4::new(0.8, 0.0, 0.0, 0.5)),
                (1.0, Vec4::new(0.4, 0.0, 0.0, 0.0)),
            ],
        };

        let pillar_emitter = ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Sphere { radius: 1.5 },
            position: Vec3::new(position.x, 0.0, position.z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 200,
            particle_lifetime_min: 0.8,
            particle_lifetime_max: 1.5,
            initial_velocity_min: 8.0,
            initial_velocity_max: 15.0,
            velocity_spread: 0.3,
            gravity: Vec3::new(0.0, 2.0, 0.0),
            drag: 0.2,
            size_start: 0.6,
            size_end: 0.1,
            color_gradient: pillar_gradient,
            emissive_strength: 25.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.4,
            turbulence_frequency: 3.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(pillar_entity, pillar_emitter);
    }

    fn spawn_enemy(&mut self, world: &mut World) {
        let mut rng = rand::rng();

        let enemy_type = self.pick_enemy_type(&mut rng);

        let (base_speed, base_health, radius_mult, xp_value) = match enemy_type {
            EnemyType::Normal => (ENEMY_SPEED, 1.0, 1.0, 5u32),
            EnemyType::Fast => (ENEMY_SPEED * 1.8, 0.5, 0.7, 7),
            EnemyType::Tank => (ENEMY_SPEED * 0.5, 4.0, 1.5, 15),
            EnemyType::Exploder => (ENEMY_SPEED * 1.2, 1.5, 0.9, 10),
            EnemyType::Boss => (BOSS_SPEED, BOSS_HEALTH, BOSS_RADIUS / ENEMY_RADIUS, BOSS_XP),
        };

        let radius = ENEMY_RADIUS * radius_mult;

        let spawn_distance = rng.random_range(25.0..35.0);
        let spawn_angle = rng.random_range(0.0..std::f32::consts::TAU);
        let spawn_position = Vec3::new(
            self.player_position.x + spawn_angle.cos() * spawn_distance,
            radius,
            self.player_position.z + spawn_angle.sin() * spawn_distance,
        );

        let engine_entity = self.spawn_enemy_mesh(world, spawn_position, radius, enemy_type);

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | POSITION | VELOCITY | ENEMY, 1)[0];

        let speed_multiplier = 1.0 + (self.player_level as f32 - 1.0) * 0.08;
        let enemy_speed = base_speed * speed_multiplier * rng.random_range(0.9..1.1);
        let health_multiplier = 1.0 + (self.player_level as f32 - 1.0) * 0.1;
        let enemy_health = base_health * health_multiplier;

        let shield_hits = match enemy_type {
            EnemyType::Tank => 2,
            EnemyType::Boss => 3,
            _ => 0,
        };

        self.game_world
            .set_entity_handle(game_entity, EntityHandle(engine_entity));
        self.game_world
            .set_position(game_entity, Position(spawn_position));
        self.game_world
            .set_velocity(game_entity, Velocity(Vec3::zeros()));
        self.game_world.set_enemy(
            game_entity,
            Enemy {
                speed: enemy_speed,
                health: enemy_health,
                enemy_type,
                xp_value,
                shield_hits,
            },
        );

        if shield_hits > 0 {
            self.spawn_enemy_shield(world, game_entity, engine_entity, radius);
        }

        self.game_world.resources.enemy_list.push(game_entity);
        self.game_world.resources.enemies_spawned += 1;

        self.spawn_enemy_spawn_effect(world, spawn_position, enemy_type);
    }

    fn spawn_enemy_mesh(
        &mut self,
        world: &mut World,
        position: Vec3,
        radius: f32,
        enemy_type: EnemyType,
    ) -> Entity {
        match enemy_type {
            EnemyType::Normal => {
                let body = spawn_mesh(
                    world,
                    "Cube",
                    position,
                    Vec3::new(radius * 1.6, radius * 1.6, radius * 1.6),
                );
                let mat = format!("NormalEnemy_{}", body.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    mat.clone(),
                    Material {
                        base_color: [0.9, 0.2, 0.2, 1.0],
                        roughness: 0.6,
                        metallic: 0.1,
                        emissive_factor: [0.3, 0.0, 0.0],
                        ..Default::default()
                    },
                );
                self.apply_material(world, body, &mat);
                body
            }
            EnemyType::Fast => {
                let body = spawn_mesh(
                    world,
                    "Cone",
                    position,
                    Vec3::new(radius * 1.2, radius * 2.0, radius * 1.2),
                );
                let mat = format!("FastEnemy_{}", body.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    mat.clone(),
                    Material {
                        base_color: [1.0, 0.5, 0.1, 1.0],
                        roughness: 0.3,
                        metallic: 0.5,
                        emissive_factor: [0.5, 0.2, 0.0],
                        ..Default::default()
                    },
                );
                self.apply_material(world, body, &mat);

                let tail = spawn_mesh(
                    world,
                    "Cone",
                    Vec3::new(0.0, -radius * 0.8, 0.0),
                    Vec3::new(radius * 0.6, radius * 0.8, radius * 0.6),
                );
                world.core.set_parent(tail, Parent(Some(body)));
                if let Some(t) = world.core.get_local_transform_mut(tail) {
                    t.rotation = nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::x());
                }
                let tail_mat = format!("FastEnemyTail_{}", tail.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    tail_mat.clone(),
                    Material {
                        base_color: [1.0, 0.7, 0.3, 1.0],
                        emissive_factor: [0.6, 0.3, 0.0],
                        ..Default::default()
                    },
                );
                self.apply_material(world, tail, &tail_mat);
                body
            }
            EnemyType::Tank => {
                let body = spawn_mesh(
                    world,
                    "Cylinder",
                    position,
                    Vec3::new(radius * 2.0, radius * 1.4, radius * 2.0),
                );
                let mat = format!("TankEnemy_{}", body.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    mat.clone(),
                    Material {
                        base_color: [0.3, 0.2, 0.4, 1.0],
                        roughness: 0.8,
                        metallic: 0.3,
                        emissive_factor: [0.1, 0.05, 0.15],
                        ..Default::default()
                    },
                );
                self.apply_material(world, body, &mat);

                let turret = spawn_mesh(
                    world,
                    "Cube",
                    Vec3::new(0.0, radius * 1.0, 0.0),
                    Vec3::new(radius * 1.0, radius * 0.6, radius * 1.0),
                );
                world.core.set_parent(turret, Parent(Some(body)));
                let turret_mat = format!("TankTurret_{}", turret.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    turret_mat.clone(),
                    Material {
                        base_color: [0.2, 0.15, 0.3, 1.0],
                        roughness: 0.7,
                        metallic: 0.4,
                        emissive_factor: [0.2, 0.1, 0.3],
                        ..Default::default()
                    },
                );
                self.apply_material(world, turret, &turret_mat);
                body
            }
            EnemyType::Exploder => {
                let body = spawn_mesh(
                    world,
                    "Sphere",
                    position,
                    Vec3::new(radius * 1.6, radius * 1.6, radius * 1.6),
                );
                let mat = format!("ExploderEnemy_{}", body.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    mat.clone(),
                    Material {
                        base_color: [1.0, 0.9, 0.2, 1.0],
                        roughness: 0.4,
                        metallic: 0.2,
                        emissive_factor: [0.6, 0.5, 0.0],
                        ..Default::default()
                    },
                );
                self.apply_material(world, body, &mat);

                for index in 0..6 {
                    let angle = (index as f32 / 6.0) * std::f32::consts::TAU;
                    let spike_pos =
                        Vec3::new(angle.cos() * radius * 0.7, 0.0, angle.sin() * radius * 0.7);
                    let spike = spawn_mesh(
                        world,
                        "Cone",
                        spike_pos,
                        Vec3::new(radius * 0.3, radius * 0.5, radius * 0.3),
                    );
                    world.core.set_parent(spike, Parent(Some(body)));
                    if let Some(t) = world.core.get_local_transform_mut(spike) {
                        t.rotation = nalgebra_glm::quat_angle_axis(
                            std::f32::consts::FRAC_PI_2,
                            &Vec3::new(-angle.sin(), 0.0, angle.cos()),
                        );
                    }
                    let spike_mat = format!("ExploderSpike_{}_{}", body.id, index);
                    material_registry_insert(
                        &mut world.resources.material_registry,
                        spike_mat.clone(),
                        Material {
                            base_color: [1.0, 0.4, 0.1, 1.0],
                            emissive_factor: [0.8, 0.3, 0.0],
                            ..Default::default()
                        },
                    );
                    self.apply_material(world, spike, &spike_mat);
                }
                body
            }
            EnemyType::Boss => {
                let body = spawn_mesh(
                    world,
                    "Torus",
                    position,
                    Vec3::new(radius * 2.0, radius * 0.6, radius * 2.0),
                );
                let mat = format!("BossEnemy_{}", body.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    mat.clone(),
                    Material {
                        base_color: [0.5, 0.0, 0.1, 1.0],
                        roughness: 0.3,
                        metallic: 0.7,
                        emissive_factor: [0.4, 0.0, 0.1],
                        ..Default::default()
                    },
                );
                self.apply_material(world, body, &mat);

                let core = spawn_mesh(
                    world,
                    "Sphere",
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(radius * 1.0, radius * 1.0, radius * 1.0),
                );
                world.core.set_parent(core, Parent(Some(body)));
                let core_mat = format!("BossCore_{}", core.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    core_mat.clone(),
                    Material {
                        base_color: [0.1, 0.0, 0.05, 1.0],
                        roughness: 0.1,
                        metallic: 0.9,
                        emissive_factor: [0.6, 0.0, 0.2],
                        ..Default::default()
                    },
                );
                self.apply_material(world, core, &core_mat);

                for index in 0..4 {
                    let angle = (index as f32 / 4.0) * std::f32::consts::TAU;
                    let horn_pos = Vec3::new(
                        angle.cos() * radius * 1.2,
                        radius * 0.5,
                        angle.sin() * radius * 1.2,
                    );
                    let horn = spawn_mesh(
                        world,
                        "Cone",
                        horn_pos,
                        Vec3::new(radius * 0.4, radius * 0.8, radius * 0.4),
                    );
                    world.core.set_parent(horn, Parent(Some(body)));
                    let horn_mat = format!("BossHorn_{}_{}", body.id, index);
                    material_registry_insert(
                        &mut world.resources.material_registry,
                        horn_mat.clone(),
                        Material {
                            base_color: [0.2, 0.0, 0.05, 1.0],
                            emissive_factor: [0.5, 0.0, 0.15],
                            ..Default::default()
                        },
                    );
                    self.apply_material(world, horn, &horn_mat);
                }
                body
            }
        }
    }

    fn pick_enemy_type(&self, rng: &mut impl rand::Rng) -> EnemyType {
        let roll: f32 = rng.random();

        if self.player_level < 3 {
            EnemyType::Normal
        } else if self.player_level < 5 {
            if roll < 0.7 {
                EnemyType::Normal
            } else {
                EnemyType::Fast
            }
        } else if self.player_level < 8 {
            if roll < 0.5 {
                EnemyType::Normal
            } else if roll < 0.75 {
                EnemyType::Fast
            } else {
                EnemyType::Tank
            }
        } else if roll < 0.35 {
            EnemyType::Normal
        } else if roll < 0.55 {
            EnemyType::Fast
        } else if roll < 0.75 {
            EnemyType::Tank
        } else {
            EnemyType::Exploder
        }
    }

    fn enemy_chase_system(&mut self, world: &mut World, delta: f32) {
        let player_pos = self.player_position;

        let enemies: Vec<freecs::Entity> = self
            .game_world
            .query_entities(ENEMY | POSITION | ENTITY_HANDLE)
            .collect();

        for game_entity in enemies {
            let enemy = self.game_world.get_enemy(game_entity).unwrap();
            let position = self.game_world.get_position(game_entity).unwrap();

            let direction = player_pos - position.0;
            let distance = nalgebra_glm::length(&direction);

            if distance > 0.01 {
                let normalized = direction / distance;
                let velocity = normalized * enemy.speed;
                let new_position = Position(position.0 + velocity * delta);

                self.game_world.set_position(game_entity, new_position);

                let handle = self.game_world.get_entity_handle(game_entity).unwrap();
                if let Some(transform) = world.core.get_local_transform_mut(handle.0) {
                    transform.translation = new_position.0;
                }
                mark_local_transform_dirty(world, handle.0);
            }
        }
    }

    fn find_nearest_enemy(&self) -> Option<(freecs::Entity, Vec3, f32)> {
        let player_pos = self.player_position;
        let range = PROJECTILE_RANGE * self.stats.range_multiplier;
        let mut nearest: Option<(freecs::Entity, Vec3, f32)> = None;

        for game_entity in self.game_world.query_entities(ENEMY | POSITION) {
            let position = self.game_world.get_position(game_entity).unwrap();
            let distance = nalgebra_glm::distance(&player_pos, &position.0);

            if distance <= range {
                match nearest {
                    None => nearest = Some((game_entity, position.0, distance)),
                    Some((_, _, nearest_dist)) if distance < nearest_dist => {
                        nearest = Some((game_entity, position.0, distance));
                    }
                    _ => {}
                }
            }
        }

        nearest
    }

    fn attack_system(&mut self, world: &mut World, delta: f32) {
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= delta;
            return;
        }

        if let Some((_, enemy_pos, _)) = self.find_nearest_enemy() {
            for index in 0..self.stats.projectile_count {
                let angle_offset = if self.stats.projectile_count > 1 {
                    let spread = 0.3;
                    (index as f32 - (self.stats.projectile_count - 1) as f32 / 2.0) * spread
                } else {
                    0.0
                };
                self.spawn_projectile(world, enemy_pos, angle_offset);
            }
            self.attack_cooldown = PROJECTILE_COOLDOWN * self.stats.cooldown_multiplier;
        }
    }

    fn spawn_projectile(&mut self, world: &mut World, target_pos: Vec3, angle_offset: f32) {
        let spawn_pos = self.player_position;
        let base_direction = nalgebra_glm::normalize(&(target_pos - spawn_pos));

        let rotation = nalgebra_glm::quat_angle_axis(angle_offset, &Vec3::y());
        let direction = nalgebra_glm::quat_rotate_vec3(&rotation, &base_direction);
        let velocity = direction * PROJECTILE_SPEED;

        let engine_entity = spawn_mesh(
            world,
            "Cylinder",
            spawn_pos,
            Vec3::new(
                PROJECTILE_RADIUS * 1.0,
                PROJECTILE_RADIUS * 3.0,
                PROJECTILE_RADIUS * 1.0,
            ),
        );

        if let Some(transform) = world.core.get_local_transform_mut(engine_entity) {
            let forward = Vec3::new(0.0, 1.0, 0.0);
            let axis = nalgebra_glm::cross(&forward, &direction);
            let axis_len = nalgebra_glm::length(&axis);
            if axis_len > 0.001 {
                let axis_norm = axis / axis_len;
                let angle = forward.dot(&direction).acos();
                transform.rotation = nalgebra_glm::quat_angle_axis(angle, &axis_norm);
            }
        }
        mark_local_transform_dirty(world, engine_entity);

        let mat = format!("Projectile_{}", engine_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            mat.clone(),
            Material {
                base_color: [0.2, 0.7, 1.0, 1.0],
                roughness: 0.1,
                metallic: 0.8,
                emissive_factor: [0.3, 0.6, 1.0],
                ..Default::default()
            },
        );
        self.apply_material(world, engine_entity, &mat);

        let tip = spawn_mesh(
            world,
            "Cone",
            Vec3::new(0.0, PROJECTILE_RADIUS * 1.5, 0.0),
            Vec3::new(
                PROJECTILE_RADIUS * 1.2,
                PROJECTILE_RADIUS * 1.0,
                PROJECTILE_RADIUS * 1.2,
            ),
        );
        world.core.set_parent(tip, Parent(Some(engine_entity)));
        let tip_mat = format!("ProjectileTip_{}", tip.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            tip_mat.clone(),
            Material {
                base_color: [0.5, 0.9, 1.0, 1.0],
                emissive_factor: [0.5, 0.8, 1.0],
                ..Default::default()
            },
        );
        self.apply_material(world, tip, &tip_mat);

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | POSITION | VELOCITY | PROJECTILE, 1)[0];

        let damage = 50.0 * self.stats.damage_multiplier * self.stats.buff_damage_multiplier;

        let trail_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
        let trail_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.3, 0.8, 1.0, 0.8)),
                (0.3, Vec4::new(0.5, 0.9, 1.0, 0.5)),
                (0.7, Vec4::new(0.7, 0.95, 1.0, 0.2)),
                (1.0, Vec4::new(1.0, 1.0, 1.0, 0.0)),
            ],
        };
        let trail_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            position: spawn_pos,
            direction: Vec3::new(0.0, 0.0, -1.0),
            spawn_rate: 60.0,
            burst_count: 0,
            particle_lifetime_min: 0.1,
            particle_lifetime_max: 0.25,
            initial_velocity_min: 0.5,
            initial_velocity_max: 1.5,
            velocity_spread: 0.8,
            gravity: Vec3::zeros(),
            drag: 2.0,
            size_start: 0.08,
            size_end: 0.02,
            color_gradient: trail_gradient,
            emissive_strength: 5.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.0,
            turbulence_frequency: 0.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(trail_entity, trail_emitter);

        self.game_world
            .set_entity_handle(game_entity, EntityHandle(engine_entity));
        self.game_world
            .set_position(game_entity, Position(spawn_pos));
        self.game_world
            .set_velocity(game_entity, Velocity(velocity));
        self.game_world.set_projectile(
            game_entity,
            Projectile {
                damage,
                speed: PROJECTILE_SPEED,
                particle_emitter: Some(trail_entity),
            },
        );

        self.game_world.resources.projectile_list.push(game_entity);

        self.spawn_muzzle_flash(world, spawn_pos, direction);
    }

    fn spawn_muzzle_flash(&self, world: &mut World, position: Vec3, direction: Vec3) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let flash_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.5, 0.9, 1.0, 1.0)),
                (0.3, Vec4::new(0.3, 0.7, 1.0, 0.8)),
                (0.6, Vec4::new(0.2, 0.5, 0.9, 0.4)),
                (1.0, Vec4::new(0.1, 0.3, 0.7, 0.0)),
            ],
        };

        let flash_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            position,
            direction,
            spawn_rate: 0.0,
            burst_count: 12,
            particle_lifetime_min: 0.05,
            particle_lifetime_max: 0.15,
            initial_velocity_min: 3.0,
            initial_velocity_max: 8.0,
            velocity_spread: 0.5,
            gravity: Vec3::zeros(),
            drag: 2.0,
            size_start: 0.12,
            size_end: 0.02,
            color_gradient: flash_gradient,
            emissive_strength: 20.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.3,
            turbulence_frequency: 5.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, flash_emitter);
    }

    fn projectile_movement_system(&mut self, world: &mut World, delta: f32) {
        let max_distance_from_player = 50.0;
        let mut to_remove = Vec::new();

        let projectiles: Vec<freecs::Entity> = self
            .game_world
            .query_entities(PROJECTILE | POSITION | VELOCITY | ENTITY_HANDLE)
            .collect();

        for game_entity in projectiles {
            let position = self.game_world.get_position(game_entity).unwrap();
            let velocity = self.game_world.get_velocity(game_entity).unwrap();
            let particle_emitter = self
                .game_world
                .get_projectile(game_entity)
                .unwrap()
                .particle_emitter;

            let new_position = Position(position.0 + velocity.0 * delta);

            let distance_from_player = nalgebra_glm::length(&Vec2::new(
                new_position.0.x - self.player_position.x,
                new_position.0.z - self.player_position.z,
            ));
            if distance_from_player > max_distance_from_player {
                to_remove.push((game_entity, particle_emitter));
                continue;
            }

            self.game_world.set_position(game_entity, new_position);

            let handle = self.game_world.get_entity_handle(game_entity).unwrap();
            if let Some(transform) = world.core.get_local_transform_mut(handle.0) {
                transform.translation = new_position.0;
            }
            mark_local_transform_dirty(world, handle.0);

            if let Some(emitter_entity) = particle_emitter
                && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
            {
                emitter.position = new_position.0;
            }
        }

        for (game_entity, particle_emitter) in to_remove {
            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            if let Some(emitter_entity) = particle_emitter {
                world.queue_command(WorldCommand::DespawnRecursive {
                    entity: emitter_entity,
                });
            }
            self.game_world.despawn_entities(&[game_entity]);
            self.game_world
                .resources
                .projectile_list
                .retain(|&e| e != game_entity);
        }
    }

    fn projectile_collision_system(&mut self, world: &mut World) {
        let mut projectiles_to_remove: Vec<(freecs::Entity, Option<Entity>)> = Vec::new();
        let mut enemies_to_remove = Vec::new();
        let mut gem_spawn_data: Vec<(Vec3, u32, EnemyType)> = Vec::new();
        let mut damage_popups: Vec<(Vec3, f32, bool)> = Vec::new();
        let mut hit_positions: Vec<Vec3> = Vec::new();
        let mut shield_break_positions: Vec<Vec3> = Vec::new();
        let mut crystals_to_damage: Vec<(freecs::Entity, f32)> = Vec::new();

        let projectiles: Vec<freecs::Entity> = self
            .game_world
            .query_entities(PROJECTILE | POSITION | ENTITY_HANDLE)
            .collect();
        let enemies: Vec<freecs::Entity> = self
            .game_world
            .query_entities(ENEMY | POSITION | ENTITY_HANDLE)
            .collect();
        let crystals: Vec<freecs::Entity> = self
            .game_world
            .query_entities(HEALTH_CRYSTAL | POSITION)
            .collect();

        for proj_entity in &projectiles {
            let proj_pos = *self.game_world.get_position(*proj_entity).unwrap();
            let projectile = *self.game_world.get_projectile(*proj_entity).unwrap();

            let mut hit_something = false;

            for crystal_entity in &crystals {
                let crystal_pos = *self.game_world.get_position(*crystal_entity).unwrap();
                let distance = nalgebra_glm::distance(&proj_pos.0, &crystal_pos.0);

                if distance < 1.5 {
                    projectiles_to_remove.push((*proj_entity, projectile.particle_emitter));
                    crystals_to_damage.push((*crystal_entity, projectile.damage));
                    hit_something = true;
                    break;
                }
            }

            if hit_something {
                continue;
            }

            for enemy_entity in &enemies {
                if enemies_to_remove.contains(enemy_entity) {
                    continue;
                }

                let enemy_pos = *self.game_world.get_position(*enemy_entity).unwrap();
                let enemy = *self.game_world.get_enemy(*enemy_entity).unwrap();
                let distance = nalgebra_glm::distance(&proj_pos.0, &enemy_pos.0);

                if distance < PROJECTILE_HIT_DISTANCE {
                    projectiles_to_remove.push((*proj_entity, projectile.particle_emitter));

                    if enemy.shield_hits > 0 {
                        shield_break_positions.push(enemy_pos.0);
                        self.game_world.set_enemy(
                            *enemy_entity,
                            Enemy {
                                shield_hits: enemy.shield_hits - 1,
                                ..enemy
                            },
                        );
                    } else {
                        hit_positions.push(enemy_pos.0);

                        let new_health = enemy.health - projectile.damage;
                        damage_popups.push((
                            enemy_pos.0,
                            projectile.damage,
                            enemy.enemy_type == EnemyType::Boss,
                        ));

                        if new_health <= 0.0 {
                            enemies_to_remove.push(*enemy_entity);
                            gem_spawn_data.push((enemy_pos.0, enemy.xp_value, enemy.enemy_type));
                            self.game_world.resources.enemies_killed += 1;
                        } else {
                            self.game_world.set_enemy(
                                *enemy_entity,
                                Enemy {
                                    health: new_health,
                                    ..enemy
                                },
                            );
                        }
                    }
                    break;
                }
            }
        }

        for (pos, damage, is_boss) in damage_popups {
            self.spawn_damage_popup(world, pos, damage, Vec4::new(1.0, 0.95, 0.4, 1.0), is_boss);
        }

        for pos in shield_break_positions {
            self.spawn_enemy_shield_break_effect(world, pos);
            self.spawn_popup_typed(
                world,
                pos + Vec3::new(0.0, 0.8, 0.0),
                "SHIELD!".to_string(),
                Vec4::new(1.0, 0.4, 0.3, 1.0),
                PopupType::Damage,
            );
        }

        for pos in hit_positions {
            self.spawn_hit_effect(world, pos);
        }

        for (game_entity, particle_emitter) in projectiles_to_remove {
            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            if let Some(emitter_entity) = particle_emitter {
                world.queue_command(WorldCommand::DespawnRecursive {
                    entity: emitter_entity,
                });
            }
            self.game_world.despawn_entities(&[game_entity]);
            self.game_world
                .resources
                .projectile_list
                .retain(|&e| e != game_entity);
        }

        for game_entity in enemies_to_remove {
            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            self.game_world.despawn_entities(&[game_entity]);
            self.game_world
                .resources
                .enemy_list
                .retain(|&e| e != game_entity);
        }

        for (pos, xp_value, enemy_type) in gem_spawn_data {
            let is_boss = enemy_type == EnemyType::Boss;
            if is_boss {
                self.game_world.resources.boss_alive = false;
                self.spawn_boss_death_effect(world, pos);
            }
            self.spawn_death_particles_for_type(world, pos, enemy_type);
            self.spawn_gem_with_xp(world, pos, xp_value);
            self.maybe_spawn_enemy_health_drop(world, pos, enemy_type);
            self.add_kill(world, is_boss);
        }

        for (crystal_entity, damage) in crystals_to_damage {
            self.damage_health_crystal(world, crystal_entity, damage);
        }
    }

    fn spawn_death_particles_for_type(
        &mut self,
        world: &mut World,
        position: Vec3,
        enemy_type: EnemyType,
    ) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let (color_start, color_end, burst_count) = match enemy_type {
            EnemyType::Normal => (
                Vec4::new(1.0, 0.3, 0.2, 1.0),
                Vec4::new(1.0, 0.9, 0.6, 0.0),
                60,
            ),
            EnemyType::Fast => (
                Vec4::new(1.0, 0.6, 0.1, 1.0),
                Vec4::new(1.0, 1.0, 0.5, 0.0),
                80,
            ),
            EnemyType::Tank => (
                Vec4::new(0.5, 0.2, 0.6, 1.0),
                Vec4::new(0.8, 0.5, 1.0, 0.0),
                120,
            ),
            EnemyType::Exploder => (
                Vec4::new(0.2, 1.0, 0.3, 1.0),
                Vec4::new(0.5, 1.0, 0.7, 0.0),
                150,
            ),
            EnemyType::Boss => (
                Vec4::new(1.0, 0.1, 0.1, 1.0),
                Vec4::new(1.0, 0.5, 0.3, 0.0),
                300,
            ),
        };

        let death_gradient = ColorGradient {
            colors: vec![
                (0.0, color_start),
                (
                    0.3,
                    Vec4::new(
                        (color_start.x + color_end.x) * 0.5,
                        (color_start.y + color_end.y) * 0.5,
                        (color_start.z + color_end.z) * 0.5,
                        0.8,
                    ),
                ),
                (1.0, color_end),
            ],
        };

        let death_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere {
                radius: ENEMY_RADIUS * 1.5,
            },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count,
            particle_lifetime_min: 0.4,
            particle_lifetime_max: 1.0,
            initial_velocity_min: 6.0,
            initial_velocity_max: 15.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -8.0, 0.0),
            drag: 0.2,
            size_start: 0.35,
            size_end: 0.05,
            color_gradient: death_gradient,
            emissive_strength: 15.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.8,
            turbulence_frequency: 3.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(particle_entity, death_emitter);
    }

    fn spawn_gem_with_xp(&mut self, world: &mut World, position: Vec3, xp_value: u32) {
        let spawn_pos = Vec3::new(position.x, GEM_RADIUS, position.z);

        let (mesh_type, scale_mult, base_color, emissive) = if xp_value >= 50 {
            ("Torus", 1.8, [1.0, 0.3, 0.9, 1.0], [0.6, 0.1, 0.5])
        } else if xp_value >= 15 {
            ("Sphere", 1.4, [0.3, 0.9, 1.0, 1.0], [0.1, 0.5, 0.6])
        } else if xp_value >= 10 {
            ("Cylinder", 1.2, [1.0, 0.9, 0.2, 1.0], [0.5, 0.4, 0.0])
        } else {
            ("Cube", 1.0, [0.3, 1.0, 0.4, 1.0], [0.1, 0.5, 0.2])
        };

        let scale = GEM_RADIUS * 2.0 * scale_mult;
        let engine_entity = spawn_mesh(world, mesh_type, spawn_pos, Vec3::new(scale, scale, scale));

        if mesh_type == "Cube" {
            if let Some(transform) = world.core.get_local_transform_mut(engine_entity) {
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_4, &Vec3::y());
            }
            mark_local_transform_dirty(world, engine_entity);
        }

        let mat = format!("Gem_{}", engine_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            mat.clone(),
            Material {
                base_color,
                roughness: 0.15,
                metallic: 0.7,
                emissive_factor: emissive,
                ..Default::default()
            },
        );
        self.apply_material(world, engine_entity, &mat);

        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
        let gem_gradient = ColorGradient {
            colors: vec![
                (
                    0.0,
                    Vec4::new(base_color[0], base_color[1], base_color[2], 0.0),
                ),
                (
                    0.2,
                    Vec4::new(base_color[0], base_color[1], base_color[2], 0.8),
                ),
                (
                    0.5,
                    Vec4::new(base_color[0], base_color[1], base_color[2], 0.6),
                ),
                (0.8, Vec4::new(1.0, 1.0, 1.0, 0.3)),
                (1.0, Vec4::new(1.0, 1.0, 1.0, 0.0)),
            ],
        };

        let gem_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere {
                radius: GEM_RADIUS * 0.5,
            },
            position: spawn_pos,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 8.0 + xp_value as f32 * 0.5,
            burst_count: 0,
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.0,
            initial_velocity_min: 0.5,
            initial_velocity_max: 1.5,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 0.3, 0.0),
            drag: 0.5,
            size_start: 0.06 + xp_value as f32 * 0.002,
            size_end: 0.02,
            color_gradient: gem_gradient,
            emissive_strength: 8.0 + xp_value as f32 * 0.2,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 2.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(particle_entity, gem_emitter);

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | POSITION | GEM, 1)[0];

        self.game_world
            .set_entity_handle(game_entity, EntityHandle(engine_entity));
        self.game_world
            .set_position(game_entity, Position(spawn_pos));
        self.game_world.set_gem(
            game_entity,
            Gem {
                xp_value,
                particle_emitter: Some(particle_entity),
            },
        );

        self.game_world.resources.gem_list.push(game_entity);
    }

    fn gem_system(&mut self, world: &mut World, delta: f32) {
        let player_pos = self.player_position;
        let mut gems_to_remove = Vec::new();
        let mut xp_popups: Vec<(Vec3, u32)> = Vec::new();
        let magnet_range = GEM_MAGNET_RANGE * self.stats.magnet_multiplier;

        let gems: Vec<freecs::Entity> = self
            .game_world
            .query_entities(GEM | POSITION | ENTITY_HANDLE)
            .collect();

        for game_entity in gems {
            let position = *self.game_world.get_position(game_entity).unwrap();
            let gem = *self.game_world.get_gem(game_entity).unwrap();
            let distance = nalgebra_glm::distance(&player_pos, &position.0);

            if distance < GEM_COLLECT_DISTANCE {
                xp_popups.push((position.0, gem.xp_value));
                gems_to_remove.push((game_entity, gem.particle_emitter));
            } else if distance < magnet_range {
                let direction = nalgebra_glm::normalize(&(player_pos - position.0));
                let new_position = Position(position.0 + direction * GEM_MAGNET_SPEED * delta);
                self.game_world.set_position(game_entity, new_position);

                let handle = self.game_world.get_entity_handle(game_entity).unwrap();
                if let Some(transform) = world.core.get_local_transform_mut(handle.0) {
                    transform.translation = new_position.0;
                }
                mark_local_transform_dirty(world, handle.0);

                if let Some(emitter_entity) = gem.particle_emitter
                    && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
                {
                    emitter.position = new_position.0;
                }
            } else {
                let handle = self.game_world.get_entity_handle(game_entity).unwrap();
                let phase = (game_entity.id as f32) * 1.7;
                let bob_offset = (self.game_time * 4.0 + phase).sin() * 0.15;
                let spin = self.game_time * 2.0 + phase;

                if let Some(transform) = world.core.get_local_transform_mut(handle.0) {
                    transform.translation.y = GEM_RADIUS + bob_offset;
                    transform.rotation = nalgebra_glm::quat_angle_axis(spin, &Vec3::y());
                }
                mark_local_transform_dirty(world, handle.0);

                if let Some(emitter_entity) = gem.particle_emitter
                    && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
                {
                    emitter.position =
                        Vec3::new(position.0.x, GEM_RADIUS + bob_offset, position.0.z);
                }
            }
        }

        for (game_entity, particle_emitter) in &gems_to_remove {
            if let Some(handle) = self.game_world.get_entity_handle(*game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            if let Some(emitter_entity) = particle_emitter {
                world.queue_command(WorldCommand::DespawnRecursive {
                    entity: *emitter_entity,
                });
            }
            self.game_world.despawn_entities(&[*game_entity]);
            self.game_world
                .resources
                .gem_list
                .retain(|&e| e != *game_entity);
        }

        let mut xp_gained = 0u32;
        for (pos, xp) in xp_popups {
            self.spawn_popup_typed(
                world,
                pos,
                format!("+{}", xp),
                Vec4::new(0.4, 1.0, 0.5, 1.0),
                PopupType::Xp,
            );
            xp_gained += xp;
        }

        if xp_gained > 0 {
            self.player_xp += xp_gained;
            let xp_for_next = XP_PER_LEVEL * self.player_level;
            if self.player_xp >= xp_for_next {
                self.player_xp -= xp_for_next;
                self.player_level += 1;
                self.generate_upgrade_choices();
                self.spawn_levelup_effect(world);
                self.spawn_popup_typed(
                    world,
                    self.player_position + Vec3::new(0.0, 4.0, 0.0),
                    format!("LEVEL {}!", self.player_level),
                    Vec4::new(1.0, 0.9, 0.2, 1.0),
                    PopupType::LevelUp,
                );
                self.level_up_flash = 1.0;
                self.camera_shake = 0.5;
                self.game_state = GameState::LevelUp;
            }
        }
    }

    fn health_crystal_spawn_system(&mut self, world: &mut World, delta: f32) {
        self.health_crystal_spawn_timer += delta;

        if self.health_crystal_spawn_timer >= self.health_crystal_spawn_interval {
            self.health_crystal_spawn_timer = 0.0;

            let mut rng = rand::rng();
            self.health_crystal_spawn_interval = rng.random_range(45.0..75.0);

            let spawn_distance = rng.random_range(15.0..25.0);
            let spawn_angle = rng.random_range(0.0..std::f32::consts::TAU);
            let spawn_position = Vec3::new(
                self.player_position.x + spawn_angle.cos() * spawn_distance,
                0.5,
                self.player_position.z + spawn_angle.sin() * spawn_distance,
            );

            self.spawn_health_crystal(world, spawn_position, 30.0, 3.0);
        }
    }

    fn spawn_health_crystal(
        &mut self,
        world: &mut World,
        position: Vec3,
        health_value: f32,
        hp: f32,
    ) {
        let engine_entity = spawn_mesh(world, "Cone", position, Vec3::new(0.6, 1.2, 0.6));

        if let Some(transform) = world.core.get_local_transform_mut(engine_entity) {
            transform.rotation = nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::x());
        }
        mark_local_transform_dirty(world, engine_entity);

        let mat = format!("HealthCrystal_{}", engine_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            mat.clone(),
            Material {
                base_color: [1.0, 0.3, 0.5, 1.0],
                roughness: 0.1,
                metallic: 0.8,
                emissive_factor: [0.8, 0.2, 0.3],
                ..Default::default()
            },
        );
        self.apply_material(world, engine_entity, &mat);

        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
        let crystal_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.3, 0.5, 0.0)),
                (0.2, Vec4::new(1.0, 0.5, 0.6, 0.7)),
                (0.6, Vec4::new(1.0, 0.6, 0.7, 0.5)),
                (1.0, Vec4::new(1.0, 0.8, 0.9, 0.0)),
            ],
        };

        let crystal_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.4 },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 15.0,
            burst_count: 0,
            particle_lifetime_min: 0.8,
            particle_lifetime_max: 1.5,
            initial_velocity_min: 0.3,
            initial_velocity_max: 0.8,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 0.2, 0.0),
            drag: 0.3,
            size_start: 0.08,
            size_end: 0.02,
            color_gradient: crystal_gradient,
            emissive_strength: 5.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 2.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(particle_entity, crystal_emitter);

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | POSITION | HEALTH_CRYSTAL, 1)[0];

        self.game_world
            .set_entity_handle(game_entity, EntityHandle(engine_entity));
        self.game_world
            .set_position(game_entity, Position(position));
        self.game_world.set_health_crystal(
            game_entity,
            HealthCrystal {
                health_value,
                current_hp: hp,
                particle_emitter: Some(particle_entity),
            },
        );

        self.game_world
            .resources
            .health_crystal_list
            .push(game_entity);
    }

    fn health_crystal_system(&mut self, world: &mut World) {
        let crystals: Vec<freecs::Entity> = self
            .game_world
            .query_entities(HEALTH_CRYSTAL | POSITION | ENTITY_HANDLE)
            .collect();

        for game_entity in crystals {
            let position = *self.game_world.get_position(game_entity).unwrap();
            let handle = self.game_world.get_entity_handle(game_entity).unwrap();

            let phase = (game_entity.id as f32) * 1.7;
            let bob_offset = (self.game_time * 3.0 + phase).sin() * 0.1;
            let pulse = (self.game_time * 5.0 + phase).sin() * 0.5 + 0.5;
            let spin = self.game_time * 1.5 + phase;

            if let Some(transform) = world.core.get_local_transform_mut(handle.0) {
                transform.translation.y = 0.5 + bob_offset;
                transform.rotation = nalgebra_glm::quat_angle_axis(spin, &Vec3::y())
                    * nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::x());
                let scale_pulse = 1.0 + pulse * 0.1;
                transform.scale =
                    Vec3::new(0.6 * scale_pulse, 1.2 * scale_pulse, 0.6 * scale_pulse);
            }
            mark_local_transform_dirty(world, handle.0);

            if let Some(crystal) = self.game_world.get_health_crystal(game_entity)
                && let Some(emitter_entity) = crystal.particle_emitter
                && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
            {
                emitter.position = Vec3::new(position.0.x, 0.5 + bob_offset, position.0.z);
            }
        }
    }

    fn spawn_health_gem(&mut self, world: &mut World, position: Vec3, health_value: f32) {
        let spawn_pos = Vec3::new(position.x, GEM_RADIUS, position.z);
        let scale = GEM_RADIUS * 2.5;

        let engine_entity = spawn_mesh(world, "Cube", spawn_pos, Vec3::new(scale, scale, scale));

        if let Some(transform) = world.core.get_local_transform_mut(engine_entity) {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_4, &Vec3::y())
                    * nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_4, &Vec3::x());
        }
        mark_local_transform_dirty(world, engine_entity);

        let mat = format!("HealthGem_{}", engine_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            mat.clone(),
            Material {
                base_color: [1.0, 0.4, 0.6, 1.0],
                roughness: 0.15,
                metallic: 0.7,
                emissive_factor: [0.6, 0.2, 0.3],
                ..Default::default()
            },
        );
        self.apply_material(world, engine_entity, &mat);

        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
        let health_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.4, 0.6, 0.0)),
                (0.2, Vec4::new(1.0, 0.5, 0.7, 0.8)),
                (0.5, Vec4::new(1.0, 0.6, 0.7, 0.6)),
                (0.8, Vec4::new(1.0, 0.8, 0.9, 0.3)),
                (1.0, Vec4::new(1.0, 0.9, 0.95, 0.0)),
            ],
        };

        let health_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere {
                radius: GEM_RADIUS * 0.5,
            },
            position: spawn_pos,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 10.0,
            burst_count: 0,
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.0,
            initial_velocity_min: 0.5,
            initial_velocity_max: 1.5,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 0.3, 0.0),
            drag: 0.5,
            size_start: 0.06,
            size_end: 0.02,
            color_gradient: health_gradient,
            emissive_strength: 6.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 2.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(particle_entity, health_emitter);

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | POSITION | HEALTH_GEM, 1)[0];

        self.game_world
            .set_entity_handle(game_entity, EntityHandle(engine_entity));
        self.game_world
            .set_position(game_entity, Position(spawn_pos));
        self.game_world.set_health_gem(
            game_entity,
            HealthGem {
                health_value,
                particle_emitter: Some(particle_entity),
            },
        );

        self.game_world.resources.health_gem_list.push(game_entity);
    }

    fn health_gem_system(&mut self, world: &mut World, delta: f32) {
        let player_pos = self.player_position;
        let mut gems_to_remove = Vec::new();
        let mut health_popups: Vec<(Vec3, f32)> = Vec::new();
        let magnet_range = GEM_MAGNET_RANGE * self.stats.magnet_multiplier;

        let gems: Vec<freecs::Entity> = self
            .game_world
            .query_entities(HEALTH_GEM | POSITION | ENTITY_HANDLE)
            .collect();

        for game_entity in gems {
            let position = *self.game_world.get_position(game_entity).unwrap();
            let health_gem = *self.game_world.get_health_gem(game_entity).unwrap();
            let distance = nalgebra_glm::distance(&player_pos, &position.0);

            if distance < GEM_COLLECT_DISTANCE {
                health_popups.push((position.0, health_gem.health_value));
                gems_to_remove.push((game_entity, health_gem.particle_emitter));
            } else if distance < magnet_range {
                let direction = nalgebra_glm::normalize(&(player_pos - position.0));
                let new_position = Position(position.0 + direction * GEM_MAGNET_SPEED * delta);
                self.game_world.set_position(game_entity, new_position);

                let handle = self.game_world.get_entity_handle(game_entity).unwrap();
                if let Some(transform) = world.core.get_local_transform_mut(handle.0) {
                    transform.translation = new_position.0;
                }
                mark_local_transform_dirty(world, handle.0);

                if let Some(emitter_entity) = health_gem.particle_emitter
                    && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
                {
                    emitter.position = new_position.0;
                }
            } else {
                let handle = self.game_world.get_entity_handle(game_entity).unwrap();
                let phase = (game_entity.id as f32) * 1.7;
                let bob_offset = (self.game_time * 4.0 + phase).sin() * 0.15;
                let spin = self.game_time * 2.0 + phase;

                if let Some(transform) = world.core.get_local_transform_mut(handle.0) {
                    transform.translation.y = GEM_RADIUS + bob_offset;
                    let rotation = nalgebra_glm::quat_angle_axis(spin, &Vec3::y())
                        * nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_4, &Vec3::x());
                    transform.rotation = rotation;
                }
                mark_local_transform_dirty(world, handle.0);

                if let Some(emitter_entity) = health_gem.particle_emitter
                    && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
                {
                    emitter.position =
                        Vec3::new(position.0.x, GEM_RADIUS + bob_offset, position.0.z);
                }
            }
        }

        for (game_entity, particle_emitter) in &gems_to_remove {
            if let Some(handle) = self.game_world.get_entity_handle(*game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            if let Some(emitter_entity) = particle_emitter {
                world.queue_command(WorldCommand::DespawnRecursive {
                    entity: *emitter_entity,
                });
            }
            self.game_world.despawn_entities(&[*game_entity]);
            self.game_world
                .resources
                .health_gem_list
                .retain(|&e| e != *game_entity);
        }

        for (pos, health) in health_popups {
            let heal_amount = health.min(self.stats.max_health - self.player_health);
            if heal_amount > 0.0 {
                self.player_health = (self.player_health + heal_amount).min(self.stats.max_health);
                self.spawn_popup_typed(
                    world,
                    pos,
                    format!("+{:.0}", heal_amount),
                    Vec4::new(1.0, 0.4, 0.6, 1.0),
                    PopupType::Xp,
                );
                self.spawn_heal_effect(world, self.player_position);
            }
        }
    }

    fn spawn_heal_effect(&self, world: &mut World, position: Vec3) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let heal_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.5, 0.7, 1.0)),
                (0.3, Vec4::new(1.0, 0.6, 0.8, 0.8)),
                (0.6, Vec4::new(1.0, 0.8, 0.9, 0.5)),
                (1.0, Vec4::new(1.0, 1.0, 1.0, 0.0)),
            ],
        };

        let heal_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.8 },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 30,
            particle_lifetime_min: 0.6,
            particle_lifetime_max: 1.2,
            initial_velocity_min: 1.0,
            initial_velocity_max: 3.0,
            velocity_spread: 0.8,
            gravity: Vec3::new(0.0, 2.0, 0.0),
            drag: 0.3,
            size_start: 0.1,
            size_end: 0.02,
            color_gradient: heal_gradient,
            emissive_strength: 6.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 2.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(particle_entity, heal_emitter);
    }

    fn damage_health_crystal(
        &mut self,
        world: &mut World,
        game_entity: freecs::Entity,
        damage: f32,
    ) {
        let crystal_info =
            if let Some(crystal) = self.game_world.get_health_crystal_mut(game_entity) {
                crystal.current_hp -= damage;
                Some((
                    crystal.current_hp,
                    crystal.health_value,
                    crystal.particle_emitter,
                ))
            } else {
                None
            };

        if let Some((current_hp, health_value, particle_emitter)) = crystal_info {
            if current_hp <= 0.0 {
                let position = self.game_world.get_position(game_entity).unwrap().0;

                if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                    world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
                }
                if let Some(emitter_entity) = particle_emitter {
                    world.queue_command(WorldCommand::DespawnRecursive {
                        entity: emitter_entity,
                    });
                }

                self.game_world.despawn_entities(&[game_entity]);
                self.game_world
                    .resources
                    .health_crystal_list
                    .retain(|&e| e != game_entity);

                self.spawn_health_gem(world, position, health_value);
                self.spawn_crystal_break_effect(world, position);
            } else if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                let mat = format!("HealthCrystal_hit_{}", handle.0.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    mat.clone(),
                    Material {
                        base_color: [1.0, 1.0, 1.0, 1.0],
                        roughness: 0.1,
                        metallic: 0.8,
                        emissive_factor: [1.0, 0.8, 0.9],
                        ..Default::default()
                    },
                );
                self.apply_material(world, handle.0, &mat);
            }
        }
    }

    fn spawn_crystal_break_effect(&self, world: &mut World, position: Vec3) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let break_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.3, 0.5, 1.0)),
                (0.3, Vec4::new(1.0, 0.5, 0.7, 0.9)),
                (0.6, Vec4::new(1.0, 0.7, 0.8, 0.5)),
                (1.0, Vec4::new(1.0, 0.9, 0.95, 0.0)),
            ],
        };

        let break_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.3 },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 40,
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.0,
            initial_velocity_min: 3.0,
            initial_velocity_max: 8.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -5.0, 0.0),
            drag: 0.2,
            size_start: 0.15,
            size_end: 0.03,
            color_gradient: break_gradient,
            emissive_strength: 8.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 2.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(particle_entity, break_emitter);
    }

    fn maybe_spawn_enemy_health_drop(
        &mut self,
        world: &mut World,
        position: Vec3,
        enemy_type: EnemyType,
    ) {
        let mut rng = rand::rng();
        let (drop_chance, health_value) = match enemy_type {
            EnemyType::Normal => (0.05, 10.0),
            EnemyType::Fast => (0.05, 10.0),
            EnemyType::Tank => (0.15, 20.0),
            EnemyType::Exploder => (0.08, 10.0),
            EnemyType::Boss => (1.0, 50.0),
        };

        if rng.random::<f32>() < drop_chance {
            self.spawn_health_gem(world, position, health_value);
        }
    }

    fn try_spawn_treasure_zone(&mut self, world: &mut World) {
        if self.max_distance_traveled < self.next_zone_distance {
            return;
        }

        let mut rng = rand::rng();

        let spawn_angle = rng.random_range(0.0..std::f32::consts::TAU);
        let spawn_offset = rng.random_range(30.0..50.0);
        let zone_center = Vec3::new(
            self.player_position.x + spawn_angle.cos() * spawn_offset,
            0.0,
            self.player_position.z + spawn_angle.sin() * spawn_offset,
        );

        let min_distance_from_existing = 40.0;
        for existing_zone in &self.treasure_zones {
            let distance = nalgebra_glm::length(&(zone_center - existing_zone.center));
            if distance < min_distance_from_existing {
                self.next_zone_distance += rng.random_range(10.0..20.0);
                return;
            }
        }

        let zone_types = [
            ZoneType::MaxHealth,
            ZoneType::Damage,
            ZoneType::Berserk,
            ZoneType::Haste,
            ZoneType::Invincible,
            ZoneType::HealthCache,
            ZoneType::BombCache,
        ];
        let zone_type = zone_types[rng.random_range(0..zone_types.len())];

        self.spawn_treasure_zone(world, zone_center, zone_type);

        self.next_zone_distance += rng.random_range(50.0..70.0);
    }

    fn spawn_treasure_zone(&mut self, world: &mut World, center: Vec3, zone_type: ZoneType) {
        let zone_radius = 10.0;
        let fence_post_count = 16;
        let gap_index = 0;

        let mut fence_entities = Vec::new();

        for index in 0..fence_post_count {
            if index == gap_index {
                continue;
            }

            let angle = (index as f32 / fence_post_count as f32) * std::f32::consts::TAU;
            let post_x = center.x + angle.cos() * zone_radius;
            let post_z = center.z + angle.sin() * zone_radius;
            let post_position = Vec3::new(post_x, 1.0, post_z);

            let post_entity =
                spawn_mesh(world, "Cylinder", post_position, Vec3::new(0.5, 2.0, 0.5));

            let mat_name = format!("FencePost_{}", post_entity.id);
            let (r, g, b) = match zone_type {
                ZoneType::MaxHealth => (1.0, 0.3, 0.3),
                ZoneType::Damage => (1.0, 0.5, 0.0),
                ZoneType::Berserk => (0.8, 0.0, 0.0),
                ZoneType::Haste => (0.0, 0.8, 1.0),
                ZoneType::Invincible => (1.0, 1.0, 0.0),
                ZoneType::HealthCache => (1.0, 0.4, 0.6),
                ZoneType::BombCache => (0.5, 0.0, 0.8),
            };

            material_registry_insert(
                &mut world.resources.material_registry,
                mat_name.clone(),
                Material {
                    base_color: [r, g, b, 1.0],
                    roughness: 0.2,
                    metallic: 0.6,
                    emissive_factor: [r * 2.0, g * 2.0, b * 2.0],
                    ..Default::default()
                },
            );
            self.apply_material(world, post_entity, &mat_name);

            fence_entities.push(post_entity);
        }

        let power_up_position = Vec3::new(center.x, 1.5, center.z);
        let power_up_entity =
            spawn_mesh(world, "Cube", power_up_position, Vec3::new(0.8, 0.8, 0.8));

        if let Some(transform) = world.core.get_local_transform_mut(power_up_entity) {
            transform.rotation = nalgebra_glm::quat_angle_axis(
                std::f32::consts::FRAC_PI_4,
                &Vec3::new(1.0, 0.0, 1.0).normalize(),
            );
        }
        mark_local_transform_dirty(world, power_up_entity);

        let (r, g, b) = match zone_type {
            ZoneType::MaxHealth => (1.0, 0.3, 0.3),
            ZoneType::Damage => (1.0, 0.5, 0.0),
            ZoneType::Berserk => (0.8, 0.0, 0.0),
            ZoneType::Haste => (0.0, 0.8, 1.0),
            ZoneType::Invincible => (1.0, 1.0, 0.0),
            ZoneType::HealthCache => (1.0, 0.4, 0.6),
            ZoneType::BombCache => (0.5, 0.0, 0.8),
        };

        let powerup_mat = format!("PowerUp_{}", power_up_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            powerup_mat.clone(),
            Material {
                base_color: [r, g, b, 1.0],
                roughness: 0.1,
                metallic: 0.9,
                emissive_factor: [r * 0.8, g * 0.8, b * 0.8],
                ..Default::default()
            },
        );
        self.apply_material(world, power_up_entity, &powerup_mat);

        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
        let powerup_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(r, g, b, 0.0)),
                (0.2, Vec4::new(r, g, b, 0.6)),
                (0.6, Vec4::new(r, g, b, 0.4)),
                (1.0, Vec4::new(r, g, b, 0.0)),
            ],
        };

        let powerup_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.5 },
            position: power_up_position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 20.0,
            burst_count: 0,
            particle_lifetime_min: 1.0,
            particle_lifetime_max: 2.0,
            initial_velocity_min: 0.3,
            initial_velocity_max: 0.6,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 0.3, 0.0),
            drag: 0.2,
            size_start: 0.1,
            size_end: 0.02,
            color_gradient: powerup_gradient,
            emissive_strength: 8.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.3,
            turbulence_frequency: 1.5,

            ..Default::default()
        };
        world.core.set_particle_emitter(particle_entity, powerup_emitter);

        let zone = TreasureZone {
            center,
            radius: zone_radius,
            fence_entities,
            power_up_entity: Some(power_up_entity),
            power_up_emitter: Some(particle_entity),
            zone_type,
            cleared: false,
            activated: false,
            zone_enemies: Vec::new(),
        };

        self.treasure_zones.push(zone);
    }

    fn treasure_zone_system(&mut self, world: &mut World, _delta: f32) {
        let mut zones_to_collect = Vec::new();

        for zone_index in 0..self.treasure_zones.len() {
            let (center, radius, activated, power_up_entity) = {
                let zone = &self.treasure_zones[zone_index];
                (
                    zone.center,
                    zone.radius,
                    zone.activated,
                    zone.power_up_entity,
                )
            };

            let distance_to_player = nalgebra_glm::length(&(self.player_position - center));
            let in_zone = distance_to_player < radius;

            if !activated && in_zone {
                self.treasure_zones[zone_index].activated = true;

                let enemy_count = 5;
                for _ in 0..enemy_count {
                    let elite = self.spawn_elite_enemy(world, center);
                    self.treasure_zones[zone_index].zone_enemies.push(elite);
                }
            }

            if self.treasure_zones[zone_index].activated && !self.treasure_zones[zone_index].cleared
            {
                let alive_enemies: Vec<freecs::Entity> = self.treasure_zones[zone_index]
                    .zone_enemies
                    .iter()
                    .filter(|e| self.game_world.get_enemy(**e).is_some())
                    .copied()
                    .collect();

                self.treasure_zones[zone_index].zone_enemies = alive_enemies.clone();

                if alive_enemies.is_empty() {
                    self.treasure_zones[zone_index].cleared = true;
                }
            }

            let zone = &self.treasure_zones[zone_index];
            if zone.cleared
                && let Some(pue) = power_up_entity
            {
                if let Some(transform) = world.core.get_local_transform_mut(pue) {
                    transform.rotation =
                        nalgebra_glm::quat_angle_axis(self.game_time * 2.0, &Vec3::y())
                            * nalgebra_glm::quat_angle_axis(
                                std::f32::consts::FRAC_PI_4,
                                &Vec3::new(1.0, 0.0, 1.0).normalize(),
                            );
                    transform.translation.y = 1.0 + (self.game_time * 3.0).sin() * 0.3;
                }
                mark_local_transform_dirty(world, pue);

                let distance_to_powerup = nalgebra_glm::length(&Vec2::new(
                    self.player_position.x - center.x,
                    self.player_position.z - center.z,
                ));

                if distance_to_powerup < 2.0 {
                    zones_to_collect.push(zone_index);
                }
            }

            if !zone.cleared
                && let Some(pue) = power_up_entity
            {
                if let Some(transform) = world.core.get_local_transform_mut(pue) {
                    transform.rotation =
                        nalgebra_glm::quat_angle_axis(self.game_time * 0.5, &Vec3::y())
                            * nalgebra_glm::quat_angle_axis(
                                std::f32::consts::FRAC_PI_4,
                                &Vec3::new(1.0, 0.0, 1.0).normalize(),
                            );
                }
                mark_local_transform_dirty(world, pue);
            }
        }

        for zone_index in zones_to_collect {
            self.collect_power_up(world, zone_index);
        }
    }

    fn spawn_elite_enemy(&mut self, world: &mut World, zone_center: Vec3) -> freecs::Entity {
        let mut rng = rand::rng();

        let enemy_type = EnemyType::Tank;

        let (base_speed, base_health, radius_mult, xp_value) = match enemy_type {
            EnemyType::Normal => (ENEMY_SPEED, 1.0, 1.0, 5u32),
            EnemyType::Fast => (ENEMY_SPEED * 1.8, 0.5, 0.7, 7),
            EnemyType::Tank => (ENEMY_SPEED * 0.5, 4.0, 1.5, 15),
            EnemyType::Exploder => (ENEMY_SPEED * 1.2, 1.5, 0.9, 10),
            EnemyType::Boss => (BOSS_SPEED, BOSS_HEALTH, BOSS_RADIUS / ENEMY_RADIUS, BOSS_XP),
        };

        let radius = ENEMY_RADIUS * radius_mult;

        let spawn_angle = rng.random_range(0.0..std::f32::consts::TAU);
        let spawn_distance = rng.random_range(3.0..8.0);
        let spawn_position = Vec3::new(
            zone_center.x + spawn_angle.cos() * spawn_distance,
            radius,
            zone_center.z + spawn_angle.sin() * spawn_distance,
        );

        let engine_entity = self.spawn_enemy_mesh(world, spawn_position, radius, enemy_type);

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | POSITION | VELOCITY | ENEMY, 1)[0];

        let speed_multiplier = 1.0 + (self.player_level as f32 - 1.0) * 0.08;
        let enemy_speed = base_speed * speed_multiplier * 1.1 * rng.random_range(0.9..1.1);
        let health_multiplier = 1.0 + (self.player_level as f32 - 1.0) * 0.1;
        let enemy_health = base_health * health_multiplier * 1.5;

        let shield_hits = 2;

        self.game_world
            .set_entity_handle(game_entity, EntityHandle(engine_entity));
        self.game_world
            .set_position(game_entity, Position(spawn_position));
        self.game_world
            .set_velocity(game_entity, Velocity(Vec3::zeros()));
        self.game_world.set_enemy(
            game_entity,
            Enemy {
                speed: enemy_speed,
                health: enemy_health,
                enemy_type,
                xp_value,
                shield_hits,
            },
        );

        self.game_world.resources.enemy_list.push(game_entity);

        if shield_hits > 0 {
            self.spawn_enemy_shield(world, game_entity, engine_entity, radius);
        }

        self.spawn_enemy_spawn_effect(world, spawn_position, enemy_type);

        game_entity
    }

    fn collect_power_up(&mut self, world: &mut World, zone_index: usize) {
        let zone = &self.treasure_zones[zone_index];
        let zone_type = zone.zone_type;
        let center = zone.center;

        if let Some(power_up_entity) = zone.power_up_entity {
            world.queue_command(WorldCommand::DespawnRecursive {
                entity: power_up_entity,
            });
        }

        if let Some(emitter) = zone.power_up_emitter {
            world.queue_command(WorldCommand::DespawnRecursive { entity: emitter });
        }

        self.treasure_zones[zone_index].power_up_entity = None;
        self.treasure_zones[zone_index].power_up_emitter = None;

        match zone_type {
            ZoneType::MaxHealth => {
                self.stats.max_health += 25.0;
                self.player_health = (self.player_health + 25.0).min(self.stats.max_health);
                self.spawn_popup_typed(
                    world,
                    center + Vec3::new(0.0, 2.0, 0.0),
                    "+25 MAX HP".to_string(),
                    Vec4::new(1.0, 0.3, 0.3, 1.0),
                    PopupType::PowerUp,
                );
            }
            ZoneType::Damage => {
                self.stats.damage_multiplier += 0.1;
                self.spawn_popup_typed(
                    world,
                    center + Vec3::new(0.0, 2.0, 0.0),
                    "+10% DAMAGE".to_string(),
                    Vec4::new(1.0, 0.5, 0.0, 1.0),
                    PopupType::PowerUp,
                );
            }
            ZoneType::Berserk => {
                self.active_buffs.push(ActiveBuff {
                    buff_type: BuffType::Berserk,
                    remaining_time: 30.0,
                });
                self.spawn_popup_typed(
                    world,
                    center + Vec3::new(0.0, 2.0, 0.0),
                    "BERSERK!".to_string(),
                    Vec4::new(0.8, 0.0, 0.0, 1.0),
                    PopupType::PowerUp,
                );
            }
            ZoneType::Haste => {
                self.active_buffs.push(ActiveBuff {
                    buff_type: BuffType::Haste,
                    remaining_time: 30.0,
                });
                self.spawn_popup_typed(
                    world,
                    center + Vec3::new(0.0, 2.0, 0.0),
                    "HASTE!".to_string(),
                    Vec4::new(0.0, 0.8, 1.0, 1.0),
                    PopupType::PowerUp,
                );
            }
            ZoneType::Invincible => {
                self.active_buffs.push(ActiveBuff {
                    buff_type: BuffType::Invincible,
                    remaining_time: 10.0,
                });
                self.invincibility_timer = 10.0;
                self.spawn_popup_typed(
                    world,
                    center + Vec3::new(0.0, 2.0, 0.0),
                    "INVINCIBLE!".to_string(),
                    Vec4::new(1.0, 1.0, 0.0, 1.0),
                    PopupType::PowerUp,
                );
            }
            ZoneType::HealthCache => {
                let heal_amount = 100.0;
                self.player_health = (self.player_health + heal_amount).min(self.stats.max_health);
                self.spawn_heal_effect(world, self.player_position);
                self.spawn_popup_typed(
                    world,
                    center + Vec3::new(0.0, 2.0, 0.0),
                    "+100 HP".to_string(),
                    Vec4::new(1.0, 0.4, 0.6, 1.0),
                    PopupType::PowerUp,
                );
            }
            ZoneType::BombCache => {
                self.bomb_cooldown = 0.0;
                self.spawn_popup_typed(
                    world,
                    center + Vec3::new(0.0, 2.0, 0.0),
                    "BOMB READY!".to_string(),
                    Vec4::new(0.5, 0.0, 0.8, 1.0),
                    PopupType::PowerUp,
                );
            }
        }

        self.spawn_power_up_collect_effect(world, center);
    }

    fn spawn_power_up_collect_effect(&mut self, world: &mut World, position: Vec3) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let collect_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.3, Vec4::new(1.0, 0.9, 0.5, 0.8)),
                (0.7, Vec4::new(1.0, 0.7, 0.3, 0.4)),
                (1.0, Vec4::new(1.0, 0.5, 0.2, 0.0)),
            ],
        };

        let collect_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.8 },
            position: Vec3::new(position.x, 1.0, position.z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 60,
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.2,
            initial_velocity_min: 3.0,
            initial_velocity_max: 8.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 2.0, 0.0),
            drag: 0.3,
            size_start: 0.2,
            size_end: 0.02,
            color_gradient: collect_gradient,
            emissive_strength: 15.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.4,
            turbulence_frequency: 2.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, collect_emitter);
    }

    fn update_active_buffs(&mut self, delta: f32) {
        let mut damage_multiplier_buff = 1.0;
        let mut speed_multiplier_buff = 1.0;

        self.active_buffs.retain_mut(|buff| {
            buff.remaining_time -= delta;
            if buff.remaining_time <= 0.0 {
                false
            } else {
                match buff.buff_type {
                    BuffType::Berserk => {
                        damage_multiplier_buff = 2.0;
                    }
                    BuffType::Haste => {
                        speed_multiplier_buff = 2.0;
                    }
                    BuffType::Invincible => {}
                }
                true
            }
        });

        self.stats.buff_damage_multiplier = damage_multiplier_buff;
        self.stats.buff_speed_multiplier = speed_multiplier_buff;
    }

    fn spawn_levelup_effect(&self, world: &mut World) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let levelup_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.9, 0.3, 1.0)),
                (0.2, Vec4::new(1.0, 0.8, 0.4, 0.9)),
                (0.5, Vec4::new(1.0, 0.7, 0.5, 0.6)),
                (0.8, Vec4::new(1.0, 0.6, 0.3, 0.3)),
                (1.0, Vec4::new(1.0, 0.5, 0.2, 0.0)),
            ],
        };

        let levelup_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere {
                radius: PLAYER_RADIUS,
            },
            position: self.player_position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 50,
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.2,
            initial_velocity_min: 4.0,
            initial_velocity_max: 8.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -3.0, 0.0),
            drag: 0.3,
            size_start: 0.15,
            size_end: 0.05,
            color_gradient: levelup_gradient,
            emissive_strength: 3.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 2.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, levelup_emitter);
    }

    fn spawn_hit_effect(&self, world: &mut World, position: Vec3) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let hit_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 0.8, 1.0)),
                (0.2, Vec4::new(1.0, 0.8, 0.3, 0.95)),
                (0.5, Vec4::new(1.0, 0.5, 0.1, 0.7)),
                (0.8, Vec4::new(0.9, 0.3, 0.1, 0.4)),
                (1.0, Vec4::new(0.6, 0.2, 0.0, 0.0)),
            ],
        };

        let hit_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            position: position + Vec3::new(0.0, 0.3, 0.0),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 35,
            particle_lifetime_min: 0.2,
            particle_lifetime_max: 0.45,
            initial_velocity_min: 5.0,
            initial_velocity_max: 12.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -8.0, 0.0),
            drag: 0.4,
            size_start: 0.25,
            size_end: 0.04,
            color_gradient: hit_gradient,
            emissive_strength: 18.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 4.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, hit_emitter);
    }

    fn generate_upgrade_choices(&mut self) {
        let all_upgrades = [
            UpgradeType::Damage,
            UpgradeType::FireRate,
            UpgradeType::ProjectileCount,
            UpgradeType::Range,
            UpgradeType::Speed,
            UpgradeType::MaxHealth,
            UpgradeType::OrbitingOrbs,
            UpgradeType::AreaPulse,
            UpgradeType::Magnetism,
            UpgradeType::Regeneration,
            UpgradeType::Whip,
            UpgradeType::Lightning,
            UpgradeType::Garlic,
            UpgradeType::Bomb,
            UpgradeType::Shield,
        ];

        let mut rng = rand::rng();
        let mut choices = Vec::new();
        let mut available: Vec<_> = all_upgrades
            .iter()
            .copied()
            .filter(|upgrade| !self.stats.is_maxed(*upgrade))
            .collect();

        for _ in 0..3.min(available.len()) {
            if available.is_empty() {
                break;
            }
            let index = rng.random_range(0..available.len());
            choices.push(available.remove(index));
        }

        self.upgrade_choices = choices;
        self.selected_upgrade_index = 0;
    }

    fn spawn_damage_popup(
        &mut self,
        world: &mut World,
        position: Vec3,
        damage: f32,
        color: Vec4,
        is_boss: bool,
    ) {
        let base_size = if is_boss { 44.0 } else { 28.0 };
        let damage_scale = if damage >= 100.0 {
            2.0
        } else if damage >= 50.0 {
            1.6
        } else if damage >= 25.0 {
            1.3
        } else {
            1.0
        };
        let font_size = base_size * damage_scale;

        let popup_type = if is_boss {
            PopupType::BossDamage
        } else {
            PopupType::Damage
        };
        let text = format!("{}", damage as i32);

        let text_position = position + Vec3::new(0.0, 1.0, 0.0);

        let text_entity = spawn_3d_billboard_text_with_properties(
            world,
            &text,
            text_position,
            TextProperties {
                font_size,
                color,
                alignment: TextAlignment::Center,
                outline_width: 0.08,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let game_entity = self.game_world.spawn_entities(POPUP, 1)[0];
        self.game_world.set_popup(
            game_entity,
            Popup {
                text_entity,
                lifetime: 0.0,
                popup_type,
                start_scale: 1.0,
                max_scale: 1.0,
                base_position: text_position,
                velocity: Vec3::zeros(),
            },
        );

        self.game_world.resources.popup_list.push(game_entity);
    }

    fn spawn_popup_typed(
        &mut self,
        world: &mut World,
        position: Vec3,
        text: String,
        color: Vec4,
        popup_type: PopupType,
    ) {
        let font_size = match popup_type {
            PopupType::Damage => 32.0,
            PopupType::CriticalDamage => 40.0,
            PopupType::Xp => 28.0,
            PopupType::Combo => 48.0,
            PopupType::LevelUp => 64.0,
            PopupType::Wave => 72.0,
            PopupType::BossDamage => 44.0,
            PopupType::PowerUp => 48.0,
        };

        let text_position = position + Vec3::new(0.0, 1.0, 0.0);

        let text_entity = spawn_3d_billboard_text_with_properties(
            world,
            &text,
            text_position,
            TextProperties {
                font_size,
                color,
                alignment: TextAlignment::Center,
                outline_width: 0.08,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let game_entity = self.game_world.spawn_entities(POPUP, 1)[0];
        self.game_world.set_popup(
            game_entity,
            Popup {
                text_entity,
                lifetime: 0.0,
                popup_type,
                start_scale: 1.0,
                max_scale: 1.0,
                base_position: text_position,
                velocity: Vec3::zeros(),
            },
        );

        self.game_world.resources.popup_list.push(game_entity);
    }

    fn update_popups(&mut self, world: &mut World, delta: f32) {
        let entities: Vec<freecs::Entity> = self.game_world.query_entities(POPUP).collect();
        let mut popups_to_remove = Vec::new();

        for entity in entities {
            if let Some(mut popup) = self.game_world.get_popup(entity).copied() {
                popup.lifetime += delta;

                let max_lifetime = 1.2;

                if popup.lifetime > max_lifetime {
                    popups_to_remove.push((entity, popup.text_entity));
                    continue;
                }

                popup.base_position.y += delta * 1.5;

                self.game_world.set_popup(entity, popup);

                if let Some(transform) = world.core.get_local_transform_mut(popup.text_entity) {
                    transform.translation = popup.base_position;
                    world.core.set_local_transform_dirty(popup.text_entity, LocalTransformDirty);
                }

                let alpha = (1.0 - (popup.lifetime / max_lifetime)).max(0.0);

                if let Some(text_component) = world.core.get_text_mut(popup.text_entity) {
                    text_component.properties.color.w = alpha;
                    text_component.dirty = true;
                }
            }
        }

        for (entity, text_entity) in popups_to_remove {
            if world.core.get_text(text_entity).is_some() {
                world.queue_command(WorldCommand::DespawnRecursive {
                    entity: text_entity,
                });
            }
            self.game_world.despawn_entities(&[entity]);
            self.game_world
                .resources
                .popup_list
                .retain(|&e| e != entity);
        }
    }

    fn apply_upgrade(&mut self, upgrade: UpgradeType, world: &mut World) {
        match upgrade {
            UpgradeType::Damage => {
                self.stats.damage_multiplier *= 1.25;
                self.stats.damage_level += 1;
            }
            UpgradeType::FireRate => {
                self.stats.cooldown_multiplier *= 0.8;
                self.stats.fire_rate_level += 1;
            }
            UpgradeType::ProjectileCount => {
                self.stats.projectile_count += 1;
                self.stats.projectile_level += 1;
            }
            UpgradeType::Range => {
                self.stats.range_multiplier *= 1.25;
                self.stats.range_level += 1;
            }
            UpgradeType::Speed => {
                self.stats.speed_multiplier *= 1.15;
                self.stats.speed_level += 1;
            }
            UpgradeType::MaxHealth => {
                self.stats.max_health += 25.0;
                self.player_health += 25.0;
                self.stats.health_level += 1;
            }
            UpgradeType::OrbitingOrbs => {
                self.stats.orb_count += 2;
            }
            UpgradeType::AreaPulse => {
                self.stats.area_pulse_level += 1;
            }
            UpgradeType::Magnetism => {
                self.stats.magnet_multiplier *= 1.5;
                self.stats.magnetism_level += 1;
            }
            UpgradeType::Regeneration => {
                self.stats.regen_level += 1;
            }
            UpgradeType::Whip => {
                self.stats.whip_level += 1;
            }
            UpgradeType::Lightning => {
                self.stats.lightning_level += 1;
            }
            UpgradeType::Garlic => {
                self.stats.garlic_level += 1;
            }
            UpgradeType::Bomb => {
                self.stats.bomb_level += 1;
            }
            UpgradeType::Shield => {
                self.stats.shield_level += 1;
                self.spawn_shield_layer(world, self.stats.shield_level - 1);
            }
        }
    }

    fn get_shield_layer_color(layer_index: u32) -> Vec4 {
        match layer_index {
            0 => Vec4::new(0.4, 0.9, 1.0, 1.0),
            1 => Vec4::new(0.3, 0.5, 1.0, 1.0),
            2 => Vec4::new(0.7, 0.3, 1.0, 1.0),
            3 => Vec4::new(1.0, 0.3, 0.8, 1.0),
            _ => Vec4::new(1.0, 0.85, 0.3, 1.0),
        }
    }

    fn get_shield_layer_duration(layer_index: u32) -> f32 {
        SHIELD_BASE_DURATION + layer_index as f32 * SHIELD_DURATION_PER_LAYER
    }

    fn get_shield_layer_radius(layer_index: u32) -> f32 {
        PLAYER_RADIUS * (SHIELD_RADIUS_BASE + layer_index as f32 * SHIELD_RADIUS_STEP)
    }

    fn spawn_shield_layer(&mut self, world: &mut World, layer_index: u32) {
        let radius = Self::get_shield_layer_radius(layer_index);
        let duration = Self::get_shield_layer_duration(layer_index);
        let color = Self::get_shield_layer_color(layer_index);

        let shield_entity = spawn_mesh(
            world,
            "Sphere",
            Vec3::zeros(),
            Vec3::new(radius * 2.0, radius * 2.0, radius * 2.0),
        );

        if let Some(player_entity) = self.player_entity {
            world.core.set_parent(shield_entity, Parent(Some(player_entity)));
        }

        let shield_material_name = format!("ShieldLayer_{}_{}", layer_index, shield_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            shield_material_name.clone(),
            Material {
                base_color: [color.x, color.y, color.z, 0.35],
                roughness: 0.1,
                metallic: 0.0,
                emissive_factor: [color.x * 0.4, color.y * 0.4, color.z * 0.4],
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        );
        self.apply_material(world, shield_entity, &shield_material_name);

        self.player_shield_layers
            .push((shield_entity, duration, duration, layer_index));
    }

    fn spawn_shield_layer_break_effect(
        &mut self,
        world: &mut World,
        position: Vec3,
        layer_index: u32,
    ) {
        let effect_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
        let layer_color = Self::get_shield_layer_color(layer_index);
        let layer_radius = Self::get_shield_layer_radius(layer_index);

        let shield_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (
                    0.3,
                    Vec4::new(layer_color.x, layer_color.y, layer_color.z, 0.8),
                ),
                (
                    1.0,
                    Vec4::new(
                        layer_color.x * 0.5,
                        layer_color.y * 0.5,
                        layer_color.z * 0.5,
                        0.0,
                    ),
                ),
            ],
        };

        let shield_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere {
                radius: layer_radius,
            },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 40 + layer_index * 10,
            particle_lifetime_min: 0.4,
            particle_lifetime_max: 0.8,
            initial_velocity_min: 4.0,
            initial_velocity_max: 8.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -2.0, 0.0),
            drag: 0.5,
            size_start: 0.15,
            size_end: 0.02,
            color_gradient: shield_gradient,
            emissive_strength: 15.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.3,
            turbulence_frequency: 3.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(effect_entity, shield_emitter);

        self.line_effects.push(LineEffect {
            entity: world.spawn_entities(LINES, 1)[0],
            timer: 0.0,
            max_time: 0.3,
            center: position,
            start_radius: layer_radius,
            end_radius: layer_radius * 1.5,
            segments: 24,
            color_start: Vec4::new(layer_color.x, layer_color.y, layer_color.z, 1.0),
            color_end: Vec4::new(
                layer_color.x * 0.5,
                layer_color.y * 0.5,
                layer_color.z * 0.5,
                0.0,
            ),
        });
    }

    fn update_player_shield_system(&mut self, world: &mut World, delta: f32) {
        if self.stats.shield_level == 0 {
            return;
        }

        let mut layers_to_remove = Vec::new();
        let mut break_effects = Vec::new();

        for (index, (entity, timer, max_timer, layer_index)) in
            self.player_shield_layers.iter_mut().enumerate()
        {
            *timer -= delta;

            if *timer <= 0.0 {
                layers_to_remove.push(index);
                break_effects.push((self.player_position, *layer_index));
                world.queue_command(WorldCommand::DespawnRecursive { entity: *entity });
            } else {
                let time_ratio = *timer / *max_timer;
                let base_radius = Self::get_shield_layer_radius(*layer_index);
                let pulse = (self.game_time * (2.0 + *layer_index as f32 * 0.3)).sin() * 0.05 + 1.0;
                let radius = base_radius * pulse;

                if let Some(transform) = world.core.get_local_transform_mut(*entity) {
                    transform.scale = Vec3::new(radius * 2.0, radius * 2.0, radius * 2.0);
                    let rotation_speed = 0.5 + *layer_index as f32 * 0.2;
                    let rotation_axis = match *layer_index % 3 {
                        0 => Vec3::new(0.0, 1.0, 0.0),
                        1 => Vec3::new(0.3, 1.0, 0.0).normalize(),
                        _ => Vec3::new(-0.3, 1.0, 0.0).normalize(),
                    };
                    transform.rotation = nalgebra_glm::quat_angle_axis(
                        self.game_time * rotation_speed,
                        &rotation_axis,
                    );
                }
                mark_local_transform_dirty(world, *entity);

                let layer_color = Self::get_shield_layer_color(*layer_index);
                let alpha = 0.2 + time_ratio * 0.25;
                let material_name = format!("ShieldLayer_{}_{}", layer_index, entity.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [layer_color.x, layer_color.y, layer_color.z, alpha],
                        roughness: 0.1,
                        metallic: 0.0,
                        emissive_factor: [
                            layer_color.x * 0.4 * time_ratio,
                            layer_color.y * 0.4 * time_ratio,
                            layer_color.z * 0.4 * time_ratio,
                        ],
                        alpha_mode: AlphaMode::Blend,
                        ..Default::default()
                    },
                );
            }
        }

        for index in layers_to_remove.into_iter().rev() {
            self.player_shield_layers.remove(index);
        }

        for (position, layer_index) in break_effects {
            self.spawn_shield_layer_break_effect(world, position, layer_index);
        }

        if self.player_shield_layers.is_empty() && self.stats.shield_level > 0 {
            self.player_shield_regen_timer += delta;
            if self.player_shield_regen_timer >= SHIELD_REGEN_DELAY {
                self.player_shield_regen_timer = 0.0;
                for layer_index in 0..self.stats.shield_level {
                    self.spawn_shield_layer(world, layer_index);
                }
                self.spawn_popup_typed(
                    world,
                    self.player_position + Vec3::new(0.0, 1.5, 0.0),
                    "Shields Restored!".to_string(),
                    Vec4::new(0.5, 0.8, 1.0, 1.0),
                    PopupType::Xp,
                );
            }
        }
    }

    fn player_collision_system(&mut self, world: &mut World, delta: f32) {
        if self.damage_cooldown > 0.0 {
            self.damage_cooldown -= delta;
        }

        if self.invincibility_timer > 0.0 {
            return;
        }

        let player_pos = self.player_position;
        let enemies: Vec<freecs::Entity> = self
            .game_world
            .query_entities(ENEMY | POSITION | ENTITY_HANDLE)
            .collect();

        for game_entity in enemies {
            let position = self.game_world.get_position(game_entity).unwrap();
            let distance = nalgebra_glm::distance(&player_pos, &position.0);

            if distance < COLLISION_DISTANCE && self.damage_cooldown <= 0.0 {
                if !self.player_shield_layers.is_empty()
                    && let Some(outermost_index) = self
                        .player_shield_layers
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, (_, _, _, layer_index))| *layer_index)
                        .map(|(index, _)| index)
                {
                    let (entity, _, _, layer_index) =
                        self.player_shield_layers.remove(outermost_index);
                    world.queue_command(WorldCommand::DespawnRecursive { entity });
                    self.spawn_shield_layer_break_effect(world, player_pos, layer_index);
                    self.player_shield_regen_timer = 0.0;
                    self.damage_cooldown = DAMAGE_COOLDOWN * 0.5;

                    let layer_color = Self::get_shield_layer_color(layer_index);
                    self.spawn_popup_typed(
                        world,
                        player_pos + Vec3::new(0.0, 1.0, 0.0),
                        "BLOCKED!".to_string(),
                        layer_color,
                        PopupType::Xp,
                    );

                    let handle = self.game_world.get_entity_handle(game_entity).unwrap();
                    world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
                    self.game_world.despawn_entities(&[game_entity]);
                    self.game_world
                        .resources
                        .enemy_list
                        .retain(|&entity| entity != game_entity);
                    break;
                }

                self.player_health -= ENEMY_DAMAGE;
                self.damage_cooldown = DAMAGE_COOLDOWN;
                self.invincibility_timer = INVINCIBILITY_DURATION;
                self.camera_shake = 1.0;

                self.spawn_popup_typed(
                    world,
                    player_pos,
                    format!("-{}", ENEMY_DAMAGE as i32),
                    Vec4::new(1.0, 0.2, 0.2, 1.0),
                    PopupType::CriticalDamage,
                );

                self.spawn_damage_effect(world, player_pos);

                let handle = self.game_world.get_entity_handle(game_entity).unwrap();
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });

                self.game_world.despawn_entities(&[game_entity]);
                self.game_world
                    .resources
                    .enemy_list
                    .retain(|&entity| entity != game_entity);

                break;
            }
        }
    }

    fn start_game(&mut self, _world: &mut World) {
        self.game_state = GameState::Playing;
    }

    fn restart_game(&mut self, world: &mut World) {
        let all_entities: Vec<freecs::Entity> =
            self.game_world.query_entities(ENTITY_HANDLE).collect();
        for game_entity in all_entities {
            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
        }

        for popup_entity in self.game_world.resources.popup_list.clone() {
            if let Some(popup) = self.game_world.get_popup(popup_entity) {
                world.queue_command(WorldCommand::DespawnRecursive {
                    entity: popup.text_entity,
                });
            }
        }

        for orb_entity in &self.orb_entities {
            world.queue_command(WorldCommand::DespawnRecursive {
                entity: *orb_entity,
            });
        }
        self.orb_entities.clear();

        let mut to_despawn = self.game_world.resources.enemy_list.clone();
        to_despawn.extend(self.game_world.resources.projectile_list.clone());
        to_despawn.extend(self.game_world.resources.gem_list.clone());
        to_despawn.extend(self.game_world.resources.popup_list.clone());
        self.game_world.despawn_entities(&to_despawn);

        self.game_world.resources.enemy_list.clear();
        self.game_world.resources.projectile_list.clear();
        self.game_world.resources.gem_list.clear();
        self.game_world.resources.popup_list.clear();
        self.game_world.resources.spawn_timer = 0.0;
        self.game_world.resources.enemies_spawned = 0;
        self.game_world.resources.enemies_killed = 0;
        self.game_world.resources.current_wave = 0;
        self.game_world.resources.wave_timer = 0.0;
        self.game_world.resources.wave_enemies_remaining = 0;
        self.game_world.resources.boss_alive = false;

        if let Some(emitter) = self.garlic_emitter {
            world.queue_command(WorldCommand::DespawnRecursive { entity: emitter });
        }
        self.garlic_emitter = None;

        for (shield_entity, _, _, _) in &self.player_shield_layers {
            world.queue_command(WorldCommand::DespawnRecursive {
                entity: *shield_entity,
            });
        }
        self.player_shield_layers.clear();
        self.player_shield_regen_timer = 0.0;

        for (_, shield_visual, _) in &self.enemy_shield_entities {
            world.queue_command(WorldCommand::DespawnRecursive {
                entity: *shield_visual,
            });
        }
        self.enemy_shield_entities.clear();

        self.player_position = Vec3::new(0.0, PLAYER_RADIUS, 0.0);
        self.stats = PlayerStats::default();
        self.player_health = self.stats.max_health;
        self.damage_cooldown = 0.0;
        self.attack_cooldown = 0.0;
        self.player_xp = 0;
        self.player_level = 1;
        self.orb_angle = 0.0;
        self.pulse_cooldown = 0.0;
        self.regen_timer = 0.0;
        self.game_time = 0.0;
        self.camera_shake = 0.0;
        self.invincibility_timer = 0.0;
        self.whip_cooldown = 0.0;
        self.whip_angle = 0.0;
        self.lightning_cooldown = 0.0;
        self.garlic_timer = 0.0;
        self.bomb_cooldown = 0.0;
        self.player_facing = Vec3::new(1.0, 0.0, 0.0);
        self.dust_timer = 0.0;
        self.combo_count = 0;
        self.combo_timer = 0.0;
        self.speed_boost_timer = 0.0;
        self.last_wave_announced = 0;
        self.kill_flash_timer = 0.0;
        self.game_speed = 1.0;
        self.lob_bombs.clear();
        self.game_state = GameState::Playing;

        if let Some(entity) = self.player_entity {
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.translation = self.player_position;
            }
            mark_local_transform_dirty(world, entity);
        }
    }

    fn orb_system(&mut self, world: &mut World, delta: f32) {
        let target_orb_count = self.stats.orb_count as usize;
        let current_orb_count = self.orb_entities.len();

        if current_orb_count < target_orb_count {
            for _ in current_orb_count..target_orb_count {
                self.spawn_orb(world);
            }
        }

        if self.orb_entities.is_empty() {
            return;
        }

        self.orb_angle += ORB_ORBIT_SPEED * delta;
        if self.orb_angle > std::f32::consts::TAU {
            self.orb_angle -= std::f32::consts::TAU;
        }

        let orb_count = self.orb_entities.len();
        let angle_step = std::f32::consts::TAU / orb_count as f32;

        for (index, orb_entity) in self.orb_entities.iter().enumerate() {
            let angle = self.orb_angle + angle_step * index as f32;
            let orb_offset = Vec3::new(
                angle.cos() * ORB_ORBIT_RADIUS,
                0.5,
                angle.sin() * ORB_ORBIT_RADIUS,
            );
            let orb_position = self.player_position + orb_offset;

            if let Some(transform) = world.core.get_local_transform_mut(*orb_entity) {
                transform.translation = orb_position;
            }
            mark_local_transform_dirty(world, *orb_entity);
        }

        self.orb_collision_system(world);
    }

    fn spawn_orb(&mut self, world: &mut World) {
        let orb_index = self.orb_entities.len();
        let hue = (orb_index as f32 * 0.3) % 1.0;
        let (ring_r, ring_g, ring_b) = hsv_to_rgb(hue, 0.8, 1.0);
        let (core_r, core_g, core_b) = hsv_to_rgb(hue, 0.5, 1.0);

        let engine_entity = spawn_mesh(
            world,
            "Torus",
            self.player_position,
            Vec3::new(ORB_RADIUS * 2.5, ORB_RADIUS * 0.8, ORB_RADIUS * 2.5),
        );

        let ring_mat = format!("OrbRing_{}", engine_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            ring_mat.clone(),
            Material {
                base_color: [ring_r, ring_g, ring_b, 1.0],
                roughness: 0.2,
                metallic: 0.7,
                emissive_factor: [ring_r * 0.6, ring_g * 0.6, ring_b * 0.6],
                ..Default::default()
            },
        );
        self.apply_material(world, engine_entity, &ring_mat);

        let core = spawn_mesh(
            world,
            "Sphere",
            Vec3::zeros(),
            Vec3::new(ORB_RADIUS * 1.0, ORB_RADIUS * 1.0, ORB_RADIUS * 1.0),
        );
        world.core.set_parent(core, Parent(Some(engine_entity)));
        let core_mat = format!("OrbCore_{}", core.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            core_mat.clone(),
            Material {
                base_color: [core_r, core_g, core_b, 1.0],
                roughness: 0.1,
                metallic: 0.9,
                emissive_factor: [core_r * 0.8, core_g * 0.8, core_b * 0.8],
                ..Default::default()
            },
        );
        self.apply_material(world, core, &core_mat);

        self.orb_entities.push(engine_entity);
    }

    fn orb_collision_system(&mut self, world: &mut World) {
        if self.orb_entities.is_empty() {
            return;
        }

        let orb_count = self.orb_entities.len();
        let angle_step = std::f32::consts::TAU / orb_count as f32;

        let orb_positions: Vec<Vec3> = (0..orb_count)
            .map(|index| {
                let angle = self.orb_angle + angle_step * index as f32;
                self.player_position
                    + Vec3::new(
                        angle.cos() * ORB_ORBIT_RADIUS,
                        0.5,
                        angle.sin() * ORB_ORBIT_RADIUS,
                    )
            })
            .collect();

        let enemies: Vec<freecs::Entity> = self
            .game_world
            .query_entities(ENEMY | POSITION | ENTITY_HANDLE)
            .collect();
        let mut enemies_to_remove = Vec::new();
        let mut gem_spawn_data: Vec<(Vec3, u32, EnemyType)> = Vec::new();
        let mut damage_popups: Vec<(Vec3, f32)> = Vec::new();

        for enemy_entity in &enemies {
            if enemies_to_remove.contains(enemy_entity) {
                continue;
            }

            let enemy_pos = *self.game_world.get_position(*enemy_entity).unwrap();
            let enemy = *self.game_world.get_enemy(*enemy_entity).unwrap();

            for orb_pos in &orb_positions {
                let distance = nalgebra_glm::distance(orb_pos, &enemy_pos.0);

                if distance < ORB_HIT_DISTANCE {
                    let orb_damage = ORB_DAMAGE
                        * self.stats.damage_multiplier
                        * self.stats.buff_damage_multiplier;
                    let new_health = enemy.health - orb_damage;
                    damage_popups.push((enemy_pos.0, orb_damage));

                    if new_health <= 0.0 {
                        enemies_to_remove.push(*enemy_entity);
                        gem_spawn_data.push((enemy_pos.0, enemy.xp_value, enemy.enemy_type));
                        self.game_world.resources.enemies_killed += 1;
                    } else {
                        self.game_world.set_enemy(
                            *enemy_entity,
                            Enemy {
                                health: new_health,
                                ..enemy
                            },
                        );
                    }
                    break;
                }
            }
        }

        for (pos, damage) in damage_popups {
            self.spawn_damage_popup(world, pos, damage, Vec4::new(0.5, 0.9, 1.0, 1.0), false);
        }

        for game_entity in enemies_to_remove {
            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            self.game_world.despawn_entities(&[game_entity]);
            self.game_world
                .resources
                .enemy_list
                .retain(|&e| e != game_entity);
        }

        for (pos, xp_value, enemy_type) in gem_spawn_data {
            let is_boss = enemy_type == EnemyType::Boss;
            if is_boss {
                self.game_world.resources.boss_alive = false;
                self.spawn_boss_death_effect(world, pos);
            }
            self.spawn_death_particles_for_type(world, pos, enemy_type);
            self.spawn_gem_with_xp(world, pos, xp_value);
            self.maybe_spawn_enemy_health_drop(world, pos, enemy_type);
            self.add_kill(world, is_boss);
        }
    }

    fn pulse_system(&mut self, world: &mut World, delta: f32) {
        if self.stats.area_pulse_level == 0 {
            return;
        }

        self.pulse_cooldown -= delta;
        if self.pulse_cooldown > 0.0 {
            return;
        }

        self.pulse_cooldown = PULSE_COOLDOWN / (1.0 + self.stats.area_pulse_level as f32 * 0.2);

        let pulse_damage = PULSE_BASE_DAMAGE
            * self.stats.area_pulse_level as f32
            * self.stats.damage_multiplier
            * self.stats.buff_damage_multiplier;
        let pulse_radius = PULSE_RADIUS + self.stats.area_pulse_level as f32 * 1.0;
        let player_pos = self.player_position;

        self.spawn_pulse_effect(world, player_pos, pulse_radius);

        let enemies: Vec<freecs::Entity> = self
            .game_world
            .query_entities(ENEMY | POSITION | ENTITY_HANDLE)
            .collect();
        let mut enemies_to_remove = Vec::new();
        let mut gem_spawn_data: Vec<(Vec3, u32, EnemyType)> = Vec::new();
        let mut damage_popups: Vec<(Vec3, f32)> = Vec::new();

        for enemy_entity in &enemies {
            if enemies_to_remove.contains(enemy_entity) {
                continue;
            }

            let enemy_pos = *self.game_world.get_position(*enemy_entity).unwrap();
            let enemy = *self.game_world.get_enemy(*enemy_entity).unwrap();
            let distance = nalgebra_glm::distance(&player_pos, &enemy_pos.0);

            if distance <= pulse_radius {
                let new_health = enemy.health - pulse_damage;
                damage_popups.push((enemy_pos.0, pulse_damage));

                if new_health <= 0.0 {
                    enemies_to_remove.push(*enemy_entity);
                    gem_spawn_data.push((enemy_pos.0, enemy.xp_value, enemy.enemy_type));
                    self.game_world.resources.enemies_killed += 1;
                } else {
                    self.game_world.set_enemy(
                        *enemy_entity,
                        Enemy {
                            health: new_health,
                            ..enemy
                        },
                    );
                }
            }
        }

        for (pos, damage) in damage_popups {
            self.spawn_damage_popup(world, pos, damage, Vec4::new(1.0, 0.5, 0.9, 1.0), false);
        }

        for game_entity in enemies_to_remove {
            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            self.game_world.despawn_entities(&[game_entity]);
            self.game_world
                .resources
                .enemy_list
                .retain(|&e| e != game_entity);
        }

        for (pos, xp_value, enemy_type) in gem_spawn_data {
            let is_boss = enemy_type == EnemyType::Boss;
            if is_boss {
                self.game_world.resources.boss_alive = false;
                self.spawn_boss_death_effect(world, pos);
            }
            self.spawn_death_particles_for_type(world, pos, enemy_type);
            self.spawn_gem_with_xp(world, pos, xp_value);
            self.maybe_spawn_enemy_health_drop(world, pos, enemy_type);
            self.add_kill(world, is_boss);
        }
    }

    fn spawn_pulse_effect(&mut self, world: &mut World, position: Vec3, radius: f32) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let pulse_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.4, 0.7, 1.0)),
                (0.2, Vec4::new(1.0, 0.6, 0.9, 0.9)),
                (0.5, Vec4::new(0.8, 0.5, 1.0, 0.6)),
                (1.0, Vec4::new(0.6, 0.4, 0.9, 0.0)),
            ],
        };

        let pulse_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere {
                radius: radius * 0.9,
            },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: (radius * 30.0) as u32,
            particle_lifetime_min: 0.4,
            particle_lifetime_max: 0.8,
            initial_velocity_min: radius * 1.5,
            initial_velocity_max: radius * 3.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -3.0, 0.0),
            drag: 0.4,
            size_start: 0.4,
            size_end: 0.1,
            color_gradient: pulse_gradient,
            emissive_strength: 20.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.6,
            turbulence_frequency: 3.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, pulse_emitter);

        let ring_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let ring_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.8, 1.0, 0.8)),
                (0.5, Vec4::new(0.9, 0.5, 0.9, 0.5)),
                (1.0, Vec4::new(0.7, 0.3, 0.8, 0.0)),
            ],
        };

        let ring_emitter = ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Sphere {
                radius: radius * 0.3,
            },
            position: position + Vec3::new(0.0, 0.2, 0.0),
            direction: Vec3::new(0.0, 0.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 40,
            particle_lifetime_min: 0.3,
            particle_lifetime_max: 0.5,
            initial_velocity_min: radius * 2.0,
            initial_velocity_max: radius * 4.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 0.0, 0.0),
            drag: 0.8,
            size_start: 0.8,
            size_end: 1.5,
            color_gradient: ring_gradient,
            emissive_strength: 12.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.2,
            turbulence_frequency: 1.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(ring_entity, ring_emitter);

        self.spawn_pulse_lines(world, position, radius);
    }

    fn spawn_pulse_lines(&mut self, world: &mut World, position: Vec3, radius: f32) {
        for ring_index in 0..3 {
            let line_entity = world.spawn_entities(
                LINES | VISIBILITY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
                1,
            )[0];

            let delay_factor = ring_index as f32 * 0.15;
            let start_radius = radius * 0.1;
            let end_radius = radius * (1.2 + ring_index as f32 * 0.3);

            let effect = LineEffect {
                entity: line_entity,
                timer: 0.0,
                max_time: 0.5 + delay_factor,
                center: Vec3::new(position.x, 0.2, position.z),
                start_radius,
                end_radius,
                segments: 32,
                color_start: Vec4::new(1.0, 0.6, 0.9, 0.9 - ring_index as f32 * 0.2),
                color_end: Vec4::new(0.8, 0.4, 1.0, 0.0),
            };

            self.line_effects.push(effect);
        }
    }

    fn update_line_effects(&mut self, world: &mut World, delta: f32) {
        let mut effects_to_remove = Vec::new();

        for (index, effect) in self.line_effects.iter_mut().enumerate() {
            effect.timer += delta;

            let progress = (effect.timer / effect.max_time).min(1.0);

            if progress >= 1.0 {
                effects_to_remove.push(index);
                world.queue_command(WorldCommand::DespawnRecursive {
                    entity: effect.entity,
                });
                continue;
            }

            let current_alpha = effect.color_start.w * (1.0 - progress);
            let current_color = Vec4::new(
                effect.color_start.x + (effect.color_end.x - effect.color_start.x) * progress,
                effect.color_start.y + (effect.color_end.y - effect.color_start.y) * progress,
                effect.color_start.z + (effect.color_end.z - effect.color_start.z) * progress,
                current_alpha,
            );

            if effect.segments == 0 {
                if let Some(lines_component) = world.core.get_lines_mut(effect.entity) {
                    for line in lines_component.lines.iter_mut() {
                        line.color = current_color;
                    }
                    lines_component.mark_dirty();
                }
            } else {
                let current_radius =
                    effect.start_radius + (effect.end_radius - effect.start_radius) * progress;
                let mut lines = Vec::new();
                let segments = effect.segments;

                for segment_index in 0..segments {
                    let angle1 = (segment_index as f32 / segments as f32) * std::f32::consts::TAU;
                    let angle2 =
                        ((segment_index + 1) as f32 / segments as f32) * std::f32::consts::TAU;

                    let point1 = effect.center
                        + Vec3::new(
                            angle1.cos() * current_radius,
                            0.0,
                            angle1.sin() * current_radius,
                        );
                    let point2 = effect.center
                        + Vec3::new(
                            angle2.cos() * current_radius,
                            0.0,
                            angle2.sin() * current_radius,
                        );

                    lines.push(Line {
                        start: point1,
                        end: point2,
                        color: current_color,
                    });
                }

                if let Some(lines_component) = world.core.get_lines_mut(effect.entity) {
                    lines_component.lines = lines;
                    lines_component.mark_dirty();
                }
            }
        }

        for index in effects_to_remove.into_iter().rev() {
            self.line_effects.remove(index);
        }
    }

    fn regen_system(&mut self, world: &mut World, delta: f32) {
        if self.stats.regen_level == 0 {
            return;
        }

        self.regen_timer += delta;
        if self.regen_timer >= REGEN_INTERVAL {
            self.regen_timer = 0.0;

            let regen_amount = REGEN_AMOUNT * self.stats.regen_level as f32;
            let old_health = self.player_health;
            self.player_health = (self.player_health + regen_amount).min(self.stats.max_health);
            let actual_heal = self.player_health - old_health;

            if actual_heal > 0.0 {
                self.spawn_popup_typed(
                    world,
                    self.player_position,
                    format!("+{}", actual_heal as i32),
                    Vec4::new(0.3, 1.0, 0.4, 1.0),
                    PopupType::Xp,
                );
            }
        }
    }

    fn update_invincibility(&mut self, world: &mut World, delta: f32) {
        if self.invincibility_timer > 0.0 {
            self.invincibility_timer -= delta;

            if let Some(player) = self.player_entity {
                let flash = ((self.invincibility_timer * INVINCIBILITY_FLASH_RATE)
                    * std::f32::consts::TAU)
                    .sin()
                    * 0.5
                    + 0.5;
                if let Some(visibility) = world.core.get_visibility_mut(player) {
                    visibility.visible = flash > 0.3;
                }
            }
        } else if let Some(player) = self.player_entity
            && let Some(visibility) = world.core.get_visibility_mut(player)
        {
            visibility.visible = true;
        }
    }

    fn spawn_damage_effect(&self, world: &mut World, position: Vec3) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let damage_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.2, 0.2, 1.0)),
                (0.3, Vec4::new(1.0, 0.4, 0.1, 0.8)),
                (0.6, Vec4::new(0.8, 0.2, 0.1, 0.5)),
                (1.0, Vec4::new(0.5, 0.0, 0.0, 0.0)),
            ],
        };

        let damage_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere {
                radius: PLAYER_RADIUS,
            },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 30,
            particle_lifetime_min: 0.3,
            particle_lifetime_max: 0.6,
            initial_velocity_min: 3.0,
            initial_velocity_max: 6.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -3.0, 0.0),
            drag: 0.4,
            size_start: 0.12,
            size_end: 0.03,
            color_gradient: damage_gradient,
            emissive_strength: 12.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.4,
            turbulence_frequency: 3.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, damage_emitter);
    }

    fn whip_system(&mut self, world: &mut World, delta: f32) {
        if self.stats.whip_level == 0 {
            return;
        }

        self.whip_cooldown -= delta;
        if self.whip_cooldown > 0.0 {
            return;
        }

        self.whip_cooldown = WHIP_COOLDOWN / (1.0 + self.stats.whip_level as f32 * 0.1);

        self.whip_angle += std::f32::consts::PI;
        if self.whip_angle > std::f32::consts::TAU {
            self.whip_angle -= std::f32::consts::TAU;
        }

        let whip_direction = Vec3::new(self.whip_angle.cos(), 0.0, self.whip_angle.sin());
        let whip_damage = WHIP_DAMAGE
            * self.stats.damage_multiplier
            * self.stats.buff_damage_multiplier
            * (1.0 + self.stats.whip_level as f32 * 0.2);
        let whip_range = WHIP_RANGE * (1.0 + self.stats.whip_level as f32 * 0.15);

        let mut enemies_hit = Vec::new();
        let mut damage_popups = Vec::new();

        let enemies: Vec<freecs::Entity> = self
            .game_world
            .query_entities(ENEMY | POSITION | ENTITY_HANDLE)
            .collect();

        for game_entity in enemies {
            let pos = self.game_world.get_position(game_entity).unwrap().0;
            let to_enemy = pos - self.player_position;
            let distance = nalgebra_glm::length(&to_enemy);

            if distance > whip_range {
                continue;
            }

            let to_enemy_norm = nalgebra_glm::normalize(&to_enemy);
            let dot = nalgebra_glm::dot(&to_enemy_norm, &whip_direction);
            let angle = dot.acos();

            if angle < WHIP_ARC * 0.5 {
                let enemy = *self.game_world.get_enemy(game_entity).unwrap();
                let new_health = enemy.health - whip_damage;

                if new_health <= 0.0 {
                    enemies_hit.push((game_entity, pos, enemy.xp_value, enemy.enemy_type));
                } else {
                    self.game_world.set_enemy(
                        game_entity,
                        Enemy {
                            health: new_health,
                            ..enemy
                        },
                    );
                    damage_popups.push((pos, whip_damage));
                }
            }
        }

        self.spawn_whip_effect(world, self.player_position, whip_direction, whip_range);

        for (pos, damage) in damage_popups {
            self.spawn_damage_popup(world, pos, damage, Vec4::new(1.0, 0.85, 0.3, 1.0), false);
            self.spawn_hit_effect(world, pos);
        }

        for (game_entity, pos, xp, enemy_type) in enemies_hit {
            let is_boss = enemy_type == EnemyType::Boss;
            self.spawn_damage_popup(
                world,
                pos,
                whip_damage,
                Vec4::new(1.0, 0.85, 0.3, 1.0),
                is_boss,
            );
            if is_boss {
                self.game_world.resources.boss_alive = false;
                self.spawn_boss_death_effect(world, pos);
            }
            self.spawn_death_particles_for_type(world, pos, enemy_type);
            self.spawn_gem_with_xp(world, pos, xp);
            self.maybe_spawn_enemy_health_drop(world, pos, enemy_type);
            self.add_kill(world, is_boss);

            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            self.game_world.despawn_entities(&[game_entity]);
            self.game_world
                .resources
                .enemy_list
                .retain(|&e| e != game_entity);
            self.game_world.resources.enemies_killed += 1;
        }
    }

    fn spawn_whip_effect(&self, world: &mut World, position: Vec3, direction: Vec3, range: f32) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let whip_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 0.8, 1.0)),
                (0.2, Vec4::new(1.0, 0.9, 0.5, 0.95)),
                (0.5, Vec4::new(1.0, 0.6, 0.2, 0.7)),
                (1.0, Vec4::new(0.8, 0.3, 0.1, 0.0)),
            ],
        };

        let offset = direction * range * 0.5;
        let effect_pos = position + offset + Vec3::new(0.0, 0.5, 0.0);

        let whip_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Cone {
                angle: 1.2,
                height: range * 0.7,
            },
            position: effect_pos,
            direction,
            spawn_rate: 0.0,
            burst_count: 80,
            particle_lifetime_min: 0.2,
            particle_lifetime_max: 0.5,
            initial_velocity_min: 12.0,
            initial_velocity_max: 25.0,
            velocity_spread: 0.8,
            gravity: Vec3::new(0.0, -4.0, 0.0),
            drag: 0.4,
            size_start: 0.35,
            size_end: 0.05,
            color_gradient: whip_gradient,
            emissive_strength: 25.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.4,
            turbulence_frequency: 5.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, whip_emitter);

        let trail_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let trail_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.8, 0.4, 0.6)),
                (0.5, Vec4::new(0.9, 0.5, 0.2, 0.3)),
                (1.0, Vec4::new(0.7, 0.3, 0.1, 0.0)),
            ],
        };

        let trail_emitter = ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Cone {
                angle: 1.5,
                height: range * 0.6,
            },
            position: effect_pos,
            direction,
            spawn_rate: 0.0,
            burst_count: 30,
            particle_lifetime_min: 0.15,
            particle_lifetime_max: 0.35,
            initial_velocity_min: 8.0,
            initial_velocity_max: 18.0,
            velocity_spread: 0.9,
            gravity: Vec3::new(0.0, 1.0, 0.0),
            drag: 0.5,
            size_start: 0.5,
            size_end: 1.2,
            color_gradient: trail_gradient,
            emissive_strength: 8.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.3,
            turbulence_frequency: 2.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(trail_entity, trail_emitter);
    }

    fn lightning_system(&mut self, world: &mut World, delta: f32) {
        if self.stats.lightning_level == 0 {
            return;
        }

        self.lightning_cooldown -= delta;
        if self.lightning_cooldown > 0.0 {
            return;
        }

        self.lightning_cooldown =
            LIGHTNING_COOLDOWN / (1.0 + self.stats.lightning_level as f32 * 0.1);

        let lightning_damage = LIGHTNING_DAMAGE
            * self.stats.damage_multiplier
            * self.stats.buff_damage_multiplier
            * (1.0 + self.stats.lightning_level as f32 * 0.25);
        let chain_count = LIGHTNING_CHAIN_COUNT + self.stats.lightning_level - 1;
        let range = LIGHTNING_RANGE * (1.0 + self.stats.lightning_level as f32 * 0.1);

        let enemies: Vec<freecs::Entity> = self
            .game_world
            .query_entities(ENEMY | POSITION | ENTITY_HANDLE)
            .collect();

        if enemies.is_empty() {
            return;
        }

        let mut first_target: Option<(freecs::Entity, Vec3, f32)> = None;
        for game_entity in &enemies {
            if let Some(position) = self.game_world.get_position(*game_entity) {
                let distance = nalgebra_glm::distance(&self.player_position, &position.0);
                if distance < range
                    && (first_target.is_none() || distance < first_target.unwrap().2)
                {
                    first_target = Some((*game_entity, position.0, distance));
                }
            }
        }

        let Some((first_entity, first_pos, _)) = first_target else {
            return;
        };

        let mut chain_targets = vec![(first_entity, first_pos)];
        let mut hit_entities = std::collections::HashSet::new();
        hit_entities.insert(first_entity);

        let mut last_pos = first_pos;
        for _ in 1..chain_count {
            let mut next_target: Option<(freecs::Entity, Vec3, f32)> = None;

            for game_entity in &enemies {
                if hit_entities.contains(game_entity) {
                    continue;
                }
                if let Some(position) = self.game_world.get_position(*game_entity) {
                    let distance = nalgebra_glm::distance(&last_pos, &position.0);
                    if distance < LIGHTNING_CHAIN_RANGE
                        && (next_target.is_none() || distance < next_target.unwrap().2)
                    {
                        next_target = Some((*game_entity, position.0, distance));
                    }
                }
            }

            if let Some((entity, pos, _)) = next_target {
                chain_targets.push((entity, pos));
                hit_entities.insert(entity);
                last_pos = pos;
            } else {
                break;
            }
        }

        self.spawn_lightning_effect(world, self.player_position, &chain_targets);

        let mut enemies_to_kill = Vec::new();
        let mut damage_popups = Vec::new();

        for (game_entity, pos) in chain_targets {
            if let Some(enemy) = self.game_world.get_enemy(game_entity) {
                let new_health = enemy.health - lightning_damage;
                if new_health <= 0.0 {
                    enemies_to_kill.push((game_entity, pos, enemy.xp_value, enemy.enemy_type));
                } else {
                    self.game_world.set_enemy(
                        game_entity,
                        Enemy {
                            health: new_health,
                            ..*enemy
                        },
                    );
                    damage_popups.push((pos, lightning_damage));
                }
            }
        }

        for (pos, damage) in damage_popups {
            self.spawn_damage_popup(world, pos, damage, Vec4::new(0.6, 0.85, 1.0, 1.0), false);
        }

        for (game_entity, pos, xp, enemy_type) in enemies_to_kill {
            let is_boss = enemy_type == EnemyType::Boss;
            self.spawn_damage_popup(
                world,
                pos,
                lightning_damage,
                Vec4::new(0.6, 0.85, 1.0, 1.0),
                is_boss,
            );
            if is_boss {
                self.game_world.resources.boss_alive = false;
                self.spawn_boss_death_effect(world, pos);
            }
            self.spawn_death_particles_for_type(world, pos, enemy_type);
            self.spawn_gem_with_xp(world, pos, xp);
            self.maybe_spawn_enemy_health_drop(world, pos, enemy_type);
            self.add_kill(world, is_boss);

            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            self.game_world.despawn_entities(&[game_entity]);
            self.game_world
                .resources
                .enemy_list
                .retain(|&e| e != game_entity);
            self.game_world.resources.enemies_killed += 1;
        }
    }

    fn spawn_lightning_effect(
        &mut self,
        world: &mut World,
        start: Vec3,
        targets: &[(freecs::Entity, Vec3)],
    ) {
        let mut prev_pos = start + Vec3::new(0.0, 1.0, 0.0);

        self.spawn_lightning_bolt_lines(world, start, targets);

        for (_, target_pos) in targets {
            let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

            let lightning_gradient = ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                    (0.1, Vec4::new(0.8, 0.9, 1.0, 1.0)),
                    (0.3, Vec4::new(0.5, 0.7, 1.0, 0.9)),
                    (0.6, Vec4::new(0.3, 0.5, 1.0, 0.6)),
                    (1.0, Vec4::new(0.2, 0.3, 0.8, 0.0)),
                ],
            };

            let mid = (prev_pos + *target_pos) * 0.5;
            let direction = nalgebra_glm::normalize(&(*target_pos - prev_pos));
            let distance = nalgebra_glm::distance(&prev_pos, target_pos);

            let lightning_emitter = ParticleEmitter {
                emitter_type: EmitterType::Sparks,
                shape: EmitterShape::Sphere {
                    radius: distance * 0.3,
                },
                position: mid,
                direction,
                spawn_rate: 0.0,
                burst_count: 60,
                particle_lifetime_min: 0.15,
                particle_lifetime_max: 0.35,
                initial_velocity_min: 8.0,
                initial_velocity_max: 20.0,
                velocity_spread: 0.6,
                gravity: Vec3::new(0.0, -2.0, 0.0),
                drag: 0.6,
                size_start: 0.25,
                size_end: 0.04,
                color_gradient: lightning_gradient,
                emissive_strength: 35.0,
                enabled: true,
                accumulated_spawn: 0.0,
                one_shot: true,
                has_fired: false,
                turbulence_strength: 0.8,
                turbulence_frequency: 6.0,

                ..Default::default()
            };

            world.core.set_particle_emitter(particle_entity, lightning_emitter);

            let impact_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

            let impact_gradient = ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                    (0.2, Vec4::new(0.6, 0.8, 1.0, 0.9)),
                    (0.5, Vec4::new(0.4, 0.6, 1.0, 0.5)),
                    (1.0, Vec4::new(0.2, 0.4, 0.9, 0.0)),
                ],
            };

            let impact_emitter = ParticleEmitter {
                emitter_type: EmitterType::Sparks,
                shape: EmitterShape::Point,
                position: *target_pos + Vec3::new(0.0, 0.5, 0.0),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 0.0,
                burst_count: 40,
                particle_lifetime_min: 0.2,
                particle_lifetime_max: 0.4,
                initial_velocity_min: 5.0,
                initial_velocity_max: 12.0,
                velocity_spread: 1.0,
                gravity: Vec3::new(0.0, -5.0, 0.0),
                drag: 0.3,
                size_start: 0.3,
                size_end: 0.05,
                color_gradient: impact_gradient,
                emissive_strength: 30.0,
                enabled: true,
                accumulated_spawn: 0.0,
                one_shot: true,
                has_fired: false,
                turbulence_strength: 0.5,
                turbulence_frequency: 4.0,

                ..Default::default()
            };

            world.core.set_particle_emitter(impact_entity, impact_emitter);

            prev_pos = *target_pos;
        }
    }

    fn spawn_lightning_bolt_lines(
        &mut self,
        world: &mut World,
        start: Vec3,
        targets: &[(freecs::Entity, Vec3)],
    ) {
        let line_entity = world.spawn_entities(
            LINES | VISIBILITY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
            1,
        )[0];

        let mut lines = Vec::new();
        let mut prev_pos = start + Vec3::new(0.0, 1.0, 0.0);
        let mut rng = rand::rng();

        for (_, target_pos) in targets {
            let target = *target_pos + Vec3::new(0.0, 0.5, 0.0);
            let direction = target - prev_pos;
            let distance = nalgebra_glm::length(&direction);
            let segments = (distance * 3.0).max(4.0) as u32;
            let dir_norm = nalgebra_glm::normalize(&direction);

            let perpendicular = if dir_norm.y.abs() < 0.9 {
                nalgebra_glm::normalize(&nalgebra_glm::cross(&dir_norm, &Vec3::new(0.0, 1.0, 0.0)))
            } else {
                nalgebra_glm::normalize(&nalgebra_glm::cross(&dir_norm, &Vec3::new(1.0, 0.0, 0.0)))
            };

            let mut current_pos = prev_pos;

            for segment_index in 0..segments {
                let progress = (segment_index + 1) as f32 / segments as f32;
                let base_pos = prev_pos + direction * progress;

                let jitter_amount = if segment_index == segments - 1 {
                    0.0
                } else {
                    (1.0 - progress) * 0.3 * (rng.random::<f32>() - 0.5) * 2.0
                };

                let next_pos = base_pos + perpendicular * jitter_amount;

                let bolt_color = Vec4::new(0.7, 0.85, 1.0, 1.0);
                lines.push(Line {
                    start: current_pos,
                    end: next_pos,
                    color: bolt_color,
                });

                current_pos = next_pos;
            }

            prev_pos = target;
        }

        if let Some(lines_component) = world.core.get_lines_mut(line_entity) {
            lines_component.lines = lines;
            lines_component.mark_dirty();
        }

        let effect = LineEffect {
            entity: line_entity,
            timer: 0.0,
            max_time: 0.15,
            center: start,
            start_radius: 0.0,
            end_radius: 0.0,
            segments: 0,
            color_start: Vec4::new(0.7, 0.85, 1.0, 1.0),
            color_end: Vec4::new(0.5, 0.7, 1.0, 0.0),
        };

        self.line_effects.push(effect);
    }

    fn garlic_system(&mut self, world: &mut World, delta: f32) {
        if self.stats.garlic_level == 0 {
            if let Some(emitter) = self.garlic_emitter {
                world.queue_command(WorldCommand::DespawnRecursive { entity: emitter });
                self.garlic_emitter = None;
            }
            return;
        }

        if self.garlic_emitter.is_none() {
            let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
            self.garlic_emitter = Some(particle_entity);

            let garlic_gradient = ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(0.6, 1.0, 0.6, 0.6)),
                    (0.5, Vec4::new(0.4, 0.9, 0.4, 0.4)),
                    (1.0, Vec4::new(0.2, 0.7, 0.2, 0.0)),
                ],
            };

            let garlic_emitter = ParticleEmitter {
                emitter_type: EmitterType::Smoke,
                shape: EmitterShape::Sphere {
                    radius: GARLIC_RADIUS * 0.8,
                },
                position: self.player_position,
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 15.0,
                burst_count: 0,
                particle_lifetime_min: 0.5,
                particle_lifetime_max: 1.0,
                initial_velocity_min: 0.5,
                initial_velocity_max: 1.5,
                velocity_spread: 1.0,
                gravity: Vec3::new(0.0, 0.5, 0.0),
                drag: 0.3,
                size_start: 0.15,
                size_end: 0.3,
                color_gradient: garlic_gradient,
                emissive_strength: 3.0,
                enabled: true,
                accumulated_spawn: 0.0,
                one_shot: false,
                has_fired: false,
                turbulence_strength: 0.3,
                turbulence_frequency: 2.0,

                ..Default::default()
            };

            world.core.set_particle_emitter(particle_entity, garlic_emitter);
        }

        if let Some(emitter_entity) = self.garlic_emitter
            && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
        {
            emitter.position = self.player_position;
        }

        self.garlic_timer += delta;
        if self.garlic_timer < GARLIC_TICK_RATE {
            return;
        }
        self.garlic_timer = 0.0;

        let garlic_damage = GARLIC_DAMAGE
            * self.stats.damage_multiplier
            * self.stats.buff_damage_multiplier
            * (1.0 + self.stats.garlic_level as f32 * 0.3);
        let garlic_radius = GARLIC_RADIUS * (1.0 + self.stats.garlic_level as f32 * 0.2);

        let enemies: Vec<freecs::Entity> = self
            .game_world
            .query_entities(ENEMY | POSITION | ENTITY_HANDLE)
            .collect();

        let mut enemies_to_kill = Vec::new();

        for game_entity in enemies {
            let position = self.game_world.get_position(game_entity).unwrap();
            let distance = nalgebra_glm::distance(&self.player_position, &position.0);

            if distance < garlic_radius {
                let enemy = self.game_world.get_enemy(game_entity).unwrap();
                let new_health = enemy.health - garlic_damage;

                if new_health <= 0.0 {
                    enemies_to_kill.push((
                        game_entity,
                        position.0,
                        enemy.xp_value,
                        enemy.enemy_type,
                    ));
                } else {
                    self.game_world.set_enemy(
                        game_entity,
                        Enemy {
                            health: new_health,
                            ..*enemy
                        },
                    );
                }
            }
        }

        for (game_entity, pos, xp, enemy_type) in enemies_to_kill {
            let is_boss = enemy_type == EnemyType::Boss;
            self.spawn_damage_popup(
                world,
                pos,
                garlic_damage,
                Vec4::new(0.5, 1.0, 0.5, 1.0),
                is_boss,
            );
            if is_boss {
                self.game_world.resources.boss_alive = false;
                self.spawn_boss_death_effect(world, pos);
            }
            self.spawn_death_particles_for_type(world, pos, enemy_type);
            self.spawn_gem_with_xp(world, pos, xp);
            self.maybe_spawn_enemy_health_drop(world, pos, enemy_type);
            self.add_kill(world, is_boss);

            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            self.game_world.despawn_entities(&[game_entity]);
            self.game_world
                .resources
                .enemy_list
                .retain(|&e| e != game_entity);
            self.game_world.resources.enemies_killed += 1;
        }
    }

    fn bomb_system(&mut self, world: &mut World, delta: f32) {
        if self.stats.bomb_level == 0 {
            return;
        }

        if self.bomb_cooldown > 0.0 {
            self.bomb_cooldown -= delta;
            return;
        }

        self.use_bomb(world);
        self.bomb_cooldown = BOMB_COOLDOWN / self.stats.bomb_level as f32;
    }

    fn use_bomb(&mut self, world: &mut World) {
        let mut rng = rand::rng();
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(BOMB_RADIUS * 0.3..BOMB_RADIUS * 0.8);

        let target_position = Vec3::new(
            self.player_position.x + angle.cos() * distance,
            0.5,
            self.player_position.z + angle.sin() * distance,
        );

        let bomb_entity = spawn_mesh(
            world,
            "Sphere",
            self.player_position,
            Vec3::new(0.5, 0.5, 0.5),
        );

        let bomb_material_name = format!("LobBomb_{}", bomb_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            bomb_material_name.clone(),
            Material {
                base_color: [0.05, 0.05, 0.05, 1.0],
                roughness: 0.3,
                metallic: 0.9,
                ..Default::default()
            },
        );
        self.apply_material(world, bomb_entity, &bomb_material_name);

        let fuse_entity = spawn_mesh(
            world,
            "Cylinder",
            Vec3::new(0.0, 0.3, 0.0),
            Vec3::new(0.08, 0.25, 0.08),
        );
        world.core.set_parent(fuse_entity, Parent(Some(bomb_entity)));

        let fuse_material_name = format!("LobBombFuse_{}", fuse_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            fuse_material_name.clone(),
            Material {
                base_color: [0.15, 0.1, 0.05, 1.0],
                roughness: 0.8,
                metallic: 0.1,
                ..Default::default()
            },
        );
        self.apply_material(world, fuse_entity, &fuse_material_name);

        let fuse_emitter_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
        let fuse_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.9, 0.4, 1.0)),
                (0.2, Vec4::new(1.0, 0.6, 0.2, 0.9)),
                (0.5, Vec4::new(0.8, 0.3, 0.1, 0.6)),
                (1.0, Vec4::new(0.3, 0.15, 0.05, 0.0)),
            ],
        };
        let fuse_emitter = ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Point,
            position: self.player_position + Vec3::new(0.0, 0.45, 0.0),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 40.0,
            burst_count: 0,
            particle_lifetime_min: 0.15,
            particle_lifetime_max: 0.35,
            initial_velocity_min: 0.5,
            initial_velocity_max: 2.0,
            velocity_spread: 0.6,
            gravity: Vec3::new(0.0, 1.0, 0.0),
            drag: 0.5,
            size_start: 0.1,
            size_end: 0.02,
            color_gradient: fuse_gradient,
            emissive_strength: 12.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.4,
            turbulence_frequency: 5.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(fuse_emitter_entity, fuse_emitter);

        let smoke_emitter_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
        let smoke_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.4, 0.4, 0.4, 0.6)),
                (0.5, Vec4::new(0.3, 0.3, 0.3, 0.3)),
                (1.0, Vec4::new(0.2, 0.2, 0.2, 0.0)),
            ],
        };
        let smoke_emitter = ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Point,
            position: self.player_position + Vec3::new(0.0, 0.45, 0.0),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 15.0,
            burst_count: 0,
            particle_lifetime_min: 0.3,
            particle_lifetime_max: 0.6,
            initial_velocity_min: 0.3,
            initial_velocity_max: 1.0,
            velocity_spread: 0.5,
            gravity: Vec3::new(0.0, 0.5, 0.0),
            drag: 0.3,
            size_start: 0.08,
            size_end: 0.2,
            color_gradient: smoke_gradient,
            emissive_strength: 0.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.3,
            turbulence_frequency: 2.0,

            ..Default::default()
        };
        world.core.set_particle_emitter(smoke_emitter_entity, smoke_emitter);

        let lob_bomb = LobBomb {
            entity: bomb_entity,
            start_position: self.player_position,
            target_position,
            flight_time: 0.8,
            elapsed: 0.0,
            arc_height: 6.0,
            trail_emitter: Some(fuse_emitter_entity),
            fuse_emitter: Some(smoke_emitter_entity),
        };
        self.lob_bombs.push(lob_bomb);
    }

    fn detonate_bomb(&mut self, world: &mut World, position: Vec3) {
        self.camera_shake = 2.0;
        self.spawn_bomb_effect(world, position);

        let enemies: Vec<freecs::Entity> = self
            .game_world
            .query_entities(ENEMY | POSITION | ENTITY_HANDLE)
            .collect();

        let mut enemies_to_kill = Vec::new();

        for game_entity in enemies {
            let enemy_position = self.game_world.get_position(game_entity).unwrap();
            let distance = nalgebra_glm::distance(&position, &enemy_position.0);

            if distance < BOMB_RADIUS {
                let enemy = self.game_world.get_enemy(game_entity).unwrap();
                enemies_to_kill.push((
                    game_entity,
                    enemy_position.0,
                    enemy.xp_value,
                    enemy.enemy_type,
                ));
            }
        }

        for (game_entity, pos, xp, enemy_type) in enemies_to_kill {
            let is_boss = enemy_type == EnemyType::Boss;
            self.spawn_damage_popup(
                world,
                pos,
                BOMB_DAMAGE,
                Vec4::new(1.0, 0.6, 0.1, 1.0),
                is_boss,
            );
            if is_boss {
                self.game_world.resources.boss_alive = false;
                self.spawn_boss_death_effect(world, pos);
            }
            self.spawn_death_particles_for_type(world, pos, enemy_type);
            self.spawn_gem_with_xp(world, pos, xp);
            self.maybe_spawn_enemy_health_drop(world, pos, enemy_type);
            self.add_kill(world, is_boss);

            if let Some(handle) = self.game_world.get_entity_handle(game_entity) {
                world.queue_command(WorldCommand::DespawnRecursive { entity: handle.0 });
            }
            self.game_world.despawn_entities(&[game_entity]);
            self.game_world
                .resources
                .enemy_list
                .retain(|&e| e != game_entity);
            self.game_world.resources.enemies_killed += 1;
        }
    }

    fn update_lob_bombs(&mut self, world: &mut World, delta: f32) {
        let mut exploded_indices = Vec::new();

        for (index, bomb) in self.lob_bombs.iter_mut().enumerate() {
            bomb.elapsed += delta;
            let progress = (bomb.elapsed / bomb.flight_time).clamp(0.0, 1.0);

            let horizontal_position = bomb.start_position.lerp(&bomb.target_position, progress);
            let arc = 4.0 * bomb.arc_height * progress * (1.0 - progress);
            let current_position = Vec3::new(
                horizontal_position.x,
                horizontal_position.y + arc,
                horizontal_position.z,
            );

            let spin = progress * std::f32::consts::TAU * 2.0;
            if let Some(transform) = world.core.get_local_transform_mut(bomb.entity) {
                transform.translation = current_position;
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(spin, &Vec3::new(0.3, 1.0, 0.2).normalize());
            }
            mark_local_transform_dirty(world, bomb.entity);

            let fuse_tip_position = current_position + Vec3::new(0.0, 0.45, 0.0);

            if let Some(trail_entity) = bomb.trail_emitter
                && let Some(emitter) = world.core.get_particle_emitter_mut(trail_entity)
            {
                emitter.position = fuse_tip_position;
            }

            if let Some(fuse_emitter) = bomb.fuse_emitter
                && let Some(emitter) = world.core.get_particle_emitter_mut(fuse_emitter)
            {
                emitter.position = fuse_tip_position;
            }

            if progress >= 1.0 {
                exploded_indices.push(index);
            }
        }

        for index in exploded_indices.into_iter().rev() {
            let bomb = self.lob_bombs.remove(index);
            world.queue_command(WorldCommand::DespawnRecursive {
                entity: bomb.entity,
            });
            if let Some(trail_entity) = bomb.trail_emitter
                && let Some(emitter) = world.core.get_particle_emitter_mut(trail_entity)
            {
                emitter.enabled = false;
                emitter.one_shot = true;
            }
            if let Some(fuse_emitter) = bomb.fuse_emitter
                && let Some(emitter) = world.core.get_particle_emitter_mut(fuse_emitter)
            {
                emitter.enabled = false;
                emitter.one_shot = true;
            }
            self.detonate_bomb(world, bomb.target_position);
        }
    }

    fn spawn_bomb_effect(&mut self, world: &mut World, position: Vec3) {
        let ring_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let ring_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 0.8, 1.0)),
                (0.2, Vec4::new(1.0, 0.8, 0.3, 1.0)),
                (0.5, Vec4::new(1.0, 0.4, 0.1, 0.8)),
                (0.8, Vec4::new(0.9, 0.2, 0.0, 0.4)),
                (1.0, Vec4::new(0.6, 0.1, 0.0, 0.0)),
            ],
        };

        let ring_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere {
                radius: BOMB_RADIUS * 0.95,
            },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 400,
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.2,
            initial_velocity_min: BOMB_RADIUS * 0.8,
            initial_velocity_max: BOMB_RADIUS * 1.5,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -6.0, 0.0),
            drag: 0.3,
            size_start: 0.6,
            size_end: 0.1,
            color_gradient: ring_gradient,
            emissive_strength: 40.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.7,
            turbulence_frequency: 4.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(ring_entity, ring_emitter);

        let core_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let core_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.15, Vec4::new(1.0, 1.0, 0.8, 1.0)),
                (0.4, Vec4::new(1.0, 0.7, 0.3, 0.9)),
                (0.7, Vec4::new(1.0, 0.4, 0.1, 0.6)),
                (1.0, Vec4::new(0.8, 0.2, 0.0, 0.0)),
            ],
        };

        let core_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 200,
            particle_lifetime_min: 0.4,
            particle_lifetime_max: 0.8,
            initial_velocity_min: 15.0,
            initial_velocity_max: 30.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -8.0, 0.0),
            drag: 0.25,
            size_start: 0.5,
            size_end: 0.15,
            color_gradient: core_gradient,
            emissive_strength: 50.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.9,
            turbulence_frequency: 5.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(core_entity, core_emitter);

        let smoke_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let smoke_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.3, 0.2, 0.15, 0.7)),
                (0.3, Vec4::new(0.2, 0.15, 0.1, 0.5)),
                (0.7, Vec4::new(0.15, 0.1, 0.08, 0.3)),
                (1.0, Vec4::new(0.1, 0.08, 0.05, 0.0)),
            ],
        };

        let smoke_emitter = ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Sphere {
                radius: BOMB_RADIUS * 0.5,
            },
            position: position + Vec3::new(0.0, 0.5, 0.0),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 80,
            particle_lifetime_min: 0.8,
            particle_lifetime_max: 1.5,
            initial_velocity_min: 3.0,
            initial_velocity_max: 8.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 2.0, 0.0),
            drag: 0.4,
            size_start: 1.5,
            size_end: 4.0,
            color_gradient: smoke_gradient,
            emissive_strength: 0.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.4,
            turbulence_frequency: 1.5,

            ..Default::default()
        };

        world.core.set_particle_emitter(smoke_entity, smoke_emitter);

        self.spawn_bomb_lines(world, position);
    }

    fn spawn_bomb_lines(&mut self, world: &mut World, position: Vec3) {
        for ring_index in 0..5 {
            let line_entity = world.spawn_entities(
                LINES | VISIBILITY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
                1,
            )[0];

            let delay_factor = ring_index as f32 * 0.08;
            let start_radius = BOMB_RADIUS * 0.05;
            let end_radius = BOMB_RADIUS * (1.3 + ring_index as f32 * 0.25);

            let intensity = 1.0 - ring_index as f32 * 0.15;
            let effect = LineEffect {
                entity: line_entity,
                timer: -delay_factor,
                max_time: 0.6,
                center: Vec3::new(position.x, 0.15, position.z),
                start_radius,
                end_radius,
                segments: 48,
                color_start: Vec4::new(
                    1.0,
                    0.6 * intensity,
                    0.2 * intensity,
                    0.95 - ring_index as f32 * 0.1,
                ),
                color_end: Vec4::new(0.8, 0.2, 0.0, 0.0),
            };

            self.line_effects.push(effect);
        }
    }

    fn spawn_enemy_spawn_effect(
        &mut self,
        world: &mut World,
        position: Vec3,
        enemy_type: EnemyType,
    ) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let (color, burst, emissive) = match enemy_type {
            EnemyType::Normal => (Vec4::new(0.9, 0.3, 0.3, 1.0), 20, 8.0),
            EnemyType::Fast => (Vec4::new(1.0, 0.7, 0.2, 1.0), 25, 10.0),
            EnemyType::Tank => (Vec4::new(0.5, 0.3, 0.6, 1.0), 35, 12.0),
            EnemyType::Exploder => (Vec4::new(0.3, 0.9, 0.3, 1.0), 30, 10.0),
            EnemyType::Boss => (Vec4::new(1.0, 0.2, 0.2, 1.0), 80, 20.0),
        };

        let spawn_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.2, color),
                (
                    0.6,
                    Vec4::new(color.x * 0.7, color.y * 0.7, color.z * 0.7, 0.6),
                ),
                (
                    1.0,
                    Vec4::new(color.x * 0.3, color.y * 0.3, color.z * 0.3, 0.0),
                ),
            ],
        };

        let spawn_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.3 },
            position: Vec3::new(position.x, 0.1, position.z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: burst,
            particle_lifetime_min: 0.3,
            particle_lifetime_max: 0.6,
            initial_velocity_min: 2.0,
            initial_velocity_max: 5.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 3.0, 0.0),
            drag: 0.3,
            size_start: 0.15,
            size_end: 0.03,
            color_gradient: spawn_gradient,
            emissive_strength: emissive,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 3.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, spawn_emitter);

        let ring_entity = world.spawn_entities(
            LINES | VISIBILITY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
            1,
        )[0];

        let portal_radius = match enemy_type {
            EnemyType::Boss => 1.5,
            EnemyType::Tank => 0.8,
            _ => 0.5,
        };

        let effect = LineEffect {
            entity: ring_entity,
            timer: 0.0,
            max_time: 0.4,
            center: Vec3::new(position.x, 0.05, position.z),
            start_radius: portal_radius * 1.5,
            end_radius: 0.0,
            segments: 24,
            color_start: color,
            color_end: Vec4::new(color.x, color.y, color.z, 0.0),
        };

        self.line_effects.push(effect);
    }

    fn spawn_enemy_shield(
        &mut self,
        world: &mut World,
        game_entity: freecs::Entity,
        engine_entity: Entity,
        radius: f32,
    ) {
        let shield_radius = radius * 1.4;

        let shield_entity = spawn_mesh(
            world,
            "Sphere",
            Vec3::zeros(),
            Vec3::new(
                shield_radius * 2.0,
                shield_radius * 2.0,
                shield_radius * 2.0,
            ),
        );

        world.core.set_parent(shield_entity, Parent(Some(engine_entity)));

        let shield_material_name = format!("EnemyShield_{}", shield_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            shield_material_name.clone(),
            Material {
                base_color: [1.0, 0.3, 0.2, 0.4],
                roughness: 0.1,
                metallic: 0.0,
                emissive_factor: [0.5, 0.15, 0.1],
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        );
        self.apply_material(world, shield_entity, &shield_material_name);

        self.enemy_shield_entities
            .push((game_entity, shield_entity, shield_radius));
    }

    fn update_enemy_shields(&mut self, world: &mut World) {
        let mut to_remove = Vec::new();

        for (index, (game_entity, shield_entity, base_radius)) in
            self.enemy_shield_entities.iter().enumerate()
        {
            if let Some(enemy) = self.game_world.get_enemy(*game_entity) {
                if enemy.shield_hits == 0 {
                    world.queue_command(WorldCommand::DespawnRecursive {
                        entity: *shield_entity,
                    });
                    to_remove.push(index);
                } else {
                    let pulse = (self.game_time * 2.0).sin() * 0.05 + 1.0;
                    let shield_scale = base_radius * 2.0 * pulse;
                    if let Some(transform) = world.core.get_local_transform_mut(*shield_entity) {
                        transform.scale = Vec3::new(shield_scale, shield_scale, shield_scale);
                        transform.rotation = nalgebra_glm::quat_angle_axis(
                            self.game_time * 0.5,
                            &Vec3::new(0.0, 1.0, 0.0),
                        );
                    }
                    mark_local_transform_dirty(world, *shield_entity);
                }
            } else {
                world.queue_command(WorldCommand::DespawnRecursive {
                    entity: *shield_entity,
                });
                to_remove.push(index);
            }
        }

        for index in to_remove.into_iter().rev() {
            self.enemy_shield_entities.remove(index);
        }
    }

    fn spawn_enemy_shield_break_effect(&mut self, world: &mut World, position: Vec3) {
        let effect_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let shield_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.3, Vec4::new(1.0, 0.4, 0.3, 0.8)),
                (1.0, Vec4::new(0.5, 0.1, 0.05, 0.0)),
            ],
        };

        let shield_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.5 },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 40,
            particle_lifetime_min: 0.3,
            particle_lifetime_max: 0.6,
            initial_velocity_min: 3.0,
            initial_velocity_max: 6.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -2.0, 0.0),
            drag: 0.5,
            size_start: 0.12,
            size_end: 0.02,
            color_gradient: shield_gradient,
            emissive_strength: 12.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.3,
            turbulence_frequency: 3.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(effect_entity, shield_emitter);

        self.line_effects.push(LineEffect {
            entity: world.spawn_entities(LINES, 1)[0],
            timer: 0.0,
            max_time: 0.25,
            center: position,
            start_radius: 0.6,
            end_radius: 1.2,
            segments: 16,
            color_start: Vec4::new(1.0, 0.4, 0.3, 1.0),
            color_end: Vec4::new(0.5, 0.1, 0.05, 0.0),
        });
    }

    fn spawn_dust_particle(&self, world: &mut World, position: Vec3) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let dust_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.6, 0.5, 0.4, 0.4)),
                (0.5, Vec4::new(0.5, 0.45, 0.35, 0.2)),
                (1.0, Vec4::new(0.4, 0.35, 0.3, 0.0)),
            ],
        };

        let dust_emitter = ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Point,
            position: Vec3::new(position.x, 0.1, position.z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 3,
            particle_lifetime_min: 0.2,
            particle_lifetime_max: 0.4,
            initial_velocity_min: 0.3,
            initial_velocity_max: 0.8,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 0.2, 0.0),
            drag: 0.5,
            size_start: 0.08,
            size_end: 0.15,
            color_gradient: dust_gradient,
            emissive_strength: 0.5,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.2,
            turbulence_frequency: 2.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, dust_emitter);
    }

    fn update_combo(&mut self, delta: f32) {
        if self.combo_count > 0 {
            self.combo_timer += delta;
            if self.combo_timer >= COMBO_DECAY_TIME {
                if self.combo_count > self.combo_max {
                    self.combo_max = self.combo_count;
                }
                self.combo_count = 0;
                self.combo_timer = 0.0;
            }
        }
    }

    fn add_kill(&mut self, world: &mut World, is_boss: bool) {
        self.combo_count += 1;
        self.combo_timer = 0.0;
        self.speed_boost_timer = SPEED_BOOST_DURATION;

        if self.combo_count >= 10 && self.combo_count.is_multiple_of(10) {
            self.spawn_combo_milestone_effect(world);
            let combo_color = if self.combo_count >= 50 {
                Vec4::new(1.0, 0.4, 1.0, 1.0)
            } else if self.combo_count >= 30 {
                Vec4::new(1.0, 0.8, 0.2, 1.0)
            } else {
                Vec4::new(1.0, 0.9, 0.4, 1.0)
            };
            self.spawn_popup_typed(
                world,
                self.player_position + Vec3::new(0.0, 4.0, 0.0),
                format!("{}x COMBO!", self.combo_count),
                combo_color,
                PopupType::Combo,
            );
        }

        if is_boss {
            self.camera_shake = 3.0;
            self.kill_flash_timer = 0.3;
        } else if self.combo_count >= 5 {
            self.kill_flash_timer = 0.1;
        }
    }

    fn spawn_combo_milestone_effect(&self, world: &mut World) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let combo_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 0.5, 1.0)),
                (0.3, Vec4::new(1.0, 0.8, 0.2, 0.9)),
                (0.6, Vec4::new(1.0, 0.5, 0.1, 0.6)),
                (1.0, Vec4::new(1.0, 0.3, 0.0, 0.0)),
            ],
        };

        let combo_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 1.0 },
            position: self.player_position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 40,
            particle_lifetime_min: 0.4,
            particle_lifetime_max: 0.8,
            initial_velocity_min: 5.0,
            initial_velocity_max: 10.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -2.0, 0.0),
            drag: 0.3,
            size_start: 0.12,
            size_end: 0.03,
            color_gradient: combo_gradient,
            emissive_strength: 15.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 3.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, combo_emitter);
    }

    fn update_speed_boost(&mut self, delta: f32) {
        if self.speed_boost_timer > 0.0 {
            self.speed_boost_timer -= delta;
        }
    }

    fn update_kill_flash(&mut self, delta: f32) {
        if self.kill_flash_timer > 0.0 {
            self.kill_flash_timer -= delta;
        }
    }

    fn update_flashes(&mut self, delta: f32) {
        if self.level_up_flash > 0.0 {
            self.level_up_flash = (self.level_up_flash - delta * 3.0).max(0.0);
        }
        if self.boss_kill_flash > 0.0 {
            self.boss_kill_flash = (self.boss_kill_flash - delta * 2.0).max(0.0);
        }
        if self.new_high_score_timer > 0.0 {
            self.new_high_score_timer = (self.new_high_score_timer - delta).max(0.0);
            self.score_popup_scale = 1.0 + (self.new_high_score_timer * 8.0).sin().abs() * 0.3;
        }
    }

    fn check_high_scores(&mut self) {
        let kills = self.game_world.resources.enemies_killed;
        let wave = self.game_world.resources.current_wave;
        let time = self.game_time;
        let combo = self.combo_max;

        let mut new_record = false;

        if kills > self.high_score_kills {
            self.high_score_kills = kills;
            self.new_high_score_type = HighScoreType::Kills;
            new_record = true;
        }
        if wave > self.high_score_wave {
            self.high_score_wave = wave;
            if !new_record {
                self.new_high_score_type = HighScoreType::Wave;
            }
            new_record = true;
        }
        if time > self.high_score_time {
            self.high_score_time = time;
            if !new_record {
                self.new_high_score_type = HighScoreType::Time;
            }
            new_record = true;
        }
        if combo > self.high_score_combo {
            self.high_score_combo = combo;
            if !new_record {
                self.new_high_score_type = HighScoreType::Combo;
            }
            new_record = true;
        }

        if new_record {
            self.new_high_score_timer = 5.0;
            self.score_popup_scale = 1.5;
        }
    }

    fn update_combo_fire(&mut self, world: &mut World) {
        let should_have_fire = self.combo_count >= 10 && self.game_state == GameState::Playing;

        if should_have_fire && self.combo_emitter.is_none() {
            let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
            let fire_gradient = ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(1.0, 0.9, 0.3, 0.9)),
                    (0.3, Vec4::new(1.0, 0.5, 0.1, 0.7)),
                    (0.6, Vec4::new(0.9, 0.2, 0.0, 0.4)),
                    (1.0, Vec4::new(0.4, 0.1, 0.0, 0.0)),
                ],
            };

            let intensity = (self.combo_count as f32 / 20.0).min(2.0);
            let fire_emitter = ParticleEmitter {
                emitter_type: EmitterType::Fire,
                shape: EmitterShape::Sphere {
                    radius: PLAYER_RADIUS * 1.2,
                },
                position: self.player_position,
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 30.0 * intensity,
                burst_count: 0,
                particle_lifetime_min: 0.2,
                particle_lifetime_max: 0.5,
                initial_velocity_min: 1.0,
                initial_velocity_max: 3.0 * intensity,
                velocity_spread: 0.8,
                gravity: Vec3::new(0.0, 4.0, 0.0),
                drag: 0.3,
                size_start: 0.15 * intensity,
                size_end: 0.05,
                color_gradient: fire_gradient,
                emissive_strength: 10.0 * intensity,
                enabled: true,
                accumulated_spawn: 0.0,
                one_shot: false,
                has_fired: false,
                turbulence_strength: 0.5,
                turbulence_frequency: 4.0,

                ..Default::default()
            };
            world.core.set_particle_emitter(particle_entity, fire_emitter);
            self.combo_emitter = Some(particle_entity);
        } else if !should_have_fire && self.combo_emitter.is_some() {
            if let Some(emitter) = self.combo_emitter.take() {
                world.queue_command(WorldCommand::DespawnRecursive { entity: emitter });
            }
        } else if let Some(emitter_entity) = self.combo_emitter
            && let Some(emitter) = world.core.get_particle_emitter_mut(emitter_entity)
        {
            emitter.position = self.player_position;
            let intensity = (self.combo_count as f32 / 20.0).min(2.0);
            emitter.spawn_rate = 30.0 * intensity;
            emitter.emissive_strength = 10.0 * intensity;
        }
    }

    fn check_wave_announcement(&mut self, world: &mut World) {
        let current_wave = self.game_world.resources.current_wave;
        if current_wave > self.last_wave_announced && current_wave > 0 {
            self.last_wave_announced = current_wave;
            self.spawn_wave_announcement(world, current_wave);
        }
    }

    fn spawn_wave_announcement(&mut self, world: &mut World, wave: u32) {
        let is_boss_wave = wave.is_multiple_of(BOSS_WAVE_INTERVAL);

        let text = if is_boss_wave {
            format!("WAVE {} - BOSS!", wave)
        } else {
            format!("WAVE {}", wave)
        };

        let color = if is_boss_wave {
            Vec4::new(1.0, 0.15, 0.15, 1.0)
        } else {
            Vec4::new(1.0, 0.95, 0.4, 1.0)
        };

        self.spawn_popup_typed(
            world,
            self.player_position + Vec3::new(0.0, 5.0, 0.0),
            text,
            color,
            PopupType::Wave,
        );

        if is_boss_wave {
            self.camera_shake = 1.5;
            self.spawn_wave_effect(world, true);
        } else if wave > 1 {
            self.spawn_wave_effect(world, false);
        }
    }

    fn spawn_wave_effect(&self, world: &mut World, is_boss: bool) {
        let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let color = if is_boss {
            Vec4::new(1.0, 0.3, 0.2, 1.0)
        } else {
            Vec4::new(0.3, 0.8, 1.0, 1.0)
        };

        let wave_gradient = ColorGradient {
            colors: vec![
                (0.0, color),
                (
                    0.5,
                    Vec4::new(color.x * 0.7, color.y * 0.7, color.z * 0.7, 0.6),
                ),
                (
                    1.0,
                    Vec4::new(color.x * 0.4, color.y * 0.4, color.z * 0.4, 0.0),
                ),
            ],
        };

        let wave_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 3.0 },
            position: self.player_position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: if is_boss { 80 } else { 40 },
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.0,
            initial_velocity_min: 3.0,
            initial_velocity_max: 8.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 2.0, 0.0),
            drag: 0.3,
            size_start: 0.15,
            size_end: 0.05,
            color_gradient: wave_gradient,
            emissive_strength: if is_boss { 15.0 } else { 8.0 },
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.4,
            turbulence_frequency: 2.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(particle_entity, wave_emitter);
    }

    fn spawn_wave_complete_effect(&mut self, world: &mut World) {
        let wave = self.game_world.resources.current_wave;
        if wave == 0 {
            return;
        }

        for index in 0..8 {
            let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

            let hue = index as f32 / 8.0;
            let (r, g, b) = hsv_to_rgb(hue, 0.8, 1.0);

            let confetti_gradient = ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(r, g, b, 1.0)),
                    (0.5, Vec4::new(r * 0.8, g * 0.8, b * 0.8, 0.8)),
                    (1.0, Vec4::new(r * 0.5, g * 0.5, b * 0.5, 0.0)),
                ],
            };

            let angle = (index as f32 / 8.0) * std::f32::consts::TAU;
            let offset = Vec3::new(angle.cos() * 2.0, 0.0, angle.sin() * 2.0);

            let confetti_emitter = ParticleEmitter {
                emitter_type: EmitterType::Sparks,
                shape: EmitterShape::Point,
                position: self.player_position + offset,
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 0.0,
                burst_count: 20,
                particle_lifetime_min: 1.0,
                particle_lifetime_max: 2.0,
                initial_velocity_min: 5.0,
                initial_velocity_max: 12.0,
                velocity_spread: 0.6,
                gravity: Vec3::new(0.0, -4.0, 0.0),
                drag: 0.2,
                size_start: 0.15,
                size_end: 0.05,
                color_gradient: confetti_gradient,
                emissive_strength: 8.0,
                enabled: true,
                accumulated_spawn: 0.0,
                one_shot: true,
                has_fired: false,
                turbulence_strength: 0.5,
                turbulence_frequency: 3.0,

                ..Default::default()
            };

            world.core.set_particle_emitter(particle_entity, confetti_emitter);
        }

        for ring_index in 0..3 {
            let line_entity = world.spawn_entities(
                LINES | VISIBILITY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
                1,
            )[0];

            let effect = LineEffect {
                entity: line_entity,
                timer: -ring_index as f32 * 0.1,
                max_time: 0.5,
                center: Vec3::new(self.player_position.x, 0.1, self.player_position.z),
                start_radius: 0.5,
                end_radius: 5.0 + ring_index as f32 * 2.0,
                segments: 32,
                color_start: Vec4::new(0.3, 1.0, 0.5, 0.8),
                color_end: Vec4::new(0.2, 0.8, 0.4, 0.0),
            };
            self.line_effects.push(effect);
        }
    }

    fn update_ambient_particles(&mut self, world: &mut World) {
        if self.ambient_emitter.is_none() {
            let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
            self.ambient_emitter = Some(particle_entity);

            let ambient_gradient = ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(0.8, 0.9, 1.0, 0.0)),
                    (0.3, Vec4::new(0.7, 0.85, 0.95, 0.15)),
                    (0.7, Vec4::new(0.6, 0.8, 0.9, 0.15)),
                    (1.0, Vec4::new(0.5, 0.7, 0.85, 0.0)),
                ],
            };

            let ambient_emitter = ParticleEmitter {
                emitter_type: EmitterType::Smoke,
                shape: EmitterShape::Box {
                    half_extents: Vec3::new(ARENA_SIZE * 0.4, 0.5, ARENA_SIZE * 0.4),
                },
                position: Vec3::new(0.0, 0.5, 0.0),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 8.0,
                burst_count: 0,
                particle_lifetime_min: 3.0,
                particle_lifetime_max: 5.0,
                initial_velocity_min: 0.1,
                initial_velocity_max: 0.4,
                velocity_spread: 1.0,
                gravity: Vec3::new(0.0, 0.1, 0.0),
                drag: 0.1,
                size_start: 0.3,
                size_end: 0.6,
                color_gradient: ambient_gradient,
                emissive_strength: 1.0,
                enabled: true,
                accumulated_spawn: 0.0,
                one_shot: false,
                has_fired: false,
                turbulence_strength: 0.2,
                turbulence_frequency: 0.5,

                ..Default::default()
            };

            world.core.set_particle_emitter(particle_entity, ambient_emitter);
        }
    }

    fn spawn_boss_death_effect(&mut self, world: &mut World, position: Vec3) {
        self.boss_kill_flash = 1.0;
        self.camera_shake = 2.5;

        for index in 0..5 {
            let particle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

            let delay_offset = index as f32 * 0.08;
            let radius = (index as f32 + 1.0) * 3.0;

            let boss_gradient = ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(1.0, 1.0, 0.9, 1.0)),
                    (0.15, Vec4::new(1.0, 0.9, 0.5, 1.0)),
                    (0.35, Vec4::new(1.0, 0.6, 0.2, 0.9)),
                    (0.6, Vec4::new(1.0, 0.3, 0.1, 0.6)),
                    (0.85, Vec4::new(0.8, 0.1, 0.0, 0.3)),
                    (1.0, Vec4::new(0.3, 0.0, 0.0, 0.0)),
                ],
            };

            let boss_emitter = ParticleEmitter {
                emitter_type: EmitterType::Sparks,
                shape: EmitterShape::Sphere { radius },
                position: position + Vec3::new(0.0, delay_offset * 3.0, 0.0),
                direction: Vec3::new(0.0, 1.0, 0.0),
                spawn_rate: 0.0,
                burst_count: 150,
                particle_lifetime_min: 0.6 + delay_offset,
                particle_lifetime_max: 1.5 + delay_offset,
                initial_velocity_min: radius * 3.0,
                initial_velocity_max: radius * 6.0,
                velocity_spread: 1.0,
                gravity: Vec3::new(0.0, -6.0, 0.0),
                drag: 0.25,
                size_start: 0.7,
                size_end: 0.1,
                color_gradient: boss_gradient,
                emissive_strength: 45.0,
                enabled: true,
                accumulated_spawn: 0.0,
                one_shot: true,
                has_fired: false,
                turbulence_strength: 0.8,
                turbulence_frequency: 4.0,

                ..Default::default()
            };

            world.core.set_particle_emitter(particle_entity, boss_emitter);
        }

        let shockwave_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let shockwave_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.2, Vec4::new(1.0, 0.9, 0.6, 0.8)),
                (0.5, Vec4::new(1.0, 0.6, 0.2, 0.5)),
                (0.8, Vec4::new(1.0, 0.3, 0.1, 0.2)),
                (1.0, Vec4::new(0.5, 0.1, 0.0, 0.0)),
            ],
        };

        let shockwave_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            position,
            direction: Vec3::new(0.0, 0.0, 1.0),
            spawn_rate: 0.0,
            burst_count: 300,
            particle_lifetime_min: 0.4,
            particle_lifetime_max: 0.7,
            initial_velocity_min: 25.0,
            initial_velocity_max: 40.0,
            velocity_spread: 0.15,
            gravity: Vec3::new(0.0, -1.0, 0.0),
            drag: 0.4,
            size_start: 0.5,
            size_end: 1.0,
            color_gradient: shockwave_gradient,
            emissive_strength: 50.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.2,
            turbulence_frequency: 2.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(shockwave_entity, shockwave_emitter);

        let core_flash_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let core_flash_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                (0.1, Vec4::new(1.0, 1.0, 0.9, 1.0)),
                (0.3, Vec4::new(1.0, 0.8, 0.4, 0.8)),
                (0.6, Vec4::new(1.0, 0.5, 0.2, 0.4)),
                (1.0, Vec4::new(0.8, 0.2, 0.0, 0.0)),
            ],
        };

        let core_flash_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            position: position + Vec3::new(0.0, 1.0, 0.0),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 200,
            particle_lifetime_min: 0.3,
            particle_lifetime_max: 0.6,
            initial_velocity_min: 20.0,
            initial_velocity_max: 40.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -10.0, 0.0),
            drag: 0.2,
            size_start: 0.8,
            size_end: 0.15,
            color_gradient: core_flash_gradient,
            emissive_strength: 60.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 1.0,
            turbulence_frequency: 5.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(core_flash_entity, core_flash_emitter);

        let debris_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];

        let debris_gradient = ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.8, 0.3, 0.1, 1.0)),
                (0.3, Vec4::new(0.6, 0.2, 0.1, 0.9)),
                (0.7, Vec4::new(0.3, 0.1, 0.05, 0.6)),
                (1.0, Vec4::new(0.1, 0.05, 0.02, 0.0)),
            ],
        };

        let debris_emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 3.0 },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: 100,
            particle_lifetime_min: 0.8,
            particle_lifetime_max: 1.8,
            initial_velocity_min: 10.0,
            initial_velocity_max: 25.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -15.0, 0.0),
            drag: 0.15,
            size_start: 0.4,
            size_end: 0.2,
            color_gradient: debris_gradient,
            emissive_strength: 12.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: true,
            has_fired: false,
            turbulence_strength: 0.5,
            turbulence_frequency: 3.0,

            ..Default::default()
        };

        world.core.set_particle_emitter(debris_entity, debris_emitter);
    }
}
