use std::collections::HashMap;

use nightshade::ecs::world::WorldCommand;
use nightshade::prelude::*;
use noise::Perlin;
use rand::{Rng, SeedableRng};

use crate::building::{
    BuildingType, describe_building_body, describe_building_detail, proxy_material_position_scale,
};
use crate::city::{self, CHUNK_SIZE, CityChunkLayout, generate_chunk_layout};
use crate::descriptors::ChunkData;
use crate::waterfront::{self, EdgeDirection};

const LOD0_RADIUS: i32 = 3;
const LOD1_RADIUS: i32 = 6;
const BASE_RADIUS: i32 = LOD1_RADIUS + 2;
const FRUSTUM_BIAS_AHEAD: i32 = 2;
const FRUSTUM_BIAS_BEHIND: i32 = -2;
const MAX_MANAGEMENT_RADIUS: i32 = BASE_RADIUS + FRUSTUM_BIAS_AHEAD + 2;

const COMBINED_BUDGET: usize = 320;

const LAMP_POLE_HEIGHT: f32 = 3.5;
const LAMP_GLOBE_RADIUS: f32 = 0.25;

const SMOKE_PROBABILITY: f32 = 0.40;

const FADE_DURATION: f32 = 0.3;
const MAX_PREGEN_PER_FRAME: usize = 4;
const MAX_PREGEN_CACHE: usize = 800;
const MAX_LAYOUT_CACHE: usize = 2000;
const LAYOUT_EVICT_BATCH: usize = 64;

const PROXY_RADIUS: i32 = 16;
const PROXY_REBUILD_THRESHOLD: i32 = 6;

const SEED_BODY: u64 = 10000;
const SEED_DETAIL: u64 = 20000;
const SEED_WATERFRONT: u64 = 12345;
const SEED_WATERFRONT_DETAIL: u64 = 12346;
const SEED_VEHICLES: u64 = 54321;
const SEED_CRANES: u64 = 91919;

const CAR_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const CAR_MIN_SIDEWALK: f32 = 4.0;

const MARKING_LENGTH: f32 = 2.0;
const MARKING_WIDTH: f32 = 0.3;
const MARKING_HEIGHT: f32 = 0.1;
const MARKING_GAP: f32 = 3.0;

const CRANE_PROBABILITY: f32 = 0.30;
const CRANE_TOWER_HEIGHT: f32 = 25.0;
const CRANE_TOWER_RADIUS: f32 = 0.6;
const CRANE_ARM_LENGTH: f32 = 15.0;
const CRANE_ARM_THICKNESS: f32 = 0.5;
const CRANE_COUNTERWEIGHT_SIZE: f32 = 2.0;

const BOAT_WATER_Y: f32 = -1.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LayerKind {
    Base,
    Buildings,
    Detail,
}

struct BoatInfo {
    entity: Entity,
    initial_yaw: f32,
}

struct ChunkState {
    base_entities: Vec<Entity>,
    building_entities: Vec<Entity>,
    detail_entities: Vec<Entity>,
    boats: Vec<BoatInfo>,
}

struct LoadingCursor {
    cursor: usize,
    entities: Vec<Entity>,
}

enum FadeDirection {
    In,
    Out,
}

struct FadeState {
    alpha: f32,
    direction: FadeDirection,
}

struct PregenChunk {
    base: ChunkData,
    buildings: ChunkData,
    detail: ChunkData,
}

pub struct ChunkStreamer {
    city_min: i32,
    city_max: i32,
    noise: Perlin,
    chunks: HashMap<(i32, i32), ChunkState>,
    loading: HashMap<((i32, i32), LayerKind), LoadingCursor>,
    pregen: HashMap<(i32, i32), PregenChunk>,
    layouts: HashMap<(i32, i32), city::CityChunkLayout>,
    proxy_instanced_entities: Vec<Entity>,
    proxy_center: (i32, i32),
    proxies_initialized: bool,
    last_camera_chunk: (i32, i32),
    fading_entities: HashMap<Entity, FadeState>,
    cached_entity_count: usize,
    pregen_budget: usize,
    pending_pregen: bool,
    scratch_to_cancel: Vec<((i32, i32), LayerKind)>,
    scratch_loading_keys: Vec<((i32, i32), LayerKind)>,
    scratch_completed_loading: Vec<((i32, i32), LayerKind)>,
    scratch_chunks_to_unload: Vec<(i32, i32)>,
}

fn chunk_seed(coords: (i32, i32)) -> u64 {
    (coords.0 as u64).wrapping_mul(73856093) ^ (coords.1 as u64).wrapping_mul(19349663)
}

impl ChunkStreamer {
    pub fn new(city_half: i32, seed: u32) -> Self {
        let city_min = -city_half;
        let city_max = city_half - 1;

        Self {
            city_min,
            city_max,
            noise: Perlin::new(seed),
            chunks: HashMap::new(),
            loading: HashMap::new(),
            pregen: HashMap::new(),
            layouts: HashMap::new(),
            proxy_instanced_entities: Vec::new(),
            proxy_center: (i32::MAX, i32::MAX),
            proxies_initialized: false,
            last_camera_chunk: (i32::MAX, i32::MAX),
            fading_entities: HashMap::new(),
            cached_entity_count: 0,
            pregen_budget: MAX_PREGEN_PER_FRAME,
            pending_pregen: false,
            scratch_to_cancel: Vec::new(),
            scratch_loading_keys: Vec::new(),
            scratch_completed_loading: Vec::new(),
            scratch_chunks_to_unload: Vec::new(),
        }
    }

    pub fn city_min(&self) -> i32 {
        self.city_min
    }

    pub fn city_max(&self) -> i32 {
        self.city_max
    }

