use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EnemyType {
    Grunt,
    Archer,
    Brute,
    Mage,
    Boss,
}

#[derive(Clone)]
pub struct EnemyDefinition {
    pub enemy_type: EnemyType,
    pub health: f32,
    pub damage: f32,
    pub speed: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub detection_range: f32,
    pub color: [f32; 4],
    pub scale: f32,
    pub experience_value: u32,
    pub loot_chance: f32,
}

pub const ENEMY_DEFINITIONS: &[EnemyDefinition] = &[
    EnemyDefinition {
        enemy_type: EnemyType::Grunt,
        health: 50.0,
        damage: 10.0,
        speed: 3.0,
        attack_range: 2.0,
        attack_cooldown: 1.5,
        detection_range: 10.0,
        color: [0.6, 0.3, 0.3, 1.0],
        scale: 1.0,
        experience_value: 25,
        loot_chance: 0.3,
    },
    EnemyDefinition {
        enemy_type: EnemyType::Archer,
        health: 35.0,
        damage: 15.0,
        speed: 2.5,
        attack_range: 12.0,
        attack_cooldown: 2.0,
        detection_range: 15.0,
        color: [0.3, 0.5, 0.3, 1.0],
        scale: 0.9,
        experience_value: 35,
        loot_chance: 0.4,
    },
    EnemyDefinition {
        enemy_type: EnemyType::Brute,
        health: 150.0,
        damage: 25.0,
        speed: 1.5,
        attack_range: 2.5,
        attack_cooldown: 2.5,
        detection_range: 8.0,
        color: [0.5, 0.3, 0.2, 1.0],
        scale: 1.5,
        experience_value: 75,
        loot_chance: 0.6,
    },
    EnemyDefinition {
        enemy_type: EnemyType::Mage,
        health: 40.0,
        damage: 30.0,
        speed: 2.0,
        attack_range: 10.0,
        attack_cooldown: 3.0,
        detection_range: 12.0,
        color: [0.4, 0.2, 0.6, 1.0],
        scale: 1.0,
        experience_value: 50,
        loot_chance: 0.5,
    },
    EnemyDefinition {
        enemy_type: EnemyType::Boss,
        health: 500.0,
        damage: 40.0,
        speed: 2.0,
        attack_range: 3.0,
        attack_cooldown: 1.0,
        detection_range: 20.0,
        color: [0.2, 0.0, 0.3, 1.0],
        scale: 2.5,
        experience_value: 500,
        loot_chance: 1.0,
    },
];

pub fn get_enemy_definition(enemy_type: EnemyType) -> Option<&'static EnemyDefinition> {
    ENEMY_DEFINITIONS
        .iter()
        .find(|d| d.enemy_type == enemy_type)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyState {
    Idle,
    Patrol,
    Chase,
    Attack,
    Stunned,
    Dead,
}

#[derive(Clone)]
pub struct Enemy {
    pub entity: Entity,
    pub enemy_type: EnemyType,
    pub health: f32,
    pub state: EnemyState,
    pub attack_cooldown: f32,
    pub stun_duration: f32,
    pub patrol_target: Option<Vec3>,
    pub home_position: Vec3,
    pub last_known_player_pos: Option<Vec3>,
    pub damage_flash_timer: f32,
}

impl Enemy {
    pub fn new(entity: Entity, enemy_type: EnemyType, position: Vec3) -> Self {
        let def = get_enemy_definition(enemy_type).unwrap();
        Self {
            entity,
            enemy_type,
            health: def.health,
            state: EnemyState::Idle,
            attack_cooldown: 0.0,
            stun_duration: 0.0,
            patrol_target: None,
            home_position: position,
            last_known_player_pos: None,
            damage_flash_timer: 0.0,
        }
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
        self.damage_flash_timer = 0.2;
        if self.health <= 0.0 {
            self.state = EnemyState::Dead;
        }
    }

    pub fn is_dead(&self) -> bool {
        self.state == EnemyState::Dead
    }
}

pub struct EnemySpawn {
    pub position: Vec3,
    pub enemy_type: EnemyType,
}
