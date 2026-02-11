use std::collections::{HashMap, HashSet};

use nightshade::ecs::animation::components::AnimationClip;
use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::RigidBodyHandle as PhysicsHandle;
use nightshade::ecs::physics::debug::physics_debug_draw_system;
use nightshade::ecs::physics::*;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::prefab::{GltfSkin, Prefab};
use nightshade::ecs::transform::queries::query_children;
use nightshade::ecs::world::{
    BOUNDING_VOLUME, CASTS_SHADOW, GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY,
    MATERIAL_REF, NAME, RENDER_MESH, VISIBILITY,
};
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use nightshade::run::RenderResources;

const DANCE_MODEL: &[u8] = include_bytes!("../../../assets/models/dance.glb");
const HDR_BYTES: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(RagdollState::default())
}

struct BoneBodyMapping {
    bone_entity: Entity,
    body_handle: PhysicsHandle,
    bone_local_scale: Vec3,
}

struct RagdollInstance {
    root_entity: Entity,
    #[allow(dead_code)]
    mesh_entity: Entity,
    bone_entities: Vec<Entity>,
    bone_body_mappings: Vec<BoneBodyMapping>,
    rapier_handles: Vec<PhysicsHandle>,
    is_ragdolled: bool,
    pending_ragdoll: bool,
    frames_alive: u32,
}

struct RagdollState {
    prefab: Option<Prefab>,
    animations: Vec<AnimationClip>,
    skins: Vec<GltfSkin>,
    ragdolls: Vec<RagdollInstance>,
    camera_entity: Option<Entity>,
    loaded: bool,
    show_debug_physics: bool,
    debug_key_was_pressed: bool,
    spawn_radius: f32,
    spawn_height: f32,
    home_focus: Vec3,
    home_radius: f32,
    home_yaw: f32,
    home_pitch: f32,
    physics_paused: bool,
    pause_key_was_pressed: bool,
    saved_max_substeps: u32,
    capsule_radius_multiplier: f32,
    capsule_min_radius: f32,
    leaf_radius: f32,
    linear_damping: f32,
    angular_damping: f32,
}

impl Default for RagdollState {
    fn default() -> Self {
        Self {
            prefab: None,
            animations: Vec::new(),
            skins: Vec::new(),
            ragdolls: Vec::new(),
            camera_entity: None,
            loaded: false,
            show_debug_physics: false,
            debug_key_was_pressed: false,
            spawn_radius: 3.0,
            spawn_height: 10.0,
            home_focus: Vec3::zeros(),
            home_radius: 12.0,
            home_yaw: 0.0,
            home_pitch: 0.4,
            physics_paused: false,
            pause_key_was_pressed: false,
            saved_max_substeps: 4,
            capsule_radius_multiplier: 0.2,
            capsule_min_radius: 0.08,
            leaf_radius: 0.12,
            linear_damping: 0.5,
            angular_damping: 0.5,
        }
    }
}

impl State for RagdollState {
    fn title(&self) -> &str {
        "Ragdoll Demo"
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
        world.resources.graphics.show_grid = false;
        world.resources.graphics.use_fullscreen = true;
        world.resources.graphics.ui_scale = Some(1.0);
        world.resources.graphics.atmosphere = Atmosphere::Hdr;

        load_hdr_skybox(world, HDR_BYTES.to_vec());

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 2.0;
        }

        self.spawn_ground(world);

        self.home_focus = Vec3::new(0.0, 1.5, 0.0);
        self.home_radius = 12.0;
        self.home_yaw = 0.0;
        self.home_pitch = 0.4;

        let camera_entity = spawn_pan_orbit_camera(
            world,
            self.home_focus,
            self.home_radius,
            self.home_yaw,
            self.home_pitch,
            "Ragdoll Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);
        self.camera_entity = Some(camera_entity);

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
                    self.prefab = Some(prefab);
                    self.spawn_dancer(world, Vec3::zeros());
                }

