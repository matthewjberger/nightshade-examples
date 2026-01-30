use nightshade::prelude::*;

#[derive(Debug, Clone)]
pub struct TreeChoppedEvent {
    pub position: Vec3,
    pub wood: u32,
}

#[derive(Debug, Clone)]
pub struct CropHarvestedEvent {
    pub position: Vec3,
    pub item_name: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct DayChangedEvent {
    pub new_day: u32,
    pub new_season: Option<crate::types::Season>,
}
