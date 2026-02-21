use nightshade::prelude::*;

use crate::data::{
    get_crop_definition, get_crop_from_seed, get_crop_material_name, get_crop_scale,
};
use crate::ecs::{
    CROP, Crop, HANDLE, Handle, POSITION, Position, TILE, Tile, World as GameWorld, tile_center,
    tile_coords,
};
use crate::events::CropHarvestedEvent;
use crate::systems::player::{get_equipped_tool, get_player_facing, get_player_position};
use crate::types::{TILE_SIZE, TOOL_STAMINA_COST, ToolType, Weather};

pub fn try_use_tool(game: &mut GameWorld, world: &mut World) -> (bool, Option<CropHarvestedEvent>) {
    let tool = get_equipped_tool(game);

    match tool {
        ToolType::Hoe => (try_till(game, world), None),
        ToolType::WateringCan => (try_water(game, world), None),
        ToolType::Scythe => {
            let event = try_harvest(game, world);
            (event.is_some(), event)
        }
        ToolType::Hand => (try_plant(game, world), None),
        _ => (false, None),
    }
}

fn get_target_tile(game: &GameWorld) -> (i32, i32) {
    let player_pos = get_player_position(game);
    let facing = get_player_facing(game);

    let facing_normalized = if nalgebra_glm::length(&facing) > 0.01 {
        nalgebra_glm::normalize(&facing)
    } else {
        Vec3::new(0.0, 0.0, 1.0)
    };

    let target_pos = player_pos + facing_normalized * 1.5;
    tile_coords(target_pos.x, target_pos.z)
}

fn try_till(game: &mut GameWorld, world: &mut World) -> bool {
    let (tile_x, tile_z) = get_target_tile(game);

    if game.resources.farm.tiles.contains_key(&(tile_x, tile_z)) {
        return false;
    }

    let Some(player_entity) = game.resources.player_entity else {
        return false;
    };

    let has_stamina = game
        .get_player(player_entity)
        .map(|p| p.stamina >= TOOL_STAMINA_COST)
        .unwrap_or(false);

    if !has_stamina {
        return false;
    }

    game.modify_player(player_entity, |p| p.stamina -= TOOL_STAMINA_COST);

    let tile_pos = tile_center(tile_x, tile_z);

    let visual = spawn_mesh(
        world,
        "Cube",
        Vec3::new(tile_pos.x, 0.02, tile_pos.z),
        Vec3::new(TILE_SIZE * 0.9, 0.05, TILE_SIZE * 0.9),
    );
    crate::systems::init::apply_material_by_name(world, visual, "TilledSoil");

    let tile_entity = game.spawn_entities(HANDLE | POSITION | TILE, 1)[0];
    game.set_handle(tile_entity, Handle(visual));
    game.set_position(tile_entity, Position(tile_pos));
    game.set_tile(tile_entity, Tile { watered: false });
    game.resources
        .farm
        .tiles
        .insert((tile_x, tile_z), tile_entity);

    true
}

fn try_water(game: &mut GameWorld, world: &mut World) -> bool {
    let (tile_x, tile_z) = get_target_tile(game);

    let Some(&tile_entity) = game.resources.farm.tiles.get(&(tile_x, tile_z)) else {
        return false;
    };

    let is_watered = game
        .get_tile(tile_entity)
        .map(|t| t.watered)
        .unwrap_or(true);
    if is_watered {
        return false;
    }

    let Some(player_entity) = game.resources.player_entity else {
        return false;
    };

    let has_stamina = game
        .get_player(player_entity)
        .map(|p| p.stamina >= TOOL_STAMINA_COST * 0.5)
        .unwrap_or(false);

    if !has_stamina {
        return false;
    }

    game.modify_player(player_entity, |p| p.stamina -= TOOL_STAMINA_COST * 0.5);
    game.modify_tile(tile_entity, |t| t.watered = true);

    if let Some(&crop_entity) = game.resources.farm.crops.get(&(tile_x, tile_z)) {
        game.modify_crop(crop_entity, |c| c.watered_today = true);
    }

    if let Some(handle) = game.get_handle(tile_entity) {
        crate::systems::init::apply_material_by_name(world, handle.0, "WateredSoil");
    }

    true
}