                self.loaded = true;
            }
            Err(error) => {
                tracing::error!("Failed to load dance model: {}", error);
            }
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        if world.resources.input.keyboard.is_key_pressed(KeyCode::KeyC)
            || world.resources.input.keyboard.is_key_pressed(KeyCode::Home)
        {
            self.reset_camera_to_home(world);
        }

        let pause_key_pressed = world.resources.input.keyboard.is_key_pressed(KeyCode::KeyP);
        if pause_key_pressed && !self.pause_key_was_pressed {
            self.toggle_physics_pause(world);
        }
        self.pause_key_was_pressed = pause_key_pressed;

        for index in 0..self.ragdolls.len() {
            self.ragdolls[index].frames_alive += 1;
        }

        let pending_indices: Vec<usize> = self
            .ragdolls
            .iter()
            .enumerate()
            .filter(|(_, instance)| instance.pending_ragdoll && instance.frames_alive >= 2)
            .map(|(index, _)| index)
            .collect();

        for index in pending_indices {
            self.convert_to_ragdoll(world, index);
        }

        self.sync_ragdoll_bones(world);

        let debug_key_pressed = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Digit4);
        if debug_key_pressed && !self.debug_key_was_pressed {
            self.show_debug_physics = !self.show_debug_physics;
            world.resources.physics.debug_draw = self.show_debug_physics;
        }
        self.debug_key_was_pressed = debug_key_pressed;

        if self.show_debug_physics {
            physics_debug_draw_system(world);
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let fps = world.resources.window.timing.frames_per_second;
        let total_ragdolls = self.ragdolls.len();
        let ragdolled_count = self.ragdolls.iter().filter(|r| r.is_ragdolled).count();
        let dancing_count = total_ragdolls - ragdolled_count;

        egui::Window::new("Ragdoll Demo")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.label(format!("FPS: {:.0}", fps));
                ui.label(format!("Dancing: {}", dancing_count));
                ui.label(format!("Ragdolled: {}", ragdolled_count));
                ui.label(format!("Total: {}", total_ragdolls));

                ui.separator();

                let pause_label = if self.physics_paused {
                    "Resume Physics [P]"
                } else {
                    "Pause Physics [P]"
                };
                if ui.button(pause_label).clicked() {
                    self.toggle_physics_pause(world);
                }

                ui.separator();
                ui.strong("Spawn Settings");

                ui.horizontal(|ui| {
                    ui.label("Height:");
                    ui.add(egui::Slider::new(&mut self.spawn_height, 3.0..=30.0).suffix("m"));
                });

                ui.horizontal(|ui| {
                    ui.label("Radius:");
                    ui.add(egui::Slider::new(&mut self.spawn_radius, 0.5..=10.0).suffix("m"));
                });

                ui.separator();
                ui.strong("Collider Settings");

                ui.horizontal(|ui| {
                    ui.label("Capsule radius mult:");
                    ui.add(
                        egui::Slider::new(&mut self.capsule_radius_multiplier, 0.05..=1.0)
                            .step_by(0.01),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Capsule min radius:");
                    ui.add(
                        egui::Slider::new(&mut self.capsule_min_radius, 0.01..=0.3).step_by(0.005),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Leaf sphere radius:");
                    ui.add(egui::Slider::new(&mut self.leaf_radius, 0.02..=0.3).step_by(0.005));
                });

                ui.separator();
                ui.strong("Physics Settings");

                ui.horizontal(|ui| {
                    ui.label("Linear damping:");
                    ui.add(egui::Slider::new(&mut self.linear_damping, 0.0..=5.0).step_by(0.01));
                });

                ui.horizontal(|ui| {
                    ui.label("Angular damping:");
                    ui.add(egui::Slider::new(&mut self.angular_damping, 0.0..=5.0).step_by(0.01));
                });

                ui.separator();
                ui.strong("Actions");

                ui.horizontal(|ui| {
                    if ui.button("Ragdoll All [Space]").clicked() {
                        self.ragdoll_all_dancers();
                    }
                    if ui.button("Drop [D]").clicked() {
                        self.spawn_ragdoll_from_sky(world);
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Spawn Dancer [F]").clicked() {
                        self.spawn_dancer(world, Vec3::zeros());
                    }
                    if ui.button("Reset [R]").clicked() {
                        self.reset(world);
                    }
                });

                ui.separator();

                let mut debug = self.show_debug_physics;
                if ui.checkbox(&mut debug, "Physics Debug [4]").changed() {
                    self.show_debug_physics = debug;
                    world.resources.physics.debug_draw = debug;
                }

                ui.separator();
                ui.collapsing("Controls", |ui| {
                    ui.label("  Space - Ragdoll all dancers");
                    ui.label("  D - Drop ragdoll from sky");
                    ui.label("  F - Spawn new dancer");
                    ui.label("  R - Reset");
                    ui.label("  P - Pause/resume physics");
                    ui.label("  4 - Toggle physics debug");
                    ui.label("  C / Home - Reset camera");
                    ui.label("  Mouse drag - Orbit camera");
                    ui.label("  Scroll - Zoom");
                });
            });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        match key {
            KeyCode::Space => {
                self.ragdoll_all_dancers();
            }
            KeyCode::KeyD => {
                self.spawn_ragdoll_from_sky(world);
            }
            KeyCode::KeyF => {
                self.spawn_dancer(world, Vec3::zeros());
            }
            KeyCode::KeyR => {
                self.reset(world);
            }
            KeyCode::KeyP => {
                self.toggle_physics_pause(world);
            }
            KeyCode::KeyC | KeyCode::Home => {
                self.reset_camera_to_home(world);
            }
            _ => {}
        }
    }
}

