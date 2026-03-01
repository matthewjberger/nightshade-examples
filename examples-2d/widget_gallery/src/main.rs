use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Gallery::default())
}

const SECTION_NAMES: &[&str] = &[
    "Animations",
    "Buttons",
    "Canvas",
    "Color Picker",
    "Command Palette",
    "Composites",
    "Data Grid",
    "Date Picker",
    "Dropdowns",
    "Layout",
    "Lists & Trees",
    "Menus",
    "Modals & Dialogs",
    "Multi-Select",
    "Panels & Tiles",
    "Property Grid",
    "Rich Text",
    "Scroll Areas",
    "Sliders",
    "Syntax Highlighting",
    "Tables",
    "Tabs",
    "Text Inputs",
    "Themes",
    "Toasts",
    "Toggles",
    "Typography",
    "Breadcrumbs",
    "Range Sliders",
    "Splitters",
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
        let input_height = tree.active_theme().button_height;
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
        let x_drag = tree.add_drag_value(-1000.0, 1000.0, 0.0);
        tree.add_label("Y");
        let y_drag = tree.add_drag_value(-1000.0, 1000.0, 0.0);
        tree.add_label("Z");
        let z_drag = tree.add_drag_value(-1000.0, 1000.0, 0.0);

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
            world
                .widget::<UiDragValueData>(self.x_drag)
                .map(|d| d.value)
                .unwrap_or(0.0),
            world
                .widget::<UiDragValueData>(self.y_drag)
                .map(|d| d.value)
                .unwrap_or(0.0),
            world
                .widget::<UiDragValueData>(self.z_drag)
                .map(|d| d.value)
                .unwrap_or(0.0),
        )
    }
}

struct ClickCounter {
    label: Entity,
    button: Entity,
    count: u32,
}

impl CompositeWidget for ClickCounter {
    type Value = u32;

    fn build(tree: &mut UiTreeBuilder) -> Self {
        let label = tree.add_label("Pressed: 0 keys");
        let button = tree.add_button("Click me");
        Self {
            label,
            button,
            count: 0,
        }
    }

    fn interact(&mut self, world: &mut World, _entity: Entity, ctx: &UiInteractionContext) {
        for &(_, pressed) in &ctx.frame_keys {
            if pressed {
                self.count += 1;
                world.ui_set_label_text(self.label, &format!("Pressed: {} keys", self.count));
            }
        }
    }

    fn update(&mut self, world: &mut World) {
        if world.ui_clicked(self.button) {
            self.count = 0;
            world.ui_set_label_text(self.label, "Pressed: 0 keys");
        }
    }

    fn value(&self, _world: &World) -> u32 {
        self.count
    }
}

#[derive(Default)]
struct Gallery {
    grid_small: Entity,
    grid_large: Entity,
    context_menu: Entity,
    vec3_editor: Entity,
    vec3_label: Entity,
    progress_bar: Entity,
    progress_value: f32,
    command_palette: Entity,
    canvas_entity: Entity,
    compass_canvas_entity: Entity,
    canvas_time: f32,
    virtual_list: Entity,
    editable_grid: Entity,
    table_entity: Entity,
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

        let mut tree = UiTreeBuilder::new(world);

        let font_size = tree.active_theme().font_size;

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

        tree.add_theme_dropdown();

        tree.pop_parent();

