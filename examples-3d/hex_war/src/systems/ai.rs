use crate::ecs::{
    CombatEvent, Difficulty, Faction, GameEvents, GameWorld, MOVEMENT, TileType, faction_index,
    get_defense_bonus_at, get_tile_type_at,
};
use crate::hex::{HexCoord, hex_distance};
use crate::map::CAPITAL_POSITIONS;
use crate::systems::{calculate_valid_moves, move_unit_to, resolve_combat};
use nightshade::prelude::*;

fn get_capital_coord(faction: Faction) -> HexCoord {
    let index = faction_index(faction);
    let (col, row, _) = CAPITAL_POSITIONS[index];
    HexCoord { column: col, row }
}

fn get_attack_threshold(difficulty: Difficulty, is_capital: bool) -> f32 {
    let base = match difficulty {
        Difficulty::Easy => 0.8,
        Difficulty::Normal => 0.7,
        Difficulty::Hard => 0.6,
    };
    if is_capital { base - 0.2 } else { base }
}

fn should_make_suboptimal_move(
    difficulty: Difficulty,
    rng_seed: u32,
    turn: u32,
    unit_index: usize,
) -> bool {
    if difficulty != Difficulty::Easy {
        return false;
    }
    let hash = rng_seed
        .wrapping_mul(31)
        .wrapping_add(turn)
        .wrapping_mul(17)
        .wrapping_add(unit_index as u32);
    hash.is_multiple_of(5)
}

fn should_prefer_human_target(difficulty: Difficulty) -> bool {
    matches!(difficulty, Difficulty::Normal | Difficulty::Hard)
}

fn should_avoid_ai_vs_ai(difficulty: Difficulty) -> bool {
    difficulty == Difficulty::Hard
}

fn calculate_win_chance(
    attacker_soldiers: i32,
    attacker_morale: i32,
    defender_soldiers: i32,
    defender_morale: i32,
    defense_bonus: f32,
) -> f32 {
    let attacker_strength = attacker_soldiers as f32 * (1.0 + attacker_morale as f32 / 100.0);
    let defender_strength =
        defender_soldiers as f32 * (1.0 + defender_morale as f32 / 100.0) * defense_bonus;

    attacker_strength / (attacker_strength + defender_strength)
}

struct AiTurnContext {
    enemy_units: Vec<(freecs::Entity, HexCoord, Faction, i32, i32)>,
    enemy_capitals: Vec<HexCoord>,
}

fn build_ai_context(game_world: &GameWorld, current_faction: Faction) -> AiTurnContext {
    let enemy_units: Vec<(freecs::Entity, HexCoord, Faction, i32, i32)> = game_world
        .resources
        .unit_position_map
        .iter()
        .filter_map(|(&coord, &entity)| {
            let unit = game_world.get_unit(entity)?;
            if unit.faction == current_faction {
                return None;
            }
            Some((entity, coord, unit.faction, unit.soldiers, unit.morale))
        })
        .collect();

    let enemy_capitals: Vec<HexCoord> = [
        Faction::Redosia,
        Faction::Violetnam,
        Faction::Bluegaria,
        Faction::Greenland,
    ]
    .iter()
    .filter(|&&faction| {
        faction != current_faction
            && !game_world.resources.faction_eliminated[faction_index(faction)]
    })
    .map(|&faction| get_capital_coord(faction))
    .collect();

    AiTurnContext {
        enemy_units,
        enemy_capitals,
    }
}

pub fn build_turn_order(game_world: &mut GameWorld) {
    let current_faction = game_world.resources.current_faction;

    let units: Vec<freecs::Entity> = game_world
        .resources
        .unit_position_map
        .values()
        .filter(|&&entity| {
            game_world
                .get_unit(entity)
                .map(|unit| unit.faction == current_faction)
                .unwrap_or(false)
        })
        .copied()
        .collect();

    game_world.resources.turn_order = units;
    game_world.resources.current_unit_index = 0;
}