impl RagdollState {
    fn toggle_physics_pause(&mut self, world: &mut World) {
        self.physics_paused = !self.physics_paused;
        if self.physics_paused {
            self.saved_max_substeps = world.resources.physics.max_substeps;
            world.resources.physics.max_substeps = 0;
        } else {
            world.resources.physics.max_substeps = self.saved_max_substeps;
        }
    }

    fn spawn_ground(&self, world: &mut World) {
        let ground = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | CASTS_SHADOW
                | BOUNDING_VOLUME
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER,
            1,
        )[0];

        world.set_local_transform(
            ground,
            LocalTransform {
                translation: Vec3::new(0.0, -0.05, 0.0),
                rotation: Quat::identity(),
                scale: Vec3::new(50.0, 0.1, 50.0),
            },
        );
        world.set_render_mesh(ground, RenderMesh::new("Cube"));

        let ground_material_name = format!("Ground_{}", ground.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            ground_material_name.clone(),
            Material {
                base_color: [0.25, 0.25, 0.3, 1.0],
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
            .get(&ground_material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(ground, MaterialRef::new(ground_material_name));
        world.set_casts_shadow(ground, CastsShadow);

        if let Some(rigid_body) = world.get_rigid_body_mut(ground) {
            *rigid_body = RigidBodyComponent::new_static().with_translation(0.0, -0.05, 0.0);
        }

        if let Some(collider) = world.get_collider_mut(ground) {
            *collider = ColliderComponent::new_cuboid(25.0, 0.05, 25.0).with_friction(0.8);
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
        world
            .resources
            .physics
            .handle_to_entity
            .insert(handle, ground);
        world
            .resources
            .physics
            .entity_to_handle
            .insert(ground, handle);
    }

    fn spawn_dancer(&mut self, world: &mut World, position: Vec3) {
        let Some(prefab) = &self.prefab.clone() else {
            return;
        };

        let root_entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
            world,
            prefab,
            &self.animations,
            &self.skins,
            position,
        );

        if let Some(player) = world.get_animation_player_mut(root_entity)
            && !player.clips.is_empty()
        {
            player.play(0);
            player.looping = true;
            player.speed = 1.0;
        }

        let mesh_entity = find_skinned_mesh_entity(world, root_entity);
        let bone_entities = mesh_entity
            .and_then(|mesh| world.get_skin(mesh))
            .map(|skin| skin.joints.clone())
            .unwrap_or_default();

        self.ragdolls.push(RagdollInstance {
            root_entity,
            mesh_entity: mesh_entity.unwrap_or(root_entity),
            bone_entities,
            bone_body_mappings: Vec::new(),
            rapier_handles: Vec::new(),
            is_ragdolled: false,
            pending_ragdoll: false,
            frames_alive: 0,
        });
    }

    fn ragdoll_all_dancers(&mut self) {
        for instance in &mut self.ragdolls {
            if !instance.is_ragdolled && !instance.pending_ragdoll {
                instance.pending_ragdoll = true;
            }
        }
    }

    fn spawn_ragdoll_from_sky(&mut self, world: &mut World) {
        let angle = rand_f32() * std::f32::consts::TAU;
        let radius = rand_f32() * self.spawn_radius;
        let offset_x = angle.cos() * radius;
        let offset_z = angle.sin() * radius;
        let position = Vec3::new(offset_x, self.spawn_height, offset_z);

        self.spawn_dancer(world, position);

        let last_index = self.ragdolls.len() - 1;
        self.ragdolls[last_index].pending_ragdoll = true;
    }

    fn convert_to_ragdoll(&mut self, world: &mut World, ragdoll_index: usize) {
        let instance = &mut self.ragdolls[ragdoll_index];
        if instance.is_ragdolled {
            return;
        }

        instance.pending_ragdoll = false;
        instance.is_ragdolled = true;

        let root_entity = instance.root_entity;
        let bone_entities = instance.bone_entities.clone();

        let bone_name_to_entity: HashMap<String, Entity> = world
            .get_animation_player(root_entity)
            .map(|player| player.bone_name_to_entity.clone())
            .unwrap_or_default();

        if let Some(player) = world.get_animation_player_mut(root_entity) {
            player.stop();
        }

        if bone_entities.is_empty() {
            return;
        }

        let ragdoll_bone_names: [&str; 11] = [
            "mixamorig:Hips",
            "mixamorig:Spine2",
            "mixamorig:Head",
            "mixamorig:LeftArm",
            "mixamorig:LeftForeArm",
            "mixamorig:RightArm",
            "mixamorig:RightForeArm",
            "mixamorig:LeftUpLeg",
            "mixamorig:LeftLeg",
            "mixamorig:RightUpLeg",
            "mixamorig:RightLeg",
        ];

        let entity_to_name: HashMap<Entity, String> = bone_name_to_entity
            .iter()
            .map(|(name, &entity)| (entity, name.clone()))
            .collect();

        let ragdoll_entities: Vec<Entity> = ragdoll_bone_names
            .iter()
            .filter_map(|&name| bone_name_to_entity.get(name).copied())
            .collect();

        let ragdoll_entity_set: HashSet<Entity> = ragdoll_entities.iter().copied().collect();
        let bone_set: HashSet<Entity> = bone_entities.iter().copied().collect();

        let mut bone_global_transforms: HashMap<Entity, (Vec3, Quat)> = HashMap::new();
        let mut bone_parent_map: HashMap<Entity, Entity> = HashMap::new();

        for &bone in &bone_entities {
            if let Some(global) = world.get_global_transform(bone) {
                let matrix = global.0;
                let translation = Vec3::new(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)]);
                let rotation = extract_rotation_from_matrix(&matrix);
                bone_global_transforms.insert(bone, (translation, rotation));
            }

            if let Some(parent) = world.get_parent(bone)
                && let Some(parent_entity) = parent.0
                && bone_set.contains(&parent_entity)
            {
                bone_parent_map.insert(bone, parent_entity);
            }
        }

        let ragdoll_parent_map: HashMap<Entity, Entity> = ragdoll_entities
            .iter()
            .filter_map(|&entity| {
                let mut current = bone_parent_map.get(&entity).copied();
                while let Some(parent) = current {
                    if ragdoll_entity_set.contains(&parent) {
                        return Some((entity, parent));
                    }
                    current = bone_parent_map.get(&parent).copied();
                }
                None
            })
            .collect();

        let mut ragdoll_children_map: HashMap<Entity, Vec<Entity>> = HashMap::new();
        for (&child, &parent) in &ragdoll_parent_map {
            ragdoll_children_map.entry(parent).or_default().push(child);
        }

        let mut bone_local_scales: HashMap<Entity, Vec3> = HashMap::new();
        for &bone in &ragdoll_entities {
            if let Some(local) = world.get_local_transform(bone) {
                bone_local_scales.insert(bone, local.scale);
            }
        }

        use rapier3d::prelude::*;

        let mut entity_to_rapier_handle: HashMap<Entity, RigidBodyHandle> = HashMap::new();
        let mut all_handles: Vec<PhysicsHandle> = Vec::new();
        let mut bone_body_mappings: Vec<BoneBodyMapping> = Vec::new();

        for &bone in &ragdoll_entities {
            let (translation, rotation) = bone_global_transforms
                .get(&bone)
                .copied()
                .unwrap_or((Vec3::zeros(), nalgebra_glm::quat_identity()));

            let bone_name = entity_to_name.get(&bone).map(|s| s.as_str()).unwrap_or("");
            let mass = match bone_name {
                "mixamorig:Hips" | "mixamorig:Spine2" => 2.0,
                "mixamorig:Head" => 1.5,
                "mixamorig:LeftUpLeg" | "mixamorig:RightUpLeg" => 1.5,
                _ => 0.5,
            };

            let rapier_translation = vector![translation.x, translation.y, translation.z];
            let rapier_rotation = rapier3d::na::UnitQuaternion::new_normalize(
                rapier3d::na::Quaternion::new(rotation.w, rotation.i, rotation.j, rotation.k),
            );
            let position =
                rapier3d::na::Isometry3::from_parts(rapier_translation.into(), rapier_rotation);

            let mut rigid_body = RigidBodyBuilder::new(rapier3d::prelude::RigidBodyType::Dynamic)
                .pose(position)
                .linear_damping(self.linear_damping)
                .angular_damping(self.angular_damping)
                .build();
            rigid_body.set_additional_mass(mass, true);

            let rapier_handle = world.resources.physics.add_rigid_body(rigid_body);

            let children = ragdoll_children_map.get(&bone);
            let has_children = children.map(|c| !c.is_empty()).unwrap_or(false);

            if has_children {
                for &child in children.unwrap() {
                    let (child_pos, _) = bone_global_transforms[&child];
                    let offset_world = child_pos - translation;
                    let distance = offset_world.norm();
                    if distance < 0.01 {
                        continue;
                    }

                    let inv_rot = nalgebra_glm::quat_conjugate(&rotation);
                    let local_dir = nalgebra_glm::quat_rotate_vec3(&inv_rot, &offset_world);

                    let shrink = 0.04;
                    let half_dir =
                        local_dir.normalize() * ((distance - shrink * 2.0).max(0.05) / 2.0);
                    let center = local_dir.normalize() * (distance / 2.0);

                    let point_a = nalgebra::Point3::new(
                        center.x - half_dir.x,
                        center.y - half_dir.y,
                        center.z - half_dir.z,
                    );
                    let point_b = nalgebra::Point3::new(
                        center.x + half_dir.x,
                        center.y + half_dir.y,
                        center.z + half_dir.z,
                    );

                    let radius =
                        (distance * self.capsule_radius_multiplier).max(self.capsule_min_radius);
                    let shape = SharedShape::capsule(point_a, point_b, radius);
                    let ragdoll_groups = InteractionGroups::new(Group::GROUP_2, Group::GROUP_1);
                    let collider = ColliderBuilder::new(shape)
                        .friction(0.5)
                        .restitution(0.1)
                        .collision_groups(ragdoll_groups)
                        .build();
                    world
                        .resources
                        .physics
                        .add_collider(collider, rapier_handle);
                }
            } else {
                let ragdoll_groups = InteractionGroups::new(Group::GROUP_2, Group::GROUP_1);
                let collider = ColliderBuilder::ball(self.leaf_radius)
                    .friction(0.5)
                    .restitution(0.1)
                    .collision_groups(ragdoll_groups)
                    .build();
                world
                    .resources
                    .physics
                    .add_collider(collider, rapier_handle);
            }

            entity_to_rapier_handle.insert(bone, rapier_handle);
            let physics_handle: PhysicsHandle = rapier_handle.into();
            all_handles.push(physics_handle);
            bone_body_mappings.push(BoneBodyMapping {
                bone_entity: bone,
                body_handle: physics_handle,
                bone_local_scale: bone_local_scales
                    .get(&bone)
                    .copied()
                    .unwrap_or(Vec3::new(1.0, 1.0, 1.0)),
            });
        }

        for (&child_bone, &parent_bone) in &ragdoll_parent_map {
            let Some(&parent_rapier) = entity_to_rapier_handle.get(&parent_bone) else {
                continue;
            };
            let Some(&child_rapier) = entity_to_rapier_handle.get(&child_bone) else {
                continue;
            };
            let Some(&(parent_pos, parent_rot)) = bone_global_transforms.get(&parent_bone) else {
                continue;
            };
            let Some(&(child_pos, _)) = bone_global_transforms.get(&child_bone) else {
                continue;
            };

            let world_offset = child_pos - parent_pos;
            let inv_parent_rot = nalgebra_glm::quat_conjugate(&parent_rot);
            let local_anchor1 = nalgebra_glm::quat_rotate_vec3(&inv_parent_rot, &world_offset);

            let mut joint: GenericJoint = SphericalJointBuilder::new()
                .local_anchor1(point![local_anchor1.x, local_anchor1.y, local_anchor1.z])
                .local_anchor2(point![0.0, 0.0, 0.0])
                .build()
                .into();
            joint.set_contacts_enabled(false);

            world.resources.physics.impulse_joint_set.insert(
                parent_rapier,
                child_rapier,
                joint,
                true,
            );
        }

        let instance = &mut self.ragdolls[ragdoll_index];
        instance.bone_body_mappings = bone_body_mappings;
        instance.rapier_handles = all_handles;
    }

