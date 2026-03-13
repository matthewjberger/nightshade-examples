use nightshade::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(DockingDemo::default())
}

struct SecondaryWorldInstance {
    world: World,
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
        if let Some(node) = tree.world_mut().ui.get_ui_layout_node_mut(container) {
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
            tree.add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(14.0, input_height)))
                .with_text(label_text, font_size * 0.85)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .with_color::<UiBase>(text_color)
                .without_pointer_events()
                .done();
            entities[index] = tree.add_drag_value(-100.0, 100.0, 0.0);
        }

        for &entity in &entities {
            if let Some(node) = tree.world_mut().ui.get_ui_layout_node_mut(entity) {
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
            world
                .widget::<UiDragValueData>(self.x)
                .map(|d| d.value)
                .unwrap_or(0.0),
            world
                .widget::<UiDragValueData>(self.y)
                .map(|d| d.value)
                .unwrap_or(0.0),
            world
                .widget::<UiDragValueData>(self.z)
                .map(|d| d.value)
                .unwrap_or(0.0),
        )
    }
}

struct SceneEntity {
    entity: Entity,
    tree_node: Entity,
    name: String,
}

#[derive(Default)]
struct SharedState {
    scene_entities: Vec<SceneEntity>,
    selected_scene_entity: Option<Entity>,
    next_cube_index: usize,
    log_lines: Vec<String>,
    log_text_entity: Entity,
    tile_console_lines: Vec<String>,
    tile_console_text: Entity,
    tile_output_lines: Vec<String>,
    tile_output_text: Entity,
    tree_view: Entity,
    cubes_tree_children: Entity,
    total_time: f32,
    tile_container: Entity,
    tile_pane_counter: usize,
    panels: [(Entity, &'static str); 6],
    saved_layout: Option<TileLayout>,
    next_viewport_index: usize,
    command_palette: Entity,
}

impl SharedState {
    fn push_log(&mut self, world: &mut World, message: &str) {
        self.log_lines.push(message.to_string());
        if self.log_lines.len() > 12 {
            self.log_lines.remove(0);
        }
        let log_text = self.log_lines.join("\n");
        world.ui_set_text(self.log_text_entity, &log_text);
        self.push_tile_console(world, message);
    }

    fn push_tile_console(&mut self, world: &mut World, message: &str) {
        self.tile_console_lines.push(message.to_string());
        if self.tile_console_lines.len() > 30 {
            self.tile_console_lines.remove(0);
        }
        world.ui_set_text(self.tile_console_text, &self.tile_console_lines.join("\n"));
    }

    fn push_tile_output(&mut self, world: &mut World, message: &str) {
        self.tile_output_lines.push(message.to_string());
        if self.tile_output_lines.len() > 20 {
            self.tile_output_lines.remove(0);
        }
        world.ui_set_text(self.tile_output_text, &self.tile_output_lines.join("\n"));
    }

    fn spawn_mesh_entity(&mut self, world: &mut World, mesh_name: &str) -> String {
        let name = format!("{}_{}", mesh_name, self.next_cube_index);
        self.next_cube_index += 1;
        let angle = self.total_time;
        let position = Vec3::new(angle.cos() * 2.0, 0.5, angle.sin() * 2.0);
        let entity = spawn_mesh(world, mesh_name, position, Vec3::new(0.8, 0.8, 0.8));
        world.core.set_name(entity, Name(name.clone()));

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
        name
    }
}

fn load_entity_to_inspector(world: &mut World, entity: Entity) {
    let Some(transform) = world.core.get_local_transform(entity) else {
        return;
    };
    let translation = transform.translation;
    let rotation = transform.rotation;
    let scale_val = transform.scale.x;

    world.ui_set_prop("inspector.pos_x", translation.x);
    world.ui_set_prop("inspector.pos_y", translation.y);
    world.ui_set_prop("inspector.pos_z", translation.z);

    let euler = nalgebra_glm::quat_euler_angles(&rotation);
    let y_degrees = euler.y.to_degrees();
    world.ui_set_prop("inspector.rot_y", y_degrees);
    world.ui_set_prop("inspector.scale", scale_val);
}

struct DockingDemo {
    shared: Rc<RefCell<SharedState>>,
    fps_text_entity: Entity,
    tile_scene_info_text: Entity,
    tile_scene_info_pane: TileId,
    secondary_worlds: HashMap<usize, SecondaryWorldInstance>,
    next_world_id: u32,
}

impl Default for DockingDemo {
    fn default() -> Self {
        Self {
            shared: Rc::new(RefCell::new(SharedState::default())),
            fps_text_entity: Entity::default(),
            tile_scene_info_text: Entity::default(),
            tile_scene_info_pane: TileId::default(),
            secondary_worlds: HashMap::new(),
            next_world_id: 0,
        }
    }
}

impl State for DockingDemo {
    fn title(&self) -> &str {
        "NIGHTSHADE // DOCKING"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.retained_ui.enabled = true;
        world.resources.graphics.clear_color = [0.02, 0.02, 0.04, 1.0];
        world.resources.graphics.show_grid = true;

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
        if let Some(light) = world.core.get_light_mut(sun) {
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
        world.core.set_name(floor, Name("Floor".to_string()));

        let sphere = spawn_mesh(
            world,
            "Sphere",
            Vec3::new(0.0, 1.5, 0.0),
            Vec3::new(1.2, 1.2, 1.2),
        );
        world.core.set_name(sphere, Name("Center Sphere".to_string()));

        let colors = ["Red", "Green", "Blue", "Yellow", "Cyan", "Magenta"];
        let mut cube_entities = Vec::new();
        for (index, color) in colors.iter().enumerate() {
            let angle = (index as f32 / colors.len() as f32) * std::f32::consts::TAU;
            let radius = 3.5;
            let position = Vec3::new(angle.cos() * radius, 0.5, angle.sin() * radius);
            let entity = spawn_mesh(world, "Cube", position, Vec3::new(0.8, 0.8, 0.8));
            let name = format!("Cube_{index}");
            world.core.set_name(entity, Name(name.clone()));
            world.core.set_material_ref(entity, MaterialRef::new(color.to_string()));
            cube_entities.push((entity, name));
        }

        let mut state = self.shared.borrow_mut();
        state.next_viewport_index = 1;
        state.next_cube_index = colors.len();

        let mut tree = UiTreeBuilder::new(world);

        let theme = tree
            .world_mut()
            .resources
            .retained_ui
            .theme_state
            .active_theme();
        let menu_font = theme.font_size;
        let menu_text_color = theme.text_color;

        let top_panel = tree.add_docked_panel_top("", 26.0);
        tree.world_mut()
            .ui_panel_set_header_visible(top_panel, false);
        if let Some(UiWidgetState::Panel(data)) =
            tree.world_mut().ui.get_ui_widget_state_mut(top_panel)
        {
            data.min_size = nalgebra_glm::Vec2::new(0.0, 26.0);
            data.resizable = false;
        }
        let mut file_button = Entity::default();
        let mut view_button = Entity::default();
        let mut add_button = Entity::default();
        if let Some(content) = tree
            .world_mut()
            .widget::<UiPanelData>(top_panel)
            .map(|d| d.content_entity)
        {
            if let Some(node) = tree.world_mut().ui.get_ui_layout_node_mut(content) {
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
                tree.add_node()
                    .flow_child(Ab(nalgebra_glm::Vec2::new(44.0, item_height)))
                    .with_text(label, menu_font * 0.85)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_color::<UiBase>(menu_text_color)
                    .with_color::<UiHover>(menu_hover_color)
                    .with_interaction()
                    .with_cursor_icon(winit::window::CursorIcon::Pointer)
                    .entity()
            };

            file_button = make_menu_item(&mut tree, "File");
            view_button = make_menu_item(&mut tree, "View");
            add_button = make_menu_item(&mut tree, "Add");

            let spacer = tree
                .add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(0.0, item_height)))
                .without_pointer_events()
                .entity();
            if let Some(node) = tree.world_mut().ui.get_ui_layout_node_mut(spacer) {
                node.flex_grow = Some(1.0);
            }

            self.fps_text_entity = tree
                .add_node()
                .flow_child(Ab(nalgebra_glm::Vec2::new(80.0, item_height)))
                .with_text("FPS: 0", menu_font * 0.85)
                .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
                .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0))
                .without_pointer_events()
                .done();

            tree.pop_parent();
        }

