use nightshade::prelude::*;
use rand::Rng;
use std::io::Write;
use std::time::Instant;

const ENTITY_COUNT: usize = 200_000;
const COMPUTATION_ITERATIONS: usize = 20;
const WARMUP_ITERATIONS: usize = 3;
const BENCHMARK_ITERATIONS: usize = 30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(StressWorld::default())?;
    Ok(())
}

::freecs::ecs! {
    StressWorld {
        transform: Transform => TRANSFORM,
        velocity: Velocity => VELOCITY,
        physics_state: PhysicsState => PHYSICS_STATE,
        tag_a: TagA => TAG_A,
        tag_b: TagB => TAG_B,
        tag_c: TagC => TAG_C,
        tag_d: TagD => TAG_D,
        tag_e: TagE => TAG_E,
        tag_f: TagF => TAG_F,
        tag_g: TagG => TAG_G,
        tag_h: TagH => TAG_H,
    }
    StressResources {
        sequential_time_ms: f64,
        parallel_time_ms: f64,
        speedup_ratio: f64,
        entity_count: usize,
        last_benchmark_frame: u64,
        frame_counter: u64,
        benchmark_interval: u64,
        use_parallel: bool,
        sequential_samples: Vec<f64>,
        parallel_samples: Vec<f64>,
        benchmark_complete: bool,
        benchmark_started: bool,
        warmup_done: bool,
        current_iteration: usize,
        report_path: String,
        status_message: String,
    }
}

#[derive(Debug, Clone, Default)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub matrix: [[f32; 4]; 4],
}

#[derive(Debug, Clone, Default)]
pub struct Velocity {
    pub linear: [f32; 3],
    pub angular: [f32; 3],
}

