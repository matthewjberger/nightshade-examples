use crate::ecs::{
    AI, Ai, BLOCKER, Blocker, COMBAT_STATS, CombatStats, ENEMY, Enemy, EnemyType, GameWorld, ITEM,
    Item, ItemType, PLAYER, POSITION, Player, Position, RENDERABLE, Renderable,
};
use nightshade::prelude::freecs;

pub fn spawn_player(game_world: &mut GameWorld, x: i32, y: i32) -> freecs::Entity {
    let entity =
        game_world.spawn_entities(POSITION | RENDERABLE | PLAYER | COMBAT_STATS | BLOCKER, 1)[0];

    game_world.set_position(entity, Position { x, y });

    game_world.set_renderable(entity, Renderable { glyph: '@' });

    game_world.set_player(entity, Player);

    game_world.set_combat_stats(
        entity,
        CombatStats {
            hp: 30,
            max_hp: 30,
            attack: 5,
            defense: 2,
        },
    );

    game_world.set_blocker(entity, Blocker);

    game_world.resources.player_entity = Some(freecs::Entity {
        id: entity.id,
        generation: entity.generation,
    });

    entity
}

pub fn spawn_enemy(
    game_world: &mut GameWorld,
    x: i32,
    y: i32,
    enemy_type: EnemyType,
) -> freecs::Entity {
    let entity = game_world.spawn_entities(
        POSITION | RENDERABLE | ENEMY | COMBAT_STATS | AI | BLOCKER,
        1,
    )[0];

    let (hp, max_hp, attack, defense) = enemy_type.base_stats();

    game_world.set_position(entity, Position { x, y });

    game_world.set_renderable(
        entity,
        Renderable {
            glyph: enemy_type.glyph(),
        },
    );

    game_world.set_enemy(entity, Enemy { enemy_type });

    game_world.set_combat_stats(
        entity,
        CombatStats {
            hp,
            max_hp,
            attack,
            defense,
        },
    );

    game_world.set_ai(entity, Ai);

    game_world.set_blocker(entity, Blocker);

    entity
}

pub fn spawn_item(
    game_world: &mut GameWorld,
    x: i32,
    y: i32,
    item_type: ItemType,
) -> freecs::Entity {
    let entity = game_world.spawn_entities(POSITION | RENDERABLE | ITEM, 1)[0];

    game_world.set_position(entity, Position { x, y });

    game_world.set_renderable(
        entity,
        Renderable {
            glyph: item_type.glyph(),
        },
    );

    game_world.set_item(entity, Item { item_type });

    entity
}

pub fn despawn_entity(game_world: &mut GameWorld, entity: crate::ecs::Entity) {
    game_world.despawn_entities(&[entity]);
}
