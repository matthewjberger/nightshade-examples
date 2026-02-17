use nightshade::prelude::*;

use crate::ecs::LocationId;

pub const CASTLE_SIZE: f32 = 15.0;
pub const WALL_HEIGHT: f32 = 3.0;
pub const WALL_THICKNESS: f32 = 0.8;
pub const WALL_SEGMENT_WIDTH: f32 = 6.0;
pub const SEGMENTS_PER_WALL: usize = 5;
pub const SEGMENT_HP: f32 = 100.0;
pub const GATE_HP: f32 = 300.0;
pub const GATE_WIDTH: f32 = 4.0;

pub const WELL_POS: Vec3 = Vec3::new(-4.0, 0.0, -2.0);
pub const ARMORY_POS: Vec3 = Vec3::new(-8.0, 0.0, -8.0);
pub const HEALING_POS: Vec3 = Vec3::new(8.0, 0.0, -6.0);
pub const REPAIR_PILE_POS: Vec3 = Vec3::new(0.0, 0.0, 4.0);
pub const GATE_POS: Vec3 = Vec3::new(0.0, 0.0, 15.0);
pub const RIVER_POS: Vec3 = Vec3::new(0.0, 0.0, -20.0);
pub const BACK_GATE_POS: Vec3 = Vec3::new(0.0, 0.0, -15.0);

pub const ARCHER_POST_POSITIONS: [Vec3; 4] = [
    Vec3::new(-14.0, 3.5, -14.0),
    Vec3::new(14.0, 3.5, -14.0),
    Vec3::new(-14.0, 3.5, 14.0),
    Vec3::new(14.0, 3.5, 14.0),
];

#[derive(Clone, Debug)]
pub struct WallSegment {
    pub health: f32,
    pub max_health: f32,
    pub entity: Entity,
    pub position: Vec3,
    pub breached: bool,
}

