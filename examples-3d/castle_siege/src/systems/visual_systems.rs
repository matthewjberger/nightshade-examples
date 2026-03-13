use nightshade::prelude::*;

use crate::ecs::{ENEMY_INVADER, EnemyInvader, GameWorld};
use crate::rendering;

pub fn replan_flash_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;

    let ring_entities: Vec<_> = game.resources.replan_rings.clone();
    let mut to_remove = Vec::new();

    for &ring_entity in &ring_entities {
        if let Some(ring) = game.get_replan_ring(ring_entity) {
            let mut ring = ring.clone();
            ring.timer += delta_time;

            let progress = (ring.timer / ring.max_time).clamp(0.0, 1.0);
            let scale = 0.5 + progress * 2.5;

            if let Some(transform) = world.core.get_local_transform_mut(ring.entity) {
                transform.scale = nalgebra_glm::vec3(scale, 0.02, scale);
            }
            world.core.set_local_transform_dirty(ring.entity, LocalTransformDirty);

            if ring.timer >= ring.max_time {
                to_remove.push((ring_entity, ring.entity));
            } else {
                game.set_replan_ring(ring_entity, ring);
            }
        }
    }

    for (game_entity, render_entity) in to_remove {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive {
                entity: render_entity,
            });
        game.despawn_entities(&[game_entity]);
        game.resources
            .replan_rings
            .retain(|&entity| entity != game_entity);
    }
}

pub fn fire_flicker_system(game: &mut GameWorld, world: &mut World) {
    let elapsed = game.resources.elapsed_time;

    for (fire_index, &fire_entity) in game.resources.fires.iter().enumerate() {
        if let Some(fire) = game.get_fire(fire_entity) {
            for (sphere_index, &sphere_entity) in fire.entities.iter().enumerate() {
                let phase = elapsed * 6.0 + fire_index as f32 * 2.0 + sphere_index as f32 * 1.5;
                let scale_jitter = 1.0 + phase.sin() * 0.3;
                let y_jitter = (phase * 1.3).sin() * 0.1;

                if let Some(transform) = world.core.get_local_transform_mut(sphere_entity) {
                    let base_scale = 0.2 + (sphere_index as f32) * 0.05;
                    transform.scale = nalgebra_glm::vec3(
                        base_scale * scale_jitter,
                        base_scale * 1.5 * scale_jitter,
                        base_scale * scale_jitter,
                    );
                    transform.translation.y =
                        fire.position.y + 0.2 + (sphere_index as f32) * 0.15 + y_jitter;
                }
                world.core.set_local_transform_dirty(sphere_entity, LocalTransformDirty);
            }

            if let Some(smoke_entity) = fire.smoke_entity {
                let smoke_sway = (elapsed * 0.8 + fire_index as f32).sin() * 0.3;
                if let Some(transform) = world.core.get_local_transform_mut(smoke_entity) {
                    transform.translation =
                        fire.position + nalgebra_glm::vec3(smoke_sway, 2.0, smoke_sway * 0.5);
                }
                world.core.set_local_transform_dirty(smoke_entity, LocalTransformDirty);
            }
        }
    }
}

pub fn impact_flash_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;

    let mut to_remove = Vec::new();
    for (index, flash) in game.resources.impact_flashes.iter_mut().enumerate() {
        flash.timer += delta_time;
        let progress = (flash.timer / flash.max_time).clamp(0.0, 1.0);
        let scale = 0.5 + progress * 3.0;
        let fade = 1.0 - progress;

        if let Some(transform) = world.core.get_local_transform_mut(flash.entity) {
            transform.scale = nalgebra_glm::vec3(scale, scale * 0.5, scale);
            transform.scale *= fade;
        }
        world.core.set_local_transform_dirty(flash.entity, LocalTransformDirty);

        if flash.timer >= flash.max_time {
            to_remove.push(index);
        }
    }

    for &index in to_remove.iter().rev() {
        let flash = game.resources.impact_flashes.remove(index);
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive {
                entity: flash.entity,
            });
    }
}

pub fn trail_particle_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;

    let mut to_remove = Vec::new();
    for (index, particle) in game.resources.trail_particles.iter_mut().enumerate() {
        particle.timer += delta_time;
        let progress = (particle.timer / particle.max_time).clamp(0.0, 1.0);
        let scale = 0.3 * (1.0 - progress);

        if let Some(transform) = world.core.get_local_transform_mut(particle.entity) {
            transform.scale = nalgebra_glm::vec3(scale, scale, scale);
        }
        world.core.set_local_transform_dirty(particle.entity, LocalTransformDirty);

        if particle.timer >= particle.max_time {
            to_remove.push(index);
        }
    }

    for &index in to_remove.iter().rev() {
        let particle = game.resources.trail_particles.remove(index);
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive {
                entity: particle.entity,
            });
    }
}

pub fn failure_system(game: &mut GameWorld, world: &mut World) {
    if !game.resources.failure_triggered {
        return;
    }

    let delta_time = world.resources.window.timing.delta_time;
    game.resources.failure_timer += delta_time;

    if !game.resources.failure_invaders_spawned {
        game.resources.failure_invaders_spawned = true;
        for agent_index in 0..game.resources.agents.len() {
            let agent_entity = game.resources.agents[agent_index];
            if let Some(agent) = game.get_agent(agent_entity) {
                let agent = agent.clone();
                rendering::set_agent_color(world, &agent.body, agent_index, [0.4, 0.4, 0.4, 1.0]);
            }
        }

        let gate_pos = crate::castle::GATE_POS;
        for invader_index in 0..20 {
            let offset = nalgebra_glm::vec3(
                (invader_index as f32 - 10.0) * 0.7,
                0.4,
                crate::castle::CASTLE_SIZE + 2.0 + invader_index as f32 * 0.5,
            );
            let position = gate_pos + offset;
            let render_entity = rendering::spawn_invader(world, position, invader_index);

            let game_entity = game.spawn_entities(ENEMY_INVADER, 1)[0];
            let velocity = nalgebra_glm::vec3(
                (invader_index as f32 - 10.0) * 0.1,
                0.0,
                -2.0 - (invader_index as f32) * 0.1,
            );
            game.set_enemy_invader(
                game_entity,
                EnemyInvader {
                    position,
                    entity: render_entity,
                    velocity,
                },
            );
            game.resources.invaders.push(game_entity);
        }
    }

    let invader_entities: Vec<_> = game.resources.invaders.clone();
    for invader_entity in invader_entities {
        if let Some(invader) = game.get_enemy_invader(invader_entity) {
            let mut invader = invader.clone();
            invader.position += invader.velocity * delta_time;

            if let Some(transform) = world.core.get_local_transform_mut(invader.entity) {
                transform.translation = invader.position;
            }
            world.core.set_local_transform_dirty(invader.entity, LocalTransformDirty);

            game.set_enemy_invader(invader_entity, invader);
        }
    }
}
