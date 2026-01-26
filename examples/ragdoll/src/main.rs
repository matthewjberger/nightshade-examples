use nightshade::ecs::animation::components::AnimationClip;
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::{ColliderComponent, RigidBodyComponent};
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::prefab::{GltfSkin, Prefab};
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use nightshade::run::RenderResources;
use rapier3d::prelude::{RigidBodyHandle, RigidBodyType};
use std::collections::HashMap;

const DANCE_MODEL: &[u8] = include_bytes!("../../../assets/models/dance.glb");
const HDR_BYTES: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(RagdollState::default())
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum RagdollBodyPart {
    Head,
    Neck,
    Spine,
    Pelvis,
    UpperArmLeft,
    UpperArmRight,
    LowerArmLeft,
    LowerArmRight,
    UpperLegLeft,
    UpperLegRight,
    LowerLegLeft,
    LowerLegRight,
}

struct BonePhysics {
    body_part: RagdollBodyPart,
    bone_entity: Entity,
    _physics_entity: Entity,
    physics_handle: RigidBodyHandle,
    hierarchy_depth: u32,
    bone_to_capsule_offset: Quat,
}

struct Ragdoll {
    body_parts: Vec<BonePhysics>,
    physics_active: bool,
    root_entity: Entity,
}

#[derive(Default)]
struct RagdollState {
    camera_entity: Option<Entity>,
    loaded: bool,
    prefab: Option<Prefab>,
    animations: Vec<AnimationClip>,
    skins: Vec<GltfSkin>,
    ragdoll: Option<Ragdoll>,
    model_entity: Option<Entity>,
}

impl State for RagdollState {
    fn title(&self) -> &str {
        "Ragdoll Physics Demo"
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

        let ssao_pass = passes::SsaoPass::new(device);
        graph
            .pass(Box::new(ssao_pass))
            .read("depth", resources.depth)
            .read("view_normals", resources.view_normals)
            .write("ssao_raw", resources.ssao_raw);

        let ssao_blur_pass = passes::SsaoBlurPass::new(device);
        graph
            .pass(Box::new(ssao_blur_pass))
            .read("ssao_raw", resources.ssao_raw)
            .write("ssao", resources.ssao);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.08);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.swapchain);
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Hdr;

        load_hdr_skybox(world, HDR_BYTES.to_vec());

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 2.0;
        }

        self.spawn_ground(world);

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 1.0, 0.0),
            5.0,
            0.0,
            0.3,
            "Ragdoll Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);
        self.camera_entity = Some(camera_entity);

        self.load_model(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        if let Some(ragdoll) = &self.ragdoll {
            if ragdoll.physics_active {
                self.sync_physics_to_bones(world);
            } else {
                self.sync_bones_to_physics(world);
            }
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let physics_active = self.ragdoll.as_ref().map(|r| r.physics_active).unwrap_or(false);
        let ragdoll_exists = self.ragdoll.is_some();

        egui::Window::new("Ragdoll Controls")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.heading("Ragdoll Physics Demo");
                ui.separator();

                if ragdoll_exists {
                    let mode_text = if physics_active {
                        "Mode: Ragdoll (Physics)"
                    } else {
                        "Mode: Animated"
                    };
                    ui.label(mode_text);

                    ui.separator();

                    if ui.button(if physics_active { "Switch to Animated" } else { "Switch to Ragdoll" }).clicked() {
                        self.toggle_ragdoll_mode(world);
                    }

                    if physics_active {
                        ui.separator();
                        if ui.button("Apply Impulse (Up)").clicked() {
                            self.apply_impulse(world, Vec3::new(0.0, 8.0, 0.0));
                        }
                        if ui.button("Apply Impulse (Forward)").clicked() {
                            self.apply_impulse(world, Vec3::new(0.0, 2.0, 5.0));
                        }
                        if ui.button("Apply Impulse (Backward)").clicked() {
                            self.apply_impulse(world, Vec3::new(0.0, 2.0, -5.0));
                        }
                    }
                } else {
                    ui.label("Model not loaded yet...");
                }

                ui.separator();
                if ui.button("Reset Position").clicked() {
                    self.reset_position(world);
                }

                ui.separator();
                ui.label("Controls:");
                ui.label("  Mouse drag - Orbit camera");
                ui.label("  Scroll - Zoom");
                ui.label("  Escape - Exit");
            });
    }
}

