use nightshade::prelude::*;

use crate::agent::{AgentState, CarriedItem};
use crate::castle;
use crate::ecs::{GameWorld, LocationId};
use crate::goap::{
    self, AGENT_WOUNDED, ARMORY_EXISTS, ActionTarget, BACK_GATE_INTACT, CARRYING_ARROWS,
    CARRYING_REPAIR, CARRYING_WATER, GoalSelectionContext, GoalType, GoapWorldState,
    HEALING_EXISTS, PATH_TO_RIVER_CLEAR, REPAIR_PILE_EXISTS, RUBBLE_EXISTS, WELL_HAS_WATER,
    build_action_table, select_goal_and_plan,
};
use crate::pathfinding;
use crate::rendering;

pub fn agent_planning_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;
    let actions = build_action_table();

    let agent_count = game.resources.agents.len();
    for agent_index in 0..agent_count {
        let entity = game.resources.agents[agent_index];
        let agent = match game.get_agent(entity) {
            Some(agent) => agent.clone(),
            None => continue,
        };

        if game.resources.failure_triggered {
            continue;
        }

        match agent.state {
            AgentState::Idle => {
                let mut agent = agent;
                agent.idle_timer += delta_time;
                if agent.idle_timer < 0.5 {
                    game.set_agent(entity, agent);
                    continue;
                }
                agent.idle_timer = 0.0;

                let current_state = build_world_state(game, &agent);
                let fire_count = game.resources.fires.len();
                let breach_count = count_breaches(&game.resources.castle);
                let archer_posts_empty = game
                    .resources
                    .castle
                    .archer_posts
                    .iter()
                    .filter(|post| post.arrows_remaining == 0)
                    .count();
                let gate_damage = ((1.0
                    - game.resources.castle.gate_health / game.resources.castle.gate_max_health)
                    * 3.0) as i32;
                let rubble_blocking = !game.resources.rubble_list.is_empty()
                    && !game.resources.castle.river_accessible;

                let force_heal = agent.wounded
                    && game.resources.castle.healing_station_exists
                    && agent.health < 50.0;

                let result = if force_heal {
                    let heal_plan =
                        goap::plan_for_goal(&current_state, 0, AGENT_WOUNDED, &actions, 6);
                    heal_plan.map(|plan| (GoalType::TendWounded, plan))
                } else {
                    select_goal_and_plan(&GoalSelectionContext {
                        current_state: &current_state,
                        actions: &actions,
                        fire_count,
                        breach_count,
                        archer_posts_empty,
                        gate_damage_level: gate_damage,
                        agent_wounded: agent.wounded,
                        rubble_blocking,
                        claimed_goals: &game.resources.castle.claimed_goals,
                        agent_index,
                    })
                };

                if let Some((goal, mut plan)) = result {
                    resolve_plan_targets(&mut plan, game, &agent);

                    game.resources
                        .castle
                        .claimed_goals
                        .retain(|(idx, _)| *idx != agent_index);
                    game.resources
                        .castle
                        .claimed_goals
                        .push((agent_index, goal));

                    agent.current_goal = Some(goal);
                    agent.current_plan = plan;
                    agent.current_step = 0;
                    agent.action_progress = 0.0;
                    agent.plan_generation += 1;

                    let [red, green, blue] = goal.color();
                    rendering::update_goal_marker_color(
                        world,
                        agent_index,
                        [
                            red as f32 / 255.0,
                            green as f32 / 255.0,
                            blue as f32 / 255.0,
                            1.0,
                        ],
                        [
                            red as f32 / 255.0 * 1.5,
                            green as f32 / 255.0 * 1.5,
                            blue as f32 / 255.0 * 1.5,
                        ],
                    );

                    if !agent.current_plan.is_empty()
                        && let Some(target_pos) = agent.current_plan[0].target_position
                    {
                        let from_node = game.resources.waypoints.nearest_node(&agent.position);
                        let to_node = game.resources.waypoints.nearest_node(&target_pos);
                        if let Some(path) = game.resources.waypoints.find_path(from_node, to_node) {
                            agent.waypoint_path = path;
                            agent.waypoint_index = 0;
                            agent.target_position = Some(target_pos);
                            agent.state = AgentState::Moving;
                        }
                    }

                    game.resources.planner_feed_dirty = true;
                }

                game.set_agent(entity, agent);
            }
            AgentState::Replanning => {
                let mut agent = agent;
                agent.replan_timer -= delta_time;
                if agent.replan_timer <= 0.0 {
                    agent.state = AgentState::Idle;
                    agent.replan_timer = 0.0;
                    rendering::set_agent_emissive(world, agent_index, [0.0, 0.0, 0.0]);
                }
                game.set_agent(entity, agent);
            }
            _ => {}
        }
    }
}

