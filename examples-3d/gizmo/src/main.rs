use nightshade::ecs::gizmos::{self, GizmoDragMode, GizmoMode, GizmoState};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

struct GizmoDemo {
    gizmo: Option<GizmoState>,
    selectable_entities: Vec<Entity>,
    selected_entity: Option<Entity>,
    current_mode: GizmoMode,
    hover_axis: Option<Vec3>,
    drag_mode: GizmoDragMode,
}

impl Default for GizmoDemo {
    fn default() -> Self {
        Self {
            gizmo: None,
            selectable_entities: Vec::new(),
            selected_entity: None,
            current_mode: GizmoMode::LocalTranslation,
            hover_axis: None,
            drag_mode: GizmoDragMode::None,
        }
    }
}

impl GizmoDemo {
    fn select_entity(&mut self, world: &mut World, entity: Entity) {
        if self.selected_entity == Some(entity) {
            return;
        }

        if let Some(old_gizmo) = &self.gizmo {
            gizmos::destroy_gizmo(world, old_gizmo);
        }

        self.selected_entity = Some(entity);
        world.resources.graphics.bounding_volume_selected_entity = Some(entity);
        self.hover_axis = None;
        self.drag_mode = GizmoDragMode::None;

        let new_gizmo = match self.current_mode {
            GizmoMode::LocalTranslation | GizmoMode::GlobalTranslation => {
                gizmos::create_translation_gizmo(world, entity, self.current_mode)
            }
            GizmoMode::Rotation => gizmos::create_rotation_gizmo(world, entity),
            GizmoMode::Scale => gizmos::create_scale_gizmo(world, entity),
        };

        if let Some(camera_entity) = world.resources.active_camera {
            if let (Some(camera_transform), Some(target_global_transform)) = (
                world.core.get_global_transform(camera_entity),
                world.core.get_global_transform(entity),
            ) {
                let camera_pos = nalgebra_glm::vec3(
                    camera_transform.0[(0, 3)],
                    camera_transform.0[(1, 3)],
                    camera_transform.0[(2, 3)],
                );
                let target_pos = target_global_transform.translation();
                let distance = nalgebra_glm::distance(&camera_pos, &target_pos);

                let gizmo_rotation = match self.current_mode {
                    GizmoMode::LocalTranslation => {
                        extract_rotation_from_matrix(&target_global_transform.0)
                    }
                    _ => Quat::identity(),
                };

                gizmos::update_gizmo_transform(
                    world,
                    &new_gizmo,
                    target_pos,
                    gizmo_rotation,
                    distance,
                );
            }
        }

        self.gizmo = Some(new_gizmo);
    }
}

fn extract_rotation_from_matrix(matrix: &Mat4) -> Quat {
    let col0 = nalgebra_glm::vec3(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
    let col1 = nalgebra_glm::vec3(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)]);
    let col2 = nalgebra_glm::vec3(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)]);

    let scale_x = nalgebra_glm::length(&col0);
    let scale_y = nalgebra_glm::length(&col1);
    let scale_z = nalgebra_glm::length(&col2);

    let rot_mat = nalgebra_glm::mat3(
        col0.x / scale_x,
        col1.x / scale_y,
        col2.x / scale_z,
        col0.y / scale_x,
        col1.y / scale_y,
        col2.y / scale_z,
        col0.z / scale_x,
        col1.z / scale_y,
        col2.z / scale_z,
    );

    nalgebra_glm::mat3_to_quat(&rot_mat)
}

impl State for GizmoDemo {
    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.selection_outline_enabled = true;

