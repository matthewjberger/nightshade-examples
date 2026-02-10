use std::collections::HashMap;

use nightshade::prelude::*;
use noise::Perlin;
use rand::{Rng, SeedableRng};

use crate::building::{
    BuildingType, describe_building_body, describe_building_detail, describe_building_proxy,
};
use crate::city::{self, CHUNK_SIZE, CityChunkLayout, generate_chunk_layout};
use crate::descriptors::ChunkData;
use crate::waterfront::{self, EdgeDirection};

const LOD0_RADIUS: i32 = 4;
const LOD1_RADIUS: i32 = 8;
const BASE_RADIUS: i32 = LOD1_RADIUS + 2;
const FRUSTUM_BIAS_AHEAD: i32 = 2;
const FRUSTUM_BIAS_BEHIND: i32 = -2;
const MAX_MANAGEMENT_RADIUS: i32 = BASE_RADIUS + FRUSTUM_BIAS_AHEAD + 2;

const COMBINED_BUDGET: usize = 192;
const MIN_BUDGET_PER_SIDE: usize = 32;

const CITY_HALF: i32 = 8;
pub const CITY_MIN: i32 = -CITY_HALF;
pub const CITY_MAX: i32 = CITY_HALF - 1;

const LAMP_POLE_HEIGHT: f32 = 3.5;
const LAMP_GLOBE_RADIUS: f32 = 0.25;

const SMOKE_PROBABILITY: f32 = 0.40;

const FADE_DURATION: f32 = 0.3;
const MAX_PREGEN_PER_FRAME: usize = 2;

const SEED_BODY: u64 = 10000;
const SEED_DETAIL: u64 = 20000;
const SEED_WATERFRONT: u64 = 12345;
const SEED_WATERFRONT_DETAIL: u64 = 12346;
const SEED_VEHICLES: u64 = 54321;
const SEED_CRANES: u64 = 91919;

const CAR_BODY_WIDTH: f32 = 2.0;
const CAR_BODY_HEIGHT: f32 = 0.8;
const CAR_BODY_DEPTH: f32 = 1.0;
const CAR_CABIN_WIDTH: f32 = 1.0;
const CAR_CABIN_HEIGHT: f32 = 0.5;
const CAR_CABIN_DEPTH: f32 = 0.8;
const CAR_COLORS: &[&str] = &["CarRed", "CarBlue", "CarWhite", "CarBlack", "CarSilver"];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LayerKind {
    Base,
    Buildings,
    Detail,
}

struct ChunkState {
    base_entities: Vec<Entity>,
    proxy_entities: Vec<Entity>,
    building_entities: Vec<Entity>,
    detail_entities: Vec<Entity>,
}

struct LoadingCursor {
    cursor: usize,
    entities: Vec<Entity>,
}

struct DespawningChunk {
    entities: Vec<Entity>,
    cursor: usize,
}

struct FadeEntry {
    entity: Entity,
    elapsed: f32,
}

struct PregenChunk {
    base_proxy: ChunkData,
    base_count: usize,
    buildings: ChunkData,
    detail: ChunkData,
}

pub struct ChunkManager {
    chunks: HashMap<(i32, i32), ChunkState>,
    loading: HashMap<((i32, i32), LayerKind), LoadingCursor>,
    despawning: Vec<DespawningChunk>,
    pregen: HashMap<(i32, i32), PregenChunk>,
    layouts: HashMap<(i32, i32), city::CityChunkLayout>,
    last_camera_chunk: (i32, i32),
    fading_entities: Vec<FadeEntry>,
    pregen_budget: usize,
    pending_pregen: bool,
    scratch_completed_fade_indices: Vec<usize>,
    scratch_to_cancel: Vec<((i32, i32), LayerKind)>,
    scratch_loading_keys: Vec<((i32, i32), LayerKind)>,
    scratch_completed_loading: Vec<((i32, i32), LayerKind)>,
    scratch_finished_despawns: Vec<usize>,
    scratch_chunks_to_unload: Vec<(i32, i32)>,
}

fn chunk_seed(coords: (i32, i32)) -> u64 {
    (coords.0 as u64).wrapping_mul(73856093) ^ (coords.1 as u64).wrapping_mul(19349663)
}

