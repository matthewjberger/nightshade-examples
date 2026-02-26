use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(DockingDemo::default())
}

struct Vec3Editor {
    x: Entity,
    y: Entity,
    z: Entity,
}

impl CompositeWidget for Vec3Editor {
    type Value = nalgebra_glm::Vec3;

    fn build(tree: &mut UiTreeBuilder) -> Self {
        let container = tree.current_parent();
        let theme = tree
            .world_mut()
            .resources
            .retained_ui
            .theme_state
            .active_theme();
        let input_height = theme.button_height;
        let font_size = theme.font_size;
        let text_color = theme.text_color;
        if let Some(node) = tree.world_mut().get_ui_layout_node_mut(container) {
            node.flow_layout = Some(FlowLayout {
                direction: FlowDirection::Horizontal,
                padding: 0.0,
                spacing: 2.0,
                alignment: FlowAlignment::Start,
                cross_alignment: FlowAlignment::Center,
                wrap: false,
            });
        }

        let labels = ["X", "Y", "Z"];
        let mut entities = [Entity::default(); 3];
        for (index, label_text) in labels.iter().enumerate() {
            let slot = tree.world_mut().resources.text_cache.add_text(*label_text);
            tree.add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(14.0, input_height)))
                .with_text_slot(slot, font_size * 0.85)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .with_color::<UiBase>(text_color)
                .without_pointer_events()
                .done();
            entities[index] = tree.add_drag_value(0.0, -100.0, 100.0, 0.1, 2);
        }

        for &entity in &entities {
            if let Some(node) = tree.world_mut().get_ui_layout_node_mut(entity) {
                node.flow_child_size = Some(Ab(nalgebra_glm::Vec2::new(0.0, input_height)).into());
                node.flex_grow = Some(1.0);
            }
        }

        Self {
            x: entities[0],
            y: entities[1],
            z: entities[2],
        }
    }

    fn value(&self, world: &World) -> nalgebra_glm::Vec3 {
        nalgebra_glm::Vec3::new(
            world.ui_drag_value(self.x),
            world.ui_drag_value(self.y),
            world.ui_drag_value(self.z),
        )
    }
}

struct SceneEntity {
    entity: Entity,
    tree_node: Entity,
    name: String,
}

#[derive(Default)]
struct DockingDemo {
    left_panel: Entity,
    right_panel: Entity,
    bottom_panel: Entity,
    floating_panel_a: Entity,
    floating_panel_b: Entity,
    log_text_slot: usize,
    log_lines: Vec<String>,
    total_time: f32,
    tree_view: Entity,
    cubes_tree_children: Entity,
    context_menu: Entity,
    confirm_dialog: Entity,
    delete_button: Entity,
    add_entity_button: Entity,
    position_editor: Entity,
    rot_y: Entity,
    scale_x: Entity,
    visible_toggle: Entity,
    shadow_checkbox: Entity,
    scene_entities: Vec<SceneEntity>,
    selected_scene_entity: Option<Entity>,
    next_cube_index: usize,
    pos: UiProperty<nalgebra_glm::Vec3>,
    rotation_y: UiProperty<f32>,
    scale: UiProperty<f32>,
    top_panel: Entity,
    file_button: Entity,
    view_button: Entity,
    add_button: Entity,
    file_menu: Entity,
    view_menu: Entity,
    add_menu: Entity,
    fps_text_slot: usize,
    inspector_just_loaded: bool,
    command_palette: Entity,
    tree_filter_input: Entity,
    grid_toggle_command_id: usize,
    tile_container: Entity,
    tile_console_text: usize,
    tile_console_lines: Vec<String>,
    tile_output_text: usize,
    tile_output_lines: Vec<String>,
    tile_scene_info_text: usize,
    tile_scene_info_pane: TileId,
    tile_add_pane_button: Entity,
    tile_pane_counter: usize,
}