    fn sync_ragdoll_bones(&self, world: &mut World) {
        for instance in &self.ragdolls {
            if !instance.is_ragdolled {
                continue;
            }

            for mapping in &instance.bone_body_mappings {
                let rapier_handle: rapier3d::prelude::RigidBodyHandle = mapping.body_handle.into();
                let Some(rigid_body) = world.resources.physics.rigid_body_set.get(rapier_handle)
                else {
                    continue;
                };

                let body_translation = rigid_body.translation();
                let body_rotation = rigid_body.rotation();
                let body_pos =
                    Vec3::new(body_translation.x, body_translation.y, body_translation.z);
                let body_rot = nalgebra_glm::quat(
                    body_rotation.i,
                    body_rotation.j,
                    body_rotation.k,
                    body_rotation.w,
                );

                let parent_entity = world
                    .get_parent(mapping.bone_entity)
                    .and_then(|parent| parent.0);

                let (local_pos, local_rot) = if let Some(parent) = parent_entity
                    && let Some(parent_global) = world.get_global_transform(parent)
                {
                    let parent_matrix = parent_global.0;
                    let inv_parent = nalgebra_glm::inverse(&parent_matrix);

                    let world_pos_homogeneous =
                        nalgebra_glm::vec4(body_pos.x, body_pos.y, body_pos.z, 1.0);
                    let local_pos_homogeneous = inv_parent * world_pos_homogeneous;
                    let local_pos = Vec3::new(
                        local_pos_homogeneous.x,
                        local_pos_homogeneous.y,
                        local_pos_homogeneous.z,
                    );

                    let parent_rot = extract_rotation_from_matrix(&parent_matrix);
                    let inv_parent_rot = nalgebra_glm::quat_conjugate(&parent_rot);
                    let local_rot = nalgebra_glm::quat_cross(&inv_parent_rot, &body_rot);

                    (local_pos, local_rot)
                } else {
                    (body_pos, body_rot)
                };

                world.set_local_transform(
                    mapping.bone_entity,
                    LocalTransform {
                        translation: local_pos,
                        rotation: local_rot,
                        scale: mapping.bone_local_scale,
                    },
                );
                world.mark_local_transform_dirty(mapping.bone_entity);
            }
        }
    }

