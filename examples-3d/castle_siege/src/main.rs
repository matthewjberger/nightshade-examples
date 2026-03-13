mod agent;
mod bombardment;
mod castle;
mod ecs;
mod goap;
mod pathfinding;
mod rendering;
mod systems;
mod ui;

use nightshade::prelude::*;

use crate::agent::{Agent, AgentState};
use crate::ecs::GameWorld;

const AGENT_NAMES: [&str; 6] = ["Aldric", "Brenna", "Cedric", "Dahlia", "Edmund", "Freya"];

const AGENT_SPAWN_POSITIONS: [Vec3; 6] = [
    Vec3::new(-3.0, 0.0, 0.0),
    Vec3::new(3.0, 0.0, 0.0),
    Vec3::new(0.0, 0.0, -5.0),
    Vec3::new(-6.0, 0.0, 3.0),
    Vec3::new(6.0, 0.0, 3.0),
    Vec3::new(0.0, 0.0, 5.0),
];

#[derive(Default)]
struct CastleSiegeState {
    game: GameWorld,
    initialized: bool,
    camera_entity: Entity,
}

impl State for CastleSiegeState {
    fn title(&self) -> &str {
        "GOAP Castle Siege"
    }

    fn initialize(&mut self, world: &mut World) {
        if !self.initialized {
            self.initialized = true;

            world.resources.user_interface.enabled = true;
            world.resources.graphics.show_grid = false;
            world.resources.graphics.bloom_enabled = true;
            world.resources.graphics.bloom_intensity = 0.005;
            world.resources.graphics.atmosphere = Atmosphere::Nebula;

            let camera = spawn_pan_orbit_camera(
                world,
                nalgebra_glm::vec3(0.0, 0.0, 0.0),
                35.0,
                0.0,
                0.7,
                "MainCamera".to_string(),
            );
            world.resources.active_camera = Some(camera);
            self.camera_entity = camera;

            setup_sun(world);
            rendering::init_shared_materials(world);
        }

        self.game.resources.camera_entity = self.camera_entity;

        let castle_state = castle::spawn_castle(world);
        self.game.resources.castle = castle_state;

        for (agent_index, &spawn_pos) in AGENT_SPAWN_POSITIONS.iter().enumerate() {
            let body = rendering::spawn_agent_body(world, agent_index, spawn_pos);

            let game_entity = self.game.spawn_entities(ecs::AGENT, 1)[0];
            self.game.set_agent(
                game_entity,
                Agent {
                    name: AGENT_NAMES[agent_index].to_string(),
                    position: spawn_pos,
                    health: 100.0,
                    speed: 3.0,
                    body,
                    state: AgentState::Idle,
                    ..Default::default()
                },
            );
            self.game.resources.agents.push(game_entity);
        }

        self.game.resources.game_speed = 1.0;
        self.game.resources.waypoints = pathfinding::WaypointGraph::default();
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);

        if self.game.resources.restart_requested {
            cleanup_render_entities(&self.game, world);
            self.game = GameWorld::default();
            self.initialize(world);
            return;
        }

        if self.game.resources.paused && !self.game.resources.failure_triggered {
            return;
        }

        let delta_time = world.resources.window.timing.delta_time * self.game.resources.game_speed;
        self.game.resources.elapsed_time += delta_time;

        systems::boulder_spawn_system(&mut self.game, world);
        systems::boulder_physics_system(&mut self.game, world);
        systems::impact_system(&mut self.game, world);

        systems::fire_spread_system(&mut self.game, world);
        systems::archer_system(&mut self.game, world);
        systems::resource_depletion_system(&mut self.game, world);
        systems::fire_proximity_damage_system(&mut self.game, world);

        systems::agent_planning_system(&mut self.game, world);
        systems::agent_movement_system(&mut self.game, world);
        systems::agent_action_system(&mut self.game, world);

        systems::fire_flicker_system(&mut self.game, world);
        systems::impact_flash_system(&mut self.game, world);
        systems::trail_particle_system(&mut self.game, world);
        systems::replan_flash_system(&mut self.game, world);
        systems::failure_system(&mut self.game, world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        ui::draw_ui(&mut self.game, world, ui_context);
    }
}

fn setup_sun(world: &mut World) {
    let sun = spawn_sun(world);
    if let Some(light) = world.core.get_light_mut(sun) {
        light.color = nalgebra_glm::vec3(1.0, 0.95, 0.85);
        light.intensity = 3.0;
        light.cast_shadows = true;
    }
}

fn cleanup_render_entities(game: &GameWorld, world: &mut World) {
    for &entity in &game.resources.castle.all_render_entities {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive { entity });
    }

    for &agent_entity in &game.resources.agents {
        if let Some(agent) = game.get_agent(agent_entity) {
            for body_entity in [
                agent.body.head,
                agent.body.torso,
                agent.body.left_arm,
                agent.body.right_arm,
                agent.body.left_leg,
                agent.body.right_leg,
                agent.body.goal_marker,
            ] {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: body_entity,
                    });
            }
            if let Some(carried) = agent.carried_item_entity {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive { entity: carried });
            }
        }
    }

    for &fire_entity in &game.resources.fires {
        if let Some(fire) = game.get_fire(fire_entity) {
            for &render_entity in &fire.entities {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: render_entity,
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
        }
    }

    for &boulder_entity in &game.resources.boulders {
        if let Some(handle) = game.get_entity_handle(boulder_entity) {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive { entity: handle.0 });
        }
    }

    for &rubble_entity in &game.resources.rubble_list {
        if let Some(rubble) = game.get_rubble(rubble_entity) {
            for &render_entity in &rubble.entities {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: render_entity,
                    });
            }
        }
    }

    for &ring_entity in &game.resources.replan_rings {
        if let Some(ring) = game.get_replan_ring(ring_entity) {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive {
                    entity: ring.entity,
                });
        }
    }

    for flash in &game.resources.impact_flashes {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive {
                entity: flash.entity,
            });
    }

    for particle in &game.resources.trail_particles {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive {
                entity: particle.entity,
            });
    }

    for &invader_entity in &game.resources.invaders {
        if let Some(invader) = game.get_enemy_invader(invader_entity) {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive {
                    entity: invader.entity,
                });
        }
    }

    for post in &game.resources.castle.archer_posts {
        if let Some(line_entity) = post.line_entity {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive {
                    entity: line_entity,
                });
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(CastleSiegeState::default())
}