impl State for DockingDemo {
    fn title(&self) -> &str {
        "NIGHTSHADE // DOCKING"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.retained_ui.enabled = true;
        world.resources.graphics.clear_color = [0.02, 0.02, 0.04, 1.0];
        world.resources.graphics.show_grid = true;

        self.pos = UiProperty::new(nalgebra_glm::Vec3::zeros());
        self.rotation_y = UiProperty::new(0.0);
        self.scale = UiProperty::new(1.0);

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 1.0, 0.0),
            12.0,
            0.0,
            0.4,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.color = Vec3::new(0.9, 0.85, 0.8);
            light.intensity = 1.0;
            light.cast_shadows = true;
        }

        let floor = spawn_mesh(
            world,
            "Cube",
            Vec3::new(0.0, -0.25, 0.0),
            Vec3::new(20.0, 0.5, 20.0),
        );
        world.set_name(floor, Name("Floor".to_string()));

        let sphere = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(0.0, 1.5, 0.0),
            Vec3::new(1.2, 1.2, 1.2),
        );
        world.set_name(sphere, Name("Center Sphere".to_string()));

        let colors = ["Red", "Green", "Blue", "Yellow", "Cyan", "Magenta"];
        let mut cube_entities = Vec::new();
        for (index, color) in colors.iter().enumerate() {
            let angle = (index as f32 / colors.len() as f32) * std::f32::consts::TAU;
            let radius = 3.5;
            let position = Vec3::new(angle.cos() * radius, 0.5, angle.sin() * radius);
            let entity = spawn_mesh(world, "Cube", position, Vec3::new(0.8, 0.8, 0.8));
            let name = format!("Cube_{index}");
            world.set_name(entity, Name(name.clone()));
            world.set_material_ref(entity, MaterialRef::new(color.to_string()));
            cube_entities.push((entity, name));
        }
        self.next_cube_index = colors.len();

        self.log_text_slot = world
            .resources
            .text_cache
            .add_text("Drag panel headers to undock.\nDrop on indicators to dock.\nResize docked panels from edges.");
        self.fps_text_slot = world.resources.text_cache.add_text("FPS: 0");

        let mut tree = UiTreeBuilder::new(world);

        let theme = tree
            .world_mut()
            .resources
            .retained_ui
            .theme_state
            .active_theme();
        let menu_font = theme.font_size;
        let menu_text_color = theme.text_color;

        self.top_panel = tree.add_docked_panel_top("", 26.0);
        tree.world_mut()
            .ui_panel_set_header_visible(self.top_panel, false);
        if let Some(UiWidgetState::Panel(data)) =
            tree.world_mut().get_ui_widget_state_mut(self.top_panel)
        {
            data.min_size = nalgebra_glm::Vec2::new(0.0, 26.0);
            data.resizable = false;
        }
        if let Some(content) = tree.world_mut().ui_panel_content(self.top_panel) {
            if let Some(node) = tree.world_mut().get_ui_layout_node_mut(content) {
                node.flow_layout = Some(FlowLayout {
                    direction: FlowDirection::Horizontal,
                    padding: 4.0,
                    spacing: 0.0,
                    alignment: FlowAlignment::Start,
                    cross_alignment: FlowAlignment::Center,
                    wrap: false,
                });
            }
            tree.push_parent(content);

            let item_height = 18.0;

            let menu_hover_color = nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0);
            let make_menu_item = |tree: &mut UiTreeBuilder, label: &str| -> Entity {
                let slot = tree.world_mut().resources.text_cache.add_text(label);
                tree.add_node()
                    .flow_child(Ab(nalgebra_glm::Vec2::new(44.0, item_height)))
                    .with_text_slot(slot, menu_font * 0.85)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_color::<UiBase>(menu_text_color)
                    .with_color::<UiHover>(menu_hover_color)
                    .with_interaction()
                    .with_cursor_icon(winit::window::CursorIcon::Pointer)
                    .entity()
            };

            self.file_button = make_menu_item(&mut tree, "File");
            self.view_button = make_menu_item(&mut tree, "View");
            self.add_button = make_menu_item(&mut tree, "Add");

            let spacer = tree
                .add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(0.0, item_height)))
                .without_pointer_events()
                .entity();
            if let Some(node) = tree.world_mut().get_ui_layout_node_mut(spacer) {
                node.flex_grow = Some(1.0);
            }

            tree.add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(80.0, item_height)))
                .with_text_slot(self.fps_text_slot, menu_font * 0.85)
                .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
                .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0))
                .without_pointer_events()
                .done();

            tree.pop_parent();
        }

        self.file_menu = tree.add_context_menu(&[
            ("New Scene", Some("Ctrl+N")),
            ("", None),
            ("Save Layout", Some("Ctrl+S")),
            ("Load Layout", Some("Ctrl+O")),
            ("", None),
            ("Reset Layout", None),
        ]);

        let view_builder = ContextMenuBuilder::new()
            .item("Explorer", "")
            .item("Inspector", "")
            .item("Console", "")
            .item("Tile Tree", "T")
            .separator()
            .item("Toggle Grid", "G")
            .item("Toggle Wireframe", "Z")
            .separator()
            .widget_row("Show Grid");
        self.grid_toggle_command_id = 6;
        self.view_menu = tree.add_context_menu_from_builder(view_builder);
        if let Some(content) = tree
            .world_mut()
            .ui_context_menu_widget_content(self.view_menu, self.grid_toggle_command_id)
        {
            tree.push_parent(content);
            tree.add_toggle(true);
            tree.pop_parent();
        }

        let add_builder = ContextMenuBuilder::new()
            .submenu("Meshes", |builder| {
                builder
                    .item("Cube", "")
                    .item("Sphere", "")
                    .item("Cylinder", "")
            })
            .submenu("Lights", |builder| {
                builder.item("Point Light", "").item("Spot Light", "")
            });
        self.add_menu = tree.add_context_menu_from_builder(add_builder);

        self.left_panel = tree.add_docked_panel_left("Explorer", 220.0);
        if let Some(content) = tree.world_mut().ui_panel_content(self.left_panel) {
            tree.push_parent(content);

            self.tree_filter_input = tree.add_text_input("Filter nodes...");

            let tree_wrapper = tree
                .add_node()
                .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
                .flex_grow(1.0)
                .without_pointer_events()
                .entity();
            tree.push_parent(tree_wrapper);
            self.tree_view = tree.add_tree_view(false);
            tree.pop_parent();
            let tv_content = tree
                .world_mut()
                .ui_tree_view_content(self.tree_view)
                .unwrap();

            let scene_node = tree.add_tree_node(self.tree_view, tv_content, "Scene", 0, 0);
            tree.world_mut().ui_tree_node_set_expanded(scene_node, true);
            let children = tree.world_mut().ui_tree_node_children(scene_node).unwrap();

            let camera_node = tree.add_tree_node(self.tree_view, children, "Main Camera", 1, 1);
            self.scene_entities.push(SceneEntity {
                entity: camera,
                tree_node: camera_node,
                name: "Main Camera".to_string(),
            });

            let sun_node = tree.add_tree_node(self.tree_view, children, "Sun Light", 1, 2);
            self.scene_entities.push(SceneEntity {
                entity: sun,
                tree_node: sun_node,
                name: "Sun Light".to_string(),
            });

            let floor_node = tree.add_tree_node(self.tree_view, children, "Floor", 1, 3);
            self.scene_entities.push(SceneEntity {
                entity: floor,
                tree_node: floor_node,
                name: "Floor".to_string(),
            });

            let cubes_node = tree.add_tree_node(self.tree_view, children, "Cubes", 1, 4);
            tree.world_mut().ui_tree_node_set_expanded(cubes_node, true);
            self.cubes_tree_children = tree.world_mut().ui_tree_node_children(cubes_node).unwrap();
            for (entity, name) in &cube_entities {
                let node = tree.add_tree_node(
                    self.tree_view,
                    self.cubes_tree_children,
                    name,
                    2,
                    entity.id as u64,
                );
                self.scene_entities.push(SceneEntity {
                    entity: *entity,
                    tree_node: node,
                    name: name.clone(),
                });
            }

            let sphere_node = tree.add_tree_node(self.tree_view, children, "Center Sphere", 1, 5);
            self.scene_entities.push(SceneEntity {
                entity: sphere,
                tree_node: sphere_node,
                name: "Center Sphere".to_string(),
            });

            tree.pop_parent();
        }

        self.context_menu = tree.add_context_menu(&[
            ("Rename", Some("F2")),
            ("Duplicate", Some("Ctrl+D")),
            ("", None),
            ("Delete", Some("Del")),
        ]);

        self.right_panel = tree.add_docked_panel_right("Inspector", 300.0);
        if let Some(content) = tree.world_mut().ui_panel_content(self.right_panel) {
            tree.push_parent(content);

            let grid = tree.add_property_grid(55.0);

            let transform_section = tree.add_property_section(grid, "Transform");

            let area = tree.add_property_row(grid, transform_section, "Pos");
            tree.push_parent(area);
            self.position_editor = tree.add_composite::<Vec3Editor>();
            tree.pop_parent();

            let area = tree.add_property_row(grid, transform_section, "Rot Y");
            tree.push_parent(area);
            self.rot_y = tree.add_drag_value(0.0, 0.0, 360.0, 0.5, 1);
            tree.pop_parent();

            let area = tree.add_property_row(grid, transform_section, "Scale");
            tree.push_parent(area);
            self.scale_x = tree.add_drag_value(1.0, 0.1, 10.0, 0.01, 2);
            tree.pop_parent();

            let display_section = tree.add_property_section(grid, "Display");

            let area = tree.add_property_row(grid, display_section, "Visible");
            tree.push_parent(area);
            self.visible_toggle = tree.add_toggle(true);
            tree.pop_parent();

            let area = tree.add_property_row(grid, display_section, "Shadow");
            tree.push_parent(area);
            self.shadow_checkbox = tree.add_checkbox("Cast", true);
            tree.pop_parent();

            tree.pop_parent();
        }

        self.bottom_panel = tree.add_docked_panel_bottom("Console", 150.0);
        if let Some(content) = tree.world_mut().ui_panel_content(self.bottom_panel) {
            tree.push_parent(content);

            tree.add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 200.0)),
                )
                .with_text_slot(self.log_text_slot, 12.0)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.6, 0.6, 0.7, 1.0))
                .without_pointer_events()
                .done();
            tree.pop_parent();
        }

        self.tile_container = tree.add_tile_container(nalgebra_glm::Vec2::new(800.0, 600.0));

        let hint_color = nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0);
        self.tile_console_lines = vec![
            "[INFO] Tile docking system initialized".into(),
            "[INFO] 5 panes created".into(),
            "[INFO] Drag tab headers to rearrange".into(),
            "[INFO] Drop on edges to split, center to merge".into(),
        ];
        self.tile_output_lines = vec![
            "[build] Compiling nightshade v0.7.0".into(),
            "[build] Compiling docking v0.1.0".into(),
            "[build] Finished dev [unoptimized] in 3.2s".into(),
        ];
        let console_initial = self.tile_console_lines.join("\n");
        let output_initial = self.tile_output_lines.join("\n");

        tree.build_tiles(self.tile_container, |tiles| {
            let scene_info_text = tiles.add_text("Entities: 0");
            self.tile_scene_info_text = scene_info_text;
            let (scene_info_id, scene_info_content) = tiles.pane("Scene Info").unwrap();
            self.tile_scene_info_pane = scene_info_id;
            tiles.content(scene_info_content, |tree| {
                tree.add_label("Scene Information");
                tree.add_separator();
                tree.add_node()
                    .flow_child(
                        Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                            + Ab(nalgebra_glm::Vec2::new(0.0, 120.0)),
                    )
                    .with_text_slot(scene_info_text, 12.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                    .with_color::<UiBase>(hint_color)
                    .without_pointer_events()
                    .done();
            });

            let console_text = tiles.add_text(&console_initial);
            self.tile_console_text = console_text;
            let (console_id, console_content) = tiles
                .split_from(scene_info_id, SplitDirection::Horizontal, 0.7, "Console")
                .unwrap();
            tiles.content(console_content, |tree| {
                tree.add_node()
                    .flow_child(
                        Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                            + Ab(nalgebra_glm::Vec2::new(0.0, 400.0)),
                    )
                    .with_text_slot(console_text, 12.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                    .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.6, 0.8, 0.6, 1.0))
                    .without_pointer_events()
                    .done();
            });

            let output_text = tiles.add_text(&output_initial);
            self.tile_output_text = output_text;
            let (_, output_content) = tiles.pane_sibling(console_id, "Output").unwrap();
            tiles.content(output_content, |tree| {
                tree.add_node()
                    .flow_child(
                        Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                            + Ab(nalgebra_glm::Vec2::new(0.0, 400.0)),
                    )
                    .with_text_slot(output_text, 12.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                    .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.7, 0.7, 0.5, 1.0))
                    .without_pointer_events()
                    .done();
            });

            let (_, assets_content) = tiles
                .split_from(console_id, SplitDirection::Vertical, 0.5, "Assets")
                .unwrap();
            tiles.content(assets_content, |tree| {
                tree.add_label("Asset Browser");
                tree.add_separator();
                let asset_names = [
                    "meshes/cube.obj",
                    "meshes/sphere.obj",
                    "meshes/cylinder.obj",
                    "textures/floor.png",
                    "textures/brick.png",
                    "textures/normal_map.png",
                    "materials/red.mat",
                    "materials/green.mat",
                    "materials/blue.mat",
                    "materials/pbr_standard.mat",
                    "shaders/pbr.wgsl",
                    "shaders/unlit.wgsl",
                ];
                for name in &asset_names {
                    tree.add_selectable_label(name, Some(100));
                }
            });
        });

        self.floating_panel_a = tree.add_floating_panel(
            "Scene",
            Rect {
                min: nalgebra_glm::Vec2::new(270.0, 80.0),
                max: nalgebra_glm::Vec2::new(520.0, 280.0),
            },
        );
        if let Some(content) = tree.world_mut().ui_panel_content(self.floating_panel_a) {
            tree.push_parent(content);
            tree.add_label("Spawn new entities into the scene:");
            self.add_entity_button = tree.add_button("Add Cube");
            tree.add_separator();
            tree.add_label("Dynamic tile panes:");
            self.tile_add_pane_button = tree.add_button("Add Tile Pane");
            tree.pop_parent();
        }

        self.floating_panel_b = tree.add_floating_panel(
            "Actions",
            Rect {
                min: nalgebra_glm::Vec2::new(340.0, 150.0),
                max: nalgebra_glm::Vec2::new(600.0, 320.0),
            },
        );
        if let Some(content) = tree.world_mut().ui_panel_content(self.floating_panel_b) {
            tree.push_parent(content);
            tree.add_label("Click to open confirm dialog:");
            self.delete_button = tree.add_button("Delete All Cubes");
            tree.pop_parent();
        }

        self.confirm_dialog = tree.add_confirm_dialog(
            "Confirm Delete",
            "Are you sure you want to delete all cubes?",
        );

        self.command_palette = tree.add_command_palette(10);
        tree.finish();

        world.ui_command_palette_register(self.command_palette, "New Scene", "Ctrl+N", "File");
        world.ui_command_palette_register(self.command_palette, "Save Layout", "Ctrl+S", "File");
        world.ui_command_palette_register(self.command_palette, "Load Layout", "Ctrl+O", "File");
        world.ui_command_palette_register(self.command_palette, "Reset Layout", "", "File");
        world.ui_command_palette_register(self.command_palette, "Toggle Explorer", "", "View");
        world.ui_command_palette_register(self.command_palette, "Toggle Inspector", "", "View");
        world.ui_command_palette_register(self.command_palette, "Toggle Console", "", "View");
        world.ui_command_palette_register(self.command_palette, "Toggle Tile Tree", "T", "View");
        world.ui_command_palette_register(self.command_palette, "Toggle Grid", "G", "View");
        world.ui_command_palette_register(self.command_palette, "Add Cube", "", "Create");
        world.ui_command_palette_register(self.command_palette, "Add Sphere", "", "Create");
        world.ui_command_palette_register(self.command_palette, "Add Cylinder", "", "Create");
        world.ui_command_palette_register(self.command_palette, "Delete All Cubes", "", "Action");

        self.log_lines = vec![
            "[INFO] Application started".into(),
            "[INFO] Drag panel headers to undock".into(),
            "[INFO] Select tree nodes to inspect entities".into(),
        ];
        self.update_log_text(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        let delta_time = world.resources.window.timing.delta_time;
        self.total_time += delta_time;

        self.handle_menu_bar(world);
        self.handle_tree_filter(world);
        self.handle_tree_selection(world);
        self.handle_tree_context_menu(world);
        self.handle_inspector(world);
        self.handle_add_entity(world);
        self.handle_confirm_dialog(world);
        self.handle_command_palette(world);
        self.handle_tile_pane_management(world);
        self.handle_tile_events(world);
        self.check_panel_events(world);
        self.update_scene_info(world);

        let fps = world.resources.window.timing.frames_per_second;
        world
            .resources
            .text_cache
            .set_text(self.fps_text_slot, format!("FPS: {fps}"));
    }
}

