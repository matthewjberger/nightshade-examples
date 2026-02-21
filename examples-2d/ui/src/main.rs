use nightshade::prelude::*;
use nightshade::render::wgpu::passes::geometry::UiRect;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(UiDemoState::default())
}

const FILE_ITEMS: &[&str] = &[
    "New Project",
    "Open...",
    "Save",
    "Save As...",
    "Export",
    "Exit",
];

const EDIT_ITEMS: &[&str] = &[
    "Undo",
    "Redo",
    "Cut",
    "Copy",
    "Paste",
    "Delete",
    "Select All",
];

const VIEW_ITEMS: &[&str] = &["Color Mixer", "Metrics", "Fullscreen"];

const HELP_ITEMS: &[&str] = &["Documentation", "About"];

const TOOL_NAMES: &[&str] = &["Select", "Move", "Rotate", "Scale", "Paint"];

const DROPDOWN_OPTIONS: &[&str] = &["Apple", "Banana", "Cherry", "Date", "Elderberry"];

struct UiDemoState {
    ui_only_mode: bool,
    log: Vec<(String, Vec4)>,
    console_input: String,

    active_tool: u32,
    snap_to_grid: bool,
    show_guides: bool,
    show_axes: bool,
    wireframe: bool,

    transform_x: f32,
    transform_y: f32,
    transform_z: f32,
    object_visible: bool,
    object_locked: bool,
    object_name: String,
    material_color: Vec4,
    roughness: f32,
    metallic: f32,

    anim_speed: f32,
    anim_target: f32,

    button_count: u32,
    slider_float: f32,
    slider_range: f32,
    toggle_a: bool,
    toggle_b: bool,
    checkbox_a: bool,
    checkbox_b: bool,
    checkbox_c: bool,
    radio_choice: u32,
    dropdown_choice: usize,
    text_value: String,
    gallery_color: Vec4,

    progress: f32,
    progress_dir: f32,

    color_from: Vec4,
    color_to: Vec4,
    blend_factor: f32,

    show_color_mixer: bool,
    show_metrics: bool,

    clip_enabled: bool,
    clip_width: f32,

    scope_a_toggle: bool,
    scope_b_toggle: bool,

    elapsed: f32,
}

impl Default for UiDemoState {
    fn default() -> Self {
        Self {
            ui_only_mode: true,
            log: vec![
                (
                    "[SYS] Application started".into(),
                    Vec4::new(0.7, 0.7, 0.7, 1.0),
                ),
                (
                    "[SYS] UI system initialized".into(),
                    Vec4::new(0.7, 0.7, 0.7, 1.0),
                ),
                (
                    "[SYS] Rendering engine ready".into(),
                    Vec4::new(0.7, 0.7, 0.7, 1.0),
                ),
            ],
            console_input: String::new(),

            active_tool: 0,
            snap_to_grid: true,
            show_guides: true,
            show_axes: false,
            wireframe: false,

            transform_x: 0.0,
            transform_y: 1.5,
            transform_z: 0.0,
            object_visible: true,
            object_locked: false,
            object_name: "Cube.001".into(),
            material_color: Vec4::new(0.3, 0.6, 0.9, 1.0),
            roughness: 0.5,
            metallic: 0.0,

            anim_speed: 4.0,
            anim_target: 0.0,

            button_count: 0,
            slider_float: 0.5,
            slider_range: 50.0,
            toggle_a: true,
            toggle_b: false,
            checkbox_a: false,
            checkbox_b: true,
            checkbox_c: false,
            radio_choice: 0,
            dropdown_choice: 0,
            text_value: "Hello, World!".into(),
            gallery_color: Vec4::new(0.3, 0.6, 0.9, 1.0),

            progress: 0.0,
            progress_dir: 1.0,

            color_from: Vec4::new(0.9, 0.2, 0.2, 1.0),
            color_to: Vec4::new(0.2, 0.4, 0.9, 1.0),
            blend_factor: 0.5,

            show_color_mixer: false,
            show_metrics: false,

            clip_enabled: true,
            clip_width: 150.0,

            scope_a_toggle: false,
            scope_b_toggle: true,

            elapsed: 0.0,
        }
    }
}

impl UiDemoState {
    fn log_message(&mut self, msg: &str, color: Vec4) {
        self.log.push((msg.into(), color));
    }