    fn ensure_layout(&mut self, coords: (i32, i32)) {
        if !self.layouts.contains_key(&coords) {
            self.layouts.insert(
                coords,
                generate_chunk_layout(coords.0, coords.1, &self.noise),
            );
        }
    }

    fn evict_distant_layouts(&mut self) {
        if self.layouts.len() <= MAX_LAYOUT_CACHE {
            return;
        }
        let camera_chunk = self.last_camera_chunk;
        let mut evictable: Vec<(i32, i32)> = self
            .layouts
            .keys()
            .copied()
            .filter(|coords| {
                !self.chunks.contains_key(coords)
                    && !self.loading.keys().any(|key| key.0 == *coords)
                    && !self.pregen.contains_key(coords)
            })
            .collect();
        evictable.sort_by(|a, b| {
            let dist_a = chebyshev_distance(*a, camera_chunk);
            let dist_b = chebyshev_distance(*b, camera_chunk);
            dist_b.cmp(&dist_a)
        });
        let to_remove = evictable.len().min(LAYOUT_EVICT_BATCH);
        for coords in &evictable[..to_remove] {
            self.layouts.remove(coords);
        }
    }

    fn ensure_pregen(&mut self, coords: (i32, i32)) -> bool {
        if self.pregen.contains_key(&coords) {
            return true;
        }
        if self.pregen_budget == 0 {
            return false;
        }
        self.evict_farthest_pregen();
        self.ensure_layout(coords);
        let layout = &self.layouts[&coords];
        self.pregen.insert(
            coords,
            pregen_chunk(layout, coords, self.city_min, self.city_max),
        );
        self.pregen_budget -= 1;
        true
    }

    fn evict_farthest_pregen(&mut self) {
        if self.pregen.len() <= MAX_PREGEN_CACHE {
            return;
        }
        let camera_chunk = self.last_camera_chunk;
        let mut evictable: Vec<(i32, i32)> = self
            .pregen
            .keys()
            .copied()
            .filter(|coords| {
                !self.chunks.contains_key(coords)
                    && !self.loading.keys().any(|key| key.0 == *coords)
            })
            .collect();
        evictable.sort_by(|a, b| {
            let dist_a = chebyshev_distance(*a, camera_chunk);
            let dist_b = chebyshev_distance(*b, camera_chunk);
            dist_b.cmp(&dist_a)
        });
        let to_remove = evictable.len().min(16);
        for coords in &evictable[..to_remove] {
            self.pregen.remove(coords);
        }
    }