impl Default for WallSegment {
    fn default() -> Self {
        Self {
            health: SEGMENT_HP,
            max_health: SEGMENT_HP,
            entity: Entity::default(),
            position: Vec3::zeros(),
            breached: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WallSide {
    pub segments: Vec<WallSegment>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum WallDirection {
    #[default]
    North,
    South,
    East,
    West,
}

#[derive(Clone, Debug)]
pub struct ArcherPost {
    pub position: Vec3,
    pub arrows_remaining: u32,
    pub max_arrows: u32,
    pub fire_timer: f32,
    pub line_entity: Option<Entity>,
}

impl Default for ArcherPost {
    fn default() -> Self {
        Self {
            position: Vec3::zeros(),
            arrows_remaining: 10,
            max_arrows: 10,
            fire_timer: 0.0,
            line_entity: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CastleState {
    pub walls: [WallSide; 4],
    pub gate_health: f32,
    pub gate_max_health: f32,
    pub gate_entity: Entity,
    pub well_water_remaining: f32,
    pub well_entity: Entity,
    pub well_destroyed: bool,
    pub armory_exists: bool,
    pub armory_stock: u32,
    pub armory_entity: Entity,
    pub healing_station_exists: bool,
    pub healing_station_entity: Entity,
    pub repair_pile_count: u32,
    pub repair_pile_entity: Entity,
    pub back_gate_intact: bool,
    pub river_accessible: bool,
    pub river_entity: Entity,
    pub archer_posts: [ArcherPost; 4],
    pub claimed_goals: Vec<(usize, crate::goap::GoalType)>,
    pub ground_entity: Entity,
    pub floor_entity: Entity,
    pub all_render_entities: Vec<Entity>,
}

impl Default for CastleState {
    fn default() -> Self {
        Self {
            walls: std::array::from_fn(|_| WallSide {
                segments: Vec::new(),
            }),
            gate_health: GATE_HP,
            gate_max_health: GATE_HP,
            gate_entity: Entity::default(),
            well_water_remaining: 100.0,
            well_entity: Entity::default(),
            well_destroyed: false,
            armory_exists: true,
            armory_stock: 40,
            armory_entity: Entity::default(),
            healing_station_exists: true,
            healing_station_entity: Entity::default(),
            repair_pile_count: 20,
            repair_pile_entity: Entity::default(),
            back_gate_intact: true,
            river_accessible: true,
            river_entity: Entity::default(),
            archer_posts: std::array::from_fn(|_| ArcherPost::default()),
            claimed_goals: Vec::new(),
            ground_entity: Entity::default(),
            floor_entity: Entity::default(),
            all_render_entities: Vec::new(),
        }
    }
}

fn create_material(world: &mut World, name: &str, color: [f32; 4]) {
    use nightshade::ecs::material::resources::material_registry_insert;
    material_registry_insert(
        &mut world.resources.material_registry,
        name.to_string(),
        Material {
            base_color: color,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
}

fn create_emissive_material(world: &mut World, name: &str, color: [f32; 4], emissive: [f32; 3]) {
    use nightshade::ecs::material::resources::material_registry_insert;
    material_registry_insert(
        &mut world.resources.material_registry,
        name.to_string(),
        Material {
            base_color: color,
            emissive_factor: emissive,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
}

fn spawn_tracked(
    world: &mut World,
    tracker: &mut Vec<Entity>,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material_name: &str,
) -> Entity {
    let entity = spawn_mesh_at(world, mesh_name, position, scale);
    world.set_material_ref(entity, MaterialRef::new(material_name.to_string()));
    tracker.push(entity);
    entity
}

pub fn spawn_castle(world: &mut World) -> CastleState {
    let mut castle = CastleState::default();

    create_material(world, "ground", [0.25, 0.35, 0.2, 1.0]);
    let ground = spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Plane",
        nalgebra_glm::vec3(0.0, -0.3, 0.0),
        nalgebra_glm::vec3(60.0, 1.0, 60.0),
        "ground",
    );
    castle.ground_entity = ground;

    create_material(world, "castle_floor", [0.65, 0.55, 0.4, 1.0]);
    let floor = spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Plane",
        nalgebra_glm::vec3(0.0, -0.02, 0.0),
        nalgebra_glm::vec3(CASTLE_SIZE * 2.0 + 1.0, 1.0, CASTLE_SIZE * 2.0 + 1.0),
        "castle_floor",
    );
    castle.floor_entity = floor;

    create_material(world, "wall_healthy", [0.82, 0.71, 0.55, 1.0]);

    spawn_walls(world, &mut castle);
    spawn_gate(world, &mut castle);
    spawn_well(world, &mut castle);
    spawn_armory(world, &mut castle);
    spawn_healing_station(world, &mut castle);
    spawn_repair_pile(world, &mut castle);
    spawn_river(world, &mut castle);
    spawn_archer_posts(world, &mut castle);
    spawn_siege_engines(world, &mut castle.all_render_entities);

    castle
}

fn spawn_walls(world: &mut World, castle: &mut CastleState) {
    let wall_configs: [(WallDirection, Vec3, Vec3); 4] = [
        (
            WallDirection::North,
            nalgebra_glm::vec3(0.0, 0.0, -CASTLE_SIZE),
            nalgebra_glm::vec3(1.0, 0.0, 0.0),
        ),
        (
            WallDirection::South,
            nalgebra_glm::vec3(0.0, 0.0, CASTLE_SIZE),
            nalgebra_glm::vec3(1.0, 0.0, 0.0),
        ),
        (
            WallDirection::East,
            nalgebra_glm::vec3(CASTLE_SIZE, 0.0, 0.0),
            nalgebra_glm::vec3(0.0, 0.0, 1.0),
        ),
        (
            WallDirection::West,
            nalgebra_glm::vec3(-CASTLE_SIZE, 0.0, 0.0),
            nalgebra_glm::vec3(0.0, 0.0, 1.0),
        ),
    ];

    for (wall_index, (direction, base_pos, along)) in wall_configs.iter().enumerate() {
        let mut segments = Vec::new();
        let total_width = WALL_SEGMENT_WIDTH * SEGMENTS_PER_WALL as f32;
        let start_offset = -total_width / 2.0 + WALL_SEGMENT_WIDTH / 2.0;

        for segment_index in 0..SEGMENTS_PER_WALL {
            if direction == &WallDirection::South && (segment_index == 2) {
                segments.push(WallSegment::default());
                continue;
            }

            let offset = start_offset + segment_index as f32 * WALL_SEGMENT_WIDTH;
            let pos = base_pos + along * offset + nalgebra_glm::vec3(0.0, WALL_HEIGHT / 2.0, 0.0);

            let (scale_x, scale_z) = match direction {
                WallDirection::North | WallDirection::South => (WALL_SEGMENT_WIDTH, WALL_THICKNESS),
                WallDirection::East | WallDirection::West => (WALL_THICKNESS, WALL_SEGMENT_WIDTH),
            };

            let mat_name = format!("wall_{}_{}", wall_index, segment_index);
            create_material(world, &mat_name, [0.82, 0.71, 0.55, 1.0]);

            let entity = spawn_tracked(
                world,
                &mut castle.all_render_entities,
                "Cube",
                pos,
                nalgebra_glm::vec3(scale_x, WALL_HEIGHT, scale_z),
                &mat_name,
            );

            segments.push(WallSegment {
                health: SEGMENT_HP,
                max_health: SEGMENT_HP,
                entity,
                position: pos,
                breached: false,
            });
        }

        castle.walls[wall_index] = WallSide { segments };
    }
}

fn spawn_gate(world: &mut World, castle: &mut CastleState) {
    create_material(world, "gate", [0.45, 0.30, 0.15, 1.0]);
    let gate = spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cube",
        nalgebra_glm::vec3(0.0, WALL_HEIGHT / 2.0, CASTLE_SIZE),
        nalgebra_glm::vec3(GATE_WIDTH, WALL_HEIGHT, WALL_THICKNESS),
        "gate",
    );
    castle.gate_entity = gate;
}

fn spawn_well(world: &mut World, castle: &mut CastleState) {
    create_material(world, "well_stone", [0.6, 0.55, 0.5, 1.0]);
    let well_base = spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cylinder",
        WELL_POS + nalgebra_glm::vec3(0.0, 0.4, 0.0),
        nalgebra_glm::vec3(1.2, 0.8, 1.2),
        "well_stone",
    );

    create_emissive_material(world, "well_water", [0.1, 0.15, 0.4, 1.0], [0.05, 0.1, 0.3]);
    spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cylinder",
        WELL_POS + nalgebra_glm::vec3(0.0, 0.85, 0.0),
        nalgebra_glm::vec3(0.85, 0.05, 0.85),
        "well_water",
    );

    castle.well_entity = well_base;
}

fn spawn_armory(world: &mut World, castle: &mut CastleState) {
    create_material(world, "armory", [0.5, 0.5, 0.5, 1.0]);
    let armory = spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cube",
        ARMORY_POS + nalgebra_glm::vec3(0.0, 1.0, 0.0),
        nalgebra_glm::vec3(2.5, 2.0, 2.0),
        "armory",
    );

    create_material(world, "armory_roof", [0.4, 0.25, 0.15, 1.0]);
    spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cube",
        ARMORY_POS + nalgebra_glm::vec3(0.0, 2.2, 0.0),
        nalgebra_glm::vec3(3.0, 0.3, 2.5),
        "armory_roof",
    );

    create_material(world, "arrow_bundle", [0.45, 0.30, 0.15, 1.0]);
    spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cylinder",
        ARMORY_POS + nalgebra_glm::vec3(1.5, 0.5, 0.0),
        nalgebra_glm::vec3(0.2, 1.0, 0.2),
        "arrow_bundle",
    );
    spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cylinder",
        ARMORY_POS + nalgebra_glm::vec3(1.8, 0.4, 0.3),
        nalgebra_glm::vec3(0.2, 0.8, 0.2),
        "arrow_bundle",
    );

    castle.armory_entity = armory;
}

fn spawn_healing_station(world: &mut World, castle: &mut CastleState) {
    create_material(world, "healing_base", [0.9, 0.9, 0.9, 1.0]);
    let station = spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cube",
        HEALING_POS + nalgebra_glm::vec3(0.0, 0.5, 0.0),
        nalgebra_glm::vec3(1.5, 1.0, 1.5),
        "healing_base",
    );

    create_emissive_material(
        world,
        "healing_cross",
        [0.2, 0.8, 0.3, 1.0],
        [0.3, 1.0, 0.4],
    );
    spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Sphere",
        HEALING_POS + nalgebra_glm::vec3(0.0, 1.3, 0.0),
        nalgebra_glm::vec3(0.5, 0.5, 0.5),
        "healing_cross",
    );