    fn menu_bar(&mut self, world: &mut World, ui: &mut ImmediateUi) {
        let theme = ui.theme_state.active_theme().clone();

        ui.begin_top_panel("Menu Bar", 38.0);
        ui.begin_horizontal_at_cursor();

        if let Some(index) = ui.menu("File", FILE_ITEMS) {
            self.log_message(
                &format!("[CMD] File > {}", FILE_ITEMS[index]),
                theme.text_color_accent,
            );
        }

        if let Some(index) = ui.menu("Edit", EDIT_ITEMS) {
            self.log_message(
                &format!("[CMD] Edit > {}", EDIT_ITEMS[index]),
                theme.text_color_accent,
            );
        }

        if let Some(index) = ui.menu("View", VIEW_ITEMS) {
            match index {
                0 => self.show_color_mixer = !self.show_color_mixer,
                1 => self.show_metrics = !self.show_metrics,
                _ => self.log_message(
                    &format!("[CMD] View > {}", VIEW_ITEMS[index]),
                    theme.text_color_accent,
                ),
            }
        }

        if let Some(index) = ui.menu("Help", HELP_ITEMS) {
            self.log_message(
                &format!("[CMD] Help > {}", HELP_ITEMS[index]),
                theme.text_color_accent,
            );
        }

        ui.spacing(20.0);
        ui.theme_dropdown();

        let fps = world.resources.window.timing.frames_per_second;
        let frame_time = world.resources.window.timing.delta_time * 1000.0;
        let fps_text = format!("{:.1} FPS ({:.1}ms)", fps, frame_time);
        let fps_width = ui.measure_text_width(&fps_text, 18.0);
        ui.spring(fps_width);
        ui.label(&fps_text);

        ui.end_horizontal();
        ui.end_panel();
    }

    fn toolbox_panel(&mut self, world: &mut World, ui: &mut ImmediateUi) {
        let theme = ui.theme_state.active_theme().clone();

        ui.begin_left_panel("Toolbox", 200.0);

        if ui.collapsing_header("Tools", true) {
            ui.indent(8.0);
            for (index, tool) in TOOL_NAMES.iter().enumerate() {
                ui.radio_value(tool, &mut self.active_tool, index as u32);
            }
            ui.indent(-8.0);
        }

        ui.spacing(4.0);

        if ui.collapsing_header("Options", true) {
            ui.indent(8.0);
            ui.checkbox("opt_snap", "Snap to Grid", &mut self.snap_to_grid);
            ui.checkbox("opt_guides", "Show Guides", &mut self.show_guides);
            ui.checkbox("opt_axes", "Show Axes", &mut self.show_axes);
            ui.checkbox("opt_wire", "Wireframe", &mut self.wireframe);
            ui.indent(-8.0);
        }

        ui.spacing(4.0);

        if ui.collapsing_header("Display", false) {
            ui.indent(8.0);
            let previous = self.ui_only_mode;
            ui.toggle_with_label("UI Only:", &mut self.ui_only_mode);
            if self.ui_only_mode != previous {
                world.resources.graphics.show_grid = !self.ui_only_mode;
                world.resources.graphics.atmosphere = if self.ui_only_mode {
                    Atmosphere::None
                } else {
                    Atmosphere::Sky
                };
            }
            ui.indent(-8.0);
        }

        ui.spacing(8.0);

        if ui.collapsing_header("Quick Actions", true) {
            ui.indent(8.0);
            ui.begin_horizontal_at_cursor();
            if ui.button_with_color("Run", theme.success_color).clicked {
                self.log_message("[CMD] Run triggered", theme.success_color);
            }
            if ui.button_with_color("Stop", theme.error_color).clicked {
                self.log_message("[CMD] Stop triggered", theme.error_color);
            }
            ui.end_horizontal();
            ui.indent(-8.0);
        }

        ui.end_panel();
    }

