use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(RetainedUiDemo::default())
}

const CYAN: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.9, 1.0, 1.0);
const CYAN_DIM: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.5, 0.6, 1.0);
const MAGENTA: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(1.0, 0.0, 0.6, 1.0);
const MAGENTA_DIM: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.6, 0.0, 0.35, 1.0);
const WHITE: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0);
const LIGHT_GRAY: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.75, 0.75, 0.8, 1.0);
const DARK_PANEL_HOVER: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.08, 0.08, 0.14, 1.0);
const TRANSPARENT: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0);
const CARD_BG: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.05, 0.05, 0.09, 0.9);
const CARD_BORDER: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.4);
const ACTIVE_INDICATOR: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.9, 1.0, 0.8);
const INACTIVE_INDICATOR: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.9, 1.0, 0.0);

const SIDEBAR_PERCENT: f32 = 16.0;
const CONTENT_OFFSET_PERCENT: f32 = 16.5;

#[derive(Clone, Copy, PartialEq, Default)]
enum Screen {
    #[default]
    Dashboard,
    Systems,
    Settings,
    Widgets,
}

#[derive(Default)]
struct RetainedUiDemo {
    active_screen: Screen,
    nav_buttons: [Entity; 4],
    nav_indicators: [Entity; 4],
    screen_roots: [Entity; 4],

    fps_text_slot: usize,
    uptime_text_slot: usize,
    entity_count_text_slot: usize,

    brightness_slider: Entity,
    show_fps_toggle: Entity,
    fps_card: Entity,

    widget_button_primary: Entity,
    widget_button_success: Entity,
    widget_button_error: Entity,
    widget_click_count: u32,
    widget_click_count_text_slot: usize,
    slider_entity: Entity,
    toggle_entity: Entity,
    checkbox_entity: Entity,
    progress_entity: Entity,
    collapsing_entity: Entity,
    tab_bar_entity: Entity,
    text_input_entity: Entity,
    dropdown_entity: Entity,
    color_picker_entity: Entity,
    progress_value: f32,
    scroll_area_entity: Entity,
    menu_entity: Entity,
    theme_dropdown_entity: Entity,
    floating_panel_entity: Entity,
    status_text_slot: usize,

    pulse_entity: Entity,
    color_blend_time: f32,

    submit_input_entity: Entity,
    submit_log_text_slot: usize,
    focus_target_entity: Entity,
    focus_button_entity: Entity,
    disabled_button_entity: Entity,
    disable_toggle_entity: Entity,

    drag_value_entity: Entity,
    tree_view_entity: Entity,
    context_menu_entity: Entity,
    confirm_dialog_entity: Entity,
    confirm_trigger_button: Entity,
    prop_grid_x: Entity,
    prop_grid_y: Entity,
}

