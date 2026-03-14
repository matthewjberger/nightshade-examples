mod creature;
mod genome;

use creature::Creature;
use genome::{
    BreedingOrigin, BreedingRecord, Genome, JOINT_COUNT, JOINT_LABELS, JointSource,
    select_and_breed,
};
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

use creature::{FINISH_DISTANCE, Obstacle};

const POPULATION_SIZE: usize = 50;
pub const GENERATION_DURATION: f32 = 24.0;
const LANE_LINE_COUNT: usize = 60;
const LANE_LINE_SPACING: f32 = 4.0;
const CREATURE_Z_SPACING: f32 = 1.5;

struct GeneticWalkers {
    creatures: Vec<Creature>,
    generation: u32,
    generation_time: f32,
    best_distances: Vec<f32>,
    avg_distances: Vec<f32>,
    speed_multiplier: f32,
    camera_entity: Option<Entity>,
    lines_entity: Option<Entity>,
    ground_entity: Option<Entity>,
    skip_requested: bool,
    sim_time: f32,
    initialized: bool,
    breeding_history: Vec<BreedingRecord>,
    rank_labels: [Option<Entity>; 3],
    finish_pole_left: Option<Entity>,
    finish_pole_right: Option<Entity>,
    finish_bar: Option<Entity>,
    finish_text: Option<Entity>,
    obstacles: Vec<Obstacle>,
    obstacle_entities: Vec<Entity>,
    wall_left_entity: Option<Entity>,
    wall_right_entity: Option<Entity>,
}

impl Default for GeneticWalkers {
    fn default() -> Self {
        Self {
            creatures: Vec::new(),
            generation: 1,
            generation_time: 0.0,
            best_distances: Vec::new(),
            avg_distances: Vec::new(),
            speed_multiplier: 1.0,
            camera_entity: None,
            lines_entity: None,
            ground_entity: None,
            skip_requested: false,
            sim_time: 0.0,
            initialized: false,
            breeding_history: Vec::new(),
            rank_labels: [None; 3],
            finish_pole_left: None,
            finish_pole_right: None,
            finish_bar: None,
            finish_text: None,
            obstacles: Vec::new(),
            obstacle_entities: Vec::new(),
            wall_left_entity: None,
            wall_right_entity: None,
        }
    }
}

impl GeneticWalkers {
    fn spawn_generation(&mut self, world: &mut World, genomes: Vec<Genome>) {
        for (creature_index, genome) in genomes.into_iter().enumerate() {
            let row = creature_index / 6;
            let col = creature_index % 6;
            let z_offset = (col as f32 - 2.5) * CREATURE_Z_SPACING + (row as f32) * 0.3;
            let creature = Creature::spawn(world, genome, creature_index, z_offset);
            self.creatures.push(creature);
        }
    }

    fn despawn_all_creatures(&mut self, world: &mut World) {
        let creatures = std::mem::take(&mut self.creatures);
        for creature in creatures {
            creature.despawn(world);
        }
    }

    fn advance_generation(&mut self, world: &mut World) {
        let genomes: Vec<Genome> = self
            .creatures
            .iter()
            .map(|creature| creature.genome.clone())
            .collect();
        let fitnesses: Vec<f32> = self
            .creatures
            .iter()
            .map(|creature| creature.fitness)
            .collect();

        let best = fitnesses.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let avg = if fitnesses.is_empty() {
            0.0
        } else {
            fitnesses.iter().sum::<f32>() / fitnesses.len() as f32
        };

        self.best_distances.push(best);
        self.avg_distances.push(avg);

        if self.best_distances.len() > 30 {
            self.best_distances.remove(0);
            self.avg_distances.remove(0);
        }

        let mut rng = rand::rng();
        let breed_results = select_and_breed(&genomes, &fitnesses, POPULATION_SIZE, &mut rng);

        let (new_genomes, records): (Vec<Genome>, Vec<BreedingRecord>) =
            breed_results.into_iter().unzip();

        self.breeding_history = records;
        self.despawn_all_creatures(world);
        self.spawn_generation(world, new_genomes);
        self.generation += 1;
        self.generation_time = 0.0;
        self.sim_time = 0.0;
        self.skip_requested = false;
    }

