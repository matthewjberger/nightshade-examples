use crate::ecs::{
    Faction, FactionEliminatedEvent, GameEvents, GameWorld, TileType, UNIT, faction_index,
};
use crate::hex::HexCoord;
use crate::map::CAPITAL_POSITIONS;
use nightshade::prelude::*;

pub enum GameResult {
    Ongoing,
    Victory(Faction),
}

fn faction_from_index(index: usize) -> Faction {
    match index {
        0 => Faction::Redosia,
        1 => Faction::Violetnam,
        2 => Faction::Bluegaria,
        _ => Faction::Greenland,
    }
}

fn get_capital_coord(faction: Faction) -> HexCoord {
    let index = faction_index(faction);
    let (col, row, _) = CAPITAL_POSITIONS[index];
    HexCoord { column: col, row }
}

pub fn victory_system(
    game_world: &mut GameWorld,
    world: &mut World,
    events: &mut GameEvents,
) -> GameResult {
    for faction_idx in 0..4 {
        if game_world.resources.faction_eliminated[faction_idx] {
            continue;
        }

        let faction = faction_from_index(faction_idx);
        let capital_coord = get_capital_coord(faction);

        let capital_owner = game_world
            .resources
            .tile_map
            .get(&capital_coord)
            .and_then(|&entity| game_world.get_tile(entity))
            .and_then(|tile| {
                if tile.tile_type == TileType::Capital {
                    tile.faction
                } else {
                    None
                }
            });

        if let Some(owner) = capital_owner
            && owner != faction
        {
            game_world.resources.faction_eliminated[faction_idx] = true;
            events
                .faction_eliminated_events
                .push(FactionEliminatedEvent { faction });

            let units_to_remove: Vec<_> = game_world
                .query_entities(UNIT)
                .filter(|&entity| {
                    game_world
                        .get_unit(entity)
                        .map(|u| u.faction == faction)
                        .unwrap_or(false)
                })
                .collect();

            for entity in units_to_remove {
                crate::systems::despawn_unit(game_world, world, entity);
            }

            let tile_entities: Vec<_> = game_world.resources.tile_map.values().copied().collect();
            for entity in tile_entities {
                if let Some(tile) = game_world.get_tile(entity)
                    && tile.faction == Some(faction)
                    && let Some(tile_mut) = game_world.get_tile_mut(entity)
                {
                    tile_mut.faction = None;
                }
            }
        }
    }

    let alive_count = game_world
        .resources
        .faction_eliminated
        .iter()
        .filter(|&&eliminated| !eliminated)
        .count();

    if alive_count == 1 {
        for (index, &eliminated) in game_world.resources.faction_eliminated.iter().enumerate() {
            if !eliminated {
                return GameResult::Victory(faction_from_index(index));
            }
        }
    }

    GameResult::Ongoing
}
