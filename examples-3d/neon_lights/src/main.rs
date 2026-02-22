mod stroke_font;
mod tube_mesh;

use nightshade::ecs::material::material_registry_insert;
use nightshade::ecs::prefab::mesh_cache_insert;
use nightshade::prelude::*;

use stroke_font::{CHAR_HEIGHT, CHAR_SPACING, CHAR_WIDTH, WORD_SPACING};

const TUBE_RADIUS: f32 = 0.04;
const SIGN_SCALE: f32 = 2.0;
const CHARS_PER_ROW: usize = 13;

static UNIQUE_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn next_unique_id() -> usize {
    UNIQUE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(NeonLightsDemo::default())
}

struct NeonSign {
    text: String,
    tube_entities: Vec<Entity>,
    light_entities: Vec<Entity>,
    color: Vec3,
    position: Vec3,
}

struct NeonLightsDemo {
    signs: Vec<NeonSign>,
    editing_index: Option<usize>,
    text_input: String,
    selected_color_index: usize,
    flicker_enabled: bool,
    flicker_phases: Vec<f32>,
}

impl Default for NeonLightsDemo {
    fn default() -> Self {
        Self {
            signs: Vec::new(),
            editing_index: None,
            text_input: "HELLO".to_string(),
            selected_color_index: 0,
            flicker_enabled: true,
            flicker_phases: Vec::new(),
        }
    }
}

const NEON_COLORS: &[(Vec3, &str)] = &[
    (Vec3::new(1.0, 0.1, 0.3), "Hot Pink"),
    (Vec3::new(0.1, 0.5, 1.0), "Electric Blue"),
    (Vec3::new(0.0, 1.0, 0.4), "Neon Green"),
    (Vec3::new(1.0, 0.6, 0.0), "Orange"),
    (Vec3::new(0.7, 0.0, 1.0), "Purple"),
    (Vec3::new(1.0, 1.0, 0.2), "Yellow"),
    (Vec3::new(0.0, 1.0, 1.0), "Cyan"),
    (Vec3::new(1.0, 0.0, 0.0), "Red"),
];

const ALL_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!?.-':/<>()+=*@#\"~_\\";

impl State for NeonLightsDemo {
    fn title(&self) -> &str {
        "Neon Lights"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.01, 0.005, 0.02, 1.0];
        world.resources.graphics.show_grid = false;
        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.4;

        let camera_position = Vec3::new(0.0, 5.0, 22.0);
        let camera_entity = spawn_camera(world, camera_position);
        world.resources.active_camera = Some(camera_entity);

        spawn_backdrop(world);
        self.spawn_showcase(world);

