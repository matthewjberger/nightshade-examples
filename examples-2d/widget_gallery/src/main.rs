use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Gallery::default())
}

const SECTION_NAMES: &[&str] = &[
    "Typography",
    "Buttons",
    "Text Inputs",
    "Sliders",
    "Toggles",
    "Dropdowns",
    "Tabs",
    "Lists & Trees",
    "Data Grid",
    "Scroll Areas",
    "Modals & Dialogs",
    "Toasts",
    "Rich Text",
    "Composites",
    "Animations",
    "Layout",
    "Themes",
    "Command Palette",
    "Canvas",
];

struct Vec3Editor {
    x_drag: Entity,
    y_drag: Entity,
    z_drag: Entity,
}

impl CompositeWidget for Vec3Editor {
    type Value = nalgebra_glm::Vec3;

    fn build(tree: &mut UiTreeBuilder) -> Self {
        let container = tree.current_parent();
        let input_height = tree
            .world_mut()
            .resources
            .retained_ui
            .theme_state
            .active_theme()
            .button_height;
        if let Some(node) = tree.world_mut().get_ui_layout_node_mut(container) {
            node.flow_layout = Some(FlowLayout {
                direction: FlowDirection::Horizontal,
                padding: 0.0,
                spacing: 4.0,
                alignment: FlowAlignment::Start,
                cross_alignment: FlowAlignment::Start,
                wrap: false,
            });
        }

        tree.add_label("X");
        let x_drag = tree.add_drag_value(0.0, -1000.0, 1000.0, 0.1, 2);
        tree.add_label("Y");
        let y_drag = tree.add_drag_value(0.0, -1000.0, 1000.0, 0.1, 2);
        tree.add_label("Z");
        let z_drag = tree.add_drag_value(0.0, -1000.0, 1000.0, 0.1, 2);

        for &entity in &[x_drag, y_drag, z_drag] {
            if let Some(node) = tree.world_mut().get_ui_layout_node_mut(entity) {
                node.flow_child_size = Some(Ab(nalgebra_glm::Vec2::new(0.0, input_height)).into());
                node.flex_grow = Some(1.0);
            }
        }

        Self {
            x_drag,
            y_drag,
            z_drag,
        }
    }

    fn value(&self, world: &World) -> nalgebra_glm::Vec3 {
        nalgebra_glm::Vec3::new(
            world.ui_drag_value(self.x_drag),
            world.ui_drag_value(self.y_drag),
            world.ui_drag_value(self.z_drag),
        )
    }
}

#[derive(Default)]
struct Gallery {
    active_section: usize,
    nav_labels: Vec<Entity>,
    section_roots: Vec<Entity>,
    sidebar: Entity,

    click_count: u32,
    click_label_slot: usize,
    btn_counter: Entity,

    input_mirror_slot: usize,
    text_input: Entity,
    submit_input: Entity,
    submit_log_slot: usize,

    slider_val: f32,
    slider_entity: Entity,
    slider_label_slot: usize,
    range_slider: Entity,
    range_val: f32,
    range_label_slot: usize,
    drag_x: Entity,
    drag_y: Entity,
    drag_z: Entity,
    drag_x_val: f32,
    drag_y_val: f32,
    drag_z_val: f32,

    toggle_entity: Entity,
    toggle_val: bool,
    hidden_section: Entity,
    checkbox_a: Entity,
    checkbox_b: Entity,
    checkbox_c: Entity,
    checkbox_label_slot: usize,
    radio_label_slot: usize,

    dropdown_entity: Entity,
    dropdown_val: usize,
    dropdown_label_slot: usize,

    tab_bar: Entity,
    tab_contents: Vec<Entity>,

    tree_view: Entity,
    tree_selection_label_slot: usize,
    tree_filter_input: Entity,

    grid_small: Entity,
    grid_large: Entity,

    confirm_dialog: Entity,
    confirm_trigger: Entity,
    modal_dialog: Entity,
    modal_trigger: Entity,
    context_menu: Entity,

    toast_info_btn: Entity,
    toast_success_btn: Entity,
    toast_warning_btn: Entity,
    toast_error_btn: Entity,

    vec3_editor: Entity,
    vec3_label_slot: usize,

    anim_fade: Entity,
    anim_slide: Entity,
    anim_scale: Entity,
    anim_trigger: Entity,
    anim_visible: bool,

    progress_bar: Entity,
    progress_value: f32,

    disabled_button: Entity,
    disabled_slider: Entity,
    disabled_input: Entity,
    disable_toggle: Entity,
    disable_active: bool,

    region_disable_toggle: Entity,
    region_disable_active: bool,
    region_container: Entity,

    rich_btn_save: Entity,
    rich_btn_status: Entity,

    theme_dropdown: Entity,

    command_palette: Entity,
    command_palette_log_slot: usize,
    command_palette_trigger: Entity,

    canvas_entity: Entity,
    canvas_time: f32,

    log_slider_entity: Entity,
    log_slider_val: f32,
    log_slider_label_slot: usize,

    text_area_entity: Entity,
}

