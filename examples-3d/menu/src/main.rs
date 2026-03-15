use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::ui::state::UiStateTrait;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(MenuDemoState::default())
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum GameState {
    #[default]
    MainMenu,
    Settings,
    GraphicsSettings,
    AudioSettings,
    Playing,
    Paused,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum SettingsSource {
    #[default]
    MainMenu,
    Pause,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TransitionPhase {
    Idle,
    FadeOut { target: GameState, timer: f32 },
    FadeIn { timer: f32 },
}

const FADE_DURATION: f32 = 0.12;
const FADE_PEAK_ALPHA: f32 = 0.5;

struct GameSettings {
    sound_enabled: bool,
    music_enabled: bool,
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    resolution_index: usize,
    fullscreen: bool,
    vsync: bool,
    quality_index: usize,
    game_speed: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            music_enabled: true,
            master_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 1.0,
            resolution_index: 2,
            fullscreen: false,
            vsync: true,
            quality_index: 2,
            game_speed: 1.0,
        }
    }
}

struct MenuUi {
    transition_overlay: Entity,

    main_menu_screen: Entity,
    play_button: Entity,
    settings_button: Entity,
    quit_button: Entity,

    settings_screen: Entity,
    graphics_button: Entity,
    audio_button: Entity,
    settings_back_button: Entity,

    graphics_screen: Entity,
    resolution_dropdown: Entity,
    quality_dropdown: Entity,
    fullscreen_toggle: Entity,
    vsync_toggle: Entity,
    graphics_back_button: Entity,

    audio_screen: Entity,
    master_slider: Entity,
    music_slider: Entity,
    sfx_slider: Entity,
    sound_toggle: Entity,
    music_toggle: Entity,
    audio_back_button: Entity,

    playing_screen: Entity,

    pause_screen: Entity,
    resume_button: Entity,
    pause_settings_button: Entity,
    main_menu_button: Entity,

    quit_dialog: Entity,
    return_dialog: Entity,
}

struct MenuDemoState {
    game_state: GameState,
    settings_source: SettingsSource,
    settings: GameSettings,
    transition: TransitionPhase,

    camera_entity: Option<Entity>,
    game_entities: Vec<Entity>,
    game_rotation: f32,

    ui: Option<MenuUi>,
}

impl Default for MenuDemoState {
    fn default() -> Self {
        Self {
            game_state: GameState::default(),
            settings_source: SettingsSource::default(),
            settings: GameSettings::default(),
            transition: TransitionPhase::Idle,
            camera_entity: None,
            game_entities: Vec::new(),
            game_rotation: 0.0,
            ui: None,
        }
    }
}

