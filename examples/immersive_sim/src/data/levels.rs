use crate::data::enemies::{EnemySpawn, EnemyType};
use crate::data::items::ItemType;
use nightshade::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LevelId {
    Hub,
    Dungeon,
    Forest,
    Castle,
    FinalArena,
}

pub struct LevelDefinition {
    pub id: LevelId,
    pub fog_color: [f32; 3],
    pub fog_start: f32,
    pub fog_end: f32,
    pub player_spawn: Vec3,
    pub geometry: Vec<LevelGeometry>,
    pub item_spawns: Vec<ItemSpawn>,
    pub enemy_spawns: Vec<EnemySpawn>,
    pub portals: Vec<Portal>,
    pub lights: Vec<LevelLight>,
}

#[derive(Clone)]
pub struct LevelGeometry {
    pub mesh: &'static str,
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: f32,
    pub color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub emissive: f32,
}

#[derive(Clone)]
pub struct ItemSpawn {
    pub position: Vec3,
    pub item_type: ItemType,
    pub quantity: usize,
}

#[derive(Clone)]
pub struct Portal {
    pub position: Vec3,
    pub target_level: LevelId,
    pub color: [f32; 4],
    pub requires_key: bool,
}

#[derive(Clone)]
pub struct LevelLight {
    pub position: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub is_spotlight: bool,
    pub direction: Option<Vec3>,
}

pub fn get_level(level_id: LevelId) -> LevelDefinition {
    match level_id {
        LevelId::Hub => hub_level(),
        LevelId::Dungeon => dungeon_level(),
        LevelId::Forest => forest_level(),
        LevelId::Castle => castle_level(),
        LevelId::FinalArena => final_arena_level(),
    }
}

fn hub_level() -> LevelDefinition {
    LevelDefinition {
        id: LevelId::Hub,
        fog_color: [0.4, 0.45, 0.5],
        fog_start: 20.0,
        fog_end: 50.0,
        player_spawn: Vec3::new(0.0, 1.0, 0.0),
        geometry: vec![
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(0.0, -0.5, 0.0),
                scale: Vec3::new(40.0, 1.0, 40.0),
                rotation: 0.0,
                color: [0.3, 0.35, 0.3, 1.0],
                roughness: 0.9,
                metallic: 0.0,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(0.0, 3.0, 0.0),
                scale: Vec3::new(2.0, 6.0, 2.0),
                rotation: 0.0,
                color: [0.5, 0.5, 0.55, 1.0],
                roughness: 0.5,
                metallic: 0.3,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Sphere",
                position: Vec3::new(0.0, 7.0, 0.0),
                scale: Vec3::new(1.5, 1.5, 1.5),
                rotation: 0.0,
                color: [0.9, 0.8, 0.3, 1.0],
                roughness: 0.2,
                metallic: 0.8,
                emissive: 2.0,
            },
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(-15.0, 2.5, 0.0),
                scale: Vec3::new(1.0, 5.0, 8.0),
                rotation: 0.0,
                color: [0.4, 0.35, 0.3, 1.0],
                roughness: 0.8,
                metallic: 0.1,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(15.0, 2.5, 0.0),
                scale: Vec3::new(1.0, 5.0, 8.0),
                rotation: 0.0,
                color: [0.4, 0.35, 0.3, 1.0],
                roughness: 0.8,
                metallic: 0.1,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(0.0, 2.5, -15.0),
                scale: Vec3::new(8.0, 5.0, 1.0),
                rotation: 0.0,
                color: [0.4, 0.35, 0.3, 1.0],
                roughness: 0.8,
                metallic: 0.1,
                emissive: 0.0,
            },
        ],
        item_spawns: vec![
            ItemSpawn {
                position: Vec3::new(5.0, 0.5, 5.0),
                item_type: ItemType::HealthPotion,
                quantity: 2,
            },
            ItemSpawn {
                position: Vec3::new(-5.0, 0.5, 5.0),
                item_type: ItemType::ManaPotion,
                quantity: 2,
            },
            ItemSpawn {
                position: Vec3::new(0.0, 0.5, 8.0),
                item_type: ItemType::Coin,
                quantity: 10,
            },
        ],
        enemy_spawns: vec![],
        portals: vec![
            Portal {
                position: Vec3::new(10.0, 1.0, 10.0),
                target_level: LevelId::Dungeon,
                color: [0.8, 0.3, 0.3, 1.0],
                requires_key: false,
            },
            Portal {
                position: Vec3::new(-10.0, 1.0, 10.0),
                target_level: LevelId::Forest,
                color: [0.3, 0.8, 0.3, 1.0],
                requires_key: false,
            },
            Portal {
                position: Vec3::new(0.0, 1.0, -10.0),
                target_level: LevelId::Castle,
                color: [0.3, 0.3, 0.8, 1.0],
                requires_key: true,
            },
        ],
        lights: vec![
            LevelLight {
                position: Vec3::new(0.0, 8.0, 0.0),
                color: [1.0, 0.9, 0.7],
                intensity: 5.0,
                range: 20.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(10.0, 3.0, 10.0),
                color: [1.0, 0.3, 0.3],
                intensity: 3.0,
                range: 8.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(-10.0, 3.0, 10.0),
                color: [0.3, 1.0, 0.3],
                intensity: 3.0,
                range: 8.0,
                is_spotlight: false,
                direction: None,
            },
        ],
    }
}

