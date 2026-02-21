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
const ACTIVE_INDICATOR: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.9, 1.0, 0.8);
const INACTIVE_INDICATOR: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.9, 1.0, 0.0);

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
    volume: f32,
    brightness: f32,
    fov: f32,
    volume_bar: Entity,
    brightness_bar: Entity,
    fov_bar: Entity,
    volume_fill: Entity,
    brightness_fill: Entity,
    fov_fill: Entity,
    widget_button_primary: Entity,
    widget_button_success: Entity,
    widget_button_error: Entity,
    widget_click_count: u32,
    widget_click_count_text_slot: usize,
}

impl State for RetainedUiDemo {
    fn title(&self) -> &str {
        "NIGHTSHADE // RETAINED UI"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.retained_ui.enabled = true;
        world.resources.graphics.clear_color = [0.02, 0.02, 0.04, 1.0];

        let camera = spawn_ortho_camera(world, nalgebra_glm::Vec2::new(0.0, 0.0));
        world.resources.active_camera = Some(camera);

        self.volume = 0.75;
        self.brightness = 0.5;
        self.fov = 0.6;

        self.fps_text_slot = world.resources.text_cache.add_text("60");
        self.uptime_text_slot = world.resources.text_cache.add_text("0:00");
        self.entity_count_text_slot = world.resources.text_cache.add_text("0");
        self.widget_click_count_text_slot = world.resources.text_cache.add_text("0");

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
                Ab(nalgebra_glm::Vec2::new(202.0, 50.0)),
                Ab(nalgebra_glm::Vec2::new(0.0, -40.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .without_pointer_events()
            .entity();

        tree.push_parent(content_area);

        self.screen_roots[0] = self.build_dashboard_screen(&mut tree);
        self.screen_roots[1] = build_systems_screen(&mut tree);
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
                Ab(nalgebra_glm::Vec2::new(200.0, -40.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 100.0)),
            )
            .with_rect(0.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.95))
            .without_pointer_events()
            .entity();

        tree.push_parent(sidebar);

        let nav_labels = ["DASHBOARD", "SYSTEMS", "SETTINGS", "WIDGETS"];

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
                    Ab(nalgebra_glm::Vec2::new(184.0, 40.0)),
                    Anchor::TopLeft,
                )
                .with_hover_layout(UiLayoutType::Window(WindowLayout {
                    position: Ab(nalgebra_glm::Vec2::new(12.0, y_offset)).into(),
                    size: Ab(nalgebra_glm::Vec2::new(184.0, 40.0)).into(),
                    anchor: Anchor::TopLeft,
                }))
                .with_interaction()
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
                Ab(nalgebra_glm::Vec2::new(198.0, 0.0)),
                Ab(nalgebra_glm::Vec2::new(200.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 100.0)),
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
            let col = index % 2;
            let row = index / 2;
            let x_percent = 2.0 + col as f32 * 50.0;
            let y_offset = 60.0 + row as f32 * 130.0;

