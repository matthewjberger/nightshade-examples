use std::collections::VecDeque;

use nightshade::ecs::material::material_registry_insert;
use nightshade::ecs::world::commands::despawn_entities_with_cache_cleanup;
use nightshade::prelude::*;
use rand::Rng;

use crate::agent::{
    self, Agent, agent_name, collect_agent_entities, spawn_agent_body, spawn_agent_name_label,
    spawn_home, sync_agent_body_transforms, update_agent_material, upgrade_home,
};
use crate::environment::Environment;
use crate::genome::{Genome, produce_next_generation};
use crate::popup::{self, Popup};
use crate::qlearning::{
    Action, AgentState, QTable, RewardContext, bucket, compute_reward, select_action,
    update_q_value,
};

pub const AGENT_COUNT: usize = 18;
const MAX_LOG_ENTRIES: usize = 200;

#[derive(Clone)]
pub struct EventLogEntry {
    pub message: String,
    pub color: [f32; 3],
}

pub struct GenerationStats {
    pub avg_survival: f32,
    pub best_survival: f32,
    pub trait_averages: [f32; 5],
}

pub struct Simulation {
    pub agents: Vec<Agent>,
    pub popups: Vec<Popup>,
    pub event_log: VecDeque<EventLogEntry>,
    pub generation: usize,
    pub generation_timer: f32,
    pub generation_length: f32,
    pub tick_accumulator: f32,
    pub ticks_per_second: f32,
    pub speed_multiplier: f32,
    pub paused: bool,
    pub selected_agent: Option<usize>,
    pub history: Vec<GenerationStats>,
    pub generation_flash_timer: f32,
    pub rng: rand::rngs::ThreadRng,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            popups: Vec::new(),
            event_log: VecDeque::new(),
            generation: 0,
            generation_timer: 0.0,
            generation_length: 60.0,
            tick_accumulator: 0.0,
            ticks_per_second: 10.0,
            speed_multiplier: 1.0,
            paused: false,
            selected_agent: None,
            history: Vec::new(),
            generation_flash_timer: 0.0,
            rng: rand::rng(),
        }
    }

    pub fn log_event(&mut self, message: String, color: [f32; 3]) {
        self.event_log.push_back(EventLogEntry { message, color });
        if self.event_log.len() > MAX_LOG_ENTRIES {
            self.event_log.pop_front();
        }
    }

    pub fn start_generation(
        &mut self,
        world: &mut World,
        environment: &Environment,
        offspring: Option<Vec<(Genome, QTable)>>,
    ) {
        self.generation += 1;
        self.generation_timer = 0.0;

        let mut agents = Vec::with_capacity(AGENT_COUNT);

        let genomes_and_tables: Vec<(Genome, QTable)> = match offspring {
            Some(data) => data,
            None => (0..AGENT_COUNT)
                .map(|_| (Genome::random(&mut self.rng), QTable::new()))
                .collect(),
        };

        for (index, (genome, q_table)) in genomes_and_tables.into_iter().enumerate() {
            let angle = (index as f32 / AGENT_COUNT as f32) * std::f32::consts::TAU;
            let radius = self.rng.random_range(1.0..5.0f32);
            let position = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);

            let home_angle = (index as f32 / AGENT_COUNT as f32) * std::f32::consts::TAU;
            let home_radius = 7.0;
            let home_position = Vec3::new(
                home_angle.cos() * home_radius,
                0.0,
                home_angle.sin() * home_radius,
            );

            let material_name = format!("agent_{index}");
            register_agent_material(world, &material_name);

            let body = spawn_agent_body(world, position, &material_name);
            let name = agent_name(index).to_string();
            let name_entity = spawn_agent_name_label(world, &name, position);
            let mut agent = Agent::new(
                body,
                material_name,
                name,
                name_entity,
                genome,
                q_table,
                position,
            );
            agent.home_position = home_position;

            agent.home_entities = spawn_home(world, home_position, 0);

            agents.push(agent);
        }

        self.agents = agents;
        self.selected_agent = None;

        self.log_event(
            format!(
                "Generation {} started ({} agents)",
                self.generation, AGENT_COUNT
            ),
            [0.8, 0.8, 0.8],
        );

        let _ = environment;
    }

    pub fn end_generation(&mut self, world: &mut World, environment: &mut Environment) {
        let mut survival_times: Vec<f32> = self.agents.iter().map(|a| a.survival_time).collect();
        survival_times.sort_by(|a, b| b.partial_cmp(a).unwrap());

        let avg_survival = if survival_times.is_empty() {
            0.0
        } else {
            survival_times.iter().sum::<f32>() / survival_times.len() as f32
        };

        let best_survival = survival_times.first().copied().unwrap_or(0.0);

        let genomes: Vec<Genome> = self.agents.iter().map(|a| a.genome.clone()).collect();
        let trait_averages = Genome::trait_averages(&genomes);

        self.history.push(GenerationStats {
            avg_survival,
            best_survival,
            trait_averages,
        });

        self.log_event(
            format!(
                "Generation {} ended - avg {:.1}s, best {:.1}s",
                self.generation, avg_survival, best_survival
            ),
            [0.6, 0.6, 0.6],
        );

        let agent_fitness: Vec<(Genome, f32)> = self
            .agents
            .iter()
            .map(|a| (a.genome.clone(), a.survival_time))
            .collect();

        let best_q_table = self
            .agents
            .iter()
            .max_by(|a, b| a.survival_time.partial_cmp(&b.survival_time).unwrap())
            .map(|a| a.q_table.clone())
            .unwrap_or_else(QTable::new);

        let offspring =
            produce_next_generation(&agent_fitness, AGENT_COUNT, &best_q_table, &mut self.rng);

        let entities = collect_agent_entities(&self.agents);
        despawn_entities_with_cache_cleanup(world, &entities);
        self.agents.clear();

        popup::despawn_all_popups(&mut self.popups, world);

        self.generation_flash_timer = 0.5;

        environment.reset_wolf();
        environment.despawn_campfires(world);

        self.start_generation(world, environment, Some(offspring));
    }

    pub fn alive_count(&self) -> usize {
        self.agents.iter().filter(|a| a.alive).count()
    }

    pub fn tick(&mut self, world: &mut World, environment: &mut Environment, tick_interval: f32) {
        let is_night = environment.day_night.is_night();

        environment.day_night.advance(tick_interval);

        let ticks = tick_interval * 10.0;

        let alive_positions: Vec<(usize, Vec3)> = self
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive)
            .map(|(index, a)| (index, a.position))
            .collect();

        let mut nearby_counts = vec![0usize; self.agents.len()];
        for (outer_index, outer_pos) in &alive_positions {
            let mut count = 0usize;
            for (inner_index, inner_pos) in &alive_positions {
                if outer_index == inner_index {
                    continue;
                }
                let dist = nalgebra_glm::distance(&outer_pos.xz(), &inner_pos.xz());
                if dist < 5.0 {
                    count += 1;
                }
            }
            nearby_counts[*outer_index] = count;
        }

        for (agent_index, &nearby_count) in nearby_counts.iter().enumerate() {
            if !self.agents[agent_index].alive {
                continue;
            }
            let prev_nearby = self.agents[agent_index].nearby_agent_count;
            self.agents[agent_index].nearby_agent_count = nearby_count;

            let hunger_rate = 0.004 * (0.5 + self.agents[agent_index].genome.metabolism);
            let mut energy_rate = 0.0025 * (0.5 + self.agents[agent_index].genome.metabolism);

            if is_night {
                energy_rate *= 1.5;
            }

            let energy_multiplier = self.agents[agent_index].energy_decay_multiplier();
            let hunger_multiplier = self.agents[agent_index].hunger_decay_multiplier();

            let near_campfire = environment.agent_near_campfire(&self.agents[agent_index].position);
            let campfire_energy_mult = if near_campfire { 0.6 } else { 1.0 };

            let loneliness_cascade = if self.agents[agent_index].needs.loneliness > 0.5 {
                1.0 + (self.agents[agent_index].needs.loneliness - 0.5)
            } else {
                1.0
            };

            self.agents[agent_index].needs.hunger = (self.agents[agent_index].needs.hunger
                + hunger_rate * hunger_multiplier * loneliness_cascade * ticks)
                .min(1.0);
            self.agents[agent_index].needs.energy = (self.agents[agent_index].needs.energy
                + energy_rate
                    * energy_multiplier
                    * campfire_energy_mult
                    * loneliness_cascade
                    * ticks)
                .min(1.0);

            let sociability = self.agents[agent_index].genome.sociability;
            let loneliness_delta = match nearby_count {
                0 => 0.004 * (0.5 + sociability) * ticks,
                1 => 0.001 * ticks,
                2 => 0.0,
                _ => -0.02 * ticks,
            };
            self.agents[agent_index].needs.loneliness =
                (self.agents[agent_index].needs.loneliness + loneliness_delta).clamp(0.0, 1.0);

            let currently_lonely = self.agents[agent_index].needs.loneliness > 0.5;
            let was_lonely = self.agents[agent_index].was_lonely;
            if currently_lonely && !was_lonely {
                let pos = self.agents[agent_index].position;
                let color = nalgebra_glm::vec4(0.6, 0.4, 0.8, 1.0);
                self.popups
                    .push(popup::spawn_popup(world, "-Lonely", pos, color));
            }
            self.agents[agent_index].was_lonely = currently_lonely;

            let currently_grouped = nearby_count >= 2;
            let was_grouped = self.agents[agent_index].was_grouped;
            if currently_grouped && !was_grouped && prev_nearby < 2 {
                let pos = self.agents[agent_index].position;
                let color = nalgebra_glm::vec4(0.2, 0.8, 0.4, 1.0);
                self.popups
                    .push(popup::spawn_popup(world, "+Social", pos, color));
            }
            self.agents[agent_index].was_grouped = currently_grouped;

            self.agents[agent_index].action_cooldown -= tick_interval;
            if self.agents[agent_index].action_cooldown <= 0.0 {
                let wolf_distance = nalgebra_glm::distance(
                    &self.agents[agent_index].position.xz(),
                    &environment.wolf.position.xz(),
                );
                let hunt_radius = environment.wolf.hunt_radius(is_night);
                let threat_nearby = wolf_distance < hunt_radius;
                let at_home = self.agents[agent_index].is_at_home();

                let state = AgentState {
                    hunger_bucket: bucket(self.agents[agent_index].needs.hunger),
                    energy_bucket: bucket(self.agents[agent_index].needs.energy),
                    loneliness_bucket: bucket(self.agents[agent_index].needs.loneliness),
                    threat_nearby,
                    at_home,
                    is_night,
                };
                let state_index = state.encode();

                let action = select_action(
                    &self.agents[agent_index].q_table,
                    state_index,
                    &mut self.rng,
                );
                self.agents[agent_index].current_action = action;
                self.agents[agent_index].action_cooldown = 0.5;

                let target = compute_target(
                    action,
                    &self.agents[agent_index],
                    environment,
                    &mut self.rng,
                );
                self.agents[agent_index].target = Some(target);
            }

            let in_food = environment
                .zones
                .in_any_food(&self.agents[agent_index].position);
            let in_rest = environment
                .zones
                .in_rest(&self.agents[agent_index].position);

            if in_food {
                self.agents[agent_index].needs.hunger =
                    (self.agents[agent_index].needs.hunger - 0.06).max(0.0);
                if self.agents[agent_index].flash_timer <= 0.0 {
                    self.agents[agent_index].trigger_flash([1.0, 0.3, 0.1]);
                }
                if !self.agents[agent_index].was_in_food {
                    let pos = self.agents[agent_index].position;
                    let color = nalgebra_glm::vec4(1.0, 0.6, 0.1, 1.0);
                    self.popups
                        .push(popup::spawn_popup(world, "+Food", pos, color));
                    let name = self.agents[agent_index].name.clone();
                    self.log_event(format!("{name} is eating"), [1.0, 0.6, 0.1]);
                }
            }
            if in_rest {
                self.agents[agent_index].needs.energy =
                    (self.agents[agent_index].needs.energy - 0.06).max(0.0);
                if self.agents[agent_index].flash_timer <= 0.0 {
                    self.agents[agent_index].trigger_flash([0.1, 0.3, 1.0]);
                }
                if !self.agents[agent_index].was_in_rest {
                    let pos = self.agents[agent_index].position;
                    let color = nalgebra_glm::vec4(0.3, 0.5, 1.0, 1.0);
                    self.popups
                        .push(popup::spawn_popup(world, "+Rest", pos, color));
                    let name = self.agents[agent_index].name.clone();
                    self.log_event(format!("{name} is resting"), [0.3, 0.5, 1.0]);
                }
            }

            if self.agents[agent_index].current_action == Action::Build {
                let at_home = self.agents[agent_index].is_at_home();
                if at_home && self.agents[agent_index].home_level < 2 {
                    let build_speed =
                        0.02 * (1.0 + self.agents[agent_index].genome.home_investment);
                    self.agents[agent_index].build_progress += build_speed;
                    if self.agents[agent_index].build_progress >= 1.0 {
                        self.agents[agent_index].build_progress = 0.0;
                        upgrade_home(world, &mut self.agents[agent_index]);
                        let name = self.agents[agent_index].name.clone();
                        let level = self.agents[agent_index].home_level;
                        self.log_event(
                            format!("{name} upgraded home to level {level}"),
                            [0.6, 0.4, 0.2],
                        );
                        let pos = self.agents[agent_index].position;
                        let color = nalgebra_glm::vec4(0.6, 0.4, 0.2, 1.0);
                        self.popups
                            .push(popup::spawn_popup(world, "+Home", pos, color));
                    }
                } else if !at_home
                    && !environment.agent_near_campfire(&self.agents[agent_index].position)
                {
                    let build_speed =
                        0.02 * (1.0 + self.agents[agent_index].genome.home_investment);
                    self.agents[agent_index].campfire_build_progress += build_speed;
                    if self.agents[agent_index].campfire_build_progress >= 1.5 {
                        self.agents[agent_index].campfire_build_progress = 0.0;
                        let pos = self.agents[agent_index].position;
                        environment.spawn_campfire(world, pos);
                        let name = self.agents[agent_index].name.clone();
                        self.log_event(format!("{name} built a campfire"), [1.0, 0.5, 0.1]);
                        let color = nalgebra_glm::vec4(1.0, 0.5, 0.1, 1.0);
                        self.popups
                            .push(popup::spawn_popup(world, "+Fire", pos, color));
                    }
                }
            }

            self.agents[agent_index].was_in_food = in_food;
            self.agents[agent_index].was_in_rest = in_rest;

            let wolf_distance = nalgebra_glm::distance(
                &self.agents[agent_index].position.xz(),
                &environment.wolf.position.xz(),
            );
            let near_wolf = wolf_distance < 8.0;
            let fleeing = self.agents[agent_index].current_action == Action::Flee;
            let at_home = self.agents[agent_index].is_at_home();

            let building_safe = self.agents[agent_index].current_action == Action::Build
                && !near_wolf
                && self.agents[agent_index].needs.worst() < 0.5;

            let died = self.agents[agent_index].needs.any_critical();

            let reward = compute_reward(
                &self.agents[agent_index].genome,
                &RewardContext {
                    needs: self.agents[agent_index].needs.clone(),
                    in_food_zone: in_food,
                    in_rest_zone: in_rest,
                    near_wolf,
                    fleeing_wolf: fleeing,
                    died,
                    at_home,
                    near_campfire,
                    nearby_agents: nearby_count,
                    building_safe,
                },
            );

            let wolf_dist_now = nalgebra_glm::distance(
                &self.agents[agent_index].position.xz(),
                &environment.wolf.position.xz(),
            );
            let new_state = AgentState {
                hunger_bucket: bucket(self.agents[agent_index].needs.hunger),
                energy_bucket: bucket(self.agents[agent_index].needs.energy),
                loneliness_bucket: bucket(self.agents[agent_index].needs.loneliness),
                threat_nearby: wolf_dist_now < environment.wolf.hunt_radius(is_night),
                at_home,
                is_night,
            };
            let new_state_index = new_state.encode();

            let current_action = self.agents[agent_index].current_action;
            let old_state = AgentState {
                hunger_bucket: bucket(self.agents[agent_index].needs.hunger),
                energy_bucket: bucket(self.agents[agent_index].needs.energy),
                loneliness_bucket: bucket(self.agents[agent_index].needs.loneliness),
                threat_nearby: near_wolf,
                at_home,
                is_night,
            };
            update_q_value(
                &mut self.agents[agent_index].q_table,
                old_state.encode(),
                current_action,
                reward,
                new_state_index,
            );

            self.agents[agent_index].q_table.decay_epsilon();

            if died {
                self.agents[agent_index].alive = false;
                let name = self.agents[agent_index].name.clone();
                let cause = if self.agents[agent_index].needs.hunger >= 1.0 {
                    "starved"
                } else if self.agents[agent_index].needs.energy >= 1.0 {
                    "collapsed from exhaustion"
                } else {
                    "died of loneliness"
                };
                self.log_event(format!("{name} {cause}"), [0.8, 0.2, 0.2]);
            }

            self.agents[agent_index].survival_time += tick_interval;
        }

        environment.wolf_tick(&mut self.agents, tick_interval);
        environment.tick_campfires(world, tick_interval);

        let previous_wolf_target: Option<usize> =
            self.agents.iter().position(|agent| agent.wolf_targeted);

        for agent in &mut self.agents {
            agent.wolf_targeted = false;
        }
        if let Some(target_index) = environment.wolf.hunt_target
            && target_index < self.agents.len()
        {
            self.agents[target_index].wolf_targeted = true;

            if previous_wolf_target != Some(target_index) {
                let name = self.agents[target_index].name.clone();
                self.log_event(format!("Wolf is hunting {name}!"), [1.0, 0.1, 0.0]);
            }
        }

        self.generation_timer += tick_interval;
    }

    pub fn should_end_generation(&self) -> bool {
        self.generation_timer >= self.generation_length || self.alive_count() == 0
    }

    pub fn update_movement(&mut self, delta_time: f32) {
        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }

            if agent.current_action == Action::Build {
                continue;
            }

            if let Some(target) = agent.target {
                let direction = target - agent.position;
                let distance = nalgebra_glm::length(&direction.xz());

                if distance > 0.3 {
                    let normalized = direction.normalize();
                    let step = agent.speed * delta_time;
                    agent.position += normalized * step.min(distance);
                    agent.position.y = 0.0;
                }
            }

            agent.position.x = agent.position.x.clamp(-19.5, 19.5);
            agent.position.z = agent.position.z.clamp(-19.5, 19.5);
        }
    }

    pub fn update_visuals(&mut self, world: &mut World, delta_time: f32) {
        for agent in &mut self.agents {
            sync_agent_body_transforms(world, agent);
            update_agent_material(world, agent);

            if agent.flash_timer > 0.0 {
                agent.flash_timer -= delta_time;
            }

            if !agent.alive {
                agent.death_timer -= delta_time;
                agent::apply_death_animation(world, agent);
            }
        }

        popup::update_popups(&mut self.popups, world, delta_time);

        if self.generation_flash_timer > 0.0 {
            self.generation_flash_timer -= delta_time;
        }
    }
}