        spawn_hud_text_with_properties(
            world,
            "WASD to move, Mouse to look",
            HudAnchor::BottomCenter,
            Vec2::new(0.0, -20.0),
            TextProperties {
                font_size: 18.0,
                color: Vec4::new(0.6, 0.6, 0.6, 0.8),
                alignment: TextAlignment::Center,
                ..Default::default()
            },
        );
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);
        sync_text_meshes_system(world);

        if self.flicker_enabled {
            self.update_flicker(world);
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let mut action = UiAction::None;

        egui::Window::new("Neon Sign Maker")
            .default_width(280.0)
            .show(ui_context, |ui| {
                ui.heading("Signs");
                for sign_index in 0..self.signs.len() {
                    let is_selected = self.editing_index == Some(sign_index);
                    let sign_text = &self.signs[sign_index].text;
                    let pos_y = self.signs[sign_index].position.y;
                    let display_text = if sign_text.len() > 20 {
                        format!("{}...", &sign_text[..20])
                    } else {
                        sign_text.clone()
                    };
                    let label = if is_selected {
                        format!("> [{sign_index}] \"{display_text}\" y={pos_y:.1}")
                    } else {
                        format!("  [{sign_index}] \"{display_text}\" y={pos_y:.1}")
                    };
                    if ui.selectable_label(is_selected, label).clicked() {
                        self.editing_index = Some(sign_index);
                        self.text_input = self.signs[sign_index].text.clone();
                        for (color_index, (color, _)) in NEON_COLORS.iter().enumerate() {
                            if (color - self.signs[sign_index].color).norm() < 0.01 {
                                self.selected_color_index = color_index;
                                break;
                            }
                        }
                    }
                }

                ui.separator();

                ui.label("Text:");
                ui.text_edit_singleline(&mut self.text_input);

                ui.separator();
                ui.label("Color:");
                ui.horizontal_wrapped(|ui| {
                    for (color_index, (color, name)) in NEON_COLORS.iter().enumerate() {
                        let selected = color_index == self.selected_color_index;
                        let button = egui::Button::new(
                            egui::RichText::new(if selected {
                                format!("[{name}]")
                            } else {
                                name.to_string()
                            })
                            .color(egui::Color32::from_rgb(
                                (color.x * 255.0) as u8,
                                (color.y * 255.0) as u8,
                                (color.z * 255.0) as u8,
                            )),
                        );
                        if ui.add(button).clicked() {
                            self.selected_color_index = color_index;
                        }
                    }
                });

                ui.separator();

                let has_selection = self.editing_index.is_some();
                let text_not_empty = !self.text_input.trim().is_empty();

                if text_not_empty && ui.button("Add New Sign").clicked() {
                    action = UiAction::AddNew;
                }

                if has_selection && ui.button("Remove Selected Sign").clicked() {
                    action = UiAction::RemoveSelected;
                }

                if !self.signs.is_empty() && ui.button("Clear All Signs").clicked() {
                    action = UiAction::ClearAll;
                }

                ui.separator();
                ui.checkbox(&mut self.flicker_enabled, "Flicker Effect");

                ui.separator();
                ui.label("Bloom Intensity:");
                let mut bloom = world.resources.graphics.bloom_intensity;
                if ui.add(egui::Slider::new(&mut bloom, 0.0..=2.0)).changed() {
                    world.resources.graphics.bloom_intensity = bloom;
                }
            });

        match action {
            UiAction::AddNew => {
                let text = self.text_input.clone();
                let color = NEON_COLORS[self.selected_color_index].0;
                let row_height = (CHAR_HEIGHT * SIGN_SCALE) + 1.0;
                let y_pos = 2.0 + self.signs.len() as f32 * row_height;
                self.spawn_sign(world, &text, Vec3::new(0.0, y_pos, -5.0), color);
                self.editing_index = Some(self.signs.len() - 1);
            }
            UiAction::RemoveSelected => {
                if let Some(sign_index) = self.editing_index {
                    self.destroy_sign(world, sign_index);
                    self.signs.remove(sign_index);
                    if self.signs.is_empty() {
                        self.editing_index = None;
                    } else {
                        self.editing_index = Some(sign_index.min(self.signs.len() - 1));
                        if let Some(index) = self.editing_index {
                            self.text_input = self.signs[index].text.clone();
                        }
                    }
                }
            }
            UiAction::ClearAll => {
                self.clear_all_signs(world);
                self.editing_index = None;
            }
            UiAction::None => {}
        }
    }

}

enum UiAction {
    None,
    AddNew,
    RemoveSelected,
    ClearAll,
}

impl NeonLightsDemo {
    fn spawn_showcase(&mut self, world: &mut World) {
        let chars: Vec<char> = ALL_CHARS.chars().collect();
        let row_height = (CHAR_HEIGHT * SIGN_SCALE) + 1.5;
        let total_rows = chars.len().div_ceil(CHARS_PER_ROW);
        let top_y = (total_rows as f32 * row_height) / 2.0 + 3.0;

        let row_colors = [
            NEON_COLORS[0].0,
            NEON_COLORS[1].0,
            NEON_COLORS[2].0,
            NEON_COLORS[4].0,
            NEON_COLORS[6].0,
        ];

        for row_index in 0..total_rows {
            let start = row_index * CHARS_PER_ROW;
            let end = (start + CHARS_PER_ROW).min(chars.len());
            let row_text: String = chars[start..end].iter().collect();
            let y_pos = top_y - row_index as f32 * row_height;
            let color = row_colors[row_index % row_colors.len()];

            self.spawn_sign_individual_chars(world, &row_text, Vec3::new(0.0, y_pos, -5.0), color);
        }
    }