pub fn ai_turn_system(
    game_world: &mut GameWorld,
    world: &mut World,
    player_faction: Faction,
    events: &mut GameEvents,
) -> bool {
    let current_faction = game_world.resources.current_faction;

    if current_faction == player_faction {
        return false;
    }

    let has_active_movement = game_world.query_entities(MOVEMENT).next().is_some();
    if has_active_movement {
        return false;
    }

    if game_world.resources.actions_remaining == 0 {
        return true;
    }

    if game_world.resources.turn_order.is_empty() {
        return true;
    }

    let current_index = game_world.resources.current_unit_index;
    if current_index >= game_world.resources.turn_order.len() {
        return true;
    }

    let unit_entity = game_world.resources.turn_order[current_index];

    let Some(unit_hex) = game_world.get_hex_position(unit_entity).map(|h| h.0) else {
        game_world.resources.current_unit_index += 1;
        return false;
    };

    let Some(unit) = game_world.get_unit(unit_entity).copied() else {
        game_world.resources.current_unit_index += 1;
        return false;
    };

    if unit.has_moved {
        game_world.resources.current_unit_index += 1;
        return false;
    }

    let my_capital = get_capital_coord(current_faction);
    let context = build_ai_context(game_world, current_faction);

    let difficulty = game_world.resources.difficulty;
    let rng_seed = game_world.resources.rng_seed;
    let turn_number = game_world.resources.turn_number;

    if should_make_suboptimal_move(difficulty, rng_seed, turn_number, current_index) {
        if let Some(unit_data) = game_world.get_unit(unit_entity) {
            let mut unit_data = *unit_data;
            unit_data.has_moved = true;
            game_world.set_unit(unit_entity, unit_data);
        }
        game_world.resources.current_unit_index += 1;
        return false;
    }

    let adjacent_enemies: Vec<_> = context
        .enemy_units
        .iter()
        .filter(|(_, hex, _, _, _)| hex_distance(unit_hex, *hex) == 1)
        .copied()
        .collect();

    let mut sorted_enemies = adjacent_enemies;
    if should_prefer_human_target(difficulty) {
        sorted_enemies.sort_by_key(
            |(_, _, faction, _, _)| {
                if *faction == player_faction { 0 } else { 1 }
            },
        );
    }

    for (enemy_entity, enemy_hex, enemy_faction, enemy_soldiers, enemy_morale) in &sorted_enemies {
        if should_avoid_ai_vs_ai(difficulty) && *enemy_faction != player_faction {
            continue;
        }

        let defense_bonus = get_defense_bonus_at(game_world, *enemy_hex);
        let win_chance = calculate_win_chance(
            unit.soldiers,
            unit.morale,
            *enemy_soldiers,
            *enemy_morale,
            defense_bonus,
        );

        let tile_type = get_tile_type_at(game_world, *enemy_hex);
        let is_capital = tile_type == Some(TileType::Capital);

        let attack_threshold = get_attack_threshold(difficulty, is_capital);

        if win_chance > attack_threshold {
            if let Some(result) = resolve_combat(game_world, world, unit_entity, *enemy_entity) {
                events.combat_events.push(CombatEvent {
                    attacker_faction: result.attacker_faction,
                    defender_faction: result.defender_faction,
                    attacker_survived: result.attacker_survived,
                    defender_survived: result.defender_survived,
                });
            }
            if let Some(unit_data) = game_world.get_unit(unit_entity) {
                let mut unit_data = *unit_data;
                unit_data.has_moved = true;
                game_world.set_unit(unit_entity, unit_data);
            }
            game_world.resources.actions_remaining -= 1;
            game_world.resources.current_unit_index += 1;
            return false;
        }
    }

    let valid_moves = calculate_valid_moves(game_world, unit_entity, unit_hex, unit.movement_range);

    if valid_moves.is_empty() {
        if let Some(unit_data) = game_world.get_unit(unit_entity) {
            let mut unit_data = *unit_data;
            unit_data.has_moved = true;
            game_world.set_unit(unit_entity, unit_data);
        }
        game_world.resources.current_unit_index += 1;
        return false;
    }

    let threat_to_capital = context
        .enemy_units
        .iter()
        .any(|(_, hex, _, _, _)| hex_distance(*hex, my_capital) <= 3);

    if threat_to_capital && hex_distance(unit_hex, my_capital) > 2 {
        let best_move = valid_moves
            .iter()
            .min_by_key(|coord| hex_distance(**coord, my_capital))
            .copied();

        if let Some(destination) = best_move {
            move_unit_to(game_world, unit_entity, destination);
            if let Some(unit_data) = game_world.get_unit(unit_entity) {
                let mut unit_data = *unit_data;
                unit_data.has_moved = true;
                game_world.set_unit(unit_entity, unit_data);
            }
            game_world.resources.actions_remaining -= 1;
            game_world.resources.current_unit_index += 1;
            return false;
        }
    }

    let undefended_cities: Vec<HexCoord> = game_world
        .resources
        .tile_map
        .iter()
        .filter_map(|(&coord, &entity)| {
            let tile = game_world.get_tile(entity)?;
            if (tile.tile_type == TileType::City || tile.tile_type == TileType::Capital)
                && tile.faction != Some(current_faction)
            {
                let has_enemy = context
                    .enemy_units
                    .iter()
                    .any(|(_, eh, _, _, _)| *eh == coord);
                if !has_enemy {
                    return Some(coord);
                }
            }
            None
        })
        .collect();

    for city in &undefended_cities {
        if valid_moves.contains(city) {
            move_unit_to(game_world, unit_entity, *city);
            if let Some(unit_data) = game_world.get_unit(unit_entity) {
                let mut unit_data = *unit_data;
                unit_data.has_moved = true;
                game_world.set_unit(unit_entity, unit_data);
            }
            game_world.resources.actions_remaining -= 1;
            game_world.resources.current_unit_index += 1;
            return false;
        }
    }

    let closest_enemy_capital = context
        .enemy_capitals
        .iter()
        .min_by_key(|coord| hex_distance(unit_hex, **coord));

    let target = if let Some(&capital) = closest_enemy_capital {
        capital
    } else if let Some((_, closest_hex, _, _, _)) = context
        .enemy_units
        .iter()
        .min_by_key(|(_, hex, _, _, _)| hex_distance(unit_hex, *hex))
    {
        *closest_hex
    } else {
        let unclaimed: Vec<_> = game_world
            .resources
            .tile_map
            .iter()
            .filter_map(|(&coord, &entity)| {
                let tile = game_world.get_tile(entity)?;
                if tile.tile_type != TileType::Sea && tile.faction.is_none() {
                    Some(coord)
                } else {
                    None
                }
            })
            .collect();

        if let Some(&closest) = unclaimed
            .iter()
            .min_by_key(|hex| hex_distance(unit_hex, **hex))
        {
            closest
        } else {
            game_world.resources.current_unit_index += 1;
            return false;
        }
    };

    let best_move = valid_moves
        .iter()
        .min_by_key(|coord| hex_distance(**coord, target))
        .copied();

    if let Some(destination) = best_move {
        move_unit_to(game_world, unit_entity, destination);
        if let Some(unit_data) = game_world.get_unit(unit_entity) {
            let mut unit_data = *unit_data;
            unit_data.has_moved = true;
            game_world.set_unit(unit_entity, unit_data);
        }
        game_world.resources.actions_remaining -= 1;
    }

    game_world.resources.current_unit_index += 1;
    false
}