impl DockingDemo {
    fn menu_item_clicked(world: &World, entity: Entity) -> bool {
        world
            .get_ui_node_interaction(entity)
            .map(|i| i.clicked)
            .unwrap_or(false)
    }

    fn menu_button_bottom(world: &World, entity: Entity) -> nalgebra_glm::Vec2 {
        world
            .get_ui_layout_node(entity)
            .map(|n| nalgebra_glm::Vec2::new(n.computed_rect.min.x, n.computed_rect.max.y))
            .unwrap_or_default()
    }

    fn handle_menu_bar(&mut self, world: &mut World) {
        if Self::menu_item_clicked(world, self.file_button) {
            let pos = Self::menu_button_bottom(world, self.file_button);
            world.ui_show_context_menu(self.file_menu, pos);
        }
        if Self::menu_item_clicked(world, self.view_button) {
            let pos = Self::menu_button_bottom(world, self.view_button);
            world.ui_show_context_menu(self.view_menu, pos);
        }
        if Self::menu_item_clicked(world, self.add_button) {
            let pos = Self::menu_button_bottom(world, self.add_button);
            world.ui_show_context_menu(self.add_menu, pos);
        }

        if let Some(clicked) = world.ui_context_menu_clicked(self.file_menu) {
            match clicked {
                0 => {
                    let cube_entities: Vec<Entity> = self
                        .scene_entities
                        .iter()
                        .filter(|s| {
                            s.name.starts_with("Cube_")
                                || s.name.starts_with("Sphere_")
                                || s.name.starts_with("Cylinder_")
                        })
                        .map(|s| s.entity)
                        .collect();
                    despawn_entities_with_cache_cleanup(world, &cube_entities);
                    self.scene_entities.retain(|s| {
                        !s.name.starts_with("Cube_")
                            && !s.name.starts_with("Sphere_")
                            && !s.name.starts_with("Cylinder_")
                    });
                    self.selected_scene_entity = None;
                    self.push_log(world, "[FILE] New scene");
                }
                2 => self.push_log(world, "[FILE] Save layout"),
                3 => self.push_log(world, "[FILE] Load layout"),
                5 => self.push_log(world, "[FILE] Reset layout"),
                _ => {}
            }
        }

        if let Some(clicked) = world.ui_context_menu_clicked(self.view_menu) {
            match clicked {
                index @ 0..=2 => {
                    let (panel, name) = match index {
                        0 => (self.left_panel, "Explorer"),
                        1 => (self.right_panel, "Inspector"),
                        _ => (self.bottom_panel, "Console"),
                    };
                    let currently_visible = world
                        .get_ui_layout_node(panel)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(panel, !currently_visible);
                    let state = if currently_visible { "hidden" } else { "shown" };
                    self.push_log(world, &format!("[VIEW] {name} {state}"));
                }
                3 => {
                    let currently_visible = world
                        .get_ui_layout_node(self.tile_container)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(self.tile_container, !currently_visible);
                    let state = if currently_visible { "hidden" } else { "shown" };
                    self.push_log(world, &format!("[VIEW] Tile Tree {state}"));
                }
                4 => {
                    world.resources.graphics.show_grid = !world.resources.graphics.show_grid;
                    let state = if world.resources.graphics.show_grid {
                        "on"
                    } else {
                        "off"
                    };
                    self.push_log(world, &format!("[VIEW] Grid {state}"));
                }
                5 => self.push_log(world, "[VIEW] Wireframe toggled"),
                _ => {}
            }
        }

        if let Some(clicked) = world.ui_context_menu_clicked(self.add_menu) {
            let (mesh_name, display_name) = match clicked {
                0 => ("Cube", "Cube"),
                1 => ("Sphere", "Sphere"),
                2 => ("Cylinder", "Cylinder"),
                3 => {
                    self.push_log(world, "[ADD] Point Light (not implemented)");
                    return;
                }
                4 => {
                    self.push_log(world, "[ADD] Spot Light (not implemented)");
                    return;
                }
                _ => return,
            };
            let name = format!("{}_{}", display_name, self.next_cube_index);
            self.next_cube_index += 1;
            let angle = self.total_time;
            let position = Vec3::new(angle.cos() * 2.0, 0.5, angle.sin() * 2.0);
            let entity = spawn_mesh(world, mesh_name, position, Vec3::new(0.8, 0.8, 0.8));
            world.set_name(entity, Name(name.clone()));

            let mut tree = UiTreeBuilder::new(world);
            let node = tree.add_tree_node(
                self.tree_view,
                self.cubes_tree_children,
                &name,
                2,
                entity.id as u64,
            );
            tree.finish();

            self.scene_entities.push(SceneEntity {
                entity,
                tree_node: node,
                name: name.clone(),
            });
            self.push_log(world, &format!("[ADD] {name}"));
        }
    }