    castle.healing_station_entity = station;
}

fn spawn_repair_pile(world: &mut World, castle: &mut CastleState) {
    create_material(world, "repair_pile", [0.75, 0.65, 0.45, 1.0]);
    let pile = spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cube",
        REPAIR_PILE_POS + nalgebra_glm::vec3(0.0, 0.3, 0.0),
        nalgebra_glm::vec3(1.8, 0.6, 1.2),
        "repair_pile",
    );
    spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cube",
        REPAIR_PILE_POS + nalgebra_glm::vec3(0.5, 0.2, 0.5),
        nalgebra_glm::vec3(0.8, 0.4, 0.6),
        "repair_pile",
    );
    spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Cube",
        REPAIR_PILE_POS + nalgebra_glm::vec3(-0.4, 0.15, -0.3),
        nalgebra_glm::vec3(0.6, 0.3, 0.5),
        "repair_pile",
    );

    castle.repair_pile_entity = pile;
}

fn spawn_river(world: &mut World, castle: &mut CastleState) {
    create_emissive_material(
        world,
        "river_water",
        [0.15, 0.3, 0.6, 1.0],
        [0.05, 0.15, 0.4],
    );
    let river = spawn_tracked(
        world,
        &mut castle.all_render_entities,
        "Plane",
        RIVER_POS + nalgebra_glm::vec3(0.0, -0.1, 0.0),
        nalgebra_glm::vec3(20.0, 1.0, 4.0),
        "river_water",
    );
    castle.river_entity = river;
}

