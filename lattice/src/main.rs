use nalgebra_glm::{Vec3, vec2, vec3};
use nightshade::{
    ecs::{
        camera::{commands::spawn_pan_orbit_camera, systems::pan_orbit_camera_system},
        material::resources::material_registry_insert,
        mesh::components::{
            create_cube_mesh, create_cylinder_mesh, create_sphere_mesh, create_torus_mesh,
        },
        prefab::resources::mesh_cache_insert,
    },
    prelude::*,
    render::wgpu::rendergraph::RenderGraph,
    run::RenderResources,
};

const HDR_BYTES: &[u8] = include_bytes!("../../assets/sky/moonrise.hdr");
const HELMET_GLTF: &[u8] = include_bytes!("../../assets/gltf/DamagedHelmet.glb");
const PUDDING_GLTF: &[u8] = include_bytes!("../../assets/gltf/pudding.glb");
const JELLY_HELMET_POSITION: Vec3 = Vec3::new(0.0, 2.5, 0.0);
const JELLY_PUDDING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const JELLY_PUDDING_SCALE: f32 = 0.5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(LatticeState::default())
}

struct Primitive {
    entity: Entity,
    start_z: f32,
    speed: f32,
}

struct LatticeState {
    lattice_entity: Option<Entity>,
    primitives: Vec<Primitive>,
    helmet_entity: Option<Entity>,
    camera_entity: Option<Entity>,
    debug_state: LatticeDebugState,
    animation_speed: f32,
    show_lattice: bool,
    paused: bool,
    edit_mode: bool,
    selected_point: Option<(usize, usize, usize)>,
    is_dragging: bool,
    drag_start_displacement: Vec3,
    jelly_mode: bool,
    jelly_lattice_entity: Option<Entity>,
    jelly_helmet_entity: Option<Entity>,
    plate_entity: Option<Entity>,
    jelly_time: f32,
    use_pudding_model: bool,
}

impl Default for LatticeState {
    fn default() -> Self {
        Self {
            lattice_entity: None,
            primitives: Vec::new(),
            helmet_entity: None,
            camera_entity: None,
            debug_state: LatticeDebugState::default(),
            animation_speed: 1.0,
            show_lattice: true,
            paused: false,
            edit_mode: false,
            selected_point: None,
            is_dragging: false,
            drag_start_displacement: Vec3::zeros(),
            jelly_mode: false,
            jelly_lattice_entity: None,
            jelly_helmet_entity: None,
            plate_entity: None,
            jelly_time: 0.0,
            use_pudding_model: false,
        }
    }
}

impl State for LatticeState {
    fn title(&self) -> &str {
        "Lattice Deformation"
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.08);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .write("output", resources.swapchain);
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Hdr;
        world.resources.graphics.use_fullscreen = true;