fn build_menu_ui(world: &mut World, settings: &GameSettings) -> MenuUi {
    let placeholder = Entity {
        id: 0,
        generation: 0,
    };

    let font_size = 15.0;
    let title_font = 36.0;
    let heading_font = 24.0;
    let label_font = 14.0;

    let title_color = Vec4::new(0.92, 0.86, 0.65, 1.0);
    let subtitle_color = Vec4::new(0.5, 0.5, 0.55, 1.0);
    let label_color = Vec4::new(0.7, 0.7, 0.75, 1.0);
    let accent = Vec4::new(0.36, 0.52, 0.87, 1.0);
    let accent_dim = Vec4::new(0.24, 0.36, 0.62, 1.0);
    let panel_bg = Vec4::new(0.1, 0.1, 0.14, 0.92);
    let panel_border = Vec4::new(0.2, 0.2, 0.28, 0.6);
    let screen_bg = Vec4::new(0.02, 0.02, 0.04, 0.75);

    let mut tree = UiTreeBuilder::new(world);

    let transition_overlay = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_layer(UiLayer::Tooltips)
        .with_visible(false)
        .without_pointer_events()
        .done();

    let mut play_button = placeholder;
    let mut settings_button = placeholder;
    let mut quit_button = placeholder;

    let main_menu_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(screen_bg)
        .with_layer(UiLayer::FloatingPanels)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 45.0)),
                    Ab(Vec2::new(340.0, 360.0)),
                    Anchor::Center,
                )
                .with_rect(10.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 24.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_spacing(8.0);

                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 1.6)),
                        )
                        .with_text("NIGHTSHADE", title_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(title_color)
                        .without_pointer_events()
                        .done();

                    tree.add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.4)))
                        .with_text("Menu System Demo", font_size)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(subtitle_color)
                        .without_pointer_events()
                        .done();

                    tree.add_spacing(16.0);

                    play_button = tree.add_button_colored("PLAY", accent);
                    settings_button = tree.add_button("SETTINGS");

                    tree.add_spacing(4.0);

                    quit_button = tree.add_button("QUIT");
                })
                .done();
        })
        .done();

    let mut graphics_btn = placeholder;
    let mut audio_btn = placeholder;
    let mut settings_back = placeholder;

    let settings_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(screen_bg)
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 45.0)),
                    Ab(Vec2::new(340.0, 320.0)),
                    Anchor::Center,
                )
                .with_rect(10.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 24.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_spacing(4.0);

                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 1.6)),
                        )
                        .with_text("SETTINGS", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(title_color)
                        .without_pointer_events()
                        .done();

                    tree.add_spacing(8.0);

                    graphics_btn = tree.add_button("GRAPHICS");
                    audio_btn = tree.add_button("AUDIO");

                    tree.add_spacing(8.0);

                    settings_back = tree.add_button_colored("BACK", accent_dim);
                })
                .done();
        })
        .done();

    let mut resolution_dropdown = placeholder;
    let mut quality_dropdown = placeholder;
    let mut fullscreen_toggle = placeholder;
    let mut vsync_toggle = placeholder;
    let mut graphics_back = placeholder;

    let graphics_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(screen_bg)
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 45.0)),
                    Ab(Vec2::new(400.0, 420.0)),
                    Anchor::Center,
                )
                .with_rect(10.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 24.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_spacing(4.0);

                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 1.6)),
                        )
                        .with_text("GRAPHICS", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(title_color)
                        .without_pointer_events()
                        .done();

                    build_setting_row(tree, "Resolution", label_font, label_color, |tree| {
                        resolution_dropdown = tree.add_dropdown(
                            &[
                                "1280x720",
                                "1600x900",
                                "1920x1080",
                                "2560x1440",
                                "3840x2160",
                            ],
                            settings.resolution_index,
                        );
                    });

                    build_setting_row(tree, "Quality", label_font, label_color, |tree| {
                        quality_dropdown = tree.add_dropdown(
                            &["Low", "Medium", "High", "Ultra"],
                            settings.quality_index,
                        );
                    });

                    build_setting_row(tree, "Fullscreen", label_font, label_color, |tree| {
                        fullscreen_toggle = tree.add_toggle(settings.fullscreen);
                    });

                    build_setting_row(tree, "V-Sync", label_font, label_color, |tree| {
                        vsync_toggle = tree.add_toggle(settings.vsync);
                    });

                    tree.add_spacing(4.0);

                    graphics_back = tree.add_button_colored("BACK", accent_dim);
                })
                .done();
        })
        .done();

    let mut master_slider = placeholder;
    let mut music_slider = placeholder;
    let mut sfx_slider = placeholder;
    let mut sound_toggle = placeholder;
    let mut music_toggle = placeholder;
    let mut audio_back = placeholder;

    let audio_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(screen_bg)
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 45.0)),
                    Ab(Vec2::new(400.0, 480.0)),
                    Anchor::Center,
                )
                .with_rect(10.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 24.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_spacing(4.0);

                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 1.6)),
                        )
                        .with_text("AUDIO", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(title_color)
                        .without_pointer_events()
                        .done();

                    build_setting_row(tree, "Master Volume", label_font, label_color, |tree| {
                        master_slider = tree.add_slider_configured(
                            SliderConfig::new(0.0, 100.0, settings.master_volume * 100.0)
                                .suffix("%")
                                .precision(0),
                        );
                    });

                    build_setting_row(tree, "Music Volume", label_font, label_color, |tree| {
                        music_slider = tree.add_slider_configured(
                            SliderConfig::new(0.0, 100.0, settings.music_volume * 100.0)
                                .suffix("%")
                                .precision(0),
                        );
                    });

                    build_setting_row(tree, "SFX Volume", label_font, label_color, |tree| {
                        sfx_slider = tree.add_slider_configured(
                            SliderConfig::new(0.0, 100.0, settings.sfx_volume * 100.0)
                                .suffix("%")
                                .precision(0),
                        );
                    });

                    build_setting_row(tree, "Sound Enabled", label_font, label_color, |tree| {
                        sound_toggle = tree.add_toggle(settings.sound_enabled);
                    });

                    build_setting_row(tree, "Music Enabled", label_font, label_color, |tree| {
                        music_toggle = tree.add_toggle(settings.music_enabled);
                    });

                    tree.add_spacing(4.0);

                    audio_back = tree.add_button_colored("BACK", accent_dim);
                })
                .done();
        })
        .done();

    let playing_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 96.0)),
                    Ab(Vec2::new(240.0, 32.0)),
                    Anchor::Center,
                )
                .with_rect(6.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.4))
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_node()
                        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                        .with_text("Press P to pause", label_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(subtitle_color)
                        .without_pointer_events()
                        .done();
                })
                .done();
        })
        .done();

    let mut resume_btn = placeholder;
    let mut pause_settings_btn = placeholder;
    let mut main_menu_btn = placeholder;

    let pause_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.65))
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 45.0)),
                    Ab(Vec2::new(340.0, 320.0)),
                    Anchor::Center,
                )
                .with_rect(10.0, 1.0, panel_border)
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 24.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_spacing(4.0);

                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 1.6)),
                        )
                        .with_text("PAUSED", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(Vec4::new(0.9, 0.9, 0.95, 1.0))
                        .without_pointer_events()
                        .done();

                    tree.add_spacing(8.0);

                    resume_btn = tree.add_button_colored("RESUME", accent);
                    pause_settings_btn = tree.add_button("SETTINGS");

                    tree.add_spacing(4.0);

                    main_menu_btn = tree.add_button("MAIN MENU");
                })
                .done();
        })
        .done();

    let quit_dialog = tree.add_confirm_dialog("QUIT GAME", "Are you sure you want to quit?");
    let return_dialog =
        tree.add_confirm_dialog("RETURN TO MENU", "Are you sure? Progress will be lost.");

    tree.finish();

    MenuUi {
        transition_overlay,

        main_menu_screen,
        play_button,
        settings_button,
        quit_button,

        settings_screen,
        graphics_button: graphics_btn,
        audio_button: audio_btn,
        settings_back_button: settings_back,

        graphics_screen,
        resolution_dropdown,
        quality_dropdown,
        fullscreen_toggle,
        vsync_toggle,
        graphics_back_button: graphics_back,

        audio_screen,
        master_slider,
        music_slider,
        sfx_slider,
        sound_toggle,
        music_toggle,
        audio_back_button: audio_back,

        playing_screen,

        pause_screen,
        resume_button: resume_btn,
        pause_settings_button: pause_settings_btn,
        main_menu_button: main_menu_btn,

        quit_dialog,
        return_dialog,
    }
}

