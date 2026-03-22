use crate::constants::CITY_REINFORCEMENT;
use crate::ecs::{
    Entity, Faction, GameEvents, GameWorld, ReinforcementEvent, TileType, UnitType, unit_stats,
};
use crate::hex::{HexCoord, hex_distance};
use crate::replay::{ReinforcementEntry, ReplayAction};

pub struct PendingSpawn {
    pub coord: HexCoord,
    pub faction: Faction,
    pub soldiers: i32,
    pub unit_type: UnitType,
}

struct TileInfo {
    coord: HexCoord,
    tile_type: TileType,
    faction: Option<Faction>,
}

fn tile_type_name(tile_type: TileType) -> &'static str {
    match tile_type {
        TileType::Capital => "capital",
        TileType::City => "city",
        TileType::Port => "port",
        _ => "tile",
    }
}

fn gather_tile_info(game_world: &GameWorld) -> Vec<TileInfo> {
    game_world
        .resources
        .tile_map
        .iter()
        .filter_map(|(&coord, &entity)| {
            let tile = game_world.get_tile(entity)?;
            Some(TileInfo {
                coord,
                tile_type: tile.tile_type,
                faction: tile.faction,
            })
        })
        .collect()
}

fn city_reinforcements(
    game_world: &mut GameWorld,
    events: &mut GameEvents,
    tile_info: &[TileInfo],
    current_faction: Faction,
    pending_spawns: &mut Vec<PendingSpawn>,
    replay_entries: &mut Vec<ReinforcementEntry>,
) {
    for tile in tile_info {
        if tile.faction != Some(current_faction) {
            continue;
        }

        let reinforcement = match tile.tile_type {
            TileType::City | TileType::Capital => CITY_REINFORCEMENT,
            _ => continue,
        };

        let unit_at = game_world
            .resources
            .unit_position_map
            .get(&tile.coord)
            .copied();

        if let Some(unit_entity) = unit_at {
            if let Some(unit) = game_world.get_unit(unit_entity)
                && unit.faction == current_faction
            {
                let mut unit = *unit;
                let max = unit_stats(unit.unit_type).max_soldiers;
                unit.soldiers = (unit.soldiers + reinforcement).min(max);
                game_world.set_unit(unit_entity, unit);
                events.reinforcement_events.push(ReinforcementEvent {
                    faction: current_faction,
                    soldiers: reinforcement,
                    location_name: tile_type_name(tile.tile_type).to_string(),
                });
                replay_entries.push(ReinforcementEntry {
                    coord: tile.coord,
                    faction: current_faction,
                    soldiers_added: reinforcement,
                    is_new_unit: false,
                    unit_type: unit.unit_type,
                    new_total: unit.soldiers,
                });
            }
        } else {
            pending_spawns.push(PendingSpawn {
                coord: tile.coord,
                faction: current_faction,
                soldiers: reinforcement,
                unit_type: UnitType::Infantry,
            });
            events.reinforcement_events.push(ReinforcementEvent {
                faction: current_faction,
                soldiers: reinforcement,
                location_name: tile_type_name(tile.tile_type).to_string(),
            });
            replay_entries.push(ReinforcementEntry {
                coord: tile.coord,
                faction: current_faction,
                soldiers_added: reinforcement,
                is_new_unit: true,
                unit_type: UnitType::Infantry,
                new_total: reinforcement,
            });
        }
    }
}

fn port_reinforcements(
    game_world: &mut GameWorld,
    events: &mut GameEvents,
    tile_info: &[TileInfo],
    current_faction: Faction,
    replay_entries: &mut Vec<ReinforcementEntry>,
) {
    for tile in tile_info {
        if tile.tile_type != TileType::Port || tile.faction != Some(current_faction) {
            continue;
        }

        let port_reinforcement = 1 + (game_world.resources.rng_seed % 3) as i32;
        game_world.resources.rng_seed = game_world
            .resources
            .rng_seed
            .wrapping_mul(1103515245)
            .wrapping_add(12345);

        let mut closest_unit: Option<(Entity, i32)> = None;
        for (&unit_coord, &unit_entity) in &game_world.resources.unit_position_map {
            if let Some(unit) = game_world.get_unit(unit_entity)
                && unit.faction != current_faction
            {
                continue;
            }

            let distance = hex_distance(tile.coord, unit_coord);
            if distance <= 3 && (closest_unit.is_none() || distance < closest_unit.unwrap().1) {
                closest_unit = Some((unit_entity, distance));
            }
        }

        if let Some((unit_entity, _)) = closest_unit
            && let Some(unit) = game_world.get_unit(unit_entity)
        {
            let mut unit = *unit;
            let max = unit_stats(unit.unit_type).max_soldiers;
            unit.soldiers = (unit.soldiers + port_reinforcement).min(max);
            game_world.set_unit(unit_entity, unit);
            events.reinforcement_events.push(ReinforcementEvent {
                faction: current_faction,
                soldiers: port_reinforcement,
                location_name: "port".to_string(),
            });
            replay_entries.push(ReinforcementEntry {
                coord: *game_world
                    .get_hex_position(unit_entity)
                    .map(|h| &h.0)
                    .unwrap_or(&tile.coord),
                faction: current_faction,
                soldiers_added: port_reinforcement,
                is_new_unit: false,
                unit_type: unit.unit_type,
                new_total: unit.soldiers,
            });
        }
    }
}