#[derive(Debug, Clone, Default)]
pub struct PhysicsState {
    pub acceleration: [f32; 3],
    pub force: [f32; 3],
    pub mass: f32,
    pub drag: f32,
    pub energy: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TagA;
#[derive(Debug, Clone, Copy, Default)]
pub struct TagB;
#[derive(Debug, Clone, Copy, Default)]
pub struct TagC;
#[derive(Debug, Clone, Copy, Default)]
pub struct TagD;
#[derive(Debug, Clone, Copy, Default)]
pub struct TagE;
#[derive(Debug, Clone, Copy, Default)]
pub struct TagF;
#[derive(Debug, Clone, Copy, Default)]
pub struct TagG;
#[derive(Debug, Clone, Copy, Default)]
pub struct TagH;

impl State for StressWorld {
    fn title(&self) -> &str {
        "Parallel ECS Stress Test"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.user_interface.enabled = true;

        self.resources.benchmark_interval = 30;
        self.resources.use_parallel = true;
        self.resources.sequential_samples = Vec::with_capacity(BENCHMARK_ITERATIONS);
        self.resources.parallel_samples = Vec::with_capacity(BENCHMARK_ITERATIONS);
        self.resources.status_message = "Starting benchmark automatically...".to_string();
        self.resources.benchmark_started = true;

        spawn_stress_entities(self, ENTITY_COUNT);
        self.resources.entity_count = ENTITY_COUNT;

        let camera = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | CAMERA,
            1,
        )[0];
        world.set_name(camera, Name("Camera".to_string()));
        world.set_local_transform(
            camera,
            LocalTransform {
                translation: nalgebra_glm::vec3(0.0, 5.0, 10.0),
                ..Default::default()
            },
        );
        world.set_camera(
            camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 60.0_f32.to_radians(),
                    z_far: Some(1000.0),
                    z_near: 0.1,
                }),
                smoothing: None,
            },
        );
        world.resources.active_camera = Some(camera);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        self.resources.frame_counter += 1;

        if self.resources.benchmark_started && !self.resources.benchmark_complete {
            run_benchmark_iteration(self);
        }

        let delta = world.resources.window.timing.delta_time;
        if self.resources.use_parallel {
            run_physics_parallel(self, delta);
        } else {
            run_physics_sequential(self, delta);
        }
    }

    fn ui(&mut self, _world: &mut World, ctx: &egui::Context) {
        egui::Window::new("Parallel ECS Stress Test")
            .default_pos([10.0, 10.0])
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("freecs Parallelism Benchmark");
                ui.separator();

                ui.label(format!(
                    "Entities: {}",
                    format_with_commas(self.resources.entity_count)
                ));
                ui.label(format!(
                    "Computation iterations per entity: {}",
                    COMPUTATION_ITERATIONS
                ));
                ui.label(format!(
                    "CPU cores available: {}",
                    rayon::current_num_threads()
                ));
                ui.label(format!("Warmup iterations: {}", WARMUP_ITERATIONS));
                ui.label(format!("Benchmark iterations: {}", BENCHMARK_ITERATIONS));

                ui.separator();

                ui.colored_label(egui::Color32::LIGHT_BLUE, &self.resources.status_message);

                ui.separator();

                if self.resources.benchmark_started && !self.resources.benchmark_complete {
                    let progress = self.resources.current_iteration as f32
                        / (WARMUP_ITERATIONS + BENCHMARK_ITERATIONS) as f32;
                    ui.add(egui::ProgressBar::new(progress).show_percentage());
                    ui.label(format!(
                        "Iteration {}/{}",
                        self.resources.current_iteration,
                        WARMUP_ITERATIONS + BENCHMARK_ITERATIONS
                    ));
                }

                if !self.resources.benchmark_started && ui.button("Start Benchmark").clicked() {
                    self.resources.benchmark_started = true;
                    self.resources.benchmark_complete = false;
                    self.resources.warmup_done = false;
                    self.resources.current_iteration = 0;
                    self.resources.sequential_samples.clear();
                    self.resources.parallel_samples.clear();
                    self.resources.status_message = "Running warmup...".to_string();
                }

                if self.resources.benchmark_complete {
                    ui.separator();
                    ui.heading("Results Summary");

                    let seq_stats = calculate_stats(&self.resources.sequential_samples);
                    let par_stats = calculate_stats(&self.resources.parallel_samples);

                    ui.horizontal(|ui| {
                        ui.label("Sequential avg:");
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            format!("{:.2} ms", seq_stats.mean),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Parallel avg:");
                        ui.colored_label(egui::Color32::GREEN, format!("{:.2} ms", par_stats.mean));
                    });

                    let speedup = seq_stats.mean / par_stats.mean;
                    ui.horizontal(|ui| {
                        ui.label("Average speedup:");
                        ui.colored_label(egui::Color32::GREEN, format!("{:.2}x", speedup));
                    });

                    ui.separator();

                    if !self.resources.report_path.is_empty() {
                        ui.label(format!("Report saved to: {}", self.resources.report_path));
                    }

                    if ui.button("Run Again").clicked() {
                        self.resources.benchmark_started = true;
                        self.resources.benchmark_complete = false;
                        self.resources.warmup_done = false;
                        self.resources.current_iteration = 0;
                        self.resources.sequential_samples.clear();
                        self.resources.parallel_samples.clear();
                        self.resources.status_message = "Running warmup...".to_string();
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Live mode:");
                    if self.resources.use_parallel {
                        ui.colored_label(egui::Color32::GREEN, "Parallel");
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "Sequential");
                    }
                });

                ui.checkbox(&mut self.resources.use_parallel, "Use parallel iteration");
            });
    }
}

const NUM_ARCHETYPES: usize = 16;

fn spawn_stress_entities(stress_world: &mut StressWorld, count: usize) {
    let mut rng = rand::rng();
    let entities_per_archetype = count / NUM_ARCHETYPES;

    let archetype_masks = [
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_A,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_B,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_C,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_D,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_E,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_F,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_G,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_H,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_A | TAG_B,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_C | TAG_D,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_E | TAG_F,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_G | TAG_H,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_A | TAG_C,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_B | TAG_D,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_E | TAG_G,
        TRANSFORM | VELOCITY | PHYSICS_STATE | TAG_F | TAG_H,
    ];

    for mask in archetype_masks {
        stress_world.spawn_batch(mask, entities_per_archetype, |table, index| {
            table.transform[index] = Transform {
                position: [
                    rng.random_range(-100.0..100.0),
                    rng.random_range(-100.0..100.0),
                    rng.random_range(-100.0..100.0),
                ],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            };

            table.velocity[index] = Velocity {
                linear: [
                    rng.random_range(-10.0..10.0),
                    rng.random_range(-10.0..10.0),
                    rng.random_range(-10.0..10.0),
                ],
                angular: [
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                ],
            };

            table.physics_state[index] = PhysicsState {
                acceleration: [0.0, -9.81, 0.0],
                force: [0.0, 0.0, 0.0],
                mass: rng.random_range(0.1..10.0),
                drag: rng.random_range(0.01..0.1),
                energy: 0.0,
            };

            if mask & TAG_A != 0 {
                table.tag_a[index] = TagA;
            }
            if mask & TAG_B != 0 {
                table.tag_b[index] = TagB;
            }
            if mask & TAG_C != 0 {
                table.tag_c[index] = TagC;
            }
            if mask & TAG_D != 0 {
                table.tag_d[index] = TagD;
            }
            if mask & TAG_E != 0 {
                table.tag_e[index] = TagE;
            }
            if mask & TAG_F != 0 {
                table.tag_f[index] = TagF;
            }
            if mask & TAG_G != 0 {
                table.tag_g[index] = TagG;
            }
            if mask & TAG_H != 0 {
                table.tag_h[index] = TagH;
            }
        });
    }
}