pub fn agent_movement_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;
    let elapsed = game.resources.elapsed_time;

    let agent_count = game.resources.agents.len();
    for agent_index in 0..agent_count {
        let entity = game.resources.agents[agent_index];
        let agent = match game.get_agent(entity) {
            Some(agent) => agent.clone(),
            None => continue,
        };

        if game.resources.failure_triggered {
            continue;
        }

        let body = agent.body.clone();
        let is_moving = agent.state == AgentState::Moving;

        rendering::update_agent_body_position(world, &body, agent.position, elapsed, is_moving);

        if let Some(carried_entity) = agent.carried_item_entity {
            if let Some(transform) = world.core.get_local_transform_mut(carried_entity) {
                transform.translation = agent.position + nalgebra_glm::vec3(0.35, 0.85, 0.0);
            }
            world
                .core
                .set_local_transform_dirty(carried_entity, LocalTransformDirty);
        }

        if agent.state != AgentState::Moving {
            continue;
        }

        let mut agent = agent;
        let speed = if agent.wounded {
            agent.speed * 0.5
        } else {
            agent.speed
        };

        if agent.waypoint_index < agent.waypoint_path.len() {
            let target_node = agent.waypoint_path[agent.waypoint_index];
            let target_pos = game.resources.waypoints.positions[target_node];
            let direction = target_pos - agent.position;
            let distance = nalgebra_glm::length(&direction);

            if distance < 0.3 {
                agent.waypoint_index += 1;
                if agent.waypoint_index >= agent.waypoint_path.len()
                    && let Some(final_target) = agent.target_position
                {
                    let remaining = nalgebra_glm::distance(&agent.position, &final_target);
                    if remaining < 1.5 {
                        agent.position = final_target;
                        agent.state = AgentState::Performing;
                        agent.action_progress = 0.0;
                    }
                }
            } else {
                let move_dir = nalgebra_glm::normalize(&direction);
                agent.position += move_dir * speed * delta_time;
            }
        } else if let Some(final_target) = agent.target_position {
            let direction = final_target - agent.position;
            let distance = nalgebra_glm::length(&direction);
            if distance < 0.5 {
                agent.position = final_target;
                agent.state = AgentState::Performing;
                agent.action_progress = 0.0;
            } else {
                let move_dir = nalgebra_glm::normalize(&direction);
                agent.position += move_dir * speed * delta_time;
            }
        }

        game.set_agent(entity, agent);
    }
}

