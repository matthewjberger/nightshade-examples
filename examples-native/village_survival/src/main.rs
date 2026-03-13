mod agent;
mod environment;
mod genome;
mod popup;
mod qlearning;
mod simulation;
mod ui;

use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::text::systems::sync_text_meshes_system;
use nightshade::prelude::*;

use environment::Environment;
use simulation::Simulation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(VillageSurvival::default())
}

struct VillageSurvival {
    simulation: Simulation,
    environment: Option<Environment>,
    sun_entity: Option<Entity>,
    current_hour: f32,
    last_pick_mouse_pos: (u32, u32),
}

impl Default for VillageSurvival {
    fn default() -> Self {
        Self {
            simulation: Simulation::new(),
            environment: None,
            sun_entity: None,
            current_hour: 8.0,
            last_pick_mouse_pos: (0, 0),
        }
    }
}

impl VillageSurvival {
    fn get_sun_direction(hour: f32) -> Vec3 {
        if !(6.0..=18.0).contains(&hour) {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            let sun_angle = (hour - 6.0) / 12.0 * std::f32::consts::PI;
            nalgebra_glm::normalize(&Vec3::new(-sun_angle.cos(), sun_angle.sin(), -0.3))
        }
    }

    fn update_sun_for_hour(&self, world: &mut World) {
        let sun = match self.sun_entity {
            Some(entity) => entity,
            None => return,
        };

        let sun_dir = Self::get_sun_direction(self.current_hour);
        let is_night = !(6.0..=18.0).contains(&self.current_hour);

        let sun_intensity = if is_night {
            0.0
        } else {
            let elevation = sun_dir.y.max(0.0);
            3.5 * elevation.sqrt()
        };

        let warm = Vec3::new(1.0, 0.7, 0.4);
        let white = Vec3::new(1.0, 0.95, 0.8);
        let sun_color = if is_night {
            Vec3::new(0.0, 0.0, 0.0)
        } else if self.current_hour < 7.5 {
            let interpolation = ((self.current_hour - 6.0) / 1.5).clamp(0.0, 1.0);
            nalgebra_glm::lerp(&warm, &white, interpolation)
        } else if self.current_hour > 16.5 {
            let interpolation = ((18.0 - self.current_hour) / 1.5).clamp(0.0, 1.0);
            nalgebra_glm::lerp(&warm, &white, interpolation)
        } else {
            white
        };

        if let Some(light) = world.core.get_light_mut(sun) {
            light.intensity = sun_intensity;
            light.color = sun_color;
            light.cast_shadows = !is_night;
        }

        let sun_position = sun_dir * 100.0;
        if let Some(transform) = world.core.get_local_transform_mut(sun) {
            transform.translation = sun_position;
            let direction = -sun_dir;
            let up = Vec3::y();
            let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&direction, &up));
            if right.norm() > 0.001 {
                let corrected_up = nalgebra_glm::cross(&right, &direction);
                transform.rotation =
                    nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[
                        right,
                        corrected_up,
                        -direction,
                    ]));
            }
        }
        mark_local_transform_dirty(world, sun);
    }

    fn update_environment_for_hour(&self, world: &mut World) {
        let hour = self.current_hour;
        let is_night = !(6.0..=18.0).contains(&hour);

        let ambient_color = if is_night {
            [0.05, 0.05, 0.1, 1.0]
        } else if !(7.5..=16.5).contains(&hour) {
            let interpolation = if hour < 7.5 {
                (hour - 6.0) / 1.5
            } else {
                (18.0 - hour) / 1.5
            };
            [
                0.05 + 0.20 * interpolation,
                0.05 + 0.17 * interpolation,
                0.10 + 0.10 * interpolation,
                1.0,
            ]
        } else {
            [0.25, 0.22, 0.20, 1.0]
        };
        world.resources.graphics.ambient_light = ambient_color;
    }

    fn handle_gpu_picking(&mut self, world: &mut World) {
        if let Some(result) = world.resources.gpu_picking.take_result() {
            if let Some(entity_id) = result.entity_id {
                let mut found_agent = None;
                for (agent_index, agent) in self.simulation.agents.iter().enumerate() {
                    for entity in agent.body.all_entities() {
                        if entity.id == entity_id {
                            found_agent = Some(agent_index);
                            break;
                        }
                    }
                    if found_agent.is_some() {
                        break;
                    }
                }
                self.simulation.selected_agent = found_agent;
            } else {
                self.simulation.selected_agent = None;
            }
        }

        let mouse = &world.resources.input.mouse;
        if mouse.state.contains(MouseState::LEFT_JUST_PRESSED)
            && !world.resources.user_interface.hud_wants_pointer
        {
            let mouse_pos = mouse.position;
            let current = (mouse_pos.x as u32, mouse_pos.y as u32);
            if current != self.last_pick_mouse_pos {
                world
                    .resources
                    .gpu_picking
                    .request_pick(current.0, current.1);
                self.last_pick_mouse_pos = current;
            }
        }

        if let Some(selected_index) = self.simulation.selected_agent {
            if selected_index < self.simulation.agents.len() {
                let torso = self.simulation.agents[selected_index].body.torso;
                world.resources.graphics.bounding_volume_selected_entity = Some(torso);
            }
        } else {
            world.resources.graphics.bounding_volume_selected_entity = None;
        }
    }
}