    fn handle_tree_selection(&mut self, world: &mut World) {
        let mut newly_selected: Option<(Entity, String)> = None;
        for event in world.ui_events().to_vec() {
            if let UiEvent::TreeNodeSelected {
                tree,
                node,
                selected,
            } = event
                && tree == self.tree_view
                && selected
                && let Some(scene) = self.scene_entities.iter().find(|s| s.tree_node == node)
            {
                newly_selected = Some((scene.entity, scene.name.clone()));
            }
        }

        if let Some((entity, name)) = newly_selected {
            self.selected_scene_entity = Some(entity);
            self.push_log(world, &format!("[SELECT] {name}"));
            self.load_entity_to_inspector(world, entity);
        }
    }

    fn load_entity_to_inspector(&mut self, world: &mut World, entity: Entity) {
        let Some(transform) = world.get_local_transform(entity) else {
            return;
        };
        let translation = transform.translation;
        let rotation = transform.rotation;
        let scale_val = transform.scale.x;

        self.pos.set(translation);
        let editor_entities = world
            .ui_composite::<Vec3Editor>(self.position_editor)
            .map(|editor| (editor.x, editor.y, editor.z));
        if let Some((ex, ey, ez)) = editor_entities {
            world.ui_set_drag_value(ex, translation.x);
            world.ui_set_drag_value(ey, translation.y);
            world.ui_set_drag_value(ez, translation.z);
        }

        let euler = nalgebra_glm::quat_euler_angles(&rotation);
        let y_degrees = euler.y.to_degrees();
        self.rotation_y.set(y_degrees);
        world.ui_set_drag_value(self.rot_y, y_degrees);

        self.scale.set(scale_val);
        world.ui_set_drag_value(self.scale_x, scale_val);

        self.inspector_just_loaded = true;
    }