    fn spawn_ground_and_lines(&mut self, world: &mut World) {
        material_registry_insert(
            &mut world.resources.material_registry,
            "ground_material".to_string(),
            Material {
                base_color: [0.12, 0.12, 0.12, 1.0],
                roughness: 0.9,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get("ground_material")
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }

        let ground = nightshade::ecs::world::commands::spawn_mesh_at(
            world,
            "Cube",
            nalgebra_glm::vec3(0.0, -0.05, 0.0),
            nalgebra_glm::vec3(4000.0, 0.1, 100.0),
        );
        world
            .core
            .set_material_ref(ground, MaterialRef::new("ground_material"));
        self.ground_entity = Some(ground);

        let lines_entity = world.spawn_entities(LINES, 1)[0];
        self.lines_entity = Some(lines_entity);
    }

    fn update_lane_lines(&self, world: &mut World) {
        let lines_entity = match self.lines_entity {
            Some(entity) => entity,
            None => return,
        };

        let camera_x = if let Some(camera) = self.camera_entity {
            world
                .core
                .get_local_transform(camera)
                .map(|transform| transform.translation.x)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let line_color = Vec4::new(0.4, 0.4, 0.4, 1.0);
        let half_width = 40.0;
        let total_span = LANE_LINE_COUNT as f32 * LANE_LINE_SPACING;

        if let Some(lines) = world.core.get_lines_mut(lines_entity) {
            lines.clear();
            for line_index in 0..LANE_LINE_COUNT {
                let base_x = (line_index as f32 - LANE_LINE_COUNT as f32 * 0.5) * LANE_LINE_SPACING;
                let mut x_position =
                    base_x + (camera_x / LANE_LINE_SPACING).floor() * LANE_LINE_SPACING;

                let relative = x_position - camera_x;
                if relative < -total_span * 0.5 {
                    x_position += total_span;
                } else if relative > total_span * 0.5 {
                    x_position -= total_span;
                }

                lines.push(Line {
                    start: nalgebra_glm::vec3(x_position, 0.01, -half_width),
                    end: nalgebra_glm::vec3(x_position, 0.01, half_width),
                    color: line_color,
                });
            }
        }
    }

    fn update_creature_colors(&mut self, world: &mut World) {
        for creature in &mut self.creatures {
            creature.set_greyed(world, creature.fallen && !creature.finished);
        }
    }

    fn spawn_rank_labels(&mut self, world: &mut World) {
        let labels = ["1st", "2nd", "3rd"];
        let colors = [
            Vec4::new(1.0, 0.84, 0.0, 1.0),
            Vec4::new(0.75, 0.75, 0.78, 1.0),
            Vec4::new(0.80, 0.50, 0.20, 1.0),
        ];

        for rank in 0..3 {
            let entity = spawn_3d_billboard_text_with_properties(
                world,
                labels[rank],
                nalgebra_glm::vec3(0.0, 10.0, 0.0),
                TextProperties {
                    font_size: 64.0,
                    color: colors[rank],
                    alignment: TextAlignment::Center,
                    vertical_alignment: VerticalAlignment::Bottom,
                    outline_width: 0.04,
                    outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                },
            );
            self.rank_labels[rank] = Some(entity);
        }
    }

    fn spawn_finish_line(&mut self, world: &mut World) {
        material_registry_insert(
            &mut world.resources.material_registry,
            "finish_pole_material".to_string(),
            Material {
                base_color: [0.9, 0.15, 0.1, 1.0],
                roughness: 0.4,
                metallic: 0.1,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get("finish_pole_material")
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }

        material_registry_insert(
            &mut world.resources.material_registry,
            "finish_bar_material".to_string(),
            Material {
                base_color: [1.0, 1.0, 1.0, 1.0],
                roughness: 0.3,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get("finish_bar_material")
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }

        let pole_height = 4.0;
        let pole_z_spread = 6.0;

        let left_pole = nightshade::ecs::world::commands::spawn_mesh_at(
            world,
            "Cube",
            nalgebra_glm::vec3(FINISH_DISTANCE, pole_height * 0.5, pole_z_spread),
            nalgebra_glm::vec3(0.12, pole_height, 0.12),
        );
        world
            .core
            .set_material_ref(left_pole, MaterialRef::new("finish_pole_material"));
        self.finish_pole_left = Some(left_pole);

        let right_pole = nightshade::ecs::world::commands::spawn_mesh_at(
            world,
            "Cube",
            nalgebra_glm::vec3(FINISH_DISTANCE, pole_height * 0.5, -pole_z_spread),
            nalgebra_glm::vec3(0.12, pole_height, 0.12),
        );
        world
            .core
            .set_material_ref(right_pole, MaterialRef::new("finish_pole_material"));
        self.finish_pole_right = Some(right_pole);

        let bar = nightshade::ecs::world::commands::spawn_mesh_at(
            world,
            "Cube",
            nalgebra_glm::vec3(FINISH_DISTANCE, pole_height, 0.0),
            nalgebra_glm::vec3(0.15, 0.15, pole_z_spread * 2.0),
        );
        world
            .core
            .set_material_ref(bar, MaterialRef::new("finish_bar_material"));
        self.finish_bar = Some(bar);

        let text_entity = spawn_3d_billboard_text_with_properties(
            world,
            "FINISH",
            nalgebra_glm::vec3(FINISH_DISTANCE, pole_height + 0.8, 0.0),
            TextProperties {
                font_size: 48.0,
                color: Vec4::new(1.0, 0.2, 0.1, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                ..Default::default()
            },
        );
        self.finish_text = Some(text_entity);
    }

    fn spawn_obstacles_and_walls(&mut self, world: &mut World) {
        material_registry_insert(
            &mut world.resources.material_registry,
            "obstacle_material".to_string(),
            Material {
                base_color: [0.45, 0.18, 0.12, 1.0],
                roughness: 0.8,
                metallic: 0.1,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get("obstacle_material")
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }

        material_registry_insert(
            &mut world.resources.material_registry,
            "wall_material".to_string(),
            Material {
                base_color: [0.3, 0.3, 0.32, 1.0],
                roughness: 0.9,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get("wall_material")
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }

        self.obstacles = creature::generate_obstacles();

        for obstacle in &self.obstacles {
            let center_x = (obstacle.x_start + obstacle.x_end) * 0.5;
            let width = obstacle.x_end - obstacle.x_start;
            let entity = nightshade::ecs::world::commands::spawn_mesh_at(
                world,
                "Cube",
                nalgebra_glm::vec3(center_x, obstacle.height * 0.5, 0.0),
                nalgebra_glm::vec3(width, obstacle.height, 16.0),
            );
            world
                .core
                .set_material_ref(entity, MaterialRef::new("obstacle_material"));
            self.obstacle_entities.push(entity);
        }

        let wall_height = 0.5;
        let wall_length = 200.0;
        let wall_z = 7.0;

        let left_wall = nightshade::ecs::world::commands::spawn_mesh_at(
            world,
            "Cube",
            nalgebra_glm::vec3(wall_length * 0.5 - 10.0, wall_height * 0.5, wall_z),
            nalgebra_glm::vec3(wall_length, wall_height, 0.15),
        );
        world
            .core
            .set_material_ref(left_wall, MaterialRef::new("wall_material"));
        self.wall_left_entity = Some(left_wall);

        let right_wall = nightshade::ecs::world::commands::spawn_mesh_at(
            world,
            "Cube",
            nalgebra_glm::vec3(wall_length * 0.5 - 10.0, wall_height * 0.5, -wall_z),
            nalgebra_glm::vec3(wall_length, wall_height, 0.15),
        );
        world
            .core
            .set_material_ref(right_wall, MaterialRef::new("wall_material"));
        self.wall_right_entity = Some(right_wall);
    }

    fn update_rank_labels(&self, world: &mut World) {
        let mut ranked: Vec<(usize, f32)> = self
            .creatures
            .iter()
            .enumerate()
            .filter(|(_, creature)| !creature.fallen)
            .map(|(index, creature)| (index, creature.fitness))
            .collect();

        if ranked.is_empty() {
            ranked = self
                .creatures
                .iter()
                .enumerate()
                .map(|(index, creature)| (index, creature.fitness))
                .collect();
        }

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (rank, label_entity) in self.rank_labels.iter().enumerate() {
            if let Some(entity) = label_entity
                && rank < ranked.len()
            {
                let creature_index = ranked[rank].0;
                let creature_pos = self.creatures[creature_index].position;
                let label_height = 2.5 - rank as f32 * 0.3;
                world.assign_local_transform(
                    *entity,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(
                            creature_pos.x,
                            creature_pos.y + label_height,
                            creature_pos.z,
                        ),
                        rotation: Quat::identity(),
                        scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
                    },
                );
                mark_local_transform_dirty(world, *entity);
            }
        }
    }
}

impl State for GeneticWalkers {
    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Sunset;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.ambient_light = [0.35, 0.35, 0.4, 1.0];

        let camera = spawn_pan_orbit_camera(
            world,
            nalgebra_glm::vec3(0.0, 0.5, 0.0),
            15.0,
            0.0,
            0.5,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);
        self.camera_entity = Some(camera);

        nightshade::ecs::world::commands::spawn_sun(world);

        self.spawn_ground_and_lines(world);
        self.spawn_finish_line(world);
        self.spawn_obstacles_and_walls(world);
        self.spawn_rank_labels(world);

        let mut rng = rand::rng();
        let genomes: Vec<Genome> = (0..POPULATION_SIZE)
            .map(|_| Genome::random(&mut rng))
            .collect();
        self.spawn_generation(world, genomes);
        self.initialized = true;
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.initialized {
            return;
        }

        let delta_time = world.resources.window.timing.delta_time;
        let scaled_dt = delta_time * self.speed_multiplier;

        self.generation_time += scaled_dt;
        self.sim_time += scaled_dt;

        let all_done = self.creatures.iter().all(|creature| creature.is_done());
        let time_up = self.generation_time >= GENERATION_DURATION;

        if all_done || time_up || self.skip_requested {
            self.advance_generation(world);
            return;
        }

        let sim_time = self.sim_time;
        let obstacles = &self.obstacles;
        for creature in &mut self.creatures {
            creature.update(world, sim_time, scaled_dt, obstacles);
        }

        self.update_creature_colors(world);
        self.update_rank_labels(world);
        pan_orbit_camera_system(world);
        nightshade::ecs::text::systems::sync_text_meshes_system(world);
        self.update_lane_lines(world);
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Evolution")
            .default_pos(egui::pos2(10.0, 10.0))
            .default_width(280.0)
            .resizable(true)
            .show(ui_context, |ui| {
                ui.heading("Genetic Walkers");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Generation:");
                    ui.strong(format!("{}", self.generation));
                });

                let alive_count = self
                    .creatures
                    .iter()
                    .filter(|creature| !creature.is_done())
                    .count();
                let finished_count = self
                    .creatures
                    .iter()
                    .filter(|creature| creature.finished)
                    .count();
                let fallen_count = self
                    .creatures
                    .iter()
                    .filter(|creature| creature.fallen)
                    .count();

                let best_time = self
                    .creatures
                    .iter()
                    .filter_map(|creature| creature.finish_time)
                    .fold(f32::INFINITY, f32::min);

                let leader_distance = self
                    .creatures
                    .iter()
                    .filter(|creature| !creature.fallen)
                    .map(|creature| creature.position.x - creature.start_x)
                    .fold(0.0_f32, f32::max);

                ui.horizontal(|ui| {
                    ui.label("Running:");
                    ui.strong(format!("{}", alive_count));
                    ui.label("Finished:");
                    ui.strong(format!("{}", finished_count));
                    ui.label("Fallen:");
                    ui.strong(format!("{}", fallen_count));
                });

                if best_time.is_finite() {
                    ui.horizontal(|ui| {
                        ui.label("Best time:");
                        ui.strong(format!("{:.2}s", best_time));
                    });
                }

                ui.horizontal(|ui| {
                    ui.label("Leader distance:");
                    ui.strong(format!("{:.1} / {:.0}", leader_distance, FINISH_DISTANCE));
                });
                ui.horizontal(|ui| {
                    ui.label("Time:");
                    ui.strong(format!(
                        "{:.1}s / {:.0}s",
                        self.generation_time, GENERATION_DURATION
                    ));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Speed:");
                    ui.add(
                        egui::Slider::new(&mut self.speed_multiplier, 0.25..=8.0).logarithmic(true),
                    );
                });

                if ui.button("Skip to Next Generation").clicked() {
                    self.skip_requested = true;
                }

                ui.separator();
                ui.label("Best Fitness over Generations");

                if !self.best_distances.is_empty() {
                    draw_fitness_graph(ui, &self.best_distances, &self.avg_distances);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 140, 15), "\u{25A0}");
                    ui.label("Best");
                    ui.colored_label(egui::Color32::from_rgb(100, 180, 255), "\u{25A0}");
                    ui.label("Average");
                });

                if !self.breeding_history.is_empty() {
                    ui.separator();
                    ui.collapsing("Breeding", |ui| {
                        draw_breeding_grid(ui, &self.breeding_history);
                    });
                }
            });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::Escape, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
        if matches!((key_code, key_state), (KeyCode::Space, KeyState::Pressed)) {
            self.skip_requested = true;
        }
    }
}