fn heavy_computation(
    transform: &mut Transform,
    velocity: &mut Velocity,
    physics: &mut PhysicsState,
    delta: f32,
) {
    for _ in 0..COMPUTATION_ITERATIONS {
        physics.force[0] = -physics.drag * velocity.linear[0] * velocity.linear[0].abs();
        physics.force[1] =
            physics.mass * physics.acceleration[1] - physics.drag * velocity.linear[1];
        physics.force[2] = -physics.drag * velocity.linear[2] * velocity.linear[2].abs();

        let inv_mass = 1.0 / physics.mass;
        velocity.linear[0] += physics.force[0] * inv_mass * delta;
        velocity.linear[1] += physics.force[1] * inv_mass * delta;
        velocity.linear[2] += physics.force[2] * inv_mass * delta;

        transform.position[0] += velocity.linear[0] * delta;
        transform.position[1] += velocity.linear[1] * delta;
        transform.position[2] += velocity.linear[2] * delta;

        let (sin_x, cos_x) = (velocity.angular[0] * delta).sin_cos();
        let (sin_y, cos_y) = (velocity.angular[1] * delta).sin_cos();
        let (sin_z, cos_z) = (velocity.angular[2] * delta).sin_cos();

        transform.matrix[0][0] = cos_y * cos_z;
        transform.matrix[0][1] = cos_y * sin_z;
        transform.matrix[0][2] = -sin_y;
        transform.matrix[1][0] = sin_x * sin_y * cos_z - cos_x * sin_z;
        transform.matrix[1][1] = sin_x * sin_y * sin_z + cos_x * cos_z;
        transform.matrix[1][2] = sin_x * cos_y;
        transform.matrix[2][0] = cos_x * sin_y * cos_z + sin_x * sin_z;
        transform.matrix[2][1] = cos_x * sin_y * sin_z - sin_x * cos_z;
        transform.matrix[2][2] = cos_x * cos_y;

        transform.matrix[3][0] = transform.position[0];
        transform.matrix[3][1] = transform.position[1];
        transform.matrix[3][2] = transform.position[2];

        physics.energy = 0.5
            * physics.mass
            * (velocity.linear[0] * velocity.linear[0]
                + velocity.linear[1] * velocity.linear[1]
                + velocity.linear[2] * velocity.linear[2]);

        if transform.position[1] < -100.0 {
            transform.position[1] = -100.0;
            velocity.linear[1] = -velocity.linear[1] * 0.8;
        }
    }
}

fn run_physics_sequential(stress_world: &mut StressWorld, delta: f32) {
    stress_world.for_each_mut(
        TRANSFORM | VELOCITY | PHYSICS_STATE,
        0,
        |_, table, index| {
            heavy_computation(
                &mut table.transform[index],
                &mut table.velocity[index],
                &mut table.physics_state[index],
                delta,
            );
        },
    );
}

fn run_physics_parallel(stress_world: &mut StressWorld, delta: f32) {
    stress_world.par_for_each_mut(
        TRANSFORM | VELOCITY | PHYSICS_STATE,
        0,
        |_, table, index| {
            heavy_computation(
                &mut table.transform[index],
                &mut table.velocity[index],
                &mut table.physics_state[index],
                delta,
            );
        },
    );
}

