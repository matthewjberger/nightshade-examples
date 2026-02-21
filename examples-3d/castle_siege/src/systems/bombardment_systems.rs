use nightshade::prelude::*;

use crate::castle::{self, CASTLE_SIZE};
use crate::ecs::{
    BOULDER, Boulder, FIRE, Fire, GameWorld, LocationId, RUBBLE, Rubble, TimedEffect,
};
use crate::pathfinding;
use crate::rendering;

static TRAIL_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn boulder_spawn_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;

    if game.resources.failure_triggered || game.resources.paused {
        return;
    }

    game.resources
        .bombardment
        .update_act(game.resources.elapsed_time);
    game.resources.bombardment.boulder_timer -= delta_time;

    let active_archers = game
        .resources
        .castle
        .archer_posts
        .iter()
        .filter(|post| post.arrows_remaining > 0)
        .count();
    let archer_modifier = 1.0 + active_archers as f32 * 0.2;
    game.resources.bombardment.boulder_interval *= archer_modifier;

    let should_fire =
        game.resources.bombardment.boulder_timer <= 0.0 || game.resources.manual_boulder_requested;

    if should_fire {
        game.resources.manual_boulder_requested = false;
        game.resources.bombardment.boulder_timer = game.resources.bombardment.boulder_interval;

        let target = game.resources.bombardment.pick_target();
        let start = game.resources.bombardment.pick_spawn_position(&target);
        let arc_height = 15.0 + game.resources.bombardment.next_random_range(0.0, 5.0);
        let speed = 0.7 + game.resources.bombardment.next_random_range(0.0, 0.3);

        let render_entity = rendering::spawn_boulder(world, start);
        let game_entity = game.spawn_entities(BOULDER, 1)[0];
        game.set_boulder(
            game_entity,
            Boulder {
                start,
                target,
                arc_height,
                progress: 0.0,
                speed,
            },
        );
        game.set_entity_handle(game_entity, crate::ecs::EntityHandle(render_entity));
        game.resources.boulders.push(game_entity);
        game.resources.bombardment.total_boulders_fired += 1;
    }
}

pub fn boulder_physics_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;

    let boulder_entities: Vec<_> = game.resources.boulders.clone();
    let mut to_remove = Vec::new();

    for &boulder_entity in &boulder_entities {
        let boulder = match game.get_boulder(boulder_entity) {
            Some(boulder) => boulder.clone(),
            None => continue,
        };

        let mut boulder = boulder;
        boulder.progress += boulder.speed * delta_time;

        let progress = boulder.progress.clamp(0.0, 1.0);
        let position = nalgebra_glm::lerp(&boulder.start, &boulder.target, progress);
        let arc_offset = boulder.arc_height * (1.0 - (2.0 * progress - 1.0).powi(2));
        let final_position = position + nalgebra_glm::vec3(0.0, arc_offset, 0.0);

        if let Some(handle) = game.get_entity_handle(boulder_entity) {
            let render_entity = handle.0;
            if let Some(transform) = world.get_local_transform_mut(render_entity) {
                transform.translation = final_position;
            }
            world.set_local_transform_dirty(render_entity, LocalTransformDirty);
        }

        let trail_index = TRAIL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if trail_index.is_multiple_of(3) {
            let trail_entity = rendering::spawn_trail_particle(world, final_position);
            game.resources.trail_particles.push(TimedEffect {
                entity: trail_entity,
                timer: 0.0,
                max_time: 0.4,
            });
        }

        if boulder.progress >= 1.0 {
            to_remove.push((boulder_entity, boulder.target));
        } else {
            game.set_boulder(boulder_entity, boulder);
        }
    }

    for (boulder_entity, impact_pos) in to_remove {
        if let Some(handle) = game.get_entity_handle(boulder_entity) {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive { entity: handle.0 });
        }
        game.despawn_entities(&[boulder_entity]);
        game.resources
            .boulders
            .retain(|&entity| entity != boulder_entity);

        let flash_entity = rendering::spawn_impact_flash(world, impact_pos);
        game.resources.impact_flashes.push(TimedEffect {
            entity: flash_entity,
            timer: 0.0,
            max_time: 0.3,
        });

        apply_impact(game, world, impact_pos);
    }
}

