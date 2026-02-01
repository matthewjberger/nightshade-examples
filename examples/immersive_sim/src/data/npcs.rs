use nightshade::prelude::*;

pub struct NpcDefinition {
    pub name: &'static str,
    pub position: Vec3,
    pub color: [f32; 4],
    pub dialogue_id: usize,
}

pub const NPC_DEFINITIONS: &[NpcDefinition] = &[
    NpcDefinition {
        name: "Guard",
        position: Vec3::new(0.0, 3.0, 5.0),
        color: [0.6, 0.3, 0.3, 1.0],
        dialogue_id: 0,
    },
    NpcDefinition {
        name: "Merchant",
        position: Vec3::new(-3.0, 3.0, 4.0),
        color: [0.3, 0.5, 0.3, 1.0],
        dialogue_id: 1,
    },
    NpcDefinition {
        name: "Scholar",
        position: Vec3::new(3.0, 3.0, 4.0),
        color: [0.3, 0.3, 0.6, 1.0],
        dialogue_id: 2,
    },
];
