use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(PongGame::default())
}

const PADDLE_WIDTH: f32 = 0.3;
const PADDLE_HEIGHT: f32 = 2.0;
const PADDLE_DEPTH: f32 = 0.3;
const PADDLE_SPEED: f32 = 8.0;
const BALL_SIZE: f32 = 0.3;
const BALL_SPEED: f32 = 6.0;
const ARENA_WIDTH: f32 = 12.0;
const ARENA_HEIGHT: f32 = 8.0;
const WINNING_SCORE: u32 = 5;
const AI_SPEED_MULTIPLIER: f32 = 0.75;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
}

#[derive(Default)]
struct PongGame {
    state: GameState,
    left_paddle_y: f32,
    right_paddle_y: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vel_x: f32,
    ball_vel_y: f32,
    left_score: u32,
    right_score: u32,
    left_paddle_entity: Option<Entity>,
    right_paddle_entity: Option<Entity>,
    ball_entity: Option<Entity>,
    ball_start_direction: f32,
}

impl State for PongGame {
    fn title(&self) -> &str {
        "Pong"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Space;
        world.resources.graphics.show_grid = false;

        spawn_sun_without_shadows(world);

        let camera = spawn_camera(world, Vec3::new(0.0, 0.0, 15.0), "Main Camera".to_string());

        if let Some(camera_component) = world.core.get_camera_mut(camera) {
            camera_component.projection = Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_far: Some(1000.0),
                z_near: 0.1,
            });
        }

        world.resources.active_camera = Some(camera);

        self.create_game_objects(world);
        self.ball_start_direction = 1.0;
        self.reset_ball();
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if self.state == GameState::Playing {
            self.input_system(world);
            self.ai_system(world);
            self.ball_movement_system(world);
            self.collision_system(world);
        }

        self.update_visuals(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state == KeyState::Pressed {
            match key {
                KeyCode::Space => match self.state {
                    GameState::Playing => self.state = GameState::Paused,
                    GameState::Paused => self.state = GameState::Playing,
                    GameState::GameOver => {}
                },
                KeyCode::KeyR => {
                    self.reset_game(world);
                }
                _ => {}
            }
        }
    }

    fn ui(&mut self, _world: &mut World, ctx: &egui::Context) {
        egui::Window::new("Pong")
            .anchor(egui::Align2::CENTER_TOP, [0.0, 10.0])
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(format!("{} - {}", self.left_score, self.right_score));
                });
            });

        match self.state {
            GameState::Paused => {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_black_alpha(180)))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(100.0);
                            ui.heading("PAUSED");
                            ui.add_space(20.0);
                            ui.label("Press SPACE to resume");
                            ui.label("Press R to restart");
                        });
                    });
            }
            GameState::GameOver => {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_black_alpha(180)))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(100.0);
                            let winner = if self.left_score >= WINNING_SCORE {
                                "You Win!"
                            } else {
                                "AI Wins!"
                            };
                            ui.heading(winner);
                            ui.add_space(10.0);
                            ui.label(format!(
                                "Final Score: {} - {}",
                                self.left_score, self.right_score
                            ));
                            ui.add_space(20.0);
                            ui.label("Press R to play again");
                        });
                    });
            }
            GameState::Playing => {}
        }

        egui::Window::new("Controls")
            .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("W/S or ↑/↓ - Move paddle");
                ui.label("SPACE - Pause");
                ui.label("R - Restart");
                ui.label("ESC - Exit");
            });
    }
}