fn build_setting_row(
    tree: &mut UiTreeBuilder,
    label: &str,
    label_font: f32,
    label_color: Vec4,
    mut build_widget: impl FnMut(&mut UiTreeBuilder),
) {
    let row = tree
        .add_node()
        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 32.0)))
        .flow(FlowDirection::Horizontal, 0.0, 12.0)
        .without_pointer_events()
        .entity();

    tree.push_parent(row);

    tree.add_node()
        .flow_child(Ab(Vec2::new(130.0, 32.0)))
        .with_text(label, label_font)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .with_color::<UiBase>(label_color)
        .without_pointer_events()
        .done();

    let widget_container = tree
        .add_node()
        .flow_child(Ab(Vec2::new(210.0, 32.0)))
        .without_pointer_events()
        .entity();
    tree.push_parent(widget_container);
    build_widget(tree);
    tree.pop_parent();

    tree.pop_parent();
}

fn screen_for_state(ui: &MenuUi, state: GameState) -> Entity {
    match state {
        GameState::MainMenu => ui.main_menu_screen,
        GameState::Settings => ui.settings_screen,
        GameState::GraphicsSettings => ui.graphics_screen,
        GameState::AudioSettings => ui.audio_screen,
        GameState::Playing => ui.playing_screen,
        GameState::Paused => ui.pause_screen,
    }
}