pub fn agent_action_system(game: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game.resources.game_speed;

    let agent_count = game.resources.agents.len();
    for agent_index in 0..agent_count {
        let entity = game.resources.agents[agent_index];
        let agent = match game.get_agent(entity) {
            Some(agent) => agent.clone(),
            None => continue,
        };

        if agent.state != AgentState::Performing {
            continue;
        }
        if game.resources.failure_triggered {
            continue;
        }
        if agent.current_step >= agent.current_plan.len() {
            let mut agent = agent;
            agent.state = AgentState::Idle;
            agent.current_plan.clear();
            agent.current_step = 0;
            game.resources
                .castle
                .claimed_goals
                .retain(|(idx, _)| *idx != agent_index);
            release_fire_claim(game, agent_index);
            game.set_agent(entity, agent);
            continue;
        }

        let current_action = agent.current_plan[agent.current_step].action.clone();

        if !validate_preconditions(game, current_action.name, agent_index) {
            trigger_replan(game, world, agent_index, entity, current_action.name);
            continue;
        }

        let mut agent = agent;
        agent.action_progress += delta_time;

        if agent.action_progress >= current_action.duration {
            apply_action_effects(game, world, current_action.name, agent_index);

            agent.current_step += 1;
            agent.action_progress = 0.0;

            if agent.current_step < agent.current_plan.len() {
                let next_action = &agent.current_plan[agent.current_step];
                if let Some(target_pos) = next_action.target_position {
                    let from_node = game.resources.waypoints.nearest_node(&agent.position);
                    let to_node = game.resources.waypoints.nearest_node(&target_pos);
                    if let Some(path) = game.resources.waypoints.find_path(from_node, to_node) {
                        agent.waypoint_path = path;
                        agent.waypoint_index = 0;
                        agent.target_position = Some(target_pos);
                        agent.state = AgentState::Moving;
                    } else {
                        agent.state = AgentState::Idle;
                    }
                }
            } else {
                agent.state = AgentState::Idle;
                agent.current_plan.clear();
                agent.current_step = 0;
                game.resources
                    .castle
                    .claimed_goals
                    .retain(|(idx, _)| *idx != agent_index);
                release_fire_claim(game, agent_index);
            }

            game.resources.planner_feed_dirty = true;
        }

        game.set_agent(entity, agent);
    }
}

fn build_world_state(game: &GameWorld, agent: &crate::agent::Agent) -> GoapWorldState {
    let mut state = GoapWorldState::default();

    if agent.carrying == Some(CarriedItem::Water) {
        state.set_flag(CARRYING_WATER);
    }
    if agent.carrying == Some(CarriedItem::RepairMaterials) {
        state.set_flag(CARRYING_REPAIR);
    }
    if agent.carrying == Some(CarriedItem::Arrows) {
        state.set_flag(CARRYING_ARROWS);
    }
    if !game.resources.castle.well_destroyed && game.resources.castle.well_water_remaining > 0.0 {
        state.set_flag(WELL_HAS_WATER);
    }
    if game.resources.castle.armory_exists {
        state.set_flag(ARMORY_EXISTS);
    }
    if game.resources.castle.healing_station_exists {
        state.set_flag(HEALING_EXISTS);
    }
    if game.resources.castle.repair_pile_count > 0 {
        state.set_flag(REPAIR_PILE_EXISTS);
    }
    if game.resources.castle.back_gate_intact {
        state.set_flag(BACK_GATE_INTACT);
    }
    if game.resources.castle.river_accessible {
        state.set_flag(PATH_TO_RIVER_CLEAR);
    }
    if agent.wounded {
        state.set_flag(AGENT_WOUNDED);
    }
    if !game.resources.rubble_list.is_empty() {
        state.set_flag(RUBBLE_EXISTS);
    }

    state
}

fn count_breaches(castle: &castle::CastleState) -> usize {
    castle
        .walls
        .iter()
        .flat_map(|wall| wall.segments.iter())
        .filter(|segment| segment.breached)
        .count()
}

fn resolve_plan_targets(
    plan: &mut [goap::PlannedAction],
    game: &GameWorld,
    agent: &crate::agent::Agent,
) {
    for step in plan.iter_mut() {
        let (location, position) = match step.action.target {
            ActionTarget::Well => (Some(LocationId::Well), castle::WELL_POS),
            ActionTarget::River => (Some(LocationId::River), castle::RIVER_POS),
            ActionTarget::RepairPile => (Some(LocationId::RepairPile), castle::REPAIR_PILE_POS),
            ActionTarget::Armory => (Some(LocationId::Armory), castle::ARMORY_POS),
            ActionTarget::Gate => (Some(LocationId::Gate), castle::GATE_POS),
            ActionTarget::HealStation => (Some(LocationId::HealingStation), castle::HEALING_POS),
            ActionTarget::Fire => {
                let fire_pos = find_nearest_fire_position(game, &agent.position);
                (None, fire_pos.unwrap_or(castle::WELL_POS))
            }
            ActionTarget::Breach => {
                let (location, pos) = find_nearest_breach(game, &agent.position);
                (location, pos)
            }
            ActionTarget::ArcherPost => {
                let location = find_empty_archer_post(game);
                let pos = location
                    .map(castle::location_position)
                    .unwrap_or(Vec3::zeros());
                (location, pos)
            }
            ActionTarget::RubbleNearest => {
                let rubble_pos = find_nearest_rubble_position(game, &agent.position);
                (None, rubble_pos.unwrap_or(castle::REPAIR_PILE_POS))
            }
            ActionTarget::BackGate => (None, castle::BACK_GATE_POS),
        };
        step.resolved_target = location;
        step.target_position = Some(position);
    }
}

