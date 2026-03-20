use crate::constants::{MAX_MORALE, SPEECH_MORALE_BOOST};
use crate::ecs::{
    ActionRecord, CombatEvent, Faction, GameEvents, GameWorld, SpeechEvent, UNIT, unit_stats,
};
use crate::hex::{HexCoord, hex_distance};
use crate::selection::{clear_selection, get_selected_unit, select_unit};
use crate::systems::{
    calculate_valid_moves, despawn_unit, move_unit_to, resolve_combat, spawn_merge_popup,
};
use nightshade::prelude::*;

pub enum GameAction {
    Move {
        unit: freecs::Entity,
        destination: HexCoord,
    },
    PortTravel {
        unit: freecs::Entity,
        destination: HexCoord,
    },
    Attack {
        attacker: freecs::Entity,
        defender: freecs::Entity,
    },
    Merge {
        source: freecs::Entity,
        target: freecs::Entity,
    },
    Speech,
}

pub enum InputResult {
    Execute(GameAction),
    Select(freecs::Entity),
    Deselect,
    Nothing,
}

pub struct MergeResult {
    pub soldiers_gained: i32,
    pub position: Vec3,
}

fn is_port_tile(game_world: &GameWorld, coord: HexCoord) -> bool {
    game_world.resources.port_tiles.contains(&coord)
}

fn can_reach_tile(
    game_world: &GameWorld,
    source_entity: freecs::Entity,
    source_hex: HexCoord,
    target_hex: HexCoord,
    movement_range: i32,
) -> bool {
    let reachable_tiles =
        calculate_valid_moves(game_world, source_entity, source_hex, movement_range);

    let adjacent_to_reachable = reachable_tiles
        .iter()
        .any(|&tile| hex_distance(tile, target_hex) <= 1);
    let directly_adjacent = hex_distance(source_hex, target_hex) == 1;

    adjacent_to_reachable || directly_adjacent
}

fn get_unit_at_tile(game_world: &GameWorld, coord: HexCoord) -> Option<freecs::Entity> {
    game_world.resources.unit_position_map.get(&coord).copied()
}

fn merge_units(
    game_world: &mut GameWorld,
    world: &mut World,
    source_entity: freecs::Entity,
    target_entity: freecs::Entity,
) -> Option<MergeResult> {
    let source_unit = game_world.get_unit(source_entity).copied()?;
    let target_unit = game_world.get_unit(target_entity).copied()?;

    if source_unit.faction != target_unit.faction {
        return None;
    }

    if source_unit.unit_type != target_unit.unit_type {
        return None;
    }

    let position = game_world
        .get_world_position(target_entity)
        .map(|p| p.0)
        .unwrap_or_default();

    let max_soldiers = unit_stats(target_unit.unit_type).max_soldiers;
    let total_soldiers = source_unit.soldiers + target_unit.soldiers;
    let new_soldiers = total_soldiers.min(max_soldiers);
    let soldiers_gained = new_soldiers - target_unit.soldiers;

    let weighted_morale = (source_unit.soldiers * source_unit.morale
        + target_unit.soldiers * target_unit.morale)
        / total_soldiers;

    if let Some(unit) = game_world.get_unit_mut(target_entity) {
        unit.soldiers = new_soldiers;
        unit.morale = weighted_morale;
    }

    despawn_unit(game_world, world, source_entity);
    Some(MergeResult {
        soldiers_gained,
        position,
    })
}

fn finalize_unit_action(game_world: &mut GameWorld, unit: freecs::Entity) {
    if let Some(unit_data) = game_world.get_unit(unit) {
        let mut unit_data = *unit_data;
        unit_data.has_moved = true;
        game_world.set_unit(unit, unit_data);
    }
    game_world.resources.actions_remaining -= 1;
}