impl MenuDemoState {
    fn start_transition(&mut self, world: &mut World, target: GameState) {
        if !matches!(self.transition, TransitionPhase::Idle) {
            return;
        }
        let ui = self.ui.as_ref().unwrap();
        world.ui_set_visible(ui.transition_overlay, true);
        self.transition = TransitionPhase::FadeOut {
            target,
            timer: FADE_DURATION,
        };
    }

    fn apply_state(&mut self, world: &mut World, state: GameState) {
        let ui = self.ui.as_ref().unwrap();
        let old_screen = screen_for_state(ui, self.game_state);
        let new_screen = screen_for_state(ui, state);
        world.ui_set_visible(old_screen, false);

        if state == GameState::Playing {
            self.setup_playing(world);
        } else if self.game_state == GameState::Playing {
            self.teardown_playing(world);
        }
        if state == GameState::MainMenu && !self.game_entities.is_empty() {
            self.cleanup_game(world);
        }

        world.ui_set_visible(new_screen, true);
        self.game_state = state;
    }

    fn update_transition(&mut self, world: &mut World, delta_time: f32) {
        let ui = self.ui.as_ref().unwrap();
        match self.transition {
            TransitionPhase::FadeOut { target, timer } => {
                let new_timer = timer - delta_time;
                let alpha = 1.0 - (new_timer / FADE_DURATION).clamp(0.0, 1.0);
                if let Some(color) = world.ui.get_ui_node_color_mut(ui.transition_overlay) {
                    color.colors[UiBase::INDEX] =
                        Some(Vec4::new(0.0, 0.0, 0.0, alpha * FADE_PEAK_ALPHA));
                }
                if new_timer <= 0.0 {
                    self.apply_state(world, target);
                    self.transition = TransitionPhase::FadeIn {
                        timer: FADE_DURATION,
                    };
                } else {
                    self.transition = TransitionPhase::FadeOut {
                        target,
                        timer: new_timer,
                    };
                }
            }
            TransitionPhase::FadeIn { timer } => {
                let new_timer = timer - delta_time;
                let alpha = (new_timer / FADE_DURATION).clamp(0.0, 1.0);
                if let Some(color) = world.ui.get_ui_node_color_mut(ui.transition_overlay) {
                    color.colors[UiBase::INDEX] =
                        Some(Vec4::new(0.0, 0.0, 0.0, alpha * FADE_PEAK_ALPHA));
                }
                if new_timer <= 0.0 {
                    world.ui_set_visible(ui.transition_overlay, false);
                    self.transition = TransitionPhase::Idle;
                } else {
                    self.transition = TransitionPhase::FadeIn { timer: new_timer };
                }
            }
            TransitionPhase::Idle => {}
        }
    }

    fn setup_playing(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = true;

        if let Some(camera) = self.camera_entity
            && let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera)
        {
            pan_orbit.enabled = true;
        }

