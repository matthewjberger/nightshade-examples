use nightshade::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NpcType(pub u32);

pub const NPC_CEDAR: NpcType = NpcType(0);
pub const NPC_MASON: NpcType = NpcType(1);
pub const NPC_FERN: NpcType = NpcType(2);
pub const NPC_WILLOW: NpcType = NpcType(3);

pub struct NpcDefinition {
    pub npc_type: NpcType,
    pub name: &'static str,
    pub position: Vec3,
    pub color: [f32; 4],
    pub dialogue: &'static [&'static str],
    pub is_shop_keeper: bool,
}

pub const NPC_DEFINITIONS: &[NpcDefinition] = &[
    NpcDefinition {
        npc_type: NPC_CEDAR,
        name: "Cedar",
        position: Vec3::new(8.0, 0.0, 5.0),
        color: [0.7, 0.4, 0.3, 1.0],
        dialogue: &[
            "Hello there, farmer!",
            "Need any buildings constructed?",
            "I'm the local carpenter.",
        ],
        is_shop_keeper: false,
    },
    NpcDefinition {
        npc_type: NPC_MASON,
        name: "Mason",
        position: Vec3::new(-6.0, 0.0, 10.0),
        color: [0.3, 0.5, 0.3, 1.0],
        dialogue: &[
            "Welcome to my shop!",
            "I sell seeds for every season.",
            "Come back anytime!",
        ],
        is_shop_keeper: true,
    },
    NpcDefinition {
        npc_type: NPC_FERN,
        name: "Fern",
        position: Vec3::new(15.0, 0.0, -8.0),
        color: [0.5, 0.45, 0.4, 1.0],
        dialogue: &[
            "Ah, you're the new farmer.",
            "I live out here in nature.",
            "The forest provides everything I need.",
        ],
        is_shop_keeper: false,
    },
    NpcDefinition {
        npc_type: NPC_WILLOW,
        name: "Willow",
        position: Vec3::new(-10.0, 0.0, -5.0),
        color: [0.6, 0.4, 0.7, 1.0],
        dialogue: &[
            "Hey! Nice to meet you.",
            "I love exploring the caves nearby.",
            "Sometimes I wander the fields at night.",
        ],
        is_shop_keeper: false,
    },
];

pub fn get_npc_definition(npc_type: NpcType) -> Option<&'static NpcDefinition> {
    NPC_DEFINITIONS.iter().find(|d| d.npc_type == npc_type)
}

pub fn get_shop_keeper_position() -> Option<Vec3> {
    NPC_DEFINITIONS
        .iter()
        .find(|npc| npc.is_shop_keeper)
        .map(|npc| npc.position)
}

pub fn get_shop_keeper_name() -> &'static str {
    NPC_DEFINITIONS
        .iter()
        .find(|npc| npc.is_shop_keeper)
        .map(|npc| npc.name)
        .unwrap_or("Shop")
}