fn dungeon_level() -> LevelDefinition {
    LevelDefinition {
        id: LevelId::Dungeon,
        fog_color: [0.05, 0.03, 0.03],
        fog_start: 5.0,
        fog_end: 25.0,
        player_spawn: Vec3::new(0.0, 1.0, 0.0),
        geometry: vec![
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(0.0, -0.5, 0.0),
                scale: Vec3::new(50.0, 1.0, 50.0),
                rotation: 0.0,
                color: [0.15, 0.12, 0.1, 1.0],
                roughness: 0.95,
                metallic: 0.0,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(0.0, 10.0, 0.0),
                scale: Vec3::new(50.0, 1.0, 50.0),
                rotation: 0.0,
                color: [0.1, 0.08, 0.06, 1.0],
                roughness: 0.95,
                metallic: 0.0,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(-20.0, 5.0, 0.0),
                scale: Vec3::new(1.0, 10.0, 40.0),
                rotation: 0.0,
                color: [0.2, 0.18, 0.15, 1.0],
                roughness: 0.9,
                metallic: 0.1,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(20.0, 5.0, 0.0),
                scale: Vec3::new(1.0, 10.0, 40.0),
                rotation: 0.0,
                color: [0.2, 0.18, 0.15, 1.0],
                roughness: 0.9,
                metallic: 0.1,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(-10.0, 5.0, -10.0),
                scale: Vec3::new(1.5, 10.0, 1.5),
                rotation: 0.0,
                color: [0.25, 0.22, 0.18, 1.0],
                roughness: 0.8,
                metallic: 0.2,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(10.0, 5.0, -10.0),
                scale: Vec3::new(1.5, 10.0, 1.5),
                rotation: 0.0,
                color: [0.25, 0.22, 0.18, 1.0],
                roughness: 0.8,
                metallic: 0.2,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(-10.0, 5.0, 10.0),
                scale: Vec3::new(1.5, 10.0, 1.5),
                rotation: 0.0,
                color: [0.25, 0.22, 0.18, 1.0],
                roughness: 0.8,
                metallic: 0.2,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(10.0, 5.0, 10.0),
                scale: Vec3::new(1.5, 10.0, 1.5),
                rotation: 0.0,
                color: [0.25, 0.22, 0.18, 1.0],
                roughness: 0.8,
                metallic: 0.2,
                emissive: 0.0,
            },
        ],
        item_spawns: vec![
            ItemSpawn {
                position: Vec3::new(-15.0, 0.5, -15.0),
                item_type: ItemType::HealthPotion,
                quantity: 1,
            },
            ItemSpawn {
                position: Vec3::new(15.0, 0.5, -15.0),
                item_type: ItemType::ManaPotion,
                quantity: 1,
            },
            ItemSpawn {
                position: Vec3::new(0.0, 0.5, 15.0),
                item_type: ItemType::Key,
                quantity: 1,
            },
            ItemSpawn {
                position: Vec3::new(-8.0, 0.5, 0.0),
                item_type: ItemType::Coin,
                quantity: 25,
            },
            ItemSpawn {
                position: Vec3::new(8.0, 0.5, 0.0),
                item_type: ItemType::Gem,
                quantity: 5,
            },
        ],
        enemy_spawns: vec![
            EnemySpawn {
                position: Vec3::new(-10.0, 1.0, 5.0),
                enemy_type: EnemyType::Grunt,
            },
            EnemySpawn {
                position: Vec3::new(10.0, 1.0, 5.0),
                enemy_type: EnemyType::Grunt,
            },
            EnemySpawn {
                position: Vec3::new(0.0, 1.0, -10.0),
                enemy_type: EnemyType::Archer,
            },
            EnemySpawn {
                position: Vec3::new(-15.0, 1.0, 10.0),
                enemy_type: EnemyType::Grunt,
            },
            EnemySpawn {
                position: Vec3::new(15.0, 1.0, 10.0),
                enemy_type: EnemyType::Mage,
            },
        ],
        portals: vec![Portal {
            position: Vec3::new(0.0, 1.0, -18.0),
            target_level: LevelId::Hub,
            color: [0.5, 0.5, 0.8, 1.0],
            requires_key: false,
        }],
        lights: vec![
            LevelLight {
                position: Vec3::new(0.0, 8.0, 0.0),
                color: [1.0, 0.6, 0.3],
                intensity: 2.0,
                range: 15.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(-10.0, 3.0, -10.0),
                color: [1.0, 0.5, 0.2],
                intensity: 1.5,
                range: 8.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(10.0, 3.0, -10.0),
                color: [1.0, 0.5, 0.2],
                intensity: 1.5,
                range: 8.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(-10.0, 3.0, 10.0),
                color: [1.0, 0.5, 0.2],
                intensity: 1.5,
                range: 8.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(10.0, 3.0, 10.0),
                color: [1.0, 0.5, 0.2],
                intensity: 1.5,
                range: 8.0,
                is_spotlight: false,
                direction: None,
            },
        ],
    }
}

