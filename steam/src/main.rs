use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SteamDemo::default())
}

const SPACEWAR_STATS: &[&str] = &[
    "NumGames",
    "NumWins",
    "NumLosses",
    "FeetTraveled",
    "MaxFeetTraveled",
];

#[derive(Default)]
struct SteamDemo {
    initialized: bool,
    stats_requested: bool,
    selected_tab: Tab,
    status_message: Option<(String, Instant)>,
    selected_friend_index: Option<usize>,
    message_input: String,
    selected_session_index: Option<usize>,
    networking_initialized: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Tab {
    #[default]
    UserInfo,
    Achievements,
    Stats,
    Friends,
    Networking,
}

impl State for SteamDemo {
    fn title(&self) -> &str {
        "Steam Integration Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Space;
        world.resources.graphics.show_grid = false;

        spawn_sun_without_shadows(world);

        let camera_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | CAMERA,
            1,
        )[0];

        world.set_name(camera_entity, Name("Main Camera".to_string()));
        world.set_local_transform(
            camera_entity,
            LocalTransform {
                translation: nalgebra_glm::vec3(0.0, 2.0, 5.0),
                rotation: nalgebra_glm::Quat::identity(),
                ..Default::default()
            },
        );
        world.set_global_transform(camera_entity, GlobalTransform::default());
        world.set_local_transform_dirty(camera_entity, LocalTransformDirty);
        world.set_camera(
            camera_entity,
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

        world.resources.active_camera = Some(camera_entity);

        if let Err(error) = world.resources.steam.initialize() {
            tracing::error!("Steam initialization failed: {}", error);
        }
        self.initialized = world.resources.steam.is_initialized();
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if self.initialized && !self.stats_requested {
            world.resources.steam.request_stats();
            self.stats_requested = true;
        }

        if self.stats_requested && !world.resources.steam.stats_received {
            world.resources.steam.refresh_achievements();
            world.resources.steam.refresh_stats(SPACEWAR_STATS);
        }

        if self.initialized && !self.networking_initialized {
            world.resources.steam.setup_networking_callbacks();
            self.networking_initialized = true;
        }

        if self.initialized {
            world.resources.steam.receive_messages(0, 100);
            world.resources.steam.process_pending_requests();
        }
    }

    fn ui(&mut self, world: &mut World, ui: &egui::Context) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Steam Integration Demo");
            ui.separator();

            if !self.initialized {
                ui.colored_label(egui::Color32::RED, "Steam not initialized!");
                if let Some(error) = &world.resources.steam.initialization_error {
                    ui.label(format!("Error: {}", error));
                }
                ui.separator();
                ui.label("Make sure:");
                ui.label("  1. Steam client is running");
                ui.label("  2. steam_appid.txt exists with '480'");
                return;
            }

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::UserInfo, "User Info");
                ui.selectable_value(&mut self.selected_tab, Tab::Achievements, "Achievements");
                ui.selectable_value(&mut self.selected_tab, Tab::Stats, "Stats");
                ui.selectable_value(&mut self.selected_tab, Tab::Friends, "Friends");
                ui.selectable_value(&mut self.selected_tab, Tab::Networking, "Networking");
            });

            ui.separator();

            if let Some((message, time)) = &self.status_message {
                if time.elapsed().as_secs() < 3 {
                    ui.colored_label(egui::Color32::GREEN, message);
                    ui.separator();
                } else {
                    self.status_message = None;
                }
            }

            match self.selected_tab {
                Tab::UserInfo => self.render_user_info(world, ui),
                Tab::Achievements => self.render_achievements(world, ui),
                Tab::Stats => self.render_stats(world, ui),
                Tab::Friends => self.render_friends(world, ui),
                Tab::Networking => self.render_networking(world, ui),
            }
        });
    }
}

