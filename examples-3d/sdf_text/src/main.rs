use nightshade::ecs::camera::commands::spawn_camera;
use nightshade::prelude::*;

#[derive(Default)]
struct SdfTextDemoState {
    selected_entity: Option<Entity>,
    custom_font_loaded: bool,
    pending_font_index: Option<usize>,
}

impl State for SdfTextDemoState {
    fn title(&self) -> &str {
        "SDF Text Demo - Nightshade"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;

        let text_entity = spawn_default_text(world);
        self.selected_entity = Some(text_entity);

        spawn_outline_demo(world);
        spawn_kerning_demo(world);

        let camera = spawn_camera(world, Vec3::new(0.0, 4.0, 10.0), "Main Camera".to_string());
        if let Some(camera_component) = world.core.get_camera_mut(camera) {
            camera_component.projection = Projection::Perspective(PerspectiveCamera::default());
        }
        world.resources.active_camera = Some(camera);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::TopBottomPanel::top("top").show(ui_context, |ui| {
            ui.horizontal(|ui| {
                ui.label("Text Tests:");
                if ui.button("Spawn 3D Text").clicked() {
                    spawn_3d_text(world);
                }
                if ui.button("Spawn Billboard Text").clicked() {
                    spawn_billboard_text(world);
                }
                if ui.button("Spawn 5K Text Lattice (Stress Test)").clicked() {
                    spawn_text_lattice(world);
                }
                ui.separator();
                if !self.custom_font_loaded && ui.button("Load Custom Font").clicked() {
                    let font_data = include_bytes!("../../../assets/fonts/LoveDays.ttf").to_vec();
                    if let Ok(font_index) = load_font_from_bytes(world, font_data, 32.0) {
                        self.pending_font_index = Some(font_index);
                        self.custom_font_loaded = true;
                    }
                }
                ui.separator();
                ui.label(format!("Entities: {}", world.core.get_all_entities().len()));
            });
        });

        egui::Window::new("Text Controls")
            .default_width(300.0)
            .resizable(true)
            .show(ui_context, |ui| {
                if let Some(entity) = self.selected_entity
                    && world.core.get_text(entity).is_some()
                {
                    self.text_property_ui(world, ui, entity);
                }

                ui.separator();
                ui.collapsing("Text Resources", |ui| {
                    self.text_resources_ui(world, ui);
                });
            });
    }

    fn run_systems(&mut self, world: &mut World) {
        nightshade::ecs::camera::systems::fly_camera_system(world);
        nightshade::ecs::text::systems::sync_text_meshes_system(world);

        if let Some(font_index) = self.pending_font_index
            && world
                .resources
                .text_cache
                .font_manager
                .get_font(font_index)
                .is_some()
        {
            if let Some(entity) = self.selected_entity
                && let Some(text) = world.core.get_text_mut(entity)
            {
                text.font_index = font_index;
                text.dirty = true;
            }
            self.pending_font_index = None;
        }
    }
}