    fn handle_inspector(&mut self, world: &mut World) {
        let Some(selected) = self.selected_scene_entity else {
            return;
        };

        if self.inspector_just_loaded {
            self.inspector_just_loaded = false;
            self.rotation_y.take_dirty();
            self.scale.take_dirty();
            return;
        }

        let mut transform_changed = false;

        if let Some(editor) = world.ui_composite::<Vec3Editor>(self.position_editor) {
            let new_pos = editor.value(world);
            if new_pos != *self.pos.get() {
                self.pos.set(new_pos);
                transform_changed = true;
            }
        }

        world.ui_bind_reactive_drag_value(self.rot_y, &mut self.rotation_y);
        if self.rotation_y.take_dirty() {
            transform_changed = true;
        }

        world.ui_bind_reactive_drag_value(self.scale_x, &mut self.scale);
        if self.scale.take_dirty() {
            transform_changed = true;
        }

        if transform_changed {
            let pos = *self.pos.get();
            let rot_y = *self.rotation_y.get();
            let scale_val = *self.scale.get();

            if let Some(transform) = world.get_local_transform_mut(selected) {
                transform.translation = pos;
                transform.rotation = nalgebra_glm::quat_angle_axis(rot_y.to_radians(), &Vec3::y());
                transform.scale = Vec3::new(scale_val, scale_val, scale_val);
            }
            world.set_local_transform_dirty(selected, LocalTransformDirty);
        }

        if world.ui_toggle_changed(self.visible_toggle) {
            let visible = world.ui_toggle_value(self.visible_toggle);
            self.push_log(world, &format!("[TOGGLE] Visible = {visible}"));
        }
    }

