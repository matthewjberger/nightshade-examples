use crate::types::{CropType, ItemId, Season};

pub const CROP_PARSNIP: CropType = CropType(1);
pub const CROP_CAULIFLOWER: CropType = CropType(2);
pub const CROP_POTATO: CropType = CropType(3);
pub const CROP_TOMATO: CropType = CropType(4);
pub const CROP_CORN: CropType = CropType(5);
pub const CROP_PUMPKIN: CropType = CropType(6);

pub const ITEM_PARSNIP_SEED: ItemId = ItemId(100);
pub const ITEM_CAULIFLOWER_SEED: ItemId = ItemId(101);
pub const ITEM_POTATO_SEED: ItemId = ItemId(102);
pub const ITEM_TOMATO_SEED: ItemId = ItemId(103);
pub const ITEM_CORN_SEED: ItemId = ItemId(104);
pub const ITEM_PUMPKIN_SEED: ItemId = ItemId(105);

pub const ITEM_PARSNIP: ItemId = ItemId(200);
pub const ITEM_CAULIFLOWER: ItemId = ItemId(201);
pub const ITEM_POTATO: ItemId = ItemId(202);
pub const ITEM_TOMATO: ItemId = ItemId(203);
pub const ITEM_CORN: ItemId = ItemId(204);
pub const ITEM_PUMPKIN: ItemId = ItemId(205);

pub const ITEM_WOOD: ItemId = ItemId(300);
pub const ITEM_STONE: ItemId = ItemId(301);

pub struct CropDefinition {
    pub crop_type: CropType,
    pub name: &'static str,
    pub seed_item: ItemId,
    pub harvest_item: ItemId,
    pub days_to_grow: u8,
    pub growth_stages: u8,
    pub valid_seasons: &'static [Season],
    pub regrows: bool,
}

pub const CROP_DEFINITIONS: &[CropDefinition] = &[
    CropDefinition {
        crop_type: CROP_PARSNIP,
        name: "Parsnip",
        seed_item: ITEM_PARSNIP_SEED,
        harvest_item: ITEM_PARSNIP,
        days_to_grow: 4,
        growth_stages: 4,
        valid_seasons: &[Season::Spring],
        regrows: false,
    },
    CropDefinition {
        crop_type: CROP_CAULIFLOWER,
        name: "Cauliflower",
        seed_item: ITEM_CAULIFLOWER_SEED,
        harvest_item: ITEM_CAULIFLOWER,
        days_to_grow: 12,
        growth_stages: 5,
        valid_seasons: &[Season::Spring],
        regrows: false,
    },
    CropDefinition {
        crop_type: CROP_POTATO,
        name: "Potato",
        seed_item: ITEM_POTATO_SEED,
        harvest_item: ITEM_POTATO,
        days_to_grow: 6,
        growth_stages: 5,
        valid_seasons: &[Season::Spring],
        regrows: false,
    },
    CropDefinition {
        crop_type: CROP_TOMATO,
        name: "Tomato",
        seed_item: ITEM_TOMATO_SEED,
        harvest_item: ITEM_TOMATO,
        days_to_grow: 11,
        growth_stages: 5,
        valid_seasons: &[Season::Summer],
        regrows: true,
    },
    CropDefinition {
        crop_type: CROP_CORN,
        name: "Corn",
        seed_item: ITEM_CORN_SEED,
        harvest_item: ITEM_CORN,
        days_to_grow: 14,
        growth_stages: 6,
        valid_seasons: &[Season::Summer, Season::Fall],
        regrows: true,
    },
    CropDefinition {
        crop_type: CROP_PUMPKIN,
        name: "Pumpkin",
        seed_item: ITEM_PUMPKIN_SEED,
        harvest_item: ITEM_PUMPKIN,
        days_to_grow: 13,
        growth_stages: 5,
        valid_seasons: &[Season::Fall],
        regrows: false,
    },
];

pub fn get_crop_definition(crop_type: CropType) -> Option<&'static CropDefinition> {
    CROP_DEFINITIONS.iter().find(|d| d.crop_type == crop_type)
}

