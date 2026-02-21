use nightshade::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ItemId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CropType(pub u32);

#[derive(Debug, Clone, Copy, Default)]
pub struct ItemSlot {
    pub item_id: Option<ItemId>,
    pub quantity: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolType {
    #[default]
    Hand,
    Hoe,
    WateringCan,
    Axe,
    Pickaxe,
    Scythe,
    Sword,
}

impl ToolType {
    pub fn name(&self) -> &'static str {
        match self {
            ToolType::Hand => "Hand",
            ToolType::Hoe => "Hoe",
            ToolType::WateringCan => "Watering Can",
            ToolType::Axe => "Axe",
            ToolType::Pickaxe => "Pickaxe",
            ToolType::Scythe => "Scythe",
            ToolType::Sword => "Sword",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    pub fn name(&self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Fall => "Fall",
            Season::Winter => "Winter",
        }
    }

    pub fn next(&self) -> Season {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Fall,
            Season::Fall => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weather {
    #[default]
    Sunny,
    Cloudy,
    Rainy,
    Stormy,
    Snowy,
}

impl Weather {
    pub fn name(&self) -> &'static str {
        match self {
            Weather::Sunny => "Sunny",
            Weather::Cloudy => "Cloudy",
            Weather::Rainy => "Rainy",
            Weather::Stormy => "Stormy",
            Weather::Snowy => "Snowy",
        }
    }
}

pub const TILE_SIZE: f32 = 1.0;
pub const CHUNK_SIZE: f32 = 20.0;
pub const RENDER_DISTANCE: i32 = 3;

pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_SPEED: f32 = 5.0;
pub const PLAYER_STAMINA_MAX: f32 = 100.0;
pub const STAMINA_REGEN_RATE: f32 = 2.0;
pub const TOOL_STAMINA_COST: f32 = 5.0;
pub const JUMP_VELOCITY: f32 = 8.0;
pub const GRAVITY: f32 = 20.0;

pub const CAMERA_HEIGHT: f32 = 20.0;
pub const CAMERA_DISTANCE: f32 = 12.0;
pub const GROUND_SIZE: f32 = 500.0;

pub const DAY_LENGTH_SECONDS: f32 = 120.0;
pub const DAYS_PER_SEASON: u32 = 28;

pub const INTERACTION_RANGE: f32 = 2.5;
pub const CHOP_RANGE: f32 = 3.5;

pub fn format_time(hour: f32) -> String {
    let h = hour as u32 % 24;
    let m = ((hour.fract()) * 60.0) as u32;
    let suffix = if h < 12 { "AM" } else { "PM" };
    let display_h = if h == 0 {
        12
    } else if h > 12 {
        h - 12
    } else {
        h
    };
    format!("{}:{:02} {}", display_h, m, suffix)
}

pub fn get_sun_color(hour: f32) -> Vec3 {
    if !(6.0..=20.0).contains(&hour) {
        Vec3::new(0.2, 0.2, 0.4)
    } else if hour < 8.0 {
        let t = (hour - 6.0) / 2.0;
        Vec3::new(0.8 + t * 0.2, 0.5 + t * 0.4, 0.3 + t * 0.6)
    } else if hour < 17.0 {
        Vec3::new(1.0, 0.95, 0.9)
    } else {
        let t = (hour - 17.0) / 3.0;
        Vec3::new(1.0 - t * 0.8, 0.95 - t * 0.75, 0.9 - t * 0.5)
    }
}

pub fn get_sun_intensity(hour: f32) -> f32 {
    if !(6.0..=20.0).contains(&hour) {
        0.1
    } else if hour < 8.0 {
        0.3 + (hour - 6.0) / 2.0 * 0.7
    } else if hour < 17.0 {
        1.0
    } else {
        1.0 - (hour - 17.0) / 3.0 * 0.9
    }
}
