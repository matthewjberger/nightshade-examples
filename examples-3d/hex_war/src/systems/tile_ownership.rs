use crate::ecs::{Faction, GameWorld, TileType, modify_faction_morale};
use crate::hex::HexCoord;

pub struct TileCapture {
    pub coord: HexCoord,
    pub tile_type: TileType,
    pub faction: Faction,
}

fn morale_change_for_capture(tile_type: TileType, was_enemy: bool) -> i32 {
    match (tile_type, was_enemy) {
        (TileType::Capital, true) => 10,
        (TileType::City, true) => 3,
        (TileType::City, false) => 2,
        (TileType::Port, true) => 2,
        (TileType::Port, false) => 1,
        (_, true) => 1,
        (_, false) => 0,
    }
}

fn morale_change_for_loss(tile_type: TileType) -> i32 {
    match tile_type {
        TileType::City | TileType::Capital => -3,
        TileType::Port => -2,
        _ => -1,
    }
}

pub fn tile_ownership_system(game_world: &mut GameWorld) -> Vec<TileCapture> {
    let unit_positions: Vec<(HexCoord, Faction)> = game_world
        .resources
        .unit_position_map
        .iter()
        .filter_map(|(&coord, &entity)| {
            let unit = game_world.get_unit(entity)?;
            Some((coord, unit.faction))
        })
        .collect();

    let mut morale_changes: Vec<(Faction, i32)> = Vec::new();
    let mut captures: Vec<TileCapture> = Vec::new();

    for (coord, unit_faction) in &unit_positions {
        let Some(&tile_entity) = game_world.resources.tile_map.get(coord) else {
            continue;
        };
        let Some(tile) = game_world.get_tile(tile_entity).copied() else {
            continue;
        };

        if tile.tile_type == TileType::Sea {
            continue;
        }

        let old_owner = tile.faction;
        if old_owner == Some(*unit_faction) {
            continue;
        }

        let was_enemy = old_owner.is_some();
        let gain = morale_change_for_capture(tile.tile_type, was_enemy);
        if gain > 0 {
            morale_changes.push((*unit_faction, gain));
        }

        if let Some(old_faction) = old_owner {
            let loss = morale_change_for_loss(tile.tile_type);
            morale_changes.push((old_faction, loss));
        }

        if matches!(
            tile.tile_type,
            TileType::City | TileType::Port | TileType::Capital
        ) {
            captures.push(TileCapture {
                coord: *coord,
                tile_type: tile.tile_type,
                faction: *unit_faction,
            });
        }

        if let Some(tile_mut) = game_world.get_tile_mut(tile_entity) {
            tile_mut.faction = Some(*unit_faction);
        }
    }

    for (faction, delta) in morale_changes {
        modify_faction_morale(&mut game_world.resources, faction, delta);
    }

    captures
}
