use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::mesh::components::{MorphTarget, MorphTargetData};
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use nightshade::run::RenderResources;

const MORPH_PRIMITIVES: &[u8] = include_bytes!("../../../assets/models/MorphPrimitivesTest.glb");
const ANIMATED_CUBE: &[u8] = include_bytes!("../../../assets/models/AnimatedMorphCube.glb");
const MORPH_STRESS_TEST: &[u8] = include_bytes!("../../../assets/models/MorphStressTest.glb");
const FOX: &[u8] = include_bytes!("../../../assets/models/fox.glb");
const HDR_BYTES: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(MorphState::default())
}

#[derive(Default)]
struct MorphState {
    primitives_entity: Option<Entity>,
    cube_entity: Option<Entity>,
    stress_test_entity: Option<Entity>,
    fox_entity: Option<Entity>,
    fox_morph_entities: Vec<Entity>,
    camera_entity: Option<Entity>,
    loaded: bool,
    home_focus: Vec3,
    home_radius: f32,
    home_yaw: f32,
    home_pitch: f32,
}

impl State for MorphState {
    fn title(&self) -> &str {
        "Morph Targets Demo"
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
            .read("ssao", resources.ssao)
            .write("output", resources.swapchain);
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.use_fullscreen = true;
        world.resources.graphics.ui_scale = Some(1.0);
        world.resources.graphics.atmosphere = Atmosphere::Hdr;

        load_hdr_skybox(world, HDR_BYTES.to_vec());

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 2.0;
        }

        self.home_focus = Vec3::new(0.0, 0.5, -1.0);
        self.home_radius = 8.0;
        self.home_yaw = 0.0;
        self.home_pitch = 0.4;

