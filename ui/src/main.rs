use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(UiDemoState::default())
}

#[derive(Default)]
struct UiDemoState {
    ui_only_mode: bool,
    log_messages: Vec<String>,

    button_count: u32,
    slider_float: f32,
    slider_int: f32,
    toggle_a: bool,
    toggle_b: bool,
    checkbox_a: bool,
    checkbox_b: bool,
    checkbox_c: bool,
    radio_choice: u32,
    dropdown_choice: usize,
    text_single: String,
    text_multi: String,
    color: nalgebra_glm::Vec4,
    progress: f32,
    progress_dir: f32,

    transform_x: f32,
    transform_y: f32,
    transform_z: f32,
    obj_visible: bool,
    obj_locked: bool,

    show_about: bool,
}

impl State for UiDemoState {
    fn title(&self) -> &str {
        "Nightshade UI Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        self.ui_only_mode = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;

        self.slider_float = 0.5;
        self.slider_int = 50.0;
        self.toggle_a = true;
        self.checkbox_b = true;
        self.color = nalgebra_glm::vec4(0.3, 0.6, 0.9, 1.0);
        self.progress = 0.0;
        self.progress_dir = 1.0;
        self.text_single = "Hello, World!".to_string();
        self.text_multi = "Nightshade UI".to_string();

        self.transform_x = 0.0;
        self.transform_y = 1.5;
        self.transform_z = 0.0;
        self.obj_visible = true;

        self.log_messages = vec![
            "[INFO] Application started".to_string(),
            "[INFO] UI system initialized".to_string(),
            "[INFO] Rendering engine ready".to_string(),
        ];

        let camera_position = Vec3::new(0.0, 4.0, 10.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);
    }

    fn immediate_ui(&mut self, world: &mut World, ui: &mut ImmediateUi) {
        let delta_time = world.resources.window.timing.delta_time;
        self.progress += delta_time * 0.25 * self.progress_dir;
        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.progress_dir = -1.0;
        } else if self.progress <= 0.0 {
            self.progress = 0.0;
            self.progress_dir = 1.0;
        }

        if self.ui_only_mode {
            let theme = ui.theme_state.active_theme();
            ui.set_background(Some(theme.panel_color));
        } else {
            ui.set_background(None);
        }

        let fps = world.resources.window.timing.frames_per_second;
        let frame_time = world.resources.window.timing.delta_time * 1000.0;

        ui.begin_top_panel("Menu Bar", 38.0);
        ui.begin_horizontal_at_cursor();

        if let Some(idx) = ui.menu(
            "File",
            &[
                "New Project",
                "Open...",
                "Save",
                "Save As...",
                "Export",
                "Exit",
            ],
        ) {
            let items = [
                "New Project",
                "Open...",
                "Save",
                "Save As...",
                "Export",
                "Exit",
            ];
            self.log_messages
                .push(format!("[CMD] File > {}", items[idx]));
        }
        if let Some(idx) = ui.menu(
            "Edit",
            &[
                "Undo",
                "Redo",
                "Cut",
                "Copy",
                "Paste",
                "Delete",
                "Select All",
            ],
        ) {
            let items = [
                "Undo",
                "Redo",
                "Cut",
                "Copy",
                "Paste",
                "Delete",
                "Select All",
            ];
            self.log_messages
                .push(format!("[CMD] Edit > {}", items[idx]));
        }
        if let Some(idx) = ui.menu("View", &["Properties", "Console", "Fullscreen"]) {
            let items = ["Properties", "Console", "Fullscreen"];
            self.log_messages
                .push(format!("[CMD] View > {}", items[idx]));
        }
        if let Some(idx) = ui.menu("Help", &["Documentation", "About"]) {
            if idx == 1 {
                self.show_about = !self.show_about;
            } else {
                self.log_messages
                    .push("[CMD] Help > Documentation".to_string());
            }
        }

        ui.spacing(20.0);
        ui.theme_dropdown();

        let fps_text = format!("{:.1} FPS ({:.2}ms)", fps, frame_time);
        let fps_width = fps_text.len() as f32 * 18.0 * 0.55;
        ui.spring(fps_width);
        ui.label(&fps_text);
        ui.end_horizontal();
        ui.end_panel();