        load_hdr_skybox(world, HDR_BYTES.to_vec());

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
        }

        self.animation_speed = 1.0;
        self.show_lattice = true;

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 2.0, 0.0),
            8.0,
            0.3,
            0.4,
            "Main Camera".to_string(),
        );

        self.camera_entity = Some(camera_entity);
        world.resources.active_camera = Some(camera_entity);

        self.setup_lattice(world);
        self.spawn_primitives(world);
        self.spawn_helmet(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if !self.edit_mode || !self.is_dragging {
            pan_orbit_camera_system(world);
        }

        if self.edit_mode && !self.jelly_mode {
            self.handle_lattice_editing(world);
        }

        if !self.paused {
            if self.jelly_mode {
                self.animate_jelly(world);
            } else {
                self.animate_primitives(world);
            }
        }

        lattice_deformation_system(world);

        let active_lattice = if self.jelly_mode {
            self.jelly_lattice_entity
        } else {
            self.lattice_entity
        };

        if let Some(lattice_entity) = active_lattice {
            self.debug_state.enabled = self.show_lattice;
            self.debug_state.selected_point = if self.jelly_mode {
                None
            } else {
                self.selected_point
            };
            lattice_debug_draw(world, lattice_entity, &mut self.debug_state);
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Lattice Deformation")
            .default_pos(egui::pos2(10.0, 10.0))
            .default_width(280.0)
            .show(ui_context, |ui| {
                let jelly_changed = ui.checkbox(&mut self.jelly_mode, "Jelly Mode").changed();
                if jelly_changed {
                    self.toggle_jelly_mode(world);
                }

                if self.jelly_mode {
                    let pudding_changed = ui
                        .checkbox(&mut self.use_pudding_model, "Use Pudding Model")
                        .changed();
                    if pudding_changed {
                        self.swap_jelly_model(world);
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Animation Speed:");
                    ui.add(
                        egui::Slider::new(&mut self.animation_speed, 0.0..=3.0).fixed_decimals(2),
                    );
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.show_lattice, "Show Lattice");
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.paused, "Paused");
                });

                if !self.jelly_mode {
                    ui.separator();

                    let edit_label = if self.edit_mode {
                        "Edit Mode: ON"
                    } else {
                        "Edit Mode: OFF"
                    };
                    if ui.checkbox(&mut self.edit_mode, edit_label).changed() && !self.edit_mode {
                        self.selected_point = None;
                        self.is_dragging = false;
                    }

                    if self.edit_mode {
                        if let Some((x, y, z)) = self.selected_point {
                            ui.label(format!("Selected: ({}, {}, {})", x, y, z));

                            if let Some(lattice_entity) = self.lattice_entity
                                && let Some(lattice) = world.get_lattice(lattice_entity)
                            {
                                let disp = lattice.get_displacement(x, y, z);
                                ui.label(format!(
                                    "Displacement: ({:.2}, {:.2}, {:.2})",
                                    disp.x, disp.y, disp.z
                                ));
                            }
                        } else {
                            ui.label("Click a lattice point to select");
                        }
                    }

                    ui.separator();

                    if ui.button("Reset Lattice").clicked()
                        && let Some(lattice_entity) = self.lattice_entity
                        && let Some(lattice) = world.get_lattice_mut(lattice_entity)
                    {
                        lattice.reset_displacements();
                        self.selected_point = None;
                    }

                    if ui.button("Apply Tube Pinch").clicked() {
                        self.apply_tube_pinch(world);
                    }

                    if ui.button("Spawn More Primitives").clicked() {
                        self.spawn_primitives(world);
                    }
                }

                ui.separator();

                ui.label("Controls:");
                if self.edit_mode && !self.jelly_mode {
                    ui.label("- Click: Select lattice point");
                    ui.label("- Drag: Move selected point");
                } else {
                    ui.label("- Mouse drag: Orbit camera");
                }
                ui.label("- Scroll: Zoom");
                ui.label("- ESC: Exit");
            });
    }
}

impl LatticeState {
    fn handle_lattice_editing(&mut self, world: &mut World) {
        if world.resources.user_interface.consumed_event {
            return;
        }

        let Some(lattice_entity) = self.lattice_entity else {
            return;
        };

        let mouse = &world.resources.input.mouse;
        let mouse_pos = mouse.position;
        let mouse_delta = mouse.position_delta;
        let left_clicked = mouse.state.contains(MouseState::LEFT_CLICKED);
        let left_just_pressed = mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
        let left_just_released = mouse.state.contains(MouseState::LEFT_JUST_RELEASED);

        let Some((width, height)) = world.resources.window.cached_viewport_size else {
            return;
        };
        let screen_size = vec2(width as f32, height as f32);

        if left_just_pressed {
            let picked =
                get_lattice_point_at_screen_position(world, lattice_entity, mouse_pos, screen_size);

            if let Some((x, y, z)) = picked {
                self.selected_point = Some((x, y, z));
                self.is_dragging = true;

                if let Some(lattice) = world.get_lattice(lattice_entity) {
                    self.drag_start_displacement = lattice.get_displacement(x, y, z);
                }
            } else {
                self.selected_point = None;
                self.is_dragging = false;
            }
        } else if left_clicked && self.is_dragging {
            if let Some((x, y, z)) = self.selected_point {
                let camera_vectors = self.get_camera_vectors(world);

                if let Some((camera_right, camera_up)) = camera_vectors {
                    let sensitivity = 0.01;
                    let world_delta = camera_right * mouse_delta.x * sensitivity
                        - camera_up * mouse_delta.y * sensitivity;

                    if let Some(lattice) = world.get_lattice_mut(lattice_entity) {
                        let current = lattice.get_displacement(x, y, z);
                        lattice.set_displacement(x, y, z, current + world_delta);
                    }
                }
            }
        } else if left_just_released {
            self.is_dragging = false;
        }
    }

