use crate::constants::UNIT_HEIGHT_OFFSET;
use crate::ecs::{Difficulty, FACTION_COUNT, Faction, GameWorld, UNIT, UnitType};
use crate::hex::{HexCoord, hex_to_world_position};
use crate::systems::{SpawnUnitParams, despawn_unit, spawn_unit, unit_radius_for_soldiers};
use nightshade::prelude::*;

#[derive(Debug, Clone)]
pub enum ReplayAction {
    TurnStart {
        faction: Faction,
        turn: u32,
    },
    Move {
        faction: Faction,
        from: HexCoord,
        to: HexCoord,
    },
    PortTravel {
        faction: Faction,
        from: HexCoord,
        to: HexCoord,
    },
    Attack {
        attacker_coord: HexCoord,
        defender_coord: HexCoord,
        attacker_faction: Faction,
        defender_faction: Faction,
        attacker_survived: bool,
        defender_survived: bool,
        attacker_remaining: i32,
        defender_remaining: i32,
    },
    Merge {
        faction: Faction,
        source_coord: HexCoord,
        target_coord: HexCoord,
        new_soldiers: i32,
    },
    Speech {
        faction: Faction,
    },
    Reinforcement {
        entries: Vec<ReinforcementEntry>,
    },
    Elimination {
        faction: Faction,
    },
}

#[derive(Debug, Clone)]
pub struct ReinforcementEntry {
    pub coord: HexCoord,
    pub faction: Faction,
    pub soldiers_added: i32,
    pub is_new_unit: bool,
    pub unit_type: UnitType,
    pub new_total: i32,
}

#[derive(Clone)]
pub struct GameSnapshot {
    pub units: Vec<(HexCoord, Faction, i32, i32, UnitType)>,
    pub tile_owners: Vec<(HexCoord, Option<Faction>)>,
    pub current_faction: Faction,
    pub turn_number: u32,
    pub actions_remaining: u8,
    pub faction_eliminated: [bool; FACTION_COUNT],
    pub faction_morale: [i32; FACTION_COUNT],
    pub speech_used: bool,
    pub rng_seed: u32,
    pub difficulty: Difficulty,
}

pub fn take_snapshot(game_world: &GameWorld) -> GameSnapshot {
    let units: Vec<_> = game_world
        .resources
        .unit_position_map
        .iter()
        .filter_map(|(&coord, &entity)| {
            let unit = game_world.get_unit(entity)?;
            Some((
                coord,
                unit.faction,
                unit.soldiers,
                unit.morale,
                unit.unit_type,
            ))
        })
        .collect();

    let tile_owners: Vec<_> = game_world
        .resources
        .tile_map
        .iter()
        .filter_map(|(&coord, &entity)| {
            let tile = game_world.get_tile(entity)?;
            if tile.faction.is_some() {
                Some((coord, tile.faction))
            } else {
                None
            }
        })
        .collect();

    GameSnapshot {
        units,
        tile_owners,
        current_faction: game_world.resources.current_faction,
        turn_number: game_world.resources.turn_number,
        actions_remaining: game_world.resources.actions_remaining,
        faction_eliminated: game_world.resources.faction_eliminated,
        faction_morale: game_world.resources.faction_morale,
        speech_used: game_world.resources.speech_used,
        rng_seed: game_world.resources.rng_seed,
        difficulty: game_world.resources.difficulty,
    }
}

pub fn restore_snapshot(game_world: &mut GameWorld, world: &mut World, snapshot: &GameSnapshot) {
    let unit_entities: Vec<_> = game_world.query_entities(UNIT).collect();
    for entity in unit_entities {
        despawn_unit(game_world, world, entity);
    }

    let tile_entities: Vec<_> = game_world.resources.tile_map.values().copied().collect();
    for entity in tile_entities {
        if let Some(tile) = game_world.get_tile_mut(entity) {
            tile.faction = None;
        }
    }

    for &(coord, owner) in &snapshot.tile_owners {
        if let Some(&entity) = game_world.resources.tile_map.get(&coord)
            && let Some(tile) = game_world.get_tile_mut(entity)
        {
            tile.faction = owner;
        }
    }

    let hex_width = game_world.resources.hex_metrics.hex_width;
    let hex_depth = game_world.resources.hex_metrics.hex_depth;
    for &(coord, faction, soldiers, morale, unit_type) in &snapshot.units {
        let entity = spawn_unit(
            game_world,
            world,
            SpawnUnitParams {
                coord,
                hex_width,
                hex_depth,
                faction,
                soldiers,
                unit_type,
            },
        );
        if let Some(unit) = game_world.get_unit(entity) {
            let mut unit = *unit;
            unit.morale = morale;
            game_world.set_unit(entity, unit);
        }
    }

    crate::selection::clear_selection(game_world);
    game_world.resources.frame_cache = Default::default();
    game_world.resources.valid_move_tiles.clear();
    game_world.resources.current_faction = snapshot.current_faction;
    game_world.resources.turn_number = snapshot.turn_number;
    game_world.resources.actions_remaining = snapshot.actions_remaining;
    game_world.resources.faction_eliminated = snapshot.faction_eliminated;
    game_world.resources.faction_morale = snapshot.faction_morale;
    game_world.resources.speech_used = snapshot.speech_used;
    game_world.resources.rng_seed = snapshot.rng_seed;
    game_world.resources.difficulty = snapshot.difficulty;
}