impl RagdollState {
    fn spawn_ground(&self, world: &mut World) {
        let ground = world.spawn_entities(
            LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | CASTS_SHADOW
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER,
            1,
        )[0];

        world.set_local_transform(
            ground,
            LocalTransform {
                translation: Vec3::new(0.0, -0.05, 0.0),
                rotation: Quat::identity(),
                scale: Vec3::new(20.0, 0.1, 20.0),
            },
        );
        world.set_render_mesh(ground, RenderMesh::new("Cube"));

        let ground_material = format!("Ground_{}", ground.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            ground_material.clone(),
            Material {
                base_color: [0.3, 0.3, 0.35, 1.0],
                roughness: 0.9,
                metallic: 0.0,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&ground_material)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(ground, MaterialRef::new(ground_material));
        world.set_casts_shadow(ground, CastsShadow);

        if let Some(rigid_body) = world.get_rigid_body_mut(ground) {
            *rigid_body = RigidBodyComponent::new_static().with_translation(0.0, -0.05, 0.0);
        }

        if let Some(collider) = world.get_collider_mut(ground) {
            *collider = ColliderComponent::new_cuboid(10.0, 0.05, 10.0).with_friction(0.8);
        }

        let rigid_body_comp = world.get_rigid_body(ground).cloned().unwrap();
        let collider_comp = world.get_collider(ground).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.get_rigid_body_mut(ground) {
            rigid_body_mut.handle = Some(handle.into());
        }
        world.resources.physics.handle_to_entity.insert(handle, ground);
        world.resources.physics.entity_to_handle.insert(ground, handle);
    }

    fn load_model(&mut self, world: &mut World) {
        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(DANCE_MODEL);

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

                self.animations = result.animations.clone();
                self.skins = result.skins.clone();

                if let Some(prefab) = result.prefabs.into_iter().next() {
                    self.prefab = Some(prefab.clone());

                    let position = Vec3::new(0.0, 0.0, 0.0);
                    let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                        world,
                        &prefab,
                        &self.animations,
                        &self.skins,
                        position,
                    );
                    self.model_entity = Some(entity);

                    self.setup_ragdoll(world, entity);
                }

                self.loaded = true;
            }
            Err(e) => {
                tracing::error!("Failed to load model: {}", e);
            }
        }
    }

    fn setup_ragdoll(&mut self, world: &mut World, root_entity: Entity) {
        let bone_map = self.find_bones(world, root_entity);

        if bone_map.is_empty() {
            tracing::warn!("No bones found for ragdoll");
            return;
        }

        let mut body_parts = Vec::new();
        let mut physics_handles: HashMap<RagdollBodyPart, RigidBodyHandle> = HashMap::new();

        for (body_part, bone_entity) in &bone_map {
            let (half_height, radius) = get_capsule_dimensions(*body_part);

            let bone_translation = world.get_global_transform(*bone_entity)
                .map(|t| t.translation())
                .unwrap_or(Vec3::zeros());

            let bone_rotation = world.get_global_transform(*bone_entity)
                .map(|t| extract_rotation_from_matrix(&t.0))
                .unwrap_or(Quat::identity());

            let bone_to_capsule_offset = calculate_bone_to_capsule_offset(world, *bone_entity, *body_part);
            let capsule_rotation = bone_rotation * bone_to_capsule_offset;

            let physics_entity = world.spawn_entities(
                NAME
                    | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | nightshade::ecs::world::RIGID_BODY
                    | nightshade::ecs::world::COLLIDER
                    | nightshade::ecs::world::PHYSICS_INTERPOLATION,
                1,
            )[0];

            world.set_name(physics_entity, Name(format!("Ragdoll_{:?}", body_part)));
            world.set_local_transform(
                physics_entity,
                LocalTransform {
                    translation: bone_translation,
                    rotation: capsule_rotation,
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
            );

            if let Some(rigid_body) = world.get_rigid_body_mut(physics_entity) {
                *rigid_body = RigidBodyComponent::new_kinematic()
                    .with_translation(
                        bone_translation.x,
                        bone_translation.y,
                        bone_translation.z,
                    )
                    .with_rotation(
                        capsule_rotation.i,
                        capsule_rotation.j,
                        capsule_rotation.k,
                        capsule_rotation.w,
                    )
                    .with_mass(get_body_part_mass(*body_part));
            }

            if let Some(collider) = world.get_collider_mut(physics_entity) {
                *collider = ColliderComponent::new_capsule(half_height, radius).with_friction(0.5);
            }

            let rigid_body_comp = world.get_rigid_body(physics_entity).cloned().unwrap();
            let collider_comp = world.get_collider(physics_entity).cloned();
            let rigid_body = rigid_body_comp.to_rapier_rigid_body();
            let handle = world.resources.physics.add_rigid_body(rigid_body);

            if let Some(collider_comp) = collider_comp {
                let collider = collider_comp.to_rapier_collider();
                world.resources.physics.add_collider(collider, handle);
            }

            if let Some(rigid_body_mut) = world.get_rigid_body_mut(physics_entity) {
                rigid_body_mut.handle = Some(handle.into());
            }

            if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(handle) {
                rb.set_linear_damping(0.5);
                rb.set_angular_damping(0.5);
            }

            world.resources.physics.handle_to_entity.insert(handle, physics_entity);
            world.resources.physics.entity_to_handle.insert(physics_entity, handle);

            if let Some(interpolation) = world.get_physics_interpolation_mut(physics_entity) {
                interpolation.previous_translation = bone_translation;
                interpolation.previous_rotation = capsule_rotation;
                interpolation.current_translation = bone_translation;
                interpolation.current_rotation = capsule_rotation;
                interpolation.enabled = true;
            }

            physics_handles.insert(*body_part, handle);

            let hierarchy_depth = calculate_bone_depth(world, *bone_entity);

            body_parts.push(BonePhysics {
                body_part: *body_part,
                bone_entity: *bone_entity,
                _physics_entity: physics_entity,
                physics_handle: handle,
                hierarchy_depth,
                bone_to_capsule_offset,
            });
        }

        body_parts.sort_by_key(|bp| bp.hierarchy_depth);

        self.create_joints(world, &physics_handles, &bone_map);

        self.ragdoll = Some(Ragdoll {
            body_parts,
            physics_active: false,
            root_entity,
        });
    }

    fn find_bones(&self, world: &World, root_entity: Entity) -> HashMap<RagdollBodyPart, Entity> {
        let mut bone_map = HashMap::new();

        let bone_entities: Vec<Entity> = if let Some(player) = world.get_animation_player(root_entity) {
            player.node_index_to_entity.values().copied().collect()
        } else {
            Vec::new()
        };

        for bone_entity in bone_entities {
            if let Some(name) = world.get_name(bone_entity) {
                let name_lower = name.0.to_lowercase();

                if (name_lower.contains("head") || name_lower.ends_with(":head"))
                    && !name_lower.contains("neck")
                    && !name_lower.contains("top_end")
                {
                    bone_map.insert(RagdollBodyPart::Head, bone_entity);
                } else if name_lower.contains("neck") {
                    bone_map.insert(RagdollBodyPart::Neck, bone_entity);
                } else if name_lower.contains("spine") && !name_lower.contains("spine1") && !name_lower.contains("spine2") {
                    bone_map.insert(RagdollBodyPart::Spine, bone_entity);
                } else if name_lower.contains("hip") || name_lower.contains("pelvis") {
                    bone_map.insert(RagdollBodyPart::Pelvis, bone_entity);
                } else if (name_lower.ends_with("arm") || name_lower.ends_with(":leftarm") || name_lower.ends_with(":rightarm"))
                    && !name_lower.contains("fore")
                {
                    if name_lower.contains("left") {
                        bone_map.insert(RagdollBodyPart::UpperArmLeft, bone_entity);
                    } else if name_lower.contains("right") {
                        bone_map.insert(RagdollBodyPart::UpperArmRight, bone_entity);
                    }
                } else if name_lower.contains("forearm") {
                    if name_lower.contains("left") {
                        bone_map.insert(RagdollBodyPart::LowerArmLeft, bone_entity);
                    } else if name_lower.contains("right") {
                        bone_map.insert(RagdollBodyPart::LowerArmRight, bone_entity);
                    }
                } else if name_lower.contains("upleg") {
                    if name_lower.contains("left") {
                        bone_map.insert(RagdollBodyPart::UpperLegLeft, bone_entity);
                    } else if name_lower.contains("right") {
                        bone_map.insert(RagdollBodyPart::UpperLegRight, bone_entity);
                    }
                } else if (name_lower.ends_with("leg") || name_lower.ends_with(":leftleg") || name_lower.ends_with(":rightleg"))
                    && !name_lower.contains("up")
                {
                    if name_lower.contains("left") {
                        bone_map.insert(RagdollBodyPart::LowerLegLeft, bone_entity);
                    } else if name_lower.contains("right") {
                        bone_map.insert(RagdollBodyPart::LowerLegRight, bone_entity);
                    }
                }
            }
        }

        bone_map
    }

    fn create_joints(
        &self,
        world: &mut World,
        physics_handles: &HashMap<RagdollBodyPart, RigidBodyHandle>,
        bone_map: &HashMap<RagdollBodyPart, Entity>,
    ) {
        use rapier3d::prelude::*;

        let joint_connections: &[(RagdollBodyPart, RagdollBodyPart)] = &[
            (RagdollBodyPart::Neck, RagdollBodyPart::Head),
            (RagdollBodyPart::Spine, RagdollBodyPart::Neck),
            (RagdollBodyPart::Pelvis, RagdollBodyPart::Spine),
            (RagdollBodyPart::Spine, RagdollBodyPart::UpperArmLeft),
            (RagdollBodyPart::Spine, RagdollBodyPart::UpperArmRight),
            (RagdollBodyPart::UpperArmLeft, RagdollBodyPart::LowerArmLeft),
            (RagdollBodyPart::UpperArmRight, RagdollBodyPart::LowerArmRight),
            (RagdollBodyPart::Pelvis, RagdollBodyPart::UpperLegLeft),
            (RagdollBodyPart::Pelvis, RagdollBodyPart::UpperLegRight),
            (RagdollBodyPart::UpperLegLeft, RagdollBodyPart::LowerLegLeft),
            (RagdollBodyPart::UpperLegRight, RagdollBodyPart::LowerLegRight),
        ];

        for (parent_part, child_part) in joint_connections {
            let Some(&parent_handle) = physics_handles.get(parent_part) else { continue };
            let Some(&child_handle) = physics_handles.get(child_part) else { continue };
            let Some(&parent_bone) = bone_map.get(parent_part) else { continue };
            let Some(&child_bone) = bone_map.get(child_part) else { continue };

            let child_world_pos = world.get_global_transform(child_bone)
                .map(|t| t.translation())
                .unwrap_or(Vec3::zeros());

            let parent_world_pos = world.get_global_transform(parent_bone)
                .map(|t| t.translation())
                .unwrap_or(Vec3::zeros());
            let parent_world_rot = world.get_global_transform(parent_bone)
                .map(|t| extract_rotation_from_matrix(&t.0))
                .unwrap_or(Quat::identity());
            let parent_offset = calculate_bone_to_capsule_offset(world, parent_bone, *parent_part);
            let parent_capsule_rot = parent_world_rot * parent_offset;

            let child_world_rot = world.get_global_transform(child_bone)
                .map(|t| extract_rotation_from_matrix(&t.0))
                .unwrap_or(Quat::identity());
            let child_offset = calculate_bone_to_capsule_offset(world, child_bone, *child_part);
            let child_capsule_rot = child_world_rot * child_offset;

            let joint_world_pos = child_world_pos;

            let parent_local_anchor = nalgebra_glm::quat_rotate_vec3(
                &nalgebra_glm::quat_inverse(&parent_capsule_rot),
                &(joint_world_pos - parent_world_pos),
            );

            let child_local_anchor = nalgebra_glm::quat_rotate_vec3(
                &nalgebra_glm::quat_inverse(&child_capsule_rot),
                &(joint_world_pos - child_world_pos),
            );

            let joint = SphericalJointBuilder::new()
                .local_anchor1(point![parent_local_anchor.x, parent_local_anchor.y, parent_local_anchor.z])
                .local_anchor2(point![child_local_anchor.x, child_local_anchor.y, child_local_anchor.z]);
            world.resources.physics.add_joint(parent_handle, child_handle, joint);
        }
    }

    fn toggle_ragdoll_mode(&mut self, world: &mut World) {
        let Some(ragdoll) = &mut self.ragdoll else {
            return;
        };

        ragdoll.physics_active = !ragdoll.physics_active;

        for body_part in &ragdoll.body_parts {
            if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(body_part.physics_handle) {
                if ragdoll.physics_active {
                    rb.set_body_type(RigidBodyType::Dynamic, true);
                    rb.wake_up(true);
                } else {
                    rb.set_body_type(RigidBodyType::KinematicPositionBased, true);
                }
            }
        }

        if ragdoll.physics_active {
            if let Some(player) = world.get_animation_player_mut(ragdoll.root_entity) {
                player.pause();
            }
        } else if let Some(player) = world.get_animation_player_mut(ragdoll.root_entity) {
            player.resume();
        }
    }

    fn sync_bones_to_physics(&self, world: &mut World) {
        let Some(ragdoll) = &self.ragdoll else {
            return;
        };

        for body_part in &ragdoll.body_parts {
            let bone_translation = world.get_global_transform(body_part.bone_entity)
                .map(|t| t.translation())
                .unwrap_or(Vec3::zeros());

            let bone_rotation = world.get_global_transform(body_part.bone_entity)
                .map(|t| extract_rotation_from_matrix(&t.0))
                .unwrap_or(Quat::identity());

            let capsule_rotation = bone_rotation * body_part.bone_to_capsule_offset;

            if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(body_part.physics_handle) {
                rb.set_next_kinematic_translation(rapier3d::prelude::Vector::new(
                    bone_translation.x,
                    bone_translation.y,
                    bone_translation.z,
                ));
                rb.set_next_kinematic_rotation(rapier3d::prelude::Rotation::from_quaternion(
                    rapier3d::na::Quaternion::new(
                        capsule_rotation.w,
                        capsule_rotation.i,
                        capsule_rotation.j,
                        capsule_rotation.k,
                    ),
                ));
            }
        }
    }

    fn sync_physics_to_bones(&self, world: &mut World) {
        let Some(ragdoll) = &self.ragdoll else {
            return;
        };

        let mut updated_global_rotations: HashMap<Entity, Quat> = HashMap::new();

        for body_part in &ragdoll.body_parts {
            let Some(rb) = world.resources.physics.rigid_body_set.get(body_part.physics_handle) else {
                continue;
            };

            let rb_rot = rb.rotation();
            let capsule_world_rotation = Quat::new(rb_rot.w, rb_rot.i, rb_rot.j, rb_rot.k);

            let bone_world_rotation = capsule_world_rotation * nalgebra_glm::quat_inverse(&body_part.bone_to_capsule_offset);

            let effective_parent_global_rotation = self.get_effective_parent_rotation(
                world,
                body_part.bone_entity,
                &updated_global_rotations,
            );

            let local_rotation = nalgebra_glm::quat_inverse(&effective_parent_global_rotation) * bone_world_rotation;

            if body_part.body_part == RagdollBodyPart::Pelvis {
                let rb_pos = rb.translation();
                let physics_translation = Vec3::new(rb_pos.x, rb_pos.y, rb_pos.z);

                let parent_global_translation = world.get_parent(body_part.bone_entity)
                    .and_then(|parent| parent.0)
                    .and_then(|parent_entity| world.get_global_transform(parent_entity))
                    .map(|gt| gt.translation())
                    .unwrap_or(Vec3::zeros());

                let local_translation = nalgebra_glm::quat_rotate_vec3(
                    &nalgebra_glm::quat_inverse(&effective_parent_global_rotation),
                    &(physics_translation - parent_global_translation),
                );

                if let Some(transform) = world.get_local_transform_mut(body_part.bone_entity) {
                    transform.translation = local_translation;
                    transform.rotation = local_rotation;
                }
            } else if let Some(transform) = world.get_local_transform_mut(body_part.bone_entity) {
                transform.rotation = local_rotation;
            }

            updated_global_rotations.insert(body_part.bone_entity, bone_world_rotation);

            world.mark_local_transform_dirty(body_part.bone_entity);
        }
    }

    fn get_effective_parent_rotation(
        &self,
        world: &World,
        bone_entity: Entity,
        updated_rotations: &HashMap<Entity, Quat>,
    ) -> Quat {
        let Some(direct_parent) = world.get_parent(bone_entity).and_then(|p| p.0) else {
            return Quat::identity();
        };

        if let Some(&cached_rotation) = updated_rotations.get(&direct_parent) {
            return cached_rotation;
        }

        let mut current = direct_parent;
        let mut intermediate_local_rotations: Vec<Quat> = Vec::new();

        if let Some(local_transform) = world.get_local_transform(direct_parent) {
            intermediate_local_rotations.push(local_transform.rotation);
        }

        while let Some(parent_entity) = world.get_parent(current).and_then(|p| p.0) {
            if let Some(&cached_rotation) = updated_rotations.get(&parent_entity) {
                let mut result = cached_rotation;
                for local_rot in &intermediate_local_rotations {
                    result = result * *local_rot;
                }
                return result;
            }

            if let Some(local_transform) = world.get_local_transform(parent_entity) {
                intermediate_local_rotations.insert(0, local_transform.rotation);
            }

            current = parent_entity;
        }

        world.get_global_transform(direct_parent)
            .map(|gt| extract_rotation_from_matrix(&gt.0))
            .unwrap_or(Quat::identity())
    }

    fn apply_impulse(&self, world: &mut World, impulse: Vec3) {
        let Some(ragdoll) = &self.ragdoll else {
            return;
        };

        for body_part in &ragdoll.body_parts {
            if matches!(body_part.body_part, RagdollBodyPart::Pelvis | RagdollBodyPart::Spine)
                && let Some(rb) = world.resources.physics.rigid_body_set.get_mut(body_part.physics_handle)
            {
                rb.apply_impulse(rapier3d::prelude::Vector::new(impulse.x, impulse.y, impulse.z), true);
            }
        }
    }

    fn reset_position(&mut self, world: &mut World) {
        let Some(ragdoll) = &mut self.ragdoll else {
            return;
        };

        if ragdoll.physics_active {
            ragdoll.physics_active = false;
            for body_part in &ragdoll.body_parts {
                if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(body_part.physics_handle) {
                    rb.set_body_type(RigidBodyType::KinematicPositionBased, true);
                }
            }
            if let Some(player) = world.get_animation_player_mut(ragdoll.root_entity) {
                player.resume();
            }
        }

        if let Some(transform) = world.get_local_transform_mut(ragdoll.root_entity) {
            transform.translation = Vec3::new(0.0, 0.0, 0.0);
            transform.rotation = Quat::identity();
        }
        world.mark_local_transform_dirty(ragdoll.root_entity);
    }
}