fn find_nearest_fire_position(game: &GameWorld, agent_pos: &Vec3) -> Option<Vec3> {
    let mut best_pos = None;
    let mut best_dist = f32::MAX;

    for &fire_entity in &game.resources.fires {
        if let Some(fire) = game.get_fire(fire_entity) {
            let dist = nalgebra_glm::distance(agent_pos, &fire.position);
            let already_claimed = game
                .resources
                .claimed_fire_targets
                .iter()
                .any(|(_, claimed_entity)| *claimed_entity == fire_entity);
            if dist < best_dist && !already_claimed {
                best_dist = dist;
                best_pos = Some(fire.position);
            }
        }
    }

    if best_pos.is_none() {
        for &fire_entity in &game.resources.fires {
            if let Some(fire) = game.get_fire(fire_entity) {
                let dist = nalgebra_glm::distance(agent_pos, &fire.position);
                if dist < best_dist {
                    best_dist = dist;
                    best_pos = Some(fire.position);
                }
            }
        }
    }

    best_pos
}

fn find_nearest_rubble_position(game: &GameWorld, agent_pos: &Vec3) -> Option<Vec3> {
    let mut best_pos = None;
    let mut best_dist = f32::MAX;

    for &rubble_entity in &game.resources.rubble_list {
        if let Some(rubble) = game.get_rubble(rubble_entity) {
            let dist = nalgebra_glm::distance(agent_pos, &rubble.position);
            if dist < best_dist {
                best_dist = dist;
                best_pos = Some(rubble.position);
            }
        }
    }

    best_pos
}

fn find_nearest_breach(game: &GameWorld, agent_pos: &Vec3) -> (Option<LocationId>, Vec3) {
    let mut best_location = None;
    let mut best_pos = Vec3::zeros();
    let mut best_dist = f32::MAX;

    for (wall_index, wall) in game.resources.castle.walls.iter().enumerate() {
        for segment in &wall.segments {
            if segment.breached {
                let dist = nalgebra_glm::distance(agent_pos, &segment.position);
                if dist < best_dist {
                    best_dist = dist;
                    best_pos = segment.position;
                    best_location = Some(match wall_index {
                        0 => LocationId::WallNorth,
                        1 => LocationId::WallSouth,
                        2 => LocationId::WallEast,
                        _ => LocationId::WallWest,
                    });
                }
            }
        }
    }

    (best_location, best_pos)
}

fn find_empty_archer_post(game: &GameWorld) -> Option<LocationId> {
    for (index, post) in game.resources.castle.archer_posts.iter().enumerate() {
        if post.arrows_remaining == 0 {
            return Some(LocationId::ArcherPost(index));
        }
    }
    None
}

fn release_fire_claim(game: &mut GameWorld, agent_index: usize) {
    game.resources
        .claimed_fire_targets
        .retain(|(idx, _)| *idx != agent_index);
}