fn territory_reinforcements(
    game_world: &mut GameWorld,
    events: &mut GameEvents,
    tile_info: &[TileInfo],
    current_faction: Faction,
    pending_spawns: &mut Vec<PendingSpawn>,
    replay_entries: &mut Vec<ReinforcementEntry>,
) {
    let territory_count = tile_info
        .iter()
        .filter(|t| t.faction == Some(current_faction) && t.tile_type != TileType::Sea)
        .count();
    let territory_bonus = (territory_count / 10) as i32;

    if territory_bonus <= 0 {
        return;
    }

    let capital_coord = current_faction.capital_coord(&game_world.resources.map_params);

    let unit_at_capital = game_world
        .resources
        .unit_position_map
        .get(&capital_coord)
        .copied();

    if let Some(unit_entity) = unit_at_capital {
        if let Some(unit) = game_world.get_unit(unit_entity)
            && unit.faction == current_faction
        {
            let mut unit = *unit;
            let max = unit_stats(unit.unit_type).max_soldiers;
            unit.soldiers = (unit.soldiers + territory_bonus).min(max);
            game_world.set_unit(unit_entity, unit);
            events.reinforcement_events.push(ReinforcementEvent {
                faction: current_faction,
                soldiers: territory_bonus,
                location_name: "territory".to_string(),
            });
            replay_entries.push(ReinforcementEntry {
                coord: capital_coord,
                faction: current_faction,
                soldiers_added: territory_bonus,
                is_new_unit: false,
                unit_type: unit.unit_type,
                new_total: unit.soldiers,
            });
        }
    } else {
        let capital_owned = game_world
            .resources
            .tile_map
            .get(&capital_coord)
            .and_then(|&entity| game_world.get_tile(entity))
            .is_some_and(|tile| tile.faction == Some(current_faction));

        if capital_owned {
            pending_spawns.push(PendingSpawn {
                coord: capital_coord,
                faction: current_faction,
                soldiers: territory_bonus.max(1),
                unit_type: UnitType::Infantry,
            });
            events.reinforcement_events.push(ReinforcementEvent {
                faction: current_faction,
                soldiers: territory_bonus.max(1),
                location_name: "territory".to_string(),
            });
            replay_entries.push(ReinforcementEntry {
                coord: capital_coord,
                faction: current_faction,
                soldiers_added: territory_bonus.max(1),
                is_new_unit: true,
                unit_type: UnitType::Infantry,
                new_total: territory_bonus.max(1),
            });
        }
    }
}

pub fn reinforcement_system(
    game_world: &mut GameWorld,
    events: &mut GameEvents,
) -> Vec<PendingSpawn> {
    let current_faction = game_world.resources.current_faction;
    let tile_info = gather_tile_info(game_world);
    let mut pending_spawns = Vec::new();
    let mut replay_entries: Vec<ReinforcementEntry> = Vec::new();

    city_reinforcements(
        game_world,
        events,
        &tile_info,
        current_faction,
        &mut pending_spawns,
        &mut replay_entries,
    );
    port_reinforcements(
        game_world,
        events,
        &tile_info,
        current_faction,
        &mut replay_entries,
    );
    territory_reinforcements(
        game_world,
        events,
        &tile_info,
        current_faction,
        &mut pending_spawns,
        &mut replay_entries,
    );

    if !replay_entries.is_empty() {
        events.replay_actions.push(ReplayAction::Reinforcement {
            entries: replay_entries,
        });
    }

    pending_spawns
}