fn draw_fitness_graph(ui: &mut egui::Ui, best_history: &[f32], avg_history: &[f32]) {
    let desired_size = egui::vec2(ui.available_width().min(280.0), 120.0);
    let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, 4.0, egui::Color32::from_gray(25));
    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
        egui::StrokeKind::Outside,
    );

    if best_history.is_empty() {
        return;
    }

    let max_val = best_history
        .iter()
        .chain(avg_history.iter())
        .copied()
        .fold(0.1_f32, f32::max);
    let min_val = avg_history
        .iter()
        .chain(best_history.iter())
        .copied()
        .fold(f32::INFINITY, f32::min)
        .min(0.0);
    let range = (max_val - min_val).max(0.1);

    let padding = 4.0;
    let plot_rect = rect.shrink(padding);

    let point_count = best_history.len();
    if point_count < 2 {
        return;
    }

    let to_screen = |index: usize, value: f32| -> egui::Pos2 {
        let x_fraction = index as f32 / (point_count - 1) as f32;
        let y_fraction = 1.0 - (value - min_val) / range;
        egui::pos2(
            plot_rect.left() + x_fraction * plot_rect.width(),
            plot_rect.top() + y_fraction * plot_rect.height(),
        )
    };

    let avg_points: Vec<egui::Pos2> = avg_history
        .iter()
        .enumerate()
        .map(|(index, &value)| to_screen(index, value))
        .collect();
    painter.add(egui::Shape::line(
        avg_points,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 180, 255)),
    ));

    let best_points: Vec<egui::Pos2> = best_history
        .iter()
        .enumerate()
        .map(|(index, &value)| to_screen(index, value))
        .collect();
    painter.add(egui::Shape::line(
        best_points,
        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 140, 15)),
    ));

    let label_color = egui::Color32::from_gray(150);
    painter.text(
        egui::pos2(plot_rect.left() + 2.0, plot_rect.top()),
        egui::Align2::LEFT_TOP,
        format!("{:.0}", max_val),
        egui::FontId::proportional(9.0),
        label_color,
    );
    painter.text(
        egui::pos2(plot_rect.left() + 2.0, plot_rect.bottom() - 10.0),
        egui::Align2::LEFT_TOP,
        format!("{:.0}", min_val),
        egui::FontId::proportional(9.0),
        label_color,
    );
}