pub fn execute_replay_step(game_world: &mut GameWorld, world: &mut World, action: &ReplayAction) {
    let hex_width = game_world.resources.hex_metrics.hex_width;
    let hex_depth = game_world.resources.hex_metrics.hex_depth;

    match action {
        ReplayAction::TurnStart { faction, turn } => {
            for entity in game_world.query_entities(UNIT).collect::<Vec<_>>() {
                if let Some(unit) = game_world.get_unit(entity) {
                    let mut unit = *unit;
                    unit.has_moved = false;
                    game_world.set_unit(entity, unit);
                }
            }
            game_world.resources.current_faction = *faction;
            game_world.resources.turn_number = *turn;
            game_world.resources.actions_remaining = crate::constants::ACTIONS_PER_TURN;
            game_world.resources.speech_used = false;
        }
        ReplayAction::Move { from, to, .. } | ReplayAction::PortTravel { from, to, .. } => {
            if let Some(&entity) = game_world.resources.unit_position_map.get(from) {
                instant_move(game_world, world, entity, *to, hex_width, hex_depth);
            }
        }
        ReplayAction::Attack {
            attacker_coord,
            defender_coord,
            attacker_faction,
            attacker_survived,
            defender_survived,
            attacker_remaining,
            defender_remaining,
            ..
        } => {
            let attacker_entity = game_world
                .resources
                .unit_position_map
                .get(attacker_coord)
                .copied();
            let defender_entity = game_world
                .resources
                .unit_position_map
                .get(defender_coord)
                .copied();

            if !defender_survived {
                if let Some(defender) = defender_entity {
                    despawn_unit(game_world, world, defender);
                }
            } else if let Some(defender) = defender_entity
                && let Some(unit) = game_world.get_unit(defender)
            {
                let mut unit = *unit;
                unit.soldiers = *defender_remaining;
                game_world.set_unit(defender, unit);
            }

            if !attacker_survived {
                if let Some(attacker) = attacker_entity {
                    despawn_unit(game_world, world, attacker);
                }
            } else if let Some(attacker) = attacker_entity {
                if let Some(unit) = game_world.get_unit(attacker) {
                    let mut unit = *unit;
                    unit.soldiers = *attacker_remaining;
                    game_world.set_unit(attacker, unit);
                }
                if !defender_survived {
                    instant_move(
                        game_world,
                        world,
                        attacker,
                        *defender_coord,
                        hex_width,
                        hex_depth,
                    );
                }
            }

            if *attacker_survived
                && !defender_survived
                && let Some(&tile_entity) = game_world.resources.tile_map.get(defender_coord)
                && let Some(tile) = game_world.get_tile_mut(tile_entity)
            {
                tile.faction = Some(*attacker_faction);
            }
        }
        ReplayAction::Merge {
            source_coord,
            target_coord,
            new_soldiers,
            ..
        } => {
            if let Some(&source) = game_world.resources.unit_position_map.get(source_coord) {
                despawn_unit(game_world, world, source);
            }
            if let Some(&target) = game_world.resources.unit_position_map.get(target_coord)
                && let Some(unit) = game_world.get_unit(target)
            {
                let mut unit = *unit;
                unit.soldiers = *new_soldiers;
                game_world.set_unit(target, unit);
            }
        }
        ReplayAction::Speech { faction } => {
            let boost = crate::constants::SPEECH_MORALE_BOOST;
            let max = crate::constants::MAX_MORALE;
            let units: Vec<_> = game_world
                .query_entities(UNIT)
                .filter(|&entity| {
                    game_world
                        .get_unit(entity)
                        .map(|u| u.faction == *faction)
                        .unwrap_or(false)
                })
                .collect();
            for entity in units {
                if let Some(unit) = game_world.get_unit(entity) {
                    let mut unit = *unit;
                    unit.morale = (unit.morale + boost).min(max);
                    game_world.set_unit(entity, unit);
                }
            }
            game_world.resources.speech_used = true;
        }
        ReplayAction::Reinforcement { entries } => {
            for entry in entries {
                if entry.is_new_unit {
                    spawn_unit(
                        game_world,
                        world,
                        SpawnUnitParams {
                            coord: entry.coord,
                            hex_width,
                            hex_depth,
                            faction: entry.faction,
                            soldiers: entry.new_total,
                            unit_type: entry.unit_type,
                        },
                    );
                } else if let Some(&entity) =
                    game_world.resources.unit_position_map.get(&entry.coord)
                    && let Some(unit) = game_world.get_unit(entity)
                {
                    let mut unit = *unit;
                    unit.soldiers = entry.new_total;
                    game_world.set_unit(entity, unit);
                }
            }
        }
        ReplayAction::Elimination { faction } => {
            let units_to_remove: Vec<_> = game_world
                .query_entities(UNIT)
                .filter(|&entity| {
                    game_world
                        .get_unit(entity)
                        .map(|u| u.faction == *faction)
                        .unwrap_or(false)
                })
                .collect();
            for entity in units_to_remove {
                despawn_unit(game_world, world, entity);
            }

            let tile_entities: Vec<_> = game_world.resources.tile_map.values().copied().collect();
            for entity in tile_entities {
                if let Some(tile) = game_world.get_tile(entity)
                    && tile.faction == Some(*faction)
                    && let Some(tile_mut) = game_world.get_tile_mut(entity)
                {
                    tile_mut.faction = None;
                }
            }

            game_world.resources.faction_eliminated[faction.index()] = true;
        }
    }
}

