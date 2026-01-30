use nightshade::prelude::*;
use std::collections::{HashMap, HashSet};

pub use freecs::Entity as GameEntity;

use crate::data::{NpcType, ShopItem};
use crate::types::{CropType, ItemId, ItemSlot, Season, ToolType, Weather};

freecs::ecs! {
    World {
        handle: Handle => HANDLE,
        position: Position => POSITION,
        player: Player => PLAYER,
        tree: Tree => TREE,
        tile: Tile => TILE,
        crop: Crop => CROP,
        npc: Npc => NPC,
    }
    Resources {
        player_entity: Option<GameEntity>,
        npcs: Vec<GameEntity>,
        loaded_chunks: HashSet<(i32, i32)>,

        day: u32,
        hour: f32,
        season: Season,
        weather: Weather,

        money: u32,
        inventory: Inventory,

        camera_mode: CameraMode,
        camera_yaw: f32,
        targeted_tree: Option<GameEntity>,

        dialogue: Option<DialogueState>,
        shop: Option<ShopState>,
        shop_items: Vec<ShopItem>,

        trees: TreeStore,
        farm: FarmStore,

        popups: PopupStore,
        visuals: VisualEntities,
    }
}

#[derive(Clone, Default)]
pub struct TreeStore {
    pub by_chunk: HashMap<(i32, i32), Vec<GameEntity>>,
}

#[derive(Clone, Default)]
pub struct FarmStore {
    pub tiles: HashMap<(i32, i32), GameEntity>,
    pub crops: HashMap<(i32, i32), GameEntity>,
}

#[derive(Clone, Default)]
pub struct PopupStore {
    pub popups: Vec<Popup>,
}

#[derive(Clone)]
pub struct Popup {
    pub entity: Option<Entity>,
    pub text: String,
    pub lifetime: f32,
    pub start_position: Vec3,
}

#[derive(Clone, Default)]
pub struct VisualEntities {
    pub camera: Option<Entity>,
    pub sun: Option<Entity>,
    pub ground: Option<Entity>,
    pub grass_region: Option<Entity>,
    pub player_visual: Option<Entity>,
    pub tool_visual: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Handle(pub Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct Position(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Player {
    pub facing: Vec3,
    pub height: f32,
    pub vertical_velocity: f32,
    pub grounded: bool,
    pub stamina: f32,
    pub max_stamina: f32,
    pub equipped_tool: ToolType,
    pub attack_cooldown: f32,
    pub interaction_cooldown: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeState {
    #[default]
    Standing,
    Falling,
    Shrinking,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tree {
    pub chunk: (i32, i32),
    pub health: f32,
    pub max_health: f32,
    pub state: TreeState,
    pub fall_direction: Vec3,
    pub fall_progress: f32,
    pub shrink_progress: f32,
    pub trunk_height: f32,
    pub trunk_radius: f32,
    pub tree_scale: f32,
    pub trunk_visual: Option<Entity>,
    pub foliage_visuals: [Option<Entity>; 3],
    pub foliage_y_offsets: [f32; 3],
    pub original_trunk_scale: Vec3,
    pub original_foliage_scales: [Vec3; 3],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tile {
    pub watered: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Crop {
    pub crop_type: CropType,
    pub growth_stage: u8,
    pub max_growth_stage: u8,
    pub days_in_stage: u8,
    pub watered_today: bool,
    pub watered_days: u8,
    pub total_days: u8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Npc {
    pub npc_type: NpcType,
    pub friendship: i32,
    pub talked_today: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    #[default]
    TopDown,
    ThirdPerson,
}

#[derive(Debug, Clone, Copy)]
pub struct DialogueState {
    pub npc: GameEntity,
    pub line_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopMode {
    #[default]
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ShopState {
    pub selected: usize,
    pub mode: ShopMode,
}

#[derive(Clone, Default)]
pub struct Inventory {
    pub hotbar: [ItemSlot; 10],
    pub selected_slot: usize,
}

impl Inventory {
    pub fn add_item(&mut self, item_id: ItemId, quantity: u32) -> bool {
        for slot in &mut self.hotbar {
            if slot.item_id == Some(item_id) && slot.quantity < 999 {
                let can_add = (999 - slot.quantity).min(quantity);
                slot.quantity += can_add;
                if can_add == quantity {
                    return true;
                }
            }
        }
        for slot in &mut self.hotbar {
            if slot.item_id.is_none() {
                slot.item_id = Some(item_id);
                slot.quantity = quantity;
                return true;
            }
        }
        false
    }

    pub fn remove_item(&mut self, item_id: ItemId, quantity: u32) -> bool {
        let mut remaining = quantity;
        for slot in &mut self.hotbar {
            if slot.item_id == Some(item_id) {
                let can_remove = slot.quantity.min(remaining);
                slot.quantity -= can_remove;
                remaining -= can_remove;
                if slot.quantity == 0 {
                    slot.item_id = None;
                }
                if remaining == 0 {
                    return true;
                }
            }
        }
        false
    }

    pub fn count_item(&self, item_id: ItemId) -> u32 {
        self.hotbar
            .iter()
            .filter(|s| s.item_id == Some(item_id))
            .map(|s| s.quantity)
            .sum()
    }

    pub fn selected_item(&self) -> Option<(ItemId, u32)> {
        let slot = &self.hotbar[self.selected_slot];
        slot.item_id.map(|id| (id, slot.quantity))
    }

    pub fn consume_selected(&mut self, amount: u32) -> bool {
        let slot = &mut self.hotbar[self.selected_slot];
        if let Some(item_id) = slot.item_id {
            if slot.quantity >= amount {
                slot.quantity -= amount;
                if slot.quantity == 0 {
                    slot.item_id = None;
                }
                return true;
            }
            return self.remove_item(item_id, amount);
        }
        false
    }
}

pub fn tile_coords(x: f32, z: f32) -> (i32, i32) {
    use crate::types::TILE_SIZE;
    (
        (x / TILE_SIZE).floor() as i32,
        (z / TILE_SIZE).floor() as i32,
    )
}

pub fn tile_center(tx: i32, tz: i32) -> Vec3 {
    use crate::types::TILE_SIZE;
    Vec3::new(
        tx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        0.0,
        tz as f32 * TILE_SIZE + TILE_SIZE * 0.5,
    )
}

pub fn chunk_coords(x: f32, z: f32) -> (i32, i32) {
    use crate::types::CHUNK_SIZE;
    (
        (x / CHUNK_SIZE).floor() as i32,
        (z / CHUNK_SIZE).floor() as i32,
    )
}