    fn inspector_panel(&mut self, ui: &mut ImmediateUi) {
        let theme = ui.theme_state.active_theme().clone();

        ui.begin_right_panel("Inspector", 280.0);

        ui.begin_horizontal_at_cursor();
        ui.begin_tab_bar("inspector_tabs");
        let tab_properties = ui.tab("Properties");
        let tab_animation = ui.tab("Animation");
        ui.end_tab_bar();
        ui.end_horizontal();

        if tab_properties {
            if ui.collapsing_header("Transform", true) {
                ui.indent(8.0);
                let response_x = ui.slider_with_label("X:", &mut self.transform_x, -10.0, 10.0);
                ui.tooltip(&response_x, "World X position");
                let response_y = ui.slider_with_label("Y:", &mut self.transform_y, -10.0, 10.0);
                ui.tooltip(&response_y, "World Y position");
                let response_z = ui.slider_with_label("Z:", &mut self.transform_z, -10.0, 10.0);
                ui.tooltip(&response_z, "World Z position");
                ui.indent(-8.0);
            }

            ui.spacing(4.0);

            if ui.collapsing_header("Properties", true) {
                ui.indent(8.0);
                ui.toggle_with_label("Visible:", &mut self.object_visible);
                ui.toggle_with_label("Locked:", &mut self.object_locked);
                ui.spacing(4.0);
                ui.text_input_with_label("Name:", &mut self.object_name);
                ui.indent(-8.0);
            }

            ui.spacing(4.0);

            if ui.collapsing_header("Material", true) {
                ui.indent(8.0);
                ui.label("Color");
                ui.color_picker("mat_color", &mut self.material_color);
                ui.spacing(4.0);
                ui.slider_with_label("Roughness:", &mut self.roughness, 0.0, 1.0);
                ui.slider_with_label("Metallic:", &mut self.metallic, 0.0, 1.0);
                ui.indent(-8.0);
            }
        }

        if tab_animation {
            ui.spacing(4.0);
            ui.slider_with_label("Speed:", &mut self.anim_speed, 0.5, 12.0);
            ui.slider_with_label("Target:", &mut self.anim_target, 0.0, 1.0);

            let animated_value = ui.animate(
                UiId::new("inspector_anim"),
                ANIM_OPEN,
                self.anim_target,
                self.anim_speed,
            );

            ui.spacing(4.0);
            ui.label(&format!(
                "Target: {:.2}  Current: {:.2}",
                self.anim_target, animated_value
            ));

            ui.spacing(8.0);
            ui.label("Blended color:");
            let blended =
                ImmediateUi::blend_color(theme.accent_color, theme.success_color, animated_value);
            let cursor = ui.cursor();
            ui.draw_rect(
                cursor,
                Vec2::new(ui.available_width() - 16.0, 24.0),
                blended,
            );
            ui.spacing(32.0);

            ui.spacing(8.0);
            ui.label("Progress bars:");
            let phase_a = (self.elapsed * 0.5).sin() * 0.5 + 0.5;
            let phase_b = (self.elapsed * 0.5 + 2.0).sin() * 0.5 + 0.5;
            let phase_c = (self.elapsed * 0.5 + 4.0).sin() * 0.5 + 0.5;
            let bar_width = ui.available_width() - 16.0;
            ui.progress_bar_colored(phase_a, bar_width, theme.success_color);
            ui.spacing(4.0);
            ui.progress_bar_colored(phase_b, bar_width, theme.warning_color);
            ui.spacing(4.0);
            ui.progress_bar_colored(phase_c, bar_width, theme.error_color);
        }

        ui.end_panel();
    }

    fn console_panel(&mut self, ui: &mut ImmediateUi) {
        let theme = ui.theme_state.active_theme().clone();

        ui.begin_bottom_panel("Console", 120.0);

        ui.begin_horizontal_at_cursor();
        ui.text_input("console_input", &mut self.console_input);
        if ui.button("Send").clicked && !self.console_input.is_empty() {
            let msg = format!("[USR] {}", self.console_input);
            self.log_message(&msg, theme.success_color);
            self.console_input.clear();
        }
        if ui.button("Clear").clicked {
            self.log.clear();
            self.log_message("[SYS] Console cleared", theme.text_color_disabled);
        }
        ui.end_horizontal();

        ui.separator();

        let scroll_height = 50.0;
        let scroll_width = ui.available_width() - 8.0;
        ui.begin_scroll_area("console_scroll", Vec2::new(scroll_width, scroll_height));
        for (message, color) in self.log.clone() {
            ui.label_colored(&message, color);
        }
        ui.end_scroll_area();

        ui.end_panel();
    }

