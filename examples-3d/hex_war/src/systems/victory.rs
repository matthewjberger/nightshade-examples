use crate::ecs::{
    ActionRecord, Faction, FactionEliminatedEvent, GameEvents, GameWorld, TileType, UNIT,
};
use nightshade::prelude::*;

pub enum GameResult {
    Ongoing,
    Victory(Faction),
}

pub fn victory_system(
    game_world: &mut GameWorld,
    world: &mut World,
    events: &mut GameEvents,
) -> GameResult {
    for (faction_idx, &faction) in Faction::ALL.iter().enumerate() {
        if game_world.resources.faction_eliminated[faction_idx] {
            continue;
        }

        let capital_coord = faction.capital_coord(&game_world.resources.map_params);

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
            events.action_history.push(ActionRecord {
                faction,
                turn: game_world.resources.turn_number,
                description: format!("{} has been eliminated!", faction.name()),
            });

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
                return GameResult::Victory(Faction::ALL[index]);
            }
        }
    }

    GameResult::Ongoing
}