    fn handle_add_entity(&mut self, world: &mut World) {
        if world.ui_button_clicked(self.add_entity_button) {
            let name = format!("Cube_{}", self.next_cube_index);
            self.next_cube_index += 1;

            let angle = self.total_time;
            let position = Vec3::new(angle.cos() * 2.0, 0.5, angle.sin() * 2.0);
            let entity = spawn_mesh(world, "Cube", position, Vec3::new(0.8, 0.8, 0.8));
            world.set_name(entity, Name(name.clone()));

            let mut tree = UiTreeBuilder::new(world);
            let node = tree.add_tree_node(
                self.tree_view,
                self.cubes_tree_children,
                &name,
                2,
                entity.id as u64,
            );
            tree.finish();

            self.scene_entities.push(SceneEntity {
                entity,
                tree_node: node,
                name: name.clone(),
            });

            self.push_log(world, &format!("[SPAWN] {name}"));
        }
    }

    fn handle_tree_context_menu(&mut self, world: &mut World) {
        for event in world.ui_events().to_vec() {
            if let UiEvent::TreeNodeContextMenu { tree, position, .. } = event
                && tree == self.tree_view
            {
                world.ui_show_context_menu(self.context_menu, position);
            }
        }

        if let Some(item_index) = world.ui_context_menu_clicked(self.context_menu) {
            let selected_nodes = world.ui_tree_view_selected(self.tree_view);
            let selected_name = selected_nodes.first().and_then(|node| {
                self.scene_entities
                    .iter()
                    .find(|s| s.tree_node == *node)
                    .map(|s| s.name.clone())
            });

            let action = match item_index {
                0 => "Rename",
                1 => "Duplicate",
                3 => "Delete",
                _ => "Unknown",
            };

            if let Some(name) = &selected_name {
                self.push_log(world, &format!("[ACTION] {action}: {name}"));
            } else {
                self.push_log(world, &format!("[ACTION] {action}"));
            }

            if item_index == 3
                && let Some(node) = selected_nodes.first()
                && let Some(index) = self
                    .scene_entities
                    .iter()
                    .position(|s| s.tree_node == *node)
            {
                let scene = self.scene_entities.remove(index);
                despawn_entities_with_cache_cleanup(world, &[scene.entity]);
                if self.selected_scene_entity == Some(scene.entity) {
                    self.selected_scene_entity = None;
                }
            }
        }
    }

