use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ItemType {
    HealthPotion,
    ManaPotion,
    SpeedPotion,
    Key,
    Coin,
    Gem,
    Sword,
    Staff,
    Shield,
    Scroll,
    Artifact,
}

#[derive(Clone)]
pub struct ItemDefinition {
    pub item_type: ItemType,
    pub name: &'static str,
    pub color: [f32; 4],
    pub mesh: &'static str,
    pub scale: f32,
    pub stackable: bool,
    pub max_stack: usize,
}

pub const ITEM_DEFINITIONS: &[ItemDefinition] = &[
    ItemDefinition {
        item_type: ItemType::HealthPotion,
        name: "Health Potion",
        color: [0.9, 0.2, 0.2, 1.0],
        mesh: "Sphere",
        scale: 0.15,
        stackable: true,
        max_stack: 10,
    },
    ItemDefinition {
        item_type: ItemType::ManaPotion,
        name: "Mana Potion",
        color: [0.2, 0.2, 0.9, 1.0],
        mesh: "Sphere",
        scale: 0.15,
        stackable: true,
        max_stack: 10,
    },
    ItemDefinition {
        item_type: ItemType::SpeedPotion,
        name: "Speed Potion",
        color: [0.2, 0.9, 0.2, 1.0],
        mesh: "Sphere",
        scale: 0.15,
        stackable: true,
        max_stack: 5,
    },
    ItemDefinition {
        item_type: ItemType::Key,
        name: "Key",
        color: [0.9, 0.8, 0.2, 1.0],
        mesh: "Cube",
        scale: 0.1,
        stackable: true,
        max_stack: 5,
    },
    ItemDefinition {
        item_type: ItemType::Coin,
        name: "Gold Coin",
        color: [1.0, 0.85, 0.0, 1.0],
        mesh: "Cylinder",
        scale: 0.08,
        stackable: true,
        max_stack: 999,
    },
    ItemDefinition {
        item_type: ItemType::Gem,
        name: "Mystic Gem",
        color: [0.8, 0.2, 0.8, 1.0],
        mesh: "Sphere",
        scale: 0.12,
        stackable: true,
        max_stack: 50,
    },
    ItemDefinition {
        item_type: ItemType::Sword,
        name: "Steel Sword",
        color: [0.7, 0.7, 0.75, 1.0],
        mesh: "Cube",
        scale: 0.2,
        stackable: false,
        max_stack: 1,
    },
    ItemDefinition {
        item_type: ItemType::Staff,
        name: "Mystic Staff",
        color: [0.5, 0.3, 0.7, 1.0],
        mesh: "Cylinder",
        scale: 0.25,
        stackable: false,
        max_stack: 1,
    },
    ItemDefinition {
        item_type: ItemType::Shield,
        name: "Iron Shield",
        color: [0.5, 0.5, 0.55, 1.0],
        mesh: "Cube",
        scale: 0.25,
        stackable: false,
        max_stack: 1,
    },
    ItemDefinition {
        item_type: ItemType::Scroll,
        name: "Spell Scroll",
        color: [0.95, 0.9, 0.7, 1.0],
        mesh: "Cylinder",
        scale: 0.1,
        stackable: true,
        max_stack: 5,
    },
    ItemDefinition {
        item_type: ItemType::Artifact,
        name: "Ancient Artifact",
        color: [0.0, 1.0, 1.0, 1.0],
        mesh: "Sphere",
        scale: 0.2,
        stackable: false,
        max_stack: 1,
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
        Self {
            slots: vec![InventorySlot::default(); 10],
            selected_slot: 0,
            gold: 0,
        }
    }
}

impl Inventory {
    pub fn add_item(&mut self, item_type: ItemType, quantity: usize) -> bool {
        let def = match get_item_definition(item_type) {
            Some(d) => d,
            None => return false,
        };

        if def.stackable {
            for slot in &mut self.slots {
                if slot.item_type == Some(item_type) && slot.quantity < def.max_stack {
                    let can_add = def.max_stack - slot.quantity;
                    let to_add = quantity.min(can_add);
                    slot.quantity += to_add;
                    if to_add == quantity {
                        return true;
                    }
                }
            }
        }

        for slot in &mut self.slots {
            if slot.item_type.is_none() {
                slot.item_type = Some(item_type);
                slot.quantity = quantity.min(def.max_stack);
                return true;
            }
        }

        false
    }

    pub fn has_item(&self, item_type: ItemType, quantity: usize) -> bool {
        let total: usize = self
            .slots
            .iter()
            .filter(|s| s.item_type == Some(item_type))
            .map(|s| s.quantity)
            .sum();
        total >= quantity
    }
}

pub struct WorldItem {
    pub entity: Entity,
    pub item_type: ItemType,
    pub quantity: usize,
    pub spawn_time: f32,
}