    pub fn is_ready(&self) -> bool {
        self.proxies_initialized
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn layouts(&self) -> &HashMap<(i32, i32), city::CityChunkLayout> {
        &self.layouts
    }

    pub fn despawn_all(&mut self, world: &mut World) {
        for (_, chunk) in self.chunks.drain() {
            for entity in chunk
                .base_entities
                .iter()
                .chain(chunk.building_entities.iter())
                .chain(chunk.detail_entities.iter())
            {
                world.queue_command(WorldCommand::DespawnRecursive { entity: *entity });
            }
        }

        for entity in self.proxy_instanced_entities.drain(..) {
            world.queue_command(WorldCommand::DespawnRecursive { entity });
        }

        for (_, loading) in self.loading.drain() {
            for entity in &loading.entities {
                world.queue_command(WorldCommand::DespawnRecursive { entity: *entity });
            }
        }

        for (entity, fade) in self.fading_entities.drain() {
            if matches!(fade.direction, FadeDirection::Out) {
                world.queue_command(WorldCommand::DespawnRecursive { entity });
            }
        }
        self.pregen.clear();
        self.layouts.clear();
        self.proxy_center = (i32::MAX, i32::MAX);
        self.proxies_initialized = false;
        self.cached_entity_count = 0;
    }

    pub fn entity_count(&self) -> usize {
        self.cached_entity_count
    }

    fn rebuild_proxies(&mut self, world: &mut World, center: (i32, i32)) {
        for entity in self.proxy_instanced_entities.drain(..) {
            world.queue_command(WorldCommand::DespawnRecursive { entity });
            self.cached_entity_count -= 1;
        }

        let proxy_min_x = (center.0 - PROXY_RADIUS).max(self.city_min);
        let proxy_max_x = (center.0 + PROXY_RADIUS).min(self.city_max);
        let proxy_min_z = (center.1 - PROXY_RADIUS).max(self.city_min);
        let proxy_max_z = (center.1 + PROXY_RADIUS).min(self.city_max);

        let mut groups: HashMap<&'static str, Vec<InstanceTransform>> = HashMap::new();
        let mut ground_instances = Vec::new();
        for x in proxy_min_x..=proxy_max_x {
            for z in proxy_min_z..=proxy_max_z {
                self.ensure_layout((x, z));
                let layout = &self.layouts[&(x, z)];
                for spec in &layout.buildings {
                    if crate::interiors::building_has_interior(spec) {
                        continue;
                    }
                    let (material, position, scale) = proxy_material_position_scale(spec);
                    groups
                        .entry(material)
                        .or_default()
                        .push(InstanceTransform::from_translation_scale(position, scale));
                }

                let chunk_base_x = x as f32 * CHUNK_SIZE;
                let chunk_base_z = z as f32 * CHUNK_SIZE;
                ground_instances.push(InstanceTransform::from_translation_scale(
                    Vec3::new(
                        chunk_base_x + CHUNK_SIZE / 2.0,
                        -0.15,
                        chunk_base_z + CHUNK_SIZE / 2.0,
                    ),
                    Vec3::new(CHUNK_SIZE, 0.1, CHUNK_SIZE),
                ));
            }
        }
        for (material, instances) in groups {
            let entity = spawn_instanced_mesh_with_material(world, "Cube", instances, material);
            self.proxy_instanced_entities.push(entity);
        }
        if !ground_instances.is_empty() {
            let entity =
                spawn_instanced_mesh_with_material(world, "Cube", ground_instances, "Ground");
            world.core.remove_casts_shadow(entity);
            self.proxy_instanced_entities.push(entity);
        }
        world
            .resources
            .mesh_render_state
            .mark_instanced_meshes_changed();
        self.cached_entity_count += self.proxy_instanced_entities.len();
        self.proxy_center = center;
        self.proxies_initialized = true;
    }

    fn advance_fades(&mut self, world: &mut World, delta_time: f32) {
        let fade_speed = 1.0 / FADE_DURATION;
        let mut completed: Vec<Entity> = Vec::new();
        for (&entity, fade) in self.fading_entities.iter_mut() {
            match fade.direction {
                FadeDirection::In => {
                    fade.alpha = (fade.alpha + delta_time * fade_speed).min(1.0);
                    if fade.alpha >= 1.0 {
                        world
                            .resources
                            .mesh_render_state
                            .mark_entity_fade_complete(entity);
                        completed.push(entity);
                    } else {
                        world
                            .resources
                            .mesh_render_state
                            .set_entity_custom_data(entity, [1.0, 1.0, 1.0, fade.alpha]);
                    }
                }
                FadeDirection::Out => {
                    fade.alpha = (fade.alpha - delta_time * fade_speed).max(0.0);
                    if fade.alpha <= 0.0 {
                        world
                            .resources
                            .mesh_render_state
                            .mark_entity_fade_complete(entity);
                        world.queue_command(WorldCommand::DespawnRecursive { entity });
                        self.cached_entity_count -= 1;
                        completed.push(entity);
                    } else {
                        world
                            .resources
                            .mesh_render_state
                            .set_entity_custom_data(entity, [1.0, 1.0, 1.0, fade.alpha]);
                    }
                }
            }
        }
        for entity in completed {
            self.fading_entities.remove(&entity);
        }
    }

    fn begin_unload_entities(&mut self, world: &mut World, entities: Vec<Entity>) {
        let mut had_instanced = false;
        for entity in entities {
            if let Some(existing) = self.fading_entities.get_mut(&entity) {
                existing.direction = FadeDirection::Out;
                continue;
            }

            if world.core.entity_has_components(entity, RENDER_MESH)
                && !world.core.entity_has_components(entity, INSTANCED_MESH)
            {
                self.fading_entities.insert(
                    entity,
                    FadeState {
                        alpha: 1.0,
                        direction: FadeDirection::Out,
                    },
                );
            } else {
                if world.core.entity_has_components(entity, INSTANCED_MESH) {
                    had_instanced = true;
                }
                world.queue_command(WorldCommand::DespawnRecursive { entity });
                self.cached_entity_count -= 1;
            }
        }
        if had_instanced {
            world
                .resources
                .mesh_render_state
                .mark_instanced_meshes_changed();
        }
    }

    pub fn update_boat_bobbing(&self, world: &mut World, time: f32) {
        for chunk in self.chunks.values() {
            for boat in &chunk.boats {
                let Some(transform) = world.core.get_local_transform_mut(boat.entity) else {
                    continue;
                };
                let phase = boat.entity.id as f32 * 1.7;
                let bob_offset = (time * 0.8 + phase).sin() * 0.15;
                transform.translation.y = BOAT_WATER_Y + bob_offset;

                let yaw = nalgebra_glm::quat_angle_axis(boat.initial_yaw, &Vec3::y());
                let roll =
                    nalgebra_glm::quat_angle_axis((time * 0.6 + phase).sin() * 0.03, &Vec3::z());
                let pitch = nalgebra_glm::quat_angle_axis(
                    (time * 0.5 + phase * 1.3).sin() * 0.02,
                    &Vec3::x(),
                );
                transform.rotation = yaw * pitch * roll;
                world.mark_local_transform_dirty(boat.entity);
            }
        }
    }

    pub fn update(&mut self, world: &mut World, camera_pos: Vec3, camera_forward: Vec3) {
        let camera_chunk = (
            (camera_pos.x / CHUNK_SIZE).floor() as i32,
            (camera_pos.z / CHUNK_SIZE).floor() as i32,
        );

        let camera_changed = camera_chunk != self.last_camera_chunk;
        if camera_changed {
            self.last_camera_chunk = camera_chunk;
        }

        let needs_proxy_rebuild = !self.proxies_initialized
            || chebyshev_distance(camera_chunk, self.proxy_center) >= PROXY_REBUILD_THRESHOLD;
        if needs_proxy_rebuild {
            self.rebuild_proxies(world, camera_chunk);
        }

        if !camera_changed
            && self.loading.is_empty()
            && self.fading_entities.is_empty()
            && !self.pending_pregen
            && !needs_proxy_rebuild
        {
            return;
        }

        self.pending_pregen = false;
        self.pregen_budget = MAX_PREGEN_PER_FRAME;

        let delta_time = world.resources.window.timing.delta_time;
        self.advance_fades(world, delta_time);

        let camera_forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z);
        let forward_len = nalgebra_glm::length(&camera_forward_xz);
        let camera_forward_normalized = if forward_len > 0.001 {
            camera_forward_xz / forward_len
        } else {
            Vec3::new(0.0, 0.0, -1.0)
        };

        self.scratch_to_cancel.clear();
        for &(coords, kind) in self.loading.keys() {
            let distance = chebyshev_distance(coords, camera_chunk);
            let should_cancel = match kind {
                LayerKind::Base => !desired_base(distance, true, 0),
                LayerKind::Buildings => {
                    let (want, _) = desired_layers(distance, true, true, 0);
                    !want
                }
                LayerKind::Detail => {
                    let (_, want) = desired_layers(distance, true, true, 0);
                    !want
                }
            };
            if should_cancel {
                self.scratch_to_cancel.push((coords, kind));
            }
        }
        for index in 0..self.scratch_to_cancel.len() {
            let key = self.scratch_to_cancel[index];
            if let Some(loading) = self.loading.remove(&key)
                && !loading.entities.is_empty()
            {
                self.begin_unload_entities(world, loading.entities);
            }
            if key.1 == LayerKind::Base {
                self.chunks.remove(&key.0);
            }
        }

        self.scratch_loading_keys.clear();
        self.scratch_loading_keys
            .extend(self.loading.keys().copied());
        self.scratch_loading_keys.sort_by_key(|&(coords, _)| {
            let distance = chebyshev_distance(coords, camera_chunk);
            let chunk_center = Vec3::new(
                (coords.0 as f32 + 0.5) * CHUNK_SIZE,
                0.0,
                (coords.1 as f32 + 0.5) * CHUNK_SIZE,
            );
            let to_chunk = chunk_center - Vec3::new(camera_pos.x, 0.0, camera_pos.z);
            let to_chunk_len = nalgebra_glm::length(&to_chunk);
            let dot = if to_chunk_len > 0.001 {
                nalgebra_glm::dot(&(to_chunk / to_chunk_len), &camera_forward_normalized)
            } else {
                1.0
            };
            (distance * 1000) - (dot * 500.0) as i32
        });

        let active_count = self
            .scratch_loading_keys
            .iter()
            .filter(|key| {
                let lc = &self.loading[key];
                let pregen = &self.pregen[&key.0];
                let total = match key.1 {
                    LayerKind::Base => pregen.base.total_count(),
                    LayerKind::Buildings => pregen.buildings.total_count(),
                    LayerKind::Detail => pregen.detail.total_count(),
                };
                lc.cursor < total
            })
            .count()
            .max(1);

        let per_chunk_budget = (COMBINED_BUDGET / active_count).max(1);
        let mut spawn_budget = COMBINED_BUDGET;

        self.scratch_completed_loading.clear();

        for key_index in 0..self.scratch_loading_keys.len() {
            let key = self.scratch_loading_keys[key_index];
            if spawn_budget == 0 {
                break;
            }

            let pregen = &self.pregen[&key.0];
            let lc = self.loading.get_mut(&key).unwrap();

            let data = match key.1 {
                LayerKind::Base => &pregen.base,
                LayerKind::Buildings => &pregen.buildings,
                LayerKind::Detail => &pregen.detail,
            };

            let total = data.total_count();
            if lc.cursor >= total {
                self.scratch_completed_loading.push(key);
                continue;
            }

            let chunk_allowance = per_chunk_budget.min(spawn_budget);
            let cursor_before = lc.cursor;
            let (regular_mesh_count, instance_group_count) =
                data.mesh_and_instance_counts_in_range(cursor_before, chunk_allowance);
            let renderable_count = regular_mesh_count + instance_group_count;
            let new_entities = data.instantiate_range(world, cursor_before, chunk_allowance);
            let spawned = new_entities.len();
            let mut has_instance_groups = false;
            for (offset, &entity) in new_entities.iter().enumerate() {
                let is_instance_group = offset >= regular_mesh_count && offset < renderable_count;
                if is_instance_group {
                    has_instance_groups = true;
                } else if offset < regular_mesh_count {
                    if key.1 == LayerKind::Detail {
                        world
                            .resources
                            .mesh_render_state
                            .set_entity_custom_data(entity, [1.0, 1.0, 1.0, 0.0]);
                        world.resources.mesh_render_state.mark_entity_added(entity);
                        self.fading_entities.insert(
                            entity,
                            FadeState {
                                alpha: 0.0,
                                direction: FadeDirection::In,
                            },
                        );
                    } else {
                        world.resources.mesh_render_state.mark_entity_added(entity);
                    }
                }
            }
            if has_instance_groups {
                world
                    .resources
                    .mesh_render_state
                    .mark_instanced_meshes_changed();
            }
            lc.entities.extend(new_entities);
            lc.cursor += spawned;
            spawn_budget -= spawned;
            self.cached_entity_count += spawned;

            if lc.cursor >= total {
                self.scratch_completed_loading.push(key);
            }
        }

        for index in 0..self.scratch_completed_loading.len() {
            let key = self.scratch_completed_loading[index];
            let lc = self.loading.remove(&key).unwrap();
            match key.1 {
                LayerKind::Base => {
                    let chunk = self.chunks.get_mut(&key.0).unwrap();
                    chunk.base_entities = lc.entities;
                }
                LayerKind::Buildings => {
                    let chunk = self.chunks.get_mut(&key.0).unwrap();
                    chunk.building_entities = lc.entities;
                }
                LayerKind::Detail => {
                    let chunk = self.chunks.get_mut(&key.0).unwrap();
                    let mut boats = Vec::new();
                    for &entity in &lc.entities {
                        if let Some(render_mesh) = world.core.get_render_mesh(entity) {
                            let mesh_name = render_mesh.name.as_str();
                            if crate::kenney::BOAT_MODELS.contains(&mesh_name) {
                                let initial_yaw = world
                                    .core.get_local_transform(entity)
                                    .map(|t| {
                                        let euler = nalgebra_glm::quat_euler_angles(&t.rotation);
                                        euler.z
                                    })
                                    .unwrap_or(0.0);
                                boats.push(BoatInfo {
                                    entity,
                                    initial_yaw,
                                });
                            }
                        }
                    }
                    chunk.boats = boats;
                    chunk.detail_entities = lc.entities;
                }
            }
        }

        let scan_min_x = (camera_chunk.0 - MAX_MANAGEMENT_RADIUS).max(self.city_min);
        let scan_max_x = (camera_chunk.0 + MAX_MANAGEMENT_RADIUS).min(self.city_max);
        let scan_min_z = (camera_chunk.1 - MAX_MANAGEMENT_RADIUS).max(self.city_min);
        let scan_max_z = (camera_chunk.1 + MAX_MANAGEMENT_RADIUS).min(self.city_max);

        for x in scan_min_x..=scan_max_x {
            for z in scan_min_z..=scan_max_z {
                let coords = (x, z);
                let distance = chebyshev_distance(coords, camera_chunk);

                let bias = facing_bias(camera_chunk, coords, &camera_forward_normalized);
                let has_base = self
                    .chunks
                    .get(&coords)
                    .is_some_and(|chunk| !chunk.base_entities.is_empty());
                let want_base = desired_base(distance, has_base, bias);

                if want_base
                    && !has_base
                    && !self.chunks.contains_key(&coords)
                    && !self.loading.contains_key(&(coords, LayerKind::Base))
                {
                    if self.ensure_pregen(coords) {
                        self.chunks.insert(
                            coords,
                            ChunkState {
                                base_entities: Vec::new(),
                                building_entities: Vec::new(),
                                detail_entities: Vec::new(),
                                boats: Vec::new(),
                            },
                        );
                        self.loading.insert(
                            (coords, LayerKind::Base),
                            LoadingCursor {
                                cursor: 0,
                                entities: Vec::new(),
                            },
                        );
                    } else {
                        self.pending_pregen = true;
                    }
                }

                if !has_base {
                    continue;
                }

                let chunk = self.chunks.get_mut(&coords).unwrap();
                let has_buildings = !chunk.building_entities.is_empty();
                let has_detail = !chunk.detail_entities.is_empty();
                let (want_buildings, want_detail) =
                    desired_layers(distance, has_buildings, has_detail, bias);

                let mut detail_to_unload = Vec::new();
                let mut buildings_to_unload = Vec::new();

                if !want_detail && has_detail {
                    detail_to_unload = std::mem::take(&mut chunk.detail_entities);
                }

                if !want_buildings && has_buildings {
                    if has_detail && detail_to_unload.is_empty() {
                        detail_to_unload = std::mem::take(&mut chunk.detail_entities);
                    }
                    buildings_to_unload = std::mem::take(&mut chunk.building_entities);
                }

                if want_buildings
                    && !has_buildings
                    && !self.loading.contains_key(&(coords, LayerKind::Buildings))
                {
                    self.loading.insert(
                        (coords, LayerKind::Buildings),
                        LoadingCursor {
                            cursor: 0,
                            entities: Vec::new(),
                        },
                    );
                }

                if want_detail
                    && has_buildings
                    && !has_detail
                    && !self.loading.contains_key(&(coords, LayerKind::Detail))
                {
                    self.loading.insert(
                        (coords, LayerKind::Detail),
                        LoadingCursor {
                            cursor: 0,
                            entities: Vec::new(),
                        },
                    );
                }

                if !detail_to_unload.is_empty() {
                    self.begin_unload_entities(world, detail_to_unload);
                }
                if !buildings_to_unload.is_empty() {
                    self.begin_unload_entities(world, buildings_to_unload);
                }
            }
        }

        self.scratch_chunks_to_unload.clear();
        for &coords in self.chunks.keys() {
            let distance = chebyshev_distance(coords, camera_chunk);
            let bias = facing_bias(camera_chunk, coords, &camera_forward_normalized);
            let has_base = !self.chunks[&coords].base_entities.is_empty();
            if !desired_base(distance, has_base, bias) && has_base {
                self.scratch_chunks_to_unload.push(coords);
            }
        }
        for index in 0..self.scratch_chunks_to_unload.len() {
            let coords = self.scratch_chunks_to_unload[index];
            for kind in [LayerKind::Buildings, LayerKind::Detail] {
                if let Some(loading) = self.loading.remove(&(coords, kind))
                    && !loading.entities.is_empty()
                {
                    self.begin_unload_entities(world, loading.entities);
                }
            }
            if let Some(chunk) = self.chunks.remove(&coords) {
                let mut all_entities = chunk.base_entities;
                all_entities.extend(chunk.building_entities);
                all_entities.extend(chunk.detail_entities);
                if !all_entities.is_empty() {
                    self.begin_unload_entities(world, all_entities);
                }
            }
        }

        self.evict_distant_layouts();
    }
}

