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

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum GameState {
    #[default]
    Menu,
    WaitingForOpponent,
    Playing,
    GameOver,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlayerRole {
    Host,
    Guest,
}

#[derive(Default)]
struct PongGame {
    state: GameState,
    role: Option<PlayerRole>,
    opponent_id: Option<SteamId>,
    opponent_name: String,
    left_paddle_y: f32,
    right_paddle_y: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vel_x: f32,
    ball_vel_y: f32,
    left_score: u32,
    right_score: u32,
    prev_left_score: u32,
    prev_right_score: u32,
    left_paddle_entity: Option<Entity>,
    right_paddle_entity: Option<Entity>,
    ball_entity: Option<Entity>,
    left_score_text: Option<Entity>,
    right_score_text: Option<Entity>,
    left_score_text_index: Option<usize>,
    right_score_text_index: Option<usize>,
    steam_initialized: bool,
    networking_initialized: bool,
    selected_friend_index: Option<usize>,
    status_message: Option<(String, Instant)>,
    last_network_update: Option<Instant>,
    last_join_attempt: Option<Instant>,
    session_id: u32,
    network_channel: u32,
}

#[derive(Clone, Copy)]
struct NetworkState {
    session_id: u32,
    channel: u32,
    paddle_y: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vel_x: f32,
    ball_vel_y: f32,
    left_score: u32,
    right_score: u32,
}

impl NetworkState {
    fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(36);
        bytes.extend_from_slice(&self.session_id.to_le_bytes());
        bytes.extend_from_slice(&self.channel.to_le_bytes());
        bytes.extend_from_slice(&self.paddle_y.to_le_bytes());
        bytes.extend_from_slice(&self.ball_x.to_le_bytes());
        bytes.extend_from_slice(&self.ball_y.to_le_bytes());
        bytes.extend_from_slice(&self.ball_vel_x.to_le_bytes());
        bytes.extend_from_slice(&self.ball_vel_y.to_le_bytes());
        bytes.extend_from_slice(&self.left_score.to_le_bytes());
        bytes.extend_from_slice(&self.right_score.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 36 {
            return None;
        }
        Some(Self {
            session_id: u32::from_le_bytes(bytes[0..4].try_into().ok()?),
            channel: u32::from_le_bytes(bytes[4..8].try_into().ok()?),
            paddle_y: f32::from_le_bytes(bytes[8..12].try_into().ok()?),
            ball_x: f32::from_le_bytes(bytes[12..16].try_into().ok()?),
            ball_y: f32::from_le_bytes(bytes[16..20].try_into().ok()?),
            ball_vel_x: f32::from_le_bytes(bytes[20..24].try_into().ok()?),
            ball_vel_y: f32::from_le_bytes(bytes[24..28].try_into().ok()?),
            left_score: u32::from_le_bytes(bytes[28..32].try_into().ok()?),
            right_score: u32::from_le_bytes(bytes[32..36].try_into().ok()?),
        })
    }
}

impl State for PongGame {
    fn title(&self) -> &str {
        "Multiplayer Pong"
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

        if let Err(error) = world.resources.steam.initialize() {
            tracing::error!("Steam initialization failed: {}", error);
        }
        self.steam_initialized = world.resources.steam.is_initialized();

        self.create_game_objects(world);
        self.reset_ball();
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if self.steam_initialized && !self.networking_initialized {
            world.resources.steam.setup_networking_callbacks();
            world.resources.steam.refresh_friends();
            self.networking_initialized = true;
        }

        if self.steam_initialized {
            world.resources.steam.run_callbacks();
            world.resources.steam.receive_messages(0, 100);
            if self.network_channel != 0 {
                world
                    .resources
                    .steam
                    .receive_messages(self.network_channel, 100);
            }
            world.resources.steam.process_pending_requests();
            self.process_network_messages(world);

            if self.state == GameState::WaitingForOpponent
                && self.role == Some(PlayerRole::Guest)
                && let Some(opponent_id) = self.opponent_id
            {
                let should_retry = self
                    .last_join_attempt
                    .map(|t| t.elapsed().as_millis() >= 1000)
                    .unwrap_or(true);

                if should_retry {
                    let join_msg = NetworkState {
                        session_id: 0,
                        channel: 0,
                        paddle_y: 0.0,
                        ball_x: 0.0,
                        ball_y: 0.0,
                        ball_vel_x: 0.0,
                        ball_vel_y: 0.0,
                        left_score: 0,
                        right_score: 0,
                    };
                    let _ = world.resources.steam.send_message(
                        opponent_id,
                        &join_msg.to_bytes(),
                        0,
                        true,
                    );
                    self.last_join_attempt = Some(Instant::now());
                }
            }
        }

        if self.state == GameState::Playing {
            self.input_system(world);
            if self.role == Some(PlayerRole::Host) {
                self.ball_movement_system(world);
                self.collision_system();
            }
            self.send_network_state(world);
        }

        self.update_visuals(world);
        self.update_score_display(world);
        nightshade::ecs::text::systems::sync_text_meshes_system(world);
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        match self.state {
            GameState::Menu => self.render_menu(world, ctx),
            GameState::WaitingForOpponent => self.render_waiting(world, ctx),
            GameState::Playing => self.render_playing(world, ctx),
            GameState::GameOver => self.render_game_over(world, ctx),
        }
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

        let left_score_text = spawn_3d_billboard_text_with_properties(
            world,
            "0",
            Vec3::new(-3.0, ARENA_HEIGHT / 2.0 + 1.5, 0.0),
            TextProperties {
                font_size: 72.0,
                color: Vec4::new(0.2, 0.6, 1.0, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        self.left_score_text = Some(left_score_text);
        if let Some(text) = world.core.get_text(left_score_text) {
            self.left_score_text_index = Some(text.text_index);
        }

        let right_score_text = spawn_3d_billboard_text_with_properties(
            world,
            "0",
            Vec3::new(3.0, ARENA_HEIGHT / 2.0 + 1.5, 0.0),
            TextProperties {
                font_size: 72.0,
                color: Vec4::new(1.0, 0.4, 0.2, 1.0),
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                outline_width: 0.03,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        self.right_score_text = Some(right_score_text);
        if let Some(text) = world.core.get_text(right_score_text) {
            self.right_score_text_index = Some(text.text_index);
        }
    }

    fn reset_ball(&mut self) {
        self.ball_x = 0.0;
        self.ball_y = 0.0;
        self.ball_vel_x = 8.0;
        self.ball_vel_y = 3.0;
    }

    fn input_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;

        let my_paddle_y = if self.role == Some(PlayerRole::Host) {
            &mut self.left_paddle_y
        } else {
            &mut self.right_paddle_y
        };

        if world.resources.input.keyboard.is_key_pressed(KeyCode::KeyW)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::ArrowUp)
        {
            *my_paddle_y += PADDLE_SPEED * delta_time;
        }
        if world.resources.input.keyboard.is_key_pressed(KeyCode::KeyS)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::ArrowDown)
        {
            *my_paddle_y -= PADDLE_SPEED * delta_time;
        }

        let max_y = ARENA_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        *my_paddle_y = my_paddle_y.clamp(-max_y, max_y);
    }

    fn ball_movement_system(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        self.ball_x += self.ball_vel_x * delta_time;
        self.ball_y += self.ball_vel_y * delta_time;
    }

    fn collision_system(&mut self) {
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

    fn send_network_state(&mut self, world: &mut World) {
        let now = Instant::now();
        let should_send = self
            .last_network_update
            .map(|last| now.duration_since(last).as_millis() >= 16)
            .unwrap_or(true);

        if !should_send {
            return;
        }

        let Some(opponent_id) = self.opponent_id else {
            return;
        };

        let my_paddle_y = if self.role == Some(PlayerRole::Host) {
            self.left_paddle_y
        } else {
            self.right_paddle_y
        };

        let state = NetworkState {
            session_id: self.session_id,
            channel: self.network_channel,
            paddle_y: my_paddle_y,
            ball_x: self.ball_x,
            ball_y: self.ball_y,
            ball_vel_x: self.ball_vel_x,
            ball_vel_y: self.ball_vel_y,
            left_score: self.left_score,
            right_score: self.right_score,
        };

        let _ = world.resources.steam.send_message(
            opponent_id,
            &state.to_bytes(),
            self.network_channel,
            false,
        );

        self.last_network_update = Some(now);
    }

    fn process_network_messages(&mut self, world: &mut World) {
        let messages: Vec<_> = world
            .resources
            .steam
            .received_messages
            .iter()
            .filter(|m| !m.is_outgoing && (m.channel == 0 || m.channel == self.network_channel))
            .cloned()
            .collect();

        for message in messages {
            let Some(net_state) = NetworkState::from_bytes(&message.data) else {
                continue;
            };

            if self.role == Some(PlayerRole::Host)
                && message.channel == 0
                && net_state.session_id == 0
            {
                if self.state == GameState::WaitingForOpponent {
                    self.opponent_id = Some(message.sender_id);
                    self.opponent_name = message.sender_name.clone();
                    self.state = GameState::Playing;
                    self.set_status(format!("{} joined the game!", self.opponent_name));
                    self.reset_ball();
                }

                if self.opponent_id == Some(message.sender_id) {
                    let initial_state = NetworkState {
                        session_id: self.session_id,
                        channel: self.network_channel,
                        paddle_y: self.left_paddle_y,
                        ball_x: self.ball_x,
                        ball_y: self.ball_y,
                        ball_vel_x: self.ball_vel_x,
                        ball_vel_y: self.ball_vel_y,
                        left_score: self.left_score,
                        right_score: self.right_score,
                    };
                    let _ = world.resources.steam.send_message(
                        message.sender_id,
                        &initial_state.to_bytes(),
                        0,
                        true,
                    );
                }
                continue;
            }

            if self.state == GameState::WaitingForOpponent
                && self.role == Some(PlayerRole::Guest)
                && self.opponent_id == Some(message.sender_id)
                && net_state.session_id != 0
                && message.channel == 0
            {
                self.session_id = net_state.session_id;
                self.network_channel = net_state.channel;
                self.opponent_name = message.sender_name.clone();
                self.state = GameState::Playing;
                self.set_status("Connected to host!");

                self.left_paddle_y = net_state.paddle_y;
                self.ball_x = net_state.ball_x;
                self.ball_y = net_state.ball_y;
                self.ball_vel_x = net_state.ball_vel_x;
                self.ball_vel_y = net_state.ball_vel_y;
                self.left_score = net_state.left_score;
                self.right_score = net_state.right_score;
            }

            if self.state == GameState::Playing
                && let Some(opponent) = self.opponent_id
                && message.sender_id == opponent
                && net_state.session_id == self.session_id
                && message.channel == self.network_channel
            {
                if self.role == Some(PlayerRole::Host) {
                    self.right_paddle_y = net_state.paddle_y;
                } else {
                    self.left_paddle_y = net_state.paddle_y;
                    self.ball_x = net_state.ball_x;
                    self.ball_y = net_state.ball_y;
                    self.ball_vel_x = net_state.ball_vel_x;
                    self.ball_vel_y = net_state.ball_vel_y;
                    self.left_score = net_state.left_score;
                    self.right_score = net_state.right_score;

                    if self.left_score >= WINNING_SCORE || self.right_score >= WINNING_SCORE {
                        self.state = GameState::GameOver;
                    }
                }
            }
        }

        world
            .resources
            .steam
            .received_messages
            .retain(|m| m.is_outgoing || (m.channel != 0 && m.channel != self.network_channel));
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

    fn update_score_display(&mut self, world: &mut World) {
        if self.left_score != self.prev_left_score {
            if let Some(text_index) = self.left_score_text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, self.left_score.to_string());
            }
            if let Some(entity) = self.left_score_text
                && let Some(text) = world.core.get_text_mut(entity)
            {
                text.dirty = true;
            }
            self.prev_left_score = self.left_score;
        }

        if self.right_score != self.prev_right_score {
            if let Some(text_index) = self.right_score_text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, self.right_score.to_string());
            }
            if let Some(entity) = self.right_score_text
                && let Some(text) = world.core.get_text_mut(entity)
            {
                text.dirty = true;
            }
            self.prev_right_score = self.right_score;
        }
    }

    fn render_menu(&mut self, world: &mut World, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("Multiplayer Pong");
                ui.add_space(20.0);

                if !self.steam_initialized {
                    ui.colored_label(egui::Color32::RED, "Steam not initialized!");
                    if let Some(error) = &world.resources.steam.initialization_error {
                        ui.label(format!("Error: {}", error));
                    }
                    return;
                }

                ui.label(format!("Logged in as: {}", world.resources.steam.user_name));
                ui.add_space(20.0);

                if let Some((message, time)) = &self.status_message {
                    if time.elapsed().as_secs() < 3 {
                        ui.colored_label(egui::Color32::GREEN, message);
                        ui.add_space(10.0);
                    } else {
                        self.status_message = None;
                    }
                }

                ui.separator();
                ui.add_space(10.0);

                if ui.button("Host Game").clicked() {
                    world.resources.steam.close_all_sessions();
                    world.resources.steam.clear_messages();
                    self.session_id = rand::random::<u32>().saturating_add(1);
                    self.network_channel = rand::random::<u32>().saturating_add(1);
                    self.role = Some(PlayerRole::Host);
                    self.state = GameState::WaitingForOpponent;
                    self.reset_game();
                    self.set_status("Waiting for opponent to join...");
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label("Join a friend's game:");
                ui.add_space(5.0);

                if ui.button("Refresh Friends").clicked() {
                    world.resources.steam.refresh_friends();
                }

                let friends = world.resources.steam.friends.clone();
                let online_friends: Vec<_> = friends
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.state.is_online())
                    .map(|(index, f)| (index, f.name.clone(), f.steam_id))
                    .collect();

                if online_friends.is_empty() {
                    ui.label("No online friends");
                } else {
                    let selected_name = self
                        .selected_friend_index
                        .and_then(|index| friends.get(index))
                        .map(|f| f.name.clone())
                        .unwrap_or_else(|| "Select friend...".to_string());

                    egui::ComboBox::from_id_salt("friend_select")
                        .selected_text(&selected_name)
                        .show_ui(ui, |ui| {
                            for (index, name, _) in &online_friends {
                                ui.selectable_value(
                                    &mut self.selected_friend_index,
                                    Some(*index),
                                    name,
                                );
                            }
                        });

                    if let Some(friend_index) = self.selected_friend_index
                        && let Some((_, _, steam_id)) = online_friends
                            .iter()
                            .find(|(index, _, _)| *index == friend_index)
                    {
                        ui.horizontal(|ui| {
                            if ui.button("Join Game").clicked() {
                                world.resources.steam.close_all_sessions();
                                world.resources.steam.clear_messages();
                                self.session_id = 0;
                                self.network_channel = 0;
                                self.role = Some(PlayerRole::Guest);
                                self.opponent_id = Some(*steam_id);
                                self.state = GameState::WaitingForOpponent;
                                self.reset_game();
                                self.last_join_attempt = None;
                                self.set_status("Connecting to host...");
                            }

                            if ui.button("Message Friend").clicked() {
                                world
                                    .resources
                                    .steam
                                    .open_overlay_to_user("chat", *steam_id);
                            }
                        });
                    }
                }
            });
        });
    }

    fn render_waiting(&mut self, world: &mut World, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("Waiting for Opponent...");
                ui.add_space(20.0);

                if self.role == Some(PlayerRole::Host) {
                    ui.label("Waiting for a friend to join...");
                    ui.add_space(10.0);
                    ui.label("Tell your friend to:");
                    ui.label("1. Open the game");
                    ui.label("2. Select your name from their friend list");
                    ui.label("3. Click 'Join Game'");
                } else {
                    ui.label("Connecting to host...");
                }

                ui.add_space(20.0);

                if ui.button("Cancel").clicked() {
                    world.resources.steam.close_all_sessions();
                    world.resources.steam.clear_messages();
                    self.state = GameState::Menu;
                    self.role = None;
                    self.opponent_id = None;
                    self.session_id = 0;
                    self.network_channel = 0;
                }
            });
        });
    }

    fn render_playing(&mut self, _world: &mut World, ctx: &egui::Context) {
        egui::TopBottomPanel::top("score_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(50.0);

                let left_label = if self.role == Some(PlayerRole::Host) {
                    "You"
                } else {
                    &self.opponent_name
                };
                let right_label = if self.role == Some(PlayerRole::Guest) {
                    "You"
                } else {
                    &self.opponent_name
                };

                ui.heading(format!("{}: {}", left_label, self.left_score));
                ui.add_space(100.0);
                ui.heading(format!("{}: {}", right_label, self.right_score));
            });
        });

        egui::TopBottomPanel::bottom("controls_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Controls: W/S or Arrow Up/Down to move paddle");
            });
        });
    }

    fn render_game_over(&mut self, world: &mut World, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("Game Over!");
                ui.add_space(20.0);

                let winner = if self.left_score >= WINNING_SCORE {
                    if self.role == Some(PlayerRole::Host) {
                        "You Win!".to_string()
                    } else {
                        format!("{} Wins!", self.opponent_name)
                    }
                } else if self.role == Some(PlayerRole::Guest) {
                    "You Win!".to_string()
                } else {
                    format!("{} Wins!", self.opponent_name)
                };

                ui.heading(&winner);
                ui.add_space(10.0);
                ui.label(format!(
                    "Final Score: {} - {}",
                    self.left_score, self.right_score
                ));
                ui.add_space(20.0);

                if ui.button("Back to Menu").clicked() {
                    if let Some(opponent) = self.opponent_id {
                        world.resources.steam.close_session(opponent);
                    }
                    self.state = GameState::Menu;
                    self.role = None;
                    self.opponent_id = None;
                    self.reset_game();
                }
            });
        });
    }

    fn reset_game(&mut self) {
        self.left_paddle_y = 0.0;
        self.right_paddle_y = 0.0;
        self.left_score = 0;
        self.right_score = 0;
        self.reset_ball();
    }

    fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some((message.into(), Instant::now()));
    }
}