        let file_menu = tree.add_context_menu(&[
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
            .item("New Viewport", "")
            .separator()
            .widget_row("Show Grid");
        let grid_toggle_command_id = 7;
        let view_menu = tree.add_context_menu_from_builder(view_builder);
        if let Some(content) = tree
            .world_mut()
            .ui_context_menu_widget_content(view_menu, grid_toggle_command_id)
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
        let add_menu = tree.add_context_menu_from_builder(add_builder);

        let left_panel = tree.add_docked_panel_left("Explorer", 220.0);
        if let Some(content) = tree
            .world_mut()
            .widget::<UiPanelData>(left_panel)
            .map(|d| d.content_entity)
        {
            tree.push_parent(content);

            tree.build_ui(tree.current_parent(), |ui| {
                ui.text_input("tree_filter", "Filter nodes...");
            });

            let tree_wrapper = tree
                .add_node()
                .flow_child(Rl(nalgebra_glm::Vec2::new(100.0, 0.0)))
                .flex_grow(1.0)
                .without_pointer_events()
                .entity();
            tree.push_parent(tree_wrapper);
            state.tree_view = tree.add_tree_view(false);
            tree.pop_parent();
            let tv_content = tree
                .world_mut()
                .widget::<UiTreeViewData>(state.tree_view)
                .map(|d| d.content_entity)
                .unwrap();

            let scene_node = tree.add_tree_node(state.tree_view, tv_content, "Scene", 0, 0);
            tree.world_mut().ui_tree_node_set_expanded(scene_node, true);
            let children = tree
                .world_mut()
                .widget::<UiTreeNodeData>(scene_node)
                .map(|d| d.children_container)
                .unwrap();

            let camera_node = tree.add_tree_node(state.tree_view, children, "Main Camera", 1, 1);
            state.scene_entities.push(SceneEntity {
                entity: camera,
                tree_node: camera_node,
                name: "Main Camera".to_string(),
            });

            let sun_node = tree.add_tree_node(state.tree_view, children, "Sun Light", 1, 2);
            state.scene_entities.push(SceneEntity {
                entity: sun,
                tree_node: sun_node,
                name: "Sun Light".to_string(),
            });

            let floor_node = tree.add_tree_node(state.tree_view, children, "Floor", 1, 3);
            state.scene_entities.push(SceneEntity {
                entity: floor,
                tree_node: floor_node,
                name: "Floor".to_string(),
            });

            let cubes_node = tree.add_tree_node(state.tree_view, children, "Cubes", 1, 4);
            tree.world_mut().ui_tree_node_set_expanded(cubes_node, true);
            state.cubes_tree_children = tree
                .world_mut()
                .widget::<UiTreeNodeData>(cubes_node)
                .map(|d| d.children_container)
                .unwrap();
            for (entity, name) in &cube_entities {
                let node = tree.add_tree_node(
                    state.tree_view,
                    state.cubes_tree_children,
                    name,
                    2,
                    entity.id as u64,
                );
                state.scene_entities.push(SceneEntity {
                    entity: *entity,
                    tree_node: node,
                    name: name.clone(),
                });
            }

            let sphere_node = tree.add_tree_node(state.tree_view, children, "Center Sphere", 1, 5);
            state.scene_entities.push(SceneEntity {
                entity: sphere,
                tree_node: sphere_node,
                name: "Center Sphere".to_string(),
            });

            let tree_filter_target = state.tree_view;
            tree.world_mut().ui_react::<String, _>(
                "tree_filter",
                move |val: String, world: &mut World| {
                    world.ui_tree_view_set_filter(tree_filter_target, &val);
                },
            );

            tree.pop_parent();
        }

        let context_menu = tree.add_context_menu(&[
            ("Rename", Some("F2")),
            ("Duplicate", Some("Ctrl+D")),
            ("", None),
            ("Delete", Some("Del")),
        ]);

        let right_panel = tree.add_docked_panel_right("Inspector", 300.0);
        if let Some(content) = tree
            .world_mut()
            .widget::<UiPanelData>(right_panel)
            .map(|d| d.content_entity)
        {
            tree.push_parent(content);

            let grid = tree.add_property_grid(55.0);

            let transform_section = tree.add_property_section(grid, "Transform");

            let area = tree.add_property_row(grid, transform_section, "Pos");
            tree.push_parent(area);
            let pos_editor = tree.add_composite::<Vec3Editor>();
            let (pos_x, pos_y, pos_z) = tree
                .world_mut()
                .ui_composite::<Vec3Editor>(pos_editor)
                .map(|e| (e.x, e.y, e.z))
                .unwrap();
            tree.world_mut()
                .ui_register_named("inspector.pos_x", pos_x, 0.0_f32);
            tree.world_mut()
                .ui_register_named("inspector.pos_y", pos_y, 0.0_f32);
            tree.world_mut()
                .ui_register_named("inspector.pos_z", pos_z, 0.0_f32);
            tree.pop_parent();

            let area = tree.add_property_row(grid, transform_section, "Rot Y");
            tree.push_parent(area);
            let rot_y_entity = tree.add_drag_value_configured(
                DragValueConfig::new(0.0, 360.0, 0.0)
                    .speed(0.5)
                    .precision(1),
            );
            tree.world_mut()
                .ui_register_named("inspector.rot_y", rot_y_entity, 0.0_f32);
            tree.pop_parent();

            let area = tree.add_property_row(grid, transform_section, "Scale");
            tree.push_parent(area);
            let scale_entity =
                tree.add_drag_value_configured(DragValueConfig::new(0.1, 10.0, 1.0).speed(0.01));
            tree.world_mut()
                .ui_register_named("inspector.scale", scale_entity, 1.0_f32);
            tree.pop_parent();

            let display_section = tree.add_property_section(grid, "Display");

            let area = tree.add_property_row(grid, display_section, "Visible");
            tree.push_parent(area);
            let visible_toggle = tree.add_toggle(true);
            tree.world_mut()
                .ui_register_named("inspector.visible", visible_toggle, true);
            tree.pop_parent();

            let area = tree.add_property_row(grid, display_section, "Shadow");
            tree.push_parent(area);
            tree.add_checkbox("Cast", true);
            tree.pop_parent();

            tree.pop_parent();
        }

        let bottom_panel = tree.add_docked_panel_bottom("Console", 150.0);
        if let Some(content) = tree
            .world_mut()
            .widget::<UiPanelData>(bottom_panel)
            .map(|d| d.content_entity)
        {
            tree.push_parent(content);

            state.log_text_entity = tree
                .add_node()
                .flow_child(
                    Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                        + Ab(nalgebra_glm::Vec2::new(0.0, 200.0)),
                )
                .with_text("", 12.0)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.6, 0.6, 0.7, 1.0))
                .without_pointer_events()
                .done();
            tree.pop_parent();
        }