    fn get_camera_vectors(&self, world: &World) -> Option<(Vec3, Vec3)> {
        let camera_entity = world.resources.active_camera?;
        let camera_transform = world.get_global_transform(camera_entity)?;

        let right = vec3(
            camera_transform.0[(0, 0)],
            camera_transform.0[(1, 0)],
            camera_transform.0[(2, 0)],
        )
        .normalize();

        let up = vec3(
            camera_transform.0[(0, 1)],
            camera_transform.0[(1, 1)],
            camera_transform.0[(2, 1)],
        )
        .normalize();

        Some((right, up))
    }

    fn setup_lattice(&mut self, world: &mut World) {
        let bounds_min = vec3(-2.0, -2.0, -4.0);
        let bounds_max = vec3(2.0, 2.0, 4.0);
        let dimensions = [4, 4, 8];

        let lattice_entity = create_lattice_entity(world, bounds_min, bounds_max, dimensions);
        self.lattice_entity = Some(lattice_entity);

        self.apply_tube_pinch(world);
    }

    fn apply_tube_pinch(&mut self, world: &mut World) {
        let Some(lattice_entity) = self.lattice_entity else {
            return;
        };

        let Some(lattice) = world.get_lattice_mut(lattice_entity) else {
            return;
        };

        let [nx, ny, nz] = lattice.dimensions;

        for z in 0..nz {
            let z_normalized = z as f32 / (nz - 1) as f32;
            let pinch_strength = if z_normalized > 0.2 && z_normalized < 0.8 {
                let center_dist = (z_normalized - 0.5).abs();
                let falloff = 1.0 - (center_dist / 0.3).min(1.0);
                falloff * 0.9
            } else {
                0.0
            };

            for y in 0..ny {
                for x in 0..nx {
                    let base_pos = lattice.base_points[lattice.get_index(x, y, z)];

                    let to_center = vec3(-base_pos.x, -base_pos.y, 0.0);
                    let displacement = to_center * pinch_strength;

                    lattice.set_displacement(x, y, z, displacement);
                }
            }
        }
    }