fn desired_base(distance: i32, has_base: bool, bias: i32) -> bool {
    if has_base {
        distance <= BASE_RADIUS + FRUSTUM_BIAS_AHEAD + 2
    } else {
        distance <= BASE_RADIUS + bias
    }
}

fn desired_layers(distance: i32, has_buildings: bool, has_detail: bool, bias: i32) -> (bool, bool) {
    let want_buildings = if has_buildings {
        distance <= LOD1_RADIUS + FRUSTUM_BIAS_AHEAD + 2
    } else {
        distance <= LOD1_RADIUS + bias
    };

    let want_detail = if has_detail {
        distance <= LOD0_RADIUS + FRUSTUM_BIAS_AHEAD + 2
    } else {
        distance <= LOD0_RADIUS + bias
    };

    (want_buildings, want_detail && want_buildings)
}

fn pregen_chunk(
    layout: &CityChunkLayout,
    coords: (i32, i32),
    city_min: i32,
    city_max: i32,
) -> PregenChunk {
    let seed = chunk_seed(coords);
    let edges = edge_directions(coords, city_min, city_max);
    let bridge_edges = corner_bridge_edges(coords, city_min, city_max);

    PregenChunk {
        base: pregen_base(layout, coords, &edges),
        buildings: pregen_buildings(layout, coords, seed, &edges, &bridge_edges),
        detail: pregen_detail(layout, coords, seed, &edges),
    }
}