    fn central_content(&mut self, ui: &mut ImmediateUi) {
        ui.begin_central_panel("Central");

        ui.begin_horizontal_at_cursor();
        ui.begin_tab_bar("central_tabs");
        let tab_gallery = ui.tab("Gallery");
        let tab_drawing = ui.tab("Drawing");
        let tab_theme = ui.tab("Theme");
        ui.end_tab_bar();
        ui.end_horizontal();

        let available_height = ui.screen_size.y - 200.0;

        if tab_gallery {
            self.widget_gallery(ui, available_height);
        }

        if tab_drawing {
            self.custom_drawing(ui, available_height);
        }

        if tab_theme {
            self.theme_showcase(ui, available_height);
        }

        ui.end_central_panel();
    }

    fn widget_gallery(&mut self, ui: &mut ImmediateUi, available_height: f32) {
        let theme = ui.theme_state.active_theme().clone();

        ui.begin_scroll_area(
            "gallery_scroll",
            Vec2::new(ui.available_width() - 20.0, available_height),
        );

        ui.heading("Widget Gallery");
        ui.label("Comprehensive demonstration of all UI widgets.");
        ui.separator();

        if ui.collapsing_header("Buttons", true) {
            ui.indent(8.0);

            ui.begin_horizontal_at_cursor();
            let click_response = ui.button("Click Me");
            if click_response.clicked {
                self.button_count += 1;
            }
            if click_response.double_clicked {
                self.button_count += 10;
            }
            ui.tooltip(&click_response, "Click or double-click me");

            let reset_response = ui.button("Reset");
            if reset_response.clicked {
                self.button_count = 0;
            }
            ui.tooltip(&reset_response, "Reset click counter to zero");
            ui.end_horizontal();

            ui.label(&format!("Click count: {}", self.button_count));
            ui.spacing(4.0);

            ui.begin_horizontal_at_cursor();
            if ui.button_with_color("Success", theme.success_color).clicked {
                self.log_message("[CMD] Success button", theme.success_color);
            }
            if ui.button_with_color("Warning", theme.warning_color).clicked {
                self.log_message("[WRN] Warning button", theme.warning_color);
            }
            if ui.button_with_color("Danger", theme.error_color).clicked {
                self.log_message("[ERR] Danger button", theme.error_color);
            }
            ui.end_horizontal();

            ui.indent(-8.0);
        }

        if ui.collapsing_header("Sliders", true) {
            ui.indent(8.0);
            ui.slider_with_label("Float:", &mut self.slider_float, 0.0, 1.0);
            ui.label(&format!("Value: {:.3}", self.slider_float));
            ui.spacing(4.0);
            ui.slider_with_label("Range:", &mut self.slider_range, 0.0, 100.0);
            ui.label(&format!("Value: {:.0}", self.slider_range));
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Toggles & Checkboxes", true) {
            ui.indent(8.0);
            let toggle_a_response = ui.toggle_with_label("Toggle A:", &mut self.toggle_a);
            ui.tooltip(&toggle_a_response, "First toggle switch");
            let toggle_b_response = ui.toggle_with_label("Toggle B:", &mut self.toggle_b);
            ui.tooltip(&toggle_b_response, "Second toggle switch");
            ui.spacing(4.0);
            ui.checkbox("cb_a", "Checkbox A", &mut self.checkbox_a);
            ui.checkbox("cb_b", "Checkbox B", &mut self.checkbox_b);
            ui.checkbox("cb_c", "Checkbox C", &mut self.checkbox_c);
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Radio Buttons", true) {
            ui.indent(8.0);
            for index in 0..5u32 {
                let label = format!("Option {}", index + 1);
                ui.radio_value(&label, &mut self.radio_choice, index);
            }
            ui.label(&format!("Selected: Option {}", self.radio_choice + 1));
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Dropdowns", true) {
            ui.indent(8.0);
            ui.dropdown_with_label("Fruit:", DROPDOWN_OPTIONS, &mut self.dropdown_choice);
            ui.label(&format!(
                "Selected: {}",
                DROPDOWN_OPTIONS[self.dropdown_choice]
            ));
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Text Input", true) {
            ui.indent(8.0);
            ui.text_input_with_label("Text:", &mut self.text_value);
            ui.label(&format!("Length: {} chars", self.text_value.len()));
            ui.spacing(4.0);
            ui.label("Arrow keys move cursor, Shift+Arrow selects");
            ui.label("Ctrl+A select all, Ctrl+Backspace delete word");
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Progress Bars", true) {
            ui.indent(8.0);
            let bar_width = 200.0;
            ui.label("Animated:");
            ui.progress_bar(self.progress, bar_width);
            ui.label(&format!("{:.0}%", self.progress * 100.0));
            ui.spacing(4.0);
            ui.label("Colored:");
            ui.progress_bar_colored(0.7, bar_width, theme.success_color);
            ui.spacing(2.0);
            ui.progress_bar_colored(0.4, bar_width, theme.warning_color);
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Color Picker", true) {
            ui.indent(8.0);
            ui.color_picker("gallery_color", &mut self.gallery_color);
            ui.label(&format!(
                "RGBA: ({:.2}, {:.2}, {:.2}, {:.2})",
                self.gallery_color.x,
                self.gallery_color.y,
                self.gallery_color.z,
                self.gallery_color.w,
            ));
            ui.indent(-8.0);
        }

        ui.end_scroll_area();
    }

