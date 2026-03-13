use nightshade::prelude::*;

use crate::castle::CASTLE_SIZE;
use crate::ecs::{GameWorld, LocationId};
use crate::rendering;
use crate::systems::bombardment_systems::spawn_fire_at;

pub fn fire_spread_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;

    if game.resources.failure_triggered || game.resources.paused {
        return;
    }

    let fire_count = game.resources.fires.len();
    let mut fires_to_damage = Vec::new();
    let mut spread_positions = Vec::new();

    for fire_index in 0..fire_count {
        let fire_entity = game.resources.fires[fire_index];
        if let Some(fire) = game.get_fire(fire_entity) {
            let mut fire = fire.clone();
            fire.spread_timer += delta_time;

            if let Some(near_loc) = fire.near_location {
                fires_to_damage.push((near_loc, delta_time * 10.0));
            }

            if fire.spread_timer >= 3.0 {
                fire.spread_timer = 0.0;
                let spread_roll = game.resources.bombardment.next_random();
                if spread_roll < 0.2 {
                    let offset_x = game.resources.bombardment.next_random_range(-4.0, 4.0);
                    let offset_z = game.resources.bombardment.next_random_range(-4.0, 4.0);
                    let new_pos = fire.position + nalgebra_glm::vec3(offset_x, 0.0, offset_z);
                    if new_pos.x.abs() < CASTLE_SIZE && new_pos.z.abs() < CASTLE_SIZE {
                        let too_close = game.resources.fires.iter().any(|&other_entity| {
                            game.get_fire(other_entity).is_some_and(|other| {
                                nalgebra_glm::distance(&other.position, &new_pos) < 2.0
                            })
                        });
                        if !too_close {
                            spread_positions.push(new_pos);
                        }
                    }
                }
            }

            game.set_fire(fire_entity, fire);
        }
    }

    for (location, damage) in fires_to_damage {
        apply_location_damage(game, world, location, damage);
    }

    for position in spread_positions {
        spawn_fire_at(game, world, position);
    }

    if game.resources.manual_burn_armory {
        game.resources.manual_burn_armory = false;
        game.resources.castle.armory_exists = false;
        game.resources.castle.armory_stock = 0;
        rendering::darken_structure(world, "armory");
        rendering::darken_structure(world, "armory_roof");
        rendering::darken_structure(world, "arrow_bundle");
    }

    if game.resources.manual_drain_well {
        game.resources.manual_drain_well = false;
        game.resources.castle.well_water_remaining = 0.0;
        game.resources.castle.well_destroyed = true;
        rendering::darken_structure(world, "well_stone");
        rendering::darken_structure(world, "well_water");
    }
}

fn apply_location_damage(
    game: &mut GameWorld,
    world: &mut World,
    location: LocationId,
    damage: f32,
) {
    match location {
        LocationId::Armory => {
            if game.resources.castle.armory_exists {
                game.resources.castle.armory_stock =
                    game.resources.castle.armory_stock.saturating_sub(1);
                if game.resources.castle.armory_stock == 0 {
                    game.resources.castle.armory_exists = false;
                    rendering::darken_structure(world, "armory");
                    rendering::darken_structure(world, "armory_roof");
                    rendering::darken_structure(world, "arrow_bundle");
                }
            }
        }
        LocationId::HealingStation => {
            if game.resources.castle.healing_station_exists {
                game.resources.castle.healing_station_exists = false;
                rendering::darken_structure(world, "healing_base");
                rendering::darken_structure(world, "healing_cross");
            }
        }
        LocationId::RepairPile => {
            game.resources.castle.repair_pile_count =
                game.resources.castle.repair_pile_count.saturating_sub(1);
        }
        LocationId::Well => {
            game.resources.castle.well_water_remaining =
                (game.resources.castle.well_water_remaining - damage).max(0.0);
            if game.resources.castle.well_water_remaining <= 0.0 {
                game.resources.castle.well_destroyed = true;
                rendering::darken_structure(world, "well_stone");
                rendering::darken_structure(world, "well_water");
            }
        }
        _ => {}
    }
}

