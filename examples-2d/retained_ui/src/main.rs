use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(RetainedUiDemo::default())
}

fn rebuild_status(
    world: &mut World,
    status_text: Entity,
    last_menu: &std::rc::Rc<std::cell::Cell<Option<usize>>>,
) {
    let slider_val: f32 = world.ui_prop("slider");
    let drag_val: f32 = world.ui_prop("drag_value");
    let toggle_val: bool = world.ui_prop("toggle");
    let checkbox_val: bool = world.ui_prop("checkbox");
    let radio_val: usize = world.ui_prop("radio");
    let tab_val: usize = world.ui_prop("tab_bar");
    let dropdown_val: usize = world.ui_prop("dropdown");
    let text_val: String = world.ui_prop("text_input");
    let color_val: nalgebra_glm::Vec4 = world.ui_prop("color_picker");

    let radio_label = ["A", "B", "C"][radio_val.min(2)];
    let tab_label = ["General", "Audio", "Display"][tab_val.min(2)];
    let drop_label = ["Low", "Medium", "High", "Ultra"][dropdown_val.min(3)];
    let input_display = if text_val.len() > 20 {
        &text_val[..20]
    } else {
        &text_val
    };
    let menu_suffix = last_menu
        .get()
        .map_or(String::new(), |index| format!("\nMenu: clicked #{index}"));

    let status = format!(
        "Slider: {slider_val:.1} | Drag: {drag_val:.2} | Toggle: {} | Check: {}\nRadio: {radio_label} | Tab: {tab_label} | Drop: {drop_label}\nInput: \"{input_display}\"\nColor: ({:.2}, {:.2}, {:.2}, {:.2}){menu_suffix}",
        if toggle_val { "ON" } else { "OFF" },
        if checkbox_val { "ON" } else { "OFF" },
        color_val.x,
        color_val.y,
        color_val.z,
        color_val.w,
    );
    world.ui_set_text(status_text, &status);
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

#[derive(Default)]
struct RetainedUiDemo {
    fps_text: Entity,
    uptime_text: Entity,
    entity_count_text: Entity,
    progress_entity: Entity,
    progress_value: f32,
    pulse_entity: Entity,
    color_blend_time: f32,
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
        let (nav_buttons, nav_indicators) = build_sidebar(&mut tree);

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

        let mut screen_roots = [Entity::default(); 4];
        let (dashboard, fps_card) = self.build_dashboard_screen(&mut tree);
        screen_roots[0] = dashboard;
        screen_roots[1] = build_systems_screen(&mut tree);
        screen_roots[2] = build_settings_screen(&mut tree, fps_card);
        screen_roots[3] = self.build_widgets_screen(&mut tree);

        tree.pop_parent();

        build_bottom_bar(&mut tree);

        tree.pop_parent();

        tree.finish();

        let active_screen = std::rc::Rc::new(std::cell::Cell::new(0usize));
        for (index, &nav) in nav_buttons.iter().enumerate() {
            let active_screen = active_screen.clone();
            world.ui_react_clicked(nav, move |world: &mut World| {
                let prev = active_screen.get();
                if index != prev {
                    world.ui_set_visible(screen_roots[prev], false);
                    if let Some(color) = world.get_ui_node_color_mut(nav_indicators[prev]) {
                        color.computed_color = INACTIVE_INDICATOR;
                    }
                    active_screen.set(index);
                    world.ui_set_visible(screen_roots[index], true);
                    if let Some(color) = world.get_ui_node_color_mut(nav_indicators[index]) {
                        color.computed_color = ACTIVE_INDICATOR;
                    }
                }
            });
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let fps = world.resources.window.timing.frames_per_second;
        world.ui_set_text(self.fps_text, &format!("{:.0}", fps));

        let uptime_ms = world.resources.window.timing.uptime_milliseconds;
        let seconds = uptime_ms / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        world.ui_set_text(
            self.uptime_text,
            &format!("{}:{:02}", minutes, remaining_seconds),
        );

        let entity_count = world.entity_count();
        world.ui_set_text(self.entity_count_text, &format!("{}", entity_count));

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

fn build_sidebar(tree: &mut UiTreeBuilder) -> ([Entity; 4], [Entity; 4]) {
    let mut nav_buttons = [Entity::default(); 4];
    let mut nav_indicators = [Entity::default(); 4];

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

        nav_indicators[index] = tree
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

        nav_buttons[index] = tree
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

    (nav_buttons, nav_indicators)
}

impl RetainedUiDemo {
    fn build_dashboard_screen(&mut self, tree: &mut UiTreeBuilder) -> (Entity, Entity) {
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

        let card_data: [(&str, &str, nalgebra_glm::Vec4); 3] = [
            ("FPS", "60", CYAN),
            ("ENTITIES", "0", MAGENTA),
            (
                "UPTIME",
                "0:00",
                nalgebra_glm::Vec4::new(0.4, 1.0, 0.4, 1.0),
            ),
        ];

        let mut card_text_entities = [Entity::default(); 3];
        let mut fps_card = Entity::default();
        for (index, (label, initial_text, accent_color)) in card_data.iter().enumerate() {
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

            card_text_entities[index] = tree
                .add_node()
                .window(
                    Ab(nalgebra_glm::Vec2::new(16.0, 55.0)),
                    Ab(nalgebra_glm::Vec2::new(200.0, 42.0)),
                    Anchor::TopLeft,
                )
                .with_text(initial_text, 36.0)
                .with_color::<UiBase>(WHITE)
                .without_pointer_events()
                .done();

            if index == 0 {
                fps_card = card;
            }

            tree.pop_parent();
        }
        self.fps_text = card_text_entities[0];
        self.entity_count_text = card_text_entities[1];
        self.uptime_text = card_text_entities[2];

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
        (screen, fps_card)
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
        let left_content_entity = tree
            .world_mut()
            .widget::<UiScrollAreaData>(left_scroll)
            .map(|d| d.content_entity)
            .unwrap();
        tree.build_ui(left_content_entity, |ui| {
            ui.heading("Value Widgets");
            ui.separator();

            ui.label("Buttons");

            let button_primary = ui.button("Primary Button");
            let theme = ui.theme().clone();
            let button_success = ui.button_colored("Success Action", theme.success_color);
            let button_error = ui.button_colored("Danger Action", theme.error_color);

            let click_count_row = ui
                .tree()
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 24.0)),
                )
                .flow(FlowDirection::Horizontal, 0.0, 8.0)
                .entity();

            ui.tree().push_parent(click_count_row);

            ui.tree()
                .add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(110.0, 24.0)))
                .with_text("Click count:", 14.0)
                .with_color::<UiBase>(LIGHT_GRAY)
                .without_pointer_events()
                .done();

            let click_count_text = ui
                .tree()
                .add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(50.0, 24.0)))
                .with_text("0", 14.0)
                .with_color::<UiBase>(CYAN)
                .without_pointer_events()
                .done();

            ui.tree().pop_parent();

            let click_count = std::rc::Rc::new(std::cell::Cell::new(0u32));
            for (entity, toast_msg) in [
                (button_primary, None),
                (button_success, Some("Action completed")),
                (button_error, Some("Something went wrong")),
            ] {
                let click_count = click_count.clone();
                let severity = match toast_msg {
                    None => ToastSeverity::Info,
                    Some("Action completed") => ToastSeverity::Success,
                    _ => ToastSeverity::Error,
                };
                ui.react_clicked(entity, move |world: &mut World| {
                    let count = click_count.get() + 1;
                    click_count.set(count);
                    world.ui_set_text(click_count_text, &format!("{count}"));
                    let msg = match severity {
                        ToastSeverity::Success => "Action completed".to_string(),
                        ToastSeverity::Error => "Something went wrong".to_string(),
                        _ => format!("Button clicked ({count})"),
                    };
                    world.ui_show_toast(&msg, severity, 3.0);
                });
            }

            ui.spacing(4.0);
            ui.label("Slider");
            ui.slider("slider", 0.0, 100.0, 50.0);

            ui.spacing(4.0);
            ui.label("Drag Value");
            ui.drag_value_configured(
                "drag_value",
                DragValueConfig::new(0.0, 10.0, 0.5).speed(0.01),
            );

            ui.spacing(4.0);
            ui.label("Toggle");
            ui.toggle("toggle", false);

            ui.spacing(4.0);
            ui.label("Checkbox");
            ui.checkbox("checkbox", "Enable notifications", false);

            ui.spacing(4.0);
            ui.label("Radio Buttons");
            ui.radio("Option A", 1, 0);
            ui.radio("Option B", 1, 1);
            ui.radio("Option C", 1, 2);
            ui.radio_group("radio", 1);

            ui.spacing(4.0);
            ui.label("Progress Bar");
            self.progress_entity = ui.progress_bar("", 0.0);

            ui.spacing(4.0);
            ui.label("Text Input");
            ui.text_input("text_input", "Type here...");

            ui.spacing(8.0);
            ui.heading("New Features");
            ui.separator();

            ui.label("Submit Detection (Enter key)");
            let submit_input = ui.text_input("", "Type and press Enter...");
            let submit_log_text = ui
                .tree()
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 24.0)),
                )
                .with_text("Press Enter to submit", 12.0)
                .with_color::<UiBase>(CYAN_DIM)
                .without_pointer_events()
                .done();
            ui.react_submitted(submit_input, move |text: String, world: &mut World| {
                world.ui_set_text(submit_log_text, &format!("Submitted: \"{text}\""));
            });

            ui.spacing(4.0);
            ui.label("Programmatic Focus");
            let focus_target = ui.text_input("", "Focus target");
            let focus_button = ui.button("Focus the input above");
            ui.react_clicked(focus_button, move |world: &mut World| {
                world.ui_focus(focus_target);
            });

            ui.spacing(4.0);
            ui.label("Disabled State");
            let disabled_button = ui.button("I can be disabled");
            ui.react_clicked(disabled_button, |world: &mut World| {
                world.ui_show_toast("Disabled button was clicked!", ToastSeverity::Info, 3.0);
            });
            ui.toggle("disable_button", false);
            ui.react("disable_button", move |val: bool, world: &mut World| {
                world.ui_set_disabled(disabled_button, val);
            });
            ui.label("Toggle to disable button");
        });
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
        let right_content_entity = tree
            .world_mut()
            .widget::<UiScrollAreaData>(right_scroll)
            .map(|d| d.content_entity)
            .unwrap();
        let tree_view_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
        let confirm_trigger_cell = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
        let tv_cell = tree_view_cell.clone();
        let ct_cell = confirm_trigger_cell.clone();
        tree.build_ui(right_content_entity, |ui| {
            ui.heading("Layout & Compound Widgets");
            ui.separator();

            ui.label("Collapsing Header");
            ui.collapsing_header("", "Click to expand", true, |ui| {
                ui.label("This content is inside the collapsing header.");
                ui.label("It can be toggled by clicking the header above.");
            });

            ui.spacing(4.0);
            ui.label("Tab Bar");
            ui.tab_bar("tab_bar", &["General", "Audio", "Display"], 0);

            ui.spacing(4.0);
            ui.label("Dropdown");
            ui.dropdown("dropdown", &["Low", "Medium", "High", "Ultra"], 1);

            ui.spacing(4.0);
            ui.label("Menu");
            let menu_entity = ui.menu("Actions", &["New", "Open", "Save", "Export"]);

            ui.spacing(4.0);
            ui.label("Color Picker");
            ui.color_picker("color_picker", nalgebra_glm::Vec4::new(0.3, 0.5, 0.9, 1.0));

            ui.spacing(8.0);
            ui.heading("Editor Widgets");
            ui.separator();

            ui.label("Selectable Labels");
            ui.selectable_label("", "Renderer: wgpu", Some(1));
            ui.selectable_label("", "Audio: disabled", Some(1));
            ui.selectable_label("", "Physics: rapier", Some(1));

            ui.spacing(4.0);
            ui.label("Property Grid");
            let grid = ui.property_grid(60.0);
            let section = ui.property_section(grid, "Transform");
            let area = ui.property_row(grid, section, "X");
            ui.tree().push_parent(area);
            ui.tree().add_drag_value(-10.0, 10.0, 1.0);
            ui.tree().pop_parent();
            let area = ui.property_row(grid, section, "Y");
            ui.tree().push_parent(area);
            ui.tree().add_drag_value(-10.0, 10.0, 2.0);
            ui.tree().pop_parent();

            ui.spacing(4.0);
            ui.label("Tree View");
            let tree_view = ui.tree().add_tree_view(false);
            tv_cell.set(tree_view);
            let tv_content = ui
                .world_mut()
                .widget::<UiTreeViewData>(tree_view)
                .map(|d| d.content_entity)
                .unwrap();
            let root_node = ui
                .tree()
                .add_tree_node(tree_view, tv_content, "Project", 0, 0);
            ui.world_mut().ui_tree_node_set_expanded(root_node, true);
            let root_children = ui
                .world_mut()
                .widget::<UiTreeNodeData>(root_node)
                .map(|d| d.children_container)
                .unwrap();
            ui.tree()
                .add_tree_node(tree_view, root_children, "Assets", 1, 1);
            ui.tree()
                .add_tree_node(tree_view, root_children, "Scripts", 1, 2);
            ui.tree()
                .add_tree_node(tree_view, root_children, "Scenes", 1, 3);

            ui.spacing(4.0);
            ui.label("Confirm Dialog");
            let confirm_trigger = ui.button("Show Confirm Dialog");
            ct_cell.set(confirm_trigger);

            ui.spacing(4.0);
            ui.label("Scroll Area");
            ui.scroll_area(nalgebra_glm::Vec2::new(0.0, 120.0), |ui| {
                for index in 0..20 {
                    ui.label(&format!("Scrollable item {}", index + 1));
                }
            });

            ui.spacing(4.0);
            ui.separator();
            ui.label("Widget Status:");

            let status_text = ui
                .tree()
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 60.0)),
                )
                .with_text("", 12.0)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .with_color::<UiBase>(CYAN_DIM)
                .without_pointer_events()
                .done();

            let last_menu = std::rc::Rc::new(std::cell::Cell::new(None::<usize>));

            let lm = last_menu.clone();
            ui.react_any(
                &[
                    "slider",
                    "drag_value",
                    "toggle",
                    "checkbox",
                    "radio",
                    "tab_bar",
                    "dropdown",
                    "text_input",
                    "color_picker",
                ],
                move |world: &mut World| {
                    rebuild_status(world, status_text, &lm);
                },
            );
            let lm = last_menu.clone();
            ui.react_menu_selected(menu_entity, move |index: usize, world: &mut World| {
                lm.set(Some(index));
                rebuild_status(world, status_text, &lm);
            });
        });
        tree.pop_parent();

        tree.add_floating_panel(
            "Floating Panel",
            Rect {
                min: nalgebra_glm::Vec2::new(100.0, 150.0),
                max: nalgebra_glm::Vec2::new(350.0, 350.0),
            },
        );

        let confirm_dialog =
            tree.add_confirm_dialog("Confirm Action", "Are you sure you want to proceed?");

        let context_menu = tree.add_context_menu(&[
            ("Cut", Some("Ctrl+X")),
            ("Copy", Some("Ctrl+C")),
            ("Paste", Some("Ctrl+V")),
        ]);

        tree.world_mut()
            .ui_react_clicked(confirm_trigger_cell.get(), move |world: &mut World| {
                world.ui_show_modal(confirm_dialog);
            });
        tree.world_mut().ui_react_confirmed(
            confirm_dialog,
            |confirmed: bool, world: &mut World| {
                if confirmed {
                    world.ui_show_toast("Confirmed!", ToastSeverity::Success, 3.0);
                } else {
                    world.ui_show_toast("Cancelled", ToastSeverity::Info, 3.0);
                }
            },
        );
        tree.world_mut()
            .ui_react_menu_selected(context_menu, |index: usize, world: &mut World| {
                let action = match index {
                    0 => "Cut",
                    1 => "Copy",
                    2 => "Paste",
                    _ => "Unknown",
                };
                world.ui_show_toast(&format!("Context menu: {action}"), ToastSeverity::Info, 2.0);
            });
        tree.world_mut().ui_react_tree_context_menu(
            tree_view_cell.get(),
            move |_node: Entity, position: nalgebra_glm::Vec2, world: &mut World| {
                world.ui_show_context_menu(context_menu, position);
            },
        );

        tree.pop_parent();
        screen
    }

    fn handle_widget_interactions(&mut self, world: &mut World) {
        let delta = world.resources.window.timing.delta_time;
        self.progress_value += delta * 0.1;
        if self.progress_value > 1.0 {
            self.progress_value = 0.0;
        }
        world.ui_progress_bar_set_value(self.progress_entity, self.progress_value);
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

fn build_settings_screen(tree: &mut UiTreeBuilder, fps_card: Entity) -> Entity {
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
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(-10.0, -10.0)),
        )
        .with_rect(6.0, 1.0, CARD_BORDER)
        .with_color::<UiBase>(CARD_BG)
        .entity();

    tree.push_parent(settings_card);

    let scroll = tree.add_scroll_area_fill(12.0, 6.0);
    let content = tree
        .world_mut()
        .widget::<UiScrollAreaData>(scroll)
        .map(|d| d.content_entity)
        .unwrap();
    tree.build_ui(content, |ui| {
        ui.heading("Display");
        ui.separator();

        ui.label("Brightness");
        ui.slider("brightness", 0.0, 100.0, 50.0);
        ui.react("brightness", |val: f32, world: &mut World| {
            let brightness = val / 100.0;
            let base = 0.02;
            let value = base + brightness * 0.06;
            world.resources.graphics.clear_color = [value, value, value + 0.02, 1.0];
            world.resources.retained_ui.background_color =
                Some(nalgebra_glm::Vec4::new(value, value, value + 0.02, 1.0));
        });

        ui.spacing(4.0);
        ui.label("UI Scale");
        ui.dropdown("", &["75%", "100%", "125%", "150%"], 1);

        ui.spacing(4.0);
        ui.label("V-Sync");
        ui.toggle("", true);

        ui.spacing(4.0);
        ui.label("Show FPS Counter");
        ui.toggle("show_fps", true);
        ui.react("show_fps", move |val: bool, world: &mut World| {
            if let Some(node) = world.get_ui_layout_node_mut(fps_card) {
                node.visible = val;
            }
        });

        ui.spacing(12.0);
        ui.heading("Theme");
        ui.separator();
        ui.theme_dropdown();
    });
    tree.pop_parent();

    tree.pop_parent();
    screen
}

fn build_systems_screen(tree: &mut UiTreeBuilder) -> Entity {
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
            Rl(nalgebra_glm::Vec2::new(100.0, 100.0)) + Ab(nalgebra_glm::Vec2::new(-10.0, -10.0)),
        )
        .with_rect(6.0, 1.0, CARD_BORDER)
        .with_color::<UiBase>(CARD_BG)
        .entity();

    tree.push_parent(systems_card);

    let scroll = tree.add_scroll_area_fill(8.0, 0.0);
    let scroll_content = tree
        .world_mut()
        .widget::<UiScrollAreaData>(scroll)
        .map(|d| d.content_entity)
        .unwrap();
    tree.build_ui(scroll_content, |ui| {
        ui.heading("Active Systems");
        ui.separator();

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
            ui.checkbox("", name, initially_on);
        }
    });
    tree.pop_parent();

    tree.pop_parent();
    screen
}