fn pregen_base(
    layout: &CityChunkLayout,
    coords: (i32, i32),
    edges: &[Option<EdgeDirection>; 4],
) -> ChunkData {
    let mut data = ChunkData::default();
    let chunk_base_x = coords.0 as f32 * CHUNK_SIZE;
    let chunk_base_z = coords.1 as f32 * CHUNK_SIZE;

    data.instance(
        "Cube",
        Vec3::new(
            chunk_base_x + CHUNK_SIZE / 2.0,
            -0.1,
            chunk_base_z + CHUNK_SIZE / 2.0,
        ),
        Vec3::new(CHUNK_SIZE, 0.1, CHUNK_SIZE),
        "Ground",
        None,
    );

    for segment in &layout.road_segments {
        let material = if segment.is_sidewalk {
            "Sidewalk"
        } else {
            "Road"
        };
        data.instance(
            "Cube",
            Vec3::new(segment.x, 0.02, segment.z),
            Vec3::new(segment.width, 0.04, segment.depth),
            material,
            None,
        );
    }

    for edge in edges.iter().flatten() {
        waterfront::describe_dock_base(&mut data, coords, *edge);
    }

    data
}

fn pregen_buildings(
    layout: &CityChunkLayout,
    coords: (i32, i32),
    seed: u64,
    edges: &[Option<EdgeDirection>; 4],
    bridge_edges: &Option<(EdgeDirection, EdgeDirection)>,
) -> ChunkData {
    let mut data = ChunkData::default();

    for (building_index, spec) in layout.buildings.iter().enumerate() {
        let building_seed = seed
            .wrapping_add(SEED_BODY)
            .wrapping_add(building_index as u64 * 31337);
        let mut body_rng = rand::rngs::StdRng::seed_from_u64(building_seed);
        describe_building_body(&mut data, spec, &mut body_rng);
    }

    for streetlight in &layout.streetlight_positions {
        describe_streetlight_mesh(&mut data, streetlight.x, streetlight.z);
    }

    describe_road_markings(&mut data, &layout.road_segments);

    let mut crane_rng = rand::rngs::StdRng::seed_from_u64(seed.wrapping_add(SEED_CRANES));
    describe_construction_cranes(&mut data, layout, &mut crane_rng);

    describe_traffic_lights(&mut data, layout);

    let mut waterfront_rng = rand::rngs::StdRng::seed_from_u64(seed.wrapping_add(SEED_WATERFRONT));
    for edge in edges.iter().flatten() {
        waterfront::describe_dock_buildings(&mut data, coords, *edge, &mut waterfront_rng);
    }
    if let Some((edge_a, edge_b)) = bridge_edges {
        waterfront::describe_bridge(&mut data, coords, *edge_a, *edge_b);
    }

    data
}