    fn spawn_primitives(&mut self, world: &mut World) {
        let Some(lattice_entity) = self.lattice_entity else {
            return;
        };

        let primitive_count = 5;
        let colors = [
            [1.0, 0.3, 0.3, 1.0],
            [0.3, 1.0, 0.3, 1.0],
            [0.3, 0.3, 1.0, 1.0],
            [1.0, 1.0, 0.3, 1.0],
            [1.0, 0.3, 1.0, 1.0],
        ];

        for index in 0..primitive_count {
            let mesh_type = index % 3;
            let mesh_name = format!("lattice_primitive_{}", self.primitives.len());

            let mesh = match mesh_type {
                0 => create_sphere_mesh(0.3, 16),
                1 => create_cube_mesh(),
                _ => create_torus_mesh(0.25, 0.1, 16, 8),
            };

            mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.clone(), mesh);

            let entity = world.spawn_entities(
                LOCAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | GLOBAL_TRANSFORM
                    | RENDER_MESH
                    | MATERIAL_REF
                    | CASTS_SHADOW,
                1,
            )[0];

            let x_offset = (index as f32 - 2.0) * 0.8;
            let start_z = -6.0 - (index as f32 * 0.5);

            world.set_local_transform(
                entity,
                LocalTransform {
                    translation: vec3(x_offset, 0.0, start_z),
                    rotation: Quat::identity(),
                    scale: vec3(1.0, 1.0, 1.0),
                },
            );

            world.set_render_mesh(entity, RenderMesh::new(mesh_name.clone()));

            let material_name = format!("lattice_material_{}", self.primitives.len());
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: colors[index % colors.len()],
                    roughness: 0.4,
                    metallic: 0.1,
                    ..Default::default()
                },
            );

            if let Some(&mat_index) = world
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
                    .add_reference(mat_index);
            }

            world.set_material_ref(entity, MaterialRef::new(material_name));
            world.set_casts_shadow(entity, CastsShadow);

            register_entity_for_lattice_deformation(world, entity, lattice_entity);

            self.primitives.push(Primitive {
                entity,
                start_z,
                speed: 0.8 + (index as f32 * 0.1),
            });
        }
    }

    fn animate_primitives(&mut self, world: &mut World) {
        let dt = world.resources.window.timing.delta_time;

        for primitive in &self.primitives {
            if let Some(transform) = world.get_local_transform_mut(primitive.entity) {
                transform.translation.z += primitive.speed * self.animation_speed * dt;

                if transform.translation.z > 6.0 {
                    transform.translation.z = primitive.start_z;
                }
            }
            world.mark_local_transform_dirty(primitive.entity);
        }

        if let Some(helmet_entity) = self.helmet_entity {
            if let Some(transform) = world.get_local_transform_mut(helmet_entity) {
                transform.translation.z += 0.5 * self.animation_speed * dt;

                if transform.translation.z > 6.0 {
                    transform.translation.z = -6.0;
                }
            }
            world.mark_local_transform_dirty(helmet_entity);
        }
    }

    fn spawn_helmet(&mut self, world: &mut World) {
        let Some(lattice_entity) = self.lattice_entity else {
            return;
        };

        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(HELMET_GLTF);

        match load_result {
            Ok(result) => {
                for (name, (rgba_data, width, height)) in result.textures {
                    world.queue_command(WorldCommand::LoadTexture {
                        name,
                        rgba_data,
                        width,
                        height,
                    });
                }

                for (name, mesh) in result.meshes {
                    mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                }

                for prefab in result.prefabs {
                    let entity =
                        nightshade::ecs::prefab::spawn_prefab(world, &prefab, vec3(0.0, 0.0, -6.0));

                    self.helmet_entity = Some(entity);

                    register_helmet_meshes(world, entity, lattice_entity);
                }
            }
            Err(_e) => {}
        }
    }

    fn toggle_jelly_mode(&mut self, world: &mut World) {
        if self.jelly_mode {
            self.enter_jelly_mode(world);
        } else {
            self.exit_jelly_mode(world);
        }
    }

    fn enter_jelly_mode(&mut self, world: &mut World) {
        self.set_tube_mode_visibility(world, false);
        self.show_lattice = false;

        let bounds_min = vec3(-1.5, 1.0, -1.5);
        let bounds_max = vec3(1.5, 4.0, 1.5);
        let dimensions = [4, 5, 4];
        let jelly_lattice = create_lattice_entity(world, bounds_min, bounds_max, dimensions);
        self.jelly_lattice_entity = Some(jelly_lattice);

        self.spawn_plate(world);
        self.spawn_jelly_helmet(world);

        self.jelly_time = 0.0;
    }

    fn exit_jelly_mode(&mut self, world: &mut World) {
        if let Some(entity) = self.jelly_helmet_entity.take() {
            despawn_recursive_immediate(world, entity);
        }
        if let Some(entity) = self.plate_entity.take() {
            despawn_recursive_immediate(world, entity);
        }
        if let Some(entity) = self.jelly_lattice_entity.take() {
            despawn_recursive_immediate(world, entity);
        }

        self.set_tube_mode_visibility(world, true);
        self.show_lattice = true;
    }

    fn set_tube_mode_visibility(&mut self, world: &mut World, visible: bool) {
        for primitive in &self.primitives {
            world.set_visibility(primitive.entity, Visibility { visible });
        }

        if let Some(helmet_entity) = self.helmet_entity {
            set_visibility_recursive(world, helmet_entity, visible);
        }
    }

    fn spawn_plate(&mut self, world: &mut World) {
        let plate_mesh = create_cylinder_mesh(1.8, 0.08, 32);
        mesh_cache_insert(
            &mut world.resources.mesh_cache,
            "dinner_plate".to_string(),
            plate_mesh,
        );

        let entity = world.spawn_entities(
            LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | CASTS_SHADOW,
            1,
        )[0];

        world.set_local_transform(
            entity,
            LocalTransform {
                translation: vec3(0.0, 1.04, 0.0),
                rotation: Quat::identity(),
                scale: vec3(1.0, 1.0, 1.0),
            },
        );

        world.set_render_mesh(entity, RenderMesh::new("dinner_plate"));

        material_registry_insert(
            &mut world.resources.material_registry,
            "plate_material".to_string(),
            Material {
                base_color: [0.95, 0.95, 0.92, 1.0],
                roughness: 0.3,
                metallic: 0.0,
                ..Default::default()
            },
        );

        if let Some(&mat_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get("plate_material")
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(mat_index);
        }

        world.set_material_ref(entity, MaterialRef::new("plate_material"));
        world.set_casts_shadow(entity, CastsShadow);

        self.plate_entity = Some(entity);
    }

    fn spawn_jelly_helmet(&mut self, world: &mut World) {
        let Some(jelly_lattice) = self.jelly_lattice_entity else {
            return;
        };

        let (gltf_bytes, position, scale) = if self.use_pudding_model {
            (PUDDING_GLTF, JELLY_PUDDING_POSITION, JELLY_PUDDING_SCALE)
        } else {
            (HELMET_GLTF, JELLY_HELMET_POSITION, 1.0)
        };

        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(gltf_bytes);

        if let Ok(result) = load_result {
            for (name, (rgba_data, width, height)) in result.textures {
                world.queue_command(WorldCommand::LoadTexture {
                    name,
                    rgba_data,
                    width,
                    height,
                });
            }

            for (name, mesh) in result.meshes {
                mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
            }

            for prefab in result.prefabs {
                let entity = nightshade::ecs::prefab::spawn_prefab(world, &prefab, position);

                if let Some(transform) = world.get_local_transform_mut(entity) {
                    transform.scale = vec3(scale, scale, scale);
                }
                world.mark_local_transform_dirty(entity);

                self.jelly_helmet_entity = Some(entity);

                register_helmet_meshes(world, entity, jelly_lattice);
            }
        }
    }

    fn swap_jelly_model(&mut self, world: &mut World) {
        if let Some(entity) = self.jelly_helmet_entity.take() {
            despawn_recursive_immediate(world, entity);
        }
        self.spawn_jelly_helmet(world);
    }

    fn animate_jelly(&mut self, world: &mut World) {
        let dt = world.resources.window.timing.delta_time;
        self.jelly_time += dt * self.animation_speed * 2.0;

        if self.use_pudding_model {
            self.animate_pudding_shake(world);
        } else {
            self.animate_helmet_jelly(world);
        }
    }

    fn animate_pudding_shake(&mut self, world: &mut World) {
        let shake_freq = 2.5;
        let shake_amplitude = 0.4;

        let plate_x = (self.jelly_time * shake_freq).sin() * shake_amplitude;
        let plate_z = (self.jelly_time * shake_freq * 0.7 + 1.0).sin() * shake_amplitude * 0.6;

        let velocity_x = (self.jelly_time * shake_freq).cos() * shake_amplitude * shake_freq;
        let velocity_z =
            (self.jelly_time * shake_freq * 0.7 + 1.0).cos() * shake_amplitude * 0.6 * shake_freq;

        if let Some(plate_entity) = self.plate_entity {
            if let Some(transform) = world.get_local_transform_mut(plate_entity) {
                transform.translation.x = plate_x;
                transform.translation.z = plate_z;
            }
            world.mark_local_transform_dirty(plate_entity);
        }

        if let Some(pudding_entity) = self.jelly_helmet_entity {
            if let Some(transform) = world.get_local_transform_mut(pudding_entity) {
                transform.translation.x = plate_x;
                transform.translation.z = plate_z;
            }
            world.mark_local_transform_dirty(pudding_entity);
        }

        let Some(jelly_lattice) = self.jelly_lattice_entity else {
            return;
        };

        if let Some(lattice) = world.get_lattice_mut(jelly_lattice) {
            lattice.bounds_min.x = -1.5 + plate_x;
            lattice.bounds_max.x = 1.5 + plate_x;
            lattice.bounds_min.z = -1.5 + plate_z;
            lattice.bounds_max.z = 1.5 + plate_z;
        }

        let Some(lattice) = world.get_lattice_mut(jelly_lattice) else {
            return;
        };

        let [nx, ny, nz] = lattice.dimensions;

        let inertia_strength = 0.35;

        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let height_factor = y as f32 / (ny - 1) as f32;
                    let height_squared = height_factor * height_factor;

                    let inertia_x = -velocity_x * inertia_strength * height_squared;
                    let inertia_z = -velocity_z * inertia_strength * height_squared;

                    let phase_x = x as f32 * 0.8 + z as f32 * 0.5;
                    let phase_z = z as f32 * 0.8 + x as f32 * 0.5;

                    let secondary_freq = 10.0;
                    let secondary_amp = 0.08 * height_factor;
                    let secondary_x =
                        (self.jelly_time * secondary_freq + phase_x).sin() * secondary_amp;
                    let secondary_z =
                        (self.jelly_time * secondary_freq * 1.1 + phase_z).sin() * secondary_amp;

                    let tertiary_freq = 15.0;
                    let tertiary_amp = 0.04 * height_factor;
                    let tertiary_x =
                        (self.jelly_time * tertiary_freq + phase_x * 1.5).sin() * tertiary_amp;
                    let tertiary_z = (self.jelly_time * tertiary_freq * 0.9 + phase_z * 1.5).sin()
                        * tertiary_amp;

                    let squash = -velocity_x.abs() * 0.05 * height_factor
                        - velocity_z.abs() * 0.05 * height_factor;

                    let base_pos = lattice.base_points[lattice.get_index(x, y, z)];
                    let radial_dir = vec3(base_pos.x, 0.0, base_pos.z);
                    let radial_dist = radial_dir.magnitude();
                    let bulge = if radial_dist > 0.01 {
                        let accel_magnitude =
                            (velocity_x * velocity_x + velocity_z * velocity_z).sqrt();
                        radial_dir.normalize() * accel_magnitude * 0.04 * height_factor
                    } else {
                        Vec3::zeros()
                    };

                    let displacement = vec3(
                        inertia_x + secondary_x + tertiary_x + bulge.x,
                        squash + (secondary_x.abs() + tertiary_x.abs()) * 0.5,
                        inertia_z + secondary_z + tertiary_z + bulge.z,
                    );

                    lattice.set_displacement(x, y, z, displacement);
                }
            }
        }
    }

    fn animate_helmet_jelly(&mut self, world: &mut World) {
        let Some(jelly_lattice) = self.jelly_lattice_entity else {
            return;
        };

        let Some(lattice) = world.get_lattice_mut(jelly_lattice) else {
            return;
        };

        let [nx, ny, nz] = lattice.dimensions;

        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let base_pos = lattice.base_points[lattice.get_index(x, y, z)];

                    let phase_x = x as f32 * 1.3 + z as f32 * 0.7;
                    let phase_y = y as f32 * 0.9 + x as f32 * 1.1;
                    let phase_z = z as f32 * 1.5 + y as f32 * 0.6;

                    let freq1 = 3.5;
                    let freq2 = 5.2;
                    let freq3 = 2.8;

                    let height_factor = (y as f32 / (ny - 1) as f32).powf(0.5);
                    let amplitude = 0.15 * height_factor;

                    let edge_x = 1.0 - ((x as f32 / (nx - 1) as f32) - 0.5).abs() * 2.0;
                    let edge_z = 1.0 - ((z as f32 / (nz - 1) as f32) - 0.5).abs() * 2.0;
                    let edge_factor = (edge_x * edge_z).max(0.3);

                    let wiggle_x = (self.jelly_time * freq1 + phase_x).sin()
                        + 0.5 * (self.jelly_time * freq2 * 1.3 + phase_x * 2.0).sin();
                    let wiggle_y = (self.jelly_time * freq2 + phase_y).sin()
                        + 0.3 * (self.jelly_time * freq3 * 1.5 + phase_y * 1.5).sin();
                    let wiggle_z = (self.jelly_time * freq3 + phase_z).sin()
                        + 0.5 * (self.jelly_time * freq1 * 0.8 + phase_z * 2.0).sin();

                    let displacement = vec3(
                        wiggle_x * amplitude * edge_factor,
                        wiggle_y * amplitude * 0.5,
                        wiggle_z * amplitude * edge_factor,
                    );

                    let radial_dir = vec3(base_pos.x, 0.0, base_pos.z);
                    let radial_dist = radial_dir.magnitude();
                    if radial_dist > 0.01 {
                        let bulge =
                            (self.jelly_time * 4.0 + y as f32 * 0.5).sin() * 0.08 * height_factor;
                        let radial_displacement = radial_dir.normalize() * bulge;
                        lattice.set_displacement(x, y, z, displacement + radial_displacement);
                    } else {
                        lattice.set_displacement(x, y, z, displacement);
                    }
                }
            }
        }
    }
}

fn register_helmet_meshes(world: &mut World, entity: Entity, lattice_entity: Entity) {
    if world.get_render_mesh(entity).is_some() {
        register_entity_for_lattice_deformation(world, entity, lattice_entity);
    }

    let children: Vec<Entity> = world
        .query_entities(PARENT)
        .filter(|e| {
            world
                .get_parent(*e)
                .map(|p| p.0 == Some(entity))
                .unwrap_or(false)
        })
        .collect();

    for child in children {
        register_helmet_meshes(world, child, lattice_entity);
    }
}

fn set_visibility_recursive(world: &mut World, entity: Entity, visible: bool) {
    world.set_visibility(entity, Visibility { visible });

    let children: Vec<Entity> = world
        .query_entities(PARENT)
        .filter(|e| {
            world
                .get_parent(*e)
                .map(|p| p.0 == Some(entity))
                .unwrap_or(false)
        })
        .collect();

    for child in children {
        set_visibility_recursive(world, child, visible);
    }
}