fn spawn_archer_posts(world: &mut World, castle: &mut CastleState) {
    create_material(world, "archer_platform", [0.7, 0.6, 0.45, 1.0]);
    create_material(world, "archer_figure", [0.5, 0.5, 0.55, 1.0]);

    for (index, post_pos) in ARCHER_POST_POSITIONS.iter().enumerate() {
        spawn_tracked(
            world,
            &mut castle.all_render_entities,
            "Cube",
            *post_pos - nalgebra_glm::vec3(0.0, 0.25, 0.0),
            nalgebra_glm::vec3(2.0, 0.5, 2.0),
            "archer_platform",
        );

        spawn_tracked(
            world,
            &mut castle.all_render_entities,
            "Cube",
            *post_pos + nalgebra_glm::vec3(0.0, 0.5, 0.0),
            nalgebra_glm::vec3(0.4, 1.0, 0.3),
            "archer_figure",
        );

        spawn_tracked(
            world,
            &mut castle.all_render_entities,
            "Sphere",
            *post_pos + nalgebra_glm::vec3(0.0, 1.2, 0.0),
            nalgebra_glm::vec3(0.35, 0.35, 0.35),
            "archer_figure",
        );

        castle.archer_posts[index] = ArcherPost {
            position: *post_pos,
            arrows_remaining: 10,
            max_arrows: 10,
            fire_timer: 0.0,
            line_entity: None,
        };
    }
}