fn apply_impact(game: &mut GameWorld, world: &mut World, impact_pos: Vec3) {
    let wall_hit_range = 2.5;
    for wall_index in 0..4 {
        let segment_count = game.resources.castle.walls[wall_index].segments.len();
        for segment_index in 0..segment_count {
            let segment = &game.resources.castle.walls[wall_index].segments[segment_index];
            if segment.entity == Entity::default() {
                continue;
            }
            let dist = nalgebra_glm::distance(&impact_pos, &segment.position);
            if dist < wall_hit_range && !segment.breached {
                let damage = 30.0 + game.resources.bombardment.next_random_range(0.0, 20.0);
                let segment = &mut game.resources.castle.walls[wall_index].segments[segment_index];
                segment.health = (segment.health - damage).max(0.0);
                let ratio = segment.health / segment.max_health;
                rendering::update_wall_segment_color(world, wall_index, segment_index, ratio);

                if segment.health <= 0.0 {
                    segment.breached = true;
                    let entity = segment.entity;
                    if let Some(transform) = world.get_local_transform_mut(entity) {
                        transform.scale.y = 0.3;
                        transform.translation.y = 0.15;
                    }
                    world.set_local_transform_dirty(entity, LocalTransformDirty);
                }
                return;
            }
        }
    }

    let gate_dist = nalgebra_glm::distance(&impact_pos, &castle::GATE_POS);
    if gate_dist < 3.0 {
        let damage = 20.0 + game.resources.bombardment.next_random_range(0.0, 15.0);
        game.resources.castle.gate_health = (game.resources.castle.gate_health - damage).max(0.0);
        rendering::update_gate_color(
            world,
            game.resources.castle.gate_health / game.resources.castle.gate_max_health,
        );
    }

    let back_gate_pos = nalgebra_glm::vec3(0.0, 0.0, -CASTLE_SIZE);
    let back_gate_dist = nalgebra_glm::distance(&impact_pos, &back_gate_pos);
    if back_gate_dist < 3.0 && game.resources.castle.back_gate_intact {
        let collapse_chance = game.resources.bombardment.next_random();
        if collapse_chance < 0.25 {
            game.resources.castle.back_gate_intact = false;
            game.resources.castle.river_accessible = false;
            game.resources
                .waypoints
                .block_edge(pathfinding::NODE_BACK_GATE, pathfinding::NODE_RIVER);

            let north_wall = &mut game.resources.castle.walls[0];
            if north_wall.segments.len() > 2 {
                let segment = &mut north_wall.segments[2];
                segment.breached = true;
                segment.health = 0.0;
                rendering::update_wall_segment_color(world, 0, 2, 0.0);
                if let Some(transform) = world.get_local_transform_mut(segment.entity) {
                    transform.scale.y = 0.3;
                    transform.translation.y = 0.15;
                }
                world.set_local_transform_dirty(segment.entity, LocalTransformDirty);
            }
        }
    }

    let fire_chance = game.resources.bombardment.next_random();
    if fire_chance < 0.4 && impact_pos.x.abs() < CASTLE_SIZE && impact_pos.z.abs() < CASTLE_SIZE {
        spawn_fire_at(game, world, impact_pos);
    }

    let agent_count = game.resources.agents.len();
    for agent_index in 0..agent_count {
        let agent_entity = game.resources.agents[agent_index];
        if let Some(agent) = game.get_agent(agent_entity) {
            let dist = nalgebra_glm::distance(&impact_pos, &agent.position);
            if dist < 2.0 && !agent.wounded {
                let mut agent = agent.clone();
                agent.wounded = true;
                agent.health = (agent.health - 30.0).max(10.0);
                rendering::set_agent_wounded_color(world, &agent.body, agent_index);
                game.set_agent(agent_entity, agent);
            }
        }
    }

    let rubble_count = 3 + (game.resources.bombardment.next_random() * 3.0) as usize;
    let seed = game.resources.bombardment.rng_seed;
    let render_entities = rendering::spawn_rubble_pieces(world, impact_pos, rubble_count, seed);

    let back_gate_node_pos = game.resources.waypoints.positions[pathfinding::NODE_BACK_GATE];
    let blocks_path = if nalgebra_glm::distance(&impact_pos, &back_gate_node_pos) < 4.0 {
        game.resources
            .waypoints
            .block_edge(pathfinding::NODE_BACK_GATE, pathfinding::NODE_RIVER);
        game.resources.castle.river_accessible = false;
        Some((pathfinding::NODE_BACK_GATE, pathfinding::NODE_RIVER))
    } else {
        None
    };

    let game_entity = game.spawn_entities(RUBBLE, 1)[0];
    game.set_rubble(
        game_entity,
        Rubble {
            position: impact_pos,
            entities: render_entities,
            blocks_path,
        },
    );
    game.resources.rubble_list.push(game_entity);
}

pub fn spawn_fire_at(game: &mut GameWorld, world: &mut World, position: Vec3) {
    let seed = game.resources.bombardment.rng_seed;
    let fire_entities = rendering::spawn_fire_cluster(world, position, seed);

    let smoke_entity = rendering::spawn_smoke_column(world, position);
    let light_entity = rendering::spawn_fire_point_light(world, position);

    let near_location = check_near_location(position);

    let game_entity = game.spawn_entities(FIRE, 1)[0];
    game.set_fire(
        game_entity,
        Fire {
            position,
            spread_timer: 0.0,
            entities: fire_entities,
            light_entity: Some(light_entity),
            smoke_entity: Some(smoke_entity),
            near_location,
            doused_amount: 0.0,
        },
    );
    game.resources.fires.push(game_entity);
}

fn check_near_location(position: Vec3) -> Option<LocationId> {
    let locations = [
        (LocationId::Well, castle::WELL_POS, 3.0),
        (LocationId::Armory, castle::ARMORY_POS, 4.0),
        (LocationId::HealingStation, castle::HEALING_POS, 3.0),
        (LocationId::RepairPile, castle::REPAIR_PILE_POS, 3.0),
    ];

    for (location_id, location_pos, radius) in &locations {
        if nalgebra_glm::distance(&position, location_pos) < *radius {
            return Some(*location_id);
        }
    }
    None
}

pub fn impact_system(game: &mut GameWorld, world: &mut World) {
    if game.resources.castle.gate_health <= 0.0 && !game.resources.failure_triggered {
        game.resources.failure_triggered = true;
        game.resources.survival_time = game.resources.elapsed_time;

        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive {
                entity: game.resources.castle.gate_entity,
            });

        for gate_piece in 0..6 {
            let offset = nalgebra_glm::vec3(
                (gate_piece as f32 - 3.0) * 0.8,
                0.2 + (gate_piece as f32) * 0.1,
                castle::CASTLE_SIZE - 0.5 + (gate_piece as f32) * 0.3,
            );
            rendering::spawn_rubble_pieces(
                world,
                castle::GATE_POS + offset,
                2,
                game.resources.bombardment.rng_seed + gate_piece as u64,
            );
        }
    }
}