impl ChunkManager {
    pub fn new() -> Self {
        let noise = Perlin::new(42);
        let mut layouts = HashMap::new();
        for x in CITY_MIN..=CITY_MAX {
            for z in CITY_MIN..=CITY_MAX {
                layouts.insert((x, z), generate_chunk_layout(x, z, &noise));
            }
        }

        Self {
            chunks: HashMap::new(),
            loading: HashMap::new(),
            despawning: Vec::new(),
            pregen: HashMap::new(),
            layouts,
            last_camera_chunk: (i32::MAX, i32::MAX),
            fading_entities: Vec::new(),
            pregen_budget: MAX_PREGEN_PER_FRAME,
            pending_pregen: false,
            scratch_completed_fade_indices: Vec::new(),
            scratch_to_cancel: Vec::new(),
            scratch_loading_keys: Vec::new(),
            scratch_completed_loading: Vec::new(),
            scratch_finished_despawns: Vec::new(),
            scratch_chunks_to_unload: Vec::new(),
        }
    }

    fn ensure_pregen(&mut self, coords: (i32, i32)) -> bool {
        if self.pregen.contains_key(&coords) {
            return true;
        }
        if self.pregen_budget == 0 {
            return false;
        }
        let layout = &self.layouts[&coords];
        self.pregen.insert(coords, pregen_chunk(layout, coords));
        self.pregen_budget -= 1;
        true
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn layouts(&self) -> &HashMap<(i32, i32), city::CityChunkLayout> {
        &self.layouts
    }

    pub fn entity_count(&self) -> usize {
        let chunk_entities: usize = self
            .chunks
            .values()
            .map(|chunk| {
                chunk.base_entities.len()
                    + chunk.proxy_entities.len()
                    + chunk.building_entities.len()
                    + chunk.detail_entities.len()
            })
            .sum();
        let loading_entities: usize = self.loading.values().map(|lc| lc.entities.len()).sum();
        let despawning_entities: usize = self
            .despawning
            .iter()
            .map(|dc| dc.entities.len() - dc.cursor)
            .sum();
        chunk_entities + loading_entities + despawning_entities
    }

    fn advance_fades(&mut self, world: &mut World, delta_time: f32) {
        self.scratch_completed_fade_indices.clear();
        for (index, fade) in self.fading_entities.iter_mut().enumerate() {
            fade.elapsed += delta_time;
            let alpha = (fade.elapsed / FADE_DURATION).min(1.0);
            if alpha >= 1.0 {
                world
                    .resources
                    .mesh_render_state
                    .mark_entity_fade_complete(fade.entity);
                self.scratch_completed_fade_indices.push(index);
            } else {
                world
                    .resources
                    .mesh_render_state
                    .set_entity_custom_data(fade.entity, [1.0, 1.0, 1.0, alpha]);
            }
        }
        for &index in self.scratch_completed_fade_indices.iter().rev() {
            self.fading_entities.swap_remove(index);
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

        if !camera_changed
            && self.loading.is_empty()
            && self.despawning.is_empty()
            && self.fading_entities.is_empty()
            && !self.pending_pregen
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
                self.fading_entities
                    .retain(|fade| !loading.entities.contains(&fade.entity));
                self.despawning.push(DespawningChunk {
                    entities: loading.entities,
                    cursor: 0,
                });
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
                    LayerKind::Base => pregen.base_proxy.total_count(),
                    LayerKind::Buildings => pregen.buildings.total_count(),
                    LayerKind::Detail => pregen.detail.total_count(),
                };
                lc.cursor < total
            })
            .count()
            .max(1);

        let spawn_queue_depth: usize = self
            .scratch_loading_keys
            .iter()
            .map(|key| {
                let lc = &self.loading[key];
                let pregen = &self.pregen[&key.0];
                let total = match key.1 {
                    LayerKind::Base => pregen.base_proxy.total_count(),
                    LayerKind::Buildings => pregen.buildings.total_count(),
                    LayerKind::Detail => pregen.detail.total_count(),
                };
                total.saturating_sub(lc.cursor)
            })
            .sum();
        let despawn_queue_depth: usize = self
            .despawning
            .iter()
            .map(|dc| dc.entities.len().saturating_sub(dc.cursor))
            .sum();
        let total_depth = spawn_queue_depth + despawn_queue_depth;
        let (spawn_budget_max, despawn_budget_max) = if total_depth == 0 {
            (COMBINED_BUDGET / 2, COMBINED_BUDGET / 2)
        } else {
            let spawn_share =
                (COMBINED_BUDGET * spawn_queue_depth / total_depth).clamp(MIN_BUDGET_PER_SIDE, COMBINED_BUDGET - MIN_BUDGET_PER_SIDE);
            (spawn_share, COMBINED_BUDGET - spawn_share)
        };