fn get_capsule_dimensions(body_part: RagdollBodyPart) -> (f32, f32) {
    match body_part {
        RagdollBodyPart::Head => (0.06, 0.08),
        RagdollBodyPart::Neck => (0.04, 0.035),
        RagdollBodyPart::Spine => (0.12, 0.10),
        RagdollBodyPart::Pelvis => (0.08, 0.10),
        RagdollBodyPart::UpperArmLeft | RagdollBodyPart::UpperArmRight => (0.12, 0.035),
        RagdollBodyPart::LowerArmLeft | RagdollBodyPart::LowerArmRight => (0.10, 0.03),
        RagdollBodyPart::UpperLegLeft | RagdollBodyPart::UpperLegRight => (0.18, 0.05),
        RagdollBodyPart::LowerLegLeft | RagdollBodyPart::LowerLegRight => (0.16, 0.04),
    }
}

fn get_body_part_mass(body_part: RagdollBodyPart) -> f32 {
    match body_part {
        RagdollBodyPart::Head => 5.0,
        RagdollBodyPart::Neck => 1.0,
        RagdollBodyPart::Spine => 15.0,
        RagdollBodyPart::Pelvis => 12.0,
        RagdollBodyPart::UpperArmLeft | RagdollBodyPart::UpperArmRight => 2.5,
        RagdollBodyPart::LowerArmLeft | RagdollBodyPart::LowerArmRight => 1.5,
        RagdollBodyPart::UpperLegLeft | RagdollBodyPart::UpperLegRight => 8.0,
        RagdollBodyPart::LowerLegLeft | RagdollBodyPart::LowerLegRight => 5.0,
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
        col0.x / scale_x, col0.y / scale_x, col0.z / scale_x,
        col1.x / scale_y, col1.y / scale_y, col1.z / scale_y,
        col2.x / scale_z, col2.y / scale_z, col2.z / scale_z,
    );

    nalgebra_glm::mat3_to_quat(&rot_mat)
}