fn try_harvest(game: &mut GameWorld, world: &mut World) -> Option<CropHarvestedEvent> {
    let (tile_x, tile_z) = get_target_tile(game);

    let &crop_entity = game.resources.farm.crops.get(&(tile_x, tile_z))?;
    let crop = game.get_crop(crop_entity)?;

    if crop.growth_stage < crop.max_growth_stage {
        return None;
    }

    let crop_type = crop.crop_type;
    let definition = get_crop_definition(crop_type)?;
    let player_entity = game.resources.player_entity?;

    let has_stamina = game
        .get_player(player_entity)
        .map(|p| p.stamina >= TOOL_STAMINA_COST * 0.5)
        .unwrap_or(false);

    if !has_stamina {
        return None;
    }

    game.modify_player(player_entity, |p| p.stamina -= TOOL_STAMINA_COST * 0.5);

    let harvest_item = definition.harvest_item;
    let crop_name = definition.name;
    let regrows = definition.regrows;

    game.resources.inventory.add_item(harvest_item, 1);

    let tile_pos = tile_center(tile_x, tile_z);
    let event = CropHarvestedEvent {
        position: tile_pos,
        item_name: crop_name,
    };

    if let Some(handle) = game.get_handle(crop_entity) {
        world.queue_despawn_entity(handle.0);
    }

    if regrows {
        game.modify_crop(crop_entity, |crop| {
            crop.growth_stage = crop.max_growth_stage - 1;
            crop.days_in_stage = 0;
        });

        let Some(crop) = game.get_crop(crop_entity) else {
            return Some(event);
        };
        let growth_stage = crop.growth_stage;
        let max_stage = crop.max_growth_stage;

        let scale = get_crop_scale(growth_stage, max_stage);
        let visual = spawn_mesh(
            world,
            "Cube",
            Vec3::new(tile_pos.x, scale * 0.5, tile_pos.z),
            Vec3::new(scale * 0.4, scale, scale * 0.4),
        );
        world.set_casts_shadow(visual, CastsShadow);

        let material_name = get_crop_material_name(crop_type, growth_stage, max_stage);
        crate::systems::init::apply_material_by_name(world, visual, material_name);

        game.set_handle(crop_entity, Handle(visual));
    } else {
        game.resources.farm.crops.remove(&(tile_x, tile_z));
        game.queue_despawn_entity(crop_entity);
        game.apply_commands();
    }

    Some(event)
}

fn try_plant(game: &mut GameWorld, world: &mut World) -> bool {
    let (tile_x, tile_z) = get_target_tile(game);

    let Some((item_id, quantity)) = game.resources.inventory.selected_item() else {
        return false;
    };

    if quantity == 0 {
        return false;
    }

    let Some(crop_type) = get_crop_from_seed(item_id) else {
        return false;
    };

    if !game.resources.farm.tiles.contains_key(&(tile_x, tile_z)) {
        return false;
    }

    if game.resources.farm.crops.contains_key(&(tile_x, tile_z)) {
        return false;
    }

    let Some(definition) = get_crop_definition(crop_type) else {
        return false;
    };

    if !definition.valid_seasons.contains(&game.resources.season) {
        return false;
    }

    let is_watered = game
        .resources
        .farm
        .tiles
        .get(&(tile_x, tile_z))
        .and_then(|&entity| game.get_tile(entity))
        .map(|tile| tile.watered)
        .unwrap_or(false);

    game.resources.inventory.consume_selected(1);

    let tile_pos = tile_center(tile_x, tile_z);

    let scale = get_crop_scale(1, definition.growth_stages);
    let visual = spawn_mesh(
        world,
        "Cube",
        Vec3::new(tile_pos.x, scale * 0.5, tile_pos.z),
        Vec3::new(scale * 0.4, scale, scale * 0.4),
    );
    world.set_casts_shadow(visual, CastsShadow);

    let material_name = get_crop_material_name(crop_type, 1, definition.growth_stages);
    crate::systems::init::apply_material_by_name(world, visual, material_name);

    let crop_entity = game.spawn_entities(HANDLE | POSITION | CROP, 1)[0];
    game.set_handle(crop_entity, Handle(visual));
    game.set_position(crop_entity, Position(tile_pos));
    game.set_crop(
        crop_entity,
        Crop {
            crop_type,
            growth_stage: 1,
            max_growth_stage: definition.growth_stages,
            days_in_stage: 0,
            watered_today: is_watered,
            watered_days: 0,
            total_days: 0,
        },
    );
    game.resources
        .farm
        .crops
        .insert((tile_x, tile_z), crop_entity);

    true
}