fn pregen_detail(
    layout: &CityChunkLayout,
    coords: (i32, i32),
    seed: u64,
    edges: &[Option<EdgeDirection>; 4],
) -> ChunkData {
    let mut data = ChunkData::default();

    for (building_index, spec) in layout.buildings.iter().enumerate() {
        let building_seed = seed
            .wrapping_add(SEED_DETAIL)
            .wrapping_add(building_index as u64 * 31337);
        let mut detail_rng = rand::rngs::StdRng::seed_from_u64(building_seed);
        describe_building_detail(&mut data, spec, &mut detail_rng);

        let mut smoke_rng = rand::rngs::StdRng::seed_from_u64(building_seed.wrapping_add(999));
        if matches!(spec.building_type, BuildingType::Warehouse)
            && smoke_rng.random_range(0.0f32..1.0) < SMOKE_PROBABILITY
        {
            data.smoke(Vec3::new(spec.x, spec.height + 0.5, spec.z));
        }
    }

    for streetlight in &layout.streetlight_positions {
        data.light(
            Vec3::new(
                streetlight.x,
                LAMP_POLE_HEIGHT + LAMP_GLOBE_RADIUS,
                streetlight.z,
            ),
            Vec3::new(1.0, 0.9, 0.7),
            5.0,
            18.0,
        );
    }

    let mut vehicle_rng = rand::rngs::StdRng::seed_from_u64(seed.wrapping_add(SEED_VEHICLES));
    describe_parked_vehicles(&mut data, &layout.road_segments, &mut vehicle_rng);

    let mut waterfront_rng =
        rand::rngs::StdRng::seed_from_u64(seed.wrapping_add(SEED_WATERFRONT_DETAIL));
    for edge in edges.iter().flatten() {
        waterfront::describe_dock_detail(&mut data, coords, *edge, &mut waterfront_rng);
    }

    data
}

fn describe_streetlight_mesh(data: &mut ChunkData, x: f32, z: f32) {
    data.instance(
        "Cylinder",
        Vec3::new(x, LAMP_POLE_HEIGHT / 2.0, z),
        Vec3::new(0.15, LAMP_POLE_HEIGHT, 0.15),
        "LampPole",
        None,
    );

    data.instance(
        "Sphere",
        Vec3::new(x, LAMP_POLE_HEIGHT + LAMP_GLOBE_RADIUS, z),
        Vec3::new(
            LAMP_GLOBE_RADIUS * 2.0,
            LAMP_GLOBE_RADIUS * 2.0,
            LAMP_GLOBE_RADIUS * 2.0,
        ),
        "LampGlow",
        None,
    );
}

const CROSSWALK_STRIPE_WIDTH: f32 = 0.4;
const CROSSWALK_STRIPE_GAP: f32 = 0.6;
const CROSSWALK_STRIPE_COUNT: i32 = 5;

