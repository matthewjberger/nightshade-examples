use crate::ecs::{Entity, GameWorld, HEX_POSITION, HexPosition, TILE, Tile, TileType};
use crate::hex::HexCoord;

pub fn spawn_tile(game_world: &mut GameWorld, hex_coord: HexCoord, tile_type: TileType) -> Entity {
    let entity = game_world.spawn_entities(HEX_POSITION | TILE, 1)[0];
    game_world.set_hex_position(entity, HexPosition(hex_coord));
    game_world.set_tile(
        entity,
        Tile {
            tile_type,
            faction: None,
        },
    );

    game_world.resources.tile_map.insert(hex_coord, entity);
    if tile_type != TileType::Sea {
        game_world.resources.passable_tiles.insert(hex_coord);
    }
    if tile_type == TileType::Port {
        game_world.resources.port_tiles.insert(hex_coord);
    }

    entity
}

pub fn despawn_all_tiles(game_world: &mut GameWorld) {
    let tile_entities: Vec<_> = game_world.query_entities(TILE).collect();
    game_world.despawn_entities(&tile_entities);
    game_world.resources.tile_map.clear();
    game_world.resources.passable_tiles.clear();
    game_world.resources.port_tiles.clear();
}
