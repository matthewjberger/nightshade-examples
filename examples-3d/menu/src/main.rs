use nightshade::ecs::material::resources::material_registry_insert;
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
    let font_size = 16.0;
    let title_font = 28.0;
    let heading_font = 22.0;
    let white = Vec4::new(1.0, 1.0, 1.0, 1.0);
    let gold = Vec4::new(1.0, 0.8, 0.2, 1.0);
    let dim = Vec4::new(0.6, 0.6, 0.6, 1.0);
    let panel_bg = Vec4::new(0.08, 0.08, 0.12, 0.85);

    let mut tree = UiTreeBuilder::new(world);

    let mut play_button = placeholder;
    let mut settings_button = placeholder;
    let mut quit_button = placeholder;

    let main_menu_screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.6))
        .with_layer(UiLayer::FloatingPanels)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 50.0)),
                    Ab(Vec2::new(400.0, 400.0)),
                    Anchor::Center,
                )
                .with_rect(8.0, 1.0, Vec4::new(0.3, 0.3, 0.4, 0.5))
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 20.0, 4.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, title_font * 2.0)),
                        )
                        .with_text("NIGHTSHADE", title_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(gold)
                        .without_pointer_events()
                        .done();

                    tree.add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.5)))
                        .with_text("Menu Demo", font_size)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(dim)
                        .without_pointer_events()
                        .done();

                    tree.add_spacing(20.0);

                    play_button = tree.add_button("PLAY");
                    settings_button = tree.add_button("SETTINGS");
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
        .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.6))
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 50.0)),
                    Ab(Vec2::new(400.0, 350.0)),
                    Anchor::Center,
                )
                .with_rect(8.0, 1.0, Vec4::new(0.3, 0.3, 0.4, 0.5))
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 20.0, 4.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 2.0)),
                        )
                        .with_text("SETTINGS", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(gold)
                        .without_pointer_events()
                        .done();

                    tree.add_spacing(16.0);

                    graphics_btn = tree.add_button("GRAPHICS");
                    audio_btn = tree.add_button("AUDIO");

                    tree.add_spacing(8.0);

                    settings_back = tree.add_button("BACK");
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
        .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.6))
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 50.0)),
                    Ab(Vec2::new(450.0, 420.0)),
                    Anchor::Center,
                )
                .with_rect(8.0, 1.0, Vec4::new(0.3, 0.3, 0.4, 0.5))
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 20.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 2.0)),
                        )
                        .with_text("GRAPHICS", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(gold)
                        .without_pointer_events()
                        .done();

                    tree.add_label("Resolution");
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

                    tree.add_label("Quality");
                    quality_dropdown = tree
                        .add_dropdown(&["Low", "Medium", "High", "Ultra"], settings.quality_index);

                    let row = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 30.0)))
                        .flow(FlowDirection::Horizontal, 0.0, 12.0)
                        .without_pointer_events()
                        .entity();

                    tree.push_parent(row);
                    tree.add_label("Fullscreen");
                    fullscreen_toggle = tree.add_toggle(settings.fullscreen);
                    tree.add_spacing(20.0);
                    tree.add_label("V-Sync");
                    vsync_toggle = tree.add_toggle(settings.vsync);
                    tree.pop_parent();

                    tree.add_spacing(8.0);

                    graphics_back = tree.add_button("BACK");
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
        .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.6))
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 50.0)),
                    Ab(Vec2::new(450.0, 480.0)),
                    Anchor::Center,
                )
                .with_rect(8.0, 1.0, Vec4::new(0.3, 0.3, 0.4, 0.5))
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 20.0, 6.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 2.0)),
                        )
                        .with_text("AUDIO", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(gold)
                        .without_pointer_events()
                        .done();

                    tree.add_label("Master Volume");
                    master_slider = tree.add_slider_configured(
                        SliderConfig::new(0.0, 100.0, settings.master_volume * 100.0)
                            .suffix("%")
                            .precision(0),
                    );

                    tree.add_label("Music Volume");
                    music_slider = tree.add_slider_configured(
                        SliderConfig::new(0.0, 100.0, settings.music_volume * 100.0)
                            .suffix("%")
                            .precision(0),
                    );

                    tree.add_label("SFX Volume");
                    sfx_slider = tree.add_slider_configured(
                        SliderConfig::new(0.0, 100.0, settings.sfx_volume * 100.0)
                            .suffix("%")
                            .precision(0),
                    );

                    let row = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 30.0)))
                        .flow(FlowDirection::Horizontal, 0.0, 12.0)
                        .without_pointer_events()
                        .entity();

                    tree.push_parent(row);
                    tree.add_label("Sound");
                    sound_toggle = tree.add_toggle(settings.sound_enabled);
                    tree.add_spacing(20.0);
                    tree.add_label("Music");
                    music_toggle = tree.add_toggle(settings.music_enabled);
                    tree.pop_parent();

                    tree.add_spacing(8.0);

                    audio_back = tree.add_button("BACK");
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
                    Rl(Vec2::new(50.0, 95.0)),
                    Ab(Vec2::new(300.0, 30.0)),
                    Anchor::Center,
                )
                .with_text("Press P to pause", font_size)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .with_color::<UiBase>(dim)
                .without_pointer_events()
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
        .with_color::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.7))
        .with_layer(UiLayer::FloatingPanels)
        .with_visible(false)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .window(
                    Rl(Vec2::new(50.0, 50.0)),
                    Ab(Vec2::new(400.0, 350.0)),
                    Anchor::Center,
                )
                .with_rect(8.0, 1.0, Vec4::new(0.3, 0.3, 0.4, 0.5))
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 20.0, 4.0)
                .without_pointer_events()
                .with_children(|tree| {
                    tree.add_node()
                        .flow_child(
                            Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, heading_font * 2.0)),
                        )
                        .with_text("PAUSED", heading_font)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_color::<UiBase>(white)
                        .without_pointer_events()
                        .done();

                    tree.add_spacing(16.0);

                    resume_btn = tree.add_button("RESUME");
                    pause_settings_btn = tree.add_button("SETTINGS");
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

