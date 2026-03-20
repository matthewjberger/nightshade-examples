use crate::ecs::{
    Difficulty, Faction, GameEvents, GameWorld, MOVEMENT, TileType, UnitType, get_defense_bonus_at,
    get_tile_type_at, unit_stats,
};
use crate::hex::{HexCoord, hex_distance};
use crate::systems::action::{GameAction, execute_action};
use crate::systems::{calculate_valid_moves, combat_win_chance};
use nightshade::prelude::*;

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

    let enemy_capitals: Vec<HexCoord> = Faction::ALL
        .iter()
        .filter(|&&faction| {
            faction != current_faction && !game_world.resources.faction_eliminated[faction.index()]
        })
        .map(|&faction| faction.capital_coord(&game_world.resources.map_params))
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

struct FactionPersonality {
    attack_bias: f32,
    defense_bias: f32,
    capture_bias: f32,
    advance_bias: f32,
    attack_threshold_offset: f32,
}

fn faction_personality(faction: Faction) -> FactionPersonality {
    match faction {
        Faction::Redosia => FactionPersonality {
            attack_bias: 1.0,
            defense_bias: 1.0,
            capture_bias: 1.0,
            advance_bias: 1.0,
            attack_threshold_offset: 0.0,
        },
        Faction::Violetnam => FactionPersonality {
            attack_bias: 1.5,
            defense_bias: 0.3,
            capture_bias: 1.2,
            advance_bias: 1.3,
            attack_threshold_offset: -0.15,
        },
        Faction::Bluegaria => FactionPersonality {
            attack_bias: 1.0,
            defense_bias: 1.0,
            capture_bias: 1.0,
            advance_bias: 1.0,
            attack_threshold_offset: 0.0,
        },
        Faction::Greenland => FactionPersonality {
            attack_bias: 0.8,
            defense_bias: 1.5,
            capture_bias: 1.3,
            advance_bias: 0.7,
            attack_threshold_offset: 0.05,
        },
    }
}

struct AiUnitState {
    entity: freecs::Entity,
    hex: HexCoord,
    soldiers: i32,
    morale: i32,
    unit_type: UnitType,
}

fn score_attack_actions(
    game_world: &GameWorld,
    unit: &AiUnitState,
    context: &AiTurnContext,
    difficulty: Difficulty,
    player_faction: Faction,
    personality: &FactionPersonality,
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
        let attacker_stats = unit_stats(unit.unit_type);
        let defender_stats = game_world
            .get_unit(*enemy_entity)
            .map(|u| unit_stats(u.unit_type));
        let defender_defense = defender_stats
            .as_ref()
            .map(|s| s.defense_multiplier)
            .unwrap_or(1.0);
        let win_chance = combat_win_chance(
            unit.soldiers,
            unit.morale,
            attacker_stats.attack_multiplier,
            *enemy_soldiers,
            *enemy_morale,
            defender_defense,
            defense_bonus,
        );

        let attack_threshold = match difficulty {
            Difficulty::Easy => 0.8,
            Difficulty::Normal => 0.7,
            Difficulty::Hard => 0.6,
        };

        let tile_type = get_tile_type_at(game_world, *enemy_hex);
        let is_capital = tile_type == Some(TileType::Capital);
        let threshold = (if is_capital {
            attack_threshold - 0.2
        } else {
            attack_threshold
        }) + personality.attack_threshold_offset;

        if win_chance > threshold {
            let mut score = 100.0 * win_chance;
            if is_capital {
                score += 50.0;
            }
            if prefer_human && *enemy_faction == player_faction {
                score += 20.0;
            }
            score *= personality.attack_bias;

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

fn apply_bias(actions: &mut [ScoredAction], bias: f32) {
    for action in actions.iter_mut() {
        action.score *= bias;
    }
}

struct EvalContext<'a> {
    game_world: &'a GameWorld,
    context: &'a AiTurnContext,
    difficulty: Difficulty,
    player_faction: Faction,
    valid_moves: &'a [HexCoord],
    my_capital: HexCoord,
    personality: &'a FactionPersonality,
}

fn evaluate_actions(unit: &AiUnitState, eval: &EvalContext) -> Vec<ScoredAction> {
    let mut all_actions = Vec::new();

    all_actions.extend(score_attack_actions(
        eval.game_world,
        unit,
        eval.context,
        eval.difficulty,
        eval.player_faction,
        eval.personality,
    ));

    let mut defense = score_defense_actions(unit, eval.my_capital, eval.context, eval.valid_moves);
    apply_bias(&mut defense, eval.personality.defense_bias);
    all_actions.extend(defense);

    let mut capture = score_capture_actions(eval.game_world, unit, eval.context, eval.valid_moves);
    apply_bias(&mut capture, eval.personality.capture_bias);
    all_actions.extend(capture);

    let mut advance = score_advance_actions(eval.game_world, unit, eval.context, eval.valid_moves);
    apply_bias(&mut advance, eval.personality.advance_bias);
    all_actions.extend(advance);

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

    let my_capital = current_faction.capital_coord(&game_world.resources.map_params);
    let context = build_ai_context(game_world, current_faction);
    let personality = faction_personality(current_faction);
    let movement_range = unit_stats(unit.unit_type).movement_range;
    let valid_moves = calculate_valid_moves(game_world, unit_entity, unit_hex, movement_range);

    let ai_unit = AiUnitState {
        entity: unit_entity,
        hex: unit_hex,
        soldiers: unit.soldiers,
        morale: unit.morale,
        unit_type: unit.unit_type,
    };

    let eval = EvalContext {
        game_world,
        context: &context,
        difficulty,
        player_faction,
        valid_moves: &valid_moves,
        my_capital,
        personality: &personality,
    };

    let mut scored_actions = evaluate_actions(&ai_unit, &eval);

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