impl SteamDemo {
    fn render_user_info(&mut self, world: &World, ui: &mut egui::Ui) {
        ui.heading("User Information");
        ui.add_space(10.0);

        egui::Grid::new("user_info_grid")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                ui.label("Username:");
                ui.label(&world.resources.steam.user_name);
                ui.end_row();

                ui.label("Steam ID:");
                if let Some(steam_id) = world.resources.steam.user_id {
                    ui.label(format!("{}", steam_id.raw()));
                } else {
                    ui.label("N/A");
                }
                ui.end_row();

                ui.label("App ID:");
                ui.label(format!("{} (Spacewar)", world.resources.steam.app_id));
                ui.end_row();

                ui.label("Stats Received:");
                ui.label(if world.resources.steam.stats_received {
                    "Yes"
                } else {
                    "No"
                });
                ui.end_row();
            });
    }

    fn render_achievements(&mut self, world: &mut World, ui: &mut egui::Ui) {
        ui.heading("Achievements");
        ui.add_space(10.0);

        if !world.resources.steam.stats_received {
            ui.label("Loading achievements...");
            return;
        }

        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                world.resources.steam.refresh_achievements();
                self.set_status("Achievements refreshed");
            }
            if ui.button("Reset All").clicked() {
                if let Err(error) = world.resources.steam.reset_all_stats(true) {
                    tracing::error!("Failed to reset stats: {}", error);
                } else if let Err(error) = world.resources.steam.store_stats() {
                    tracing::error!("Failed to store stats: {}", error);
                } else {
                    world.resources.steam.refresh_achievements();
                    self.set_status("All achievements reset");
                }
            }
        });

        ui.separator();

        let achievements = world.resources.steam.achievements.clone();

        if achievements.is_empty() {
            ui.label("No achievements found.");
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for achievement in &achievements {
                    ui.horizontal(|ui| {
                        let status_icon = if achievement.achieved { "v" } else { " " };
                        let status_color = if achievement.achieved {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::GRAY
                        };

                        ui.colored_label(status_color, format!("[{}]", status_icon));
                        ui.label(&achievement.api_name);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if achievement.achieved {
                                if ui.button("Clear").clicked() {
                                    if let Err(error) = world
                                        .resources
                                        .steam
                                        .clear_achievement(&achievement.api_name)
                                    {
                                        tracing::error!("Failed to clear achievement: {}", error);
                                    } else if let Err(error) = world.resources.steam.store_stats() {
                                        tracing::error!("Failed to store stats: {}", error);
                                    } else {
                                        world.resources.steam.refresh_achievements();
                                        self.set_status(format!(
                                            "Cleared: {}",
                                            achievement.api_name
                                        ));
                                    }
                                }
                            } else if ui.button("Unlock").clicked() {
                                if let Err(error) = world
                                    .resources
                                    .steam
                                    .unlock_achievement(&achievement.api_name)
                                {
                                    tracing::error!("Failed to unlock achievement: {}", error);
                                } else if let Err(error) = world.resources.steam.store_stats() {
                                    tracing::error!("Failed to store stats: {}", error);
                                } else {
                                    world.resources.steam.refresh_achievements();
                                    self.set_status(format!("Unlocked: {}", achievement.api_name));
                                }
                            }
                        });
                    });
                    ui.separator();
                }
            });
    }

    fn render_stats(&mut self, world: &mut World, ui: &mut egui::Ui) {
        ui.heading("Stats");
        ui.add_space(10.0);

        if !world.resources.steam.stats_received {
            ui.label("Loading stats...");
            return;
        }

        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                world.resources.steam.refresh_stats(SPACEWAR_STATS);
                self.set_status("Stats refreshed");
            }
            if ui.button("Reset Stats Only").clicked() {
                if let Err(error) = world.resources.steam.reset_all_stats(false) {
                    tracing::error!("Failed to reset stats: {}", error);
                } else if let Err(error) = world.resources.steam.store_stats() {
                    tracing::error!("Failed to store stats: {}", error);
                } else {
                    world.resources.steam.refresh_stats(SPACEWAR_STATS);
                    self.set_status("Stats reset (achievements kept)");
                }
            }
        });

        ui.separator();

        let stats = world.resources.steam.stats.clone();

        if stats.is_empty() {
            ui.label("No stats found.");
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for stat in &stats {
                    ui.horizontal(|ui| {
                        ui.label(&stat.api_name);
                        ui.label(":");

                        match stat.value {
                            StatValue::Int(value) => {
                                ui.label(format!("{}", value));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("+10").clicked() {
                                            if let Err(error) = world
                                                .resources
                                                .steam
                                                .set_stat_int(&stat.api_name, value + 10)
                                            {
                                                tracing::error!("Failed to set stat: {}", error);
                                            } else if let Err(error) =
                                                world.resources.steam.store_stats()
                                            {
                                                tracing::error!("Failed to store stats: {}", error);
                                            } else {
                                                world.resources.steam.refresh_stats(SPACEWAR_STATS);
                                                self.set_status(format!("{} += 10", stat.api_name));
                                            }
                                        }
                                        if ui.button("+1").clicked() {
                                            if let Err(error) = world
                                                .resources
                                                .steam
                                                .set_stat_int(&stat.api_name, value + 1)
                                            {
                                                tracing::error!("Failed to set stat: {}", error);
                                            } else if let Err(error) =
                                                world.resources.steam.store_stats()
                                            {
                                                tracing::error!("Failed to store stats: {}", error);
                                            } else {
                                                world.resources.steam.refresh_stats(SPACEWAR_STATS);
                                                self.set_status(format!("{} += 1", stat.api_name));
                                            }
                                        }
                                    },
                                );
                            }
                            StatValue::Float(value) => {
                                ui.label(format!("{:.2}", value));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("+100.0").clicked() {
                                            if let Err(error) = world
                                                .resources
                                                .steam
                                                .set_stat_float(&stat.api_name, value + 100.0)
                                            {
                                                tracing::error!("Failed to set stat: {}", error);
                                            } else if let Err(error) =
                                                world.resources.steam.store_stats()
                                            {
                                                tracing::error!("Failed to store stats: {}", error);
                                            } else {
                                                world.resources.steam.refresh_stats(SPACEWAR_STATS);
                                                self.set_status(format!(
                                                    "{} += 100.0",
                                                    stat.api_name
                                                ));
                                            }
                                        }
                                        if ui.button("+10.0").clicked() {
                                            if let Err(error) = world
                                                .resources
                                                .steam
                                                .set_stat_float(&stat.api_name, value + 10.0)
                                            {
                                                tracing::error!("Failed to set stat: {}", error);
                                            } else if let Err(error) =
                                                world.resources.steam.store_stats()
                                            {
                                                tracing::error!("Failed to store stats: {}", error);
                                            } else {
                                                world.resources.steam.refresh_stats(SPACEWAR_STATS);
                                                self.set_status(format!(
                                                    "{} += 10.0",
                                                    stat.api_name
                                                ));
                                            }
                                        }
                                    },
                                );
                            }
                        }
                    });
                    ui.separator();
                }
            });
    }

    fn render_friends(&mut self, world: &mut World, ui: &mut egui::Ui) {
        ui.heading("Friends List");
        ui.add_space(10.0);

        if ui.button("Refresh").clicked() {
            world.resources.steam.refresh_friends();
            self.set_status("Friends list refreshed");
        }

        ui.separator();

        let friends = world.resources.steam.friends.clone();

        if friends.is_empty() {
            ui.label("No friends found or friends list not loaded.");
            ui.label("Click 'Refresh' to load friends.");
            return;
        }

        let online_count = friends.iter().filter(|f| f.state.is_online()).count();
        ui.label(format!(
            "{} friends ({} online)",
            friends.len(),
            online_count
        ));
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for friend in &friends {
                    ui.horizontal(|ui| {
                        let status_color = if friend.state.is_online() {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::GRAY
                        };

                        ui.colored_label(status_color, "[*]");
                        ui.label(&friend.name);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(status_color, friend.state.as_str());
                        });
                    });
                    ui.separator();
                }
            });
    }

    fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some((message.into(), Instant::now()));
    }

    fn render_networking(&mut self, world: &mut World, ui: &mut egui::Ui) {
        ui.heading("P2P Networking");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Select a friend to chat with:");
            if ui.button("Refresh Friends").clicked() {
                world.resources.steam.refresh_friends();
                self.set_status("Friends list refreshed");
            }
        });

        ui.add_space(5.0);

        let friends = world.resources.steam.friends.clone();
        let online_friends: Vec<_> = friends
            .iter()
            .enumerate()
            .filter(|(_, friend)| friend.state.is_online())
            .map(|(index, friend)| (index, friend.name.clone(), friend.steam_id))
            .collect();

        let selected_friend_name = self
            .selected_friend_index
            .and_then(|index| friends.get(index))
            .map(|friend| friend.name.clone())
            .unwrap_or_else(|| "Select a friend...".to_string());

        let active_sessions = world.resources.steam.active_sessions.clone();
        let user_name = world.resources.steam.user_name.clone();

        let mut start_chat_target: Option<(SteamId, String)> = None;

        if online_friends.is_empty() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "No online friends found. Click 'Refresh Friends' to load.",
            );
        } else {
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("")
                    .selected_text(&selected_friend_name)
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
                    && let Some((_, name, steam_id)) = online_friends
                        .iter()
                        .find(|(index, _, _)| *index == friend_index)
                {
                    let is_active = active_sessions.iter().any(|s| s.steam_id == *steam_id);

                    if !is_active && ui.button("Start Chat").clicked() {
                        start_chat_target = Some((*steam_id, name.clone()));
                    }
                }
            });
        }

        if let Some((steam_id, friend_name)) = start_chat_target {
            let hello_msg = format!("Hello from {}!", user_name);
            if let Err(error) =
                world
                    .resources
                    .steam
                    .send_message(steam_id, hello_msg.as_bytes(), 0, true)
            {
                tracing::error!("Failed to send message: {}", error);
                self.set_status(format!("Failed to connect: {}", error));
            } else {
                self.set_status(format!("Started chat with {}", friend_name));
            }
        }

        ui.separator();

        ui.heading("Active Sessions");
        ui.add_space(5.0);

        let sessions = world.resources.steam.active_sessions.clone();

        if sessions.is_empty() {
            ui.label("No active sessions.");
        } else {
            for (index, session) in sessions.iter().enumerate() {
                ui.horizontal(|ui| {
                    let is_selected = self.selected_session_index == Some(index);
                    let state_color = match session.state {
                        SessionState::Connected => egui::Color32::GREEN,
                        SessionState::Connecting => egui::Color32::YELLOW,
                        _ => egui::Color32::RED,
                    };

                    if ui
                        .selectable_label(
                            is_selected,
                            format!("{} ({:?})", session.name, session.state),
                        )
                        .clicked()
                    {
                        self.selected_session_index = Some(index);
                    }

                    ui.colored_label(state_color, "●");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            world.resources.steam.close_session(session.steam_id);
                            if self.selected_session_index == Some(index) {
                                self.selected_session_index = None;
                            }
                            self.set_status(format!("Closed session with {}", session.name));
                        }
                    });
                });
            }
        }

        ui.separator();

        ui.heading("Send Message");
        ui.add_space(5.0);

        let can_send = self
            .selected_session_index
            .and_then(|index| sessions.get(index))
            .map(|s| s.state == SessionState::Connected)
            .unwrap_or(false);

        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.message_input)
                    .hint_text("Type a message...")
                    .desired_width(300.0),
            );

            let send_clicked = ui
                .add_enabled(
                    can_send && !self.message_input.is_empty(),
                    egui::Button::new("Send"),
                )
                .clicked();
            let enter_pressed =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

            if (send_clicked || enter_pressed)
                && can_send
                && !self.message_input.is_empty()
                && let Some(session) = self
                    .selected_session_index
                    .and_then(|index| sessions.get(index))
            {
                let message = self.message_input.clone();
                if let Err(error) = world.resources.steam.send_message(
                    session.steam_id,
                    message.as_bytes(),
                    0,
                    true,
                ) {
                    tracing::error!("Failed to send message: {}", error);
                    self.set_status(format!("Send failed: {}", error));
                }
                self.message_input.clear();
            }
        });

        if !can_send && self.selected_session_index.is_some() {
            ui.colored_label(egui::Color32::YELLOW, "Session not connected yet.");
        } else if self.selected_session_index.is_none() && !sessions.is_empty() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "Select a session above to send messages.",
            );
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.heading("Messages");
            if ui.button("Clear").clicked() {
                world.resources.steam.clear_messages();
                self.set_status("Messages cleared");
            }
        });
        ui.add_space(5.0);

        let messages = world.resources.steam.received_messages.clone();

        egui::ScrollArea::vertical()
            .max_height(250.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if messages.is_empty() {
                    ui.label("No messages yet.");
                } else {
                    for message in &messages {
                        let text = String::from_utf8_lossy(&message.data);
                        let prefix = if message.is_outgoing { "[You]" } else { "" };
                        let color = if message.is_outgoing {
                            egui::Color32::LIGHT_BLUE
                        } else {
                            egui::Color32::WHITE
                        };

                        ui.horizontal(|ui| {
                            if message.is_outgoing {
                                ui.colored_label(color, format!("{} {}", prefix, text));
                            } else {
                                ui.colored_label(
                                    egui::Color32::GREEN,
                                    format!("[{}]", message.sender_name),
                                );
                                ui.colored_label(color, text.to_string());
                            }
                        });
                    }
                }
            });
    }
}