impl MenuDemoState {
    fn show_screen(&mut self, world: &mut World, state: GameState) {
        let ui = self.ui.as_ref().unwrap();
        world.ui_set_visible(ui.main_menu_screen, state == GameState::MainMenu);
        world.ui_set_visible(ui.settings_screen, state == GameState::Settings);
        world.ui_set_visible(ui.graphics_screen, state == GameState::GraphicsSettings);
        world.ui_set_visible(ui.audio_screen, state == GameState::AudioSettings);
        world.ui_set_visible(ui.playing_screen, state == GameState::Playing);
        world.ui_set_visible(ui.pause_screen, state == GameState::Paused);
        self.game_state = state;
    }

    fn enter_playing(&mut self, world: &mut World) {
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

        self.show_screen(world, GameState::Playing);
    }

    fn leave_playing(&mut self, world: &mut World) {
        if let Some(camera) = self.camera_entity
            && let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera)
        {
            pan_orbit.enabled = false;
        }
    }

    fn return_to_main_menu(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;

        if let Some(camera) = self.camera_entity
            && let Some(pan_orbit) = world.core.get_pan_orbit_camera_mut(camera)
        {
            pan_orbit.enabled = false;
        }

        for entity in self.game_entities.drain(..) {
            world.queue_command(WorldCommand::DespawnRecursive { entity });
        }

        self.show_screen(world, GameState::MainMenu);
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
            self.return_to_main_menu(world);
            return;
        }

        match self.game_state {
            GameState::MainMenu => {
                let ui = self.ui.as_ref().unwrap();
                if world.ui_clicked(ui.play_button) {
                    self.enter_playing(world);
                } else if world.ui_clicked(ui.settings_button) {
                    self.settings_source = SettingsSource::MainMenu;
                    self.show_screen(world, GameState::Settings);
                } else if world.ui_clicked(ui.quit_button) {
                    world.ui_show_modal(ui.quit_dialog);
                }
            }
            GameState::Settings => {
                let ui = self.ui.as_ref().unwrap();
                if world.ui_clicked(ui.graphics_button) {
                    self.show_screen(world, GameState::GraphicsSettings);
                } else if world.ui_clicked(ui.audio_button) {
                    self.show_screen(world, GameState::AudioSettings);
                } else if world.ui_clicked(ui.settings_back_button) {
                    match self.settings_source {
                        SettingsSource::MainMenu => self.show_screen(world, GameState::MainMenu),
                        SettingsSource::Pause => self.show_screen(world, GameState::Paused),
                    }
                }
            }
            GameState::GraphicsSettings => {
                self.read_settings_from_widgets(world);
                let ui = self.ui.as_ref().unwrap();
                if world.ui_clicked(ui.graphics_back_button) {
                    self.show_screen(world, GameState::Settings);
                }
            }
            GameState::AudioSettings => {
                self.read_settings_from_widgets(world);
                let ui = self.ui.as_ref().unwrap();
                if world.ui_clicked(ui.audio_back_button) {
                    self.show_screen(world, GameState::Settings);
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
                let ui = self.ui.as_ref().unwrap();
                if world.ui_clicked(ui.resume_button) {
                    self.enter_playing(world);
                } else if world.ui_clicked(ui.pause_settings_button) {
                    self.settings_source = SettingsSource::Pause;
                    self.show_screen(world, GameState::Settings);
                } else if world.ui_clicked(ui.main_menu_button) {
                    world.ui_show_modal(ui.return_dialog);
                }
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            KeyCode::KeyP => match self.game_state {
                GameState::Playing => {
                    self.leave_playing(world);
                    self.show_screen(world, GameState::Paused);
                }
                GameState::Paused => {
                    self.enter_playing(world);
                }
                _ => {}
            },
            KeyCode::Escape => match self.game_state {
                GameState::Settings => match self.settings_source {
                    SettingsSource::MainMenu => self.show_screen(world, GameState::MainMenu),
                    SettingsSource::Pause => self.show_screen(world, GameState::Paused),
                },
                GameState::GraphicsSettings | GameState::AudioSettings => {
                    self.show_screen(world, GameState::Settings);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
