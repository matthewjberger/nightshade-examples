use crate::ecs::{
    Difficulty, Faction, GameEvents, GameWorld, MOVEMENT, TileType, faction_index,
    get_defense_bonus_at, get_tile_type_at, unit_stats,
};
use crate::hex::{HexCoord, hex_distance};
use crate::map::CAPITAL_POSITIONS;
use crate::systems::action::{GameAction, execute_action};
use crate::systems::calculate_valid_moves;
use nightshade::prelude::*;

fn get_capital_coord(faction: Faction) -> HexCoord {
    let index = faction_index(faction);
    let (col, row, _) = CAPITAL_POSITIONS[index];
    HexCoord { column: col, row }
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

struct ScoredAction {
    action: GameAction,
    score: f32,
}

struct AiUnitState {
    entity: freecs::Entity,
    hex: HexCoord,
    soldiers: i32,
    morale: i32,
}

fn score_attack_actions(
    game_world: &GameWorld,
    unit: &AiUnitState,
    context: &AiTurnContext,
    difficulty: Difficulty,
    player_faction: Faction,
) -> Vec<ScoredAction> {
    let mut actions = Vec::new();

    let adjacent_enemies: Vec<_> = context
        .enemy_units
        .iter()
        .filter(|(_, hex, _, _, _)| hex_distance(unit.hex, *hex) == 1)
        .collect();

    let avoid_ai_vs_ai = difficulty == Difficulty::Hard;
    let prefer_human = matches!(difficulty, Difficulty::Normal | Difficulty::Hard);

    for (enemy_entity, enemy_hex, enemy_faction, enemy_soldiers, enemy_morale) in adjacent_enemies {
        if avoid_ai_vs_ai && *enemy_faction != player_faction {
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

        let attack_threshold = match difficulty {
            Difficulty::Easy => 0.8,
            Difficulty::Normal => 0.7,
            Difficulty::Hard => 0.6,
        };

        let tile_type = get_tile_type_at(game_world, *enemy_hex);
        let is_capital = tile_type == Some(TileType::Capital);
        let threshold = if is_capital {
            attack_threshold - 0.2
        } else {
            attack_threshold
        };

        if win_chance > threshold {
            let mut score = 100.0 * win_chance;
            if is_capital {
                score += 50.0;
            }
            if prefer_human && *enemy_faction == player_faction {
                score += 20.0;
            }

            actions.push(ScoredAction {
                action: GameAction::Attack {
                    attacker: unit.entity,
                    defender: *enemy_entity,
                },
                score,
            });
        }
    }

    actions
}

fn score_defense_actions(
    unit: &AiUnitState,
    my_capital: HexCoord,
    context: &AiTurnContext,
    valid_moves: &[HexCoord],
) -> Vec<ScoredAction> {
    let threat_to_capital = context
        .enemy_units
        .iter()
        .any(|(_, hex, _, _, _)| hex_distance(*hex, my_capital) <= 3);

    if !threat_to_capital || hex_distance(unit.hex, my_capital) <= 2 {
        return Vec::new();
    }

    let mut actions = Vec::new();

    if let Some(&destination) = valid_moves
        .iter()
        .min_by_key(|coord| hex_distance(**coord, my_capital))
    {
        let distance_improvement =
            hex_distance(unit.hex, my_capital) - hex_distance(destination, my_capital);
        if distance_improvement > 0 {
            actions.push(ScoredAction {
                action: GameAction::Move {
                    unit: unit.entity,
                    destination,
                },
                score: 80.0 + distance_improvement as f32 * 5.0,
            });
        }
    }

    actions
}

fn score_capture_actions(
    game_world: &GameWorld,
    unit: &AiUnitState,
    context: &AiTurnContext,
    valid_moves: &[HexCoord],
) -> Vec<ScoredAction> {
    let current_faction = game_world.resources.current_faction;
    let mut actions = Vec::new();

    for &destination in valid_moves {
        let Some(tile_entity) = game_world.resources.tile_map.get(&destination) else {
            continue;
        };
        let Some(tile) = game_world.get_tile(*tile_entity) else {
            continue;
        };

        if tile.faction == Some(current_faction) {
            continue;
        }

        let is_valuable = tile.tile_type == TileType::City || tile.tile_type == TileType::Capital;
        if !is_valuable {
            continue;
        }

        let has_defender = context
            .enemy_units
            .iter()
            .any(|(_, hex, _, _, _)| *hex == destination);
        if has_defender {
            continue;
        }

        let score = if tile.tile_type == TileType::Capital {
            75.0
        } else {
            60.0
        };

        actions.push(ScoredAction {
            action: GameAction::Move {
                unit: unit.entity,
                destination,
            },
            score,
        });
    }

    actions
}

fn score_advance_actions(
    game_world: &GameWorld,
    unit: &AiUnitState,
    context: &AiTurnContext,
    valid_moves: &[HexCoord],
) -> Vec<ScoredAction> {
    let target = if let Some(&capital) = context
        .enemy_capitals
        .iter()
        .min_by_key(|coord| hex_distance(unit.hex, **coord))
    {
        capital
    } else if let Some((_, closest_hex, _, _, _)) = context
        .enemy_units
        .iter()
        .min_by_key(|(_, hex, _, _, _)| hex_distance(unit.hex, *hex))
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
            .min_by_key(|hex| hex_distance(unit.hex, **hex))
        {
            closest
        } else {
            return Vec::new();
        }
    };

    let mut actions = Vec::new();

    if let Some(&destination) = valid_moves
        .iter()
        .min_by_key(|coord| hex_distance(**coord, target))
    {
        let current_distance = hex_distance(unit.hex, target);
        let new_distance = hex_distance(destination, target);
        let improvement = current_distance - new_distance;
        let score = 30.0 + improvement.max(0) as f32 * 10.0;

        actions.push(ScoredAction {
            action: GameAction::Move {
                unit: unit.entity,
                destination,
            },
            score,
        });
    }

    actions
}

fn evaluate_actions(
    game_world: &GameWorld,
    unit: &AiUnitState,
    context: &AiTurnContext,
    difficulty: Difficulty,
    player_faction: Faction,
    valid_moves: &[HexCoord],
    my_capital: HexCoord,
) -> Vec<ScoredAction> {
    let mut all_actions = Vec::new();

    all_actions.extend(score_attack_actions(
        game_world,
        unit,
        context,
        difficulty,
        player_faction,
    ));

    all_actions.extend(score_defense_actions(
        unit,
        my_capital,
        context,
        valid_moves,
    ));

    all_actions.extend(score_capture_actions(
        game_world,
        unit,
        context,
        valid_moves,
    ));

    all_actions.extend(score_advance_actions(
        game_world,
        unit,
        context,
        valid_moves,
    ));

    all_actions
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

fn mark_unit_skipped(game_world: &mut GameWorld, unit_entity: freecs::Entity) {
    if let Some(unit_data) = game_world.get_unit(unit_entity) {
        let mut unit_data = *unit_data;
        unit_data.has_moved = true;
        game_world.set_unit(unit_entity, unit_data);
    }
    game_world.resources.current_unit_index += 1;
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

    let difficulty = game_world.resources.difficulty;
    let rng_seed = game_world.resources.rng_seed;
    let turn_number = game_world.resources.turn_number;

    if should_make_suboptimal_move(difficulty, rng_seed, turn_number, current_index) {
        mark_unit_skipped(game_world, unit_entity);
        return false;
    }

    let my_capital = get_capital_coord(current_faction);
    let context = build_ai_context(game_world, current_faction);
    let movement_range = unit_stats(unit.unit_type).movement_range;
    let valid_moves = calculate_valid_moves(game_world, unit_entity, unit_hex, movement_range);

    let ai_unit = AiUnitState {
        entity: unit_entity,
        hex: unit_hex,
        soldiers: unit.soldiers,
        morale: unit.morale,
    };

    let mut scored_actions = evaluate_actions(
        game_world,
        &ai_unit,
        &context,
        difficulty,
        player_faction,
        &valid_moves,
        my_capital,
    );

    scored_actions.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(best) = scored_actions.into_iter().next() {
        execute_action(game_world, world, best.action, events);
        game_world.resources.current_unit_index += 1;
    } else {
        mark_unit_skipped(game_world, unit_entity);
    }

    false
}