fn spawn_siege_engines(world: &mut World, tracker: &mut Vec<Entity>) {
    create_material(world, "siege_base", [0.3, 0.2, 0.1, 1.0]);
    create_material(world, "siege_arm", [0.35, 0.22, 0.12, 1.0]);
    create_material(world, "siege_frame", [0.28, 0.18, 0.08, 1.0]);
    create_emissive_material(
        world,
        "siege_brazier",
        [1.0, 0.5, 0.1, 1.0],
        [1.5, 0.5, 0.05],
    );

    let positions = [
        nalgebra_glm::vec3(20.0, 0.0, 32.0),
        nalgebra_glm::vec3(-20.0, 0.0, 32.0),
        nalgebra_glm::vec3(32.0, 0.0, 8.0),
        nalgebra_glm::vec3(-32.0, 0.0, -5.0),
    ];

    for &position in &positions {
        spawn_tracked(
            world,
            tracker,
            "Cube",
            position + nalgebra_glm::vec3(0.0, 0.4, 0.0),
            nalgebra_glm::vec3(3.5, 0.8, 2.0),
            "siege_base",
        );

        spawn_tracked(
            world,
            tracker,
            "Cube",
            position + nalgebra_glm::vec3(-1.0, 1.6, 0.0),
            nalgebra_glm::vec3(0.4, 2.4, 0.4),
            "siege_frame",
        );
        spawn_tracked(
            world,
            tracker,
            "Cube",
            position + nalgebra_glm::vec3(1.0, 1.6, 0.0),
            nalgebra_glm::vec3(0.4, 2.4, 0.4),
            "siege_frame",
        );

        spawn_tracked(
            world,
            tracker,
            "Cube",
            position + nalgebra_glm::vec3(0.0, 3.0, 0.0),
            nalgebra_glm::vec3(4.0, 0.25, 0.25),
            "siege_arm",
        );

        spawn_tracked(
            world,
            tracker,
            "Cube",
            position + nalgebra_glm::vec3(2.0, 2.5, 0.0),
            nalgebra_glm::vec3(0.8, 0.8, 0.8),
            "siege_arm",
        );

        spawn_tracked(
            world,
            tracker,
            "Sphere",
            position + nalgebra_glm::vec3(0.0, 0.9, 1.2),
            nalgebra_glm::vec3(0.4, 0.4, 0.4),
            "siege_brazier",
        );
        spawn_tracked(
            world,
            tracker,
            "Sphere",
            position + nalgebra_glm::vec3(0.0, 0.9, -1.2),
            nalgebra_glm::vec3(0.4, 0.4, 0.4),
            "siege_brazier",
        );
    }
}

pub fn location_position(location: LocationId) -> Vec3 {
    match location {
        LocationId::Well => WELL_POS,
        LocationId::Armory => ARMORY_POS,
        LocationId::HealingStation => HEALING_POS,
        LocationId::RepairPile => REPAIR_PILE_POS,
        LocationId::Gate => GATE_POS,
        LocationId::River => RIVER_POS,
        LocationId::ArcherPost(index) => {
            if index < 4 {
                ARCHER_POST_POSITIONS[index]
            } else {
                Vec3::zeros()
            }
        }
        LocationId::WallNorth => nalgebra_glm::vec3(0.0, 0.0, -CASTLE_SIZE),
        LocationId::WallSouth => nalgebra_glm::vec3(0.0, 0.0, CASTLE_SIZE),
        LocationId::WallEast => nalgebra_glm::vec3(CASTLE_SIZE, 0.0, 0.0),
        LocationId::WallWest => nalgebra_glm::vec3(-CASTLE_SIZE, 0.0, 0.0),
    }
}