        let camera = nightshade::ecs::camera::commands::spawn_camera(
            world,
            nalgebra_glm::vec3(5.0, 5.0, 10.0),
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        nightshade::ecs::world::commands::spawn_sun(world);

        let sphere_entity = world.spawn_entities(
            LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | VISIBILITY
                | BOUNDING_VOLUME
                | NAME,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(sphere_entity) {
            name.0 = "Sphere".to_string();
        }

        world.assign_local_transform(
            sphere_entity,
            LocalTransform {
                translation: nalgebra_glm::vec3(-2.0, 0.0, 0.0),
                rotation: Quat::identity(),
                scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
            },
        );

        if let Some(mesh) = world.core.get_render_mesh_mut(sphere_entity) {
            mesh.name = "Sphere".to_string();
        }

        let sphere_material = format!("Sphere_{}", sphere_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            sphere_material.clone(),
            Material {
                base_color: nalgebra_glm::vec4(0.8, 0.4, 0.1, 1.0).into(),
                metallic: 0.2,
                roughness: 0.8,
                unlit: false,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&sphere_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world
            .core
            .set_material_ref(sphere_entity, MaterialRef::new(sphere_material));

        if let Some(visible) = world.core.get_visibility_mut(sphere_entity) {
            visible.visible = true;
        }

        if let Some(bounding_volume) = world.core.get_bounding_volume_mut(sphere_entity) {
            *bounding_volume = BoundingVolume::from_mesh_type("Sphere");
        }

        self.selectable_entities.push(sphere_entity);

        let cube_entity = world.spawn_entities(
            LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | VISIBILITY
                | BOUNDING_VOLUME
                | NAME,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(cube_entity) {
            name.0 = "Cube".to_string();
        }

        world.assign_local_transform(
            cube_entity,
            LocalTransform {
                translation: nalgebra_glm::vec3(2.0, 0.0, 0.0),
                rotation: Quat::identity(),
                scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
            },
        );

        if let Some(mesh) = world.core.get_render_mesh_mut(cube_entity) {
            mesh.name = "Cube".to_string();
        }

        let cube_material = format!("Cube_{}", cube_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            cube_material.clone(),
            Material {
                base_color: nalgebra_glm::vec4(0.3, 0.6, 0.9, 1.0).into(),
                metallic: 0.2,
                roughness: 0.8,
                unlit: false,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&cube_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world
            .core
            .set_material_ref(cube_entity, MaterialRef::new(cube_material));

        if let Some(visible) = world.core.get_visibility_mut(cube_entity) {
            visible.visible = true;
        }

        if let Some(bounding_volume) = world.core.get_bounding_volume_mut(cube_entity) {
            *bounding_volume = BoundingVolume::from_mesh_type("Cube");
        }

        self.selectable_entities.push(cube_entity);
        self.select_entity(world, cube_entity);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ui_context, |ui| {
            ui.horizontal(|ui| {
                ui.label("Gizmo Mode:");

                let old_mode = self.current_mode;
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.current_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.current_mode,
                            GizmoMode::LocalTranslation,
                            "Local Translation",
                        );
                        ui.selectable_value(
                            &mut self.current_mode,
                            GizmoMode::GlobalTranslation,
                            "Global Translation",
                        );
                        ui.selectable_value(
                            &mut self.current_mode,
                            GizmoMode::Rotation,
                            "Rotation",
                        );
                        ui.selectable_value(&mut self.current_mode, GizmoMode::Scale, "Scale");
                    });

                if old_mode != self.current_mode {
                    if let (Some(old_gizmo), Some(selected)) = (&self.gizmo, self.selected_entity) {
                        gizmos::destroy_gizmo(world, old_gizmo);

                        let new_gizmo = match self.current_mode {
                            GizmoMode::LocalTranslation | GizmoMode::GlobalTranslation => {
                                gizmos::create_translation_gizmo(world, selected, self.current_mode)
                            }
                            GizmoMode::Rotation => gizmos::create_rotation_gizmo(world, selected),
                            GizmoMode::Scale => gizmos::create_scale_gizmo(world, selected),
                        };
                        self.gizmo = Some(new_gizmo);
                    }
                }

                ui.separator();

                if let Some(selected) = self.selected_entity {
                    let name = world
                        .core
                        .get_name(selected)
                        .map(|n| n.0.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    ui.label(format!("Selected: {}", name));
                } else {
                    ui.label("Click an object to select it");
                }
            });
        });
    }

    fn run_systems(&mut self, world: &mut World) {
        nightshade::ecs::camera::systems::fly_camera_system(world);

        if world.resources.user_interface.consumed_event {
            if let Some(gizmo) = &self.gizmo {
                if !matches!(self.drag_mode, GizmoDragMode::None) {
                    self.drag_mode = GizmoDragMode::None;
                    self.hover_axis = None;
                    gizmos::set_gizmo_drag_visibility(world, gizmo, GizmoDragMode::None);
                }
            }
            return;
        }

        let mouse = &world.resources.input.mouse;
        let mouse_pos = mouse.position;
        let mouse_pressed = mouse.state.contains(MouseState::LEFT_CLICKED);
        let mouse_just_pressed = mouse.state.contains(MouseState::LEFT_JUST_PRESSED);

        if let (Some(gizmo), Some(selected_entity)) = (&self.gizmo, self.selected_entity) {
            if let Some(camera_entity) = world.resources.active_camera {
                if let (Some(camera_transform), Some(target_global_transform)) = (
                    world.core.get_global_transform(camera_entity),
                    world.core.get_global_transform(selected_entity),
                ) {
                    let camera_pos = nalgebra_glm::vec3(
                        camera_transform.0[(0, 3)],
                        camera_transform.0[(1, 3)],
                        camera_transform.0[(2, 3)],
                    );
                    let target_pos = target_global_transform.translation();
                    let distance = nalgebra_glm::distance(&camera_pos, &target_pos);

                    let gizmo_rotation = match self.current_mode {
                        GizmoMode::LocalTranslation => {
                            extract_rotation_from_matrix(&target_global_transform.0)
                        }
                        _ => Quat::identity(),
                    };

                    gizmos::update_gizmo_transform(
                        world,
                        gizmo,
                        target_pos,
                        gizmo_rotation,
                        distance,
                    );
                }
            }

            if matches!(self.drag_mode, GizmoDragMode::None) {
                let all_picked = pick_entities(world, mouse_pos, PickingOptions::default());
                let hover_axis = all_picked
                    .iter()
                    .find_map(|result| gizmos::check_gizmo_axis_hit(world, gizmo, result.entity));

                if hover_axis != self.hover_axis {
                    self.hover_axis = hover_axis;
                    gizmos::highlight_gizmo_axis(world, gizmo, hover_axis);
                }

                if mouse_just_pressed {
                    if let Some(axis) = hover_axis {
                        let new_drag_mode = determine_drag_mode(axis);
                        self.drag_mode = new_drag_mode;
                        gizmos::set_gizmo_drag_visibility(world, gizmo, new_drag_mode);
                    } else if let Some(result) = all_picked.first() {
                        if self.selectable_entities.contains(&result.entity) {
                            self.select_entity(world, result.entity);
                        }
                    }
                }
            } else if mouse_pressed {
                if let Some(camera_entity) = world.resources.active_camera {
                    let mouse_delta = world.resources.input.mouse.position_delta;
                    let sensitivity = 0.01;

                    if let Some(camera_transform) = world.core.get_global_transform(camera_entity) {
                        let camera_right = nalgebra_glm::vec3(
                            camera_transform.0[(0, 0)],
                            camera_transform.0[(1, 0)],
                            camera_transform.0[(2, 0)],
                        )
                        .normalize();
                        let camera_up = nalgebra_glm::vec3(
                            camera_transform.0[(0, 1)],
                            camera_transform.0[(1, 1)],
                            camera_transform.0[(2, 1)],
                        )
                        .normalize();

                        let target_global_rotation = world
                            .core
                            .get_global_transform(selected_entity)
                            .map(|t| extract_rotation_from_matrix(&t.0))
                            .unwrap_or(Quat::identity());

                        let drag_params = DragParams {
                            mode: self.current_mode,
                            camera_right,
                            camera_up,
                            mouse_delta,
                            sensitivity,
                            target_global_rotation,
                            parent_global_rotation: Quat::identity(),
                        };

                        match &self.drag_mode {
                            GizmoDragMode::Axis(axis) => {
                                apply_axis_drag(world, selected_entity, *axis, drag_params);
                            }
                            GizmoDragMode::Plane(plane_normal) => {
                                apply_plane_drag(
                                    world,
                                    selected_entity,
                                    *plane_normal,
                                    drag_params,
                                );
                            }
                            GizmoDragMode::Free => {
                                apply_free_drag(world, selected_entity, drag_params);
                            }
                            GizmoDragMode::None => {}
                        }
                    }
                }
            } else {
                self.drag_mode = GizmoDragMode::None;
                gizmos::set_gizmo_drag_visibility(world, gizmo, GizmoDragMode::None);

                let all_picked = pick_entities(world, mouse_pos, PickingOptions::default());
                let new_hover_axis = all_picked
                    .iter()
                    .find_map(|result| gizmos::check_gizmo_axis_hit(world, gizmo, result.entity));

                if new_hover_axis != self.hover_axis {
                    self.hover_axis = new_hover_axis;
                    gizmos::highlight_gizmo_axis(world, gizmo, new_hover_axis);
                }
            }
        } else if mouse_just_pressed {
            if let Some(result) = pick_closest_entity(world, mouse_pos) {
                if self.selectable_entities.contains(&result.entity) {
                    self.select_entity(world, result.entity);
                }
            }
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::Escape, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}

fn determine_drag_mode(axis: Vec3) -> GizmoDragMode {
    if axis.x > 1.05 || axis.y > 1.05 || axis.z > 1.05 {
        let plane_normal = if axis.x > 1.05 && axis.y > 1.05 {
            nalgebra_glm::vec3(0.0, 0.0, 1.0)
        } else if axis.y > 1.05 && axis.z > 1.05 {
            nalgebra_glm::vec3(1.0, 0.0, 0.0)
        } else if axis.x > 1.05 && axis.z > 1.05 {
            nalgebra_glm::vec3(0.0, 1.0, 0.0)
        } else {
            return GizmoDragMode::Free;
        };
        GizmoDragMode::Plane(plane_normal)
    } else {
        let single_axis = if axis.x > 0.99 {
            nalgebra_glm::vec3(1.0, 0.0, 0.0)
        } else if axis.y > 0.99 {
            nalgebra_glm::vec3(0.0, 1.0, 0.0)
        } else if axis.z > 0.99 {
            nalgebra_glm::vec3(0.0, 0.0, 1.0)
        } else {
            return GizmoDragMode::Free;
        };
        GizmoDragMode::Axis(single_axis)
    }
}

struct DragParams {
    mode: GizmoMode,
    camera_right: Vec3,
    camera_up: Vec3,
    mouse_delta: Vec2,
    sensitivity: f32,
    target_global_rotation: Quat,
    parent_global_rotation: Quat,
}

fn apply_axis_drag(world: &mut World, target: Entity, axis: Vec3, params: DragParams) {
    let DragParams {
        mode,
        camera_right,
        camera_up,
        mouse_delta,
        sensitivity,
        target_global_rotation,
        parent_global_rotation,
    } = params;

    match mode {
        GizmoMode::LocalTranslation => {
            let world_axis = nalgebra_glm::quat_rotate_vec3(&target_global_rotation, &axis);
            let screen_movement = camera_right * mouse_delta.x - camera_up * mouse_delta.y;
            let movement_amount = nalgebra_glm::dot(&screen_movement, &world_axis) * sensitivity;
            let world_movement = world_axis * movement_amount;

            let parent_rotation_inv = nalgebra_glm::quat_inverse(&parent_global_rotation);
            let local_movement =
                nalgebra_glm::quat_rotate_vec3(&parent_rotation_inv, &world_movement);

            if let Some(transform) = world.core.get_local_transform(target) {
                let mut new_transform = *transform;
                new_transform.translation += local_movement;
                world.assign_local_transform(target, new_transform);
            }
        }
        GizmoMode::GlobalTranslation => {
            let screen_movement = camera_right * mouse_delta.x - camera_up * mouse_delta.y;
            let movement_amount = nalgebra_glm::dot(&screen_movement, &axis) * sensitivity;
            let world_movement = axis * movement_amount;

            let parent_rotation_inv = nalgebra_glm::quat_inverse(&parent_global_rotation);
            let local_movement =
                nalgebra_glm::quat_rotate_vec3(&parent_rotation_inv, &world_movement);

            if let Some(transform) = world.core.get_local_transform(target) {
                let mut new_transform = *transform;
                new_transform.translation += local_movement;
                world.assign_local_transform(target, new_transform);
            }
        }
        GizmoMode::Rotation => {
            let rotation_speed = sensitivity * 2.0;
            let angle = if axis.x > 0.99 {
                -mouse_delta.y * rotation_speed
            } else if axis.y > 0.99 {
                mouse_delta.x * rotation_speed
            } else {
                -mouse_delta.x * rotation_speed
            };

            if angle.abs() > 0.001 {
                let parent_rotation_inv = nalgebra_glm::quat_inverse(&parent_global_rotation);
                let local_axis = nalgebra_glm::quat_rotate_vec3(&parent_rotation_inv, &axis);
                let local_rotation = nalgebra_glm::quat_angle_axis(angle, &local_axis);
                if let Some(transform) = world.core.get_local_transform(target) {
                    let mut new_transform = *transform;
                    new_transform.rotation = local_rotation * new_transform.rotation;
                    world.assign_local_transform(target, new_transform);
                }
            }
        }
        GizmoMode::Scale => {
            let scale_delta = (mouse_delta.x - mouse_delta.y) * sensitivity;
            if let Some(transform) = world.core.get_local_transform(target) {
                let mut new_transform = *transform;
                if axis.x > 0.99 {
                    new_transform.scale.x *= 1.0 + scale_delta;
                } else if axis.y > 0.99 {
                    new_transform.scale.y *= 1.0 + scale_delta;
                } else {
                    new_transform.scale.z *= 1.0 + scale_delta;
                }
                new_transform.scale.x = new_transform.scale.x.max(0.01);
                new_transform.scale.y = new_transform.scale.y.max(0.01);
                new_transform.scale.z = new_transform.scale.z.max(0.01);
                world.assign_local_transform(target, new_transform);
            }
        }
    }
}

fn apply_plane_drag(world: &mut World, target: Entity, plane_normal: Vec3, params: DragParams) {
    let DragParams {
        mode,
        camera_right,
        camera_up,
        mouse_delta,
        sensitivity,
        target_global_rotation,
        parent_global_rotation,
    } = params;

    match mode {
        GizmoMode::LocalTranslation => {
            let world_normal =
                nalgebra_glm::quat_rotate_vec3(&target_global_rotation, &plane_normal);
            let screen_movement = camera_right * mouse_delta.x * sensitivity
                - camera_up * mouse_delta.y * sensitivity;
            let world_movement =
                screen_movement - world_normal * nalgebra_glm::dot(&screen_movement, &world_normal);

            let parent_rotation_inv = nalgebra_glm::quat_inverse(&parent_global_rotation);
            let local_movement =
                nalgebra_glm::quat_rotate_vec3(&parent_rotation_inv, &world_movement);

            if let Some(transform) = world.core.get_local_transform(target) {
                let mut new_transform = *transform;
                new_transform.translation += local_movement;
                world.assign_local_transform(target, new_transform);
            }
        }
        GizmoMode::GlobalTranslation => {
            let screen_movement = camera_right * mouse_delta.x * sensitivity
                - camera_up * mouse_delta.y * sensitivity;
            let world_movement =
                screen_movement - plane_normal * nalgebra_glm::dot(&screen_movement, &plane_normal);

            let parent_rotation_inv = nalgebra_glm::quat_inverse(&parent_global_rotation);
            let local_movement =
                nalgebra_glm::quat_rotate_vec3(&parent_rotation_inv, &world_movement);

            if let Some(transform) = world.core.get_local_transform(target) {
                let mut new_transform = *transform;
                new_transform.translation += local_movement;
                world.assign_local_transform(target, new_transform);
            }
        }
        GizmoMode::Rotation => {
            let rotation_speed = sensitivity * 2.0;
            let angle = if plane_normal.z.abs() > 0.99 {
                mouse_delta.x * rotation_speed
            } else if plane_normal.x.abs() > 0.99 {
                -mouse_delta.y * rotation_speed
            } else {
                mouse_delta.x * rotation_speed
            };

            if angle.abs() > 0.001 {
                let parent_rotation_inv = nalgebra_glm::quat_inverse(&parent_global_rotation);
                let local_axis =
                    nalgebra_glm::quat_rotate_vec3(&parent_rotation_inv, &plane_normal);
                let local_rotation = nalgebra_glm::quat_angle_axis(angle, &local_axis);
                if let Some(transform) = world.core.get_local_transform(target) {
                    let mut new_transform = *transform;
                    new_transform.rotation = local_rotation * new_transform.rotation;
                    world.assign_local_transform(target, new_transform);
                }
            }
        }
        GizmoMode::Scale => {
            let scale_delta = (mouse_delta.x - mouse_delta.y) * sensitivity;
            if let Some(transform) = world.core.get_local_transform(target) {
                let mut new_transform = *transform;
                if plane_normal.z.abs() > 0.99 {
                    new_transform.scale.x *= 1.0 + scale_delta;
                    new_transform.scale.y *= 1.0 + scale_delta;
                } else if plane_normal.x.abs() > 0.99 {
                    new_transform.scale.y *= 1.0 + scale_delta;
                    new_transform.scale.z *= 1.0 + scale_delta;
                } else {
                    new_transform.scale.x *= 1.0 + scale_delta;
                    new_transform.scale.z *= 1.0 + scale_delta;
                }
                new_transform.scale.x = new_transform.scale.x.max(0.01);
                new_transform.scale.y = new_transform.scale.y.max(0.01);
                new_transform.scale.z = new_transform.scale.z.max(0.01);
                world.assign_local_transform(target, new_transform);
            }
        }
    }
}

fn apply_free_drag(world: &mut World, target: Entity, params: DragParams) {
    let DragParams {
        mode,
        camera_right,
        camera_up,
        mouse_delta,
        sensitivity,
        target_global_rotation: _,
        parent_global_rotation,
    } = params;

    match mode {
        GizmoMode::LocalTranslation | GizmoMode::GlobalTranslation => {
            let screen_movement = camera_right * mouse_delta.x * sensitivity
                - camera_up * mouse_delta.y * sensitivity;

            let parent_rotation_inv = nalgebra_glm::quat_inverse(&parent_global_rotation);
            let local_movement =
                nalgebra_glm::quat_rotate_vec3(&parent_rotation_inv, &screen_movement);

            if let Some(transform) = world.core.get_local_transform(target) {
                let mut new_transform = *transform;
                new_transform.translation += local_movement;
                world.assign_local_transform(target, new_transform);
            }
        }
        GizmoMode::Rotation => {
            let rotation_speed = sensitivity * 2.0;
            let world_axis_x = nalgebra_glm::vec3(1.0, 0.0, 0.0);
            let world_axis_y = nalgebra_glm::vec3(0.0, 1.0, 0.0);

            let parent_rotation_inv = nalgebra_glm::quat_inverse(&parent_global_rotation);
            let local_axis_x = nalgebra_glm::quat_rotate_vec3(&parent_rotation_inv, &world_axis_x);
            let local_axis_y = nalgebra_glm::quat_rotate_vec3(&parent_rotation_inv, &world_axis_y);

            let rotation_x =
                nalgebra_glm::quat_angle_axis(-mouse_delta.y * rotation_speed, &local_axis_x);
            let rotation_y =
                nalgebra_glm::quat_angle_axis(mouse_delta.x * rotation_speed, &local_axis_y);

            if let Some(transform) = world.core.get_local_transform(target) {
                let mut new_transform = *transform;
                new_transform.rotation = rotation_y * rotation_x * new_transform.rotation;
                world.assign_local_transform(target, new_transform);
            }
        }
        GizmoMode::Scale => {
            let scale_delta = (mouse_delta.x - mouse_delta.y) * sensitivity;
            let scale_factor = 1.0 + scale_delta;

            if let Some(transform) = world.core.get_local_transform(target) {
                let mut new_transform = *transform;
                new_transform.scale *= scale_factor;
                new_transform.scale.x = new_transform.scale.x.max(0.01);
                new_transform.scale.y = new_transform.scale.y.max(0.01);
                new_transform.scale.z = new_transform.scale.z.max(0.01);
                world.assign_local_transform(target, new_transform);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nightshade::run::launch(GizmoDemo::default())?;
    Ok(())
}
