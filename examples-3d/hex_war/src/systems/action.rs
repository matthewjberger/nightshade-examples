use crate::constants::{MAX_MORALE, SPEECH_MORALE_BOOST};
use crate::ecs::{CombatEvent, GameEvents, GameWorld, SpeechEvent, UNIT, unit_stats};
use crate::hex::{HexCoord, hex_distance};
use crate::replay::ReplayAction;
use crate::selection::{clear_selection, get_selected_unit, select_unit};
use crate::systems::{despawn_unit, move_unit_to, resolve_combat, spawn_merge_popup};
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

fn can_reach_tile(game_world: &GameWorld, source_hex: HexCoord, target_hex: HexCoord) -> bool {
    let reachable_tiles = &game_world.resources.valid_move_tiles;

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

    match action {
        GameAction::Move { unit, destination } => {
            let from = game_world
                .get_hex_position(unit)
                .map(|h| h.0)
                .unwrap_or_default();
            move_unit_to(game_world, unit, destination);
            finalize_unit_action(game_world, unit);
            events.replay_actions.push(ReplayAction::Move {
                faction,
                from,
                to: destination,
            });
        }
        GameAction::PortTravel { unit, destination } => {
            let from = game_world
                .get_hex_position(unit)
                .map(|h| h.0)
                .unwrap_or_default();
            move_unit_to(game_world, unit, destination);
            finalize_unit_action(game_world, unit);
            events.replay_actions.push(ReplayAction::PortTravel {
                faction,
                from,
                to: destination,
            });
        }
        GameAction::Attack { attacker, defender } => {
            let attacker_coord = game_world
                .get_hex_position(attacker)
                .map(|h| h.0)
                .unwrap_or_default();
            let defender_coord = game_world
                .get_hex_position(defender)
                .map(|h| h.0)
                .unwrap_or_default();
            let defender_faction_val = game_world
                .get_unit(defender)
                .map(|u| u.faction)
                .unwrap_or_default();

            if let Some(result) = resolve_combat(game_world, world, attacker, defender) {
                let attacker_remaining = if result.attacker_survived {
                    game_world
                        .get_unit(attacker)
                        .map(|u| u.soldiers)
                        .unwrap_or(0)
                } else {
                    0
                };
                let defender_remaining = if result.defender_survived {
                    game_world
                        .get_unit(defender)
                        .map(|u| u.soldiers)
                        .unwrap_or(0)
                } else {
                    0
                };

                events.combat_events.push(CombatEvent {
                    attacker_faction: result.attacker_faction,
                    defender_faction: result.defender_faction,
                    attacker_survived: result.attacker_survived,
                    defender_survived: result.defender_survived,
                });
                events.replay_actions.push(ReplayAction::Attack {
                    attacker_coord,
                    defender_coord,
                    attacker_faction: faction,
                    defender_faction: defender_faction_val,
                    attacker_survived: result.attacker_survived,
                    defender_survived: result.defender_survived,
                    attacker_remaining,
                    defender_remaining,
                });
            }
            finalize_unit_action(game_world, attacker);
        }
        GameAction::Merge { source, target } => {
            let source_coord = game_world
                .get_hex_position(source)
                .map(|h| h.0)
                .unwrap_or_default();
            let target_coord = game_world
                .get_hex_position(target)
                .map(|h| h.0)
                .unwrap_or_default();
            if let Some(result) = merge_units(game_world, world, source, target) {
                let new_soldiers = game_world.get_unit(target).map(|u| u.soldiers).unwrap_or(0);
                if result.soldiers_gained > 0 {
                    spawn_merge_popup(game_world, world, result.position, result.soldiers_gained);
                }
                finalize_unit_action(game_world, source);
                events.replay_actions.push(ReplayAction::Merge {
                    faction,
                    source_coord,
                    target_coord,
                    new_soldiers,
                });
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
            events.replay_actions.push(ReplayAction::Speech { faction });
        }
    }
}

fn try_move(
    game_world: &GameWorld,
    selected: freecs::Entity,
    hovered_tile: HexCoord,
) -> Option<InputResult> {
    if game_world.resources.actions_remaining == 0 {
        return None;
    }
    if !game_world
        .resources
        .valid_move_tiles
        .contains(&hovered_tile)
    {
        return None;
    }

    let source_hex = game_world.get_hex_position(selected).map(|h| h.0);
    let is_port_to_port = source_hex
        .is_some_and(|src| is_port_tile(game_world, src) && is_port_tile(game_world, hovered_tile));

    Some(if is_port_to_port {
        InputResult::Execute(GameAction::PortTravel {
            unit: selected,
            destination: hovered_tile,
        })
    } else {
        InputResult::Execute(GameAction::Move {
            unit: selected,
            destination: hovered_tile,
        })
    })
}

fn try_attack(
    game_world: &GameWorld,
    selected: freecs::Entity,
    target: freecs::Entity,
    hovered_tile: HexCoord,
) -> Option<InputResult> {
    if game_world.resources.actions_remaining == 0 {
        return None;
    }
    let target_data = game_world.get_unit(target)?;
    if target_data.faction == game_world.resources.current_faction {
        return None;
    }
    let selected_hex = game_world.get_hex_position(selected)?.0;
    if hex_distance(selected_hex, hovered_tile) != 1 {
        return None;
    }
    Some(InputResult::Execute(GameAction::Attack {
        attacker: selected,
        defender: target,
    }))
}

fn try_merge(
    game_world: &GameWorld,
    selected: freecs::Entity,
    target: freecs::Entity,
    hovered_tile: HexCoord,
) -> Option<InputResult> {
    if game_world.resources.actions_remaining == 0 {
        return None;
    }
    let target_data = game_world.get_unit(target)?;
    if target_data.faction != game_world.resources.current_faction || target == selected {
        return None;
    }
    let selected_data = game_world.get_unit(selected)?;
    if selected_data.has_moved || selected_data.unit_type != target_data.unit_type {
        return None;
    }
    let source_hex = game_world.get_hex_position(selected)?.0;
    if !can_reach_tile(game_world, source_hex, hovered_tile) {
        return None;
    }
    Some(InputResult::Execute(GameAction::Merge {
        source: selected,
        target,
    }))
}

fn try_reselect_or_deselect(
    game_world: &GameWorld,
    selected: freecs::Entity,
    clicked: freecs::Entity,
) -> Option<InputResult> {
    if clicked == selected {
        return Some(InputResult::Deselect);
    }
    let clicked_data = game_world.get_unit(clicked)?;
    if clicked_data.faction != game_world.resources.current_faction {
        return None;
    }
    if game_world.get_unit(selected).is_some_and(|u| u.has_moved) {
        return Some(InputResult::Select(clicked));
    }
    None
}

fn try_select_fresh(game_world: &GameWorld, hovered_tile: HexCoord) -> Option<InputResult> {
    if game_world.resources.actions_remaining == 0 {
        return None;
    }
    let clicked = get_unit_at_tile(game_world, hovered_tile)?;
    let unit_data = game_world.get_unit(clicked)?;
    if unit_data.faction != game_world.resources.current_faction {
        return None;
    }
    Some(InputResult::Select(clicked))
}

pub fn determine_action(game_world: &GameWorld, hovered_tile: HexCoord) -> InputResult {
    let selected = get_selected_unit(game_world);
    let clicked = get_unit_at_tile(game_world, hovered_tile);

    if let Some(selected) = selected {
        if let Some(result) = try_move(game_world, selected, hovered_tile) {
            return result;
        }
        if let Some(clicked) = clicked {
            if let Some(result) = try_attack(game_world, selected, clicked, hovered_tile) {
                return result;
            }
            if let Some(result) = try_merge(game_world, selected, clicked, hovered_tile) {
                return result;
            }
            if let Some(result) = try_reselect_or_deselect(game_world, selected, clicked) {
                return result;
            }
        }
    } else if let Some(result) = try_select_fresh(game_world, hovered_tile) {
        return result;
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
