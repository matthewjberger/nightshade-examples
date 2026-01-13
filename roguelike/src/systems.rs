use crate::combat::{get_enemy_at, is_position_blocked, melee_attack};
use crate::ecs::{AI, ENEMY, Entity, GameState, GameWorld, ITEM, ItemType, POSITION, TileType};
use crate::entities::despawn_entity;
use crate::fov::compute_fov;

pub enum PlayerAction {
    Move { dx: i32, dy: i32 },
    Wait,
    PickupItem,
    UsePotion,
    Descend,
}

pub fn try_player_action(game_world: &mut GameWorld, action: PlayerAction) -> bool {
    if game_world.resources.game_state != GameState::Playing {
        return false;
    }

    match action {
        PlayerAction::Move { dx, dy } => try_move_player(game_world, dx, dy),
        PlayerAction::Wait => {
            game_world
                .resources
                .message_log
                .push("You wait...".to_string());
            true
        }
        PlayerAction::PickupItem => try_pickup_item(game_world),
        PlayerAction::UsePotion => try_use_potion(game_world),
        PlayerAction::Descend => try_descend(game_world),
    }
}

fn try_move_player(game_world: &mut GameWorld, dx: i32, dy: i32) -> bool {
    let Some(player_entity) = game_world.resources.player_entity else {
        return false;
    };

    let player_entity = Entity {
        id: player_entity.id,
        generation: player_entity.generation,
    };

    let Some(position) = game_world.get_position(player_entity) else {
        return false;
    };

    let new_x = position.x + dx;
    let new_y = position.y + dy;

    if let Some(enemy_entity) = get_enemy_at(game_world, new_x, new_y) {
        let attack_bonus = game_world.resources.inventory.attack_bonus();
        if let Some(result) = melee_attack(game_world, player_entity, enemy_entity, attack_bonus, 0)
        {
            let message = if result.defender_died {
                format!(
                    "{} hit {} for {} damage. {} dies!",
                    result.attacker_name, result.defender_name, result.damage, result.defender_name
                )
            } else {
                format!(
                    "{} hit {} for {} damage.",
                    result.attacker_name, result.defender_name, result.damage
                )
            };
            game_world.resources.message_log.push(message);

            if result.defender_died {
                despawn_entity(game_world, enemy_entity);
            }
        }
        return true;
    }

    if !game_world.resources.map.is_walkable(new_x, new_y) {
        return false;
    }

    if is_position_blocked(game_world, new_x, new_y) {
        return false;
    }

    if let Some(pos) = game_world.get_position_mut(player_entity) {
        pos.x = new_x;
        pos.y = new_y;
    }

    compute_fov(
        &mut game_world.resources.fov_map,
        &game_world.resources.map,
        new_x,
        new_y,
    );

    true
}

fn try_pickup_item(game_world: &mut GameWorld) -> bool {
    let Some(player_entity) = game_world.resources.player_entity else {
        return false;
    };

    let player_entity = Entity {
        id: player_entity.id,
        generation: player_entity.generation,
    };

    let Some(player_pos) = game_world.get_position(player_entity) else {
        return false;
    };

    let player_x = player_pos.x;
    let player_y = player_pos.y;

    let item_entity = game_world.query_entities(POSITION | ITEM).find(|entity| {
        if let Some(pos) = game_world.get_position(*entity) {
            pos.x == player_x && pos.y == player_y
        } else {
            false
        }
    });

    let Some(item_entity) = item_entity else {
        game_world
            .resources
            .message_log
            .push("Nothing to pick up here.".to_string());
        return false;
    };

    let Some(item) = game_world.get_item(item_entity) else {
        return false;
    };

    let item_type = item.item_type;
    let item_name = item_type.name().to_string();

    match item_type {
        ItemType::HealthPotion => {
            game_world.resources.inventory.items.push(item_type);
        }
        ItemType::Sword => {
            if let Some(old_weapon) = game_world.resources.inventory.equipped_weapon.take() {
                game_world.resources.inventory.items.push(old_weapon);
            }
            game_world.resources.inventory.equipped_weapon = Some(item_type);
        }
        ItemType::Shield => {
            if let Some(old_armor) = game_world.resources.inventory.equipped_armor.take() {
                game_world.resources.inventory.items.push(old_armor);
            }
            game_world.resources.inventory.equipped_armor = Some(item_type);
        }
    }

    game_world
        .resources
        .message_log
        .push(format!("You pick up the {}.", item_name));

    despawn_entity(game_world, item_entity);

    true
}