    fn spawn_sign_individual_chars(
        &mut self,
        world: &mut World,
        text: &str,
        center_position: Vec3,
        color: Vec3,
    ) {
        let char_total_width = (CHAR_WIDTH + CHAR_SPACING) * SIGN_SCALE;
        let text_width = text.len() as f32 * char_total_width - CHAR_SPACING * SIGN_SCALE;
        let start_x = center_position.x - text_width / 2.0;
        let base_y = center_position.y;
        let z_pos = center_position.z;

        let mut tube_entities = Vec::new();
        let mut light_entities = Vec::new();

        for (char_index, character) in text.chars().enumerate() {
            let cursor_x = start_x + char_index as f32 * char_total_width;

            let strokes = stroke_font::get_character_strokes(character);
            if strokes.is_empty() {
                continue;
            }

            let polylines_3d: Vec<Vec<Vec3>> = strokes
                .iter()
                .map(|polyline| {
                    polyline
                        .iter()
                        .map(|point| {
                            Vec3::new(
                                cursor_x + point.x * SIGN_SCALE,
                                base_y + point.y * SIGN_SCALE,
                                z_pos,
                            )
                        })
                        .collect()
                })
                .collect();

            let unique_id = next_unique_id();
            let mesh_name = format!("neon_tube_{unique_id}");
            let mesh = tube_mesh::build_neon_tube_mesh(&polylines_3d, TUBE_RADIUS);
            mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.clone(), mesh);

            let entity = spawn_neon_tube_entity(world, &mesh_name, color);
            tube_entities.push(entity);

            let char_center_x = cursor_x + CHAR_WIDTH * SIGN_SCALE * 0.5;
            let char_center_y = base_y + CHAR_HEIGHT * SIGN_SCALE * 0.5;
            let light = spawn_point_light(
                world,
                Vec3::new(char_center_x, char_center_y, z_pos + 0.5),
                color,
                1.5,
                6.0,
            );
            light_entities.push(light);

            self.flicker_phases
                .push(char_index as f32 * 1.7 + self.signs.len() as f32 * 3.1);
        }

        self.signs.push(NeonSign {
            text: text.to_string(),
            tube_entities,
            light_entities,
            color,
            position: center_position,
        });
    }

    fn spawn_sign(&mut self, world: &mut World, text: &str, center_position: Vec3, color: Vec3) {
        let sign = build_sign(world, text, center_position, color);
        let char_count = sign.light_entities.len();
        let sign_index = self.signs.len();
        self.signs.push(sign);
        for char_index in 0..char_count {
            self.flicker_phases
                .push(char_index as f32 * 1.7 + sign_index as f32 * 3.1);
        }
    }

    fn destroy_sign(&mut self, world: &mut World, sign_index: usize) {
        let sign = &self.signs[sign_index];
        for &entity in &sign.tube_entities {
            despawn_recursive_immediate(world, entity);
        }
        for &entity in &sign.light_entities {
            despawn_recursive_immediate(world, entity);
        }
    }

    fn clear_all_signs(&mut self, world: &mut World) {
        for sign_index in (0..self.signs.len()).rev() {
            self.destroy_sign(world, sign_index);
        }
        self.signs.clear();
        self.flicker_phases.clear();
    }

    fn update_flicker(&mut self, world: &mut World) {
        let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

        let mut phase_index = 0;
        for sign in &self.signs {
            for &light_entity in &sign.light_entities {
                if phase_index >= self.flicker_phases.len() {
                    break;
                }
                let phase = self.flicker_phases[phase_index];
                let flicker = compute_flicker(time, phase);

                if let Some(light) = world.get_light_mut(light_entity) {
                    light.intensity = 1.5 * flicker;
                }

                phase_index += 1;
            }
        }
    }
}

fn build_sign(world: &mut World, text: &str, center_position: Vec3, color: Vec3) -> NeonSign {
    let text_width = stroke_font::measure_text(text) * SIGN_SCALE;
    let start_x = center_position.x - text_width / 2.0;
    let base_y = center_position.y;
    let z_pos = center_position.z;

    let mut tube_entities = Vec::new();
    let mut light_entities = Vec::new();
    let mut cursor_x = start_x;

    for character in text.chars() {
        if character == ' ' {
            cursor_x += WORD_SPACING * SIGN_SCALE;
            continue;
        }

        let strokes = stroke_font::get_character_strokes(character);
        if strokes.is_empty() {
            cursor_x += CHAR_WIDTH * SIGN_SCALE + CHAR_SPACING * SIGN_SCALE;
            continue;
        }

        let polylines_3d: Vec<Vec<Vec3>> = strokes
            .iter()
            .map(|polyline| {
                polyline
                    .iter()
                    .map(|point| {
                        Vec3::new(
                            cursor_x + point.x * SIGN_SCALE,
                            base_y + point.y * SIGN_SCALE,
                            z_pos,
                        )
                    })
                    .collect()
            })
            .collect();

        let unique_id = next_unique_id();
        let mesh_name = format!("neon_tube_{unique_id}");
        let mesh = tube_mesh::build_neon_tube_mesh(&polylines_3d, TUBE_RADIUS);
        mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.clone(), mesh);

        let entity = spawn_neon_tube_entity(world, &mesh_name, color);
        tube_entities.push(entity);

        let char_center_x = cursor_x + CHAR_WIDTH * SIGN_SCALE * 0.5;
        let char_center_y = base_y + CHAR_HEIGHT * SIGN_SCALE * 0.5;
        let light = spawn_point_light(
            world,
            Vec3::new(char_center_x, char_center_y, z_pos + 0.5),
            color,
            2.0,
            8.0,
        );
        light_entities.push(light);

        cursor_x += CHAR_WIDTH * SIGN_SCALE + CHAR_SPACING * SIGN_SCALE;
    }

    NeonSign {
        text: text.to_string(),
        tube_entities,
        light_entities,
        color,
        position: center_position,
    }
}