        let per_chunk_budget = (spawn_budget_max / active_count).max(1);
        let mut spawn_budget = spawn_budget_max;

        self.scratch_completed_loading.clear();

        for key_index in 0..self.scratch_loading_keys.len() {
            let key = self.scratch_loading_keys[key_index];
            if spawn_budget == 0 {
                break;
            }

            let pregen = &self.pregen[&key.0];
            let lc = self.loading.get_mut(&key).unwrap();

            let data = match key.1 {
                LayerKind::Base => &pregen.base_proxy,
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
            let new_entities = data.instantiate_range(world, cursor_before, chunk_allowance);
            let spawned = new_entities.len();
            for (offset, &entity) in new_entities.iter().enumerate() {
                let is_base_geometry = key.1 == LayerKind::Base
                    && (cursor_before + offset) < pregen.base_count;
                if !is_base_geometry {
                    world
                        .resources
                        .mesh_render_state
                        .set_entity_custom_data(entity, [1.0, 1.0, 1.0, 0.0]);
                    self.fading_entities.push(FadeEntry {
                        entity,
                        elapsed: 0.0,
                    });
                }
            }
            lc.entities.extend(new_entities);
            lc.cursor += spawned;
            spawn_budget -= spawned;

            if lc.cursor >= total {
                self.scratch_completed_loading.push(key);
            }
        }

        for index in 0..self.scratch_completed_loading.len() {
            let key = self.scratch_completed_loading[index];
            let lc = self.loading.remove(&key).unwrap();
            match key.1 {
                LayerKind::Base => {
                    let pregen = &self.pregen[&key.0];
                    let chunk = self.chunks.get_mut(&key.0).unwrap();
                    let split_point = pregen.base_count.min(lc.entities.len());
                    chunk.base_entities = lc.entities[..split_point].to_vec();
                    chunk.proxy_entities = lc.entities[split_point..].to_vec();
                }
                LayerKind::Buildings => {
                    let chunk = self.chunks.get_mut(&key.0).unwrap();
                    chunk.building_entities = lc.entities;
                }
                LayerKind::Detail => {
                    let chunk = self.chunks.get_mut(&key.0).unwrap();
                    chunk.detail_entities = lc.entities;
                }
            }
        }

        let mut despawn_budget = despawn_budget_max;
        self.scratch_finished_despawns.clear();
        for (index, chunk) in self.despawning.iter_mut().enumerate() {
            if despawn_budget == 0 {
                break;
            }
            let remaining = chunk.entities.len() - chunk.cursor;
            let to_despawn = remaining.min(despawn_budget);
            for entity in &chunk.entities[chunk.cursor..chunk.cursor + to_despawn] {
                world.queue_despawn_entity(*entity);
            }
            chunk.cursor += to_despawn;
            despawn_budget -= to_despawn;
            if chunk.cursor >= chunk.entities.len() {
                self.scratch_finished_despawns.push(index);
            }
        }
        for &index in self.scratch_finished_despawns.iter().rev() {
            self.despawning.swap_remove(index);
        }

        let scan_min_x = (camera_chunk.0 - MAX_MANAGEMENT_RADIUS).max(CITY_MIN);
        let scan_max_x = (camera_chunk.0 + MAX_MANAGEMENT_RADIUS).min(CITY_MAX);
        let scan_min_z = (camera_chunk.1 - MAX_MANAGEMENT_RADIUS).max(CITY_MIN);
        let scan_max_z = (camera_chunk.1 + MAX_MANAGEMENT_RADIUS).min(CITY_MAX);

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
                                proxy_entities: Vec::new(),
                                building_entities: Vec::new(),
                                detail_entities: Vec::new(),
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

                if !want_detail && has_detail {
                    let entities = std::mem::take(&mut chunk.detail_entities);
                    self.fading_entities.retain(|fade| !entities.contains(&fade.entity));
                    self.despawning.push(DespawningChunk {
                        entities,
                        cursor: 0,
                    });
                }

                if !want_buildings && has_buildings {
                    if has_detail {
                        let entities = std::mem::take(&mut chunk.detail_entities);
                        self.fading_entities.retain(|fade| !entities.contains(&fade.entity));
                        self.despawning.push(DespawningChunk {
                            entities,
                            cursor: 0,
                        });
                    }
                    let entities = std::mem::take(&mut chunk.building_entities);
                    self.fading_entities.retain(|fade| !entities.contains(&fade.entity));
                    self.despawning.push(DespawningChunk {
                        entities,
                        cursor: 0,
                    });
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
                    self.fading_entities
                        .retain(|fade| !loading.entities.contains(&fade.entity));
                    self.despawning.push(DespawningChunk {
                        entities: loading.entities,
                        cursor: 0,
                    });
                }
            }
            if let Some(chunk) = self.chunks.remove(&coords) {
                let mut all_entities = chunk.base_entities;
                all_entities.extend(chunk.proxy_entities);
                all_entities.extend(chunk.building_entities);
                all_entities.extend(chunk.detail_entities);
                self.fading_entities
                    .retain(|fade| !all_entities.contains(&fade.entity));
                if !all_entities.is_empty() {
                    self.despawning.push(DespawningChunk {
                        entities: all_entities,
                        cursor: 0,
                    });
                }
            }
        }
    }
}