            let card = tree
                .add_node()
                .boundary(
                    Rl(nalgebra_glm::Vec2::new(x_percent, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, y_offset)),
                    Rl(nalgebra_glm::Vec2::new(x_percent + 47.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, y_offset + 110.0)),
                )
                .with_rect(6.0, 1.0, nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.4))
                .with_color::<UiBase>(CARD_BG)
                .without_pointer_events()
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

            tree.pop_parent();
        }

        let memory_card = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(52.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 190.0)),
                Rl(nalgebra_glm::Vec2::new(99.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 300.0)),
            )
            .with_rect(6.0, 1.0, nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.4))
            .with_color::<UiBase>(CARD_BG)
            .without_pointer_events()
            .entity();

        tree.push_parent(memory_card);

        tree.add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Ab(nalgebra_glm::Vec2::new(0.0, 3.0)) + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
            )
            .with_rect(6.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.9, 0.6, 0.0, 1.0))
            .without_pointer_events();

        tree.add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(16.0, 22.0)),
                Ab(nalgebra_glm::Vec2::new(150.0, 20.0)),
                Anchor::TopLeft,
            )
            .with_text("MEMORY", 12.0)
            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0))
            .without_pointer_events();

        tree.add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(16.0, 55.0)),
                Ab(nalgebra_glm::Vec2::new(200.0, 42.0)),
                Anchor::TopLeft,
            )
            .with_text("N/A", 36.0)
            .with_color::<UiBase>(WHITE)
            .without_pointer_events();

        tree.pop_parent();

        tree.add_node()
            .solid(
                Ab(nalgebra_glm::Vec2::new(16.0, 9.0)),
                ScalingMode::Fit,
                nalgebra_glm::Vec2::new(0.0, 1.0),
            )
            .with_rect(4.0, 1.0, nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.3))
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
            .without_pointer_events()
            .entity();

        tree.push_parent(screen);

        tree.add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
                Ab(nalgebra_glm::Vec2::new(300.0, 28.0)),
                Anchor::TopLeft,
            )
            .with_text("CONFIGURATION", 18.0)
            .with_color::<UiBase>(CYAN)
            .without_pointer_events();

        let sliders = [
            ("VOLUME", self.volume, CYAN),
            ("BRIGHTNESS", self.brightness, MAGENTA),
            (
                "FIELD OF VIEW",
                self.fov,
                nalgebra_glm::Vec4::new(0.4, 1.0, 0.4, 1.0),
            ),
        ];

        let mut bar_entities = Vec::new();
        let mut fill_entities = Vec::new();

        for (index, (label, value, accent)) in sliders.iter().enumerate() {
            let y_offset = 70.0 + index as f32 * 90.0;

            tree.add_node()
                .window(
                    Ab(nalgebra_glm::Vec2::new(20.0, y_offset)),
                    Ab(nalgebra_glm::Vec2::new(200.0, 20.0)),
                    Anchor::TopLeft,
                )
                .with_text(label, 13.0)
                .with_color::<UiBase>(LIGHT_GRAY)
                .without_pointer_events();

            tree.add_node()
                .boundary(
                    Rl(nalgebra_glm::Vec2::new(90.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, y_offset)),
                    Rl(nalgebra_glm::Vec2::new(98.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, y_offset + 20.0)),
                )
                .with_text(&format!("{:.0}%", value * 100.0), 13.0)
                .with_color::<UiBase>(*accent)
                .without_pointer_events();

            let bar_bg = tree
                .add_node()
                .boundary(
                    Ab(nalgebra_glm::Vec2::new(20.0, y_offset + 30.0)),
                    Rl(nalgebra_glm::Vec2::new(98.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, y_offset + 54.0)),
                )
                .with_rect(4.0, 1.0, nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.3))
                .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.8))
                .with_interaction()
                .with_cursor_icon(winit::window::CursorIcon::Pointer)
                .entity();

            tree.push_parent(bar_bg);

            let fill_width = *value * 100.0;
            let fill = tree
                .add_node()
                .boundary(
                    Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                    Rl(nalgebra_glm::Vec2::new(fill_width, 100.0)),
                )
                .with_rect(4.0, 0.0, TRANSPARENT)
                .with_color::<UiBase>(accent * 0.8)
                .without_pointer_events()
                .done();

            bar_entities.push(bar_bg);
            fill_entities.push(fill);

            tree.pop_parent();
        }

        self.volume_bar = bar_entities[0];
        self.brightness_bar = bar_entities[1];
        self.fov_bar = bar_entities[2];
        self.volume_fill = fill_entities[0];
        self.brightness_fill = fill_entities[1];
        self.fov_fill = fill_entities[2];

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
            .without_pointer_events()
            .entity();

        tree.push_parent(screen);

        let panel = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
                Rl(nalgebra_glm::Vec2::new(98.0, 95.0)),
            )
            .with_rect(6.0, 1.0, nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.4))
            .with_color::<UiBase>(CARD_BG)
            .entity();

        tree.push_parent(panel);

        let flow_container = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(16.0, 16.0)),
                Ab(nalgebra_glm::Vec2::new(-16.0, -16.0))
                    + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .flow(FlowDirection::Vertical, 0.0, 8.0)
            .entity();

        tree.push_parent(flow_container);

        tree.add_heading("Widget Toolkit Demo");
        tree.add_separator();
        tree.add_spacing(4.0);

        tree.add_label("The retained UI now supports high-level widgets.");
        tree.add_label("Buttons, labels, headings, and separators are built");
        tree.add_label("using the flow layout system with theme integration.");

        tree.add_spacing(8.0);
        tree.add_heading("Buttons");
        tree.add_separator();
        tree.add_spacing(4.0);

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

        tree.add_spacing(8.0);

        let click_count_row = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 28.0)),
            )
            .flow(FlowDirection::Horizontal, 0.0, 8.0)
            .entity();

        tree.push_parent(click_count_row);

        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(120.0, 28.0)))
            .with_text("Click count:", 16.0)
            .with_color::<UiBase>(LIGHT_GRAY)
            .without_pointer_events()
            .done();

        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(60.0, 28.0)))
            .with_text_slot(self.widget_click_count_text_slot, 16.0)
            .with_color::<UiBase>(CYAN)
            .without_pointer_events()
            .done();

        tree.pop_parent();

        tree.add_spacing(8.0);
        tree.add_heading("Flow Layout");
        tree.add_separator();
        tree.add_spacing(4.0);

        tree.add_label("Content above uses vertical flow with 8px spacing.");
        tree.add_label("The click counter row uses horizontal flow.");

        tree.add_spring();

        tree.add_label_colored("This label is pushed to the bottom by a spring.", CYAN_DIM);

        tree.pop_parent();
        tree.pop_parent();
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

            if let Some(node) = world.get_ui_layout_node_mut(self.screen_roots[index]) {
                node.visible = is_active;
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
        if self.active_screen != Screen::Settings {
            return;
        }

        let pairs = [
            (self.volume_bar, self.volume_fill, 0),
            (self.brightness_bar, self.brightness_fill, 1),
            (self.fov_bar, self.fov_fill, 2),
        ];

        for (bar_entity, fill_entity, setting_index) in pairs {
            let pressed = world
                .get_ui_node_interaction(bar_entity)
                .is_some_and(|interaction| interaction.pressed);

            if !pressed {
                continue;
            }

            let bar_rect = match world.get_ui_layout_node(bar_entity) {
                Some(node) => node.computed_rect,
                None => continue,
            };

            let mouse_x = world.resources.input.mouse.position.x;
            let clamped = ((mouse_x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);

            match setting_index {
                0 => self.volume = clamped,
                1 => self.brightness = clamped,
                2 => self.fov = clamped,
                _ => {}
            }

            if let Some(fill_node) = world.get_ui_layout_node_mut(fill_entity) {
                let base_id = std::any::TypeId::of::<UiBase>();
                if let Some(UiLayoutType::Boundary(boundary)) = fill_node.layouts.get_mut(&base_id)
                {
                    boundary.position_2 =
                        Rl(nalgebra_glm::Vec2::new(clamped * 100.0, 100.0)).into();
                }
            }
        }
    }

    fn handle_widget_interactions(&mut self, world: &mut World) {
        if self.active_screen != Screen::Widgets {
            return;
        }

        let buttons = [
            self.widget_button_primary,
            self.widget_button_success,
            self.widget_button_error,
        ];

        for button in buttons {
            if world.ui_button_clicked(button) {
                self.widget_click_count += 1;
                world.resources.text_cache.set_text(
                    self.widget_click_count_text_slot,
                    format!("{}", self.widget_click_count),
                );
            }
        }
    }
}