fn describe_road_markings(data: &mut ChunkData, road_segments: &[city::RoadSegment]) {
    for segment in road_segments {
        if segment.is_sidewalk {
            continue;
        }

        let is_x_long = segment.width > segment.depth;
        let long_extent = if is_x_long {
            segment.width
        } else {
            segment.depth
        };

        let dash_spacing = MARKING_LENGTH + MARKING_GAP;
        let dash_count = (long_extent / dash_spacing).floor() as i32;
        let start_offset = -(dash_count as f32 * dash_spacing) / 2.0 + dash_spacing / 2.0;

        for dash_index in 0..dash_count {
            let offset = start_offset + dash_index as f32 * dash_spacing;

            let (x, z, sx, sz) = if is_x_long {
                (segment.x + offset, segment.z, MARKING_LENGTH, MARKING_WIDTH)
            } else {
                (segment.x, segment.z + offset, MARKING_WIDTH, MARKING_LENGTH)
            };

            data.instance(
                "Cube",
                Vec3::new(x, 0.05, z),
                Vec3::new(sx, MARKING_HEIGHT, sz),
                "RoadMarking",
                None,
            );
        }
    }

    for segment in road_segments {
        if segment.is_sidewalk {
            continue;
        }

        let is_x_long = segment.width > segment.depth;
        let road_width = 4.0;

        if is_x_long {
            let road_z = segment.z;
            let start_x = segment.x - segment.width / 2.0;
            let block_count = (segment.width / 16.0).round() as i32;
            for block_index in 0..block_count {
                let crossing_x = start_x + block_index as f32 * 16.0;
                describe_crosswalk_stripes(data, crossing_x, road_z, road_width, true);
            }
        } else {
            let road_x = segment.x;
            let start_z = segment.z - segment.depth / 2.0;
            let block_count = (segment.depth / 16.0).round() as i32;
            for block_index in 0..block_count {
                let crossing_z = start_z + block_index as f32 * 16.0;
                describe_crosswalk_stripes(data, road_x, crossing_z, road_width, false);
            }
        }
    }
}

fn describe_crosswalk_stripes(
    data: &mut ChunkData,
    center_x: f32,
    center_z: f32,
    road_width: f32,
    crossing_along_z: bool,
) {
    let total_width = CROSSWALK_STRIPE_COUNT as f32 * CROSSWALK_STRIPE_WIDTH
        + (CROSSWALK_STRIPE_COUNT - 1) as f32 * CROSSWALK_STRIPE_GAP;
    let start_offset = -total_width / 2.0 + CROSSWALK_STRIPE_WIDTH / 2.0;
    let stripe_length = road_width * 0.8;

    for stripe_index in 0..CROSSWALK_STRIPE_COUNT {
        let offset =
            start_offset + stripe_index as f32 * (CROSSWALK_STRIPE_WIDTH + CROSSWALK_STRIPE_GAP);

        let (x, z, sx, sz) = if crossing_along_z {
            (
                center_x + offset,
                center_z,
                CROSSWALK_STRIPE_WIDTH,
                stripe_length,
            )
        } else {
            (
                center_x,
                center_z + offset,
                stripe_length,
                CROSSWALK_STRIPE_WIDTH,
            )
        };

        data.instance(
            "Cube",
            Vec3::new(x, 0.06, z),
            Vec3::new(sx, MARKING_HEIGHT, sz),
            "RoadMarking",
            None,
        );
    }
}

fn describe_parked_vehicles(
    data: &mut ChunkData,
    road_segments: &[city::RoadSegment],
    rng: &mut impl Rng,
) {
    for segment in road_segments {
        if !segment.is_sidewalk {
            continue;
        }

        let is_x_long = segment.width > segment.depth;
        let long_extent = if is_x_long {
            segment.width
        } else {
            segment.depth
        };

        if long_extent < CAR_MIN_SIDEWALK {
            continue;
        }

        let car_count = rng.random_range(1u32..3);
        let spacing = long_extent / car_count as f32;

        for car_index in 0..car_count {
            let offset = -long_extent / 2.0 + (car_index as f32 + 0.5) * spacing;
            let model =
                crate::kenney::CAR_MODELS[rng.random_range(0..crate::kenney::CAR_MODELS.len())];

            let (car_x, car_z) = if is_x_long {
                (segment.x + offset, segment.z)
            } else {
                (segment.x, segment.z + offset)
            };

            let rotation = if is_x_long {
                Some(nalgebra_glm::quat_angle_axis(
                    std::f32::consts::FRAC_PI_2,
                    &Vec3::y(),
                ))
            } else {
                None
            };

            data.instance(
                model,
                Vec3::new(car_x, 0.0, car_z),
                CAR_SCALE,
                crate::kenney::MAT_CAR,
                rotation,
            );
        }
    }
}

fn describe_construction_cranes(
    data: &mut ChunkData,
    layout: &CityChunkLayout,
    rng: &mut impl Rng,
) {
    let tallest_skyscraper = layout
        .buildings
        .iter()
        .filter(|building| matches!(building.building_type, BuildingType::Skyscraper))
        .max_by(|a, b| a.height.partial_cmp(&b.height).unwrap());

    let Some(skyscraper) = tallest_skyscraper else {
        return;
    };

    if rng.random_range(0.0f32..1.0) > CRANE_PROBABILITY {
        return;
    }

    let crane_x = skyscraper.x + skyscraper.width / 2.0 + 3.0;
    let crane_z = skyscraper.z;
    let tower_height = CRANE_TOWER_HEIGHT.min(skyscraper.height * 0.9);

    data.instance(
        "Cylinder",
        Vec3::new(crane_x, tower_height / 2.0, crane_z),
        Vec3::new(
            CRANE_TOWER_RADIUS * 2.0,
            tower_height,
            CRANE_TOWER_RADIUS * 2.0,
        ),
        "CraneMetal",
        None,
    );

    data.instance(
        "Cube",
        Vec3::new(
            crane_x + CRANE_ARM_LENGTH / 2.0 - 2.0,
            tower_height,
            crane_z,
        ),
        Vec3::new(CRANE_ARM_LENGTH, CRANE_ARM_THICKNESS, CRANE_ARM_THICKNESS),
        "CraneMetal",
        None,
    );

    data.instance(
        "Cube",
        Vec3::new(crane_x - 3.0, tower_height, crane_z),
        Vec3::new(
            CRANE_COUNTERWEIGHT_SIZE,
            CRANE_COUNTERWEIGHT_SIZE,
            CRANE_COUNTERWEIGHT_SIZE,
        ),
        "CraneMetal",
        None,
    );

    data.instance(
        "Cylinder",
        Vec3::new(
            crane_x + CRANE_ARM_LENGTH - 4.0,
            tower_height - 5.0,
            crane_z,
        ),
        Vec3::new(0.08, 10.0, 0.08),
        "DockMetal",
        None,
    );
}