impl PongGame {
    fn create_game_objects(&mut self, world: &mut World) {
        let left_paddle = spawn_mesh(
            world,
            "Cube",
            Vec3::new(-ARENA_WIDTH / 2.0 + 0.5, 0.0, 0.0),
            Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT, PADDLE_DEPTH),
        );
        let left_paddle_material = format!("LeftPaddle_{}", left_paddle.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            left_paddle_material.clone(),
            Material {
                base_color: [0.2, 0.6, 1.0, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&left_paddle_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(left_paddle, MaterialRef::new(left_paddle_material));
        self.left_paddle_entity = Some(left_paddle);

        let right_paddle = spawn_mesh(
            world,
            "Cube",
            Vec3::new(ARENA_WIDTH / 2.0 - 0.5, 0.0, 0.0),
            Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT, PADDLE_DEPTH),
        );
        let right_paddle_material = format!("RightPaddle_{}", right_paddle.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            right_paddle_material.clone(),
            Material {
                base_color: [1.0, 0.4, 0.2, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&right_paddle_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(right_paddle, MaterialRef::new(right_paddle_material));
        self.right_paddle_entity = Some(right_paddle);

        let ball = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(BALL_SIZE, BALL_SIZE, BALL_SIZE),
        );
        let ball_material = format!("Ball_{}", ball.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            ball_material.clone(),
            Material {
                base_color: [1.0, 1.0, 1.0, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&ball_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(ball, MaterialRef::new(ball_material));
        self.ball_entity = Some(ball);

        let top_wall = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, ARENA_HEIGHT / 2.0 + 0.25, 0.0),
            Vec3::new(ARENA_WIDTH + 1.0, 0.5, 0.5),
        );
        let top_wall_material = format!("TopWall_{}", top_wall.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            top_wall_material.clone(),
            Material {
                base_color: [0.5, 0.5, 0.5, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&top_wall_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(top_wall, MaterialRef::new(top_wall_material));

        let bottom_wall = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, -ARENA_HEIGHT / 2.0 - 0.25, 0.0),
            Vec3::new(ARENA_WIDTH + 1.0, 0.5, 0.5),
        );
        let bottom_wall_material = format!("BottomWall_{}", bottom_wall.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            bottom_wall_material.clone(),
            Material {
                base_color: [0.5, 0.5, 0.5, 1.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&bottom_wall_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.core.set_material_ref(bottom_wall, MaterialRef::new(bottom_wall_material));
    }

    fn reset_ball(&mut self) {
        self.ball_x = 0.0;
        self.ball_y = 0.0;
        self.ball_start_direction *= -1.0;
        let angle = (rand::random::<f32>() - 0.5) * std::f32::consts::PI * 0.5;
        self.ball_vel_x = BALL_SPEED * self.ball_start_direction * angle.cos();
        self.ball_vel_y = BALL_SPEED * angle.sin();
    }

    fn reset_game(&mut self, _world: &mut World) {
        self.left_paddle_y = 0.0;
        self.right_paddle_y = 0.0;
        self.left_score = 0;
        self.right_score = 0;
        self.state = GameState::Playing;
        self.ball_start_direction = 1.0;
        self.reset_ball();
    }

    fn input_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;

        if world.resources.input.keyboard.is_key_pressed(KeyCode::KeyW)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::ArrowUp)
        {
            self.left_paddle_y += PADDLE_SPEED * delta_time;
        }
        if world.resources.input.keyboard.is_key_pressed(KeyCode::KeyS)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::ArrowDown)
        {
            self.left_paddle_y -= PADDLE_SPEED * delta_time;
        }

        let max_y = ARENA_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        self.left_paddle_y = self.left_paddle_y.clamp(-max_y, max_y);
    }

    fn ai_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;

        let target_y = self.ball_y;
        let distance = target_y - self.right_paddle_y;

        if distance.abs() > 0.2 {
            let movement = distance.signum() * PADDLE_SPEED * AI_SPEED_MULTIPLIER * delta_time;
            self.right_paddle_y += movement;
        }

        let max_y = ARENA_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        self.right_paddle_y = self.right_paddle_y.clamp(-max_y, max_y);
    }

    fn ball_movement_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;

        self.ball_x += self.ball_vel_x * delta_time;
        self.ball_y += self.ball_vel_y * delta_time;
    }

    fn collision_system(&mut self, _world: &mut World) {
        let ball_max_y = ARENA_HEIGHT / 2.0 - BALL_SIZE;
        if self.ball_y > ball_max_y {
            self.ball_y = ball_max_y;
            self.ball_vel_y = -self.ball_vel_y.abs();
        } else if self.ball_y < -ball_max_y {
            self.ball_y = -ball_max_y;
            self.ball_vel_y = self.ball_vel_y.abs();
        }

        let left_paddle_x = -ARENA_WIDTH / 2.0 + 0.5;
        if self.ball_x < left_paddle_x + PADDLE_WIDTH / 2.0 + BALL_SIZE
            && self.ball_x > left_paddle_x - PADDLE_WIDTH / 2.0
            && (self.ball_y - self.left_paddle_y).abs() < PADDLE_HEIGHT / 2.0 + BALL_SIZE
        {
            self.ball_x = left_paddle_x + PADDLE_WIDTH / 2.0 + BALL_SIZE;
            self.ball_vel_x = self.ball_vel_x.abs();
            let hit_offset = (self.ball_y - self.left_paddle_y) / (PADDLE_HEIGHT / 2.0);
            self.ball_vel_y += hit_offset * 2.0;
            self.normalize_ball_speed();
        }

        let right_paddle_x = ARENA_WIDTH / 2.0 - 0.5;
        if self.ball_x > right_paddle_x - PADDLE_WIDTH / 2.0 - BALL_SIZE
            && self.ball_x < right_paddle_x + PADDLE_WIDTH / 2.0
            && (self.ball_y - self.right_paddle_y).abs() < PADDLE_HEIGHT / 2.0 + BALL_SIZE
        {
            self.ball_x = right_paddle_x - PADDLE_WIDTH / 2.0 - BALL_SIZE;
            self.ball_vel_x = -self.ball_vel_x.abs();
            let hit_offset = (self.ball_y - self.right_paddle_y) / (PADDLE_HEIGHT / 2.0);
            self.ball_vel_y += hit_offset * 2.0;
            self.normalize_ball_speed();
        }

        if self.ball_x < -ARENA_WIDTH / 2.0 - 1.0 {
            self.right_score += 1;
            self.reset_ball();
            if self.right_score >= WINNING_SCORE {
                self.state = GameState::GameOver;
            }
        } else if self.ball_x > ARENA_WIDTH / 2.0 + 1.0 {
            self.left_score += 1;
            self.reset_ball();
            if self.left_score >= WINNING_SCORE {
                self.state = GameState::GameOver;
            }
        }
    }

    fn normalize_ball_speed(&mut self) {
        let speed = (self.ball_vel_x * self.ball_vel_x + self.ball_vel_y * self.ball_vel_y).sqrt();
        self.ball_vel_x *= BALL_SPEED / speed;
        self.ball_vel_y *= BALL_SPEED / speed;
    }

    fn update_visuals(&mut self, world: &mut World) {
        if let Some(entity) = self.left_paddle_entity {
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.translation.y = self.left_paddle_y;
            }
            mark_local_transform_dirty(world, entity);
        }

        if let Some(entity) = self.right_paddle_entity {
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.translation.y = self.right_paddle_y;
            }
            mark_local_transform_dirty(world, entity);
        }

        if let Some(entity) = self.ball_entity {
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.translation.x = self.ball_x;
                transform.translation.y = self.ball_y;
            }
            mark_local_transform_dirty(world, entity);
        }
    }
}