pub fn archer_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;

    if game.resources.failure_triggered || game.resources.paused {
        return;
    }

    for post in &mut game.resources.castle.archer_posts {
        if post.arrows_remaining == 0 {
            if let Some(line_entity) = post.line_entity.take() {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: line_entity,
                    });
            }
            continue;
        }

        post.fire_timer += delta_time;
        if post.fire_timer >= 4.0 {
            post.fire_timer = 0.0;
            post.arrows_remaining = post.arrows_remaining.saturating_sub(1);

            if let Some(line_entity) = post.line_entity {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: line_entity,
                    });
            }

            let direction = nalgebra_glm::vec3(
                if post.position.x < 0.0 { -1.0 } else { 1.0 },
                0.3,
                if post.position.z < 0.0 { -1.0 } else { 1.0 },
            );
            let end_pos = post.position + nalgebra_glm::normalize(&direction) * 20.0;

            let line_entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | LINES
                    | VISIBILITY,
                1,
            )[0];
            world.core.set_name(line_entity, Name("ArcherLine".to_string()));
            world.core.set_local_transform(
                line_entity,
                LocalTransform {
                    translation: Vec3::zeros(),
                    ..Default::default()
                },
            );
            world.core.set_global_transform(line_entity, GlobalTransform::default());
            world.core.set_local_transform_dirty(line_entity, LocalTransformDirty);
            world.core.set_lines(
                line_entity,
                Lines::new(vec![Line {
                    start: post.position,
                    end: end_pos,
                    color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 0.8),
                }]),
            );
            world.core.set_visibility(line_entity, Visibility { visible: true });

            post.line_entity = Some(line_entity);
        }

        if post.fire_timer > 0.3
            && let Some(line_entity) = post.line_entity.take()
        {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive {
                    entity: line_entity,
                });
        }
    }
}

pub fn resource_depletion_system(game: &mut GameWorld, _world: &mut World) {
    if game.resources.failure_triggered || game.resources.paused {
        return;
    }

    let delta_time = _world.resources.window.timing.delta_time * game.resources.game_speed;

    game.resources.castle.well_water_remaining =
        (game.resources.castle.well_water_remaining - delta_time * 0.3).max(0.0);
    if game.resources.castle.well_water_remaining <= 0.0 {
        game.resources.castle.well_destroyed = true;
    }

    if game.resources.elapsed_time > 60.0 && game.resources.castle.repair_pile_count > 0 {
        let depletion_roll = game.resources.bombardment.next_random();
        if depletion_roll < delta_time * 0.05 {
            game.resources.castle.repair_pile_count =
                game.resources.castle.repair_pile_count.saturating_sub(1);
        }
    }
}

pub fn fire_proximity_damage_system(game: &mut GameWorld, world: &mut World) {
    if game.resources.failure_triggered || game.resources.paused {
        return;
    }

    let fire_positions: Vec<Vec3> = game
        .resources
        .fires
        .iter()
        .filter_map(|&fire_entity| game.get_fire(fire_entity))
        .map(|fire| fire.position)
        .collect();

    if fire_positions.is_empty() {
        return;
    }

    let agent_count = game.resources.agents.len();
    for agent_index in 0..agent_count {
        let agent_entity = game.resources.agents[agent_index];
        if let Some(agent) = game.get_agent(agent_entity) {
            if agent.wounded {
                continue;
            }
            let near_fire = fire_positions
                .iter()
                .any(|pos| nalgebra_glm::distance(&agent.position, pos) < 2.0);
            if near_fire {
                let mut agent = agent.clone();
                agent.wounded = true;
                agent.health = (agent.health - 25.0).max(10.0);
                rendering::set_agent_wounded_color(world, &agent.body, agent_index);
                game.set_agent(agent_entity, agent);
            }
        }
    }
}