const TRAFFIC_POLE_HEIGHT: f32 = 3.0;
const TRAFFIC_POLE_RADIUS: f32 = 0.08;
const TRAFFIC_BOX_WIDTH: f32 = 0.3;
const TRAFFIC_BOX_HEIGHT: f32 = 0.6;
const TRAFFIC_LIGHT_RADIUS: f32 = 0.08;
const TRAFFIC_SIDEWALK_OFFSET: f32 = 2.8;

fn describe_traffic_lights(data: &mut ChunkData, layout: &CityChunkLayout) {
    for intersection in &layout.intersection_positions {
        let corner_hash = ((intersection.x * 73.0) as u32)
            .wrapping_mul(((intersection.z * 47.0) as u32).wrapping_add(17));
        let corner = (corner_hash.wrapping_add(2)) % 4;

        let (dx, dz) = match corner {
            0 => (TRAFFIC_SIDEWALK_OFFSET, TRAFFIC_SIDEWALK_OFFSET),
            1 => (TRAFFIC_SIDEWALK_OFFSET, -TRAFFIC_SIDEWALK_OFFSET),
            2 => (-TRAFFIC_SIDEWALK_OFFSET, TRAFFIC_SIDEWALK_OFFSET),
            _ => (-TRAFFIC_SIDEWALK_OFFSET, -TRAFFIC_SIDEWALK_OFFSET),
        };

        let pole_x = intersection.x + dx;
        let pole_z = intersection.z + dz;

        data.instance(
            "Cylinder",
            Vec3::new(pole_x, TRAFFIC_POLE_HEIGHT / 2.0, pole_z),
            Vec3::new(
                TRAFFIC_POLE_RADIUS * 2.0,
                TRAFFIC_POLE_HEIGHT,
                TRAFFIC_POLE_RADIUS * 2.0,
            ),
            "LampPole",
            None,
        );

        let box_y = TRAFFIC_POLE_HEIGHT + TRAFFIC_BOX_HEIGHT / 2.0;
        data.instance(
            "Cube",
            Vec3::new(pole_x, box_y, pole_z),
            Vec3::new(TRAFFIC_BOX_WIDTH, TRAFFIC_BOX_HEIGHT, TRAFFIC_BOX_WIDTH),
            "ConcreteDark",
            None,
        );

        let light_spacing = TRAFFIC_BOX_HEIGHT / 3.0;
        let light_diameter = TRAFFIC_LIGHT_RADIUS * 2.0;

        data.instance(
            "Sphere",
            Vec3::new(pole_x, box_y + light_spacing, pole_z),
            Vec3::new(light_diameter, light_diameter, light_diameter),
            "TrafficRed",
            None,
        );

        data.instance(
            "Sphere",
            Vec3::new(pole_x, box_y, pole_z),
            Vec3::new(light_diameter, light_diameter, light_diameter),
            "TrafficYellow",
            None,
        );

        data.instance(
            "Sphere",
            Vec3::new(pole_x, box_y - light_spacing, pole_z),
            Vec3::new(light_diameter, light_diameter, light_diameter),
            "TrafficGreen",
            None,
        );
    }
}

fn chebyshev_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

fn facing_bias(
    camera_chunk: (i32, i32),
    target_chunk: (i32, i32),
    camera_forward_xz: &Vec3,
) -> i32 {
    let dx = (target_chunk.0 - camera_chunk.0) as f32;
    let dz = (target_chunk.1 - camera_chunk.1) as f32;
    let len = (dx * dx + dz * dz).sqrt();
    if len < 0.001 {
        return 0;
    }
    let dot = (dx * camera_forward_xz.x + dz * camera_forward_xz.z) / len;
    if dot > 0.3 {
        FRUSTUM_BIAS_AHEAD
    } else if dot < -0.3 {
        FRUSTUM_BIAS_BEHIND
    } else {
        0
    }
}

fn edge_directions(coords: (i32, i32), city_min: i32, city_max: i32) -> [Option<EdgeDirection>; 4] {
    [
        if coords.0 == city_min {
            Some(EdgeDirection::West)
        } else {
            None
        },
        if coords.0 == city_max {
            Some(EdgeDirection::East)
        } else {
            None
        },
        if coords.1 == city_min {
            Some(EdgeDirection::North)
        } else {
            None
        },
        if coords.1 == city_max {
            Some(EdgeDirection::South)
        } else {
            None
        },
    ]
}

fn corner_bridge_edges(
    coords: (i32, i32),
    city_min: i32,
    city_max: i32,
) -> Option<(EdgeDirection, EdgeDirection)> {
    match coords {
        (x, z) if x == city_min && z == city_min => {
            Some((EdgeDirection::West, EdgeDirection::North))
        }
        (x, z) if x == city_min && z == city_max => {
            Some((EdgeDirection::West, EdgeDirection::South))
        }
        (x, z) if x == city_max && z == city_min => {
            Some((EdgeDirection::East, EdgeDirection::North))
        }
        (x, z) if x == city_max && z == city_max => {
            Some((EdgeDirection::East, EdgeDirection::South))
        }
        _ => None,
    }
}
