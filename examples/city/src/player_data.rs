use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    HealthPotion,
    ManaPotion,
    Key,
    Coin,
    Sword,
    Shield,
    Scroll,
    Torch,
    Map,
    Compass,
}

pub struct ItemDefinition {
    pub item_type: ItemType,
    pub name: &'static str,
    pub color: [f32; 4],
}

pub const ITEM_DEFINITIONS: &[ItemDefinition] = &[
    ItemDefinition {
        item_type: ItemType::HealthPotion,
        name: "Health Potion",
        color: [0.9, 0.2, 0.2, 1.0],
    },
    ItemDefinition {
        item_type: ItemType::ManaPotion,
        name: "Mana Potion",
        color: [0.2, 0.2, 0.9, 1.0],
    },
    ItemDefinition {
        item_type: ItemType::Key,
        name: "Key",
        color: [0.9, 0.8, 0.2, 1.0],
    },
    ItemDefinition {
        item_type: ItemType::Coin,
        name: "Gold Coin",
        color: [1.0, 0.85, 0.0, 1.0],
    },
    ItemDefinition {
        item_type: ItemType::Sword,
        name: "Steel Sword",
        color: [0.7, 0.7, 0.75, 1.0],
    },
    ItemDefinition {
        item_type: ItemType::Shield,
        name: "Iron Shield",
        color: [0.5, 0.5, 0.55, 1.0],
    },
    ItemDefinition {
        item_type: ItemType::Scroll,
        name: "Spell Scroll",
        color: [0.95, 0.9, 0.7, 1.0],
    },
    ItemDefinition {
        item_type: ItemType::Torch,
        name: "Torch",
        color: [1.0, 0.6, 0.1, 1.0],
    },
    ItemDefinition {
        item_type: ItemType::Map,
        name: "City Map",
        color: [0.8, 0.75, 0.6, 1.0],
    },
    ItemDefinition {
        item_type: ItemType::Compass,
        name: "Compass",
        color: [0.6, 0.6, 0.7, 1.0],
    },
];

pub fn get_item_definition(item_type: ItemType) -> Option<&'static ItemDefinition> {
    ITEM_DEFINITIONS.iter().find(|d| d.item_type == item_type)
}

#[derive(Clone, Default)]
pub struct InventorySlot {
    pub item_type: Option<ItemType>,
    pub quantity: usize,
}

#[derive(Clone)]
pub struct Inventory {
    pub slots: Vec<InventorySlot>,
    pub selected_slot: usize,
    pub gold: u32,
}

impl Default for Inventory {
    fn default() -> Self {
        let mut slots = vec![InventorySlot::default(); 10];
        slots[0] = InventorySlot {
            item_type: Some(ItemType::Sword),
            quantity: 1,
        };
        slots[1] = InventorySlot {
            item_type: Some(ItemType::Shield),
            quantity: 1,
        };
        slots[2] = InventorySlot {
            item_type: Some(ItemType::HealthPotion),
            quantity: 3,
        };
        slots[3] = InventorySlot {
            item_type: Some(ItemType::ManaPotion),
            quantity: 2,
        };
        slots[4] = InventorySlot {
            item_type: Some(ItemType::Key),
            quantity: 1,
        };
        slots[5] = InventorySlot {
            item_type: Some(ItemType::Torch),
            quantity: 3,
        };
        slots[6] = InventorySlot {
            item_type: Some(ItemType::Map),
            quantity: 1,
        };
        slots[7] = InventorySlot {
            item_type: Some(ItemType::Compass),
            quantity: 1,
        };
        slots[8] = InventorySlot {
            item_type: Some(ItemType::Scroll),
            quantity: 2,
        };

        Self {
            slots,
            selected_slot: 0,
            gold: 247,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    Fireball,
    IceBlast,
    LightningBolt,
    Dash,
    MagicShield,
    Heal,
}

pub struct SkillDefinition {
    pub skill_type: SkillType,
    pub name: &'static str,
    pub color: [f32; 4],
    pub key_binding: &'static str,
}

pub const SKILL_DEFINITIONS: &[SkillDefinition] = &[
    SkillDefinition {
        skill_type: SkillType::Fireball,
        name: "Fireball",
        color: [1.0, 0.4, 0.1, 1.0],
        key_binding: "1",
    },
    SkillDefinition {
        skill_type: SkillType::IceBlast,
        name: "Ice Blast",
        color: [0.5, 0.8, 1.0, 1.0],
        key_binding: "2",
    },
    SkillDefinition {
        skill_type: SkillType::LightningBolt,
        name: "Lightning",
        color: [0.9, 0.9, 1.0, 1.0],
        key_binding: "3",
    },
    SkillDefinition {
        skill_type: SkillType::Dash,
        name: "Dash",
        color: [0.8, 0.8, 0.8, 1.0],
        key_binding: "4",
    },
    SkillDefinition {
        skill_type: SkillType::MagicShield,
        name: "Shield",
        color: [0.3, 0.6, 1.0, 0.5],
        key_binding: "5",
    },
    SkillDefinition {
        skill_type: SkillType::Heal,
        name: "Heal",
        color: [0.3, 1.0, 0.3, 1.0],
        key_binding: "6",
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
}

#[derive(Clone)]
pub struct PlayerSkills {
    pub skills: HashMap<SkillType, SkillState>,
}

impl Default for PlayerSkills {
    fn default() -> Self {
        let mut skills = HashMap::new();
        skills.insert(
            SkillType::Fireball,
            SkillState {
                unlocked: true,
                cooldown_remaining: 0.0,
            },
        );
        skills.insert(
            SkillType::Dash,
            SkillState {
                unlocked: true,
                cooldown_remaining: 0.0,
            },
        );
        skills.insert(
            SkillType::Heal,
            SkillState {
                unlocked: true,
                cooldown_remaining: 0.0,
            },
        );
        skills.insert(
            SkillType::IceBlast,
            SkillState {
                unlocked: true,
                cooldown_remaining: 0.0,
            },
        );
        skills.insert(
            SkillType::LightningBolt,
            SkillState {
                unlocked: false,
                cooldown_remaining: 0.0,
            },
        );
        skills.insert(
            SkillType::MagicShield,
            SkillState {
                unlocked: false,
                cooldown_remaining: 0.0,
            },
        );
        Self { skills }
    }
}

#[derive(Clone)]
pub struct PlayerStats {
    pub health: f32,
    pub max_health: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub level: u32,
    pub experience: u32,
    pub experience_to_next_level: u32,
    pub base_damage: f32,
    pub damage_multiplier: f32,
    pub defense: f32,
    pub speed_multiplier: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 85.0,
            max_health: 100.0,
            mana: 72.0,
            max_mana: 100.0,
            level: 5,
            experience: 340,
            experience_to_next_level: 500,
            base_damage: 18.0,
            damage_multiplier: 1.0,
            defense: 12.0,
            speed_multiplier: 1.0,
        }
    }
}

impl PlayerStats {
    pub fn get_total_damage(&self) -> f32 {
        self.base_damage * self.damage_multiplier
    }
}

#[derive(Clone)]
pub struct PlayerProgress {
    pub stats: PlayerStats,
    pub inventory: Inventory,
    pub skills: PlayerSkills,
    pub enemies_killed: u32,
    pub items_collected: u32,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self {
            stats: PlayerStats::default(),
            inventory: Inventory::default(),
            skills: PlayerSkills::default(),
            enemies_killed: 23,
            items_collected: 47,
        }
    }
}
