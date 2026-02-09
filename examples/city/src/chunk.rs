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

const MAX_ENTITIES_SPAWN_PER_FRAME: usize = 64;
const MAX_ENTITIES_DESPAWN_PER_FRAME: usize = 128;

const CITY_HALF: i32 = 8;
pub const CITY_MIN: i32 = -CITY_HALF;
pub const CITY_MAX: i32 = CITY_HALF - 1;

const LAMP_POLE_HEIGHT: f32 = 3.5;
const LAMP_GLOBE_RADIUS: f32 = 0.25;

const SMOKE_PROBABILITY: f32 = 0.40;

const FADE_DURATION: f32 = 0.3;

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
    base: ChunkData,
    proxy: ChunkData,
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

        let mut pregen = HashMap::new();
        for x in CITY_MIN..=CITY_MAX {
            for z in CITY_MIN..=CITY_MAX {
                let coords = (x, z);
                let layout = &layouts[&coords];
                pregen.insert(coords, pregen_chunk(layout, coords));
            }
        }

        Self {
            chunks: HashMap::new(),
            loading: HashMap::new(),
            despawning: Vec::new(),
            pregen,
            layouts,
            last_camera_chunk: (i32::MAX, i32::MAX),
            fading_entities: Vec::new(),
        }
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
        let mut completed_indices = Vec::new();
        for (index, fade) in self.fading_entities.iter_mut().enumerate() {
            fade.elapsed += delta_time;
            let alpha = (fade.elapsed / FADE_DURATION).min(1.0);
            if alpha >= 1.0 {
                world
                    .resources
                    .mesh_render_state
                    .mark_entity_fade_complete(fade.entity);
                completed_indices.push(index);
            } else {
                world
                    .resources
                    .mesh_render_state
                    .set_entity_custom_data(fade.entity, [1.0, 1.0, 1.0, alpha]);
            }
        }
        for index in completed_indices.into_iter().rev() {
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
        {
            return;
        }

        let delta_time = world.resources.window.timing.delta_time;
        self.advance_fades(world, delta_time);

        for x in CITY_MIN..=CITY_MAX {
            for z in CITY_MIN..=CITY_MAX {
                let coords = (x, z);
                if !self.chunks.contains_key(&coords) {
                    let pregen = &self.pregen[&coords];
                    let base_entities =
                        pregen
                            .base
                            .instantiate_range(world, 0, pregen.base.total_count());
                    let proxy_entities =
                        pregen
                            .proxy
                            .instantiate_range(world, 0, pregen.proxy.total_count());
                    self.chunks.insert(
                        coords,
                        ChunkState {
                            base_entities,
                            proxy_entities,
                            building_entities: Vec::new(),
                            detail_entities: Vec::new(),
                        },
                    );
                }
            }
        }

        let mut to_cancel: Vec<((i32, i32), LayerKind)> = Vec::new();
        for &(coords, kind) in self.loading.keys() {
            let distance = chebyshev_distance(coords, camera_chunk);
            let chunk = &self.chunks[&coords];
            let (want_buildings, want_detail) = desired_layers(
                distance,
                !chunk.building_entities.is_empty(),
                !chunk.detail_entities.is_empty(),
            );
            match kind {
                LayerKind::Buildings if !want_buildings => to_cancel.push((coords, kind)),
                LayerKind::Detail if !want_detail => to_cancel.push((coords, kind)),
                _ => {}
            }
        }
        for key in to_cancel {
            if let Some(loading) = self.loading.remove(&key)
                && !loading.entities.is_empty()
            {
                self.despawning.push(DespawningChunk {
                    entities: loading.entities,
                    cursor: 0,
                });
            }
        }

        let camera_forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z);
        let forward_len = nalgebra_glm::length(&camera_forward_xz);
        let camera_forward_normalized = if forward_len > 0.001 {
            camera_forward_xz / forward_len
        } else {
            Vec3::new(0.0, 0.0, -1.0)
        };

        let mut loading_keys: Vec<((i32, i32), LayerKind)> = self.loading.keys().copied().collect();
        loading_keys.sort_by_key(|&(coords, _)| {
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

        let active_count = loading_keys
            .iter()
            .filter(|key| {
                let lc = &self.loading[key];
                let pregen = &self.pregen[&key.0];
                let total = match key.1 {
                    LayerKind::Buildings => pregen.buildings.total_count(),
                    LayerKind::Detail => pregen.detail.total_count(),
                };
                lc.cursor < total
            })
            .count()
            .max(1);

        let per_chunk_budget = (MAX_ENTITIES_SPAWN_PER_FRAME / active_count).max(1);
        let mut spawn_budget = MAX_ENTITIES_SPAWN_PER_FRAME;

        let mut completed: Vec<((i32, i32), LayerKind)> = Vec::new();

        for key in loading_keys {
            if spawn_budget == 0 {
                break;
            }

            let pregen = &self.pregen[&key.0];
            let lc = self.loading.get_mut(&key).unwrap();

            let data = match key.1 {
                LayerKind::Buildings => &pregen.buildings,
                LayerKind::Detail => &pregen.detail,
            };

            let total = data.total_count();
            if lc.cursor >= total {
                completed.push(key);
                continue;
            }

            let chunk_allowance = per_chunk_budget.min(spawn_budget);
            let new_entities = data.instantiate_range(world, lc.cursor, chunk_allowance);
            let spawned = new_entities.len();
            for &entity in &new_entities {
                world
                    .resources
                    .mesh_render_state
                    .set_entity_custom_data(entity, [1.0, 1.0, 1.0, 0.0]);
                self.fading_entities.push(FadeEntry {
                    entity,
                    elapsed: 0.0,
                });
            }
            lc.entities.extend(new_entities);
            lc.cursor += spawned;
            spawn_budget -= spawned;

            if lc.cursor >= total {
                completed.push(key);
            }
        }

        for key in completed {
            let lc = self.loading.remove(&key).unwrap();
            let chunk = self.chunks.get_mut(&key.0).unwrap();

            match key.1 {
                LayerKind::Buildings => {
                    chunk.building_entities = lc.entities;
                }
                LayerKind::Detail => {
                    chunk.detail_entities = lc.entities;
                }
            }
        }

        let mut despawn_budget = MAX_ENTITIES_DESPAWN_PER_FRAME;
        let mut finished_despawns = Vec::new();
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
                finished_despawns.push(index);
            }
        }
        for index in finished_despawns.into_iter().rev() {
            self.despawning.swap_remove(index);
        }

        for x in CITY_MIN..=CITY_MAX {
            for z in CITY_MIN..=CITY_MAX {
                let coords = (x, z);
                let distance = chebyshev_distance(coords, camera_chunk);
                let chunk = self.chunks.get_mut(&coords).unwrap();

                let has_buildings = !chunk.building_entities.is_empty();
                let has_detail = !chunk.detail_entities.is_empty();
                let (want_buildings, want_detail) =
                    desired_layers(distance, has_buildings, has_detail);

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
                    let entities: Vec<Entity> = chunk.detail_entities.drain(..).collect();
                    if !entities.is_empty() {
                        self.despawning.push(DespawningChunk {
                            entities,
                            cursor: 0,
                        });
                    }
                }

                if !want_buildings && has_buildings {
                    if has_detail {
                        let detail_entities: Vec<Entity> =
                            chunk.detail_entities.drain(..).collect();
                        if !detail_entities.is_empty() {
                            self.despawning.push(DespawningChunk {
                                entities: detail_entities,
                                cursor: 0,
                            });
                        }
                    }

                    let building_entities: Vec<Entity> =
                        chunk.building_entities.drain(..).collect();
                    if !building_entities.is_empty() {
                        self.despawning.push(DespawningChunk {
                            entities: building_entities,
                            cursor: 0,
                        });
                    }
                }
            }
        }
    }
}

fn desired_layers(distance: i32, has_buildings: bool, has_detail: bool) -> (bool, bool) {
    let want_buildings = if has_buildings {
        distance <= LOD1_RADIUS + 1
    } else {
        distance <= LOD1_RADIUS
    };

    let want_detail = if has_detail {
        distance <= LOD0_RADIUS + 1
    } else {
        distance <= LOD0_RADIUS
    };

    (want_buildings, want_detail && want_buildings)
}

fn pregen_chunk(layout: &CityChunkLayout, coords: (i32, i32)) -> PregenChunk {
    let seed = chunk_seed(coords);
    let edges = edge_directions(coords);
    let bridge_edges = corner_bridge_edges(coords);

    PregenChunk {
        base: pregen_base(layout, coords, &edges),
        proxy: pregen_proxy(layout),
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