fn build_systems_screen(tree: &mut UiTreeBuilder) -> Entity {
    let screen = tree
        .add_node()
        .boundary(
            Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
        )
        .with_visible(false)
        .without_pointer_events()
        .entity();

    tree.push_parent(screen);

    tree.add_node()
        .window(
            Ab(nalgebra_glm::Vec2::new(20.0, 20.0)),
            Ab(nalgebra_glm::Vec2::new(300.0, 28.0)),
            Anchor::TopLeft,
        )
        .with_text("ACTIVE SYSTEMS", 18.0)
        .with_color::<UiBase>(CYAN)
        .without_pointer_events();

    let systems_container = tree
        .add_node()
        .boundary(
            Ab(nalgebra_glm::Vec2::new(20.0, 60.0)),
            Rl(nalgebra_glm::Vec2::new(98.0, 95.0)),
        )
        .with_rect(4.0, 1.0, nalgebra_glm::Vec4::new(0.1, 0.1, 0.18, 0.3))
        .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.03, 0.03, 0.06, 0.7))
        .with_clip()
        .without_pointer_events()
        .entity();

    tree.push_parent(systems_container);

    let system_names = [
        ("Transform Propagation", true),
        ("Camera Update", true),
        ("Sprite Rendering", true),
        ("Text Sync", true),
        ("Animation Player", true),
        ("Particle Update", false),
        ("Physics Step", false),
        ("Collision Detection", false),
        ("Audio Mixer", false),
        ("NavMesh Agent", false),
        ("UI Layout Compute", true),
        ("UI Picking", true),
        ("UI State Update", true),
        ("UI Color Blend", true),
        ("UI Render Sync", true),
        ("Event Bus Dispatch", true),
        ("Input Reset", true),
        ("Deferred Commands", true),
    ];

    for (index, (name, active)) in system_names.iter().enumerate() {
        let y_pos = 8.0 + index as f32 * 32.0;

        tree.add_node()
            .window(
                Ab(nalgebra_glm::Vec2::new(12.0, y_pos)),
                Rl(nalgebra_glm::Vec2::new(95.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 26.0)),
                Anchor::TopLeft,
            )
            .with_rect(3.0, 0.0, TRANSPARENT)
            .with_color::<UiBase>(if index % 2 == 0 {
                nalgebra_glm::Vec4::new(0.05, 0.05, 0.09, 0.5)
            } else {
                nalgebra_glm::Vec4::new(0.04, 0.04, 0.07, 0.3)
            })
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_node()
                    .window(
                        Ab(nalgebra_glm::Vec2::new(12.0, 13.0)),
                        Ab(nalgebra_glm::Vec2::new(14.0, 14.0)),
                        Anchor::CenterLeft,
                    )
                    .with_rect(2.0, 0.0, TRANSPARENT)
                    .with_color::<UiBase>(if *active {
                        nalgebra_glm::Vec4::new(0.2, 1.0, 0.4, 1.0)
                    } else {
                        nalgebra_glm::Vec4::new(0.4, 0.15, 0.15, 1.0)
                    })
                    .without_pointer_events();

                tree.add_node()
                    .window(
                        Ab(nalgebra_glm::Vec2::new(36.0, 13.0)),
                        Ab(nalgebra_glm::Vec2::new(250.0, 22.0)),
                        Anchor::CenterLeft,
                    )
                    .with_text(name, 13.0)
                    .with_color::<UiBase>(if *active { WHITE } else { LIGHT_GRAY })
                    .without_pointer_events();

                tree.add_node()
                    .window(
                        Rl(nalgebra_glm::Vec2::new(90.0, 50.0)),
                        Ab(nalgebra_glm::Vec2::new(60.0, 18.0)),
                        Anchor::CenterRight,
                    )
                    .with_text(if *active { "ON" } else { "OFF" }, 11.0)
                    .with_color::<UiBase>(if *active {
                        nalgebra_glm::Vec4::new(0.2, 1.0, 0.4, 1.0)
                    } else {
                        nalgebra_glm::Vec4::new(0.6, 0.3, 0.3, 1.0)
                    })
                    .without_pointer_events();
            });
    }

    tree.pop_parent();
    tree.pop_parent();
    screen
}