fn validate_preconditions(game: &GameWorld, action_name: &str, agent_index: usize) -> bool {
    match action_name {
        "FetchWaterWell" => {
            !game.resources.castle.well_destroyed
                && game.resources.castle.well_water_remaining > 0.0
        }
        "FetchWaterRiver" => {
            game.resources.castle.back_gate_intact && game.resources.castle.river_accessible
        }
        "DouseFire" => {
            let entity = game.resources.agents[agent_index];
            game.get_agent(entity)
                .is_some_and(|agent| agent.carrying == Some(CarriedItem::Water))
                && !game.resources.fires.is_empty()
        }
        "FetchRepairMaterials" => game.resources.castle.repair_pile_count > 0,
        "SalvageRubble" => !game.resources.rubble_list.is_empty(),
        "RepairWall" => {
            let entity = game.resources.agents[agent_index];
            game.get_agent(entity)
                .is_some_and(|agent| agent.carrying == Some(CarriedItem::RepairMaterials))
        }
        "FetchArrows" => {
            game.resources.castle.armory_exists && game.resources.castle.armory_stock > 0
        }
        "ResupplyArcher" => {
            let entity = game.resources.agents[agent_index];
            game.get_agent(entity)
                .is_some_and(|agent| agent.carrying == Some(CarriedItem::Arrows))
        }
        "ReinforceGate" => {
            let entity = game.resources.agents[agent_index];
            game.get_agent(entity)
                .is_some_and(|agent| agent.carrying == Some(CarriedItem::RepairMaterials))
        }
        "TendWounded" => game.resources.castle.healing_station_exists,
        "ClearRubble" => !game.resources.rubble_list.is_empty(),
        "RepairBackGate" => {
            let agent_entity = game.resources.agents[agent_index];
            game.get_agent(agent_entity)
                .is_some_and(|agent| agent.carrying == Some(CarriedItem::RepairMaterials))
        }
        _ => true,
    }
}

fn trigger_replan(
    game: &mut GameWorld,
    world: &mut World,
    agent_index: usize,
    entity: freecs::Entity,
    failed_action: &str,
) {
    let mut agent = game.get_agent(entity).cloned().unwrap();
    agent.state = AgentState::Replanning;
    agent.replan_timer = 0.8;
    agent.replan_reason = format!("{} failed", failed_action);
    agent.current_plan.clear();
    agent.current_step = 0;
    agent.action_progress = 0.0;

    game.resources
        .castle
        .claimed_goals
        .retain(|(idx, _)| *idx != agent_index);
    release_fire_claim(game, agent_index);

    rendering::set_agent_emissive(world, agent_index, [2.0, 0.0, 2.0]);

    let ring_entity = rendering::spawn_replan_ring(world, agent.position);
    let game_ring = game.spawn_entities(crate::ecs::REPLAN_RING, 1)[0];
    game.set_replan_ring(
        game_ring,
        crate::ecs::ReplanRing {
            entity: ring_entity,
            timer: 0.0,
            max_time: 0.5,
        },
    );
    game.resources.replan_rings.push(game_ring);

    game.resources.planner_feed_dirty = true;
    game.set_agent(entity, agent);
}