        ui.begin_bottom_panel("Status Bar", 100.0);
        ui.begin_horizontal_at_cursor();
        ui.label("Console");
        ui.spacing(16.0);
        if ui.button("Clear").clicked {
            self.log_messages.clear();
            self.log_messages.push("[INFO] Console cleared".to_string());
        }
        ui.end_horizontal();
        ui.separator();

        let max_messages = 3;
        let start = self.log_messages.len().saturating_sub(max_messages);
        for msg in self.log_messages.iter().skip(start) {
            let color = if msg.starts_with("[ERR]") {
                nalgebra_glm::vec4(1.0, 0.4, 0.4, 1.0)
            } else if msg.starts_with("[WARN]") {
                nalgebra_glm::vec4(1.0, 0.8, 0.3, 1.0)
            } else if msg.starts_with("[CMD]") {
                nalgebra_glm::vec4(0.5, 0.8, 1.0, 1.0)
            } else {
                nalgebra_glm::vec4(0.7, 0.7, 0.7, 1.0)
            };
            ui.label_colored(msg, color);
        }
        ui.end_panel();

        ui.begin_left_panel("Toolbox", 180.0);

        if ui.collapsing_header("Tools", true) {
            ui.indent(8.0);
            let tools = ["Select", "Move", "Rotate", "Scale", "Paint"];
            for (idx, tool) in tools.iter().enumerate() {
                ui.radio_value(tool, &mut self.radio_choice, idx as u32);
            }
            ui.indent(-8.0);
        }

        ui.spacing(4.0);

        if ui.collapsing_header("Options", true) {
            ui.indent(8.0);
            ui.checkbox("opt_snap", "Snap to Grid", &mut self.checkbox_a);
            ui.checkbox("opt_guides", "Show Guides", &mut self.checkbox_b);
            ui.checkbox("opt_axes", "Show Axes", &mut self.checkbox_c);
            ui.indent(-8.0);
        }

        ui.spacing(4.0);

        if ui.collapsing_header("Display", false) {
            ui.indent(8.0);
            let prev = self.ui_only_mode;
            ui.toggle_with_label("UI Only:", &mut self.ui_only_mode);
            if self.ui_only_mode != prev {
                world.resources.graphics.show_grid = !self.ui_only_mode;
                world.resources.graphics.atmosphere = if self.ui_only_mode {
                    Atmosphere::None
                } else {
                    Atmosphere::Sky
                };
            }
            ui.indent(-8.0);
        }

        ui.end_panel();

        ui.begin_right_panel("Inspector", 260.0);

        if ui.collapsing_header("Transform", true) {
            ui.indent(8.0);
            ui.slider_with_label("X:", &mut self.transform_x, -10.0, 10.0);
            ui.slider_with_label("Y:", &mut self.transform_y, -10.0, 10.0);
            ui.slider_with_label("Z:", &mut self.transform_z, -10.0, 10.0);
            ui.indent(-8.0);
        }

        ui.spacing(4.0);

        if ui.collapsing_header("Properties", true) {
            ui.indent(8.0);
            ui.toggle_with_label("Visible:", &mut self.obj_visible);
            ui.toggle_with_label("Locked:", &mut self.obj_locked);
            ui.spacing(4.0);
            ui.text_input_with_label("Name:", &mut self.text_multi);
            ui.indent(-8.0);
        }

        ui.spacing(4.0);

        if ui.collapsing_header("Material", true) {
            ui.indent(8.0);
            ui.label("Color");
            ui.color_picker("mat_color", &mut self.color);
            ui.spacing(4.0);
            ui.slider_with_label("Roughness:", &mut self.slider_float, 0.0, 1.0);
            ui.indent(-8.0);
        }

        ui.end_panel();

        ui.begin_central_panel("Central");

        ui.begin_horizontal_at_cursor();
        ui.begin_tab_bar("central_tabs");
        let tab_gallery = ui.tab("Widget Gallery");
        let tab_info = ui.tab("Info");
        ui.end_tab_bar();
        ui.end_horizontal();

        let available_height = ui.screen_size.y - 200.0;