        if self.game_entities.is_empty() {
            spawn_sun(world);

            let cube_entity = spawn_mesh(
                world,
                "Cube",
                Vec3::new(0.0, 0.5, 0.0),
                Vec3::new(1.0, 1.0, 1.0),
            );

            let cube_material = format!("GameCube_{}", cube_entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                cube_material.clone(),
                Material {
                    base_color: [0.4, 0.6, 0.9, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&cube_material)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            };
            world
                .core
                .set_material_ref(cube_entity, MaterialRef::new(cube_material));

            self.game_entities.push(cube_entity);
            self.game_rotation = 0.0;
        }
    }

    fn teardown_playing(&mut self, world: &mut World) {
        if let Some(camera) = self.camera_entity
            && let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera)
        {
            pan_orbit.enabled = false;
        }
    }

    fn cleanup_game(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;

        if let Some(camera) = self.camera_entity
            && let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera)
        {
            pan_orbit.enabled = false;
        }

        for entity in self.game_entities.drain(..) {
            world.queue_command(WorldCommand::DespawnRecursive { entity });
        }
    }

    fn read_settings_from_widgets(&mut self, world: &World) {
        let ui = self.ui.as_ref().unwrap();

        if let Some(data) = world.widget::<UiDropdownData>(ui.resolution_dropdown) {
            self.settings.resolution_index = data.selected_index;
        }
        if let Some(data) = world.widget::<UiDropdownData>(ui.quality_dropdown) {
            self.settings.quality_index = data.selected_index;
        }
        if let Some(data) = world.widget::<UiToggleData>(ui.fullscreen_toggle) {
            self.settings.fullscreen = data.value;
        }
        if let Some(data) = world.widget::<UiToggleData>(ui.vsync_toggle) {
            self.settings.vsync = data.value;
        }
        if let Some(data) = world.widget::<UiSliderData>(ui.master_slider) {
            self.settings.master_volume = data.value / 100.0;
        }
        if let Some(data) = world.widget::<UiSliderData>(ui.music_slider) {
            self.settings.music_volume = data.value / 100.0;
        }
        if let Some(data) = world.widget::<UiSliderData>(ui.sfx_slider) {
            self.settings.sfx_volume = data.value / 100.0;
        }
        if let Some(data) = world.widget::<UiToggleData>(ui.sound_toggle) {
            self.settings.sound_enabled = data.value;
        }
        if let Some(data) = world.widget::<UiToggleData>(ui.music_toggle) {
            self.settings.music_enabled = data.value;
        }
    }
}

impl State for MenuDemoState {
    fn title(&self) -> &str {
        "Menu Demo - Nightshade"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.retained_ui.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;

        let camera = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | CAMERA
                | PAN_ORBIT_CAMERA,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(camera) {
            *name = Name("Main Camera".to_string());
        }

        world.core.set_local_transform(
            camera,
            LocalTransform {
                translation: Vec3::new(0.0, 2.0, 8.0),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world
            .core
            .set_local_transform_dirty(camera, LocalTransformDirty);
        world
            .core
            .set_global_transform(camera, GlobalTransform::default());

        world.core.set_camera(
            camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 45.0_f32.to_radians(),
                    z_far: None,
                    z_near: 0.01,
                }),
                smoothing: None,
            },
        );

        world.core.set_pan_orbit_camera(
            camera,
            PanOrbitCamera {
                focus: Vec3::new(0.0, 0.5, 0.0),
                target_focus: Vec3::new(0.0, 0.5, 0.0),
                radius: 10.0,
                target_radius: 10.0,
                pitch: 0.25,
                target_pitch: 0.25,
                yaw: 0.0,
                target_yaw: 0.0,
                enabled: false,
                ..Default::default()
            },
        );

        self.camera_entity = Some(camera);
        world.resources.active_camera = Some(camera);

        self.ui = Some(build_menu_ui(world, &self.settings));
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta_time = world.resources.window.timing.delta_time;
        self.update_transition(world, delta_time);

        if !matches!(self.transition, TransitionPhase::Idle) {
            if self.game_state == GameState::Playing {
                pan_orbit_camera_system(world);
            }
            return;
        }