fn apply_action_effects(
    game: &mut GameWorld,
    world: &mut World,
    action_name: &str,
    agent_index: usize,
) {
    let entity = game.resources.agents[agent_index];

    match action_name {
        "FetchWaterWell" => {
            game.resources.castle.well_water_remaining -= 10.0;
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.carrying = Some(CarriedItem::Water);
            let item_entity =
                rendering::spawn_carried_item(world, agent.position, CarriedItem::Water);
            agent.carried_item_entity = Some(item_entity);
            game.set_agent(entity, agent);
        }
        "FetchWaterRiver" => {
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.carrying = Some(CarriedItem::Water);
            let item_entity =
                rendering::spawn_carried_item(world, agent.position, CarriedItem::Water);
            agent.carried_item_entity = Some(item_entity);
            game.set_agent(entity, agent);
        }
        "DouseFire" => {
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.carrying = None;
            if let Some(item_entity) = agent.carried_item_entity.take() {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: item_entity,
                    });
            }
            game.set_agent(entity, agent);

            let nearest_fire = find_nearest_fire_for_dousing(game, agent_index);
            if let Some(fire_entity) = nearest_fire
                && let Some(fire) = game.get_fire(fire_entity)
            {
                let mut fire = fire.clone();
                fire.doused_amount += 50.0;
                if fire.doused_amount >= 100.0 {
                    for &fire_render_entity in &fire.entities {
                        world
                            .resources
                            .command_queue
                            .push(WorldCommand::DespawnRecursive {
                                entity: fire_render_entity,
                            });
                    }
                    if let Some(light) = fire.light_entity {
                        world
                            .resources
                            .command_queue
                            .push(WorldCommand::DespawnRecursive { entity: light });
                    }
                    if let Some(smoke) = fire.smoke_entity {
                        world
                            .resources
                            .command_queue
                            .push(WorldCommand::DespawnRecursive { entity: smoke });
                    }
                    game.despawn_entities(&[fire_entity]);
                    game.resources.fires.retain(|&entity| entity != fire_entity);
                    release_fire_claim(game, agent_index);
                } else {
                    game.set_fire(fire_entity, fire);
                }
            }
        }
        "FetchRepairMaterials" => {
            game.resources.castle.repair_pile_count =
                game.resources.castle.repair_pile_count.saturating_sub(1);
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.carrying = Some(CarriedItem::RepairMaterials);
            let item_entity =
                rendering::spawn_carried_item(world, agent.position, CarriedItem::RepairMaterials);
            agent.carried_item_entity = Some(item_entity);
            game.set_agent(entity, agent);
        }
        "SalvageRubble" => {
            game.resources.castle.repair_pile_count += 3;
            let agent_pos = game
                .get_agent(entity)
                .map(|agent| agent.position)
                .unwrap_or_default();
            if let Some(rubble_entity) = find_nearest_rubble_entity(game, &agent_pos)
                && let Some(rubble) = game.get_rubble(rubble_entity)
            {
                let rubble = rubble.clone();
                for &render_entity in &rubble.entities {
                    world
                        .resources
                        .command_queue
                        .push(WorldCommand::DespawnRecursive {
                            entity: render_entity,
                        });
                }
                game.despawn_entities(&[rubble_entity]);
                game.resources.rubble_list.retain(|&e| e != rubble_entity);
            }
        }
        "RepairWall" => {
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.carrying = None;
            if let Some(item_entity) = agent.carried_item_entity.take() {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: item_entity,
                    });
            }
            game.set_agent(entity, agent);

            for (wall_index, wall) in game.resources.castle.walls.iter_mut().enumerate() {
                for (segment_index, segment) in wall.segments.iter_mut().enumerate() {
                    if segment.breached {
                        segment.health = segment.max_health * 0.5;
                        segment.breached = false;
                        rendering::update_wall_segment_color(
                            world,
                            wall_index,
                            segment_index,
                            segment.health / segment.max_health,
                        );
                        let (scale_x, scale_z) = match wall_index {
                            0 | 1 => (
                                crate::castle::WALL_SEGMENT_WIDTH,
                                crate::castle::WALL_THICKNESS,
                            ),
                            _ => (
                                crate::castle::WALL_THICKNESS,
                                crate::castle::WALL_SEGMENT_WIDTH,
                            ),
                        };
                        if let Some(transform) = world.core.get_local_transform_mut(segment.entity)
                        {
                            transform.scale =
                                nalgebra_glm::vec3(scale_x, crate::castle::WALL_HEIGHT, scale_z);
                            transform.translation.y = segment.position.y;
                        }
                        world
                            .core
                            .set_local_transform_dirty(segment.entity, LocalTransformDirty);
                        return;
                    }
                }
            }
        }
        "ClearRubble" => {
            let agent_pos = game
                .get_agent(entity)
                .map(|agent| agent.position)
                .unwrap_or_default();
            if let Some(rubble_entity) = find_nearest_rubble_entity(game, &agent_pos)
                && let Some(rubble) = game.get_rubble(rubble_entity)
            {
                let rubble = rubble.clone();
                if let Some((from, to)) = rubble.blocks_path {
                    game.resources.waypoints.unblock_edge(from, to);
                }
                for &render_entity in &rubble.entities {
                    world
                        .resources
                        .command_queue
                        .push(WorldCommand::DespawnRecursive {
                            entity: render_entity,
                        });
                }
                game.despawn_entities(&[rubble_entity]);
                game.resources
                    .rubble_list
                    .retain(|&entity| entity != rubble_entity);
            }

            let has_blocking_rubble = game.resources.rubble_list.iter().any(|&rubble_entity| {
                game.get_rubble(rubble_entity)
                    .is_some_and(|rubble| rubble.blocks_path.is_some())
            });
            game.resources.castle.river_accessible =
                game.resources.castle.back_gate_intact && !has_blocking_rubble;
        }
        "FetchArrows" => {
            game.resources.castle.armory_stock =
                game.resources.castle.armory_stock.saturating_sub(5);
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.carrying = Some(CarriedItem::Arrows);
            let item_entity =
                rendering::spawn_carried_item(world, agent.position, CarriedItem::Arrows);
            agent.carried_item_entity = Some(item_entity);
            game.set_agent(entity, agent);
        }
        "ResupplyArcher" => {
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.carrying = None;
            if let Some(item_entity) = agent.carried_item_entity.take() {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: item_entity,
                    });
            }
            game.set_agent(entity, agent);

            for post in &mut game.resources.castle.archer_posts {
                if post.arrows_remaining == 0 {
                    post.arrows_remaining = post.max_arrows;
                    break;
                }
            }
        }
        "ReinforceGate" => {
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.carrying = None;
            if let Some(item_entity) = agent.carried_item_entity.take() {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: item_entity,
                    });
            }
            game.set_agent(entity, agent);

            game.resources.castle.gate_health = (game.resources.castle.gate_health + 50.0)
                .min(game.resources.castle.gate_max_health);
            rendering::update_gate_color(
                world,
                game.resources.castle.gate_health / game.resources.castle.gate_max_health,
            );
        }
        "TendWounded" => {
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.wounded = false;
            agent.health = 100.0;
            rendering::set_agent_healthy_color(world, &agent.body, agent_index);
            game.set_agent(entity, agent);
        }
        "RepairBackGate" => {
            let mut agent = game.get_agent(entity).cloned().unwrap();
            agent.carrying = None;
            if let Some(item_entity) = agent.carried_item_entity.take() {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: item_entity,
                    });
            }
            game.set_agent(entity, agent);

            game.resources.castle.back_gate_intact = true;

            if game.resources.castle.walls[0].segments.len() > 2
                && game.resources.castle.walls[0].segments[2].breached
            {
                let segment = &mut game.resources.castle.walls[0].segments[2];
                segment.breached = false;
                segment.health = segment.max_health * 0.5;
                let ratio = segment.health / segment.max_health;
                let segment_entity = segment.entity;
                let segment_y = segment.position.y;
                rendering::update_wall_segment_color(world, 0, 2, ratio);
                if let Some(transform) = world.core.get_local_transform_mut(segment_entity) {
                    transform.scale = nalgebra_glm::vec3(
                        crate::castle::WALL_SEGMENT_WIDTH,
                        crate::castle::WALL_HEIGHT,
                        crate::castle::WALL_THICKNESS,
                    );
                    transform.translation.y = segment_y;
                }
                world
                    .core
                    .set_local_transform_dirty(segment_entity, LocalTransformDirty);
            }

            let has_blocking_rubble = game.resources.rubble_list.iter().any(|&rubble_entity| {
                game.get_rubble(rubble_entity)
                    .is_some_and(|rubble| rubble.blocks_path.is_some())
            });
            if !has_blocking_rubble {
                game.resources.castle.river_accessible = true;
                game.resources
                    .waypoints
                    .unblock_edge(pathfinding::NODE_BACK_GATE, pathfinding::NODE_RIVER);
            }
        }
        _ => {}
    }
}

fn find_nearest_rubble_entity(game: &GameWorld, agent_pos: &Vec3) -> Option<freecs::Entity> {
    let mut best_entity = None;
    let mut best_dist = f32::MAX;

    for &rubble_entity in &game.resources.rubble_list {
        if let Some(rubble) = game.get_rubble(rubble_entity) {
            let dist = nalgebra_glm::distance(agent_pos, &rubble.position);
            if dist < best_dist {
                best_dist = dist;
                best_entity = Some(rubble_entity);
            }
        }
    }

    best_entity
}

fn find_nearest_fire_for_dousing(game: &GameWorld, agent_index: usize) -> Option<freecs::Entity> {
    let entity = game.resources.agents[agent_index];
    let agent = game.get_agent(entity)?;
    let agent_pos = agent.position;

    let mut best_entity = None;
    let mut best_dist = f32::MAX;

    for &fire_entity in &game.resources.fires {
        if let Some(fire) = game.get_fire(fire_entity) {
            let dist = nalgebra_glm::distance(&agent_pos, &fire.position);
            if dist < best_dist {
                best_dist = dist;
                best_entity = Some(fire_entity);
            }
        }
    }

    best_entity
}