impl State for VillageSurvival {
    fn title(&self) -> &str {
        "Village Survival"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::DayNight;
        world.resources.graphics.day_night_hour = self.current_hour;
        capture_procedural_atmosphere_ibl(world, Atmosphere::DayNight, self.current_hour);
        capture_ibl_snapshots(
            world,
            Atmosphere::DayNight,
            vec![0.0, 6.0, 8.0, 12.0, 17.0, 18.5, 20.0],
        );
        world.resources.graphics.show_grid = false;
        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.005;
        world.resources.graphics.ambient_light = [0.25, 0.22, 0.20, 1.0];
        world.resources.graphics.selection_outline_enabled = true;
        world.resources.graphics.selection_outline_color = [1.0, 0.45, 0.0, 1.0];

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            25.0,
            0.0,
            1.2,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);

        let sun = spawn_sun(world);
        if let Some(light) = world.core.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 3.5;
            light.shadow_bias = 0.008;
        }
        self.sun_entity = Some(sun);
        self.update_sun_for_hour(world);

        let mut rng = rand::rng();
        let environment = Environment::initialize(world, &mut rng);
        self.simulation.start_generation(world, &environment, None);
        self.environment = Some(environment);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);
        sync_text_meshes_system(world);

        self.handle_gpu_picking(world);

        if let Some(environment) = &self.environment {
            self.current_hour = environment.day_night.to_hour();
        }
        world.resources.graphics.day_night_hour = self.current_hour;
        self.update_sun_for_hour(world);
        self.update_environment_for_hour(world);

        if let Some(environment) = &mut self.environment {
            if !self.simulation.paused {
                let delta_time = world.resources.window.timing.delta_time;
                let tick_interval =
                    1.0 / (self.simulation.ticks_per_second * self.simulation.speed_multiplier);

                self.simulation.tick_accumulator += delta_time;

                while self.simulation.tick_accumulator >= tick_interval {
                    self.simulation.tick_accumulator -= tick_interval;
                    self.simulation.tick(world, environment, tick_interval);

                    if self.simulation.should_end_generation() {
                        self.simulation.end_generation(world, environment);
                        break;
                    }
                }

                self.simulation.update_movement(delta_time);
            }

            let delta_time = world.resources.window.timing.delta_time;
            self.simulation.update_visuals(world, delta_time);
            environment.sync_wolf_transform(world);
        }

        if self.simulation.generation_flash_timer > 0.0 {
            let flash_intensity = self.simulation.generation_flash_timer / 0.5;
            world.resources.graphics.bloom_intensity = 0.005 + flash_intensity * 0.1;
        } else {
            world.resources.graphics.bloom_intensity = 0.005;
        }
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        let is_night = self
            .environment
            .as_ref()
            .is_some_and(|env| env.day_night.is_night());

        let campfire_count = self
            .environment
            .as_ref()
            .map(|env| env.campfires.len())
            .unwrap_or(0);

        ui::draw_ui(&mut self.simulation, is_night, campfire_count, ui_context);
    }
}