impl State for Gallery {
    fn title(&self) -> &str {
        "NIGHTSHADE // WIDGET GALLERY"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.retained_ui.enabled = true;
        let bg = world
            .resources
            .retained_ui
            .theme_state
            .active_theme()
            .background_color;
        world.resources.graphics.clear_color = [bg.x, bg.y, bg.z, bg.w];

        self.click_label_slot = world.resources.text_cache.add_text("Clicked 0 times");
        self.input_mirror_slot = world.resources.text_cache.add_text("");
        self.submit_log_slot = world.resources.text_cache.add_text("Submit a value...");
        self.slider_label_slot = world.resources.text_cache.add_text("0.50");
        self.range_label_slot = world.resources.text_cache.add_text("500");
        self.dropdown_label_slot = world.resources.text_cache.add_text("Apple");
        self.vec3_label_slot = world.resources.text_cache.add_text("(0.00, 0.00, 0.00)");
        self.checkbox_label_slot = world.resources.text_cache.add_text("A: off  B: on  C: off");
        self.radio_label_slot = world.resources.text_cache.add_text("Small");
        self.tree_selection_label_slot = world.resources.text_cache.add_text("(none)");
        self.command_palette_log_slot = world
            .resources
            .text_cache
            .add_text("Press Ctrl+P or click button");
        self.slider_val = 0.5;
        self.range_val = 500.0;
        self.log_slider_val = 0.01;
        self.log_slider_label_slot = world.resources.text_cache.add_text("0.010");
        self.toggle_val = true;
        self.anim_visible = true;

        let mut tree = UiTreeBuilder::new(world);

        let font_size = tree
            .world_mut()
            .resources
            .retained_ui
            .theme_state
            .active_theme()
            .font_size;

        let topbar_height = 36.0;

        let root_panel = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .without_pointer_events()
            .entity();
        tree.push_parent(root_panel);

        let topbar = tree
            .add_node()
            .boundary(
                Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Ab(nalgebra_glm::Vec2::new(0.0, topbar_height))
                    + Rl(nalgebra_glm::Vec2::new(100.0, 0.0)),
            )
            .flow_with_alignment(
                FlowDirection::Horizontal,
                12.0,
                0.0,
                FlowAlignment::Start,
                FlowAlignment::Center,
            )
            .with_rect(0.0, 1.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::PanelHeader)
            .with_theme_border_color(ThemeColor::Border)
            .with_depth(UiDepthMode::Set(5.0))
            .entity();
        tree.push_parent(topbar);

        let title_slot = tree
            .world_mut()
            .resources
            .text_cache
            .add_text("Widget Gallery");
        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(0.0, topbar_height)))
            .auto_size(AutoSizeMode::Width)
            .auto_size_padding(nalgebra_glm::Vec2::new(4.0, 0.0))
            .with_text_slot(title_slot, font_size * 1.1)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::TextAccent)
            .without_pointer_events()
            .done();

        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(0.0, topbar_height)))
            .flex_grow(1.0)
            .without_pointer_events()
            .done();

        let theme_label_slot = tree.world_mut().resources.text_cache.add_text("Theme:");
        tree.add_node()
            .flow_child(Ab(nalgebra_glm::Vec2::new(0.0, topbar_height)))
            .auto_size(AutoSizeMode::Width)
            .with_text_slot(theme_label_slot, font_size * 0.85)
            .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::Text)
            .without_pointer_events()
            .done();

        self.theme_dropdown = tree.add_theme_dropdown();

        tree.pop_parent();

        self.sidebar = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(0.0, topbar_height)),
                Ab(nalgebra_glm::Vec2::new(180.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 100.0)),
            )
            .flow(FlowDirection::Vertical, 10.0, 2.0)
            .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::Panel)
            .with_clip()
            .entity();
        tree.push_parent(self.sidebar);

        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Sections");
            ui.separator();
            for (index, name) in SECTION_NAMES.iter().enumerate() {
                self.nav_labels.push(ui.selectable_label(name, Some(0)));
                if index == 0 {
                    let entity = *self.nav_labels.last().unwrap();
                    ui.world_mut().ui_set_selected(entity, true);
                }
            }
        });

        tree.pop_parent();

        let content_area = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(182.0, topbar_height)),
                Ab(nalgebra_glm::Vec2::new(-182.0, 0.0))
                    + Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
            )
            .without_pointer_events()
            .entity();
        tree.push_parent(content_area);

        for section_index in 0..SECTION_NAMES.len() {
            let section = tree
                .add_node()
                .boundary(
                    Rl(nalgebra_glm::Vec2::new(0.0, 0.0)),
                    Rl(nalgebra_glm::Vec2::new(100.0, 100.0)),
                )
                .without_pointer_events()
                .entity();

            if section_index != 0 {
                tree.world_mut().ui_set_visible(section, false);
            }

            tree.push_parent(section);

            let scroll = tree.add_scroll_area_fill(12.0, 6.0);
            let scroll_content = tree
                .world_mut()
                .ui_scroll_area_content(scroll)
                .unwrap_or(scroll);
            tree.push_parent(scroll_content);

            match section_index {
                0 => self.build_typography(&mut tree),
                1 => self.build_buttons(&mut tree),
                2 => self.build_inputs(&mut tree),
                3 => self.build_sliders(&mut tree),
                4 => self.build_toggles(&mut tree),
                5 => self.build_dropdowns(&mut tree),
                6 => self.build_tabs(&mut tree),
                7 => self.build_trees(&mut tree),
                8 => self.build_data_grid(&mut tree),
                9 => self.build_scroll_areas(&mut tree),
                10 => self.build_modals(&mut tree),
                11 => self.build_toasts(&mut tree),
                12 => self.build_rich_text(&mut tree),
                13 => self.build_composites(&mut tree),
                14 => self.build_animations(&mut tree),
                15 => self.build_layout(&mut tree),
                16 => self.build_themes(&mut tree),
                17 => self.build_command_palette(&mut tree),
                18 => self.build_canvas(&mut tree),
                _ => {}
            }

            tree.pop_parent();
            tree.pop_parent();

            self.section_roots.push(section);
        }

        tree.pop_parent();
        tree.pop_parent();

        self.confirm_dialog =
            tree.add_confirm_dialog("Confirm Action", "Are you sure you want to proceed?");

        self.modal_dialog =
            tree.add_confirm_dialog("Custom Modal", "A modal with embedded widgets:");

        self.command_palette = tree.add_command_palette(10);

        self.context_menu = tree.add_context_menu_from_builder(
            ContextMenuBuilder::new()
                .item("Cut", "Ctrl+X")
                .item("Copy", "Ctrl+C")
                .item("Paste", "Ctrl+V")
                .separator()
                .submenu("Insert", |builder| {
                    builder.item("Text", "").item("Image", "").item("Link", "")
                })
                .separator()
                .item("Select All", "Ctrl+A"),
        );

        tree.finish();

        world.ui_command_palette_register(self.command_palette, "New File", "Ctrl+N", "File");
        world.ui_command_palette_register(self.command_palette, "Open File", "Ctrl+O", "File");
        world.ui_command_palette_register(self.command_palette, "Save", "Ctrl+S", "File");
        world.ui_command_palette_register(self.command_palette, "Undo", "Ctrl+Z", "Edit");
        world.ui_command_palette_register(self.command_palette, "Redo", "Ctrl+Y", "Edit");
        world.ui_command_palette_register(self.command_palette, "Find", "Ctrl+F", "Edit");
        world.ui_command_palette_register(self.command_palette, "Replace", "Ctrl+H", "Edit");
        world.ui_command_palette_register(self.command_palette, "Toggle Theme", "", "View");
    }

    fn run_systems(&mut self, world: &mut World) {
        if world.resources.retained_ui.active_modal.is_none() {
            escape_key_exit_system(world);
        }

        for (index, &nav) in self.nav_labels.iter().enumerate() {
            if world.ui_selectable_label_changed(nav)
                && world.ui_selectable_label_selected(nav)
                && index != self.active_section
            {
                world.ui_set_visible(self.section_roots[self.active_section], false);
                self.active_section = index;
                world.ui_set_visible(self.section_roots[index], true);
            }
        }

        if world.ui_button_clicked(self.btn_counter) {
            self.click_count += 1;
            world.ui_set_text(
                self.click_label_slot,
                &format!("Clicked {} times", self.click_count),
            );
        }

        if world.ui_text_input_changed(self.text_input) {
            let text = world.ui_text_input_value(self.text_input);
            world.ui_set_text(self.input_mirror_slot, &text);
        }

        if let Some(text) = world.ui_text_input_submitted(self.submit_input) {
            world.ui_set_text(self.submit_log_slot, &format!("Submitted: {text}"));
            world.ui_text_input_set_value(self.submit_input, "");
        }

        if world.ui_bind_slider(self.slider_entity, &mut self.slider_val) {
            world.ui_set_text(self.slider_label_slot, &format!("{:.2}", self.slider_val));
        }
        if world.ui_bind_slider(self.range_slider, &mut self.range_val) {
            world.ui_set_text(self.range_label_slot, &format!("{:.0}", self.range_val));
        }

        world.ui_bind_drag_value(self.drag_x, &mut self.drag_x_val);
        world.ui_bind_drag_value(self.drag_y, &mut self.drag_y_val);
        world.ui_bind_drag_value(self.drag_z, &mut self.drag_z_val);

        if world.ui_bind_toggle(self.toggle_entity, &mut self.toggle_val) {
            world.ui_set_visible(self.hidden_section, self.toggle_val);
        }

        if world.ui_checkbox_changed(self.checkbox_a)
            || world.ui_checkbox_changed(self.checkbox_b)
            || world.ui_checkbox_changed(self.checkbox_c)
        {
            let val_a = if world.ui_checkbox_value(self.checkbox_a) {
                "on"
            } else {
                "off"
            };
            let val_b = if world.ui_checkbox_value(self.checkbox_b) {
                "on"
            } else {
                "off"
            };
            let val_c = if world.ui_checkbox_value(self.checkbox_c) {
                "on"
            } else {
                "off"
            };
            world.ui_set_text(
                self.checkbox_label_slot,
                &format!("A: {val_a}  B: {val_b}  C: {val_c}"),
            );
        }

        if let Some(selected) = world.ui_radio_group_value(0) {
            let names = ["Small", "Medium", "Large"];
            world.ui_set_text(self.radio_label_slot, names.get(selected).unwrap_or(&"?"));
        }

        if world.ui_bind_dropdown(self.dropdown_entity, &mut self.dropdown_val) {
            let names = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
            world.ui_set_text(
                self.dropdown_label_slot,
                names.get(self.dropdown_val).unwrap_or(&"?"),
            );
        }

        if world.ui_tab_bar_changed(self.tab_bar) {
            let selected = world.ui_tab_bar_selected(self.tab_bar);
            for (index, &content) in self.tab_contents.iter().enumerate() {
                world.ui_set_visible(content, index == selected);
            }
        }

        let selected_count = world.ui_tree_view_selected(self.tree_view).len();
        if selected_count == 0 {
            world.ui_set_text(self.tree_selection_label_slot, "(none)");
        } else {
            world.ui_set_text(
                self.tree_selection_label_slot,
                &format!("{selected_count} node(s) selected"),
            );
        }

        self.update_data_grids(world);

        if world.ui_button_clicked(self.confirm_trigger) {
            world.ui_show_modal(self.confirm_dialog);
        }
        if world.ui_button_clicked(self.modal_trigger) {
            world.ui_show_modal(self.modal_dialog);
        }

        if let Some(result) = world.ui_modal_result(self.confirm_dialog) {
            if result {
                world.ui_show_toast("Confirmed!", ToastSeverity::Success, 3.0);
            } else {
                world.ui_show_toast("Cancelled.", ToastSeverity::Info, 3.0);
            }
        }
        if let Some(result) = world.ui_modal_result(self.modal_dialog) {
            if result {
                world.ui_show_toast("Modal accepted", ToastSeverity::Success, 3.0);
            } else {
                world.ui_show_toast("Modal dismissed", ToastSeverity::Info, 3.0);
            }
        }

        if let Some(clicked) = world.ui_context_menu_clicked(self.context_menu) {
            let items = [
                "Cut",
                "Copy",
                "Paste",
                "Text",
                "Image",
                "Link",
                "Select All",
            ];
            if let Some(&name) = items.get(clicked) {
                world.ui_show_toast(&format!("Context menu: {name}"), ToastSeverity::Info, 2.0);
            }
        }

        if world.ui_button_clicked(self.toast_info_btn) {
            world.ui_show_toast(
                "This is an informational message.",
                ToastSeverity::Info,
                3.0,
            );
        }
        if world.ui_button_clicked(self.toast_success_btn) {
            world.ui_show_toast("Operation completed!", ToastSeverity::Success, 3.0);
        }
        if world.ui_button_clicked(self.toast_warning_btn) {
            world.ui_show_toast("Warning: check your settings.", ToastSeverity::Warning, 3.0);
        }
        if world.ui_button_clicked(self.toast_error_btn) {
            world.ui_show_toast("Error: something went wrong.", ToastSeverity::Error, 3.0);
        }

        if let Some(value) = world.ui_composite_value::<Vec3Editor>(self.vec3_editor) {
            world.ui_set_text(
                self.vec3_label_slot,
                &format!("({:.2}, {:.2}, {:.2})", value.x, value.y, value.z),
            );
        }

        if world.ui_button_clicked(self.anim_trigger) {
            self.anim_visible = !self.anim_visible;
            world.ui_set_visible(self.anim_fade, self.anim_visible);
            world.ui_set_visible(self.anim_slide, self.anim_visible);
            world.ui_set_visible(self.anim_scale, self.anim_visible);
        }

        let delta = world.resources.window.timing.delta_time;
        self.progress_value += delta * 0.1;
        if self.progress_value > 1.0 {
            self.progress_value = 0.0;
        }
        world.ui_progress_bar_set_value(self.progress_bar, self.progress_value);

        if world.ui_bind_slider(self.log_slider_entity, &mut self.log_slider_val) {
            world.ui_set_text(
                self.log_slider_label_slot,
                &format!("{:.3}", self.log_slider_val),
            );
        }

        self.canvas_time += delta;
        world.ui_canvas_clear(self.canvas_entity);
        world.ui_canvas_rect(
            self.canvas_entity,
            nalgebra_glm::Vec2::new(10.0, 10.0),
            nalgebra_glm::Vec2::new(80.0, 60.0),
            nalgebra_glm::Vec4::new(0.2, 0.4, 0.8, 1.0),
            4.0,
        );
        world.ui_canvas_rect(
            self.canvas_entity,
            nalgebra_glm::Vec2::new(110.0, 10.0),
            nalgebra_glm::Vec2::new(60.0, 60.0),
            nalgebra_glm::Vec4::new(0.8, 0.3, 0.2, 1.0),
            0.0,
        );
        world.ui_canvas_circle(
            self.canvas_entity,
            nalgebra_glm::Vec2::new(240.0, 40.0),
            25.0,
            nalgebra_glm::Vec4::new(0.3, 0.8, 0.4, 1.0),
        );
        world.ui_canvas_text(
            self.canvas_entity,
            "Canvas Demo",
            nalgebra_glm::Vec2::new(300.0, 30.0),
            14.0,
            nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0),
        );
        let wave_y_offset = 160.0;
        let wave_amplitude = 40.0;
        let wave_steps = 40;
        for step in 0..wave_steps {
            let x0 = step as f32 * (450.0 / wave_steps as f32) + 10.0;
            let x1 = (step + 1) as f32 * (450.0 / wave_steps as f32) + 10.0;
            let t0 = step as f32 / wave_steps as f32 * std::f32::consts::TAU;
            let t1 = (step + 1) as f32 / wave_steps as f32 * std::f32::consts::TAU;
            let y0 = wave_y_offset + (t0 + self.canvas_time * 2.0).sin() * wave_amplitude;
            let y1 = wave_y_offset + (t1 + self.canvas_time * 2.0).sin() * wave_amplitude;
            world.ui_canvas_line(
                self.canvas_entity,
                nalgebra_glm::Vec2::new(x0, y0),
                nalgebra_glm::Vec2::new(x1, y1),
                2.0,
                nalgebra_glm::Vec4::new(1.0, 0.8, 0.2, 1.0),
            );
        }

        if world.ui_bind_toggle(self.disable_toggle, &mut self.disable_active) {
            world.ui_set_disabled(self.disabled_button, self.disable_active);
            world.ui_set_disabled(self.disabled_slider, self.disable_active);
            world.ui_set_disabled(self.disabled_input, self.disable_active);
            if self.disable_active {
                if let Some(interaction) = world.get_ui_node_interaction_mut(self.disabled_button) {
                    interaction.tooltip_text =
                        Some("This button is disabled via the toggle above".to_string());
                }
                if let Some(interaction) = world.get_ui_node_interaction_mut(self.disabled_slider) {
                    interaction.tooltip_text =
                        Some("This slider is disabled via the toggle above".to_string());
                }
                if let Some(interaction) = world.get_ui_node_interaction_mut(self.disabled_input) {
                    interaction.tooltip_text =
                        Some("This input is disabled via the toggle above".to_string());
                }
            } else {
                if let Some(interaction) = world.get_ui_node_interaction_mut(self.disabled_button) {
                    interaction.tooltip_text = None;
                }
                if let Some(interaction) = world.get_ui_node_interaction_mut(self.disabled_slider) {
                    interaction.tooltip_text = None;
                }
                if let Some(interaction) = world.get_ui_node_interaction_mut(self.disabled_input) {
                    interaction.tooltip_text = None;
                }
            }
        }

        if world.ui_bind_toggle(self.region_disable_toggle, &mut self.region_disable_active) {
            world.ui_set_disabled_recursive(self.region_container, !self.region_disable_active);
        }

        if world.ui_text_input_changed(self.tree_filter_input) {
            let text = world.ui_text_input_value(self.tree_filter_input);
            world.ui_tree_view_set_filter(self.tree_view, &text);
        }

        for &(key_code, pressed) in &world.resources.input.keyboard.frame_keys.clone() {
            if pressed
                && key_code == KeyCode::KeyP
                && world
                    .resources
                    .input
                    .keyboard
                    .is_key_pressed(KeyCode::ControlLeft)
            {
                world.ui_show_command_palette(self.command_palette);
            }
        }

        if let Some(command_index) = world.ui_command_palette_executed(self.command_palette) {
            let names = [
                "New File",
                "Open File",
                "Save",
                "Undo",
                "Redo",
                "Find",
                "Replace",
                "Toggle Theme",
            ];
            if let Some(&name) = names.get(command_index) {
                world.ui_set_text(self.command_palette_log_slot, &format!("Executed: {name}"));
                world.ui_show_toast(&format!("Command: {name}"), ToastSeverity::Info, 2.0);
            }
        }

        if world.ui_button_clicked(self.command_palette_trigger) {
            world.ui_show_command_palette(self.command_palette);
        }
    }

    fn on_mouse_input(
        &mut self,
        world: &mut World,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) {
        if button == winit::event::MouseButton::Right
            && state == winit::event::ElementState::Pressed
        {
            world.ui_show_context_menu(self.context_menu, world.resources.input.mouse.position);
        }
    }
}