fn calculate_bone_depth(world: &World, bone_entity: Entity) -> u32 {
    let mut depth = 0;
    let mut current = bone_entity;
    while let Some(parent) = world.get_parent(current).and_then(|p| p.0) {
        depth += 1;
        current = parent;
    }
    depth
}

fn calculate_bone_to_capsule_offset(world: &World, bone_entity: Entity, body_part: RagdollBodyPart) -> Quat {
    let bone_world_rotation = world.get_global_transform(bone_entity)
        .map(|t| extract_rotation_from_matrix(&t.0))
        .unwrap_or(Quat::identity());
    let bone_world_pos = world.get_global_transform(bone_entity)
        .map(|t| t.translation())
        .unwrap_or(Vec3::zeros());

    let child_world_pos = find_child_bone_position(world, bone_entity, body_part);

    let bone_direction_world = if let Some(child_pos) = child_world_pos {
        let dir = child_pos - bone_world_pos;
        let len = nalgebra_glm::length(&dir);
        if len > 0.001 {
            dir / len
        } else {
            nalgebra_glm::quat_rotate_vec3(&bone_world_rotation, &Vec3::new(0.0, 1.0, 0.0))
        }
    } else {
        nalgebra_glm::quat_rotate_vec3(&bone_world_rotation, &Vec3::new(0.0, 1.0, 0.0))
    };

    let bone_direction_local = nalgebra_glm::quat_rotate_vec3(
        &nalgebra_glm::quat_inverse(&bone_world_rotation),
        &bone_direction_world,
    );

    rotation_from_y_to_direction(&bone_direction_local)
}