fn forest_level() -> LevelDefinition {
    let mut geometry = vec![LevelGeometry {
        mesh: "Cube",
        position: Vec3::new(0.0, -0.5, 0.0),
        scale: Vec3::new(80.0, 1.0, 80.0),
        rotation: 0.0,
        color: [0.15, 0.25, 0.1, 1.0],
        roughness: 0.95,
        metallic: 0.0,
        emissive: 0.0,
    }];

    for index in 0..20 {
        let angle = (index as f32 / 20.0) * std::f32::consts::TAU;
        let radius = 15.0 + (index as f32 * 0.5);
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        let height = 4.0 + (index as f32 % 3.0);

        geometry.push(LevelGeometry {
            mesh: "Cylinder",
            position: Vec3::new(x, height / 2.0, z),
            scale: Vec3::new(0.4, height, 0.4),
            rotation: 0.0,
            color: [0.35, 0.25, 0.15, 1.0],
            roughness: 0.9,
            metallic: 0.0,
            emissive: 0.0,
        });

        geometry.push(LevelGeometry {
            mesh: "Cone",
            position: Vec3::new(x, height + 1.5, z),
            scale: Vec3::new(2.0, 3.0, 2.0),
            rotation: 0.0,
            color: [0.1, 0.4, 0.15, 1.0],
            roughness: 0.9,
            metallic: 0.0,
            emissive: 0.0,
        });
    }

    LevelDefinition {
        id: LevelId::Forest,
        fog_color: [0.3, 0.4, 0.3],
        fog_start: 15.0,
        fog_end: 40.0,
        player_spawn: Vec3::new(0.0, 1.0, 0.0),
        geometry,
        item_spawns: vec![
            ItemSpawn {
                position: Vec3::new(5.0, 0.5, 5.0),
                item_type: ItemType::HealthPotion,
                quantity: 1,
            },
            ItemSpawn {
                position: Vec3::new(-5.0, 0.5, 5.0),
                item_type: ItemType::ManaPotion,
                quantity: 1,
            },
            ItemSpawn {
                position: Vec3::new(10.0, 0.5, 0.0),
                item_type: ItemType::SpeedPotion,
                quantity: 1,
            },
            ItemSpawn {
                position: Vec3::new(-8.0, 0.5, -8.0),
                item_type: ItemType::Gem,
                quantity: 10,
            },
            ItemSpawn {
                position: Vec3::new(0.0, 0.5, -12.0),
                item_type: ItemType::Staff,
                quantity: 1,
            },
        ],
        enemy_spawns: vec![
            EnemySpawn {
                position: Vec3::new(8.0, 1.0, 8.0),
                enemy_type: EnemyType::Archer,
            },
            EnemySpawn {
                position: Vec3::new(-8.0, 1.0, 8.0),
                enemy_type: EnemyType::Archer,
            },
            EnemySpawn {
                position: Vec3::new(0.0, 1.0, -8.0),
                enemy_type: EnemyType::Mage,
            },
            EnemySpawn {
                position: Vec3::new(12.0, 1.0, 0.0),
                enemy_type: EnemyType::Grunt,
            },
            EnemySpawn {
                position: Vec3::new(-12.0, 1.0, 0.0),
                enemy_type: EnemyType::Grunt,
            },
        ],
        portals: vec![Portal {
            position: Vec3::new(0.0, 1.0, 25.0),
            target_level: LevelId::Hub,
            color: [0.5, 0.8, 0.5, 1.0],
            requires_key: false,
        }],
        lights: vec![
            LevelLight {
                position: Vec3::new(0.0, 10.0, 0.0),
                color: [0.8, 1.0, 0.8],
                intensity: 3.0,
                range: 25.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(10.0, 2.0, 10.0),
                color: [0.3, 1.0, 0.5],
                intensity: 1.5,
                range: 6.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(-10.0, 2.0, 10.0),
                color: [0.5, 1.0, 0.3],
                intensity: 1.5,
                range: 6.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(0.0, 2.0, -10.0),
                color: [1.0, 0.8, 0.3],
                intensity: 2.0,
                range: 8.0,
                is_spotlight: false,
                direction: None,
            },
        ],
    }
}