fn run_benchmark_iteration(stress_world: &mut StressWorld) {
    let delta = 0.016;

    let start_seq = Instant::now();
    stress_world.for_each_mut(
        TRANSFORM | VELOCITY | PHYSICS_STATE,
        0,
        |_, table, index| {
            heavy_computation(
                &mut table.transform[index],
                &mut table.velocity[index],
                &mut table.physics_state[index],
                delta,
            );
        },
    );
    let seq_time = start_seq.elapsed().as_secs_f64() * 1000.0;

    let start_par = Instant::now();
    stress_world.par_for_each_mut(
        TRANSFORM | VELOCITY | PHYSICS_STATE,
        0,
        |_, table, index| {
            heavy_computation(
                &mut table.transform[index],
                &mut table.velocity[index],
                &mut table.physics_state[index],
                delta,
            );
        },
    );
    let par_time = start_par.elapsed().as_secs_f64() * 1000.0;

    stress_world.resources.current_iteration += 1;

    if !stress_world.resources.warmup_done {
        if stress_world.resources.current_iteration >= WARMUP_ITERATIONS {
            stress_world.resources.warmup_done = true;
            stress_world.resources.status_message = "Running benchmark...".to_string();
        }
        return;
    }

    stress_world.resources.sequential_samples.push(seq_time);
    stress_world.resources.parallel_samples.push(par_time);
    stress_world.resources.sequential_time_ms = seq_time;
    stress_world.resources.parallel_time_ms = par_time;
    stress_world.resources.speedup_ratio = seq_time / par_time;

    let samples_collected = stress_world.resources.sequential_samples.len();
    stress_world.resources.status_message = format!(
        "Collecting sample {}/{}",
        samples_collected, BENCHMARK_ITERATIONS
    );

    if samples_collected >= BENCHMARK_ITERATIONS {
        stress_world.resources.benchmark_complete = true;
        stress_world.resources.benchmark_started = false;
        stress_world.resources.status_message =
            "Benchmark complete! Generating report...".to_string();
        generate_report(stress_world);
    }
}

#[derive(Debug, Clone)]
struct Stats {
    mean: f64,
    median: f64,
    min: f64,
    max: f64,
    std_dev: f64,
    p5: f64,
    p95: f64,
}

fn calculate_stats(samples: &[f64]) -> Stats {
    if samples.is_empty() {
        return Stats {
            mean: 0.0,
            median: 0.0,
            min: 0.0,
            max: 0.0,
            std_dev: 0.0,
            p5: 0.0,
            p95: 0.0,
        };
    }

    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let length = sorted.len();
    let sum: f64 = sorted.iter().sum();
    let mean = sum / length as f64;

    let median = if length.is_multiple_of(2) {
        (sorted[length / 2 - 1] + sorted[length / 2]) / 2.0
    } else {
        sorted[length / 2]
    };

    let min = sorted[0];
    let max = sorted[length - 1];

    let variance: f64 = sorted.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / length as f64;
    let std_dev = variance.sqrt();

    let p5_index = ((length as f64 * 0.05).floor() as usize).min(length - 1);
    let p95_index = ((length as f64 * 0.95).floor() as usize).min(length - 1);
    let p5 = sorted[p5_index];
    let p95 = sorted[p95_index];

    Stats {
        mean,
        median,
        min,
        max,
        std_dev,
        p5,
        p95,
    }
}