fn find_child_bone_position(world: &World, bone_entity: Entity, body_part: RagdollBodyPart) -> Option<Vec3> {
    let target_child_name = match body_part {
        RagdollBodyPart::Head => return None,
        RagdollBodyPart::Neck => Some("head"),
        RagdollBodyPart::Spine => Some("spine1"),
        RagdollBodyPart::Pelvis => Some("spine"),
        RagdollBodyPart::UpperArmLeft => Some("leftforearm"),
        RagdollBodyPart::UpperArmRight => Some("rightforearm"),
        RagdollBodyPart::LowerArmLeft => Some("lefthand"),
        RagdollBodyPart::LowerArmRight => Some("righthand"),
        RagdollBodyPart::UpperLegLeft => Some("leftleg"),
        RagdollBodyPart::UpperLegRight => Some("rightleg"),
        RagdollBodyPart::LowerLegLeft => Some("leftfoot"),
        RagdollBodyPart::LowerLegRight => Some("rightfoot"),
    };

    let target = target_child_name?;

    let root = {
        let mut current = bone_entity;
        while let Some(parent) = world.get_parent(current).and_then(|p| p.0) {
            current = parent;
        }
        current
    };

    let bone_entities: Vec<Entity> = if let Some(player) = world.get_animation_player(root) {
        player.node_index_to_entity.values().copied().collect()
    } else {
        return None;
    };

    for entity in bone_entities {
        if let Some(name) = world.get_name(entity) {
            let name_lower = name.0.to_lowercase();
            if name_lower.contains(target) {
                return world.get_global_transform(entity).map(|t| t.translation());
            }
        }
    }

    None
}

fn rotation_from_y_to_direction(direction: &Vec3) -> Quat {
    let y_axis = Vec3::new(0.0, 1.0, 0.0);

    let dot = nalgebra_glm::dot(&y_axis, direction);

    if dot > 0.9999 {
        return Quat::identity();
    }

    if dot < -0.9999 {
        return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::new(1.0, 0.0, 0.0));
    }

    let axis = nalgebra_glm::cross(&y_axis, direction);
    let axis_len = nalgebra_glm::length(&axis);
    if axis_len < 0.0001 {
        return Quat::identity();
    }
    let axis_normalized = axis / axis_len;

    let angle = dot.acos();

    nalgebra_glm::quat_angle_axis(angle, &axis_normalized)
}