fn instant_move(
    game_world: &mut GameWorld,
    world: &mut World,
    entity: freecs::Entity,
    destination: HexCoord,
    hex_width: f32,
    hex_depth: f32,
) {
    crate::ecs::update_unit_position(game_world, entity, destination);

    let position = hex_to_world_position(destination.column, destination.row, hex_width, hex_depth);
    let radius = game_world
        .get_unit(entity)
        .map(|u| unit_radius_for_soldiers(u.soldiers))
        .unwrap_or(0.25);
    let unit_position = nalgebra_glm::vec3(
        position.x,
        position.y + radius + UNIT_HEIGHT_OFFSET,
        position.z,
    );

    if let Some(wp) = game_world.get_world_position_mut(entity) {
        wp.0 = unit_position;
    }

    if let Some(engine_entity) = game_world.get_engine_entity(entity) {
        if let Some(transform) = world.core.get_local_transform_mut(engine_entity.0) {
            transform.translation = unit_position;
        }
        mark_local_transform_dirty(world, engine_entity.0);
    }

    if let Some(unit) = game_world.get_unit(entity)
        && let Some(text_entity) = unit.text_entity
    {
        let text_pos = nalgebra_glm::vec3(
            unit_position.x,
            unit_position.y + radius + crate::systems::UNIT_TEXT_HEIGHT_OFFSET,
            unit_position.z,
        );
        if let Some(transform) = world.core.get_local_transform_mut(text_entity) {
            transform.translation = text_pos;
        }
        mark_local_transform_dirty(world, text_entity);
    }
}

pub fn replay_action_description(action: &ReplayAction) -> String {
    match action {
        ReplayAction::TurnStart { faction, turn } => {
            format!("Turn {} — {} begins", turn, faction.name())
        }
        ReplayAction::Move { from, to, .. } => {
            format!(
                "moved ({},{}) to ({},{})",
                from.column, from.row, to.column, to.row
            )
        }
        ReplayAction::PortTravel { from, to, .. } => {
            format!(
                "sailed ({},{}) to ({},{})",
                from.column, from.row, to.column, to.row
            )
        }
        ReplayAction::Attack {
            defender_faction,
            attacker_survived,
            defender_survived,
            ..
        } => {
            if *attacker_survived && !defender_survived {
                format!("destroyed {} unit", defender_faction.name())
            } else if !attacker_survived {
                format!("was repelled by {}", defender_faction.name())
            } else {
                format!("attacked {}", defender_faction.name())
            }
        }
        ReplayAction::Merge { new_soldiers, .. } => {
            format!("merged units (now {} soldiers)", new_soldiers)
        }
        ReplayAction::Speech { faction } => {
            format!("{} gave an inspiring speech", faction.name())
        }
        ReplayAction::Reinforcement { entries } => {
            let total: i32 = entries.iter().map(|e| e.soldiers_added).sum();
            let spawns = entries.iter().filter(|e| e.is_new_unit).count();
            if spawns > 0 {
                format!("reinforcements (+{}, {} new units)", total, spawns)
            } else {
                format!("reinforcements (+{})", total)
            }
        }
        ReplayAction::Elimination { faction } => {
            format!("{} has been eliminated!", faction.name())
        }
    }
}

pub fn replay_action_faction(action: &ReplayAction) -> Faction {
    match action {
        ReplayAction::TurnStart { faction, .. }
        | ReplayAction::Speech { faction }
        | ReplayAction::Elimination { faction } => *faction,
        ReplayAction::Attack {
            attacker_faction, ..
        } => *attacker_faction,
        ReplayAction::Reinforcement { entries } => {
            entries.first().map(|e| e.faction).unwrap_or_default()
        }
        ReplayAction::Move { faction, .. }
        | ReplayAction::PortTravel { faction, .. }
        | ReplayAction::Merge { faction, .. } => *faction,
    }
}