    fn handle_confirm_dialog(&mut self, world: &mut World) {
        if world.ui_button_clicked(self.delete_button) {
            world.ui_show_modal(self.confirm_dialog);
        }

        if let Some(confirmed) = world.ui_modal_result(self.confirm_dialog) {
            if confirmed {
                let cube_entities: Vec<Entity> = self
                    .scene_entities
                    .iter()
                    .filter(|s| s.name.starts_with("Cube_"))
                    .map(|s| s.entity)
                    .collect();

                let count = cube_entities.len();
                despawn_entities_with_cache_cleanup(world, &cube_entities);

                self.scene_entities.retain(|s| !s.name.starts_with("Cube_"));
                if let Some(selected) = self.selected_scene_entity
                    && cube_entities.contains(&selected)
                {
                    self.selected_scene_entity = None;
                }

                self.push_log(world, &format!("[DELETE] Removed {count} cubes"));
            } else {
                self.push_log(world, "[ACTION] Cancelled delete");
            }
        }
    }

    fn push_log(&mut self, world: &mut World, message: &str) {
        self.log_lines.push(message.to_string());
        if self.log_lines.len() > 12 {
            self.log_lines.remove(0);
        }
        self.update_log_text(world);
        self.push_tile_console(world, message);
    }

    fn push_tile_console(&mut self, world: &mut World, message: &str) {
        self.tile_console_lines.push(message.to_string());
        if self.tile_console_lines.len() > 30 {
            self.tile_console_lines.remove(0);
        }
        world
            .resources
            .text_cache
            .set_text(self.tile_console_text, self.tile_console_lines.join("\n"));
    }

    fn update_log_text(&self, world: &mut World) {
        let log_text = self.log_lines.join("\n");
        world
            .resources
            .text_cache
            .set_text(self.log_text_slot, log_text);
    }

    fn check_panel_events(&mut self, world: &mut World) {
        let panels = [
            (self.top_panel, "Menu"),
            (self.left_panel, "Explorer"),
            (self.right_panel, "Inspector"),
            (self.bottom_panel, "Console"),
            (self.floating_panel_a, "Scene"),
            (self.floating_panel_b, "Actions"),
        ];

        for (entity, name) in &panels {
            if let Some(UiWidgetState::Panel(data)) = world.get_ui_widget_state(*entity) {
                let kind_str = match data.panel_kind {
                    UiPanelKind::Floating => "floating",
                    UiPanelKind::DockedLeft => "docked left",
                    UiPanelKind::DockedRight => "docked right",
                    UiPanelKind::DockedTop => "docked top",
                    UiPanelKind::DockedBottom => "docked bottom",
                };
                let expected = match *name {
                    "Menu" => "docked top",
                    "Explorer" => "docked left",
                    "Inspector" => "docked right",
                    "Console" => "docked bottom",
                    _ => "floating",
                };
                if kind_str != expected {
                    let msg = format!("[{name}] {expected} -> {kind_str}");
                    if self.log_lines.last().map(|s| s.as_str()) != Some(msg.as_str()) {
                        self.push_log(world, &msg);
                    }
                }
            }
        }
    }

    fn handle_tree_filter(&mut self, world: &mut World) {
        if world.ui_text_input_changed(self.tree_filter_input) {
            let text = world.ui_text_input_value(self.tree_filter_input);
            world.ui_tree_view_set_filter(self.tree_view, &text);
        }
    }