fn try_use_potion(game_world: &mut GameWorld) -> bool {
    let Some(player_entity) = game_world.resources.player_entity else {
        return false;
    };

    let player_entity = Entity {
        id: player_entity.id,
        generation: player_entity.generation,
    };

    let potion_index = game_world
        .resources
        .inventory
        .items
        .iter()
        .position(|item| matches!(item, ItemType::HealthPotion));

    let Some(index) = potion_index else {
        game_world
            .resources
            .message_log
            .push("You don't have any health potions.".to_string());
        return false;
    };

    game_world.resources.inventory.items.remove(index);

    if let Some(stats) = game_world.get_combat_stats_mut(player_entity) {
        let heal_amount = 10;
        let old_hp = stats.hp;
        stats.hp = (stats.hp + heal_amount).min(stats.max_hp);
        let actual_heal = stats.hp - old_hp;

        game_world.resources.message_log.push(format!(
            "You drink the health potion and recover {} HP.",
            actual_heal
        ));
    }

    true
}

fn try_descend(game_world: &mut GameWorld) -> bool {
    let Some(player_entity) = game_world.resources.player_entity else {
        return false;
    };

    let player_entity = Entity {
        id: player_entity.id,
        generation: player_entity.generation,
    };

    let Some(position) = game_world.get_position(player_entity) else {
        return false;
    };

    let tile = game_world.resources.map.get_tile(position.x, position.y);

    if tile != TileType::StairsDown {
        game_world
            .resources
            .message_log
            .push("There are no stairs here.".to_string());
        return false;
    }

    game_world.resources.current_depth += 1;
    game_world.resources.message_log.push(format!(
        "You descend to level {}...",
        game_world.resources.current_depth
    ));

    true
}

pub fn run_enemy_turns(game_world: &mut GameWorld) {
    let Some(player_entity) = game_world.resources.player_entity else {
        return;
    };

    let player_entity = Entity {
        id: player_entity.id,
        generation: player_entity.generation,
    };

    let Some(player_pos) = game_world.get_position(player_entity) else {
        return;
    };

    let player_x = player_pos.x;
    let player_y = player_pos.y;

    let enemies: Vec<Entity> = game_world.query_entities(ENEMY | POSITION | AI).collect();

    for enemy_entity in enemies {
        let Some(enemy_pos) = game_world.get_position(enemy_entity) else {
            continue;
        };

        let enemy_x = enemy_pos.x;
        let enemy_y = enemy_pos.y;

        if !game_world.resources.fov_map.is_visible(enemy_x, enemy_y) {
            continue;
        }

        let dx = (player_x - enemy_x).signum();
        let dy = (player_y - enemy_y).signum();

        let adjacent_to_player = (enemy_x - player_x).abs() <= 1 && (enemy_y - player_y).abs() <= 1;

        if adjacent_to_player && (dx != 0 || dy != 0) {
            let defense_bonus = game_world.resources.inventory.defense_bonus();
            if let Some(result) =
                melee_attack(game_world, enemy_entity, player_entity, 0, defense_bonus)
            {
                let attacker_name = if let Some(enemy) = game_world.get_enemy(enemy_entity) {
                    format!("The {}", enemy.enemy_type.name())
                } else {
                    "Something".to_string()
                };

                let message = format!("{} hits you for {} damage!", attacker_name, result.damage);
                game_world.resources.message_log.push(message);

                if result.defender_died {
                    game_world.resources.game_state = GameState::PlayerDead;
                    game_world
                        .resources
                        .message_log
                        .push("You have died!".to_string());
                }
            }
        } else {
            let move_x = enemy_x + dx;
            let move_y = enemy_y + dy;

            if !is_position_blocked(game_world, move_x, move_y)
                && game_world.resources.map.is_walkable(move_x, move_y)
            {
                if let Some(pos) = game_world.get_position_mut(enemy_entity) {
                    pos.x = move_x;
                    pos.y = move_y;
                }
            } else if dx != 0 {
                let alt_x = enemy_x + dx;
                if !is_position_blocked(game_world, alt_x, enemy_y)
                    && game_world.resources.map.is_walkable(alt_x, enemy_y)
                    && let Some(pos) = game_world.get_position_mut(enemy_entity)
                {
                    pos.x = alt_x;
                }
            } else if dy != 0 {
                let alt_y = enemy_y + dy;
                if !is_position_blocked(game_world, enemy_x, alt_y)
                    && game_world.resources.map.is_walkable(enemy_x, alt_y)
                    && let Some(pos) = game_world.get_position_mut(enemy_entity)
                {
                    pos.y = alt_y;
                }
            }
        }
    }
}

pub fn check_victory(game_world: &mut GameWorld) {
    if game_world.resources.game_state != GameState::Playing {
        return;
    }

    let enemy_count = game_world.query_entities(ENEMY).count();

    if enemy_count == 0 && game_world.resources.current_depth >= 5 {
        game_world.resources.game_state = GameState::Victory;
        game_world
            .resources
            .message_log
            .push("You have conquered the dungeon! Victory!".to_string());
    }
}