fn castle_level() -> LevelDefinition {
    LevelDefinition {
        id: LevelId::Castle,
        fog_color: [0.08, 0.05, 0.1],
        fog_start: 10.0,
        fog_end: 35.0,
        player_spawn: Vec3::new(0.0, 1.0, 20.0),
        geometry: vec![
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(0.0, -0.5, 0.0),
                scale: Vec3::new(60.0, 1.0, 60.0),
                rotation: 0.0,
                color: [0.2, 0.18, 0.22, 1.0],
                roughness: 0.85,
                metallic: 0.1,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(-25.0, 7.5, 0.0),
                scale: Vec3::new(2.0, 15.0, 50.0),
                rotation: 0.0,
                color: [0.25, 0.22, 0.28, 1.0],
                roughness: 0.8,
                metallic: 0.15,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(25.0, 7.5, 0.0),
                scale: Vec3::new(2.0, 15.0, 50.0),
                rotation: 0.0,
                color: [0.25, 0.22, 0.28, 1.0],
                roughness: 0.8,
                metallic: 0.15,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cube",
                position: Vec3::new(0.0, 7.5, -25.0),
                scale: Vec3::new(50.0, 15.0, 2.0),
                rotation: 0.0,
                color: [0.25, 0.22, 0.28, 1.0],
                roughness: 0.8,
                metallic: 0.15,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(0.0, 2.0, -15.0),
                scale: Vec3::new(5.0, 4.0, 5.0),
                rotation: 0.0,
                color: [0.15, 0.1, 0.2, 1.0],
                roughness: 0.6,
                metallic: 0.3,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Sphere",
                position: Vec3::new(0.0, 6.0, -15.0),
                scale: Vec3::new(2.0, 2.0, 2.0),
                rotation: 0.0,
                color: [0.5, 0.0, 0.8, 1.0],
                roughness: 0.2,
                metallic: 0.8,
                emissive: 3.0,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(-20.0, 10.0, -20.0),
                scale: Vec3::new(3.0, 20.0, 3.0),
                rotation: 0.0,
                color: [0.3, 0.25, 0.35, 1.0],
                roughness: 0.7,
                metallic: 0.2,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(20.0, 10.0, -20.0),
                scale: Vec3::new(3.0, 20.0, 3.0),
                rotation: 0.0,
                color: [0.3, 0.25, 0.35, 1.0],
                roughness: 0.7,
                metallic: 0.2,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cone",
                position: Vec3::new(-20.0, 22.0, -20.0),
                scale: Vec3::new(4.0, 6.0, 4.0),
                rotation: 0.0,
                color: [0.2, 0.15, 0.25, 1.0],
                roughness: 0.8,
                metallic: 0.1,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cone",
                position: Vec3::new(20.0, 22.0, -20.0),
                scale: Vec3::new(4.0, 6.0, 4.0),
                rotation: 0.0,
                color: [0.2, 0.15, 0.25, 1.0],
                roughness: 0.8,
                metallic: 0.1,
                emissive: 0.0,
            },
        ],
        item_spawns: vec![
            ItemSpawn {
                position: Vec3::new(-15.0, 0.5, 0.0),
                item_type: ItemType::HealthPotion,
                quantity: 3,
            },
            ItemSpawn {
                position: Vec3::new(15.0, 0.5, 0.0),
                item_type: ItemType::ManaPotion,
                quantity: 3,
            },
            ItemSpawn {
                position: Vec3::new(0.0, 0.5, 10.0),
                item_type: ItemType::Sword,
                quantity: 1,
            },
            ItemSpawn {
                position: Vec3::new(-10.0, 0.5, -10.0),
                item_type: ItemType::Shield,
                quantity: 1,
            },
            ItemSpawn {
                position: Vec3::new(10.0, 0.5, -10.0),
                item_type: ItemType::Scroll,
                quantity: 1,
            },
        ],
        enemy_spawns: vec![
            EnemySpawn {
                position: Vec3::new(-10.0, 1.0, 10.0),
                enemy_type: EnemyType::Grunt,
            },
            EnemySpawn {
                position: Vec3::new(10.0, 1.0, 10.0),
                enemy_type: EnemyType::Grunt,
            },
            EnemySpawn {
                position: Vec3::new(-15.0, 1.0, -5.0),
                enemy_type: EnemyType::Archer,
            },
            EnemySpawn {
                position: Vec3::new(15.0, 1.0, -5.0),
                enemy_type: EnemyType::Archer,
            },
            EnemySpawn {
                position: Vec3::new(0.0, 1.0, 0.0),
                enemy_type: EnemyType::Brute,
            },
            EnemySpawn {
                position: Vec3::new(-8.0, 1.0, -15.0),
                enemy_type: EnemyType::Mage,
            },
            EnemySpawn {
                position: Vec3::new(8.0, 1.0, -15.0),
                enemy_type: EnemyType::Mage,
            },
        ],
        portals: vec![
            Portal {
                position: Vec3::new(0.0, 1.0, 25.0),
                target_level: LevelId::Hub,
                color: [0.5, 0.5, 0.8, 1.0],
                requires_key: false,
            },
            Portal {
                position: Vec3::new(0.0, 1.0, -22.0),
                target_level: LevelId::FinalArena,
                color: [0.8, 0.0, 0.8, 1.0],
                requires_key: false,
            },
        ],
        lights: vec![
            LevelLight {
                position: Vec3::new(0.0, 6.0, -15.0),
                color: [0.8, 0.0, 1.0],
                intensity: 5.0,
                range: 15.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(-20.0, 15.0, -20.0),
                color: [0.6, 0.3, 0.8],
                intensity: 2.0,
                range: 10.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(20.0, 15.0, -20.0),
                color: [0.6, 0.3, 0.8],
                intensity: 2.0,
                range: 10.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(0.0, 5.0, 10.0),
                color: [1.0, 0.5, 0.3],
                intensity: 2.0,
                range: 12.0,
                is_spotlight: false,
                direction: None,
            },
        ],
    }
}