fn compute_flicker(time: f32, phase: f32) -> f32 {
    let slow_wave = ((time * 0.5 + phase).sin() * 0.5 + 0.5).powf(0.3);
    let fast_flicker = ((time * 12.0 + phase * 7.3).sin() * 0.5 + 0.5).powf(8.0);
    let base = 0.85 + slow_wave * 0.15;
    let flicker_amount = fast_flicker * 0.3;
    (base - flicker_amount).clamp(0.4, 1.0)
}

fn spawn_neon_tube_entity(world: &mut World, mesh_name: &str, color: Vec3) -> Entity {
    let entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::RENDER_MESH
            | nightshade::ecs::world::MATERIAL_REF
            | nightshade::ecs::world::VISIBILITY,
        1,
    )[0];

    world.set_name(entity, Name(mesh_name.to_string()));
    world.set_local_transform(
        entity,
        LocalTransform {
            translation: Vec3::zeros(),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.set_local_transform_dirty(entity, LocalTransformDirty);
    world.set_global_transform(entity, GlobalTransform::default());
    world.set_render_mesh(entity, RenderMesh::new(mesh_name));

    let unique_id = next_unique_id();
    let material_name = format!("NeonMaterial_{unique_id}");
    let material = Material {
        base_color: [color.x * 0.3, color.y * 0.3, color.z * 0.3, 1.0],
        emissive_factor: [color.x, color.y, color.z],
        emissive_strength: 8.0,
        unlit: true,
        roughness: 0.1,
        metallic: 0.0,
        ..Default::default()
    };

    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        material,
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(entity, MaterialRef::new(material_name));

    entity
}

fn spawn_point_light(
    world: &mut World,
    position: Vec3,
    color: Vec3,
    intensity: f32,
    range: f32,
) -> Entity {
    let entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::LIGHT,
        1,
    )[0];

    world.set_name(entity, Name("Neon Light".to_string()));
    world.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.set_local_transform_dirty(entity, LocalTransformDirty);
    world.set_global_transform(entity, GlobalTransform::default());
    world.set_light(
        entity,
        Light {
            light_type: LightType::Point,
            color,
            intensity,
            range,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: false,
            shadow_bias: 0.007,
        },
    );

    entity
}

fn spawn_backdrop(world: &mut World) {
    let wall_material = Material {
        base_color: [0.05, 0.03, 0.07, 1.0],
        roughness: 0.9,
        metallic: 0.0,
        ..Default::default()
    };
    spawn_wall(
        world,
        "Back Wall",
        Vec3::new(0.0, 5.0, -5.5),
        Vec3::new(40.0, 30.0, 0.2),
        wall_material,
    );

    let floor_material = Material {
        base_color: [0.03, 0.02, 0.05, 1.0],
        roughness: 0.4,
        metallic: 0.3,
        ..Default::default()
    };
    spawn_wall(
        world,
        "Floor",
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(40.0, 0.2, 30.0),
        floor_material,
    );
}

fn spawn_wall(
    world: &mut World,
    name: &str,
    position: Vec3,
    scale: Vec3,
    material: Material,
) -> Entity {
    let entity = spawn_mesh(world, "Cube", position, scale);
    let unique_id = next_unique_id();
    let material_name = format!("WallMaterial_{unique_id}");
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        material,
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(entity, MaterialRef::new(material_name));
    world.set_name(entity, Name(name.to_string()));
    entity
}

fn spawn_camera(world: &mut World, position: Vec3) -> Entity {
    let cameras = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::CAMERA,
        1,
    );

    let camera = cameras[0];

    world.set_name(camera, Name("Main Camera".to_string()));

    if let Some(local_transform) = world.get_local_transform_mut(camera) {
        local_transform.translation = position;
    }

    if let Some(camera_component) = world.get_camera_mut(camera) {
        *camera_component = Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 50.0_f32.to_radians(),
                z_far: Some(200.0),
                z_near: 0.01,
            }),
            smoothing: Some(Smoothing::default()),
        };
    }

    camera
}
