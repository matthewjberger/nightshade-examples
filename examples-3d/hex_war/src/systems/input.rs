use crate::constants::MAX_SOLDIERS;
use crate::ecs::{CombatEvent, Faction, GameEvents, GameWorld};
use crate::hex::{HexCoord, hex_distance};
use crate::selection::{clear_selection, get_selected_unit, select_unit};
use crate::systems::{
    calculate_valid_moves, despawn_unit, move_unit_to, resolve_combat, spawn_merge_popup,
};
use nightshade::prelude::*;

fn get_friendly_ports(game_world: &GameWorld, faction: Faction) -> Vec<HexCoord> {
    game_world
        .resources
        .port_tiles
        .iter()
        .filter(|coord| {
            game_world
                .resources
                .tile_map
                .get(coord)
                .and_then(|&entity| game_world.get_tile(entity))
                .is_some_and(|tile| tile.faction == Some(faction))
        })
        .copied()
        .collect()
}

fn is_unit_on_friendly_port(game_world: &GameWorld, unit_hex: HexCoord, faction: Faction) -> bool {
    game_world.resources.port_tiles.contains(&unit_hex)
        && game_world
            .resources
            .tile_map
            .get(&unit_hex)
            .and_then(|&entity| game_world.get_tile(entity))
            .is_some_and(|tile| tile.faction == Some(faction))
}

pub struct MergeResult {
    pub soldiers_gained: i32,
    pub position: Vec3,
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

    let position = game_world
        .get_world_position(target_entity)
        .map(|p| p.0)
        .unwrap_or_default();

    let total_soldiers = source_unit.soldiers + target_unit.soldiers;
    let new_soldiers = total_soldiers.min(MAX_SOLDIERS);
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

pub fn input_system(game_world: &mut GameWorld, world: &mut World, events: &mut GameEvents) {
    let mouse = &world.resources.input.mouse;
    let left_clicked = mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let right_clicked = mouse.state.contains(MouseState::RIGHT_JUST_PRESSED);

    if right_clicked {
        clear_selection(game_world);
        return;
    }

    if !left_clicked {
        return;
    }

    let Some(hovered_tile) = game_world.resources.hovered_tile else {
        return;
    };

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
            move_unit_to(game_world, selected, hovered_tile);
            if let Some(unit) = game_world.get_unit(selected) {
                let mut unit = *unit;
                unit.has_moved = true;
                game_world.set_unit(selected, unit);
            }
            game_world.resources.actions_remaining -= 1;
            clear_selection(game_world);
            return;
        }

        if let Some(selected_unit_data) = game_world.get_unit(selected).copied()
            && !selected_unit_data.has_moved
            && actions_remaining > 0
            && let Some(source_hex) = game_world.get_hex_position(selected).map(|h| h.0)
            && is_unit_on_friendly_port(game_world, source_hex, current_faction)
        {
            let friendly_ports = get_friendly_ports(game_world, current_faction);
            let is_dest_port = friendly_ports.contains(&hovered_tile);
            let is_dest_unoccupied = unit_at_tile.is_none();
            let is_different_port = source_hex != hovered_tile;

            if is_dest_port && is_dest_unoccupied && is_different_port {
                move_unit_to(game_world, selected, hovered_tile);
                if let Some(unit) = game_world.get_unit(selected) {
                    let mut unit = *unit;
                    unit.has_moved = true;
                    game_world.set_unit(selected, unit);
                }
                game_world.resources.actions_remaining -= 1;
                clear_selection(game_world);
                return;
            }
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
                    if let Some(result) = resolve_combat(game_world, world, selected, clicked_unit)
                    {
                        events.combat_events.push(CombatEvent {
                            attacker_faction: result.attacker_faction,
                            defender_faction: result.defender_faction,
                            attacker_survived: result.attacker_survived,
                            defender_survived: result.defender_survived,
                        });
                        game_world.resources.actions_remaining -= 1;
                    }
                    clear_selection(game_world);
                    return;
                }
            }

            if clicked_unit_data.faction == current_faction && clicked_unit != selected {
                if let Some(selected_unit_data) = game_world.get_unit(selected).copied()
                    && !selected_unit_data.has_moved
                    && actions_remaining > 0
                    && let Some(source_hex) = game_world.get_hex_position(selected).map(|h| h.0)
                    && can_reach_tile(
                        game_world,
                        selected,
                        source_hex,
                        hovered_tile,
                        selected_unit_data.movement_range,
                    )
                    && let Some(result) = merge_units(game_world, world, selected, clicked_unit)
                {
                    if result.soldiers_gained > 0 {
                        spawn_merge_popup(
                            game_world,
                            world,
                            result.position,
                            result.soldiers_gained,
                        );
                    }
                    game_world.resources.actions_remaining -= 1;
                    clear_selection(game_world);
                    return;
                }

                if game_world.get_unit(selected).is_some_and(|u| u.has_moved) {
                    select_unit(game_world, clicked_unit);
                }
            } else if clicked_unit == selected {
                clear_selection(game_world);
            }
        }
    } else if let Some(clicked_unit) = unit_at_tile
        && let Some(unit_data) = game_world.get_unit(clicked_unit)
        && unit_data.faction == current_faction
        && actions_remaining > 0
    {
        select_unit(game_world, clicked_unit);
    }
}