        state.tile_container = tree.add_tile_container(nalgebra_glm::Vec2::new(800.0, 600.0));

        let hint_color = nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0);
        state.tile_console_lines = vec![
            "[INFO] Tile docking system initialized".into(),
            "[INFO] 5 panes created".into(),
            "[INFO] Drag tab headers to rearrange".into(),
            "[INFO] Drop on edges to split, center to merge".into(),
        ];
        state.tile_output_lines = vec![
            "[build] Compiling nightshade v0.7.0".into(),
            "[build] Compiling docking v0.1.0".into(),
            "[build] Finished dev [unoptimized] in 3.2s".into(),
        ];
        let console_initial = state.tile_console_lines.join("\n");
        let output_initial = state.tile_output_lines.join("\n");

        tree.build_tiles(state.tile_container, |tiles| {
            let (scene_info_id, scene_info_content) = tiles.pane("Scene Info").unwrap();
            self.tile_scene_info_pane = scene_info_id;
            tiles.content(scene_info_content, |tree| {
                tree.add_label("Scene Information");
                tree.add_separator();
                self.tile_scene_info_text = tree
                    .add_node()
                    .flow_child(
                        Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                            + Ab(nalgebra_glm::Vec2::new(0.0, 120.0)),
                    )
                    .with_text("Entities: 0", 12.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                    .with_color::<UiBase>(hint_color)
                    .without_pointer_events()
                    .done();
            });

            let (console_id, console_content) = tiles
                .split_from(scene_info_id, SplitDirection::Horizontal, 0.7, "Console")
                .unwrap();
            tiles.content(console_content, |tree| {
                state.tile_console_text = tree
                    .add_node()
                    .flow_child(
                        Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                            + Ab(nalgebra_glm::Vec2::new(0.0, 400.0)),
                    )
                    .with_text(&console_initial, 12.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                    .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.6, 0.8, 0.6, 1.0))
                    .without_pointer_events()
                    .done();
            });

            let (_, output_content) = tiles.pane_sibling(console_id, "Output").unwrap();
            tiles.content(output_content, |tree| {
                state.tile_output_text = tree
                    .add_node()
                    .flow_child(
                        Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                            + Ab(nalgebra_glm::Vec2::new(0.0, 400.0)),
                    )
                    .with_text(&output_initial, 12.0)
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

        let floating_panel_a = tree.add_floating_panel(
            "Scene",
            Rect {
                min: nalgebra_glm::Vec2::new(270.0, 80.0),
                max: nalgebra_glm::Vec2::new(520.0, 280.0),
            },
        );
        let mut add_entity_button = Entity::default();
        let mut tile_add_pane_button = Entity::default();
        if let Some(content) = tree
            .world_mut()
            .widget::<UiPanelData>(floating_panel_a)
            .map(|d| d.content_entity)
        {
            tree.push_parent(content);
            tree.add_label("Spawn new entities into the scene:");
            add_entity_button = tree.add_button("Add Cube");
            tree.add_separator();
            tree.add_label("Dynamic tile panes:");
            tile_add_pane_button = tree.add_button("Add Tile Pane");
            tree.pop_parent();
        }

        let floating_panel_b = tree.add_floating_panel(
            "Actions",
            Rect {
                min: nalgebra_glm::Vec2::new(340.0, 150.0),
                max: nalgebra_glm::Vec2::new(600.0, 320.0),
            },
        );
        let mut delete_button = Entity::default();
        if let Some(content) = tree
            .world_mut()
            .widget::<UiPanelData>(floating_panel_b)
            .map(|d| d.content_entity)
        {
            tree.push_parent(content);
            tree.add_label("Click to open confirm dialog:");
            delete_button = tree.add_button("Delete All Cubes");
            tree.pop_parent();
        }

        let confirm_dialog = tree.add_confirm_dialog(
            "Confirm Delete",
            "Are you sure you want to delete all cubes?",
        );

        state.panels = [
            (top_panel, "Menu"),
            (left_panel, "Explorer"),
            (right_panel, "Inspector"),
            (bottom_panel, "Console"),
            (floating_panel_a, "Scene"),
            (floating_panel_b, "Actions"),
        ];

        state.command_palette = tree.add_command_palette(10);
        tree.finish();

        let command_palette = state.command_palette;
        world.ui_command_palette_register(command_palette, "New Scene", "Ctrl+N", "File");
        world.ui_command_palette_register(command_palette, "Save Layout", "Ctrl+S", "File");
        world.ui_command_palette_register(command_palette, "Load Layout", "Ctrl+O", "File");
        world.ui_command_palette_register(command_palette, "Reset Layout", "", "File");
        world.ui_command_palette_register(command_palette, "Toggle Explorer", "", "View");
        world.ui_command_palette_register(command_palette, "Toggle Inspector", "", "View");
        world.ui_command_palette_register(command_palette, "Toggle Console", "", "View");
        world.ui_command_palette_register(command_palette, "Toggle Tile Tree", "T", "View");
        world.ui_command_palette_register(command_palette, "Toggle Grid", "G", "View");
        world.ui_command_palette_register(command_palette, "Add Cube", "", "Create");
        world.ui_command_palette_register(command_palette, "Add Sphere", "", "Create");
        world.ui_command_palette_register(command_palette, "Add Cylinder", "", "Create");
        world.ui_command_palette_register(command_palette, "Delete All Cubes", "", "Action");

        for (button, menu) in [
            (file_button, file_menu),
            (view_button, view_menu),
            (add_button, add_menu),
        ] {
            world.ui_react_clicked(button, move |world: &mut World| {
                let pos = world
                    .ui.get_ui_layout_node(button)
                    .map(|n| nalgebra_glm::Vec2::new(n.computed_rect.min.x, n.computed_rect.max.y))
                    .unwrap_or_default();
                world.ui_show_context_menu(menu, pos);
            });
        }

        world.ui_react_clicked(delete_button, move |world: &mut World| {
            world.ui_show_modal(confirm_dialog);
        });

        state.log_lines = vec![
            "[INFO] Application started".into(),
            "[INFO] Drag panel headers to undock".into(),
            "[INFO] Select tree nodes to inspect entities".into(),
        ];
        let log_text = state.log_lines.join("\n");
        world.ui_set_text(state.log_text_entity, &log_text);

        drop(state);

        let tree_view = self.shared.borrow().tree_view;

        let shared = self.shared.clone();
        world.ui_react_menu_selected(file_menu, move |clicked, world: &mut World| {
            let mut state = shared.borrow_mut();
            match clicked {
                0 => {
                    let spawned: Vec<Entity> = state
                        .scene_entities
                        .iter()
                        .filter(|s| {
                            s.name.starts_with("Cube_")
                                || s.name.starts_with("Sphere_")
                                || s.name.starts_with("Cylinder_")
                        })
                        .map(|s| s.entity)
                        .collect();
                    despawn_entities_with_cache_cleanup(world, &spawned);
                    state.scene_entities.retain(|s| {
                        !s.name.starts_with("Cube_")
                            && !s.name.starts_with("Sphere_")
                            && !s.name.starts_with("Cylinder_")
                    });
                    state.selected_scene_entity = None;
                    state.push_log(world, "[FILE] New scene");
                }
                2 => {
                    let container = state.tile_container;
                    if let Some(layout) = world.ui_tile_save_layout(container) {
                        state.saved_layout = Some(layout);
                        state.push_log(world, "[FILE] Layout saved");
                    }
                }
                3 => {
                    if let Some(layout) = state.saved_layout.clone() {
                        let container = state.tile_container;
                        let pane_mappings = world.ui_tile_load_layout(container, &layout);
                        state.push_log(
                            world,
                            &format!("[FILE] Layout loaded ({} panes)", pane_mappings.len()),
                        );
                    } else {
                        state.push_log(world, "[FILE] No saved layout");
                    }
                }
                5 => state.push_log(world, "[FILE] Reset layout"),
                _ => {}
            }
        });

        let shared = self.shared.clone();
        world.ui_react_menu_selected(view_menu, move |clicked, world: &mut World| {
            let mut state = shared.borrow_mut();
            match clicked {
                index @ 0..=2 => {
                    let panel_index = index + 1;
                    let (panel, name) = state.panels[panel_index];
                    let currently_visible = world
                        .ui.get_ui_layout_node(panel)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(panel, !currently_visible);
                    let status = if currently_visible { "hidden" } else { "shown" };
                    state.push_log(world, &format!("[VIEW] {name} {status}"));
                }
                3 => {
                    let container = state.tile_container;
                    let currently_visible = world
                        .ui.get_ui_layout_node(container)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(container, !currently_visible);
                    let status = if currently_visible { "hidden" } else { "shown" };
                    state.push_log(world, &format!("[VIEW] Tile Tree {status}"));
                }
                4 => {
                    world.resources.graphics.show_grid = !world.resources.graphics.show_grid;
                    let status = if world.resources.graphics.show_grid {
                        "on"
                    } else {
                        "off"
                    };
                    state.push_log(world, &format!("[VIEW] Grid {status}"));
                }
                5 => state.push_log(world, "[VIEW] Wireframe toggled"),
                6 => {
                    let title = format!("Viewport {}", state.next_viewport_index);
                    state.next_viewport_index += 1;
                    world
                        .resources
                        .secondary_windows
                        .pending_spawns
                        .push(WindowSpawnRequest {
                            title,
                            width: 800,
                            height: 600,
                            egui_enabled: false,
                        });
                    state.push_log(world, "[VIEW] New Viewport opened");
                }
                _ => {}
            }
        });

        let shared = self.shared.clone();
        world.ui_react_menu_selected(add_menu, move |clicked, world: &mut World| {
            let mut state = shared.borrow_mut();
            let mesh_name = match clicked {
                0 => "Cube",
                1 => "Sphere",
                2 => "Cylinder",
                3 => {
                    state.push_log(world, "[ADD] Point Light (not implemented)");
                    return;
                }
                4 => {
                    state.push_log(world, "[ADD] Spot Light (not implemented)");
                    return;
                }
                _ => return,
            };
            let name = state.spawn_mesh_entity(world, mesh_name);
            state.push_log(world, &format!("[ADD] {name}"));
        });

        let shared = self.shared.clone();
        world.ui_react_tree_selected(tree_view, move |node, world: &mut World| {
            let is_selected = world
                .widget::<UiTreeViewData>(tree_view)
                .map(|d| d.selected_nodes.contains(&node))
                .unwrap_or(false);
            if !is_selected {
                return;
            }
            let mut state = shared.borrow_mut();
            if let Some(scene) = state.scene_entities.iter().find(|s| s.tree_node == node) {
                let entity = scene.entity;
                let name = scene.name.clone();
                state.selected_scene_entity = Some(entity);
                state.push_log(world, &format!("[SELECT] {name}"));
                load_entity_to_inspector(world, entity);
            }
        });

        let shared = self.shared.clone();
        world.ui_react_tree_context_menu(tree_view, move |_node, position, world: &mut World| {
            world.ui_show_context_menu(context_menu, position);
            let _ = shared; // ensure shared is captured for lifetime
        });

        let shared = self.shared.clone();
        world.ui_react_menu_selected(context_menu, move |item_index, world: &mut World| {
            let mut state = shared.borrow_mut();
            let selected_nodes = world
                .widget::<UiTreeViewData>(tree_view)
                .map(|d| d.selected_nodes.clone())
                .unwrap_or_default();
            let selected_name = selected_nodes.first().and_then(|node| {
                state
                    .scene_entities
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
                state.push_log(world, &format!("[ACTION] {action}: {name}"));
            } else {
                state.push_log(world, &format!("[ACTION] {action}"));
            }

            if item_index == 3
                && let Some(node) = selected_nodes.first()
                && let Some(index) = state
                    .scene_entities
                    .iter()
                    .position(|s| s.tree_node == *node)
            {
                let scene = state.scene_entities.remove(index);
                despawn_entities_with_cache_cleanup(world, &[scene.entity]);
                if state.selected_scene_entity == Some(scene.entity) {
                    state.selected_scene_entity = None;
                }
            }
        });

        let shared = self.shared.clone();
        world.ui_react_any(
            &[
                "inspector.pos_x",
                "inspector.pos_y",
                "inspector.pos_z",
                "inspector.rot_y",
                "inspector.scale",
            ],
            move |world: &mut World| {
                let state = shared.borrow();
                let Some(selected) = state.selected_scene_entity else {
                    return;
                };
                drop(state);

                let pos = nalgebra_glm::Vec3::new(
                    world.ui_prop::<f32>("inspector.pos_x"),
                    world.ui_prop::<f32>("inspector.pos_y"),
                    world.ui_prop::<f32>("inspector.pos_z"),
                );
                let rot_y: f32 = world.ui_prop("inspector.rot_y");
                let scale_val: f32 = world.ui_prop("inspector.scale");

                if let Some(transform) = world.core.get_local_transform_mut(selected) {
                    transform.translation = pos;
                    transform.rotation =
                        nalgebra_glm::quat_angle_axis(rot_y.to_radians(), &Vec3::y());
                    transform.scale = Vec3::new(scale_val, scale_val, scale_val);
                }
                world.core.set_local_transform_dirty(selected, LocalTransformDirty);
            },
        );

        let shared = self.shared.clone();
        world.ui_react::<bool, _>("inspector.visible", move |val: bool, world: &mut World| {
            shared
                .borrow_mut()
                .push_log(world, &format!("[TOGGLE] Visible = {val}"));
        });

        let shared = self.shared.clone();
        world.ui_react_clicked(add_entity_button, move |world: &mut World| {
            let mut state = shared.borrow_mut();
            let name = state.spawn_mesh_entity(world, "Cube");
            state.push_log(world, &format!("[SPAWN] {name}"));
        });

        let shared = self.shared.clone();
        world.ui_react_confirmed(confirm_dialog, move |confirmed, world: &mut World| {
            let mut state = shared.borrow_mut();
            if confirmed {
                let cube_entities: Vec<Entity> = state
                    .scene_entities
                    .iter()
                    .filter(|s| s.name.starts_with("Cube_"))
                    .map(|s| s.entity)
                    .collect();

                let count = cube_entities.len();
                despawn_entities_with_cache_cleanup(world, &cube_entities);

                state
                    .scene_entities
                    .retain(|s| !s.name.starts_with("Cube_"));
                if let Some(selected) = state.selected_scene_entity
                    && cube_entities.contains(&selected)
                {
                    state.selected_scene_entity = None;
                }

                state.push_log(world, &format!("[DELETE] Removed {count} cubes"));
            } else {
                state.push_log(world, "[ACTION] Cancelled delete");
            }
        });

        let shared = self.shared.clone();
        world.ui_react_command(command_palette, move |command_index, world: &mut World| {
            let mut state = shared.borrow_mut();
            match command_index {
                0 => state.push_log(world, "[CMD] New Scene"),
                1 => state.push_log(world, "[CMD] Save Layout"),
                2 => state.push_log(world, "[CMD] Load Layout"),
                3 => state.push_log(world, "[CMD] Reset Layout"),
                index @ 4..=6 => {
                    let panel_index = index - 3;
                    let (panel, name) = state.panels[panel_index];
                    let visible = world
                        .ui.get_ui_layout_node(panel)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(panel, !visible);
                    state.push_log(world, &format!("[CMD] Toggle {name}"));
                }
                7 => {
                    let container = state.tile_container;
                    let visible = world
                        .ui.get_ui_layout_node(container)
                        .map(|n| n.visible)
                        .unwrap_or(true);
                    world.ui_set_visible(container, !visible);
                    state.push_log(world, "[CMD] Toggle Tile Tree");
                }
                8 => {
                    world.resources.graphics.show_grid = !world.resources.graphics.show_grid;
                    state.push_log(world, "[CMD] Toggle Grid");
                }
                9 => {
                    let name = state.spawn_mesh_entity(world, "Cube");
                    state.push_log(world, &format!("[CMD] Added {name}"));
                }
                10 => {
                    let name = state.spawn_mesh_entity(world, "Sphere");
                    state.push_log(world, &format!("[CMD] Added {name}"));
                }
                11 => {
                    let name = state.spawn_mesh_entity(world, "Cylinder");
                    state.push_log(world, &format!("[CMD] Added {name}"));
                }
                12 => state.push_log(world, "[CMD] Delete All Cubes"),
                _ => {}
            }
        });

        let shared = self.shared.clone();
        world.ui_react_clicked(tile_add_pane_button, move |world: &mut World| {
            let mut state = shared.borrow_mut();
            state.tile_pane_counter += 1;
            let title = format!("Pane {}", state.tile_pane_counter);
            let container = state.tile_container;
            let pane_text = format!(
                "Dynamic pane \"{title}\" created at runtime.\nDrag this tab to rearrange it.\nDrop on edges to split into new area."
            );
            world.build_tiles(container, |tiles| {
                if let Some((_pane_id, content)) = tiles.pane(&title) {
                    tiles.content(content, |tree| {
                        tree.add_label(&title);
                        tree.add_separator();
                        tree.add_node()
                            .flow_child(
                                Rl(nalgebra_glm::Vec2::new(100.0, 0.0))
                                    + Ab(nalgebra_glm::Vec2::new(0.0, 100.0)),
                            )
                            .with_text(&pane_text, 12.0)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                            .with_color::<UiBase>(nalgebra_glm::Vec4::new(0.5, 0.5, 0.6, 1.0))
                            .without_pointer_events()
                            .done();
                    });
                }
            });
            state.push_log(world, &format!("[TILE] Added {title}"));
            state.push_tile_output(world, &format!("[new] Created tile pane: {title}"));
        });
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        let delta_time = world.resources.window.timing.delta_time;
        self.shared.borrow_mut().total_time += delta_time;

        let command_palette = self.shared.borrow().command_palette;
        for &(key_code, pressed) in &world.resources.input.keyboard.frame_keys.clone() {
            if pressed
                && key_code == KeyCode::KeyP
                && world
                    .resources
                    .input
                    .keyboard
                    .is_key_pressed(KeyCode::ControlLeft)
            {
                world.ui_show_command_palette(command_palette);
            }
        }

        self.handle_tile_events(world);
        self.check_panel_events(world);
        self.update_scene_info(world);
        self.forward_input_to_secondary_worlds(world);

        let fps = world.resources.window.timing.frames_per_second;
        world.ui_set_text(self.fps_text_entity, &format!("FPS: {fps}"));
    }

    fn pre_render(&mut self, renderer: &mut dyn Render, world: &mut World) {
        let active_indices: Vec<usize> = world
            .resources
            .secondary_windows
            .states
            .iter()
            .map(|s| s.index)
            .collect();
        self.secondary_worlds
            .retain(|index, _| active_indices.contains(index));

        let new_windows: Vec<(usize, String)> = world
            .resources
            .secondary_windows
            .states
            .iter()
            .filter(|s| !self.secondary_worlds.contains_key(&s.index))
            .map(|s| (s.index, s.title.clone()))
            .collect();
        for (window_index, title) in new_windows {
            let instance = self.create_secondary_world(renderer, &title);
            self.secondary_worlds.insert(window_index, instance);
        }

        for (&index, instance) in &mut self.secondary_worlds {
            let _ = renderer.render_world_to_secondary_surface(index, &mut instance.world);
        }
    }
}

impl DockingDemo {
    fn handle_tile_events(&self, world: &mut World) {
        let mut state = self.shared.borrow_mut();
        let container = state.tile_container;
        if let Some(pane_id) = world.ui_tile_tab_activated(container) {
            let title = world
                .ui_tile_pane_title(container, pane_id)
                .unwrap_or_default();
            state.push_tile_output(world, &format!("[TILE] Tab activated: {title}"));
        }
        if let Some((_pane_id, title)) = world.ui_tile_tab_closed(container) {
            state.push_tile_output(world, &format!("[TILE] Tab closed: {title}"));
        }
        if let Some((_split_id, ratio)) = world.ui_tile_splitter_moved(container) {
            state.push_tile_output(world, &format!("[TILE] Splitter moved: {ratio:.2}"));
        }
    }

    fn check_panel_events(&self, world: &mut World) {
        let mut state = self.shared.borrow_mut();
        let panels = state.panels;
        for (entity, name) in &panels {
            if let Some(UiWidgetState::Panel(data)) = world.ui.get_ui_widget_state(*entity) {
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
                    if state.log_lines.last().map(|s| s.as_str()) != Some(msg.as_str()) {
                        state.push_log(world, &msg);
                    }
                }
            }
        }
    }

    fn update_scene_info(&self, world: &mut World) {
        let state = self.shared.borrow();
        if !world.ui_tile_active_pane(state.tile_container, self.tile_scene_info_pane) {
            return;
        }
        let entity_count = state.scene_entities.len();
        let (tile_count, pane_count) = if let Some(UiWidgetState::TileContainer(data)) =
            world.ui.get_ui_widget_state(state.tile_container)
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
            state.total_time
        );
        world.ui_set_text(self.tile_scene_info_text, &info);
    }

    fn create_secondary_world(
        &mut self,
        renderer: &dyn Render,
        title: &str,
    ) -> SecondaryWorldInstance {
        let world_id = self.next_world_id;
        self.next_world_id += 1;

        let mut new_world = World::default();
        renderer.copy_fonts_to_world(&mut new_world);
        new_world.resources.world_id = world_id as u64 + 1000;
        new_world.resources.retained_ui.enabled = true;
        new_world.resources.graphics.clear_color = [0.02, 0.02, 0.04, 1.0];
        new_world.resources.graphics.show_grid = true;

        let yaw = world_id as f32 * 0.8;
        let camera = spawn_pan_orbit_camera(
            &mut new_world,
            Vec3::new(0.0, 1.0, 0.0),
            12.0,
            yaw,
            0.4,
            format!("{title} Camera"),
        );
        new_world.resources.active_camera = Some(camera);

        let sun = spawn_sun(&mut new_world);
        if let Some(light) = new_world.core.get_light_mut(sun) {
            light.color = Vec3::new(0.9, 0.85, 0.8);
            light.intensity = 1.0;
            light.cast_shadows = true;
        }

        spawn_mesh(
            &mut new_world,
            "Cube",
            Vec3::new(0.0, -0.25, 0.0),
            Vec3::new(20.0, 0.5, 20.0),
        );

        let colors = ["Red", "Green", "Blue", "Yellow", "Cyan", "Magenta"];
        for (index, color) in colors.iter().enumerate() {
            let angle = (index as f32 / colors.len() as f32) * std::f32::consts::TAU;
            let radius = 3.5;
            let position = Vec3::new(angle.cos() * radius, 0.5, angle.sin() * radius);
            let entity = spawn_mesh(&mut new_world, "Cube", position, Vec3::new(0.8, 0.8, 0.8));
            new_world.core.set_material_ref(entity, MaterialRef::new(color.to_string()));
        }

        spawn_mesh(
            &mut new_world,
            "Sphere",
            Vec3::new(0.0, 1.5, 0.0),
            Vec3::new(1.2, 1.2, 1.2),
        );

        let mut tree = UiTreeBuilder::new(&mut new_world);

        let theme = tree
            .world_mut()
            .resources
            .retained_ui
            .theme_state
            .active_theme();
        let panel_color = theme.panel_color;
        let text_color = theme.text_color;
        let font_size = theme.font_size;

        tree.add_node()
            .boundary(
                Ab(nalgebra_glm::Vec2::new(0.0, 0.0)),
                Rl(nalgebra_glm::Vec2::new(100.0, 0.0)) + Ab(nalgebra_glm::Vec2::new(0.0, 32.0)),
            )
            .with_rect(0.0, 0.0, nalgebra_glm::Vec4::zeros())
            .with_color::<UiBase>(panel_color)
            .flow(FlowDirection::Horizontal, 8.0, 8.0)
            .with_children(|tree| {
                tree.add_node()
                    .flow_child(Ab(nalgebra_glm::Vec2::new(0.0, 20.0)))
                    .flex_grow(1.0)
                    .with_text(title, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_color::<UiBase>(text_color)
                    .without_pointer_events()
                    .done();
            })
            .done();

        let panel = tree.add_floating_panel(
            "Controls",
            Rect {
                min: nalgebra_glm::Vec2::new(16.0, 48.0),
                max: nalgebra_glm::Vec2::new(240.0, 280.0),
            },
        );
        let mut toast_button = Entity::default();
        if let Some(content) = tree
            .world_mut()
            .widget::<UiPanelData>(panel)
            .map(|d| d.content_entity)
        {
            tree.push_parent(content);
            tree.add_label("Grid");
            tree.add_toggle(true);
            tree.add_separator();
            tree.add_label("Orbit Speed");
            tree.add_slider(0.5, 0.0, 2.0);
            tree.add_separator();
            tree.add_label("Light Intensity");
            tree.add_slider(1.0, 0.0, 5.0);
            tree.add_separator();
            tree.add_button("Reset Camera");
            tree.add_separator();
            toast_button = tree.add_button("Show Toast");
            tree.pop_parent();
        }

        tree.finish();

        new_world.ui_react_clicked(toast_button, |world: &mut World| {
            world.ui_show_toast("Hello from this viewport!", ToastSeverity::Info, 3.0);
        });

        SecondaryWorldInstance { world: new_world }
    }

    fn forward_input_to_secondary_worlds(&mut self, world: &mut World) {
        for secondary_window in &world.resources.secondary_windows.states {
            if let Some(instance) = self.secondary_worlds.get_mut(&secondary_window.index) {
                instance.world.resources.input.mouse.position =
                    secondary_window.input.mouse_position;
                instance.world.resources.input.mouse.state = secondary_window.input.mouse_state;
                instance.world.resources.input.mouse.position_delta =
                    secondary_window.input.mouse_position_delta;
                instance.world.resources.input.mouse.raw_mouse_delta =
                    secondary_window.input.raw_mouse_delta;
                instance.world.resources.input.mouse.wheel_delta =
                    secondary_window.input.mouse_wheel_delta;
                instance.world.resources.input.keyboard.keystates = secondary_window
                    .input
                    .keyboard_keystates
                    .iter()
                    .map(|(k, v)| (*k, *v))
                    .collect();
                instance.world.resources.input.keyboard.frame_keys =
                    secondary_window.input.frame_keys.clone();
                instance.world.resources.input.keyboard.frame_chars =
                    secondary_window.input.frame_chars.clone();
                instance.world.resources.user_interface.hud_wants_pointer = false;
                instance.world.resources.window.timing.delta_time =
                    world.resources.window.timing.delta_time;
                let (width, height) = secondary_window.size;
                instance.world.resources.window.cached_viewport_size = Some((width, height));
                pan_orbit_camera_system(&mut instance.world);
                run_retained_ui_systems(&mut instance.world);
            }
        }

        for secondary_window in &mut world.resources.secondary_windows.states {
            secondary_window.input.mouse_state.remove(
                MouseState::LEFT_JUST_PRESSED
                    | MouseState::LEFT_JUST_RELEASED
                    | MouseState::MIDDLE_JUST_PRESSED
                    | MouseState::MIDDLE_JUST_RELEASED
                    | MouseState::RIGHT_JUST_PRESSED
                    | MouseState::RIGHT_JUST_RELEASED
                    | MouseState::MOVED
                    | MouseState::SCROLLED,
            );
            secondary_window.input.raw_mouse_delta = nalgebra_glm::Vec2::zeros();
            secondary_window.input.mouse_wheel_delta = nalgebra_glm::Vec2::zeros();
            secondary_window.input.mouse_position_delta = nalgebra_glm::Vec2::zeros();
        }
    }
}