        if tab_gallery {
            ui.begin_scroll_area(
                "gallery_scroll",
                Vec2::new(ui.available_width() - 20.0, available_height),
            );

            ui.heading("Widget Gallery");
            ui.label("Comprehensive demonstration of all UI widgets.");
            ui.label("Scroll down to see all widgets.");
            ui.separator();

            if ui.collapsing_header("Buttons", true) {
                ui.indent(8.0);
                ui.begin_horizontal_at_cursor();
                if ui.button("Click Me").clicked {
                    self.button_count += 1;
                    self.log_messages
                        .push(format!("[INFO] Button clicked {} times", self.button_count));
                }
                if ui.button("Reset").clicked {
                    self.button_count = 0;
                    self.log_messages.push("[INFO] Counter reset".to_string());
                }
                ui.end_horizontal();
                ui.label(&format!("Click count: {}", self.button_count));
                ui.indent(-8.0);
            }

            if ui.collapsing_header("Sliders", true) {
                ui.indent(8.0);
                ui.slider_with_label("Float:", &mut self.slider_float, 0.0, 1.0);
                ui.label(&format!("Value: {:.3}", self.slider_float));
                ui.spacing(4.0);
                ui.slider_with_label("Range:", &mut self.slider_int, 0.0, 100.0);
                ui.label(&format!("Value: {:.0}", self.slider_int));
                ui.indent(-8.0);
            }

            if ui.collapsing_header("Toggles & Checkboxes", true) {
                ui.indent(8.0);
                ui.toggle_with_label("Toggle A:", &mut self.toggle_a);
                ui.toggle_with_label("Toggle B:", &mut self.toggle_b);
                ui.spacing(4.0);
                ui.checkbox("cb_a", "Checkbox A", &mut self.checkbox_a);
                ui.checkbox("cb_b", "Checkbox B", &mut self.checkbox_b);
                ui.checkbox("cb_c", "Checkbox C", &mut self.checkbox_c);
                ui.indent(-8.0);
            }

            if ui.collapsing_header("Radio Buttons", true) {
                ui.indent(8.0);
                ui.radio_value("Option 1", &mut self.radio_choice, 0);
                ui.radio_value("Option 2", &mut self.radio_choice, 1);
                ui.radio_value("Option 3", &mut self.radio_choice, 2);
                ui.label(&format!("Selected: Option {}", self.radio_choice + 1));
                ui.indent(-8.0);
            }

            if ui.collapsing_header("Dropdowns", true) {
                ui.indent(8.0);
                let options = &["Apple", "Banana", "Cherry", "Date", "Elderberry"];
                ui.dropdown_with_label("Fruit:", options, &mut self.dropdown_choice);
                ui.label(&format!("Selected: {}", options[self.dropdown_choice]));
                ui.indent(-8.0);
            }

            if ui.collapsing_header("Text Input", true) {
                ui.indent(8.0);
                ui.text_input_with_label("Text:", &mut self.text_single);
                ui.label(&format!("Length: {} chars", self.text_single.len()));
                ui.indent(-8.0);
            }

            if ui.collapsing_header("Progress", true) {
                ui.indent(8.0);
                ui.label("Animated progress bar:");
                ui.progress_bar(self.progress, 200.0);
                ui.label(&format!("{:.0}%", self.progress * 100.0));
                ui.indent(-8.0);
            }

            if ui.collapsing_header("Color Picker", true) {
                ui.indent(8.0);
                ui.color_picker("gallery_color", &mut self.color);
                ui.label(&format!(
                    "RGBA: ({:.2}, {:.2}, {:.2}, {:.2})",
                    self.color.x, self.color.y, self.color.z, self.color.w
                ));
                ui.indent(-8.0);
            }

            ui.end_scroll_area();
        }

        if tab_info {
            ui.heading("Nightshade UI");
            ui.separator();
            ui.label("An immediate-mode UI system.");
            ui.spacing(8.0);
            ui.label("Features:");
            ui.label("  - Draggable & resizable panels");
            ui.label("  - Docking system");
            ui.label("  - Comprehensive widgets");
            ui.label("  - Scroll areas");
            ui.label("  - Tab bars");
        }

        ui.end_central_panel();

        if self.show_about {
            ui.begin_panel("About", Rect::new(400.0, 200.0, 300.0, 180.0));
            ui.heading("Nightshade UI");
            ui.separator();
            ui.label("An immediate-mode UI system.");
            ui.spacing(8.0);
            ui.label("Features:");
            ui.label("  - Draggable & resizable panels");
            ui.label("  - Docking system");
            ui.label("  - Comprehensive widgets");
            ui.spacing(8.0);
            if ui.button("Close").clicked {
                self.show_about = false;
            }
            ui.end_panel();
        }
    }

    fn run_systems(&mut self, _world: &mut World) {}
}