fn compute_target(
    action: Action,
    agent: &Agent,
    environment: &Environment,
    rng: &mut impl Rng,
) -> Vec3 {
    match action {
        Action::GoToFood => {
            let center = environment.zones.nearest_food_center(&agent.position);
            let offset = Vec3::new(
                rng.random_range(-1.5..1.5f32),
                0.0,
                rng.random_range(-1.5..1.5f32),
            );
            center + offset
        }
        Action::GoToRest => {
            let offset = Vec3::new(
                rng.random_range(-2.0..2.0f32),
                0.0,
                rng.random_range(-2.0..2.0f32),
            );
            environment.zones.rest.center + offset
        }
        Action::GoHome => agent.home_position,
        Action::Build => agent.position,
        Action::Wander => {
            let range = 5.0 + agent.genome.wander_range * 15.0;
            Vec3::new(
                rng.random_range(-range..range),
                0.0,
                rng.random_range(-range..range),
            )
        }
        Action::Flee => {
            let wolf_dir = agent.position - environment.wolf.position;
            let flee_dir = if nalgebra_glm::length(&wolf_dir.xz()) > 0.1 {
                wolf_dir.normalize()
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            };
            agent.position + flee_dir * 10.0
        }
    }
}

fn register_agent_material(world: &mut World, name: &str) {
    material_registry_insert(
        &mut world.resources.material_registry,
        name.to_string(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            roughness: 0.7,
            unlit: false,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
}