impl State for RetainedUiDemo {
    fn title(&self) -> &str {
        "NIGHTSHADE // RETAINED UI"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.retained_ui.enabled = true;
        world.resources.graphics.clear_color = [0.02, 0.02, 0.04, 1.0];
        world.resources.retained_ui.background_color =
            Some(nalgebra_glm::Vec4::new(0.02, 0.02, 0.04, 1.0));

        let camera = spawn_ortho_camera(world, nalgebra_glm::Vec2::new(0.0, 0.0));
        world.resources.active_camera = Some(camera);

        self.fps_text_slot = world.resources.text_cache.add_text("60");
        self.uptime_text_slot = world.resources.text_cache.add_text("0:00");
        self.entity_count_text_slot = world.resources.text_cache.add_text("0");
        self.widget_click_count_text_slot = world.resources.text_cache.add_text("0");
        self.status_text_slot = world.resources.text_cache.add_text("");
        self.submit_log_text_slot = world.resources.text_cache.add_text("Press Enter to submit");

        let mut tree = UiTreeBuilder::new(world);

        let root_panel = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .without_pointer_events()
            .entity();

        tree.push_parent(root_panel);

        build_top_bar(&mut tree);
        self.build_sidebar(&mut tree);

        let content_area = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(CONTENT_OFFSET_PERCENT, 0.0))
                    + Ab(nalgebra_glm::Vec2::new(0.0, 50.0)),
                Ab(nalgebra_glm::Vec2::new(0.0, -40.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .without_pointer_events()
            .entity();

        tree.push_parent(content_area);

        self.screen_roots[0] = self.build_dashboard_screen(&mut tree);
        self.screen_roots[1] = self.build_systems_screen(&mut tree);
        self.screen_roots[2] = self.build_settings_screen(&mut tree);
        self.screen_roots[3] = self.build_widgets_screen(&mut tree);

        tree.pop_parent();

        build_bottom_bar(&mut tree);

        tree.pop_parent();

        tree.finish();
    }

    fn run_systems(&mut self, world: &mut World) {
        let fps = world.resources.window.timing.frames_per_second;
        world
            .resources
            .text_cache
            .set_text(self.fps_text_slot, format!("{:.0}", fps));

        let uptime_ms = world.resources.window.timing.uptime_milliseconds;
        let seconds = uptime_ms / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        world.resources.text_cache.set_text(
            self.uptime_text_slot,
            format!("{}:{:02}", minutes, remaining_seconds),
        );

        let entity_count = world.entity_count();
        world
            .resources
            .text_cache
            .set_text(self.entity_count_text_slot, format!("{}", entity_count));

        for index in 0..4 {
            if let Some(interaction) = world.get_ui_node_interaction(self.nav_buttons[index])
                && interaction.clicked
            {
                let new_screen = match index {
                    0 => Screen::Dashboard,
                    1 => Screen::Systems,
                    2 => Screen::Settings,
                    3 => Screen::Widgets,
                    _ => Screen::Dashboard,
                };

                if self.active_screen != new_screen {
                    self.active_screen = new_screen;
                    self.update_screen_visibility(world);
                }
            }
        }

        self.handle_settings_interactions(world);
        self.handle_widget_interactions(world);
        self.update_animated_elements(world);
    }
}

fn build_top_bar(tree: &mut UiTreeBuilder) {
    let top_bar = tree
        .add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Ab(nalgebra_glm::Vec2::new(0.0, 48.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.98))
        .with_depth(UiDepthMode::Set(10.0))
        .entity();

    tree.push_parent(top_bar);

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(20.0, 24.0)),
            Ab(nalgebra_glm::Vec2::new(300.0, 30.0)),
            Anchor::CenterLeft,
        )
        .with_text("NIGHTSHADE", 22.0)
        .with_text_outline(nalgebra_glm::Vec4::new(0.0, 0.4, 0.5, 1.0), 1.5)
        .with_color::<UiBase>(CYAN)
        .without_pointer_events();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(180.0, 24.0)),
            Ab(nalgebra_glm::Vec2::new(200.0, 22.0)),
            Anchor::CenterLeft,
        )
        .with_text("// NEON", 14.0)
        .with_color::<UiBase>(MAGENTA)
        .without_pointer_events();

    tree.add_node()
        .window(
            Rl(nalgebra_glm::Vec2::new(98.0, 50.0)) + Ab(nalgebra_glm::Vec2::new(-10.0, 0.0)),
            Ab(nalgebra_glm::Vec2::new(120.0, 20.0)),
            Anchor::CenterRight,
        )
        .with_text("v0.7.0", 12.0)
        .with_color::<UiBase>(CYAN_DIM)
        .with_tooltip("Engine version")
        .without_pointer_events();

    tree.add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, 46.0)),
            Ab(nalgebra_glm::Vec2::new(0.0, 48.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(CYAN_DIM)
        .without_pointer_events();

    tree.pop_parent();
}

fn build_bottom_bar(tree: &mut UiTreeBuilder) {
    let bottom_bar = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(0.0, -38.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 100.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.98))
        .with_depth(UiDepthMode::Set(10.0))
        .without_pointer_events()
        .entity();

    tree.push_parent(bottom_bar);

    tree.add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Ab(nalgebra_glm::Vec2::new(0.0, 2.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
        )
        .with_rect(0.0, 0.0, TRANSPARENT)
        .with_color::<UiBase>(MAGENTA_DIM)
        .without_pointer_events();

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
            Ab(nalgebra_glm::Vec2::new(440.0, 20.0)),
            Anchor::CenterLeft,
        )
        .with_text("NIGHTSHADE ENGINE  //  RETAINED LAYOUT UI DEMO", 11.0)
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.4, 0.4, 0.5, 1.0))
        .without_pointer_events();

    tree.pop_parent();
}