    fn reset(&mut self, world: &mut World) {
        for instance in self.ragdolls.drain(..) {
            for handle in &instance.rapier_handles {
                let rapier_handle: rapier3d::prelude::RigidBodyHandle = (*handle).into();
                world.resources.physics.remove_rigid_body(rapier_handle);
            }

            world.queue_command(WorldCommand::DespawnRecursive {
                entity: instance.root_entity,
            });
        }

        self.spawn_dancer(world, Vec3::zeros());
    }

    fn reset_camera_to_home(&self, world: &mut World) {
        let Some(camera_entity) = self.camera_entity else {
            return;
        };

        let Some(pan_orbit) = world.get_pan_orbit_camera_mut(camera_entity) else {
            return;
        };

        pan_orbit.target_focus = self.home_focus;
        pan_orbit.target_radius = self.home_radius;
        pan_orbit.target_yaw = self.home_yaw;
        pan_orbit.target_pitch = self.home_pitch;
    }
}

fn find_skinned_mesh_entity(world: &World, root: Entity) -> Option<Entity> {
    if world.get_skin(root).is_some() {
        return Some(root);
    }

    let children = query_children(world, root);
    for child in children {
        if let Some(found) = find_skinned_mesh_entity(world, child) {
            return Some(found);
        }
    }

    None
}

fn extract_rotation_from_matrix(matrix: &Mat4) -> Quat {
    let col0 = Vec3::new(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
    let col1 = Vec3::new(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)]);
    let col2 = Vec3::new(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)]);

    let scale_x = col0.norm();
    let scale_y = col1.norm();
    let scale_z = col2.norm();

    let rot_matrix = nalgebra_glm::mat3(
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

    nalgebra_glm::mat3_to_quat(&rot_matrix)
}

fn rand_f32() -> f32 {
    static SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(12345);
    let mut state = SEED.load(std::sync::atomic::Ordering::Relaxed);
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    SEED.store(state, std::sync::atomic::Ordering::Relaxed);
    (state as f32 / u64::MAX as f32).abs()
}