        let camera_entity = spawn_pan_orbit_camera(
            world,
            self.home_focus,
            self.home_radius,
            self.home_yaw,
            self.home_pitch,
            "Morph Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);
        self.camera_entity = Some(camera_entity);

        tracing::info!("Loading MorphPrimitivesTest model");
        match nightshade::ecs::prefab::import_gltf_from_bytes(MORPH_PRIMITIVES) {
            Ok(result) => {
                tracing::info!("Successfully loaded MorphPrimitivesTest");
                tracing::info!("Loaded {} meshes", result.meshes.len());
                tracing::info!("Loaded {} animations", result.animations.len());

                for (name, (rgba_data, width, height)) in result.textures {
                    tracing::info!("Loading texture '{}': {}x{}", name, width, height);
                    world.queue_command(WorldCommand::LoadTexture {
                        name,
                        rgba_data,
                        width,
                        height,
                    });
                }

                for (name, mesh) in result.meshes {
                    let has_morphs = mesh.morph_targets.is_some();
                    let morph_count = mesh
                        .morph_targets
                        .as_ref()
                        .map(|mt| mt.targets.len())
                        .unwrap_or(0);
                    tracing::info!(
                        "Mesh '{}': has_morphs={}, morph_count={}",
                        name,
                        has_morphs,
                        morph_count
                    );
                    mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                }

                for prefab in result.prefabs {
                    tracing::info!(
                        "Spawning prefab '{}' with {} animations",
                        prefab.name,
                        result.animations.len()
                    );
                    let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                        world,
                        &prefab,
                        &result.animations,
                        &result.skins,
                        Vec3::new(-2.0, 0.0, 0.0),
                    );

                    self.primitives_entity = Some(entity);
                    tracing::info!("Spawned MorphPrimitivesTest with root entity {:?}", entity);

                    if let Some(player) = world.get_animation_player_mut(entity)
                        && !player.clips.is_empty()
                    {
                        player.play(0);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to load MorphPrimitivesTest: {}", e);
            }
        }

        tracing::info!("Loading AnimatedMorphCube model");
        match nightshade::ecs::prefab::import_gltf_from_bytes(ANIMATED_CUBE) {
            Ok(result) => {
                tracing::info!("Successfully loaded AnimatedMorphCube");
                tracing::info!("Loaded {} meshes", result.meshes.len());
                tracing::info!("Loaded {} animations", result.animations.len());

                for (name, (rgba_data, width, height)) in result.textures {
                    tracing::info!("Loading texture '{}': {}x{}", name, width, height);
                    world.queue_command(WorldCommand::LoadTexture {
                        name,
                        rgba_data,
                        width,
                        height,
                    });
                }

                for (name, mesh) in result.meshes {
                    let has_morphs = mesh.morph_targets.is_some();
                    let morph_count = mesh
                        .morph_targets
                        .as_ref()
                        .map(|mt| mt.targets.len())
                        .unwrap_or(0);
                    tracing::info!(
                        "Mesh '{}': has_morphs={}, morph_count={}",
                        name,
                        has_morphs,
                        morph_count
                    );
                    mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                }

                for prefab in result.prefabs {
                    tracing::info!(
                        "Spawning prefab '{}' with {} animations",
                        prefab.name,
                        result.animations.len()
                    );
                    let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                        world,
                        &prefab,
                        &result.animations,
                        &result.skins,
                        Vec3::new(2.0, 0.0, 0.0),
                    );

                    self.cube_entity = Some(entity);
                    tracing::info!("Spawned AnimatedMorphCube with root entity {:?}", entity);

                    if let Some(player) = world.get_animation_player_mut(entity)
                        && !player.clips.is_empty()
                    {
                        player.play(0);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to load AnimatedMorphCube: {}", e);
            }
        }

        tracing::info!("Loading MorphStressTest model");
        match nightshade::ecs::prefab::import_gltf_from_bytes(MORPH_STRESS_TEST) {
            Ok(result) => {
                tracing::info!("Successfully loaded MorphStressTest");
                tracing::info!("Loaded {} meshes", result.meshes.len());
                tracing::info!("Loaded {} animations", result.animations.len());

                for (name, (rgba_data, width, height)) in result.textures {
                    tracing::info!("Loading texture '{}': {}x{}", name, width, height);
                    world.queue_command(WorldCommand::LoadTexture {
                        name,
                        rgba_data,
                        width,
                        height,
                    });
                }

                for (name, mesh) in result.meshes {
                    let has_morphs = mesh.morph_targets.is_some();
                    let morph_count = mesh
                        .morph_targets
                        .as_ref()
                        .map(|mt| mt.targets.len())
                        .unwrap_or(0);
                    tracing::info!(
                        "Mesh '{}': has_morphs={}, morph_count={}",
                        name,
                        has_morphs,
                        morph_count
                    );
                    mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                }

                for prefab in result.prefabs {
                    tracing::info!(
                        "Spawning prefab '{}' with {} animations",
                        prefab.name,
                        result.animations.len()
                    );
                    let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                        world,
                        &prefab,
                        &result.animations,
                        &result.skins,
                        Vec3::new(0.0, 0.0, -3.0),
                    );

                    self.stress_test_entity = Some(entity);
                    tracing::info!("Spawned MorphStressTest with root entity {:?}", entity);

                    if let Some(player) = world.get_animation_player_mut(entity)
                        && !player.clips.is_empty()
                    {
                        player.play(0);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to load MorphStressTest: {}", e);
            }
        }

        tracing::info!("Loading Fox model (skinned mesh with programmatic morph targets)");
        match nightshade::ecs::prefab::import_gltf_from_bytes(FOX) {
            Ok(result) => {
                tracing::info!("Successfully loaded Fox");
                tracing::info!("Loaded {} meshes", result.meshes.len());
                tracing::info!("Loaded {} animations", result.animations.len());
                tracing::info!("Loaded {} skins", result.skins.len());

                for (name, (rgba_data, width, height)) in result.textures {
                    tracing::info!("Loading texture '{}': {}x{}", name, width, height);
                    world.queue_command(WorldCommand::LoadTexture {
                        name,
                        rgba_data,
                        width,
                        height,
                    });
                }

                let mut fox_mesh_names = Vec::new();
                for (name, mut mesh) in result.meshes {
                    let is_skinned = mesh.skin_data.is_some();
                    tracing::info!(
                        "Fox mesh '{}': is_skinned={}, vertex_count={}",
                        name,
                        is_skinned,
                        mesh.vertices.len()
                    );

                    if is_skinned {
                        let vertex_count = mesh.vertices.len();
                        let base_positions: Vec<[f32; 3]> =
                            mesh.vertices.iter().map(|v| v.position).collect();
                        let base_normals: Vec<[f32; 3]> =
                            mesh.vertices.iter().map(|v| v.normal).collect();

                        let mut scale_displacements = vec![[0.0f32; 3]; vertex_count];
                        let mut squash_displacements = vec![[0.0f32; 3]; vertex_count];

                        for (index, vertex) in mesh.vertices.iter().enumerate() {
                            scale_displacements[index] =
                                [vertex.position[0] * 0.3, 0.0, vertex.position[2] * 0.3];
                            squash_displacements[index] = [0.0, -vertex.position[1] * 0.3, 0.0];
                        }

                        let morph_targets = MorphTargetData::new(vec![
                            MorphTarget::new(scale_displacements),
                            MorphTarget::new(squash_displacements),
                        ])
                        .with_base_data(base_positions, base_normals)
                        .with_default_weights(vec![0.0, 0.0]);

                        mesh.morph_targets = Some(morph_targets);
                        fox_mesh_names.push(name.clone());
                        tracing::info!(
                            "Added 2 programmatic morph targets to skinned mesh '{}'",
                            name
                        );
                    }

                    mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
                }

                for prefab in result.prefabs {
                    tracing::info!(
                        "Spawning Fox prefab '{}' with {} animations",
                        prefab.name,
                        result.animations.len()
                    );
                    let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                        world,
                        &prefab,
                        &result.animations,
                        &result.skins,
                        Vec3::new(0.0, 0.0, 3.0),
                    );

                    self.fox_entity = Some(entity);
                    tracing::info!("Spawned Fox with root entity {:?}", entity);

                    if let Some(transform) = world.mutate_local_transform(entity) {
                        transform.scale = Vec3::new(0.01, 0.01, 0.01);
                    }
                }

                for mesh_name in fox_mesh_names {
                    let morph_entities: Vec<Entity> = world
                        .query_entities(RENDER_MESH)
                        .filter(|entity| {
                            if let Some(render_mesh) = world.get_render_mesh(*entity) {
                                render_mesh.name == mesh_name
                            } else {
                                false
                            }
                        })
                        .collect();

                    for morph_entity in morph_entities {
                        world.add_components(morph_entity, MORPH_WEIGHTS);
                        world.set_morph_weights(
                            morph_entity,
                            MorphWeights::new(vec![0.5, 0.0], mesh_name.clone()),
                        );
                        self.fox_morph_entities.push(morph_entity);
                        tracing::info!(
                            "Added MorphWeights component to entity {:?} for mesh '{}'",
                            morph_entity,
                            mesh_name
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to load Fox: {}", e);
            }
        }

        self.loaded = true;
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        if world.resources.input.keyboard.is_key_pressed(KeyCode::KeyC)
            || world.resources.input.keyboard.is_key_pressed(KeyCode::Home)
        {
            self.reset_camera_to_home(world);
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Morph Targets Demo")
            .default_pos([10.0, 10.0])
            .resizable(true)
            .show(ui_context, |ui| {
                if !self.loaded {
                    ui.label("Loading models...");
                    return;
                }

                ui.heading("MorphPrimitivesTest");
                ui.separator();
                self.show_model_controls(world, ui, self.primitives_entity, "primitives");

                ui.add_space(16.0);

                ui.heading("AnimatedMorphCube");
                ui.separator();
                self.show_model_controls(world, ui, self.cube_entity, "cube");

                ui.add_space(16.0);

                ui.heading("MorphStressTest");
                ui.separator();
                self.show_model_controls(world, ui, self.stress_test_entity, "stress");

                ui.add_space(16.0);

                ui.heading("Fox (Skinned + Morph)");
                ui.separator();
                self.show_fox_controls(world, ui);

                ui.add_space(16.0);
                ui.separator();

                ui.label("Controls:");
                ui.label("  Mouse drag - Orbit camera");
                ui.label("  Scroll - Zoom");
                ui.label("  C / Home - Reset camera");
                ui.label("  Escape - Exit");
            });
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        if matches!(key, KeyCode::KeyC | KeyCode::Home) {
            self.reset_camera_to_home(world);
        }
    }
}

impl MorphState {
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

    fn show_model_controls(
        &self,
        world: &mut World,
        ui: &mut egui::Ui,
        entity: Option<Entity>,
        id_suffix: &str,
    ) {
        let Some(entity) = entity else {
            ui.label("Model not loaded");
            return;
        };

        if let Some(player) = world.get_animation_player_mut(entity) {
            if player.clips.is_empty() {
                ui.label("No animations");
            } else {
                ui.label(format!("{} animation(s)", player.clips.len()));

                let mut clip_to_play = None;

                ui.horizontal(|ui| {
                    ui.label("Animation:");
                    egui::ComboBox::from_id_salt(format!("animation_{}", id_suffix))
                        .width(200.0)
                        .selected_text(
                            player
                                .current_clip
                                .and_then(|index| player.clips.get(index))
                                .map(|clip| clip.name.as_str())
                                .unwrap_or("None"),
                        )
                        .show_ui(ui, |ui| {
                            for (index, clip) in player.clips.iter().enumerate() {
                                let is_selected = player.current_clip == Some(index);
                                let label = format!("{} ({:.2}s)", clip.name, clip.duration);
                                if ui.selectable_label(is_selected, label).clicked() {
                                    clip_to_play = Some(index);
                                }
                            }
                        });
                });

                if let Some(index) = clip_to_play {
                    player.play(index);
                }

                if let Some(clip_index) = player.current_clip
                    && let Some(clip) = player.clips.get(clip_index)
                {
                    let progress = player.time / clip.duration;
                    ui.add(
                        egui::ProgressBar::new(progress)
                            .text(format!("{:.2}s / {:.2}s", player.time, clip.duration)),
                    );
                }

                ui.horizontal(|ui| {
                    ui.label("Speed:");
                    ui.add(egui::Slider::new(&mut player.speed, 0.0..=2.0).suffix("x"));
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut player.looping, "Loop");
                    if ui
                        .button(if player.playing { "Pause" } else { "Play" })
                        .clicked()
                    {
                        if player.playing {
                            player.pause();
                        } else {
                            player.resume();
                        }
                    }
                });
            }
        } else {
            ui.label("No animation player");
        }

        self.show_morph_weight_controls(world, ui, entity, id_suffix);
    }

    fn show_morph_weight_controls(
        &self,
        world: &mut World,
        ui: &mut egui::Ui,
        root_entity: Entity,
        _id_suffix: &str,
    ) {
        let entities_with_morphs: Vec<Entity> = world
            .query_entities(MORPH_WEIGHTS)
            .filter(|entity| Self::is_descendant_of(world, *entity, root_entity))
            .collect();

        if entities_with_morphs.is_empty() {
            return;
        }

        ui.add_space(8.0);
        ui.label(format!(
            "{} mesh(es) with morph targets",
            entities_with_morphs.len()
        ));

        for (entity_index, morph_entity) in entities_with_morphs.iter().enumerate() {
            if let Some(morph_weights) = world.get_morph_weights_mut(*morph_entity) {
                let weight_count = morph_weights.weights.len();
                if weight_count > 0 {
                    ui.collapsing(
                        format!(
                            "Morph weights ({} targets) [{}]",
                            weight_count, entity_index
                        ),
                        |ui| {
                            for (weight_index, weight) in
                                morph_weights.weights.iter_mut().enumerate()
                            {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Target {}:", weight_index));
                                    ui.add(
                                        egui::Slider::new(weight, 0.0..=1.0)
                                            .clamping(egui::SliderClamping::Always)
                                            .fixed_decimals(2),
                                    );
                                });
                            }
                        },
                    );
                }
            }
        }
    }

    fn is_descendant_of(world: &World, entity: Entity, ancestor: Entity) -> bool {
        if entity == ancestor {
            return true;
        }

        if let Some(parent) = world.get_parent(entity)
            && let Some(parent_entity) = parent.0
        {
            return Self::is_descendant_of(world, parent_entity, ancestor);
        }

        false
    }

    fn show_fox_controls(&self, world: &mut World, ui: &mut egui::Ui) {
        if self.fox_entity.is_none() {
            ui.label("Model not loaded");
            return;
        }

        ui.label("Skinned mesh with programmatic morph targets");

        if self.fox_morph_entities.is_empty() {
            ui.label("No morph targets found");
            return;
        }

        ui.add_space(8.0);
        ui.label(format!(
            "{} mesh(es) with morph targets",
            self.fox_morph_entities.len()
        ));

        for (entity_index, &morph_entity) in self.fox_morph_entities.iter().enumerate() {
            if let Some(morph_weights) = world.get_morph_weights_mut(morph_entity) {
                let weight_count = morph_weights.weights.len();
                if weight_count > 0 {
                    ui.horizontal(|ui| {
                        ui.label("Target 0 (Scale X/Z):");
                        if let Some(weight) = morph_weights.weights.get_mut(0) {
                            ui.add(
                                egui::Slider::new(weight, 0.0..=1.0)
                                    .clamping(egui::SliderClamping::Always)
                                    .fixed_decimals(2),
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Target 1 (Squash Y):");
                        if let Some(weight) = morph_weights.weights.get_mut(1) {
                            ui.add(
                                egui::Slider::new(weight, 0.0..=1.0)
                                    .clamping(egui::SliderClamping::Always)
                                    .fixed_decimals(2),
                            );
                        }
                    });
                    let _ = entity_index;
                }
            }
        }
    }
}