impl RetainedUiDemo {
    fn build_sidebar(&mut self, tree: &mut UiTreeBuilder) {
        let sidebar = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(0.0, 50.0)),
                Rl(nalgebra_glm::Vec2::new(SIDEBAR_PERCENT, 100.0))
                    + Ab(nalgebra_glm::Vec2::new(0.0, -40.0)),
            )
            .with_rect(0.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.95))
            .without_pointer_events()
            .entity();

        tree.push_parent(sidebar);

        let nav_labels = ["DASHBOARD", "SYSTEMS", "SETTINGS", "WIDGETS"];
        let nav_tooltips = [
            "System overview and metrics",
            "Toggle active ECS systems",
            "Display and theme settings",
            "Widget gallery and demos",
        ];

        for (index, label) in nav_labels.iter().enumerate() {
            let y_offset = 20.0 + index as f32 * 52.0;

            let indicator = tree
                .add_node()
                .window(
                    Ab(nalgebra_glm::Vec2::new(0.0, y_offset)),
                    Ab(nalgebra_glm::Vec2::new(3.0, 40.0)),
                    Anchor::TopLeft,
                )
                .with_rect(0.0, 0.0, TRANSPARENT)
                .with_color::<UiBase>(if index == 0 {
                    ACTIVE_INDICATOR
                } else {
                    INACTIVE_INDICATOR
                })
                .without_pointer_events()
                .done();

            self.nav_indicators[index] = indicator;

            let button = tree
                .add_node()
                .window(
                    Ab(nalgebra_glm::Vec2::new(8.0, y_offset)),
                    Ab(nalgebra_glm::Vec2::new(0.0, 40.0))
                        + Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(-12.0, 0.0)),
                    Anchor::TopLeft,
                )
                .with_hover_layout(UiLayoutType::Window(WindowLayout {
                    position: Ab(nalgebra_glm::Vec2::new(12.0, y_offset)).into(),
                    size: Ab(nalgebra_glm::Vec2::new(0.0, 40.0))
                        + Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(-12.0, 0.0)),
                    anchor: Anchor::TopLeft,
                }))
                .with_interaction()
                .with_tooltip(nav_tooltips[index])
                .with_cursor_icon(winit::window::CursorIcon::Pointer)
                .with_rect(4.0, 0.0, TRANSPARENT)
                .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.05, 0.05, 0.09, 0.0))
                .with_color::<UiHover>(DARK_PANEL_HOVER)
                .with_color::<UiPressed>(nalgebra_glm::Vec4::new(0.04, 0.04, 0.08, 1.0))
                .with_transition::<UiHover>(12.0, 6.0)
                .with_transition::<UiPressed>(20.0, 10.0)
                .with_children(|tree| {
                    tree.add_node()
                        .window(
                            Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                            Ab(nalgebra_glm::Vec2::new(180.0, 30.0)),
                            Anchor::Center,
                        )
                        .with_text(label, 14.0)
                        .with_color::<UiBase>(if index == 0 { CYAN } else { LIGHT_GRAY })
                        .without_pointer_events();
                })
                .done();

            self.nav_buttons[index] = button;
        }

        tree.add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(-2.0, 0.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .with_rect(0.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.5))
            .without_pointer_events();

        tree.pop_parent();
    }

    fn build_dashboard_screen(&mut self, tree: &mut UiTreeBuilder) -> Entity {
        let screen = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .with_intro(UiAnimationType::Fade, 0.2)
            .without_pointer_events()
            .entity();

        tree.push_parent(screen);

        tree.add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
                Ab(nalgebra_glm::Vec2::new(300.0, 28.0)),
                Anchor::TopLeft,
            )
            .with_text("SYSTEM OVERVIEW", 18.0)
            .with_color::<UiBase>(CYAN)
            .without_pointer_events();

        let card_data = [
            ("FPS", self.fps_text_slot, CYAN),
            ("ENTITIES", self.entity_count_text_slot, MAGENTA),
            (
                "UPTIME",
                self.uptime_text_slot,
                nalgebra_glm::Vec4::new(0.4, 1.0, 0.4, 1.0),
            ),
        ];

        for (index, (label, text_slot, accent_color)) in card_data.iter().enumerate() {
            let x_percent = 2.0 + index as f32 * 33.0;

            let card = tree
                .add_node()
                .boundary(
                    Rl(nalgebra_glm::Vec2::new(x_percent, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 60.0)),
                    Rl(nalgebra_glm::Vec2::new(x_percent + 31.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 170.0)),
                )
                .with_rect(6.0, 1.0, CARD_BORDER)
                .with_color::<UiBase>(CARD_BG)
                .with_tooltip(match index {
                    0 => "Current frames per second",
                    1 => "Total active ECS entities",
                    _ => "Time since application start",
                })
                .entity();

            tree.push_parent(card);

            tree.add_node()
                .boundary(
                    Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                    Ab(nalgebra_glm::Vec2::new(0.0, 3.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
                )
                .with_rect(6.0, 0.0, TRANSPARENT)
                .with_color::<UiBase>(*accent_color)
                .without_pointer_events();

            tree.add_node()
                .window(
                    Ab(nalgebra_glm::Vec2::new(16.0, 22.0)),
                    Ab(nalgebra_glm::Vec2::new(150.0, 20.0)),
                    Anchor::TopLeft,
                )
                .with_text(label, 12.0)
                .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0))
                .without_pointer_events();

            tree.add_node()
                .window(
                    Ab(nalgebra_glm::Vec2::new(16.0, 55.0)),
                    Ab(nalgebra_glm::Vec2::new(200.0, 42.0)),
                    Anchor::TopLeft,
                )
                .with_text_slot(*text_slot, 36.0)
                .with_color::<UiBase>(WHITE)
                .without_pointer_events();

            if index == 0 {
                self.fps_card = card;
            }

            tree.pop_parent();
        }

        self.pulse_entity = tree
            .add_node()
            .window(
                Rl(nalgebra_glm::Vec2::new(96.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 68.0)),
                Ab(nalgebra_glm::Vec2::new(10.0, 10.0)),
                Anchor::Center,
            )
            .with_rect(5.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.0, 1.0, 0.5, 1.0))
            .with_tooltip("Live status indicator (animated)")
            .done();

        tree.add_node()
            .solid(
                Ab(nalgebra_glm::Vec2::new(16.0, 9.0)),
                ScalingMode::Fit,
                nalgebra_glm::Vec2::new(0.0, 1.0),
            )
            .with_rect(4.0, 1.0, CARD_BORDER)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.6))
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Rl(nalgebra_glm::Vec2::new(50.0, 50.0)),
                        Ab(nalgebra_glm::Vec2::new(200.0, 24.0)),
                        Anchor::Center,
                    )
                    .with_text("16:9 VIEWPORT", 14.0)
                    .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.3, 0.3, 0.4, 1.0))
                    .without_pointer_events();
            });

        tree.pop_parent();
        screen
    }

    fn build_settings_screen(&mut self, tree: &mut UiTreeBuilder) -> Entity {
        let screen = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .with_visible(false)
            .with_intro(UiAnimationType::Fade, 0.2)
            .without_pointer_events()
            .entity();

        tree.push_parent(screen);

        let settings_card = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(10.0, 10.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 100.0))
                    + Ab(nalgebra_glm::Vec2::new(-10.0, -10.0)),
            )
            .with_rect(6.0, 1.0, CARD_BORDER)
            .with_color::<UiBase>(CARD_BG)
            .entity();

        tree.push_parent(settings_card);

        let scroll = tree.add_scroll_area_fill(12.0, 6.0);
        let content = tree.world_mut().ui_scroll_area_content(scroll).unwrap();
        tree.push_parent(content);

        tree.add_heading("Display");
        tree.add_separator();

        tree.add_label("Brightness");
        self.brightness_slider = tree.add_slider(0.0, 100.0, 50.0);

        tree.add_spacing(4.0);
        tree.add_label("UI Scale");
        tree.add_dropdown(&["75%", "100%", "125%", "150%"], 1);

        tree.add_spacing(4.0);
        tree.add_label("V-Sync");
        tree.add_toggle(true);

        tree.add_spacing(4.0);
        tree.add_label("Show FPS Counter");
        self.show_fps_toggle = tree.add_toggle(true);

        tree.add_spacing(12.0);
        tree.add_heading("Theme");
        tree.add_separator();
        self.theme_dropdown_entity = tree.add_theme_dropdown();

        tree.pop_parent();
        tree.pop_parent();

        tree.pop_parent();
        screen
    }

    fn build_systems_screen(&mut self, tree: &mut UiTreeBuilder) -> Entity {
        let screen = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .with_visible(false)
            .with_intro(UiAnimationType::Fade, 0.2)
            .without_pointer_events()
            .entity();

        tree.push_parent(screen);

        let systems_card = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(10.0, 10.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 100.0))
                    + Ab(nalgebra_glm::Vec2::new(-10.0, -10.0)),
            )
            .with_rect(6.0, 1.0, CARD_BORDER)
            .with_color::<UiBase>(CARD_BG)
            .entity();

        tree.push_parent(systems_card);

        let scroll = tree.add_scroll_area_fill(8.0, 0.0);
        let scroll_content = tree.world_mut().ui_scroll_area_content(scroll).unwrap();
        tree.push_parent(scroll_content);

        tree.add_heading("Active Systems");
        tree.add_separator();

        let system_names = [
            "Transform Propagation",
            "Camera Update",
            "Sprite Rendering",
            "Text Sync",
            "Animation Player",
            "Particle Update",
            "Physics Step",
            "Collision Detection",
            "Audio Mixer",
            "NavMesh Agent",
            "UI Layout Compute",
            "UI Picking",
            "UI State Update",
            "UI Color Blend",
            "UI Render Sync",
            "Event Bus Dispatch",
            "Input Reset",
            "Deferred Commands",
        ];

        for (index, name) in system_names.iter().enumerate() {
            let initially_on = !(5..10).contains(&index);
            tree.add_checkbox(name, initially_on);
        }

        tree.pop_parent();
        tree.pop_parent();

        tree.pop_parent();
        screen
    }

    fn build_widgets_screen(&mut self, tree: &mut UiTreeBuilder) -> Entity {
        let screen = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .with_visible(false)
            .with_intro(UiAnimationType::Fade, 0.2)
            .without_pointer_events()
            .entity();

        tree.push_parent(screen);

        let left_column = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(10.0, 10.0)),
                Rl(nalgebra_glm::Vec2::new(50.0, 0.0))
                    + Ab(nalgebra_glm::Vec2::new(-5.0, -10.0))
                    + Rl(nalgebra_glm::Vec2::new(0.0, 100.0)),
            )
            .with_rect(6.0, 1.0, CARD_BORDER)
            .with_color::<UiBase>(CARD_BG)
            .entity();

        tree.push_parent(left_column);

        let left_scroll = tree.add_scroll_area_fill(12.0, 6.0);
        let left_content = tree.world_mut().ui_scroll_area_content(left_scroll);
        let left_content_entity = left_content.unwrap();
        tree.push_parent(left_content_entity);

        tree.add_heading("Value Widgets");
        tree.add_separator();

        tree.add_label("Buttons");

        self.widget_button_primary = tree.add_button("Primary Button");

        let theme = tree
            .world_mut()
            .resources
            .retained_ui
            .theme_state
            .active_theme();
        let success_color = theme.success_color;
        let error_color = theme.error_color;

        self.widget_button_success = tree.add_button_colored("Success Action", success_color);
        self.widget_button_error = tree.add_button_colored("Danger Action", error_color);

        let click_count_row = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 24.0)),
            )
            .flow(FlowDirection::Horizontal, 0.0, 8.0)
            .entity();

        tree.push_parent(click_count_row);

        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(110.0, 24.0)))
            .with_text("Click count:", 14.0)
            .with_color::<UiBase>(LIGHT_GRAY)
            .without_pointer_events()
            .done();

        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(50.0, 24.0)))
            .with_text_slot(self.widget_click_count_text_slot, 14.0)
            .with_color::<UiBase>(CYAN)
            .without_pointer_events()
            .done();

        tree.pop_parent();

        tree.add_spacing(4.0);
        tree.add_label("Slider");
        self.slider_entity = tree.add_slider(0.0, 100.0, 50.0);

        tree.add_spacing(4.0);
        tree.add_label("Drag Value");
        self.drag_value_entity =
            tree.add_drag_value_configured(DragValueConfig::new(0.0, 10.0, 0.5).speed(0.01));

        tree.add_spacing(4.0);
        tree.add_label("Toggle");
        self.toggle_entity = tree.add_toggle(false);

        tree.add_spacing(4.0);
        tree.add_label("Checkbox");
        self.checkbox_entity = tree.add_checkbox("Enable notifications", false);

        tree.add_spacing(4.0);
        tree.add_label("Radio Buttons");
        tree.add_radio("Option A", 1, 0);
        tree.add_radio("Option B", 1, 1);
        tree.add_radio("Option C", 1, 2);

        tree.add_spacing(4.0);
        tree.add_label("Progress Bar");
        self.progress_entity = tree.add_progress_bar(0.0);

        tree.add_spacing(4.0);
        tree.add_label("Text Input");
        self.text_input_entity = tree.add_text_input("Type here...");

        tree.add_spacing(8.0);
        tree.add_heading("New Features");
        tree.add_separator();

        tree.add_label("Submit Detection (Enter key)");
        self.submit_input_entity = tree.add_text_input("Type and press Enter...");
        tree.add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 24.0)),
            )
            .with_text_slot(self.submit_log_text_slot, 12.0)
            .with_color::<UiBase>(CYAN_DIM)
            .without_pointer_events()
            .done();

        tree.add_spacing(4.0);
        tree.add_label("Programmatic Focus");
        self.focus_target_entity = tree.add_text_input("Focus target");
        self.focus_button_entity = tree.add_button("Focus the input above");

        tree.add_spacing(4.0);
        tree.add_label("Disabled State");
        self.disabled_button_entity = tree.add_button("I can be disabled");
        self.disable_toggle_entity = tree.add_toggle(false);
        tree.add_label("Toggle to disable button");

        tree.pop_parent();
        tree.pop_parent();

        let right_column = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(50.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(5.0, 10.0)),
                Ab(nalgebra_glm::Vec2::new(-10.0, -10.0))
                    + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .with_rect(6.0, 1.0, CARD_BORDER)
            .with_color::<UiBase>(CARD_BG)
            .entity();

        tree.push_parent(right_column);

        let right_scroll = tree.add_scroll_area_fill(12.0, 6.0);
        let right_content = tree.world_mut().ui_scroll_area_content(right_scroll);
        let right_content_entity = right_content.unwrap();
        tree.push_parent(right_content_entity);

        tree.add_heading("Layout & Compound Widgets");
        tree.add_separator();

        tree.add_label("Collapsing Header");
        self.collapsing_entity = tree.add_collapsing_header("Click to expand", true);

        let collapsing_content = tree
            .world_mut()
            .ui_collapsing_header_content(self.collapsing_entity);
        if let Some(content_entity) = collapsing_content {
            tree.push_parent(content_entity);
            tree.add_label("This content is inside the collapsing header.");
            tree.add_label("It can be toggled by clicking the header above.");
            tree.pop_parent();
        }

        tree.add_spacing(4.0);
        tree.add_label("Tab Bar");
        self.tab_bar_entity = tree.add_tab_bar(&["General", "Audio", "Display"], 0);

        tree.add_spacing(4.0);
        tree.add_label("Dropdown");
        self.dropdown_entity = tree.add_dropdown(&["Low", "Medium", "High", "Ultra"], 1);

        tree.add_spacing(4.0);
        tree.add_label("Menu");
        self.menu_entity = tree.add_menu("Actions", &["New", "Open", "Save", "Export"]);

        tree.add_spacing(4.0);
        tree.add_label("Color Picker");
        self.color_picker_entity =
            tree.add_color_picker(nalgebra_glm::Vec4::new(0.3, 0.5, 0.9, 1.0));

        tree.add_spacing(8.0);
        tree.add_heading("Editor Widgets");
        tree.add_separator();

        tree.add_label("Selectable Labels");
        tree.add_selectable_label("Renderer: wgpu", Some(1));
        tree.add_selectable_label("Audio: disabled", Some(1));
        tree.add_selectable_label("Physics: rapier", Some(1));

        tree.add_spacing(4.0);
        tree.add_label("Property Grid");
        let grid = tree.add_property_grid(60.0);
        let section = tree.add_property_section(grid, "Transform");
        let area = tree.add_property_row(grid, section, "X");
        tree.push_parent(area);
        self.prop_grid_x = tree.add_drag_value(-10.0, 10.0, 1.0);
        tree.pop_parent();
        let area = tree.add_property_row(grid, section, "Y");
        tree.push_parent(area);
        self.prop_grid_y = tree.add_drag_value(-10.0, 10.0, 2.0);
        tree.pop_parent();

        tree.add_spacing(4.0);
        tree.add_label("Tree View");
        self.tree_view_entity = tree.add_tree_view(false);
        let tv_content = tree
            .world_mut()
            .ui_tree_view_content(self.tree_view_entity)
            .unwrap();
        let root_node = tree.add_tree_node(self.tree_view_entity, tv_content, "Project", 0, 0);
        tree.world_mut().ui_tree_node_set_expanded(root_node, true);
        let root_children = tree.world_mut().ui_tree_node_children(root_node).unwrap();
        tree.add_tree_node(self.tree_view_entity, root_children, "Assets", 1, 1);
        tree.add_tree_node(self.tree_view_entity, root_children, "Scripts", 1, 2);
        tree.add_tree_node(self.tree_view_entity, root_children, "Scenes", 1, 3);

        tree.add_spacing(4.0);
        tree.add_label("Confirm Dialog");
        self.confirm_trigger_button = tree.add_button("Show Confirm Dialog");

        tree.add_spacing(4.0);
        tree.add_label("Scroll Area");
        self.scroll_area_entity = tree.add_scroll_area(nalgebra_glm::Vec2::new(0.0, 120.0));

        let scroll_content = tree
            .world_mut()
            .ui_scroll_area_content(self.scroll_area_entity);
        if let Some(scroll_content_entity) = scroll_content {
            tree.push_parent(scroll_content_entity);
            for index in 0..20 {
                tree.add_label(&format!("Scrollable item {}", index + 1));
            }
            tree.pop_parent();
        }

        tree.add_spacing(4.0);
        tree.add_separator();
        tree.add_label("Widget Status:");

        tree.add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 60.0)),
            )
            .with_text_slot(self.status_text_slot, 12.0)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .with_color::<UiBase>(CYAN_DIM)
            .without_pointer_events()
            .done();

        tree.pop_parent();
        tree.pop_parent();

        self.floating_panel_entity = tree.add_floating_panel(
            "Floating Panel",
            Rect {
                min: nalgebra_glm::Vec2::new(100.0, 150.0),
                max: nalgebra_glm::Vec2::new(350.0, 350.0),
            },
        );

        let panel_content = tree
            .world_mut()
            .ui_panel_content(self.floating_panel_entity);
        if let Some(panel_content_entity) = panel_content {
            tree.push_parent(panel_content_entity);
            tree.add_label("Drag the header to move.");
            tree.add_label("Drag to dock indicators to dock.");
            tree.add_label("Resize from edges and corners.");
            tree.add_button("Panel Button");
            tree.pop_parent();
        }

        self.confirm_dialog_entity =
            tree.add_confirm_dialog("Confirm Action", "Are you sure you want to proceed?");

        self.context_menu_entity = tree.add_context_menu(&[
            ("Cut", Some("Ctrl+X")),
            ("Copy", Some("Ctrl+C")),
            ("Paste", Some("Ctrl+V")),
        ]);

        tree.pop_parent();
        screen
    }

    fn update_screen_visibility(&self, world: &mut World) {
        let screens = [
            Screen::Dashboard,
            Screen::Systems,
            Screen::Settings,
            Screen::Widgets,
        ];

        for (index, screen) in screens.iter().enumerate() {
            let is_active = *screen == self.active_screen;
            if is_active {
                world.ui_set_visible(self.screen_roots[index], true);
            } else if let Some(node) = world.get_ui_layout_node_mut(self.screen_roots[index]) {
                node.visible = false;
            }

            if let Some(color) = world.get_ui_node_color_mut(self.nav_indicators[index]) {
                color.computed_color = if is_active {
                    ACTIVE_INDICATOR
                } else {
                    INACTIVE_INDICATOR
                };
            }
        }
    }

    fn handle_settings_interactions(&mut self, world: &mut World) {
        if world.ui_slider_changed(self.brightness_slider) {
            let brightness = world.ui_slider_value(self.brightness_slider) / 100.0;
            let base = 0.02;
            let value = base + brightness * 0.06;
            world.resources.graphics.clear_color = [value, value, value + 0.02, 1.0];
            world.resources.retained_ui.background_color =
                Some(nalgebra_glm::Vec4::new(value, value, value + 0.02, 1.0));
        }

        if world.ui_toggle_changed(self.show_fps_toggle) {
            let show = world.ui_toggle_value(self.show_fps_toggle);
            if let Some(node) = world.get_ui_layout_node_mut(self.fps_card) {
                node.visible = show;
            }
        }
    }

    fn handle_widget_interactions(&mut self, world: &mut World) {
        if self.active_screen != Screen::Widgets {
            return;
        }

        let delta = world.resources.window.timing.delta_time;
        self.progress_value += delta * 0.1;
        if self.progress_value > 1.0 {
            self.progress_value = 0.0;
        }
        world.ui_progress_bar_set_value(self.progress_entity, self.progress_value);

        if world.ui_button_clicked(self.widget_button_primary) {
            self.widget_click_count += 1;
            world.resources.text_cache.set_text(
                self.widget_click_count_text_slot,
                format!("{}", self.widget_click_count),
            );
            world.ui_show_toast(
                &format!("Button clicked ({})", self.widget_click_count),
                ToastSeverity::Info,
                3.0,
            );
        }
        if world.ui_button_clicked(self.widget_button_success) {
            self.widget_click_count += 1;
            world.resources.text_cache.set_text(
                self.widget_click_count_text_slot,
                format!("{}", self.widget_click_count),
            );
            world.ui_show_toast("Action completed", ToastSeverity::Success, 3.0);
        }
        if world.ui_button_clicked(self.widget_button_error) {
            self.widget_click_count += 1;
            world.resources.text_cache.set_text(
                self.widget_click_count_text_slot,
                format!("{}", self.widget_click_count),
            );
            world.ui_show_toast("Something went wrong", ToastSeverity::Error, 3.0);
        }

        if let Some(submitted_text) = world.ui_text_input_submitted(self.submit_input_entity) {
            world.resources.text_cache.set_text(
                self.submit_log_text_slot,
                format!("Submitted: \"{}\"", submitted_text),
            );
        }

        if world.ui_button_clicked(self.focus_button_entity) {
            world.ui_focus(self.focus_target_entity);
        }

        if world.ui_toggle_changed(self.disable_toggle_entity) {
            let disabled = world.ui_toggle_value(self.disable_toggle_entity);
            world.ui_set_disabled(self.disabled_button_entity, disabled);
        }

        if world.ui_button_clicked(self.disabled_button_entity) {
            world.ui_show_toast("Disabled button was clicked!", ToastSeverity::Info, 3.0);
        }

        if world.ui_button_clicked(self.confirm_trigger_button) {
            world.ui_show_modal(self.confirm_dialog_entity);
        }
        if let Some(confirmed) = world.ui_modal_result(self.confirm_dialog_entity) {
            if confirmed {
                world.ui_show_toast("Confirmed!", ToastSeverity::Success, 3.0);
            } else {
                world.ui_show_toast("Cancelled", ToastSeverity::Info, 3.0);
            }
        }

        for event in world.ui_events().to_vec() {
            if let UiEvent::TreeNodeContextMenu { tree, position, .. } = event
                && tree == self.tree_view_entity
            {
                world.ui_show_context_menu(self.context_menu_entity, position);
            }
        }

        if let Some(item_index) = world.ui_context_menu_clicked(self.context_menu_entity) {
            let action = match item_index {
                0 => "Cut",
                1 => "Copy",
                2 => "Paste",
                _ => "Unknown",
            };
            world.ui_show_toast(&format!("Context menu: {action}"), ToastSeverity::Info, 2.0);
        }

        let slider_val = world.ui_slider_value(self.slider_entity);
        let toggle_val = world.ui_toggle_value(self.toggle_entity);
        let checkbox_val = world.ui_checkbox_value(self.checkbox_entity);
        let radio_val = world.ui_radio_group_value(1);
        let tab_val = world.ui_tab_bar_selected(self.tab_bar_entity);
        let dropdown_val = world.ui_dropdown_selected(self.dropdown_entity);
        let text_val = world.ui_text_input_value(self.text_input_entity);
        let color_val = world.ui_color_picker_value(self.color_picker_entity);
        let menu_action = world.ui_menu_clicked(self.menu_entity);

        let radio_label = radio_val.map_or("None", |v| ["A", "B", "C"][v]);
        let tab_label = ["General", "Audio", "Display"][tab_val.min(2)];
        let drop_label = ["Low", "Medium", "High", "Ultra"][dropdown_val.min(3)];
        let input_display = if text_val.len() > 20 {
            &text_val[..20]
        } else {
            &text_val
        };
        let drag_val = world.ui_drag_value(self.drag_value_entity);
        let menu_suffix =
            menu_action.map_or(String::new(), |index| format!("\nMenu: clicked #{}", index));

        let status = format!(
            "Slider: {:.1} | Drag: {:.2} | Toggle: {} | Check: {}\nRadio: {} | Tab: {} | Drop: {}\nInput: \"{}\"\nColor: ({:.2}, {:.2}, {:.2}, {:.2}){}",
            slider_val,
            drag_val,
            if toggle_val { "ON" } else { "OFF" },
            if checkbox_val { "ON" } else { "OFF" },
            radio_label,
            tab_label,
            drop_label,
            input_display,
            color_val.x,
            color_val.y,
            color_val.z,
            color_val.w,
            menu_suffix,
        );
        world
            .resources
            .text_cache
            .set_text(self.status_text_slot, status);
    }

    fn update_animated_elements(&mut self, world: &mut World) {
        let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
        let pulse_target = (time * 2.0).sin() * 0.5 + 0.5;
        let pulse_alpha =
            world
                .resources
                .retained_ui
                .animate(self.pulse_entity, 0, pulse_target, 4.0);
        let pulse_color = blend_color(
            nalgebra_glm::Vec4::new(0.0, 1.0, 0.5, 0.3),
            nalgebra_glm::Vec4::new(0.0, 1.0, 0.5, 1.0),
            pulse_alpha,
        );
        if let Some(color_comp) = world.get_ui_node_color_mut(self.pulse_entity) {
            color_comp.colors[0] = Some(pulse_color);
        }

        let delta = world.resources.window.timing.delta_time;
        self.color_blend_time += delta * 0.3;
        if self.color_blend_time > 1.0 {
            self.color_blend_time -= 1.0;
        }
        let blended = blend_color(CYAN, MAGENTA, self.color_blend_time);

        world.resources.retained_ui.draw_rect(
            nalgebra_glm::Vec2::new(4.0, 52.0),
            nalgebra_glm::Vec2::new(3.0, 30.0),
            blended,
        );
    }
}