pub fn get_crop_from_seed(seed_item: ItemId) -> Option<CropType> {
    CROP_DEFINITIONS
        .iter()
        .find(|d| d.seed_item == seed_item)
        .map(|d| d.crop_type)
}

pub struct ItemDefinition {
    pub item_id: ItemId,
    pub name: &'static str,
}

pub const ITEM_DEFINITIONS: &[ItemDefinition] = &[
    ItemDefinition {
        item_id: ITEM_PARSNIP_SEED,
        name: "Parsnip Seeds",
    },
    ItemDefinition {
        item_id: ITEM_CAULIFLOWER_SEED,
        name: "Cauliflower Seeds",
    },
    ItemDefinition {
        item_id: ITEM_POTATO_SEED,
        name: "Potato Seeds",
    },
    ItemDefinition {
        item_id: ITEM_TOMATO_SEED,
        name: "Tomato Seeds",
    },
    ItemDefinition {
        item_id: ITEM_CORN_SEED,
        name: "Corn Seeds",
    },
    ItemDefinition {
        item_id: ITEM_PUMPKIN_SEED,
        name: "Pumpkin Seeds",
    },
    ItemDefinition {
        item_id: ITEM_PARSNIP,
        name: "Parsnip",
    },
    ItemDefinition {
        item_id: ITEM_CAULIFLOWER,
        name: "Cauliflower",
    },
    ItemDefinition {
        item_id: ITEM_POTATO,
        name: "Potato",
    },
    ItemDefinition {
        item_id: ITEM_TOMATO,
        name: "Tomato",
    },
    ItemDefinition {
        item_id: ITEM_CORN,
        name: "Corn",
    },
    ItemDefinition {
        item_id: ITEM_PUMPKIN,
        name: "Pumpkin",
    },
    ItemDefinition {
        item_id: ITEM_WOOD,
        name: "Wood",
    },
    ItemDefinition {
        item_id: ITEM_STONE,
        name: "Stone",
    },
];

pub fn get_item_definition(item_id: ItemId) -> Option<&'static ItemDefinition> {
    ITEM_DEFINITIONS.iter().find(|d| d.item_id == item_id)
}

pub fn get_crop_scale(growth_stage: u8, max_stage: u8) -> f32 {
    let progress = growth_stage as f32 / max_stage as f32;
    0.2 + progress * 0.6
}

pub fn get_crop_material_name(
    crop_type: CropType,
    growth_stage: u8,
    max_stage: u8,
) -> &'static str {
    let progress = growth_stage as f32 / max_stage as f32;

    if progress < 0.25 {
        "CropGrowth1"
    } else if progress < 0.5 {
        "CropGrowth2"
    } else if progress < 0.75 {
        "CropGrowth3"
    } else if progress < 1.0 {
        "CropGrowth4"
    } else {
        match crop_type.0 {
            1 => "CropMature_Parsnip",
            2 => "CropMature_Cauliflower",
            3 => "CropMature_Potato",
            4 => "CropMature_Tomato",
            5 => "CropMature_Corn",
            6 => "CropMature_Pumpkin",
            _ => "CropGrowth4",
        }
    }
}

#[derive(Clone)]
pub struct ShopItem {
    pub item_id: ItemId,
    pub buy_price: u32,
    pub sell_price: u32,
}

pub const SHOP_ITEMS: &[ShopItem] = &[
    ShopItem {
        item_id: ITEM_PARSNIP_SEED,
        buy_price: 20,
        sell_price: 10,
    },
    ShopItem {
        item_id: ITEM_CAULIFLOWER_SEED,
        buy_price: 80,
        sell_price: 40,
    },
    ShopItem {
        item_id: ITEM_POTATO_SEED,
        buy_price: 50,
        sell_price: 25,
    },
    ShopItem {
        item_id: ITEM_TOMATO_SEED,
        buy_price: 50,
        sell_price: 25,
    },
    ShopItem {
        item_id: ITEM_CORN_SEED,
        buy_price: 150,
        sell_price: 75,
    },
    ShopItem {
        item_id: ITEM_PUMPKIN_SEED,
        buy_price: 100,
        sell_price: 50,
    },
];