fn generate_report(stress_world: &mut StressWorld) {
    let seq_stats = calculate_stats(&stress_world.resources.sequential_samples);
    let par_stats = calculate_stats(&stress_world.resources.parallel_samples);

    let speedup_mean = seq_stats.mean / par_stats.mean;
    let speedup_median = seq_stats.median / par_stats.median;
    let speedup_best = seq_stats.min / par_stats.min;
    let speedup_worst = seq_stats.max / par_stats.max;

    let efficiency = (speedup_mean / rayon::current_num_threads() as f64) * 100.0;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let report_filename = format!("parallel_ecs_benchmark_{}.txt", timestamp);
    let report_path = std::env::current_dir()
        .unwrap()
        .join(&report_filename)
        .to_string_lossy()
        .to_string();

    let mut report = String::new();

    report.push_str(
        "================================================================================\n",
    );
    report.push_str("                    FREECS PARALLEL ECS BENCHMARK REPORT\n");
    report.push_str(
        "================================================================================\n\n",
    );

    report.push_str("SYSTEM CONFIGURATION\n");
    report.push_str("--------------------\n");
    report.push_str(&format!(
        "CPU Threads (Rayon):        {}\n",
        rayon::current_num_threads()
    ));
    report.push_str(&format!(
        "Entity Count:               {}\n",
        format_with_commas(stress_world.resources.entity_count)
    ));
    report.push_str("Components per Entity:      3 (Transform, Velocity, PhysicsState)\n");
    report.push_str(&format!(
        "Total Component Instances:  {}\n",
        format_with_commas(stress_world.resources.entity_count * 3)
    ));
    report.push_str(&format!(
        "Computation Iterations:     {} per entity per frame\n",
        COMPUTATION_ITERATIONS
    ));
    report.push_str(&format!(
        "Total Operations per Frame: {}\n",
        format_with_commas(stress_world.resources.entity_count * COMPUTATION_ITERATIONS)
    ));
    report.push('\n');

    report.push_str("BENCHMARK PARAMETERS\n");
    report.push_str("--------------------\n");
    report.push_str(&format!(
        "Warmup Iterations:          {}\n",
        WARMUP_ITERATIONS
    ));
    report.push_str(&format!(
        "Measured Iterations:        {}\n",
        BENCHMARK_ITERATIONS
    ));
    report.push_str("Fixed Delta Time:           16ms (60 FPS simulation)\n");
    report.push('\n');

    report.push_str("SEQUENTIAL EXECUTION STATISTICS (milliseconds)\n");
    report.push_str("----------------------------------------------\n");
    report.push_str(&format!(
        "Mean:                       {:.3} ms\n",
        seq_stats.mean
    ));
    report.push_str(&format!(
        "Median:                     {:.3} ms\n",
        seq_stats.median
    ));
    report.push_str(&format!(
        "Minimum:                    {:.3} ms\n",
        seq_stats.min
    ));
    report.push_str(&format!(
        "Maximum:                    {:.3} ms\n",
        seq_stats.max
    ));
    report.push_str(&format!(
        "Standard Deviation:         {:.3} ms\n",
        seq_stats.std_dev
    ));
    report.push_str(&format!(
        "5th Percentile:             {:.3} ms\n",
        seq_stats.p5
    ));
    report.push_str(&format!(
        "95th Percentile:            {:.3} ms\n",
        seq_stats.p95
    ));
    report.push_str(&format!(
        "Coefficient of Variation:   {:.2}%\n",
        (seq_stats.std_dev / seq_stats.mean) * 100.0
    ));
    report.push('\n');

    report.push_str("PARALLEL EXECUTION STATISTICS (milliseconds)\n");
    report.push_str("--------------------------------------------\n");
    report.push_str(&format!(
        "Mean:                       {:.3} ms\n",
        par_stats.mean
    ));
    report.push_str(&format!(
        "Median:                     {:.3} ms\n",
        par_stats.median
    ));
    report.push_str(&format!(
        "Minimum:                    {:.3} ms\n",
        par_stats.min
    ));
    report.push_str(&format!(
        "Maximum:                    {:.3} ms\n",
        par_stats.max
    ));
    report.push_str(&format!(
        "Standard Deviation:         {:.3} ms\n",
        par_stats.std_dev
    ));
    report.push_str(&format!(
        "5th Percentile:             {:.3} ms\n",
        par_stats.p5
    ));
    report.push_str(&format!(
        "95th Percentile:            {:.3} ms\n",
        par_stats.p95
    ));
    report.push_str(&format!(
        "Coefficient of Variation:   {:.2}%\n",
        (par_stats.std_dev / par_stats.mean) * 100.0
    ));
    report.push('\n');

    report.push_str("SPEEDUP ANALYSIS\n");
    report.push_str("----------------\n");
    report.push_str(&format!(
        "Mean Speedup:               {:.2}x\n",
        speedup_mean
    ));
    report.push_str(&format!(
        "Median Speedup:             {:.2}x\n",
        speedup_median
    ));
    report.push_str(&format!(
        "Best Case Speedup:          {:.2}x\n",
        speedup_best
    ));
    report.push_str(&format!(
        "Worst Case Speedup:         {:.2}x\n",
        speedup_worst
    ));
    report.push_str(&format!(
        "Parallel Efficiency:        {:.1}% (speedup / cores * 100)\n",
        efficiency
    ));
    report.push_str(&format!(
        "Time Saved per Frame:       {:.3} ms\n",
        seq_stats.mean - par_stats.mean
    ));
    report.push_str(&format!(
        "Throughput Improvement:     {:.1}%\n",
        (speedup_mean - 1.0) * 100.0
    ));
    report.push('\n');

    report.push_str("THEORETICAL VS ACTUAL\n");
    report.push_str("---------------------\n");
    let theoretical_max = rayon::current_num_threads() as f64;
    report.push_str(&format!(
        "Theoretical Max Speedup:    {:.1}x (with {} threads)\n",
        theoretical_max,
        rayon::current_num_threads()
    ));
    report.push_str(&format!(
        "Actual Mean Speedup:        {:.2}x\n",
        speedup_mean
    ));
    report.push_str(&format!(
        "Efficiency vs Theoretical:  {:.1}%\n",
        (speedup_mean / theoretical_max) * 100.0
    ));
    report.push('\n');

    let can_hit_60fps_seq = seq_stats.mean < 16.67;
    let can_hit_60fps_par = par_stats.mean < 16.67;
    let can_hit_120fps_seq = seq_stats.mean < 8.33;
    let can_hit_120fps_par = par_stats.mean < 8.33;

    report.push_str("FRAME RATE FEASIBILITY\n");
    report.push_str("----------------------\n");
    report.push_str(&format!(
        "Sequential 60 FPS capable:  {} (need <16.67ms, got {:.2}ms)\n",
        if can_hit_60fps_seq { "YES" } else { "NO" },
        seq_stats.mean
    ));
    report.push_str(&format!(
        "Parallel 60 FPS capable:    {} (need <16.67ms, got {:.2}ms)\n",
        if can_hit_60fps_par { "YES" } else { "NO" },
        par_stats.mean
    ));
    report.push_str(&format!(
        "Sequential 120 FPS capable: {} (need <8.33ms, got {:.2}ms)\n",
        if can_hit_120fps_seq { "YES" } else { "NO" },
        seq_stats.mean
    ));
    report.push_str(&format!(
        "Parallel 120 FPS capable:   {} (need <8.33ms, got {:.2}ms)\n",
        if can_hit_120fps_par { "YES" } else { "NO" },
        par_stats.mean
    ));
    report.push('\n');

    report.push_str("RAW DATA - SEQUENTIAL (ms)\n");
    report.push_str("--------------------------\n");
    for (index, sample) in stress_world.resources.sequential_samples.iter().enumerate() {
        if index > 0 && index % 10 == 0 {
            report.push('\n');
        }
        report.push_str(&format!("{:8.3} ", sample));
    }
    report.push_str("\n\n");

    report.push_str("RAW DATA - PARALLEL (ms)\n");
    report.push_str("------------------------\n");
    for (index, sample) in stress_world.resources.parallel_samples.iter().enumerate() {
        if index > 0 && index % 10 == 0 {
            report.push('\n');
        }
        report.push_str(&format!("{:8.3} ", sample));
    }
    report.push_str("\n\n");

    report.push_str("CONCLUSION\n");
    report.push_str("----------\n");
    if speedup_mean > 1.5 {
        report.push_str(&format!(
            "Parallel execution provides SIGNIFICANT benefit with {:.2}x speedup.\n",
            speedup_mean
        ));
        report.push_str(&format!(
            "The ECS can efficiently utilize {} CPU cores for this workload.\n",
            rayon::current_num_threads()
        ));
        if efficiency > 50.0 {
            report
                .push_str("Parallel efficiency is GOOD - the workload scales well across cores.\n");
        } else {
            report.push_str(
                "Parallel efficiency is MODERATE - some overhead from synchronization.\n",
            );
        }
    } else if speedup_mean > 1.0 {
        report.push_str("Parallel execution provides MODEST benefit.\n");
        report.push_str(
            "Consider increasing workload complexity for better parallelization gains.\n",
        );
    } else {
        report.push_str("Parallel execution provides NO benefit for this workload.\n");
        report.push_str("The overhead of thread coordination exceeds the parallelization gains.\n");
    }

    report.push_str(
        "\n================================================================================\n",
    );
    report.push_str("                              END OF REPORT\n");
    report.push_str(
        "================================================================================\n",
    );

    if let Ok(mut file) = std::fs::File::create(&report_path) {
        let _ = file.write_all(report.as_bytes());
        stress_world.resources.report_path = report_path;
        stress_world.resources.status_message = "Report saved successfully!".to_string();
    } else {
        stress_world.resources.status_message = "Failed to save report!".to_string();
    }
}

fn format_with_commas(number: usize) -> String {
    let string = number.to_string();
    let mut result = String::new();
    let chars: Vec<char> = string.chars().collect();

    for (index, character) in chars.iter().enumerate() {
        if index > 0 && (chars.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*character);
    }

    result
}