impl Gallery {
    fn build_typography(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Typography");
            ui.separator();
            ui.heading("This is a heading");
            ui.label("This is body text at default size.");
            ui.label_colored(
                "Accent colored text",
                nalgebra_glm::Vec4::new(0.4, 0.7, 1.0, 1.0),
            );
            ui.label_colored(
                "Warm colored text",
                nalgebra_glm::Vec4::new(1.0, 0.6, 0.3, 1.0),
            );
            ui.separator();
            ui.label("Rich text with mixed formatting:");
            ui.rich_text(&[
                TextSpan::new("Bold ").with_bold(),
                TextSpan::new("and "),
                TextSpan::colored("colored ", nalgebra_glm::Vec4::new(0.4, 1.0, 0.4, 1.0)),
                TextSpan::colored("spans ", nalgebra_glm::Vec4::new(1.0, 0.4, 0.4, 1.0)),
                TextSpan::new("with ").with_italic(),
                TextSpan::sized("size", 20.0),
                TextSpan::new(" variations."),
            ]);
            ui.separator();
            ui.label("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.");
        });
    }

    fn build_buttons(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Buttons");
            ui.separator();
            ui.button("Default Button");
            ui.button_colored("Success", nalgebra_glm::Vec4::new(0.2, 0.7, 0.3, 1.0));
            ui.button_colored("Danger", nalgebra_glm::Vec4::new(0.8, 0.2, 0.2, 1.0));
            ui.separator();
            ui.row(|ui| {
                let entities = [ui.button("Left"), ui.button("Center"), ui.button("Right")];
                for entity in entities {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
            ui.separator();
            self.btn_counter = ui.button("Click Me!");
            ui.label("").done_with_slot(self.click_label_slot, ui);

            ui.separator();
            ui.label("Rich text buttons:");
            self.rich_btn_save = ui.button_rich(&[
                TextSpan::colored("Save", nalgebra_glm::Vec4::new(0.3, 0.9, 0.4, 1.0)),
                TextSpan::new(" Project"),
            ]);
            self.rich_btn_status = ui.button_rich(&[
                TextSpan::new("Status: "),
                TextSpan::colored("Online", nalgebra_glm::Vec4::new(0.2, 0.8, 0.3, 1.0)),
            ]);

            ui.separator();
            ui.label("Disabled region:");
            ui.row(|ui| {
                let label = ui.label("Enable parameter group:");
                self.region_disable_toggle = ui.toggle(true);
                if let Some(node) = ui.world_mut().get_ui_layout_node_mut(label) {
                    node.flex_grow = Some(1.0);
                }
            });
            self.region_container = ui.enabled(true, |ui| {
                ui.slider(0.0, 1.0, 0.5);
                ui.toggle(false);
                ui.button("Apply Settings");
            });
        });
    }

    fn build_inputs(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Text Inputs");
            ui.separator();
            ui.label("Single-line input:");
            self.text_input = ui.text_input("Type here...");
            ui.label("Mirror:");
            ui.label("").done_with_slot(self.input_mirror_slot, ui);
            ui.separator();
            ui.label("Press Enter to submit:");
            self.submit_input = ui.text_input("Press Enter to submit...");
            ui.label("").done_with_slot(self.submit_log_slot, ui);
            ui.separator();
            ui.label("Multi-line text area (4 rows):");
            self.text_area_entity = ui.text_area("Type multiple lines here...", 4);
        });
    }

    fn build_sliders(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Sliders");
            ui.separator();
            ui.row(|ui| {
                let label = ui.label("Value:");
                let value = ui.label("");
                value.done_with_slot(self.slider_label_slot, ui);
                for entity in [label, value] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
            self.slider_entity = ui.slider(0.0, 1.0, 0.5);
            ui.separator();
            ui.row(|ui| {
                let label = ui.label("Range 0-1000:");
                let value = ui.label("");
                value.done_with_slot(self.range_label_slot, ui);
                for entity in [label, value] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
            self.range_slider = ui.slider(0.0, 1000.0, 500.0);
            ui.separator();
            ui.row(|ui| {
                let label = ui.label("Logarithmic (0.001-10.0):");
                let value = ui.label("");
                value.done_with_slot(self.log_slider_label_slot, ui);
                for entity in [label, value] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
            self.log_slider_entity = ui.slider_logarithmic(0.001, 10.0, 0.01);
            ui.separator();
            ui.label("Drag values (X, Y, Z):");
            ui.row(|ui| {
                let lx = ui.label("X");
                self.drag_x = ui.drag_value(0.0, -100.0, 100.0, 0.1, 2);
                let ly = ui.label("Y");
                self.drag_y = ui.drag_value(0.0, -100.0, 100.0, 0.1, 2);
                let lz = ui.label("Z");
                self.drag_z = ui.drag_value(0.0, -100.0, 100.0, 0.1, 2);
                let font_size = ui
                    .world_mut()
                    .resources
                    .retained_ui
                    .theme_state
                    .active_theme()
                    .font_size;
                let label_height = font_size * 1.5;
                for label in [lx, ly, lz] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(label) {
                        node.flow_child_size =
                            Some(Ab(nalgebra_glm::Vec2::new(16.0, label_height)).into());
                    }
                }
                for drag in [self.drag_x, self.drag_y, self.drag_z] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(drag) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
            ui.separator();
            ui.label("Animated progress bar:");
            self.progress_bar = ui.progress_bar(0.0);
            ui.separator();
            ui.label("Disabled widgets:");
            ui.row(|ui| {
                let label = ui.label("Disable below:");
                self.disable_toggle = ui.toggle(false);
                if let Some(node) = ui.world_mut().get_ui_layout_node_mut(label) {
                    node.flex_grow = Some(1.0);
                }
            });
            self.disabled_button = ui.button("Disabled Button");
            self.disabled_slider = ui.slider(0.0, 1.0, 0.5);
            self.disabled_input = ui.text_input("Disabled input...");
        });
    }

    fn build_toggles(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Toggles");
            ui.separator();
            ui.row(|ui| {
                let label = ui.label("Show section below:");
                self.toggle_entity = ui.toggle(true);
                if let Some(node) = ui.world_mut().get_ui_layout_node_mut(label) {
                    node.flex_grow = Some(1.0);
                }
            });
        });

        self.hidden_section = tree
            .add_node()
            .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .without_pointer_events()
            .entity();
        tree.push_parent(self.hidden_section);
        tree.add_label("This section is controlled by the toggle above.");
        tree.add_button("I appear and disappear!");
        tree.pop_parent();

        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Checkboxes:");
            self.checkbox_a = ui.checkbox("Option A", false);
            self.checkbox_b = ui.checkbox("Option B", true);
            self.checkbox_c = ui.checkbox("Option C", false);
            ui.label("").done_with_slot(self.checkbox_label_slot, ui);
            ui.separator();
            ui.label("Radio buttons (group):");
            ui.radio("Small", 0, 0);
            ui.radio("Medium", 0, 1);
            ui.radio("Large", 0, 2);
            ui.row(|ui| {
                let label = ui.label("Selected:");
                let value = ui.label("");
                value.done_with_slot(self.radio_label_slot, ui);
                for entity in [label, value] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
        });
    }

    fn build_dropdowns(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Dropdowns");
            ui.separator();
            ui.label("Fruit selector:");
            self.dropdown_entity =
                ui.dropdown(&["Apple", "Banana", "Cherry", "Date", "Elderberry"], 0);
            ui.row(|ui| {
                let label = ui.label("Selected:");
                let value = ui.label("");
                value.done_with_slot(self.dropdown_label_slot, ui);
                for entity in [label, value] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
        });
    }

    fn build_tabs(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Tabs");
            ui.separator();
            self.tab_bar = ui.tab_bar(&["General", "Settings", "Advanced"], 0);
        });

        let tab_labels = [
            "This is the general overview panel.",
            "Adjust settings here.",
            "Advanced configuration options.",
        ];
        for (index, label_text) in tab_labels.iter().enumerate() {
            let container = tree
                .add_node()
                .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
                .flow(FlowDirection::Vertical, 8.0, 4.0)
                .without_pointer_events()
                .entity();
            if index != 0 {
                tree.world_mut().ui_set_visible(container, false);
            }
            tree.push_parent(container);
            tree.add_label(label_text);
            match index {
                0 => {
                    tree.add_button("Action");
                }
                1 => {
                    tree.add_slider(0.0, 100.0, 50.0);
                    tree.add_toggle(false);
                }
                2 => {
                    tree.add_text_input("Config value...");
                }
                _ => {}
            }
            tree.pop_parent();
            self.tab_contents.push(container);
        }
    }

    fn build_trees(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Lists & Trees");
            ui.separator();
            ui.label("Selectable list:");
            for index in 0..10 {
                ui.selectable_label(&format!("Item {}", index + 1), Some(1));
            }
            ui.separator();
            ui.label("Tree view (Ctrl+click for multi-select):");
            ui.label("Filter:");
            self.tree_filter_input = ui.text_input("Search nodes...");
        });

        let tree_container = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 250.0)),
            )
            .entity();
        tree.push_parent(tree_container);

        self.tree_view = tree.add_tree_view(true);
        let tv_content = tree
            .world_mut()
            .ui_tree_view_content(self.tree_view)
            .unwrap();
        let root_a = tree.add_tree_node(self.tree_view, tv_content, "Root A", 0, 0);
        if let Some(container_a) = tree.world_mut().ui_tree_node_children(root_a) {
            let child_1 = tree.add_tree_node(self.tree_view, container_a, "Child A.1", 1, 1);
            if let Some(sub) = tree.world_mut().ui_tree_node_children(child_1) {
                tree.add_tree_node(self.tree_view, sub, "Leaf A.1.1", 2, 2);
                tree.add_tree_node(self.tree_view, sub, "Leaf A.1.2", 2, 3);
            }
            tree.add_tree_node(self.tree_view, container_a, "Child A.2", 1, 4);
        }
        let root_b = tree.add_tree_node(self.tree_view, tv_content, "Root B", 0, 5);
        if let Some(container_b) = tree.world_mut().ui_tree_node_children(root_b) {
            tree.add_tree_node(self.tree_view, container_b, "Child B.1", 1, 6);
            tree.add_tree_node(self.tree_view, container_b, "Child B.2", 1, 7);
        }
        tree.add_tree_node(self.tree_view, tv_content, "Root C", 0, 8);

        tree.pop_parent();

        tree.build_ui(tree.current_parent(), |ui| {
            ui.row(|ui| {
                let label = ui.label("Selection:");
                let value = ui.label("");
                value.done_with_slot(self.tree_selection_label_slot, ui);
                for entity in [label, value] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
        });
    }

    fn build_data_grid(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Data Grid");
            ui.separator();
            ui.label("5-column grid (50 rows):");
        });

        let small_container = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 300.0)),
            )
            .entity();
        tree.push_parent(small_container);
        self.grid_small = tree.add_data_grid(
            &[
                DataGridColumn::new("ID", 60.0).sortable(),
                DataGridColumn::new("Name", 120.0).sortable(),
                DataGridColumn::new("Value", 80.0).sortable(),
                DataGridColumn::new("Status", 80.0),
                DataGridColumn::new("Score", 80.0).sortable(),
            ],
            20,
        );
        tree.pop_parent();

        tree.add_label("100,000-row grid (virtual scrolling):");

        let large_container = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 400.0)),
            )
            .entity();
        tree.push_parent(large_container);
        self.grid_large = tree.add_data_grid(
            &[
                DataGridColumn::new("#", 80.0).sortable(),
                DataGridColumn::new("Hash", 160.0),
                DataGridColumn::new("Amount", 100.0).sortable(),
            ],
            30,
        );
        tree.pop_parent();
    }

    fn build_scroll_areas(&self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Scroll Areas");
            ui.separator();
            ui.label("Fixed-height scroll area:");
            ui.scroll_area(nalgebra_glm::Vec2::new(300.0, 150.0), |ui| {
                for index in 0..30 {
                    ui.label(&format!("Scrollable item {}", index + 1));
                }
            });
            ui.separator();
            ui.label("Scroll area with mixed widgets:");
            ui.scroll_area(nalgebra_glm::Vec2::new(300.0, 200.0), |ui| {
                ui.heading("Inside scroll");
                ui.button("A button");
                ui.slider(0.0, 100.0, 50.0);
                ui.toggle(false);
                ui.checkbox("Check me", false);
                for index in 0..10 {
                    ui.label(&format!("More content {}", index + 1));
                }
            });
        });
    }

    fn build_modals(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Modals & Dialogs");
            ui.separator();
            self.confirm_trigger = ui.button("Open Confirm Dialog");
            ui.label("Shows a toast on confirm or cancel.");
            ui.separator();
            self.modal_trigger = ui.button("Open Custom Modal");
            ui.label("A modal with embedded slider and text input.");
            ui.separator();
            ui.label("Right-click anywhere for context menu with nested submenus.");
            ui.label("Context menu actions show a toast.");
        });
    }

    fn build_toasts(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Toasts");
            ui.separator();
            ui.label("Click to trigger toast notifications:");
            self.toast_info_btn =
                ui.button_colored("Info Toast", nalgebra_glm::Vec4::new(0.3, 0.6, 1.0, 1.0));
            self.toast_success_btn =
                ui.button_colored("Success Toast", nalgebra_glm::Vec4::new(0.2, 0.8, 0.3, 1.0));
            self.toast_warning_btn =
                ui.button_colored("Warning Toast", nalgebra_glm::Vec4::new(1.0, 0.7, 0.1, 1.0));
            self.toast_error_btn =
                ui.button_colored("Error Toast", nalgebra_glm::Vec4::new(1.0, 0.25, 0.25, 1.0));
            ui.separator();
            ui.label("Toasts also appear from modals and context menus.");
        });
    }

    fn build_rich_text(&self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Rich Text");
            ui.separator();
            ui.rich_text(&[
                TextSpan::colored("Red ", nalgebra_glm::Vec4::new(1.0, 0.3, 0.3, 1.0)),
                TextSpan::colored("Green ", nalgebra_glm::Vec4::new(0.3, 1.0, 0.3, 1.0)),
                TextSpan::colored("Blue ", nalgebra_glm::Vec4::new(0.3, 0.3, 1.0, 1.0)),
                TextSpan::colored("Yellow ", nalgebra_glm::Vec4::new(1.0, 1.0, 0.3, 1.0)),
                TextSpan::colored("Cyan ", nalgebra_glm::Vec4::new(0.3, 1.0, 1.0, 1.0)),
                TextSpan::colored("Magenta", nalgebra_glm::Vec4::new(1.0, 0.3, 1.0, 1.0)),
            ]);
            ui.separator();
            ui.rich_text(&[
                TextSpan::sized("Small ", 10.0),
                TextSpan::sized("Medium ", 14.0),
                TextSpan::sized("Large ", 20.0),
                TextSpan::sized("XL", 28.0),
            ]);
            ui.separator();
            ui.label("Scrollable rich text paragraph:");
            ui.scroll_area(nalgebra_glm::Vec2::new(300.0, 120.0), |ui| {
                ui.rich_text(&[
                    TextSpan::new("Lorem ipsum dolor sit amet, ").with_bold(),
                    TextSpan::new("consectetur adipiscing elit. "),
                    TextSpan::colored(
                        "Sed do eiusmod tempor incididunt ",
                        nalgebra_glm::Vec4::new(0.6, 0.8, 1.0, 1.0),
                    ),
                    TextSpan::new("ut labore et dolore magna aliqua. "),
                    TextSpan::new("Ut enim ad minim veniam, ").with_italic(),
                    TextSpan::new("quis nostrud exercitation ullamco laboris "),
                    TextSpan::new("nisi ut aliquip ex ea commodo consequat. "),
                ]);
                ui.rich_text(&[
                    TextSpan::colored(
                        "Duis aute irure dolor ",
                        nalgebra_glm::Vec4::new(1.0, 0.7, 0.4, 1.0),
                    ),
                    TextSpan::new("in reprehenderit in voluptate velit esse cillum dolore "),
                    TextSpan::new("eu fugiat nulla pariatur. ").with_bold(),
                    TextSpan::new("Excepteur sint occaecat cupidatat non proident, "),
                    TextSpan::new("sunt in culpa qui officia deserunt ").with_italic(),
                    TextSpan::new("mollit anim id est laborum. "),
                ]);
                ui.rich_text(&[
                    TextSpan::new("Sed ut perspiciatis unde omnis iste natus error sit "),
                    TextSpan::colored(
                        "voluptatem accusantium doloremque laudantium, ",
                        nalgebra_glm::Vec4::new(0.4, 1.0, 0.7, 1.0),
                    ),
                    TextSpan::new("totam rem aperiam, eaque ipsa quae ab illo inventore "),
                    TextSpan::new("veritatis et quasi architecto beatae vitae "),
                    TextSpan::new("dicta sunt explicabo.").with_bold(),
                ]);
            });
        });
    }

    fn build_composites(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Composites");
            ui.separator();
            ui.label("Vec3Editor composite:");
            self.vec3_editor = ui.composite::<Vec3Editor>();
            ui.row(|ui| {
                let label = ui.label("Value:");
                let value = ui.label("");
                value.done_with_slot(self.vec3_label_slot, ui);
                for entity in [label, value] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
        });
    }

    fn build_animations(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Animations");
            ui.separator();
            self.anim_trigger = ui.button("Toggle Animated Widgets");
            ui.spacing(8.0);
        });

        self.anim_fade = tree
            .add_node()
            .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .without_pointer_events()
            .with_intro(UiAnimationType::Fade, 0.4)
            .with_outro(UiAnimationType::Fade, 0.3)
            .entity();
        tree.push_parent(self.anim_fade);
        tree.add_label("Fade animation");
        tree.add_button("Fades in and out");
        tree.pop_parent();

        self.anim_slide = tree
            .add_node()
            .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .without_pointer_events()
            .with_intro(UiAnimationType::SlideLeft, 0.5)
            .with_outro(UiAnimationType::SlideRight, 0.3)
            .entity();
        tree.push_parent(self.anim_slide);
        tree.add_label("Slide animation");
        tree.add_button("Slides in from the left");
        tree.pop_parent();

        self.anim_scale = tree
            .add_node()
            .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .without_pointer_events()
            .with_intro(UiAnimationType::Scale, 0.4)
            .with_outro(UiAnimationType::Scale, 0.3)
            .entity();
        tree.push_parent(self.anim_scale);
        tree.add_label("Scale animation");
        tree.add_button("Scales in and out");
        tree.pop_parent();
    }

    fn build_layout(&self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Layout");
            ui.separator();
            ui.label("Horizontal row:");
            ui.row(|ui| {
                let entities = [
                    ui.button("One"),
                    ui.button("Two"),
                    ui.button("Three"),
                    ui.button("Four"),
                ];
                for entity in entities {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_grow = Some(1.0);
                    }
                }
            });
            ui.separator();
            ui.label("Vertical column:");
            ui.column(|ui| {
                ui.label("First");
                ui.label("Second");
                ui.label("Third");
            });
            ui.separator();
            ui.label("Nested rows in columns:");
            ui.column(|ui| {
                ui.row(|ui| {
                    let entities = [ui.label("Row 1, Col A"), ui.label("Row 1, Col B")];
                    for entity in entities {
                        if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                            node.flex_grow = Some(1.0);
                        }
                    }
                });
                ui.row(|ui| {
                    let entities = [ui.label("Row 2, Col A"), ui.label("Row 2, Col B")];
                    for entity in entities {
                        if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                            node.flex_grow = Some(1.0);
                        }
                    }
                });
            });
            ui.separator();
            ui.label("Collapsing sections:");
            ui.collapsing_header("Open by default", true, |ui| {
                ui.label("Content inside collapsing header.");
                ui.slider(0.0, 100.0, 50.0);
            });
            ui.collapsing_header("Closed by default", false, |ui| {
                ui.label("Hidden content revealed on click.");
                ui.toggle(false);
            });
        });
    }

    fn build_themes(&self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Themes");
            ui.separator();
            ui.label("Use the theme dropdown in the top bar to switch themes.");
            ui.separator();
            ui.label("Preview widgets:");
            ui.button("Sample Button");
            ui.slider(0.0, 1.0, 0.5);
            ui.toggle(true);
            ui.checkbox("Sample Checkbox", true);
            ui.text_input("Sample input...");
            ui.progress_bar(0.65);
        });
    }

    fn build_command_palette(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Command Palette");
            ui.separator();
            ui.label("Press Ctrl+P or click below to open:");
            self.command_palette_trigger = ui.button("Open Command Palette");
            ui.label("Last executed:");
            ui.label("")
                .done_with_slot(self.command_palette_log_slot, ui);
        });
    }

    fn build_canvas(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Canvas");
            ui.separator();
            ui.label("2D drawing surface with shapes and animated sine wave:");
            self.canvas_entity = ui.canvas(nalgebra_glm::Vec2::new(470.0, 220.0));
        });
    }

    fn update_data_grids(&self, world: &mut World) {
        world.ui_data_grid_set_row_count(self.grid_small, 50);
        let range_small = world.ui_data_grid_visible_range(self.grid_small);
        let names = [
            "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta",
        ];
        let statuses = ["Active", "Idle", "Error", "Pending"];
        for row in range_small {
            world.ui_data_grid_set_cell(self.grid_small, row, 0, &format!("{}", row + 1));
            world.ui_data_grid_set_cell(self.grid_small, row, 1, names[row % names.len()]);
            world.ui_data_grid_set_cell(
                self.grid_small,
                row,
                2,
                &format!("{:.1}", (row as f32 * 3.7) % 100.0),
            );
            world.ui_data_grid_set_cell(self.grid_small, row, 3, statuses[row % statuses.len()]);
            world.ui_data_grid_set_cell(
                self.grid_small,
                row,
                4,
                &format!("{}", (row * 17 + 3) % 100),
            );
        }

        world.ui_data_grid_set_row_count(self.grid_large, 100_000);
        let range_large = world.ui_data_grid_visible_range(self.grid_large);
        for row in range_large {
            world.ui_data_grid_set_cell(self.grid_large, row, 0, &format!("{}", row + 1));
            world.ui_data_grid_set_cell(
                self.grid_large,
                row,
                1,
                &format!("{:08x}", row.wrapping_mul(2654435761)),
            );
            world.ui_data_grid_set_cell(
                self.grid_large,
                row,
                2,
                &format!("${:.2}", (row as f64 * 0.07) % 999.99),
            );
        }
    }
}

trait LabelSlotHelper {
    fn done_with_slot(self, slot: usize, ui: &mut Ui);
}

impl LabelSlotHelper for Entity {
    fn done_with_slot(self, slot: usize, ui: &mut Ui) {
        if let Some(UiNodeContent::Text { text_slot, .. }) =
            ui.world_mut().get_ui_node_content_mut(self)
        {
            *text_slot = slot;
        }
    }
}