        let ui = self.ui.as_ref().unwrap();

        if let Some(data) = world.widget::<UiModalDialogData>(ui.quit_dialog)
            && let Some(result) = data.result
            && result
        {
            world.resources.window.should_exit = true;
            return;
        }

        if let Some(data) = world.widget::<UiModalDialogData>(ui.return_dialog)
            && let Some(result) = data.result
            && result
        {
            self.start_transition(world, GameState::MainMenu);
            return;
        }

        match self.game_state {
            GameState::MainMenu => {
                if world.ui_clicked(ui.play_button) {
                    self.start_transition(world, GameState::Playing);
                } else if world.ui_clicked(ui.settings_button) {
                    self.settings_source = SettingsSource::MainMenu;
                    self.start_transition(world, GameState::Settings);
                } else if world.ui_clicked(ui.quit_button) {
                    world.ui_show_modal(ui.quit_dialog);
                }
            }
            GameState::Settings => {
                if world.ui_clicked(ui.graphics_button) {
                    self.start_transition(world, GameState::GraphicsSettings);
                } else if world.ui_clicked(ui.audio_button) {
                    self.start_transition(world, GameState::AudioSettings);
                } else if world.ui_clicked(ui.settings_back_button) {
                    match self.settings_source {
                        SettingsSource::MainMenu => {
                            self.start_transition(world, GameState::MainMenu);
                        }
                        SettingsSource::Pause => {
                            self.start_transition(world, GameState::Paused);
                        }
                    }
                }
            }
            GameState::GraphicsSettings => {
                self.read_settings_from_widgets(world);
                let ui = self.ui.as_ref().unwrap();
                if world.ui_clicked(ui.graphics_back_button) {
                    self.start_transition(world, GameState::Settings);
                }
            }
            GameState::AudioSettings => {
                self.read_settings_from_widgets(world);
                let ui = self.ui.as_ref().unwrap();
                if world.ui_clicked(ui.audio_back_button) {
                    self.start_transition(world, GameState::Settings);
                }
            }
            GameState::Playing => {
                pan_orbit_camera_system(world);

                let delta_time = world.resources.window.timing.delta_time;
                self.game_rotation += delta_time * self.settings.game_speed;

                for &entity in &self.game_entities {
                    if let Some(transform) = world.core.get_local_transform_mut(entity) {
                        transform.rotation = nalgebra_glm::quat_angle_axis(
                            self.game_rotation,
                            &Vec3::new(0.0, 1.0, 0.0),
                        ) * nalgebra_glm::quat_angle_axis(
                            self.game_rotation * 0.7,
                            &Vec3::new(1.0, 0.0, 0.0),
                        );
                    }
                    world
                        .core
                        .set_local_transform_dirty(entity, LocalTransformDirty);
                }
            }
            GameState::Paused => {
                if world.ui_clicked(ui.resume_button) {
                    self.start_transition(world, GameState::Playing);
                } else if world.ui_clicked(ui.pause_settings_button) {
                    self.settings_source = SettingsSource::Pause;
                    self.start_transition(world, GameState::Settings);
                } else if world.ui_clicked(ui.main_menu_button) {
                    world.ui_show_modal(ui.return_dialog);
                }
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed || !matches!(self.transition, TransitionPhase::Idle) {
            return;
        }

        match key {
            KeyCode::KeyP => match self.game_state {
                GameState::Playing => {
                    self.start_transition(world, GameState::Paused);
                }
                GameState::Paused => {
                    self.start_transition(world, GameState::Playing);
                }
                _ => {}
            },
            KeyCode::Escape => match self.game_state {
                GameState::Settings => match self.settings_source {
                    SettingsSource::MainMenu => {
                        self.start_transition(world, GameState::MainMenu);
                    }
                    SettingsSource::Pause => {
                        self.start_transition(world, GameState::Paused);
                    }
                },
                GameState::GraphicsSettings | GameState::AudioSettings => {
                    self.start_transition(world, GameState::Settings);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