pub fn execute_action(
    game_world: &mut GameWorld,
    world: &mut World,
    action: GameAction,
    events: &mut GameEvents,
) {
    let faction = game_world.resources.current_faction;
    let turn = game_world.resources.turn_number;

    match action {
        GameAction::Move { unit, destination } => {
            let from = game_world
                .get_hex_position(unit)
                .map(|h| h.0)
                .unwrap_or_default();
            move_unit_to(game_world, unit, destination);
            finalize_unit_action(game_world, unit);
            record_action(
                events,
                faction,
                turn,
                format!(
                    "moved unit ({},{}) to ({},{})",
                    from.column, from.row, destination.column, destination.row
                ),
            );
        }
        GameAction::PortTravel { unit, destination } => {
            let from = game_world
                .get_hex_position(unit)
                .map(|h| h.0)
                .unwrap_or_default();
            move_unit_to(game_world, unit, destination);
            finalize_unit_action(game_world, unit);
            record_action(
                events,
                faction,
                turn,
                format!(
                    "sailed from ({},{}) to ({},{})",
                    from.column, from.row, destination.column, destination.row
                ),
            );
        }
        GameAction::Attack { attacker, defender } => {
            let defender_faction = game_world
                .get_unit(defender)
                .map(|u| u.faction)
                .unwrap_or_default();
            if let Some(result) = resolve_combat(game_world, world, attacker, defender) {
                events.combat_events.push(CombatEvent {
                    attacker_faction: result.attacker_faction,
                    defender_faction: result.defender_faction,
                    attacker_survived: result.attacker_survived,
                    defender_survived: result.defender_survived,
                });
                let outcome_text = if result.attacker_survived && !result.defender_survived {
                    format!("destroyed {} unit", defender_faction.name())
                } else if !result.attacker_survived {
                    format!("was repelled by {}", defender_faction.name())
                } else {
                    format!("attacked {}", defender_faction.name())
                };
                record_action(events, faction, turn, outcome_text);
            }
            finalize_unit_action(game_world, attacker);
        }
        GameAction::Merge { source, target } => {
            if let Some(result) = merge_units(game_world, world, source, target) {
                if result.soldiers_gained > 0 {
                    spawn_merge_popup(game_world, world, result.position, result.soldiers_gained);
                }
                finalize_unit_action(game_world, source);
                record_action(
                    events,
                    faction,
                    turn,
                    format!("merged units (+{})", result.soldiers_gained),
                );
            }
        }
        GameAction::Speech => {
            if game_world.resources.speech_used {
                return;
            }

            let current_faction = game_world.resources.current_faction;

            let faction_units: Vec<_> = game_world
                .query_entities(UNIT)
                .filter(|entity| {
                    game_world
                        .get_unit(*entity)
                        .map(|unit| unit.faction == current_faction)
                        .unwrap_or(false)
                })
                .collect();

            for entity in faction_units {
                if let Some(unit) = game_world.get_unit(entity) {
                    let mut unit = *unit;
                    unit.morale = (unit.morale + SPEECH_MORALE_BOOST).min(MAX_MORALE);
                    game_world.set_unit(entity, unit);
                }
            }

            game_world.resources.speech_used = true;
            events.speech_events.push(SpeechEvent {
                faction: current_faction,
            });
            record_action(
                events,
                faction,
                turn,
                "gave an inspiring speech".to_string(),
            );
        }
    }
}

fn record_action(events: &mut GameEvents, faction: Faction, turn: u32, description: String) {
    events.action_history.push(ActionRecord {
        faction,
        turn,
        description,
    });
}

pub fn determine_action(game_world: &GameWorld, hovered_tile: HexCoord) -> InputResult {
    let current_faction = game_world.resources.current_faction;
    let actions_remaining = game_world.resources.actions_remaining;
    let selected_unit = get_selected_unit(game_world);
    let unit_at_tile = get_unit_at_tile(game_world, hovered_tile);

    if let Some(selected) = selected_unit {
        if game_world
            .resources
            .valid_move_tiles
            .contains(&hovered_tile)
            && actions_remaining > 0
        {
            let source_hex = game_world.get_hex_position(selected).map(|h| h.0);
            let is_port_to_port = source_hex.is_some_and(|src| {
                is_port_tile(game_world, src) && is_port_tile(game_world, hovered_tile)
            });

            return if is_port_to_port {
                InputResult::Execute(GameAction::PortTravel {
                    unit: selected,
                    destination: hovered_tile,
                })
            } else {
                InputResult::Execute(GameAction::Move {
                    unit: selected,
                    destination: hovered_tile,
                })
            };
        }

        if let Some(clicked_unit) = unit_at_tile
            && let Some(clicked_unit_data) = game_world.get_unit(clicked_unit).copied()
        {
            if clicked_unit_data.faction != current_faction && actions_remaining > 0 {
                let selected_hex = game_world.get_hex_position(selected).map(|h| h.0);
                let is_adjacent = selected_hex
                    .map(|hex| hex_distance(hex, hovered_tile) == 1)
                    .unwrap_or(false);

                if is_adjacent {
                    return InputResult::Execute(GameAction::Attack {
                        attacker: selected,
                        defender: clicked_unit,
                    });
                }
            }

            if clicked_unit_data.faction == current_faction && clicked_unit != selected {
                if let Some(selected_unit_data) = game_world.get_unit(selected).copied()
                    && !selected_unit_data.has_moved
                    && actions_remaining > 0
                    && selected_unit_data.unit_type == clicked_unit_data.unit_type
                    && let Some(source_hex) = game_world.get_hex_position(selected).map(|h| h.0)
                    && can_reach_tile(
                        game_world,
                        selected,
                        source_hex,
                        hovered_tile,
                        unit_stats(selected_unit_data.unit_type).movement_range,
                    )
                {
                    return InputResult::Execute(GameAction::Merge {
                        source: selected,
                        target: clicked_unit,
                    });
                }

                if game_world.get_unit(selected).is_some_and(|u| u.has_moved) {
                    return InputResult::Select(clicked_unit);
                }
            } else if clicked_unit == selected {
                return InputResult::Deselect;
            }
        }
    } else if let Some(clicked_unit) = unit_at_tile
        && let Some(unit_data) = game_world.get_unit(clicked_unit)
        && unit_data.faction == current_faction
        && actions_remaining > 0
    {
        return InputResult::Select(clicked_unit);
    }

    InputResult::Nothing
}

pub fn apply_input_result(
    game_world: &mut GameWorld,
    world: &mut World,
    result: InputResult,
    events: &mut GameEvents,
) {
    match result {
        InputResult::Execute(action) => {
            execute_action(game_world, world, action, events);
            clear_selection(game_world);
        }
        InputResult::Select(entity) => select_unit(game_world, entity),
        InputResult::Deselect => clear_selection(game_world),
        InputResult::Nothing => {}
    }
}
