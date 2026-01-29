use nightshade::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharacterMovementState {
    #[default]
    Idle,
    Walking,
    Jumping,
    Chopping,
}

#[derive(Debug, Clone, Default)]
pub struct FarmingAnimationIndices {
    pub idle: Option<usize>,
    pub walk: Option<usize>,
    pub pick_fruit: Option<usize>,
    pub plant: Option<usize>,
    pub watering: Option<usize>,
    pub pull_plant: Option<usize>,
    pub dig_and_plant: Option<usize>,
    pub kneeling_idle: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyType {
    #[default]
    Normal,
    Fast,
    Tank,
    Exploder,
    Boss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeState {
    #[default]
    Standing,
    BeingChopped,
    Falling,
    Shrinking,
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
    PowerUp,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    LevelUp,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    #[default]
    TopDown,
    ThirdPerson,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UpgradeType {
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
    pub fn base_name(&self) -> &'static str {
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

    pub fn tier_name(&self, level: u32) -> String {
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

    pub fn description(&self, level: u32) -> String {
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

    pub fn max_level(&self) -> u32 {
        match self {
            UpgradeType::ProjectileCount => 4,
            UpgradeType::OrbitingOrbs => 3,
            UpgradeType::Bomb => 3,
            UpgradeType::Shield => 5,
            _ => MAX_UPGRADE_LEVEL,
        }
    }

    pub fn tier_color(&self, level: u32) -> Vec4 {
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

#[derive(Clone, Copy, PartialEq, Default)]
pub enum HighScoreType {
    #[default]
    None,
    Kills,
    Wave,
    Time,
    Combo,
}

#[derive(Clone)]
pub struct PlayerStats {
    pub damage_multiplier: f32,
    pub cooldown_multiplier: f32,
    pub projectile_count: u32,
    pub range_multiplier: f32,
    pub speed_multiplier: f32,
    pub max_health: f32,
    pub orb_count: u32,
    pub area_pulse_level: u32,
    pub magnet_multiplier: f32,
    pub regen_level: u32,
    pub whip_level: u32,
    pub lightning_level: u32,
    pub garlic_level: u32,
    pub shield_level: u32,
    pub damage_level: u32,
    pub fire_rate_level: u32,
    pub projectile_level: u32,
    pub range_level: u32,
    pub speed_level: u32,
    pub health_level: u32,
    pub magnetism_level: u32,
    pub bomb_level: u32,
    pub buff_damage_multiplier: f32,
    pub buff_speed_multiplier: f32,
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
    pub fn get_upgrade_level(&self, upgrade: UpgradeType) -> u32 {
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

    pub fn is_maxed(&self, upgrade: UpgradeType) -> bool {
        self.get_upgrade_level(upgrade) >= upgrade.max_level()
    }
}

pub struct LineEffect {
    pub entity: Entity,
    pub timer: f32,
    pub max_time: f32,
    pub center: Vec3,
    pub start_radius: f32,
    pub end_radius: f32,
    pub segments: u32,
    pub color_start: Vec4,
    pub color_end: Vec4,
}

pub struct LobBomb {
    pub entity: Entity,
    pub start_position: Vec3,
    pub target_position: Vec3,
    pub flight_time: f32,
    pub elapsed: f32,
    pub arc_height: f32,
    pub trail_emitter: Option<Entity>,
    pub fuse_emitter: Option<Entity>,
}

#[derive(Default, Clone)]
pub struct EnemyMaterials {
    pub normal: Option<String>,
    pub fast: Option<String>,
    pub tank: Option<String>,
    pub exploder: Option<String>,
    pub boss: Option<String>,
}

pub const ARENA_SIZE: f32 = 40.0;
pub const GROUND_SIZE: f32 = 200.0;
pub const CHUNK_SIZE: f32 = 20.0;
pub const RENDER_DISTANCE: i32 = 3;
pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_SPEED: f32 = 8.0;
pub const CAMERA_HEIGHT: f32 = 25.0;
pub const CAMERA_DISTANCE: f32 = 15.0;

pub const ENEMY_RADIUS: f32 = 0.4;
pub const ENEMY_SPEED: f32 = 3.0;
pub const SPAWN_INTERVAL: f32 = 0.5;
pub const COLLISION_DISTANCE: f32 = PLAYER_RADIUS + ENEMY_RADIUS;

pub const PLAYER_MAX_HEALTH: f32 = 100.0;
pub const ENEMY_DAMAGE: f32 = 10.0;
pub const DAMAGE_COOLDOWN: f32 = 0.5;

pub const PROJECTILE_RADIUS: f32 = 0.15;
pub const PROJECTILE_SPEED: f32 = 15.0;
pub const PROJECTILE_COOLDOWN: f32 = 0.3;
pub const PROJECTILE_RANGE: f32 = 20.0;
pub const PROJECTILE_HIT_DISTANCE: f32 = PROJECTILE_RADIUS + ENEMY_RADIUS;

pub const GEM_RADIUS: f32 = 0.2;
pub const GEM_MAGNET_RANGE: f32 = 3.0;
pub const GEM_MAGNET_SPEED: f32 = 12.0;
pub const GEM_COLLECT_DISTANCE: f32 = PLAYER_RADIUS + GEM_RADIUS;

pub const XP_PER_LEVEL: u32 = 100;

pub const ORB_RADIUS: f32 = 0.25;
pub const ORB_ORBIT_RADIUS: f32 = 2.0;
pub const ORB_ORBIT_SPEED: f32 = 3.0;
pub const ORB_DAMAGE: f32 = 25.0;
pub const ORB_HIT_DISTANCE: f32 = ORB_RADIUS + ENEMY_RADIUS;

pub const PULSE_COOLDOWN: f32 = 2.0;
pub const PULSE_RADIUS: f32 = 5.0;
pub const PULSE_BASE_DAMAGE: f32 = 30.0;

pub const REGEN_INTERVAL: f32 = 1.0;
pub const REGEN_AMOUNT: f32 = 2.0;

pub const WHIP_COOLDOWN: f32 = 1.2;
pub const WHIP_RANGE: f32 = 4.0;
pub const WHIP_ARC: f32 = 2.5;
pub const WHIP_DAMAGE: f32 = 20.0;

pub const LIGHTNING_COOLDOWN: f32 = 1.5;
pub const LIGHTNING_RANGE: f32 = 8.0;
pub const LIGHTNING_CHAIN_COUNT: u32 = 3;
pub const LIGHTNING_CHAIN_RANGE: f32 = 4.0;
pub const LIGHTNING_DAMAGE: f32 = 15.0;

pub const GARLIC_RADIUS: f32 = 2.5;
pub const GARLIC_TICK_RATE: f32 = 0.5;
pub const GARLIC_DAMAGE: f32 = 5.0;

pub const BOMB_RADIUS: f32 = 12.0;
pub const BOMB_DAMAGE: f32 = 100.0;
pub const BOMB_COOLDOWN: f32 = 8.0;

pub const INVINCIBILITY_DURATION: f32 = 0.5;
pub const INVINCIBILITY_FLASH_RATE: f32 = 10.0;

pub const DUST_SPAWN_INTERVAL: f32 = 0.08;
pub const COMBO_DECAY_TIME: f32 = 2.0;
pub const SPEED_BOOST_DURATION: f32 = 0.3;
pub const SPEED_BOOST_MULTIPLIER: f32 = 1.3;

pub const WAVE_ENEMIES_BASE: u32 = 20;
pub const BOSS_WAVE_INTERVAL: u32 = 5;
pub const BOSS_HEALTH: f32 = 50.0;
pub const BOSS_SPEED: f32 = 1.5;
pub const BOSS_RADIUS: f32 = 1.2;
pub const BOSS_XP: u32 = 100;

pub const SHIELD_BASE_DURATION: f32 = 8.0;
pub const SHIELD_DURATION_PER_LAYER: f32 = 6.0;
pub const SHIELD_REGEN_DELAY: f32 = 5.0;
pub const SHIELD_RADIUS_BASE: f32 = 1.3;
pub const SHIELD_RADIUS_STEP: f32 = 0.2;

pub const MAX_UPGRADE_LEVEL: u32 = 5;

pub const AXE_SWING_SPEED: f32 = 12.0;
pub const CHOP_RANGE: f32 = 2.5;
pub const LOG_COLLECT_DISTANCE: f32 = 1.5;

pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (f32, f32, f32) {
    let chroma = value * saturation;
    let x = chroma * (1.0 - ((hue * 6.0) % 2.0 - 1.0).abs());
    let match_value = value - chroma;
    let (red, green, blue) = match (hue * 6.0) as i32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    (red + match_value, green + match_value, blue + match_value)
}

pub fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