fn desired_base(distance: i32, has_base: bool, bias: i32) -> bool {
    if has_base {
        distance <= BASE_RADIUS + FRUSTUM_BIAS_AHEAD + 2
    } else {
        distance <= BASE_RADIUS + bias
    }
}

fn desired_layers(
    distance: i32,
    has_buildings: bool,
    has_detail: bool,
    bias: i32,
) -> (bool, bool) {
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

fn pregen_chunk(layout: &CityChunkLayout, coords: (i32, i32)) -> PregenChunk {
    let seed = chunk_seed(coords);
    let edges = edge_directions(coords);
    let bridge_edges = corner_bridge_edges(coords);

    let mut base_proxy = pregen_base(layout, coords, &edges);
    let base_count = base_proxy.total_count();
    let proxy = pregen_proxy(layout);
    base_proxy.meshes.extend(proxy.meshes);
    base_proxy.lights.extend(proxy.lights);
    base_proxy.smoke_emitters.extend(proxy.smoke_emitters);

    PregenChunk {
        base_proxy,
        base_count,
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

    data.mesh(
        "Cube",
        Vec3::new(
            chunk_base_x + CHUNK_SIZE / 2.0,
            -0.1,
            chunk_base_z + CHUNK_SIZE / 2.0,
        ),
        Vec3::new(CHUNK_SIZE, 0.1, CHUNK_SIZE),
        "Ground",
    );

    for segment in &layout.road_segments {
        let material = if segment.is_sidewalk {
            "Sidewalk"
        } else {
            "Road"
        };
        data.mesh(
            "Cube",
            Vec3::new(segment.x, 0.02, segment.z),
            Vec3::new(segment.width, 0.04, segment.depth),
            material,
        );
    }

    for edge in edges.iter().flatten() {
        waterfront::describe_dock_base(&mut data, coords, *edge);
    }

    data
}

fn pregen_proxy(layout: &CityChunkLayout) -> ChunkData {
    let mut data = ChunkData::default();
    for spec in &layout.buildings {
        describe_building_proxy(&mut data, spec);
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
    data.mesh(
        "Cylinder",
        Vec3::new(x, LAMP_POLE_HEIGHT / 2.0, z),
        Vec3::new(0.15, LAMP_POLE_HEIGHT, 0.15),
        "LampPole",
    );

    data.mesh(
        "Sphere",
        Vec3::new(x, LAMP_POLE_HEIGHT + LAMP_GLOBE_RADIUS, z),
        Vec3::new(
            LAMP_GLOBE_RADIUS * 2.0,
            LAMP_GLOBE_RADIUS * 2.0,
            LAMP_GLOBE_RADIUS * 2.0,
        ),
        "LampGlow",
    );
}

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

            data.mesh(
                "Cube",
                Vec3::new(x, 0.05, z),
                Vec3::new(sx, MARKING_HEIGHT, sz),
                "RoadMarking",
            );
        }
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

        if long_extent < CAR_BODY_WIDTH * 2.0 {
            continue;
        }

        let car_count = rng.random_range(1u32..3);
        let spacing = long_extent / car_count as f32;

        for car_index in 0..car_count {
            let offset = -long_extent / 2.0 + (car_index as f32 + 0.5) * spacing;
            let color = CAR_COLORS[rng.random_range(0..CAR_COLORS.len())];

            let (car_x, car_z) = if is_x_long {
                (segment.x + offset, segment.z)
            } else {
                (segment.x, segment.z + offset)
            };

            data.mesh(
                "Cube",
                Vec3::new(car_x, CAR_BODY_HEIGHT / 2.0, car_z),
                Vec3::new(
                    if is_x_long {
                        CAR_BODY_WIDTH
                    } else {
                        CAR_BODY_DEPTH
                    },
                    CAR_BODY_HEIGHT,
                    if is_x_long {
                        CAR_BODY_DEPTH
                    } else {
                        CAR_BODY_WIDTH
                    },
                ),
                color,
            );

            data.mesh(
                "Cube",
                Vec3::new(car_x, CAR_BODY_HEIGHT + CAR_CABIN_HEIGHT / 2.0, car_z),
                Vec3::new(
                    if is_x_long {
                        CAR_CABIN_WIDTH
                    } else {
                        CAR_CABIN_DEPTH
                    },
                    CAR_CABIN_HEIGHT,
                    if is_x_long {
                        CAR_CABIN_DEPTH
                    } else {
                        CAR_CABIN_WIDTH
                    },
                ),
                color,
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

    data.mesh(
        "Cylinder",
        Vec3::new(crane_x, tower_height / 2.0, crane_z),
        Vec3::new(
            CRANE_TOWER_RADIUS * 2.0,
            tower_height,
            CRANE_TOWER_RADIUS * 2.0,
        ),
        "CraneMetal",
    );

    data.mesh(
        "Cube",
        Vec3::new(
            crane_x + CRANE_ARM_LENGTH / 2.0 - 2.0,
            tower_height,
            crane_z,
        ),
        Vec3::new(CRANE_ARM_LENGTH, CRANE_ARM_THICKNESS, CRANE_ARM_THICKNESS),
        "CraneMetal",
    );

    data.mesh(
        "Cube",
        Vec3::new(crane_x - 3.0, tower_height, crane_z),
        Vec3::new(
            CRANE_COUNTERWEIGHT_SIZE,
            CRANE_COUNTERWEIGHT_SIZE,
            CRANE_COUNTERWEIGHT_SIZE,
        ),
        "CraneMetal",
    );

    data.mesh(
        "Cylinder",
        Vec3::new(
            crane_x + CRANE_ARM_LENGTH - 4.0,
            tower_height - 5.0,
            crane_z,
        ),
        Vec3::new(0.08, 10.0, 0.08),
        "DockMetal",
    );
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

fn edge_directions(coords: (i32, i32)) -> [Option<EdgeDirection>; 4] {
    [
        if coords.0 == CITY_MIN {
            Some(EdgeDirection::West)
        } else {
            None
        },
        if coords.0 == CITY_MAX {
            Some(EdgeDirection::East)
        } else {
            None
        },
        if coords.1 == CITY_MIN {
            Some(EdgeDirection::North)
        } else {
            None
        },
        if coords.1 == CITY_MAX {
            Some(EdgeDirection::South)
        } else {
            None
        },
    ]
}

fn corner_bridge_edges(coords: (i32, i32)) -> Option<(EdgeDirection, EdgeDirection)> {
    match coords {
        (x, z) if x == CITY_MIN && z == CITY_MIN => {
            Some((EdgeDirection::West, EdgeDirection::North))
        }
        (x, z) if x == CITY_MIN && z == CITY_MAX => {
            Some((EdgeDirection::West, EdgeDirection::South))
        }
        (x, z) if x == CITY_MAX && z == CITY_MIN => {
            Some((EdgeDirection::East, EdgeDirection::North))
        }
        (x, z) if x == CITY_MAX && z == CITY_MAX => {
            Some((EdgeDirection::East, EdgeDirection::South))
        }
        _ => None,
    }
}