fn final_arena_level() -> LevelDefinition {
    LevelDefinition {
        id: LevelId::FinalArena,
        fog_color: [0.1, 0.0, 0.15],
        fog_start: 20.0,
        fog_end: 50.0,
        player_spawn: Vec3::new(0.0, 1.0, 15.0),
        geometry: vec![
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(0.0, -0.5, 0.0),
                scale: Vec3::new(30.0, 1.0, 30.0),
                rotation: 0.0,
                color: [0.1, 0.05, 0.12, 1.0],
                roughness: 0.7,
                metallic: 0.3,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(0.0, 0.1, 0.0),
                scale: Vec3::new(25.0, 0.2, 25.0),
                rotation: 0.0,
                color: [0.8, 0.0, 0.0, 1.0],
                roughness: 0.5,
                metallic: 0.5,
                emissive: 0.5,
            },
            LevelGeometry {
                mesh: "Cylinder",
                position: Vec3::new(0.0, 5.0, -12.0),
                scale: Vec3::new(3.0, 10.0, 3.0),
                rotation: 0.0,
                color: [0.15, 0.1, 0.2, 1.0],
                roughness: 0.6,
                metallic: 0.4,
                emissive: 0.0,
            },
            LevelGeometry {
                mesh: "Sphere",
                position: Vec3::new(0.0, 12.0, -12.0),
                scale: Vec3::new(3.0, 3.0, 3.0),
                rotation: 0.0,
                color: [1.0, 0.0, 0.5, 1.0],
                roughness: 0.1,
                metallic: 0.9,
                emissive: 5.0,
            },
        ],
        item_spawns: vec![
            ItemSpawn {
                position: Vec3::new(-8.0, 0.5, 8.0),
                item_type: ItemType::HealthPotion,
                quantity: 5,
            },
            ItemSpawn {
                position: Vec3::new(8.0, 0.5, 8.0),
                item_type: ItemType::ManaPotion,
                quantity: 5,
            },
            ItemSpawn {
                position: Vec3::new(0.0, 0.5, 10.0),
                item_type: ItemType::SpeedPotion,
                quantity: 2,
            },
        ],
        enemy_spawns: vec![EnemySpawn {
            position: Vec3::new(0.0, 1.0, -8.0),
            enemy_type: EnemyType::Boss,
        }],
        portals: vec![Portal {
            position: Vec3::new(0.0, 1.0, -12.0),
            target_level: LevelId::Hub,
            color: [1.0, 1.0, 0.0, 1.0],
            requires_key: false,
        }],
        lights: vec![
            LevelLight {
                position: Vec3::new(0.0, 12.0, -12.0),
                color: [1.0, 0.0, 0.5],
                intensity: 8.0,
                range: 25.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(0.0, 3.0, 0.0),
                color: [1.0, 0.2, 0.2],
                intensity: 3.0,
                range: 20.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(-10.0, 2.0, 0.0),
                color: [0.8, 0.0, 0.8],
                intensity: 2.0,
                range: 8.0,
                is_spotlight: false,
                direction: None,
            },
            LevelLight {
                position: Vec3::new(10.0, 2.0, 0.0),
                color: [0.8, 0.0, 0.8],
                intensity: 2.0,
                range: 8.0,
                is_spotlight: false,
                direction: None,
            },
        ],
    }
}