        let sidebar = tree
            .add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(0.0, topbar_height)),
                Ab(nalgebra_glm::Vec2::new(180.0, 0.0)) + Rl(nalgebra_glm::Vec2::new(0.0, 100.0)),
            )
            .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::Panel)
            .without_pointer_events()
            .entity();
        tree.push_parent(sidebar);

        let sidebar_scroll = tree.add_scroll_area_fill(8.0, 4.0);
        let sidebar_scroll_content = tree
            .world_mut()
            .widget::<UiScrollAreaData>(sidebar_scroll)
            .map(|d| d.content_entity)
            .unwrap_or(sidebar_scroll);
        tree.push_parent(sidebar_scroll_content);

        let mut nav_labels = Vec::new();
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Sections");
            ui.separator();
            for (index, name) in SECTION_NAMES.iter().enumerate() {
                let entity = ui.selectable_label("", name, Some(0));
                nav_labels.push(entity);
                if index == 0 {
                    ui.world_mut().ui_set_selected(entity, true);
                }
            }
        });

        tree.pop_parent();
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

        let mut section_roots = Vec::new();
        let mut floating_panel = Entity::default();
        let mut confirm_trigger = Entity::default();
        let mut modal_trigger = Entity::default();
        let mut cp_trigger = Entity::default();
        let mut cp_log_label = Entity::default();
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
                .widget::<UiScrollAreaData>(scroll)
                .map(|d| d.content_entity)
                .unwrap_or(scroll);
            tree.push_parent(scroll_content);

            match section_index {
                0 => self.build_animations(&mut tree),
                1 => self.build_buttons(&mut tree),
                2 => self.build_canvas(&mut tree),
                3 => self.build_color_picker(&mut tree),
                4 => {
                    (cp_trigger, cp_log_label) = self.build_command_palette(&mut tree);
                }
                5 => self.build_composites(&mut tree),
                6 => self.build_data_grid(&mut tree),
                7 => self.build_date_picker(&mut tree),
                8 => self.build_dropdowns(&mut tree),
                9 => self.build_layout(&mut tree),
                10 => self.build_trees(&mut tree),
                11 => self.build_menus(&mut tree),
                12 => {
                    (confirm_trigger, modal_trigger) = self.build_modals(&mut tree);
                }
                13 => self.build_multi_select(&mut tree),
                14 => {
                    floating_panel = self.build_panels_tiles(&mut tree);
                }
                15 => self.build_property_grid(&mut tree),
                16 => self.build_rich_text(&mut tree),
                17 => self.build_scroll_areas(&mut tree),
                18 => self.build_sliders(&mut tree),
                19 => self.build_syntax_highlighting(&mut tree),
                20 => self.build_tables(&mut tree),
                21 => self.build_tabs(&mut tree),
                22 => self.build_inputs(&mut tree),
                23 => self.build_themes(&mut tree),
                24 => self.build_toasts(&mut tree),
                25 => self.build_toggles(&mut tree),
                26 => self.build_typography(&mut tree),
                27 => self.build_breadcrumbs(&mut tree),
                28 => self.build_range_sliders(&mut tree),
                29 => self.build_splitters(&mut tree),
                _ => {}
            }

            tree.pop_parent();
            tree.pop_parent();

            section_roots.push(section);
        }

        tree.pop_parent();
        tree.pop_parent();

        let confirm_dialog =
            tree.add_confirm_dialog("Confirm Action", "Are you sure you want to proceed?");

        let modal_dialog =
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

        let active_section = std::rc::Rc::new(std::cell::Cell::new(0usize));
        for (index, &nav) in nav_labels.iter().enumerate() {
            let section_roots = section_roots.clone();
            let active_section = active_section.clone();
            world.ui_react_clicked(nav, move |world: &mut World| {
                let prev = active_section.get();
                if index != prev {
                    world.ui_set_visible(section_roots[prev], false);
                    if prev == 14 {
                        world.ui_set_visible(floating_panel, false);
                    }
                    active_section.set(index);
                    world.ui_set_visible(section_roots[index], true);
                    if index == 14 {
                        world.ui_set_visible(floating_panel, true);
                    }
                }
            });
        }

        world.ui_react_clicked(confirm_trigger, move |world: &mut World| {
            world.ui_show_modal(confirm_dialog);
        });
        world.ui_react_clicked(modal_trigger, move |world: &mut World| {
            world.ui_show_modal(modal_dialog);
        });

        world.ui_react_confirmed(confirm_dialog, |confirmed: bool, world: &mut World| {
            if confirmed {
                world.ui_show_toast("Confirmed!", ToastSeverity::Success, 3.0);
            } else {
                world.ui_show_toast("Cancelled.", ToastSeverity::Info, 3.0);
            }
        });

        world.ui_react_confirmed(modal_dialog, |confirmed: bool, world: &mut World| {
            if confirmed {
                world.ui_show_toast("Modal accepted", ToastSeverity::Success, 3.0);
            } else {
                world.ui_show_toast("Modal dismissed", ToastSeverity::Info, 3.0);
            }
        });

        world.ui_react_menu_selected(self.context_menu, |index: usize, world: &mut World| {
            let items = [
                "Cut",
                "Copy",
                "Paste",
                "Text",
                "Image",
                "Link",
                "Select All",
            ];
            if let Some(&name) = items.get(index) {
                world.ui_show_toast(&format!("Context menu: {name}"), ToastSeverity::Info, 2.0);
            }
        });

        let command_palette = self.command_palette;
        world.ui_react_clicked(cp_trigger, move |world: &mut World| {
            world.ui_show_command_palette(command_palette);
        });

        world.ui_command_palette_register(self.command_palette, "New File", "Ctrl+N", "File");
        world.ui_command_palette_register(self.command_palette, "Open File", "Ctrl+O", "File");
        world.ui_command_palette_register(self.command_palette, "Save", "Ctrl+S", "File");
        world.ui_command_palette_register(self.command_palette, "Undo", "Ctrl+Z", "Edit");
        world.ui_command_palette_register(self.command_palette, "Redo", "Ctrl+Y", "Edit");
        world.ui_command_palette_register(self.command_palette, "Find", "Ctrl+F", "Edit");
        world.ui_command_palette_register(self.command_palette, "Replace", "Ctrl+H", "Edit");
        world.ui_command_palette_register(self.command_palette, "Toggle Theme", "", "View");

        world.ui_react_command(
            self.command_palette,
            move |index: usize, world: &mut World| {
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
                if let Some(&name) = names.get(index) {
                    world.ui_set_text(cp_log_label, &format!("Executed: {name}"));
                    world.ui_show_toast(&format!("Command: {name}"), ToastSeverity::Info, 2.0);
                }
            },
        );
    }

    fn run_systems(&mut self, world: &mut World) {
        if world.resources.retained_ui.active_modal.is_none() {
            escape_key_exit_system(world);
        }

        self.update_data_grids(world);

        if let Some(value) = world.ui_composite_value::<Vec3Editor>(self.vec3_editor) {
            world.ui_set_text(
                self.vec3_label,
                &format!("({:.2}, {:.2}, {:.2})", value.x, value.y, value.z),
            );
        }

        let delta = world.resources.window.timing.delta_time;
        self.progress_value += delta * 0.1;
        if self.progress_value > 1.0 {
            self.progress_value = 0.0;
        }
        world.ui_progress_bar_set_value(self.progress_bar, self.progress_value);

        if world.ui_node_effectively_visible(self.canvas_entity) {
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
            world.ui_canvas_text(
                self.canvas_entity,
                "Bezier Curves",
                nalgebra_glm::Vec2::new(10.0, 85.0),
                11.0,
                nalgebra_glm::Vec4::new(0.7, 0.7, 0.7, 1.0),
            );
            world.ui_canvas_quadratic_bezier(
                self.canvas_entity,
                nalgebra_glm::Vec2::new(10.0, 140.0),
                nalgebra_glm::Vec2::new(120.0, 80.0),
                nalgebra_glm::Vec2::new(230.0, 140.0),
                2.0,
                nalgebra_glm::Vec4::new(0.5, 0.9, 1.0, 1.0),
            );
            let animated_cy = 80.0 + (self.canvas_time * 1.5).sin() * 40.0;
            world.ui_canvas_cubic_bezier(
                self.canvas_entity,
                nalgebra_glm::Vec2::new(240.0, 140.0),
                (
                    nalgebra_glm::Vec2::new(300.0, animated_cy),
                    nalgebra_glm::Vec2::new(390.0, 200.0 - animated_cy + 80.0),
                ),
                nalgebra_glm::Vec2::new(450.0, 140.0),
                2.0,
                nalgebra_glm::Vec4::new(1.0, 0.5, 0.8, 1.0),
            );

            let wave_y_offset = 230.0;
            let wave_amplitude = 30.0;
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
        }

        if world.ui_node_effectively_visible(self.compass_canvas_entity) {
            let auto_spin = world.ui_prop::<bool>("canvas.compass_auto_spin");
            let angle_degrees = if auto_spin {
                let new_angle = (self.canvas_time * 45.0) % 360.0;
                world.ui_set_prop("canvas.compass_angle", new_angle);
                new_angle
            } else {
                world.ui_prop::<f32>("canvas.compass_angle")
            };
            let angle = angle_degrees.to_radians();

            let center = nalgebra_glm::Vec2::new(100.0, 100.0);
            let outer_radius = 90.0;
            let tick_outer = 85.0;
            let tick_inner_cardinal = 70.0;
            let tick_inner_ordinal = 76.0;
            let label_radius = 62.0;
            let needle_length = 75.0;
            let needle_back_length = 50.0;

            world.ui_canvas_clear(self.compass_canvas_entity);

            world.ui_canvas_circle_stroke(
                self.compass_canvas_entity,
                center,
                outer_radius,
                nalgebra_glm::Vec4::new(0.7, 0.7, 0.7, 1.0),
                2.0,
            );

            let cardinal_labels = ["N", "E", "S", "W"];
            for index in 0..8 {
                let tick_angle = angle + index as f32 * std::f32::consts::FRAC_PI_4;
                let cos_val = tick_angle.cos();
                let sin_val = tick_angle.sin();
                let inner = if index % 2 == 0 {
                    tick_inner_cardinal
                } else {
                    tick_inner_ordinal
                };
                let start = center + nalgebra_glm::Vec2::new(cos_val * inner, sin_val * inner);
                let end =
                    center + nalgebra_glm::Vec2::new(cos_val * tick_outer, sin_val * tick_outer);
                let thickness = if index % 2 == 0 { 2.0 } else { 1.0 };
                world.ui_canvas_line(
                    self.compass_canvas_entity,
                    start,
                    end,
                    thickness,
                    nalgebra_glm::Vec4::new(0.8, 0.8, 0.8, 1.0),
                );

                if index % 2 == 0 {
                    let label_pos = center
                        + nalgebra_glm::Vec2::new(
                            cos_val * label_radius - 4.0,
                            sin_val * label_radius - 6.0,
                        );
                    world.ui_canvas_text(
                        self.compass_canvas_entity,
                        cardinal_labels[index / 2],
                        label_pos,
                        12.0,
                        nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0),
                    );
                }
            }

            let north_end = center
                + nalgebra_glm::Vec2::new(angle.cos() * needle_length, angle.sin() * needle_length);
            world.ui_canvas_line(
                self.compass_canvas_entity,
                center,
                north_end,
                3.5,
                nalgebra_glm::Vec4::new(0.9, 0.2, 0.2, 1.0),
            );

            let south_angle = angle + std::f32::consts::PI;
            let south_end = center
                + nalgebra_glm::Vec2::new(
                    south_angle.cos() * needle_back_length,
                    south_angle.sin() * needle_back_length,
                );
            world.ui_canvas_line(
                self.compass_canvas_entity,
                center,
                south_end,
                3.5,
                nalgebra_glm::Vec4::new(0.7, 0.7, 0.7, 1.0),
            );

            world.ui_canvas_circle(
                self.compass_canvas_entity,
                center,
                4.0,
                nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0),
            );
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

        let range = world
            .widget::<UiVirtualListData>(self.virtual_list)
            .map(|d| d.visible_start..d.total_items.min(d.visible_start + d.pool_size))
            .unwrap_or(0..0);
        for pool_index in 0..range.len() {
            let item_index = range.start + pool_index;
            if let Some(container) = world
                .widget::<UiVirtualListData>(self.virtual_list)
                .and_then(|d| {
                    d.pool_items
                        .get(pool_index)
                        .map(|item| item.container_entity)
                })
            {
                let label = world
                    .resources
                    .children_cache
                    .get(&container)
                    .and_then(|v| v.first().copied());
                if let Some(label) = label {
                    world.ui_set_label_text(label, &format!("Item #{}", item_index));
                }
            }
        }

        self.update_table(world);
        self.update_editable_grid(world);
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
                    ui.set_flex_grow(entity, 1.0);
                }
            });
            ui.separator();
            let btn_counter = ui.button("Click Me!");
            let click_label = ui.label("Clicked 0 times");
            let click_count = std::rc::Rc::new(std::cell::Cell::new(0u32));
            let click_count_clone = click_count.clone();
            ui.react_clicked(btn_counter, move |world: &mut World| {
                let count = click_count_clone.get() + 1;
                click_count_clone.set(count);
                world.ui_set_text(click_label, &format!("Clicked {count} times"));
            });

            ui.separator();
            ui.label("Rich text buttons:");
            ui.button_rich(&[
                TextSpan::colored("Save", nalgebra_glm::Vec4::new(0.3, 0.9, 0.4, 1.0)),
                TextSpan::new(" Project"),
            ]);
            ui.button_rich(&[
                TextSpan::new("Status: "),
                TextSpan::colored("Online", nalgebra_glm::Vec4::new(0.2, 0.8, 0.3, 1.0)),
            ]);

            ui.separator();
            ui.label("Button with text tooltip:");
            let tooltip_btn = ui.button("Hover me");
            if let Some(interaction) = ui.world_mut().get_ui_node_interaction_mut(tooltip_btn) {
                interaction.tooltip_text = Some("This is a text tooltip".to_string());
            }

            ui.separator();
            ui.label("Disabled region:");
            ui.row(|ui| {
                let label = ui.label("Enable parameter group:");
                ui.toggle("region_disable", true);
                ui.set_flex_grow(label, 1.0);
            });
            let region_container = ui.enabled(true, |ui| {
                ui.slider("", 0.0, 1.0, 0.5);
                ui.toggle("", false);
                ui.button("Apply Settings");
            });
            ui.react("region_disable", move |enabled: bool, world: &mut World| {
                world.ui_set_disabled_recursive(region_container, !enabled);
            });
        });
    }

    fn build_inputs(&self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Text Inputs");
            ui.separator();
            ui.label("Single-line input:");
            ui.text_input("text_input_mirror", "Type here...");
            ui.label("Mirror:");
            let input_mirror_label = ui.label("");
            ui.react(
                "text_input_mirror",
                move |val: String, world: &mut World| {
                    world.ui_set_text(input_mirror_label, &val);
                },
            );
            ui.separator();
            ui.label("Press Enter to submit:");
            let submit_input = ui.text_input("", "Press Enter to submit...");
            let submit_log_label = ui.label("Submit a value...");
            ui.react_submitted(submit_input, move |text: String, world: &mut World| {
                world.ui_set_text(submit_log_label, &format!("Submitted: {text}"));
                world.ui_text_input_set_value(submit_input, "");
            });
            ui.separator();
            ui.label("Multi-line text area (4 rows):");
            ui.text_area("", "Type multiple lines here...", 4);
            ui.separator();
            ui.label("Form validation (toggle to set error):");
            let validation_input = ui.text_input("", "Required field...");
            ui.toggle("validation_toggle", false);
            ui.react("validation_toggle", move |val: bool, world: &mut World| {
                if val {
                    world.ui_set_error(validation_input, Some("This field is required"));
                } else {
                    world.ui_clear_error(validation_input);
                }
            });
            ui.separator();
            ui.label("Pre-filled input (add_text_input_with_value):");
        });
        tree.add_text_input_with_value("Edit me...", "Hello, World!");
        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Numeric input mask (digits only):");
        });
        let masked_input = tree.add_text_input("Enter a number...");
        tree.world_mut()
            .ui_text_input_set_mask(masked_input, InputMask::Numeric);
        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Validated input (min 3 chars, required):");
            let validated_input = ui.text_input("validated_input", "At least 3 characters...");
            ui.world_mut().ui_set_validation_rules(
                validated_input,
                vec![ValidationRule::Required, ValidationRule::MinLength(3)],
            );
            ui.react("validated_input", move |_val: String, world: &mut World| {
                world.ui_validate(validated_input);
            });
        });
        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Max-length input (10 characters):");
            ui.text_input_max_length("", "Max 10 chars...", 10);
        });
    }

    fn build_sliders(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Sliders");
            ui.separator();
            let mut slider_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Value:");
                slider_label = ui.label("0.50");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(slider_label, 1.0);
            });
            ui.slider("slider_value", 0.0, 1.0, 0.5);
            ui.react("slider_value", move |val: f32, world: &mut World| {
                world.ui_set_text(slider_label, &format!("{val:.2}"));
            });
            ui.separator();
            let mut range_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Range 0-1000:");
                range_label = ui.label("500");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(range_label, 1.0);
            });
            ui.slider("range_slider_value", 0.0, 1000.0, 500.0);
            ui.react("range_slider_value", move |val: f32, world: &mut World| {
                world.ui_set_text(range_label, &format!("{val:.0}"));
            });
            ui.separator();
            let mut log_slider_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Logarithmic (0.001-10.0):");
                log_slider_label = ui.label("0.010");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(log_slider_label, 1.0);
            });
            ui.slider_logarithmic("log_slider", 0.001, 10.0, 0.01);
            ui.react("log_slider", move |val: f32, world: &mut World| {
                world.ui_set_text(log_slider_label, &format!("{val:.3}"));
            });
            ui.separator();
            let mut configured_slider_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Configured (prefix+suffix):");
                configured_slider_label = ui.label("50.0 Hz");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(configured_slider_label, 1.0);
            });
            ui.slider_configured(
                "configured_slider",
                SliderConfig::new(0.0, 1000.0, 50.0)
                    .precision(1)
                    .prefix("Freq: ")
                    .suffix(" Hz"),
            );
            ui.react("configured_slider", move |val: f32, world: &mut World| {
                world.ui_set_text(configured_slider_label, &format!("{val:.1} Hz"));
            });
            ui.separator();
            ui.label("Drag values (X, Y, Z):");
            ui.row(|ui| {
                let lx = ui.label("X");
                let drag_x = ui.drag_value("", -100.0, 100.0, 0.0);
                let ly = ui.label("Y");
                let drag_y = ui.drag_value("", -100.0, 100.0, 0.0);
                let lz = ui.label("Z");
                let drag_z = ui.drag_value("", -100.0, 100.0, 0.0);
                let label_height = ui.theme().font_size * 1.5;
                for label in [lx, ly, lz] {
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(label) {
                        node.flow_child_size =
                            Some(Ab(nalgebra_glm::Vec2::new(16.0, label_height)).into());
                    }
                }
                for drag in [drag_x, drag_y, drag_z] {
                    ui.set_flex_grow(drag, 1.0);
                }
            });
            ui.separator();
            ui.label("Animated progress bar:");
            self.progress_bar = ui.progress_bar("", 0.0);
            ui.separator();
            ui.label("Disabled widgets:");
            ui.row(|ui| {
                let label = ui.label("Disable below:");
                ui.toggle("disable_widgets", false);
                ui.set_flex_grow(label, 1.0);
            });
            let disabled_button = ui.button("Disabled Button");
            let disabled_slider = ui.slider("", 0.0, 1.0, 0.5);
            let disabled_input = ui.text_input("", "Disabled input...");
            ui.react(
                "disable_widgets",
                move |disabled: bool, world: &mut World| {
                    world.ui_set_disabled(disabled_button, disabled);
                    world.ui_set_disabled(disabled_slider, disabled);
                    world.ui_set_disabled(disabled_input, disabled);
                    let tip = if disabled {
                        Some("This widget is disabled via the toggle above".to_string())
                    } else {
                        None
                    };
                    for entity in [disabled_button, disabled_slider, disabled_input] {
                        if let Some(interaction) = world.get_ui_node_interaction_mut(entity) {
                            interaction.tooltip_text = tip.clone();
                        }
                    }
                },
            );
        });
    }

    fn build_toggles(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Toggles");
            ui.separator();
            ui.row(|ui| {
                let label = ui.label("Show section below:");
                ui.toggle("toggle_visibility", true);
                ui.set_flex_grow(label, 1.0);
            });
        });

        let hidden_section = tree
            .add_node()
            .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .without_pointer_events()
            .entity();
        tree.push_parent(hidden_section);
        tree.add_label("This section is controlled by the toggle above.");
        tree.add_button("I appear and disappear!");
        tree.pop_parent();
        tree.world_mut().ui_react::<bool, _>(
            "toggle_visibility",
            move |val: bool, world: &mut World| {
                world.ui_set_visible(hidden_section, val);
            },
        );

        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Checkboxes:");
            ui.checkbox("checkbox_a", "Option A", false);
            ui.checkbox("checkbox_b", "Option B", true);
            ui.checkbox("checkbox_c", "Option C", false);
            let checkbox_label = ui.label("A: off  B: on  C: off");
            for name in ["checkbox_a", "checkbox_b", "checkbox_c"] {
                ui.react(name, move |_val: bool, world: &mut World| {
                    let a: bool = world.ui_prop("checkbox_a");
                    let b: bool = world.ui_prop("checkbox_b");
                    let c: bool = world.ui_prop("checkbox_c");
                    let f = |v| if v { "on" } else { "off" };
                    world.ui_set_text(
                        checkbox_label,
                        &format!("A: {}  B: {}  C: {}", f(a), f(b), f(c)),
                    );
                });
            }
            ui.separator();
            ui.label("Radio buttons (group):");
            ui.radio("Small", 0, 0);
            ui.radio("Medium", 0, 1);
            ui.radio("Large", 0, 2);
            ui.radio_group("size", 0);
            let mut radio_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Selected:");
                radio_label = ui.label("Small");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(radio_label, 1.0);
            });
            ui.react("size", move |val: usize, world: &mut World| {
                let names = ["Small", "Medium", "Large"];
                world.ui_set_text(radio_label, names.get(val).unwrap_or(&"?"));
            });
        });
    }

    fn build_dropdowns(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Dropdowns");
            ui.separator();
            ui.label("Fruit selector:");
            ui.dropdown(
                "dropdown_fruit",
                &["Apple", "Banana", "Cherry", "Date", "Elderberry"],
                0,
            );
            let mut dropdown_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Selected:");
                dropdown_label = ui.label("Apple");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(dropdown_label, 1.0);
            });
            ui.react("dropdown_fruit", move |val: usize, world: &mut World| {
                let names = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
                world.ui_set_text(dropdown_label, names.get(val).unwrap_or(&"?"));
            });
        });
    }

    fn build_syntax_highlighting(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Syntax Highlighting");
            ui.separator();
            #[cfg(feature = "syntax_highlighting")]
            {
                ui.label("Rust:");
                ui.text_area_with_syntax_and_value(
                    "",
                    "",
                    8,
                    "rs",
                    concat!(
                        "use std::collections::HashMap;\n",
                        "\n",
                        "fn main() {\n",
                        "    let mut scores: HashMap<&str, i32> = HashMap::new();\n",
                        "    scores.insert(\"Alice\", 100);\n",
                        "    scores.insert(\"Bob\", 85);\n",
                        "\n",
                        "    for (name, score) in &scores {\n",
                        "        println!(\"{name}: {score}\");\n",
                        "    }\n",
                        "}",
                    ),
                );
                ui.separator();
                ui.label("Python:");
                ui.text_area_with_syntax_and_value(
                    "",
                    "",
                    8,
                    "py",
                    concat!(
                        "import json\n",
                        "\n",
                        "def greet(name: str) -> str:\n",
                        "    \"\"\"Return a greeting message.\"\"\"\n",
                        "    return f\"Hello, {name}!\"\n",
                        "\n",
                        "if __name__ == \"__main__\":\n",
                        "    data = {\"users\": [\"Alice\", \"Bob\"]}\n",
                        "    for user in data[\"users\"]:\n",
                        "        print(greet(user))\n",
                    ),
                );
                ui.separator();
                ui.label("JavaScript:");
                ui.text_area_with_syntax_and_value(
                    "",
                    "",
                    8,
                    "js",
                    concat!(
                        "const fetchData = async (url) => {\n",
                        "    const response = await fetch(url);\n",
                        "    const data = await response.json();\n",
                        "    return data;\n",
                        "};\n",
                        "\n",
                        "class EventEmitter {\n",
                        "    constructor() {\n",
                        "        this.listeners = new Map();\n",
                        "    }\n",
                        "\n",
                        "    on(event, callback) {\n",
                        "        if (!this.listeners.has(event)) {\n",
                        "            this.listeners.set(event, []);\n",
                        "        }\n",
                        "        this.listeners.get(event).push(callback);\n",
                        "    }\n",
                        "}\n",
                    ),
                );
            }
            #[cfg(not(feature = "syntax_highlighting"))]
            {
                ui.label("Enable the 'syntax_highlighting' feature to see this demo.");
            }
        });
    }

    fn build_tabs(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Tabs");
            ui.separator();
            ui.tab_bar("tab_bar_main", &["General", "Settings", "Advanced"], 0);
        });

        let tab_labels = [
            "This is the general overview panel.",
            "Adjust settings here.",
            "Advanced configuration options.",
        ];
        let mut tab_contents = Vec::new();
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
            tab_contents.push(container);
        }
        tree.world_mut().ui_react::<usize, _>(
            "tab_bar_main",
            move |selected: usize, world: &mut World| {
                for (index, &content) in tab_contents.iter().enumerate() {
                    world.ui_set_visible(content, index == selected);
                }
            },
        );
    }

    fn build_trees(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Lists & Trees");
            ui.separator();
            ui.label("Selectable list:");
            for index in 0..10 {
                ui.selectable_label("", &format!("Item {}", index + 1), Some(1));
            }
            ui.separator();
            ui.label("Tree view (Ctrl+click for multi-select):");
            ui.label("Filter:");
            ui.text_input("tree_filter", "Search nodes...");
        });

        let tree_container = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 250.0)),
            )
            .entity();
        tree.push_parent(tree_container);

        let tree_view = tree.add_tree_view(true);
        let tv_content = tree
            .world_mut()
            .widget::<UiTreeViewData>(tree_view)
            .map(|d| d.content_entity)
            .unwrap();
        let root_a = tree.add_tree_node(tree_view, tv_content, "Root A", 0, 0);
        if let Some(container_a) = tree
            .world_mut()
            .widget::<UiTreeNodeData>(root_a)
            .map(|d| d.children_container)
        {
            let child_1 = tree.add_tree_node(tree_view, container_a, "Child A.1", 1, 1);
            if let Some(sub) = tree
                .world_mut()
                .widget::<UiTreeNodeData>(child_1)
                .map(|d| d.children_container)
            {
                tree.add_tree_node(tree_view, sub, "Leaf A.1.1", 2, 2);
                tree.add_tree_node(tree_view, sub, "Leaf A.1.2", 2, 3);
            }
            tree.add_tree_node(tree_view, container_a, "Child A.2", 1, 4);
        }
        let root_b = tree.add_tree_node(tree_view, tv_content, "Root B", 0, 5);
        if let Some(container_b) = tree
            .world_mut()
            .widget::<UiTreeNodeData>(root_b)
            .map(|d| d.children_container)
        {
            tree.add_tree_node(tree_view, container_b, "Child B.1", 1, 6);
            tree.add_tree_node(tree_view, container_b, "Child B.2", 1, 7);
        }
        tree.add_tree_node(tree_view, tv_content, "Root C", 0, 8);

        tree.pop_parent();

        tree.world_mut().ui_react::<String, _>(
            "tree_filter",
            move |val: String, world: &mut World| {
                world.ui_tree_view_set_filter(tree_view, &val);
            },
        );

        let mut tree_selection_label = Entity::default();
        tree.build_ui(tree.current_parent(), |ui| {
            ui.row(|ui| {
                let label = ui.label("Selection:");
                tree_selection_label = ui.label("(none)");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(tree_selection_label, 1.0);
            });
            ui.separator();
            ui.label("Virtual list (10,000 items):");
        });

        tree.world_mut().ui_react_tree_selected(
            tree_view,
            move |_node: Entity, world: &mut World| {
                let selected_count = world
                    .widget::<UiTreeViewData>(tree_view)
                    .map(|d| d.selected_nodes.len())
                    .unwrap_or(0);
                if selected_count == 0 {
                    world.ui_set_text(tree_selection_label, "(none)");
                } else {
                    world.ui_set_text(
                        tree_selection_label,
                        &format!("{selected_count} node(s) selected"),
                    );
                }
            },
        );

        self.virtual_list = tree.add_virtual_list(24.0, 30);
        tree.world_mut()
            .ui_virtual_list_set_count(self.virtual_list, 10_000);

        for pool_index in 0..30 {
            if let Some(container) = tree
                .world_mut()
                .widget::<UiVirtualListData>(self.virtual_list)
                .and_then(|d| {
                    d.pool_items
                        .get(pool_index)
                        .map(|item| item.container_entity)
                })
            {
                tree.push_parent(container);
                tree.add_label("");
                tree.pop_parent();
            }
        }
    }

    fn build_data_grid(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Data Grid");
            ui.separator();
            ui.label("5-column grid with column alignment (50 rows):");
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
                DataGridColumn::new("ID", 60.0)
                    .sortable()
                    .alignment(TextAlignment::Right),
                DataGridColumn::new("Name", 120.0).sortable(),
                DataGridColumn::new("Value", 80.0)
                    .sortable()
                    .alignment(TextAlignment::Right),
                DataGridColumn::new("Status", 80.0).alignment(TextAlignment::Center),
                DataGridColumn::new("Score", 80.0)
                    .sortable()
                    .alignment(TextAlignment::Right),
            ],
            20,
        );
        tree.pop_parent();

        let grid_small = self.grid_small;
        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Filtering (toggle to show only even rows):");
            ui.toggle("grid_filter", false);
            ui.react("grid_filter", move |val: bool, world: &mut World| {
                if val {
                    let even_rows: Vec<usize> = (0..50).filter(|row| row % 2 == 0).collect();
                    world.ui_data_grid_set_filter(grid_small, &even_rows);
                } else {
                    world.ui_data_grid_clear_filter(grid_small);
                }
            });
        });

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
                DataGridColumn::new("#", 80.0)
                    .sortable()
                    .alignment(TextAlignment::Right),
                DataGridColumn::new("Hash", 160.0),
                DataGridColumn::new("Amount", 100.0)
                    .sortable()
                    .alignment(TextAlignment::Right),
            ],
            30,
        );
        tree.pop_parent();

        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Editable grid (double-click Name to edit):");
        });

        let editable_container = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 200.0)),
            )
            .entity();
        tree.push_parent(editable_container);
        self.editable_grid = tree.add_data_grid(
            &[
                DataGridColumn::new("ID", 60.0).alignment(TextAlignment::Right),
                DataGridColumn::new("Name", 150.0).editable(),
                DataGridColumn::new("Value", 100.0).alignment(TextAlignment::Right),
            ],
            10,
        );
        tree.pop_parent();
    }

    fn build_scroll_areas(&mut self, tree: &mut UiTreeBuilder) {
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
                ui.slider("", 0.0, 100.0, 50.0);
                ui.toggle("", false);
                ui.checkbox("", "Check me", false);
                for index in 0..10 {
                    ui.label(&format!("More content {}", index + 1));
                }
            });
            ui.separator();
            ui.label("Scroll snapping (30px intervals):");
            let snap_scroll = ui.scroll_area(nalgebra_glm::Vec2::new(300.0, 120.0), |ui| {
                for index in 0..20 {
                    ui.label(&format!("Snap item {}", index + 1));
                }
            });
            ui.world_mut()
                .ui_scroll_area_set_snap(snap_scroll, Some(30.0));
        });
    }

    fn build_modals(&self, tree: &mut UiTreeBuilder) -> (Entity, Entity) {
        let mut confirm_trigger = Entity::default();
        let mut modal_trigger = Entity::default();
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Modals & Dialogs");
            ui.separator();
            confirm_trigger = ui.button("Open Confirm Dialog");
            ui.label("Shows a toast on confirm or cancel.");
            ui.separator();
            modal_trigger = ui.button("Open Custom Modal");
            ui.label("A modal with embedded slider and text input.");
            ui.separator();
            ui.label("Right-click anywhere for context menu with nested submenus.");
            ui.label("Context menu actions show a toast.");
        });
        (confirm_trigger, modal_trigger)
    }

    fn build_toasts(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Toasts");
            ui.separator();
            ui.label("Click to trigger toast notifications:");
            let info_btn =
                ui.button_colored("Info Toast", nalgebra_glm::Vec4::new(0.3, 0.6, 1.0, 1.0));
            ui.react_clicked(info_btn, |world: &mut World| {
                world.ui_show_toast(
                    "This is an informational message.",
                    ToastSeverity::Info,
                    3.0,
                );
            });
            let success_btn =
                ui.button_colored("Success Toast", nalgebra_glm::Vec4::new(0.2, 0.8, 0.3, 1.0));
            ui.react_clicked(success_btn, |world: &mut World| {
                world.ui_show_toast("Operation completed!", ToastSeverity::Success, 3.0);
            });
            let warning_btn =
                ui.button_colored("Warning Toast", nalgebra_glm::Vec4::new(1.0, 0.7, 0.1, 1.0));
            ui.react_clicked(warning_btn, |world: &mut World| {
                world.ui_show_toast("Warning: check your settings.", ToastSeverity::Warning, 3.0);
            });
            let error_btn =
                ui.button_colored("Error Toast", nalgebra_glm::Vec4::new(1.0, 0.25, 0.25, 1.0));
            ui.react_clicked(error_btn, |world: &mut World| {
                world.ui_show_toast("Something went wrong.", ToastSeverity::Error, 3.0);
            });
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
                self.vec3_label = ui.label("(0.00, 0.00, 0.00)");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(self.vec3_label, 1.0);
            });
            ui.separator();
            ui.label("ClickCounter (custom interact — press any key):");
            ui.composite::<ClickCounter>();
        });
    }

    fn build_animations(&mut self, tree: &mut UiTreeBuilder) {
        let anim_visible = std::rc::Rc::new(std::cell::Cell::new(true));
        let fade_ref = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
        let slide_ref = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));
        let scale_ref = std::rc::Rc::new(std::cell::Cell::new(Entity::default()));

        {
            let anim_visible = anim_visible.clone();
            let fade_ref = fade_ref.clone();
            let slide_ref = slide_ref.clone();
            let scale_ref = scale_ref.clone();
            tree.build_ui(tree.current_parent(), |ui| {
                ui.heading("Animations");
                ui.separator();
                let anim_trigger = ui.button("Toggle Animated Widgets");
                ui.spacing(8.0);

                ui.react_clicked(anim_trigger, move |world: &mut World| {
                    let visible = !anim_visible.get();
                    anim_visible.set(visible);
                    world.ui_set_visible(fade_ref.get(), visible);
                    world.ui_set_visible(slide_ref.get(), visible);
                    world.ui_set_visible(scale_ref.get(), visible);
                });
            });
        }

        let anim_fade = tree
            .add_node()
            .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .without_pointer_events()
            .with_intro(UiAnimationType::Fade, 0.4)
            .with_outro(UiAnimationType::Fade, 0.3)
            .entity();
        tree.push_parent(anim_fade);
        tree.add_label("Fade animation");
        tree.add_button("Fades in and out");
        tree.pop_parent();
        fade_ref.set(anim_fade);

        let anim_slide = tree
            .add_node()
            .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .without_pointer_events()
            .with_intro(UiAnimationType::SlideLeft, 0.5)
            .with_outro(UiAnimationType::SlideRight, 0.3)
            .entity();
        tree.push_parent(anim_slide);
        tree.add_label("Slide animation");
        tree.add_button("Slides in from the left");
        tree.pop_parent();
        slide_ref.set(anim_slide);

        let anim_scale = tree
            .add_node()
            .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .without_pointer_events()
            .with_intro(UiAnimationType::Scale, 0.4)
            .with_outro(UiAnimationType::Scale, 0.3)
            .entity();
        tree.push_parent(anim_scale);
        tree.add_label("Scale animation");
        tree.add_button("Scales in and out");
        tree.pop_parent();
        scale_ref.set(anim_scale);
    }

    fn build_layout(&mut self, tree: &mut UiTreeBuilder) {
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
                    ui.set_flex_grow(entity, 1.0);
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
                        ui.set_flex_grow(entity, 1.0);
                    }
                });
                ui.row(|ui| {
                    let entities = [ui.label("Row 2, Col A"), ui.label("Row 2, Col B")];
                    for entity in entities {
                        ui.set_flex_grow(entity, 1.0);
                    }
                });
            });
            ui.separator();
            ui.label("Collapsing sections:");
            ui.collapsing_header("", "Open by default", true, |ui| {
                ui.label("Content inside collapsing header.");
                ui.slider("", 0.0, 100.0, 50.0);
            });
            ui.collapsing_header("", "Closed by default", false, |ui| {
                ui.label("Hidden content revealed on click.");
                ui.toggle("", false);
            });

            ui.separator();
            ui.label("Text wrapping (200px container):");
            let wrap_label = ui.tree().add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(200.0, 0.0)))
                .with_rect(0.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_text(
                    "This is a long sentence that should wrap within a 200-pixel-wide container, demonstrating word wrapping.",
                    12.0,
                )
                .with_text_wrap()
                .auto_size(AutoSizeMode::Height)
                .done();
            let _ = wrap_label;

            ui.separator();
            ui.label("Min/Max size constraints:");
            ui.row(|ui| {
                let min_box = ui.tree().add_node()
                    .flow_child(Ab(nalgebra_glm::Vec2::new(50.0, 30.0)))
                    .with_rect(4.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Accent)
                    .with_text("min 80px", 10.0)
                    .with_min_size(nalgebra_glm::Vec2::new(80.0, 0.0))
                    .done();
                let max_box = ui.tree().add_node()
                    .flow_child(Ab(nalgebra_glm::Vec2::new(300.0, 30.0)))
                    .with_rect(4.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Panel)
                    .with_text("max 120px", 10.0)
                    .with_max_size(nalgebra_glm::Vec2::new(120.0, 0.0))
                    .done();
                let _ = (min_box, max_box);
            });

            ui.separator();
            ui.label("Flex shrink (items shrink when overflowing):");
            ui.row(|ui| {
                let items = ["Short", "Medium text", "Longer text content"];
                for (index, text) in items.iter().enumerate() {
                    let entity = ui.tree().add_node()
                        .flow_child(Ab(nalgebra_glm::Vec2::new(200.0, 28.0)))
                        .with_rect(4.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
                        .with_theme_color::<UiBase>(ThemeColor::Panel)
                        .with_text(text, 11.0)
                        .done();
                    if let Some(node) = ui.world_mut().get_ui_layout_node_mut(entity) {
                        node.flex_shrink = Some(if index == 0 { 0.0 } else { 1.0 });
                    }
                }
            });

            ui.separator();
            ui.label("Responsive layout (horizontal -> vertical below 400px):");
            let responsive_row = ui.tree().add_node()
                .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
                .flow(FlowDirection::Horizontal, 4.0, 4.0)
                .with_responsive_flow(400.0, FlowDirection::Vertical)
                .entity();
            ui.tree().push_parent(responsive_row);
            for label in &["Alpha", "Beta", "Gamma"] {
                let item = ui.tree().add_node()
                    .flow_child(Ab(nalgebra_glm::Vec2::new(0.0, 28.0)))
                    .with_rect(4.0, 0.0, nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Accent)
                    .with_text(label, 11.0)
                    .done();
                if let Some(node) = ui.world_mut().get_ui_layout_node_mut(item) {
                    node.flex_grow = Some(1.0);
                }
            }
            ui.tree().pop_parent();

            ui.separator();
            ui.label("Wrapped label (add_label_wrapped):");
        });
        tree.add_label_wrapped(
            "This label automatically wraps long text to fit its container width, using add_label_wrapped for convenience instead of manual builder calls.",
        );
        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Auto-grid layout (min 100px columns):");
        });
        let auto_grid_container = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 200.0)),
            )
            .auto_grid(100.0, 40.0, 4.0)
            .entity();
        tree.push_parent(auto_grid_container);
        for index in 0..12 {
            tree.add_button(&format!("Item {}", index + 1));
        }
        tree.pop_parent();

        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Layout introspection:");
            let introspection_target = ui.button("Measure me");
            let introspection_label = ui.label("(click to measure)");
            ui.react_clicked(introspection_target, move |world: &mut World| {
                if let Some(rect) = world.ui_rect(introspection_target) {
                    world.ui_set_label_text(
                        introspection_label,
                        &format!(
                            "Rect: ({:.0}, {:.0}) - ({:.0}, {:.0}) = {:.0}x{:.0}",
                            rect.min.x,
                            rect.min.y,
                            rect.max.x,
                            rect.max.y,
                            rect.width(),
                            rect.height(),
                        ),
                    );
                }
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
            ui.slider("", 0.0, 1.0, 0.5);
            ui.toggle("", true);
            ui.checkbox("", "Sample Checkbox", true);
            ui.text_input("", "Sample input...");
            ui.progress_bar("", 0.65);
        });
    }

    fn build_command_palette(&self, tree: &mut UiTreeBuilder) -> (Entity, Entity) {
        let mut trigger = Entity::default();
        let mut log_label = Entity::default();
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Command Palette");
            ui.separator();
            ui.label("Press Ctrl+P or click below to open:");
            trigger = ui.button("Open Command Palette");
            ui.label("Last executed:");
            log_label = ui.label("Press Ctrl+P or click button");
        });
        (trigger, log_label)
    }

    fn build_canvas(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Canvas");
            ui.separator();
            ui.label("2D drawing surface with shapes, bezier curves, and animated sine wave:");
            self.canvas_entity = ui.canvas(nalgebra_glm::Vec2::new(470.0, 300.0));
            ui.separator();
            ui.label("Interactive compass with auto-spin and manual angle control:");
            ui.scope("canvas", |ui| {
                ui.checkbox("compass_auto_spin", "Auto-spin", true);
                let slider_entity = ui.slider("compass_angle", 0.0, 360.0, 0.0);
                ui.world_mut().ui_set_disabled(slider_entity, true);
                ui.react(
                    "compass_auto_spin",
                    move |auto_spin: bool, world: &mut World| {
                        world.ui_set_disabled(slider_entity, auto_spin);
                    },
                );
                self.compass_canvas_entity = ui.canvas(nalgebra_glm::Vec2::new(200.0, 200.0));
            });
        });
    }

    fn build_color_picker(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Color Picker");
            ui.separator();
            ui.label("RGBA color picker:");
            ui.color_picker(
                "color_picker_rgba",
                nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0),
            );
            let mut color_swatch_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Value:");
                color_swatch_label = ui.label("(1.00, 1.00, 1.00, 1.00)");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(color_swatch_label, 1.0);
            });
            ui.react(
                "color_picker_rgba",
                move |val: nalgebra_glm::Vec4, world: &mut World| {
                    world.ui_set_text(
                        color_swatch_label,
                        &format!("({:.2}, {:.2}, {:.2}, {:.2})", val.x, val.y, val.z, val.w),
                    );
                },
            );
            ui.separator();
            ui.label("HSV color picker:");
            let hsv_initial = nalgebra_glm::Vec4::new(0.8, 0.4, 0.2, 1.0);
            ui.color_picker_hsv("color_picker_hsv", hsv_initial);
            let mut hsv_swatch_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Value:");
                hsv_swatch_label = ui.label("(0.80, 0.40, 0.20, 1.00)");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(hsv_swatch_label, 1.0);
            });
            ui.react(
                "color_picker_hsv",
                move |val: nalgebra_glm::Vec4, world: &mut World| {
                    world.ui_set_text(
                        hsv_swatch_label,
                        &format!("({:.2}, {:.2}, {:.2}, {:.2})", val.x, val.y, val.z, val.w),
                    );
                },
            );
        });
    }

    fn build_property_grid(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Property Grid");
            ui.separator();
            ui.label("A labeled two-column property grid:");
        });

        let prop_grid_entity = tree.add_property_grid(120.0);
        let grid = prop_grid_entity;
        let parent = tree.current_parent();

        let name_area = tree.add_property_row(grid, parent, "Name");
        tree.push_parent(name_area);
        tree.add_text_input("Enter name...");
        tree.pop_parent();

        let speed_area = tree.add_property_row(grid, parent, "Speed");
        tree.push_parent(speed_area);
        tree.add_slider(0.0, 100.0, 50.0);
        tree.pop_parent();

        let active_area = tree.add_property_row(grid, parent, "Active");
        tree.push_parent(active_area);
        tree.add_toggle(true);
        tree.pop_parent();

        let color_area = tree.add_property_row(grid, parent, "Tint");
        tree.push_parent(color_area);
        tree.add_color_picker(nalgebra_glm::Vec4::new(0.5, 0.8, 1.0, 1.0));
        tree.pop_parent();

        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Property sections group related rows:");
        });

        let section = tree.add_property_section(parent, "Transform");
        let px = tree.add_property_row(grid, section, "Position X");
        tree.push_parent(px);
        tree.add_drag_value(-1000.0, 1000.0, 0.0);
        tree.pop_parent();

        let py = tree.add_property_row(grid, section, "Position Y");
        tree.push_parent(py);
        tree.add_drag_value(-1000.0, 1000.0, 0.0);
        tree.pop_parent();

        let rot = tree.add_property_row(grid, section, "Rotation");
        tree.push_parent(rot);
        tree.add_slider(0.0, 360.0, 0.0);
        tree.pop_parent();

        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Convenience methods (add_property_*):");
        });

        let conv_grid = tree.add_property_grid(130.0);
        let conv_parent = tree.current_parent();
        tree.add_property_slider(conv_grid, conv_parent, "Opacity", 0.0, 1.0, 0.8);
        tree.add_property_toggle(conv_grid, conv_parent, "Enabled", true);
        tree.add_property_text_input(conv_grid, conv_parent, "Label", "Enter text...");
        tree.add_property_dropdown(
            conv_grid,
            conv_parent,
            "Mode",
            &["Auto", "Manual", "Custom"],
            0,
        );
        tree.add_property_checkbox(conv_grid, conv_parent, "Visible", true);
        tree.add_property_drag_value(conv_grid, conv_parent, "Offset", -100.0, 100.0, 0.0);
    }

    fn build_menus(&self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Menus");
            ui.separator();
            ui.label("Standalone menu (dropdown from a label):");
            let menu_entity = ui.menu("File", &["New", "Open", "Save", "Close"]);
            let mut menu_log_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Last clicked:");
                menu_log_label = ui.label("(none)");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(menu_log_label, 1.0);
            });
            ui.react_menu_selected(menu_entity, move |index: usize, world: &mut World| {
                let items = ["New", "Open", "Save", "Close"];
                if let Some(&name) = items.get(index) {
                    world.ui_set_text(menu_log_label, &format!("Clicked: {name}"));
                }
            });
            ui.separator();
            ui.label("Context menus are shown under 'Modals & Dialogs'.");
        });
    }

    fn build_panels_tiles(&self, tree: &mut UiTreeBuilder) -> Entity {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Panels & Tiles");
            ui.separator();
            ui.label("Floating panel:");
        });

        let floating_panel =
            tree.add_floating_panel("Floating Panel", Rect::new(20.0, 40.0, 250.0, 150.0));
        tree.world_mut().ui_set_visible(floating_panel, false);
        let fp_content = tree
            .world_mut()
            .widget::<UiPanelData>(floating_panel)
            .map(|d| d.content_entity)
            .unwrap_or(floating_panel);
        tree.push_parent(fp_content);
        tree.add_label("This panel can be dragged and resized.");
        tree.add_button("A button inside");
        tree.add_slider(0.0, 1.0, 0.5);
        tree.pop_parent();

        tree.build_ui(tree.current_parent(), |ui| {
            ui.separator();
            ui.label("Tile container (dockable split panels):");
        });

        let tile_holder = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 300.0)),
            )
            .entity();
        tree.push_parent(tile_holder);
        let tile_container = tree.add_tile_container(nalgebra_glm::Vec2::new(500.0, 300.0));
        tree.build_tiles(tile_container, |tiles| {
            if let Some((left_id, left_entity)) = tiles.pane("Left Pane") {
                tiles.content(left_entity, |tree| {
                    tree.add_label("Left pane content");
                    tree.add_button("Button A");
                });
                if let Some((_right_id, right_entity)) =
                    tiles.split_from(left_id, SplitDirection::Horizontal, 0.5, "Right Pane")
                {
                    tiles.content(right_entity, |tree| {
                        tree.add_label("Right pane content");
                        tree.add_toggle(false);
                        tree.add_slider(0.0, 100.0, 25.0);
                    });
                }
            }
        });
        tree.pop_parent();

        floating_panel
    }

    fn build_breadcrumbs(&self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Breadcrumbs");
            ui.separator();
            ui.label("Navigation breadcrumb (click segments to navigate):");
        });

        let breadcrumb_entity =
            tree.add_breadcrumb(&["Home", "Documents", "Projects", "Nightshade"]);

        let mut breadcrumb_log_label = Entity::default();
        tree.build_ui(tree.current_parent(), |ui| {
            breadcrumb_log_label = ui.label("Click a breadcrumb segment");
        });

        tree.world_mut().ui_react_menu_selected(
            breadcrumb_entity,
            move |index: usize, world: &mut World| {
                let segments = ["Home", "Documents", "Projects", "Nightshade"];
                if let Some(&name) = segments.get(index) {
                    world.ui_set_text(breadcrumb_log_label, &format!("Navigated to: {name}"));
                }
            },
        );
    }

    fn build_range_sliders(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Range Sliders");
            ui.separator();
            ui.label("Dual-thumb range slider (0-100):");
            ui.range_slider("range_slider", 0.0, 100.0, 20.0, 80.0);
            let range_slider_label = ui.label("Low: 20.0, High: 80.0");
            ui.react(
                "range_slider",
                move |val: nalgebra_glm::Vec4, world: &mut World| {
                    world.ui_set_text(
                        range_slider_label,
                        &format!("Low: {:.1}, High: {:.1}", val.x, val.y),
                    );
                },
            );
        });
    }

    fn build_splitters(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Splitters");
            ui.separator();
            ui.label("Horizontal splitter (drag the divider):");
        });

        let splitter_holder = tree
            .add_node()
            .flow_child(
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 200.0)),
            )
            .entity();
        tree.push_parent(splitter_holder);
        let splitter_entity = tree.add_splitter(SplitDirection::Horizontal, 0.5);
        tree.pop_parent();

        tree.world_mut()
            .ui_register_named("splitter", splitter_entity, 0.5f32);

        if let Some(first) = tree
            .world_mut()
            .widget::<UiSplitterData>(splitter_entity)
            .map(|d| d.first_pane)
        {
            tree.push_parent(first);
            tree.add_label("Left pane");
            tree.add_button("Button A");
            tree.add_slider(0.0, 1.0, 0.5);
            tree.pop_parent();
        }
        if let Some(second) = tree
            .world_mut()
            .widget::<UiSplitterData>(splitter_entity)
            .map(|d| d.second_pane)
        {
            tree.push_parent(second);
            tree.add_label("Right pane");
            tree.add_toggle(false);
            tree.add_checkbox("Option", true);
            tree.pop_parent();
        }

        tree.build_ui(tree.current_parent(), |ui| {
            let splitter_label = ui.label("Ratio: 0.50");
            ui.react("splitter", move |val: f32, world: &mut World| {
                world.ui_set_text(splitter_label, &format!("Ratio: {val:.2}"));
            });
        });
    }

    fn build_date_picker(&self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Date Picker");
            ui.separator();
            ui.label("Select a date:");
            let date_picker_entity = ui.date_picker(2026, 2, 28);
            let mut date_picker_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Selected:");
                date_picker_label = ui.label("2026-02-28");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(date_picker_label, 1.0);
            });
            ui.world_mut().ui_react_date_changed(
                date_picker_entity,
                move |year: i32, month: u32, day: u32, world: &mut World| {
                    world.ui_set_text(date_picker_label, &format!("{year:04}-{month:02}-{day:02}"));
                },
            );
        });
    }

    fn build_multi_select(&self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Multi-Select Dropdown");
            ui.separator();
            ui.label("Pick your favorite languages:");
            let multi_select_entity = ui.multi_select(&[
                "Rust",
                "Python",
                "TypeScript",
                "Go",
                "C++",
                "Zig",
                "Haskell",
            ]);
            let mut multi_select_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Selection:");
                multi_select_label = ui.label("None selected");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(multi_select_label, 1.0);
            });
            ui.world_mut().ui_react_multi_select_changed(
                multi_select_entity,
                move |indices: Vec<usize>, world: &mut World| {
                    let names = [
                        "Rust",
                        "Python",
                        "TypeScript",
                        "Go",
                        "C++",
                        "Zig",
                        "Haskell",
                    ];
                    let display: Vec<&str> = indices
                        .iter()
                        .filter_map(|&index| names.get(index).copied())
                        .collect();
                    if display.is_empty() {
                        world.ui_set_text(multi_select_label, "None selected");
                    } else {
                        world.ui_set_text(multi_select_label, &display.join(", "));
                    }
                },
            );
            ui.separator();
            ui.label("Searchable dropdown:");
            let country_options: &[&str] = &[
                "Afghanistan",
                "Argentina",
                "Australia",
                "Brazil",
                "Canada",
                "China",
                "Denmark",
                "Egypt",
                "Finland",
                "France",
                "Germany",
                "India",
                "Japan",
                "Mexico",
                "Norway",
                "Poland",
                "Russia",
                "Spain",
                "Sweden",
                "United Kingdom",
                "United States",
            ];
            ui.dropdown_searchable("searchable_country", country_options, 0);
            let mut searchable_dropdown_label = Entity::default();
            ui.row(|ui| {
                let label = ui.label("Country:");
                searchable_dropdown_label = ui.label("Afghanistan");
                ui.set_flex_grow(label, 1.0);
                ui.set_flex_grow(searchable_dropdown_label, 1.0);
            });
            ui.react(
                "searchable_country",
                move |val: usize, world: &mut World| {
                    const COUNTRIES: &[&str] = &[
                        "Afghanistan",
                        "Argentina",
                        "Australia",
                        "Brazil",
                        "Canada",
                        "China",
                        "Denmark",
                        "Egypt",
                        "Finland",
                        "France",
                        "Germany",
                        "India",
                        "Japan",
                        "Mexico",
                        "Norway",
                        "Poland",
                        "Russia",
                        "Spain",
                        "Sweden",
                        "United Kingdom",
                        "United States",
                    ];
                    world.ui_set_text(
                        searchable_dropdown_label,
                        COUNTRIES.get(val).unwrap_or(&"?"),
                    );
                },
            );
        });
    }

    fn build_tables(&mut self, tree: &mut UiTreeBuilder) {
        tree.build_ui(tree.current_parent(), |ui| {
            ui.heading("Tables");
            ui.separator();
            ui.label("Simple table (convenience wrapper):");
            self.table_entity = ui.table(&["Name", "Score", "Status"], &[200.0, 100.0, 120.0]);
        });
        tree.world_mut()
            .ui_data_grid_set_row_count(self.table_entity, 5);
        let items = [
            ("Alice", "95", "Active"),
            ("Bob", "82", "Idle"),
            ("Charlie", "71", "Active"),
            ("Diana", "90", "Away"),
            ("Eve", "88", "Active"),
        ];
        for (row, (name, score, status)) in items.iter().enumerate() {
            tree.world_mut()
                .ui_data_grid_set_cell(self.table_entity, row, 0, name);
            tree.world_mut()
                .ui_data_grid_set_cell(self.table_entity, row, 1, score);
            tree.world_mut()
                .ui_data_grid_set_cell(self.table_entity, row, 2, status);
        }
    }

    fn update_editable_grid(&self, world: &mut World) {
        let items = ["Widget", "Engine", "Shader", "Texture", "Audio"];
        world.ui_data_grid_set_row_count(self.editable_grid, 5);
        let range = world
            .widget::<UiDataGridData>(self.editable_grid)
            .map(|d| {
                let end = (d.visible_start + d.pool_size).min(d.total_rows);
                d.visible_start..end
            })
            .unwrap_or(0..0);
        for visible_row in range {
            let data_row = world
                .widget::<UiDataGridData>(self.editable_grid)
                .and_then(|d| {
                    if let Some(indices) = &d.filtered_indices {
                        indices.get(visible_row).copied()
                    } else {
                        Some(visible_row)
                    }
                })
                .unwrap_or(visible_row);
            world.ui_data_grid_set_cell(
                self.editable_grid,
                visible_row,
                0,
                &format!("{}", data_row + 1),
            );
            world.ui_data_grid_set_cell(
                self.editable_grid,
                visible_row,
                1,
                items.get(data_row).unwrap_or(&"?"),
            );
            world.ui_data_grid_set_cell(
                self.editable_grid,
                visible_row,
                2,
                &format!("{:.1}", (data_row as f32 + 1.0) * 10.0),
            );
        }
    }

    fn update_table(&self, world: &mut World) {
        world.ui_data_grid_set_row_count(self.table_entity, 5);
        let range = world
            .widget::<UiDataGridData>(self.table_entity)
            .map(|d| {
                let end = (d.visible_start + d.pool_size).min(d.total_rows);
                d.visible_start..end
            })
            .unwrap_or(0..0);
        let items = [
            ("Alice", "95", "Active"),
            ("Bob", "82", "Idle"),
            ("Charlie", "71", "Active"),
            ("Diana", "90", "Away"),
            ("Eve", "88", "Active"),
        ];
        for visible_row in range {
            if let Some(&(name, score, status)) = items.get(visible_row) {
                world.ui_data_grid_set_cell(self.table_entity, visible_row, 0, name);
                world.ui_data_grid_set_cell(self.table_entity, visible_row, 1, score);
                world.ui_data_grid_set_cell(self.table_entity, visible_row, 2, status);
            }
        }
    }

    fn update_data_grids(&self, world: &mut World) {
        world.ui_data_grid_set_row_count(self.grid_small, 50);
        let range_small = world
            .widget::<UiDataGridData>(self.grid_small)
            .map(|d| {
                let end = (d.visible_start + d.pool_size).min(d.total_rows);
                d.visible_start..end
            })
            .unwrap_or(0..0);
        let names = [
            "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta",
        ];
        let statuses = ["Active", "Idle", "Error", "Pending"];
        for visible_row in range_small {
            let data_row = world
                .widget::<UiDataGridData>(self.grid_small)
                .and_then(|d| {
                    if let Some(indices) = &d.filtered_indices {
                        indices.get(visible_row).copied()
                    } else {
                        Some(visible_row)
                    }
                })
                .unwrap_or(visible_row);
            world.ui_data_grid_set_cell(
                self.grid_small,
                visible_row,
                0,
                &format!("{}", data_row + 1),
            );
            world.ui_data_grid_set_cell(
                self.grid_small,
                visible_row,
                1,
                names[data_row % names.len()],
            );
            world.ui_data_grid_set_cell(
                self.grid_small,
                visible_row,
                2,
                &format!("{:.1}", (data_row as f32 * 3.7) % 100.0),
            );
            world.ui_data_grid_set_cell(
                self.grid_small,
                visible_row,
                3,
                statuses[data_row % statuses.len()],
            );
            world.ui_data_grid_set_cell(
                self.grid_small,
                visible_row,
                4,
                &format!("{}", (data_row * 17 + 3) % 100),
            );
        }

        world.ui_data_grid_set_row_count(self.grid_large, 100_000);
        let range_large = world
            .widget::<UiDataGridData>(self.grid_large)
            .map(|d| {
                let end = (d.visible_start + d.pool_size).min(d.total_rows);
                d.visible_start..end
            })
            .unwrap_or(0..0);
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