impl SdfTextDemoState {
    fn text_property_ui(&mut self, world: &mut World, ui: &mut egui::Ui, entity: Entity) {
        let (text_index, current_content) = if let Some(text) = world.core.get_text(entity) {
            let content = world
                .resources
                .text_cache
                .get_text(text.text_index)
                .map(|s| s.to_string())
                .unwrap_or_default();
            (text.text_index, content)
        } else {
            return;
        };

        let mut text_content = current_content.clone();
        ui.horizontal(|ui| {
            ui.label("Content:");
        });
        ui.text_edit_multiline(&mut text_content);
        if text_content != current_content {
            world
                .resources
                .text_cache
                .set_text(text_index, text_content);
            if let Some(text) = world.core.get_text_mut(entity) {
                text.dirty = true;
            }
        }

        if let Some(text) = world.core.get_text_mut(entity) {
            let mut changed = false;

            ui.separator();

            if let Some(bounds) = text.get_bounds() {
                ui.label(format!("Bounds: {:.2} x {:.2}", bounds.x, bounds.y));
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Font Size:");
                if ui
                    .add(
                        egui::DragValue::new(&mut text.properties.font_size)
                            .speed(0.5)
                            .range(1.0..=200.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                let mut color = [
                    text.properties.color.x,
                    text.properties.color.y,
                    text.properties.color.z,
                    text.properties.color.w,
                ];
                if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
                    text.properties.color = Vec4::new(color[0], color[1], color[2], color[3]);
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Alignment:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", text.properties.alignment))
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(
                                &mut text.properties.alignment,
                                TextAlignment::Left,
                                "Left",
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .selectable_value(
                                &mut text.properties.alignment,
                                TextAlignment::Center,
                                "Center",
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .selectable_value(
                                &mut text.properties.alignment,
                                TextAlignment::Right,
                                "Right",
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("V-Align:");
                egui::ComboBox::from_label("v_align")
                    .selected_text(format!("{:?}", text.properties.vertical_alignment))
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(
                                &mut text.properties.vertical_alignment,
                                VerticalAlignment::Top,
                                "Top",
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .selectable_value(
                                &mut text.properties.vertical_alignment,
                                VerticalAlignment::Middle,
                                "Middle",
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .selectable_value(
                                &mut text.properties.vertical_alignment,
                                VerticalAlignment::Bottom,
                                "Bottom",
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .selectable_value(
                                &mut text.properties.vertical_alignment,
                                VerticalAlignment::Baseline,
                                "Baseline",
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Line Height:");
                if ui
                    .add(
                        egui::DragValue::new(&mut text.properties.line_height)
                            .speed(0.01)
                            .range(0.5..=3.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Letter Spacing:");
                if ui
                    .add(
                        egui::DragValue::new(&mut text.properties.letter_spacing)
                            .speed(0.1)
                            .range(-10.0..=50.0),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.separator();
            ui.label("Outline");

            ui.horizontal(|ui| {
                ui.label("Width:");
                if ui
                    .add(
                        egui::DragValue::new(&mut text.properties.outline_width)
                            .speed(0.001)
                            .range(0.0..=0.5),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                let mut outline_color = [
                    text.properties.outline_color.x,
                    text.properties.outline_color.y,
                    text.properties.outline_color.z,
                    text.properties.outline_color.w,
                ];
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut outline_color)
                    .changed()
                {
                    text.properties.outline_color = Vec4::new(
                        outline_color[0],
                        outline_color[1],
                        outline_color[2],
                        outline_color[3],
                    );
                    changed = true;
                }
            });

            ui.separator();
            if ui
                .checkbox(&mut text.billboard, "Billboard (face camera)")
                .changed()
            {
                changed = true;
            }

            if changed {
                text.dirty = true;
            }
        }
    }

    fn text_resources_ui(&mut self, world: &mut World, ui: &mut egui::Ui) {
        ui.label("Text Strings");
        ui.separator();

        let all_texts = world.resources.text_cache.get_all_text_with_indices();

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for (index, content) in all_texts {
                    let label = format!(
                        "[{}] {}",
                        index,
                        if content.len() > 30 {
                            format!("{}...", &content[..30])
                        } else {
                            content.to_string()
                        }
                    );

                    ui.label(label);
                }
            });
    }
}

fn spawn_default_text(world: &mut World) -> Entity {
    let text_index = world
        .resources
        .text_cache
        .add_text("Sample Text\nMultiline Support");
    let entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::TEXT
            | nightshade::ecs::world::VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(entity) {
        *name = Name("Sample Text".to_string());
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = Vec3::new(0.0, 2.0, 0.0);
    }

    if let Some(text) = world.core.get_text_mut(entity) {
        text.text_index = text_index;
        text.properties = TextProperties {
            font_size: 24.0,
            color: Vec4::new(1.0, 1.0, 0.0, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.02,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };
        text.dirty = true;
    }

    entity
}

fn spawn_3d_text(world: &mut World) {
    let text_index = world.resources.text_cache.add_text("3D Text Object");
    let entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::TEXT
            | nightshade::ecs::world::VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(entity) {
        *name = Name("3D Text".to_string());
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        let offset = rand::random::<f32>() * 10.0 - 5.0;
        transform.translation = Vec3::new(offset, 3.0, offset);
    }

    if let Some(text) = world.core.get_text_mut(entity) {
        text.text_index = text_index;
        text.properties = TextProperties {
            font_size: 32.0,
            color: Vec4::new(
                rand::random::<f32>(),
                rand::random::<f32>(),
                rand::random::<f32>(),
                1.0,
            ),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            ..Default::default()
        };
        text.dirty = true;
    }
}

fn spawn_billboard_text(world: &mut World) {
    let offset = rand::random::<f32>() * 10.0 - 5.0;
    let position = Vec3::new(offset, 3.0, offset);

    spawn_3d_billboard_text_with_properties(
        world,
        "Billboard Text",
        position,
        TextProperties {
            font_size: 32.0,
            color: Vec4::new(0.0, 1.0, 0.5, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.02,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );
}

fn spawn_text_lattice(world: &mut World) {
    const TEXT_COUNT: usize = 5_000;
    const GRID_X: usize = 25;
    const GRID_Y: usize = 20;
    const GRID_Z: usize = 10;
    const SPACING: f32 = 3.0;

    let parent_entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(parent_entity) {
        *name = Name("Text Lattice (Stress Test)".to_string());
    }

    if let Some(transform) = world.core.get_local_transform_mut(parent_entity) {
        transform.translation = Vec3::new(0.0, 5.0, 0.0);
    }

    let text_variations = [
        "Text", "Hello", "World", "Test", "Lorem", "Ipsum", "3D", "Engine", "Render", "Buffer",
        "Dynamic", "Scale", "Vertex", "Index", "GPU", "SDF",
    ];

    let offset_x = -(GRID_X as f32 * SPACING) / 2.0;
    let offset_y = -(GRID_Y as f32 * SPACING) / 2.0;
    let offset_z = -(GRID_Z as f32 * SPACING) / 2.0;

    let mut text_count = 0;

    'outer: for x in 0..GRID_X {
        for y in 0..GRID_Y {
            for z in 0..GRID_Z {
                if text_count >= TEXT_COUNT {
                    break 'outer;
                }

                let text_content = format!(
                    "{} #{}",
                    text_variations[text_count % text_variations.len()],
                    text_count + 1
                );
                let text_index = world.resources.text_cache.add_text(&text_content);

                let entity = world.spawn_entities(
                    nightshade::ecs::world::NAME
                        | nightshade::ecs::world::LOCAL_TRANSFORM
                        | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
                        | nightshade::ecs::world::GLOBAL_TRANSFORM
                        | nightshade::ecs::world::TEXT
                        | nightshade::ecs::world::VISIBILITY
                        | nightshade::ecs::world::PARENT,
                    1,
                )[0];

                if let Some(parent) = world.core.get_parent_mut(entity) {
                    parent.0 = Some(parent_entity);
                }

                if let Some(name) = world.core.get_name_mut(entity) {
                    *name = Name(format!("Text {}", text_count + 1));
                }

                if let Some(transform) = world.core.get_local_transform_mut(entity) {
                    transform.translation = Vec3::new(
                        offset_x + x as f32 * SPACING,
                        offset_y + y as f32 * SPACING,
                        offset_z + z as f32 * SPACING,
                    );
                }

                if let Some(text) = world.core.get_text_mut(entity) {
                    text.text_index = text_index;
                    text.properties = TextProperties {
                        font_size: 16.0,
                        color: Vec4::new(
                            rand::random::<f32>(),
                            rand::random::<f32>(),
                            rand::random::<f32>(),
                            1.0,
                        ),
                        alignment: TextAlignment::Center,
                        vertical_alignment: VerticalAlignment::Middle,
                        ..Default::default()
                    };
                    text.dirty = true;
                }

                text_count += 1;
            }
        }
    }
}

fn spawn_outline_demo(world: &mut World) {
    spawn_3d_text_with_properties(
        world,
        "Outlined Text",
        Vec3::new(-5.0, 4.0, 0.0),
        TextProperties {
            font_size: 48.0,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.1,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );

    spawn_3d_text_with_properties(
        world,
        "Colored Outline",
        Vec3::new(-5.0, 2.5, 0.0),
        TextProperties {
            font_size: 48.0,
            color: Vec4::new(1.0, 0.9, 0.0, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.15,
            outline_color: Vec4::new(0.8, 0.0, 0.2, 1.0),
            ..Default::default()
        },
    );

    spawn_3d_text_with_properties(
        world,
        "Thick Outline",
        Vec3::new(-5.0, 1.0, 0.0),
        TextProperties {
            font_size: 48.0,
            color: Vec4::new(0.2, 0.6, 1.0, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.25,
            outline_color: Vec4::new(0.0, 0.0, 0.3, 1.0),
            ..Default::default()
        },
    );
}

fn spawn_kerning_demo(world: &mut World) {
    spawn_3d_text_with_properties(
        world,
        "AVATAR Wave Typography",
        Vec3::new(5.0, 4.0, 0.0),
        TextProperties {
            font_size: 36.0,
            color: Vec4::new(0.9, 0.9, 0.9, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            ..Default::default()
        },
    );

    spawn_3d_billboard_text_with_properties(
        world,
        "Billboard + Outline",
        Vec3::new(0.0, 6.0, 0.0),
        TextProperties {
            font_size: 32.0,
            color: Vec4::new(0.0, 1.0, 0.5, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.08,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SdfTextDemoState::default())
}
