use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SkillType {
    Fireball,
    IceBlast,
    LightningBolt,
    Dash,
    Shield,
    Heal,
    Blink,
    Explosion,
}

#[derive(Clone)]
pub struct SkillDefinition {
    pub skill_type: SkillType,
    pub name: &'static str,
    pub mana_cost: f32,
    pub cooldown: f32,
    pub damage: f32,
    pub area_of_effect: f32,
    pub color: [f32; 4],
    pub key_binding: &'static str,
}

pub const SKILL_DEFINITIONS: &[SkillDefinition] = &[
    SkillDefinition {
        skill_type: SkillType::Fireball,
        name: "Fireball",
        mana_cost: 20.0,
        cooldown: 1.5,
        damage: 30.0,
        area_of_effect: 2.0,
        color: [1.0, 0.4, 0.1, 1.0],
        key_binding: "1",
    },
    SkillDefinition {
        skill_type: SkillType::IceBlast,
        name: "Ice Blast",
        mana_cost: 25.0,
        cooldown: 2.0,
        damage: 20.0,
        area_of_effect: 3.0,
        color: [0.5, 0.8, 1.0, 1.0],
        key_binding: "2",
    },
    SkillDefinition {
        skill_type: SkillType::LightningBolt,
        name: "Lightning Bolt",
        mana_cost: 30.0,
        cooldown: 2.5,
        damage: 50.0,
        area_of_effect: 0.5,
        color: [0.9, 0.9, 1.0, 1.0],
        key_binding: "3",
    },
    SkillDefinition {
        skill_type: SkillType::Dash,
        name: "Dash",
        mana_cost: 15.0,
        cooldown: 3.0,
        damage: 0.0,
        area_of_effect: 0.0,
        color: [0.8, 0.8, 0.8, 1.0],
        key_binding: "4",
    },
    SkillDefinition {
        skill_type: SkillType::Shield,
        name: "Magic Shield",
        mana_cost: 35.0,
        cooldown: 10.0,
        damage: 0.0,
        area_of_effect: 0.0,
        color: [0.3, 0.6, 1.0, 0.5],
        key_binding: "5",
    },
    SkillDefinition {
        skill_type: SkillType::Heal,
        name: "Heal",
        mana_cost: 40.0,
        cooldown: 8.0,
        damage: -30.0,
        area_of_effect: 0.0,
        color: [0.3, 1.0, 0.3, 1.0],
        key_binding: "6",
    },
    SkillDefinition {
        skill_type: SkillType::Blink,
        name: "Blink",
        mana_cost: 25.0,
        cooldown: 5.0,
        damage: 0.0,
        area_of_effect: 0.0,
        color: [0.8, 0.2, 1.0, 1.0],
        key_binding: "7",
    },
    SkillDefinition {
        skill_type: SkillType::Explosion,
        name: "Explosion",
        mana_cost: 60.0,
        cooldown: 15.0,
        damage: 100.0,
        area_of_effect: 6.0,
        color: [1.0, 0.5, 0.0, 1.0],
        key_binding: "8",
    },
];

pub fn get_skill_definition(skill_type: SkillType) -> Option<&'static SkillDefinition> {
    SKILL_DEFINITIONS
        .iter()
        .find(|d| d.skill_type == skill_type)
}

#[derive(Clone, Default)]
pub struct SkillState {
    pub unlocked: bool,
    pub cooldown_remaining: f32,
    pub level: u32,
}

#[derive(Clone)]
pub struct PlayerSkills {
    pub skills: std::collections::HashMap<SkillType, SkillState>,
    pub active_effects: Vec<ActiveEffect>,
}

impl Default for PlayerSkills {
    fn default() -> Self {
        let mut skills = std::collections::HashMap::new();
        skills.insert(
            SkillType::Fireball,
            SkillState {
                unlocked: true,
                cooldown_remaining: 0.0,
                level: 1,
            },
        );
        skills.insert(
            SkillType::Dash,
            SkillState {
                unlocked: true,
                cooldown_remaining: 0.0,
                level: 1,
            },
        );
        skills.insert(
            SkillType::Heal,
            SkillState {
                unlocked: true,
                cooldown_remaining: 0.0,
                level: 1,
            },
        );
        Self {
            skills,
            active_effects: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct ActiveEffect {
    pub effect_type: EffectType,
    pub duration_remaining: f32,
    pub strength: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Shield,
    SpeedBoost,
}

pub struct Projectile {
    pub entity: Entity,
    pub skill_type: SkillType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub damage: f32,
    pub aoe_radius: f32,
    pub lifetime: f32,
    pub owner_is_player: bool,
    pub is_aoe_explosion: bool,
}