    fn handle_command_palette(&mut self, world: &mut World) {
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
            match command_index {
                0 => self.push_log(world, "[CMD] New Scene"),
                1 => self.push_log(world, "[CMD] Save Layout"),
                2 => self.push_log(world, "[CMD] Load Layout"),
                3 => self.push_log(world, "[CMD] Reset Layout"),
                4 => {
                    let visible = world
                        .get_ui_layout_node(self.left_panel)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(self.left_panel, !visible);
                    self.push_log(world, "[CMD] Toggle Explorer");
                }
                5 => {
                    let visible = world
                        .get_ui_layout_node(self.right_panel)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(self.right_panel, !visible);
                    self.push_log(world, "[CMD] Toggle Inspector");
                }
                6 => {
                    let visible = world
                        .get_ui_layout_node(self.bottom_panel)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(self.bottom_panel, !visible);
                    self.push_log(world, "[CMD] Toggle Console");
                }
                7 => {
                    let visible = world
                        .get_ui_layout_node(self.tile_container)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(self.tile_container, !visible);
                    self.push_log(world, "[CMD] Toggle Tile Tree");
                }
                8 => {
                    world.resources.graphics.show_grid = !world.resources.graphics.show_grid;
                    self.push_log(world, "[CMD] Toggle Grid");
                }
                9 => self.spawn_mesh_entity(world, "Cube"),
                10 => self.spawn_mesh_entity(world, "Sphere"),
                11 => self.spawn_mesh_entity(world, "Cylinder"),
                12 => self.push_log(world, "[CMD] Delete All Cubes"),
                _ => {}
            }
        }
    }

    fn spawn_mesh_entity(&mut self, world: &mut World, mesh_name: &str) {
        let name = format!("{}_{}", mesh_name, self.next_cube_index);
        self.next_cube_index += 1;
        let angle = self.total_time;
        let position = Vec3::new(angle.cos() * 2.0, 0.5, angle.sin() * 2.0);
        let entity = spawn_mesh(world, mesh_name, position, Vec3::new(0.8, 0.8, 0.8));
        world.set_name(entity, Name(name.clone()));

        let mut tree = UiTreeBuilder::new(world);
        let node = tree.add_tree_node(
            self.tree_view,
            self.cubes_tree_children,
            &name,
            2,
            entity.id as u64,
        );
        tree.finish();

        self.scene_entities.push(SceneEntity {
            entity,
            tree_node: node,
            name: name.clone(),
        });
        self.push_log(world, &format!("[CMD] Added {name}"));
    }

    fn handle_tile_pane_management(&mut self, world: &mut World) {
        if world.ui_button_clicked(self.tile_add_pane_button) {
            self.tile_pane_counter += 1;
            let title = format!("Pane {}", self.tile_pane_counter);
            let container = self.tile_container;
            world.build_tiles(container, |tiles| {
                let text_slot = tiles.add_text(format!(
                    "Dynamic pane \"{title}\" created at runtime.\nDrag this tab to rearrange it.\nDrop on edges to split into new area."
                ));
                if let Some((_pane_id, content)) = tiles.pane(&title) {
                    tiles.content(content, |tree| {
                        tree.add_label(&title);
                        tree.add_separator();
                        tree.add_node()
                            .flow_child(
                                Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                                    + Ab(nalgebra_glm::Vec2::new(0.0, 100.0)),
                            )
                            .with_text_slot(text_slot, 12.0)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0))
                            .without_pointer_events()
                            .done();
                    });
                }
            });
            self.push_log(world, &format!("[TILE] Added {title}"));
            self.push_tile_output(world, &format!("[new] Created tile pane: {title}"));
        }
    }

    fn handle_tile_events(&mut self, world: &mut World) {
        let container = self.tile_container;
        if let Some(pane_id) = world.ui_tile_tab_activated(container) {
            let title = world
                .ui_tile_pane_title(container, pane_id)
                .unwrap_or_default();
            self.push_tile_output(world, &format!("[TILE] Tab activated: {title}"));
        }
        if let Some((_pane_id, title)) = world.ui_tile_tab_closed(container) {
            self.push_tile_output(world, &format!("[TILE] Tab closed: {title}"));
        }
        if let Some((_split_id, ratio)) = world.ui_tile_splitter_moved(container) {
            self.push_tile_output(world, &format!("[TILE] Splitter moved: {ratio:.2}"));
        }
    }

    fn push_tile_output(&mut self, world: &mut World, message: &str) {
        self.tile_output_lines.push(message.to_string());
        if self.tile_output_lines.len() > 20 {
            self.tile_output_lines.remove(0);
        }
        world
            .resources
            .text_cache
            .set_text(self.tile_output_text, self.tile_output_lines.join("\n"));
    }

    fn update_scene_info(&self, world: &mut World) {
        if !world.ui_tile_active_pane(self.tile_container, self.tile_scene_info_pane) {
            return;
        }
        let entity_count = self.scene_entities.len();
        let (tile_count, pane_count) = if let Some(UiWidgetState::TileContainer(data)) =
            world.get_ui_widget_state(self.tile_container)
        {
            (
                data.tiles.iter().filter(|tile| tile.is_some()).count(),
                data.tiles
                    .iter()
                    .filter(|tile| matches!(tile, Some(TileNode::Pane { .. })))
                    .count(),
            )
        } else {
            (0, 0)
        };
        let fps = world.resources.window.timing.frames_per_second;
        let info = format!(
            "Scene Entities: {entity_count}\nTile Nodes: {tile_count}\nVisible Panes: {pane_count}\nFPS: {fps}\nTime: {:.1}s",
            self.total_time
        );
        world
            .resources
            .text_cache
            .set_text(self.tile_scene_info_text, info);
    }
}
