use crate::ecs::{COMBAT_STATS, ENEMY, Entity, GameWorld, POSITION};

pub struct CombatResult {
    pub attacker_name: String,
    pub defender_name: String,
    pub damage: i32,
    pub defender_died: bool,
}

pub fn melee_attack(
    game_world: &mut GameWorld,
    attacker: Entity,
    defender: Entity,
    attack_bonus: i32,
    defense_bonus: i32,
) -> Option<CombatResult> {
    let attacker_stats = game_world.get_combat_stats(attacker)?;
    let defender_stats = game_world.get_combat_stats(defender)?;

    let attacker_name = if game_world.get_player(attacker).is_some() {
        "You".to_string()
    } else if let Some(enemy) = game_world.get_enemy(attacker) {
        enemy.enemy_type.name().to_string()
    } else {
        "Something".to_string()
    };

    let defender_name = if game_world.get_player(defender).is_some() {
        "you".to_string()
    } else if let Some(enemy) = game_world.get_enemy(defender) {
        format!("the {}", enemy.enemy_type.name().to_lowercase())
    } else {
        "something".to_string()
    };

    let attack_power = attacker_stats.attack + attack_bonus;
    let defense_power = defender_stats.defense + defense_bonus;
    let damage = (attack_power - defense_power).max(1);

    let defender_hp = defender_stats.hp - damage;
    let defender_died = defender_hp <= 0;

    if let Some(stats) = game_world.get_combat_stats_mut(defender) {
        stats.hp = defender_hp;
    }

    Some(CombatResult {
        attacker_name,
        defender_name,
        damage,
        defender_died,
    })
}

pub fn get_enemy_at(game_world: &GameWorld, x: i32, y: i32) -> Option<Entity> {
    for entity in game_world.query_entities(POSITION | ENEMY) {
        if let Some(position) = game_world.get_position(entity)
            && position.x == x
            && position.y == y
        {
            return Some(entity);
        }
    }
    None
}

pub fn get_blocking_entity_at(game_world: &GameWorld, x: i32, y: i32) -> Option<Entity> {
    for entity in game_world.query_entities(POSITION | COMBAT_STATS) {
        if let Some(position) = game_world.get_position(entity)
            && position.x == x
            && position.y == y
        {
            return Some(entity);
        }
    }
    None
}

pub fn is_position_blocked(game_world: &GameWorld, x: i32, y: i32) -> bool {
    if !game_world.resources.map.is_walkable(x, y) {
        return true;
    }

    get_blocking_entity_at(game_world, x, y).is_some()
}