struct CropGrowthUpdate {
    crop_type: crate::types::CropType,
    new_stage: u8,
    max_stage: u8,
    handle: Option<Entity>,
}

pub fn process_day_change(game: &mut GameWorld, world: &mut World) {
    let is_rainy = matches!(game.resources.weather, Weather::Rainy | Weather::Stormy);

    let tile_coords_list: Vec<(i32, i32)> = game.resources.farm.tiles.keys().copied().collect();

    for coords in &tile_coords_list {
        let Some(&tile_entity) = game.resources.farm.tiles.get(coords) else {
            continue;
        };

        if is_rainy {
            game.modify_tile(tile_entity, |t| t.watered = true);
            if let Some(&crop_entity) = game.resources.farm.crops.get(coords) {
                game.modify_crop(crop_entity, |c| c.watered_today = true);
            }
        }
    }

    let crop_coords: Vec<(i32, i32)> = game.resources.farm.crops.keys().copied().collect();
    let mut growth_updates: Vec<CropGrowthUpdate> = Vec::new();

    for coords in &crop_coords {
        let Some(&crop_entity) = game.resources.farm.crops.get(coords) else {
            continue;
        };

        let Some(crop) = game.get_crop(crop_entity) else {
            continue;
        };

        let watered_today = crop.watered_today;
        let can_grow = watered_today && crop.growth_stage < crop.max_growth_stage;
        let crop_type = crop.crop_type;
        let max_stage = crop.max_growth_stage;
        let current_stage = crop.growth_stage;
        let days_in_stage = crop.days_in_stage;

        game.modify_crop(crop_entity, |c| {
            c.total_days += 1;
            if watered_today {
                c.watered_days += 1;
            }
        });

        if can_grow {
            let Some(definition) = get_crop_definition(crop_type) else {
                continue;
            };

            let days_per_stage = definition.days_to_grow / definition.growth_stages;
            let new_days_in_stage = days_in_stage + 1;

            if new_days_in_stage >= days_per_stage {
                let new_stage = current_stage + 1;
                game.modify_crop(crop_entity, |c| {
                    c.growth_stage = new_stage;
                    c.days_in_stage = 0;
                });

                let handle = game.get_handle(crop_entity).map(|h| h.0);
                growth_updates.push(CropGrowthUpdate {
                    crop_type,
                    new_stage,
                    max_stage,
                    handle,
                });
            } else {
                game.modify_crop(crop_entity, |c| {
                    c.days_in_stage = new_days_in_stage;
                });
            }
        }

        game.modify_crop(crop_entity, |c| c.watered_today = false);

        if let Some(&tile_entity) = game.resources.farm.tiles.get(coords) {
            game.modify_tile(tile_entity, |t| t.watered = false);
            if let Some(handle) = game.get_handle(tile_entity) {
                crate::systems::init::apply_material_by_name(world, handle.0, "TilledSoil");
            }
        }
    }

    for update in growth_updates {
        if let Some(handle) = update.handle {
            let scale = get_crop_scale(update.new_stage, update.max_stage);

            if let Some(transform) = world.get_local_transform_mut(handle) {
                transform.translation.y = scale * 0.5;
                transform.scale = Vec3::new(scale * 0.4, scale, scale * 0.4);
            }
            mark_local_transform_dirty(world, handle);

            let material_name =
                get_crop_material_name(update.crop_type, update.new_stage, update.max_stage);
            crate::systems::init::apply_material_by_name(world, handle, material_name);
        }
    }

    for coords in tile_coords_list {
        if game.resources.farm.crops.contains_key(&coords) {
            continue;
        }

        let Some(&tile_entity) = game.resources.farm.tiles.get(&coords) else {
            continue;
        };

        game.modify_tile(tile_entity, |t| t.watered = false);
        if let Some(handle) = game.get_handle(tile_entity) {
            crate::systems::init::apply_material_by_name(world, handle.0, "TilledSoil");
        }
    }
}
