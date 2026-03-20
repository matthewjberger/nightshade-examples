use crate::hex::HexCoord;
use crate::map::MapGenParams;
use crate::turn_phase::TurnPhaseState;
use nightshade::prelude::*;
use std::collections::{HashMap, HashSet};

pub use freecs::Entity;

pub const FACTION_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Difficulty {
    #[default]
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Faction {
    #[default]
    Redosia,
    Violetnam,
    Bluegaria,
    Greenland,
}

impl Faction {
    pub const ALL: [Faction; FACTION_COUNT] = [
        Faction::Redosia,
        Faction::Violetnam,
        Faction::Bluegaria,
        Faction::Greenland,
    ];

    pub fn next(self) -> Faction {
        match self {
            Faction::Redosia => Faction::Violetnam,
            Faction::Violetnam => Faction::Bluegaria,
            Faction::Bluegaria => Faction::Greenland,
            Faction::Greenland => Faction::Redosia,
        }
    }

    pub fn color(self) -> [f32; 4] {
        match self {
            Faction::Redosia => [0.8, 0.2, 0.2, 1.0],
            Faction::Violetnam => [0.6, 0.2, 0.8, 1.0],
            Faction::Bluegaria => [0.2, 0.4, 0.8, 1.0],
            Faction::Greenland => [0.2, 0.8, 0.2, 1.0],
        }
    }

    pub fn index(self) -> usize {
        match self {
            Faction::Redosia => 0,
            Faction::Violetnam => 1,
            Faction::Bluegaria => 2,
            Faction::Greenland => 3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Faction::Redosia => "Redosia",
            Faction::Violetnam => "Violetnam",
            Faction::Bluegaria => "Bluegaria",
            Faction::Greenland => "Greenland",
        }
    }

    pub fn capital_coord(self, params: &MapGenParams) -> HexCoord {
        let (col, row, _) = params.capital_positions()[self.index()];
        HexCoord { column: col, row }
    }
}

#[derive(Clone, Copy, Default)]
pub struct HexMetrics {
    pub hex_width: f32,
    pub hex_depth: f32,
}

freecs::ecs! {
    GameWorld {
        engine_entity: EngineEntity => ENGINE_ENTITY,
        world_position: WorldPosition => WORLD_POSITION,
        hex_position: HexPosition => HEX_POSITION,
        unit: Unit => UNIT,
        movement: Movement => MOVEMENT,
        tile: Tile => TILE,
        floating_popup: FloatingPopup => FLOATING_POPUP,
    }
    Tags {
        selected => SELECTED,
    }
    GameResources {
        hex_metrics: HexMetrics,
        rng_seed: u32,
        map_params: MapGenParams,
        needs_regeneration: bool,
        valid_move_tiles: HashSet<HexCoord>,
        hovered_tile: Option<HexCoord>,
        current_faction: Faction,
        actions_remaining: u8,
        turn_number: u32,
        faction_eliminated: [bool; FACTION_COUNT],
        faction_morale: [i32; FACTION_COUNT],
        capital_owners: [Option<Faction>; FACTION_COUNT],
        speech_used: bool,
        turn_order: Vec<freecs::Entity>,
        current_unit_index: usize,
        game_speed: f32,
        difficulty: Difficulty,
        tile_map: HashMap<HexCoord, freecs::Entity>,
        passable_tiles: HashSet<HexCoord>,
        port_tiles: HashSet<HexCoord>,
        unit_position_map: HashMap<HexCoord, freecs::Entity>,
        valid_moves_generation: u32,
        turn_phase: TurnPhaseState,
        frame_cache: FrameCache,
    }
}

#[derive(Default, PartialEq)]
pub struct HudSnapshot {
    pub turn: u32,
    pub faction: Faction,
    pub actions: u8,
    pub speed_bits: u32,
    pub is_player_turn: bool,
    pub turn_phase: TurnPhaseState,
}

#[derive(Default)]
pub struct FrameCache {
    pub previous_hovered_tile: Option<HexCoord>,
    pub previous_selected_unit: Option<freecs::Entity>,
    pub previous_valid_move_count: usize,
    pub previous_hud: HudSnapshot,
    pub previous_log_scroll: usize,
    pub previous_log_count: usize,
    pub previous_highlight_generation: u32,
}

pub fn compute_combat_strength(soldiers: i32, morale: i32, multiplier: f32) -> f32 {
    soldiers as f32 * (1.0 + morale as f32 / 100.0) * multiplier
}

pub fn get_faction_morale(resources: &GameResources, faction: Faction) -> i32 {
    resources.faction_morale[faction.index()]
}

pub fn modify_faction_morale(resources: &mut GameResources, faction: Faction, delta: i32) {
    let index = faction.index();
    resources.faction_morale[index] = (resources.faction_morale[index] + delta).clamp(-50, 50);
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EngineEntity(pub Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct WorldPosition(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct HexPosition(pub HexCoord);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UnitType {
    #[default]
    Infantry,
}

pub struct UnitStats {
    pub movement_range: i32,
    pub attack_multiplier: f32,
    pub defense_multiplier: f32,
    pub max_soldiers: i32,
}

pub fn unit_stats(unit_type: UnitType) -> UnitStats {
    match unit_type {
        UnitType::Infantry => UnitStats {
            movement_range: 2,
            attack_multiplier: 1.0,
            defense_multiplier: 1.0,
            max_soldiers: 99,
        },
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Unit {
    pub faction: Faction,
    pub soldiers: i32,
    pub morale: i32,
    pub unit_type: UnitType,
    pub has_moved: bool,
    pub text_entity: Option<Entity>,
}

#[derive(Debug, Clone, Default)]
pub struct Movement {
    pub path: Vec<HexCoord>,
    pub current_segment: usize,
    pub segment_progress: f32,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TileType {
    Sea,
    #[default]
    Land,
    Forest,
    City,
    Port,
    Capital,
}

pub fn tile_defense_bonus(tile_type: TileType) -> f32 {
    match tile_type {
        TileType::Capital => 1.2,
        TileType::City => 1.1,
        TileType::Forest => 1.15,
        TileType::Port => 1.05,
        _ => 1.0,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tile {
    pub tile_type: TileType,
    pub faction: Option<Faction>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FloatingPopup {
    pub text_entity: Entity,
    pub lifetime: f32,
}

pub fn update_unit_position(
    game_world: &mut GameWorld,
    entity: freecs::Entity,
    new_coord: HexCoord,
) {
    if let Some(hex_pos) = game_world.get_hex_position(entity) {
        let old_coord = hex_pos.0;
        if old_coord != new_coord
            && game_world.resources.unit_position_map.get(&old_coord) == Some(&entity)
        {
            game_world.resources.unit_position_map.remove(&old_coord);
        }
    }
    if let Some(hex_pos) = game_world.get_hex_position_mut(entity) {
        hex_pos.0 = new_coord;
    }
    game_world
        .resources
        .unit_position_map
        .insert(new_coord, entity);
}

pub fn remove_unit_position(game_world: &mut GameWorld, entity: freecs::Entity) {
    if let Some(hex_pos) = game_world.get_hex_position(entity) {
        let coord = hex_pos.0;
        if game_world.resources.unit_position_map.get(&coord) == Some(&entity) {
            game_world.resources.unit_position_map.remove(&coord);
        }
    }
}

pub fn get_tile_at(game_world: &GameWorld, coord: HexCoord) -> Option<(freecs::Entity, Tile)> {
    let &entity = game_world.resources.tile_map.get(&coord)?;
    let tile = game_world.get_tile(entity).copied()?;
    Some((entity, tile))
}

pub fn get_tile_type_at(game_world: &GameWorld, coord: HexCoord) -> Option<TileType> {
    get_tile_at(game_world, coord).map(|(_, tile)| tile.tile_type)
}

pub fn get_defense_bonus_at(game_world: &GameWorld, coord: HexCoord) -> f32 {
    get_tile_at(game_world, coord)
        .map(|(_, tile)| tile_defense_bonus(tile.tile_type))
        .unwrap_or(1.0)
}

#[derive(Debug, Clone, Copy)]
pub struct CombatEvent {
    pub attacker_faction: Faction,
    pub defender_faction: Faction,
    pub attacker_survived: bool,
    pub defender_survived: bool,
}

#[derive(Debug, Clone)]
pub struct ReinforcementEvent {
    pub faction: Faction,
    pub soldiers: i32,
    pub location_name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct SpeechEvent {
    pub faction: Faction,
}

#[derive(Debug, Clone, Copy)]
pub struct FactionEliminatedEvent {
    pub faction: Faction,
}

use crate::replay::{GameSnapshot, ReplayAction};

#[derive(Default)]
pub struct GameEvents {
    pub combat_events: Vec<CombatEvent>,
    pub reinforcement_events: Vec<ReinforcementEvent>,
    pub speech_events: Vec<SpeechEvent>,
    pub faction_eliminated_events: Vec<FactionEliminatedEvent>,
    pub replay_actions: Vec<ReplayAction>,
    pub replay_snapshots: Vec<(usize, GameSnapshot)>,
}