fn draw_breeding_grid(ui: &mut egui::Ui, history: &[BreedingRecord]) {
    let elite_count = history
        .iter()
        .filter(|record| matches!(record.origin, BreedingOrigin::Elite(_)))
        .count();
    let crossover_count = history.len() - elite_count;
    let total_mutated: usize = history
        .iter()
        .map(|record| {
            record
                .mutated_joints
                .iter()
                .filter(|&&mutated| mutated)
                .count()
        })
        .sum();

    ui.label(format!(
        "{} elites, {} children, {} mutations",
        elite_count, crossover_count, total_mutated
    ));

    ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);

    let cell_size = 8.0;
    let label_width = 32.0;
    let grid_width = label_width + JOINT_COUNT as f32 * (cell_size + 1.0);
    let header_height = 12.0;
    let grid_height = header_height + history.len() as f32 * (cell_size + 1.0);

    let desired_size = egui::vec2(grid_width, grid_height);
    let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
    let origin = response.rect.min;

    let header_font = egui::FontId::proportional(8.0);
    let label_font = egui::FontId::proportional(7.0);

    for (joint_index, label) in JOINT_LABELS.iter().enumerate() {
        let x = origin.x + label_width + joint_index as f32 * (cell_size + 1.0);
        painter.text(
            egui::pos2(x + cell_size * 0.5, origin.y),
            egui::Align2::CENTER_TOP,
            label,
            header_font.clone(),
            egui::Color32::from_gray(160),
        );
    }

    let color_parent_a = egui::Color32::from_rgb(60, 120, 220);
    let color_parent_b = egui::Color32::from_rgb(220, 60, 60);
    let color_elite = egui::Color32::from_rgb(210, 170, 30);
    let color_mutation = egui::Color32::from_rgb(255, 255, 255);

    for (row, record) in history.iter().enumerate() {
        let y = origin.y + header_height + row as f32 * (cell_size + 1.0);

        let row_label = match &record.origin {
            BreedingOrigin::Elite(rank) => format!("E{}", rank + 1),
            BreedingOrigin::Crossover { .. } => format!("C{}", row - elite_count + 1),
        };
        painter.text(
            egui::pos2(origin.x + label_width - 2.0, y + cell_size * 0.5),
            egui::Align2::RIGHT_CENTER,
            &row_label,
            label_font.clone(),
            egui::Color32::from_gray(140),
        );

        for joint_index in 0..JOINT_COUNT {
            let x = origin.x + label_width + joint_index as f32 * (cell_size + 1.0);
            let cell_rect =
                egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_size, cell_size));

            let base_color = match &record.origin {
                BreedingOrigin::Elite(_) => color_elite,
                BreedingOrigin::Crossover { .. } => match record.parent_source[joint_index] {
                    JointSource::ParentA => color_parent_a,
                    JointSource::ParentB => color_parent_b,
                },
            };

            painter.rect_filled(cell_rect, 1.0, base_color);

            if record.mutated_joints[joint_index] {
                let center = cell_rect.center();
                painter.circle_filled(center, 2.0, color_mutation);
            }
        }
    }

    if let Some(hover_pos) = response.hover_pos() {
        let row_f = (hover_pos.y - origin.y - header_height) / (cell_size + 1.0);
        let row = row_f as usize;
        if row_f >= 0.0 && row < history.len() {
            let record = &history[row];
            let tooltip_text = match &record.origin {
                BreedingOrigin::Elite(rank) => format!("Elite #{} (preserved)", rank + 1),
                BreedingOrigin::Crossover {
                    parent_a_fitness,
                    parent_b_fitness,
                } => format!(
                    "Crossover: A={:.1} B={:.1}",
                    parent_a_fitness, parent_b_fitness
                ),
            };
            response.clone().on_hover_text(tooltip_text);
        }
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let swatch = |ui: &mut egui::Ui, color: egui::Color32, text: &str| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 1.0, color);
            ui.label(text);
        };
        swatch(ui, color_elite, "Elite");
        swatch(ui, color_parent_a, "A");
        swatch(ui, color_parent_b, "B");
        let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 2.5, color_mutation);
        ui.label("Mut");
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nightshade::run::launch(GeneticWalkers::default())?;
    Ok(())
}
