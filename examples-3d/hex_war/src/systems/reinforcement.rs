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

fn tile_type_name(tile_type: TileType) -> &'static str {
    match tile_type {
        TileType::Capital => "capital",
        TileType::City => "city",
        TileType::Port => "port",
        _ => "tile",
    }
}

pub fn reinforcement_system(
    game_world: &mut GameWorld,
    events: &mut GameEvents,
) -> Vec<PendingSpawn> {
    let current_faction = game_world.resources.current_faction;
    let mut pending_spawns = Vec::new();
    let mut replay_entries: Vec<ReinforcementEntry> = Vec::new();

    let tile_info: Vec<(HexCoord, TileType, Option<Faction>)> = game_world
        .resources
        .tile_map
        .iter()
        .filter_map(|(&coord, &entity)| {
            let tile = game_world.get_tile(entity)?;
            Some((coord, tile.tile_type, tile.faction))
        })
        .collect();

    for &(coord, tile_type, tile_faction) in &tile_info {
        if tile_faction != Some(current_faction) {
            continue;
        }

        let reinforcement = match tile_type {
            TileType::City | TileType::Capital => CITY_REINFORCEMENT,
            _ => continue,
        };

        let unit_at = game_world.resources.unit_position_map.get(&coord).copied();

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
                    location_name: tile_type_name(tile_type).to_string(),
                });
                replay_entries.push(ReinforcementEntry {
                    coord,
                    faction: current_faction,
                    soldiers_added: reinforcement,
                    is_new_unit: false,
                    unit_type: unit.unit_type,
                    new_total: unit.soldiers,
                });
            }
        } else {
            pending_spawns.push(PendingSpawn {
                coord,
                faction: current_faction,
                soldiers: reinforcement,
                unit_type: UnitType::Infantry,
            });
            events.reinforcement_events.push(ReinforcementEvent {
                faction: current_faction,
                soldiers: reinforcement,
                location_name: tile_type_name(tile_type).to_string(),
            });
            replay_entries.push(ReinforcementEntry {
                coord,
                faction: current_faction,
                soldiers_added: reinforcement,
                is_new_unit: true,
                unit_type: UnitType::Infantry,
                new_total: reinforcement,
            });
        }
    }

    for &(coord, tile_type, tile_faction) in &tile_info {
        if tile_type != TileType::Port {
            continue;
        }

        if tile_faction != Some(current_faction) {
            continue;
        }

        let port_reinforcement = 1 + (game_world.resources.rng_seed as i32 % 3);
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

            let distance = hex_distance(coord, unit_coord);
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
                    .unwrap_or(&coord),
                faction: current_faction,
                soldiers_added: port_reinforcement,
                is_new_unit: false,
                unit_type: unit.unit_type,
                new_total: unit.soldiers,
            });
        }
    }

    let territory_count = tile_info
        .iter()
        .filter(|(_, tile_type, faction)| {
            *faction == Some(current_faction) && *tile_type != TileType::Sea
        })
        .count();
    let territory_bonus = (territory_count / 10) as i32;

    if territory_bonus > 0 {
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

    if !replay_entries.is_empty() {
        events.replay_actions.push(ReplayAction::Reinforcement {
            entries: replay_entries,
        });
    }

    pending_spawns
}