    fn custom_drawing(&mut self, ui: &mut ImmediateUi, available_height: f32) {
        let theme = ui.theme_state.active_theme().clone();

        ui.begin_scroll_area(
            "drawing_scroll",
            Vec2::new(ui.available_width() - 20.0, available_height),
        );

        ui.heading("Custom Drawing & Advanced APIs");
        ui.separator();

        if ui.collapsing_header("Custom Rectangles", true) {
            ui.indent(8.0);
            let cursor = ui.cursor();
            ui.draw_rect(cursor, Vec2::new(60.0, 30.0), theme.accent_color);
            ui.draw_rect(
                cursor + Vec2::new(70.0, 0.0),
                Vec2::new(60.0, 30.0),
                theme.success_color,
            );
            ui.draw_rect(
                cursor + Vec2::new(140.0, 0.0),
                Vec2::new(60.0, 30.0),
                theme.warning_color,
            );
            ui.spacing(38.0);

            let rounded_cursor = ui.cursor();
            ui.add_rect_raw(UiRect {
                position: rounded_cursor,
                size: Vec2::new(180.0, 36.0),
                color: theme.background_color_hovered,
                corner_radius: 12.0,
                border_width: 2.0,
                border_color: theme.accent_color,
                ..Default::default()
            });
            ui.spacing(44.0);

            ui.indent(-8.0);
        }

        if ui.collapsing_header("Clip Regions", true) {
            ui.indent(8.0);
            ui.checkbox("clip_enabled", "Enable clipping", &mut self.clip_enabled);
            ui.slider_with_label("Clip width:", &mut self.clip_width, 40.0, 300.0);
            ui.spacing(4.0);

            let clip_origin = ui.cursor();
            if self.clip_enabled {
                ui.push_clip(Rect::new(
                    clip_origin.x,
                    clip_origin.y,
                    self.clip_width,
                    40.0,
                ));
            }

            ui.draw_rect(clip_origin, Vec2::new(100.0, 30.0), theme.error_color);
            ui.draw_rect(
                clip_origin + Vec2::new(80.0, 0.0),
                Vec2::new(100.0, 30.0),
                theme.success_color,
            );
            ui.draw_rect(
                clip_origin + Vec2::new(160.0, 0.0),
                Vec2::new(100.0, 30.0),
                theme.accent_color,
            );

            if self.clip_enabled {
                ui.pop_clip();
            }
            ui.spacing(38.0);

            ui.indent(-8.0);
        }

        if ui.collapsing_header("ID Scoping", true) {
            ui.indent(8.0);
            ui.label("Two toggles with same widget ID, scoped differently:");
            ui.spacing(4.0);

            ui.push_id("scope_a");
            ui.toggle_with_label("Toggle:", &mut self.scope_a_toggle);
            ui.pop_id();

            ui.push_id("scope_b");
            ui.toggle_with_label("Toggle:", &mut self.scope_b_toggle);
            ui.pop_id();

            ui.label(&format!(
                "Scope A: {}  Scope B: {}",
                self.scope_a_toggle, self.scope_b_toggle,
            ));
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Layout Alignment", true) {
            ui.indent(8.0);
            let start = ui.cursor();
            let width = ui.available_width() - 24.0;
            ui.begin_vertical(start, width);

            ui.set_alignment(LayoutAlignment::Start);
            ui.label("Left-aligned (Start)");

            ui.set_alignment(LayoutAlignment::Center);
            ui.label("Center-aligned");

            ui.set_alignment(LayoutAlignment::End);
            ui.label("Right-aligned (End)");

            ui.set_alignment(LayoutAlignment::Start);
            ui.end_vertical();
            ui.spacing(60.0);
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Layer Control", true) {
            ui.indent(8.0);
            ui.label("Orange rect drawn on FloatingPanels layer:");
            let layer_cursor = ui.cursor();
            ui.draw_rect(
                layer_cursor,
                Vec2::new(120.0, 30.0),
                theme.background_color_active,
            );
            ui.set_layer(UiLayer::FloatingPanels);
            ui.draw_rect(
                layer_cursor + Vec2::new(20.0, 5.0),
                Vec2::new(80.0, 20.0),
                Vec4::new(1.0, 0.6, 0.2, 0.9),
            );
            ui.set_layer(UiLayer::Background);
            ui.spacing(38.0);
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Animation API", true) {
            ui.indent(8.0);

            let hover_target = (self.elapsed.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            let animated_offset = ui.animate(
                UiId::new("drawing_anim_pos"),
                ANIM_HOVER,
                hover_target * 200.0,
                6.0,
            );

            let anim_cursor = ui.cursor();
            ui.draw_rect(
                anim_cursor + Vec2::new(animated_offset, 0.0),
                Vec2::new(40.0, 24.0),
                theme.accent_color,
            );
            ui.spacing(32.0);

            let cycle = (self.elapsed * 0.8).sin() * 0.5 + 0.5;
            let color_a = ImmediateUi::blend_color(theme.accent_color, theme.error_color, cycle);
            let color_b = ImmediateUi::blend_color(theme.success_color, theme.warning_color, cycle);
            let blend_cursor = ui.cursor();
            ui.draw_rect(blend_cursor, Vec2::new(80.0, 24.0), color_a);
            ui.draw_rect(
                blend_cursor + Vec2::new(90.0, 0.0),
                Vec2::new(80.0, 24.0),
                color_b,
            );
            ui.spacing(32.0);

            ui.indent(-8.0);
        }

        ui.end_scroll_area();
    }

    fn theme_showcase(&mut self, ui: &mut ImmediateUi, available_height: f32) {
        let theme = ui.theme_state.active_theme().clone();

        ui.begin_scroll_area(
            "theme_scroll",
            Vec2::new(ui.available_width() - 20.0, available_height),
        );

        ui.heading("Theme Showcase");
        ui.label(&format!("Current theme: {}", theme.name));
        ui.separator();

        if ui.collapsing_header("Core Colors", true) {
            ui.indent(8.0);
            let swatch_size = Vec2::new(50.0, 20.0);

            let entries: &[(&str, Vec4)] = &[
                ("Panel", theme.panel_color),
                ("Accent", theme.accent_color),
                ("Success", theme.success_color),
                ("Warning", theme.warning_color),
                ("Error", theme.error_color),
                ("Selection", theme.selection_color),
                ("Slider Fill", theme.slider_fill_color),
                ("Input BG", theme.input_background_color),
                ("Scrollbar", theme.scrollbar_color),
            ];

            for (name, color) in entries {
                let cursor = ui.cursor();
                ui.draw_rect(cursor, swatch_size, *color);
                ui.add_rect_raw(UiRect {
                    position: cursor,
                    size: swatch_size,
                    color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                    corner_radius: 0.0,
                    border_width: 1.0,
                    border_color: theme.border_color,
                    ..Default::default()
                });
                ui.spacing(28.0);
                ui.label(name);
            }
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Text Colors", true) {
            ui.indent(8.0);
            ui.label_colored("Normal text color", theme.text_color);
            ui.label_colored("Disabled text color", theme.text_color_disabled);
            ui.label_colored("Accent text color", theme.text_color_accent);
            ui.indent(-8.0);
        }

        if ui.collapsing_header("Background Colors", true) {
            ui.indent(8.0);
            let swatch_size = Vec2::new(80.0, 24.0);

            let cursor_normal = ui.cursor();
            ui.draw_rect(cursor_normal, swatch_size, theme.background_color);
            ui.spacing(32.0);
            ui.label("Normal");

            let cursor_hovered = ui.cursor();
            ui.draw_rect(cursor_hovered, swatch_size, theme.background_color_hovered);
            ui.spacing(32.0);
            ui.label("Hovered");

            let cursor_active = ui.cursor();
            ui.draw_rect(cursor_active, swatch_size, theme.background_color_active);
            ui.spacing(32.0);
            ui.label("Active");

            ui.indent(-8.0);
        }

        if ui.collapsing_header("Border Colors", true) {
            ui.indent(8.0);
            let border_cursor = ui.cursor();
            ui.add_rect_raw(UiRect {
                position: border_cursor,
                size: Vec2::new(80.0, 24.0),
                color: theme.background_color,
                corner_radius: 4.0,
                border_width: 2.0,
                border_color: theme.border_color,
                ..Default::default()
            });
            ui.spacing(32.0);
            ui.label("Normal border");

            let focused_cursor = ui.cursor();
            ui.add_rect_raw(UiRect {
                position: focused_cursor,
                size: Vec2::new(80.0, 24.0),
                color: theme.background_color,
                corner_radius: 4.0,
                border_width: 2.0,
                border_color: theme.border_color_focused,
                ..Default::default()
            });
            ui.spacing(32.0);
            ui.label("Focused border");

            ui.indent(-8.0);
        }

        ui.end_scroll_area();
    }

    fn color_mixer_window(&mut self, ui: &mut ImmediateUi) {
        if !self.show_color_mixer {
            return;
        }

        ui.begin_panel("Color Mixer", Rect::new(300.0, 200.0, 320.0, 280.0));

        ui.label("From:");
        ui.color_picker("mixer_from", &mut self.color_from);
        ui.spacing(4.0);
        ui.label("To:");
        ui.color_picker("mixer_to", &mut self.color_to);
        ui.spacing(4.0);
        ui.slider_with_label("Blend:", &mut self.blend_factor, 0.0, 1.0);
        ui.spacing(4.0);

        let swatch_width = 40.0;
        let swatch_height = 24.0;
        let cursor = ui.cursor();
        for step in 0..5 {
            let t = step as f32 / 4.0;
            let blended = ImmediateUi::blend_color(self.color_from, self.color_to, t);
            ui.draw_rect(
                cursor + Vec2::new(step as f32 * (swatch_width + 4.0), 0.0),
                Vec2::new(swatch_width, swatch_height),
                blended,
            );
        }
        ui.spacing(swatch_height + 8.0);

        let auto_t = (self.elapsed.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let auto_blended = ImmediateUi::blend_color(self.color_from, self.color_to, auto_t);
        let auto_cursor = ui.cursor();
        ui.draw_rect(
            auto_cursor,
            Vec2::new(ui.available_width() - 16.0, 20.0),
            auto_blended,
        );
        ui.spacing(28.0);

        if ui.button("Close").clicked {
            self.show_color_mixer = false;
        }

        ui.end_panel();
    }

    fn metrics_window(&mut self, ui: &mut ImmediateUi, fps: f32, frame_time: f32) {
        if !self.show_metrics {
            return;
        }

        ui.begin_panel("Metrics", Rect::new(350.0, 250.0, 300.0, 220.0));

        ui.label(&format!("FPS: {:.1}  Frame: {:.2}ms", fps, frame_time));
        ui.spacing(4.0);

        let sample_text = "Sample Text";
        let text_width = ui.measure_text_width(sample_text, 18.0);
        ui.label(&format!(
            "measure_text_width(\"{}\", 18) = {:.1}",
            sample_text, text_width
        ));

        let text_height = ui.measure_text_height(18.0);
        ui.label(&format!("measure_text_height(18) = {:.1}", text_height));

        ui.label(&format!(
            "screen_size: {:.0} x {:.0}",
            ui.screen_size.x, ui.screen_size.y
        ));
        ui.label(&format!("available_width: {:.0}", ui.available_width()));
        ui.label(&format!(
            "cursor: ({:.0}, {:.0})",
            ui.cursor().x,
            ui.cursor().y
        ));

        ui.spacing(8.0);
        if ui.button("Close").clicked {
            self.show_metrics = false;
        }

        ui.end_panel();
    }
}

impl State for UiDemoState {
    fn title(&self) -> &str {
        "Nightshade UI Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;

        let camera_position = Vec3::new(0.0, 4.0, 10.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);
    }

    fn immediate_ui(&mut self, world: &mut World, ui: &mut ImmediateUi) {
        let delta_time = world.resources.window.timing.delta_time;
        self.elapsed += delta_time;

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

        self.menu_bar(world, ui);
        self.toolbox_panel(world, ui);
        self.inspector_panel(ui);
        self.console_panel(ui);
        self.central_content(ui);
        self.color_mixer_window(ui);
        self.metrics_window(ui, fps, frame_time);
    }

    fn run_systems(&mut self, _world: &mut World) {}
}
