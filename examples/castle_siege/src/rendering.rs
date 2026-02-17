use nightshade::prelude::*;

use crate::agent::{AgentBody, CarriedItem};

const AGENT_COLOR: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const WOUNDED_COLOR: [f32; 4] = [0.7, 0.3, 0.3, 1.0];
const WATER_COLOR: [f32; 4] = [0.2, 0.4, 0.8, 1.0];
const REPAIR_COLOR: [f32; 4] = [0.75, 0.65, 0.45, 1.0];
const ARROW_COLOR: [f32; 4] = [0.45, 0.30, 0.15, 1.0];
const BOULDER_COLOR: [f32; 4] = [0.85, 0.45, 0.15, 1.0];
const FIRE_COLORS: [[f32; 4]; 3] = [
    [1.0, 0.6, 0.1, 1.0],
    [1.0, 0.3, 0.05, 1.0],
    [1.0, 0.8, 0.2, 1.0],
];
const RUBBLE_COLOR: [f32; 4] = [0.5, 0.45, 0.35, 1.0];
const REPLAN_COLOR: [f32; 4] = [1.0, 0.0, 1.0, 0.6];
const INVADER_COLOR: [f32; 4] = [0.6, 0.1, 0.1, 1.0];
const SMOKE_COLOR: [f32; 4] = [0.15, 0.12, 0.1, 0.7];
const CHARRED_COLOR: [f32; 4] = [0.15, 0.1, 0.08, 1.0];

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

fn spawn_mesh_with_material(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material_name: &str,
) -> Entity {
    let entity = spawn_mesh_at(world, mesh_name, position, scale);
    world.set_material_ref(entity, MaterialRef::new(material_name.to_string()));
    entity
}

pub fn init_shared_materials(world: &mut World) {
    create_emissive_material(
        world,
        "trail_particle",
        [1.0, 0.6, 0.2, 0.8],
        [1.0, 0.3, 0.05],
    );
    create_emissive_material(world, "impact_flash", [1.0, 0.9, 0.7, 0.9], [4.0, 3.0, 1.5]);
    create_emissive_material(world, "boulder_mat", BOULDER_COLOR, [1.5, 0.4, 0.05]);
    create_material(world, "smoke_mat", SMOKE_COLOR);
    create_emissive_material(world, "replan_ring_mat", REPLAN_COLOR, [2.0, 0.0, 2.0]);
    create_material(world, "carried_water", WATER_COLOR);
    create_material(world, "carried_repair", REPAIR_COLOR);
    create_material(world, "carried_arrows", ARROW_COLOR);
    create_material(world, "rubble_mat", RUBBLE_COLOR);
    for (index, &color) in FIRE_COLORS.iter().enumerate() {
        create_emissive_material(
            world,
            &format!("fire_color_{}", index),
            color,
            [2.0, 1.0, 0.2],
        );
    }
}

pub fn spawn_agent_body(world: &mut World, agent_index: usize, position: Vec3) -> AgentBody {
    let prefix = format!("agent_{}", agent_index);

    create_material(world, &format!("{}_head", prefix), AGENT_COLOR);
    create_material(world, &format!("{}_torso", prefix), AGENT_COLOR);
    create_material(world, &format!("{}_larm", prefix), AGENT_COLOR);
    create_material(world, &format!("{}_rarm", prefix), AGENT_COLOR);
    create_material(world, &format!("{}_lleg", prefix), AGENT_COLOR);
    create_material(world, &format!("{}_rleg", prefix), AGENT_COLOR);
    create_emissive_material(
        world,
        &format!("{}_marker", prefix),
        [0.5, 0.5, 0.5, 1.0],
        [0.3, 0.3, 0.3],
    );

    let head = spawn_mesh_with_material(
        world,
        "Sphere",
        position + nalgebra_glm::vec3(0.0, 1.3, 0.0),
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        &format!("{}_head", prefix),
    );

    let torso = spawn_mesh_with_material(
        world,
        "Cube",
        position + nalgebra_glm::vec3(0.0, 0.85, 0.0),
        nalgebra_glm::vec3(0.4, 0.5, 0.25),
        &format!("{}_torso", prefix),
    );

    let left_arm = spawn_mesh_with_material(
        world,
        "Cube",
        position + nalgebra_glm::vec3(-0.32, 0.85, 0.0),
        nalgebra_glm::vec3(0.12, 0.4, 0.12),
        &format!("{}_larm", prefix),
    );

    let right_arm = spawn_mesh_with_material(
        world,
        "Cube",
        position + nalgebra_glm::vec3(0.32, 0.85, 0.0),
        nalgebra_glm::vec3(0.12, 0.4, 0.12),
        &format!("{}_rarm", prefix),
    );

    let left_leg = spawn_mesh_with_material(
        world,
        "Cube",
        position + nalgebra_glm::vec3(-0.15, 0.25, 0.0),
        nalgebra_glm::vec3(0.14, 0.45, 0.14),
        &format!("{}_lleg", prefix),
    );

    let right_leg = spawn_mesh_with_material(
        world,
        "Cube",
        position + nalgebra_glm::vec3(0.15, 0.25, 0.0),
        nalgebra_glm::vec3(0.14, 0.45, 0.14),
        &format!("{}_rleg", prefix),
    );

    let goal_marker = spawn_mesh_with_material(
        world,
        "Sphere",
        position + nalgebra_glm::vec3(0.0, 1.7, 0.0),
        nalgebra_glm::vec3(0.18, 0.18, 0.18),
        &format!("{}_marker", prefix),
    );

    AgentBody {
        head,
        torso,
        left_arm,
        right_arm,
        left_leg,
        right_leg,
        goal_marker,
    }
}

pub fn update_agent_body_position(
    world: &mut World,
    body: &AgentBody,
    position: Vec3,
    time: f32,
    is_moving: bool,
) {
    let walk_phase = if is_moving {
        (time * 8.0).sin() * 0.15
    } else {
        0.0
    };
    let arm_swing = if is_moving {
        (time * 8.0).sin() * 0.1
    } else {
        0.0
    };
    let torso_bob = if is_moving {
        (time * 16.0).sin().abs() * 0.04
    } else {
        0.0
    };

    let parts: [(Entity, Vec3); 7] = [
        (body.head, nalgebra_glm::vec3(0.0, 1.3 + torso_bob, 0.0)),
        (body.torso, nalgebra_glm::vec3(0.0, 0.85 + torso_bob, 0.0)),
        (
            body.left_arm,
            nalgebra_glm::vec3(-0.32, 0.85 + arm_swing + torso_bob, 0.0),
        ),
        (
            body.right_arm,
            nalgebra_glm::vec3(0.32, 0.85 - arm_swing + torso_bob, 0.0),
        ),
        (
            body.left_leg,
            nalgebra_glm::vec3(-0.15, 0.25 + walk_phase, 0.0),
        ),
        (
            body.right_leg,
            nalgebra_glm::vec3(0.15, 0.25 - walk_phase, 0.0),
        ),
        (
            body.goal_marker,
            nalgebra_glm::vec3(0.0, 1.7 + torso_bob, 0.0),
        ),
    ];

    for (entity, offset) in &parts {
        if let Some(transform) = world.get_local_transform_mut(*entity) {
            transform.translation = position + offset;
        }
        world.set_local_transform_dirty(*entity, LocalTransformDirty);
    }
}

pub fn update_goal_marker_color(
    world: &mut World,
    agent_index: usize,
    color: [f32; 4],
    emissive: [f32; 3],
) {
    let mat_name = format!("agent_{}_marker", agent_index);
    update_material_color(world, &mat_name, color);
    update_material_emissive(world, &mat_name, emissive);
}

pub fn set_agent_color(world: &mut World, _body: &AgentBody, agent_index: usize, color: [f32; 4]) {
    let prefix = format!("agent_{}", agent_index);
    let part_names = ["head", "torso", "larm", "rarm", "lleg", "rleg"];
    for part_name in &part_names {
        let mat_name = format!("{}_{}", prefix, part_name);
        update_material_color(world, &mat_name, color);
    }
}

pub fn set_agent_emissive(world: &mut World, agent_index: usize, emissive: [f32; 3]) {
    let prefix = format!("agent_{}", agent_index);
    let part_names = ["head", "torso", "larm", "rarm", "lleg", "rleg"];
    for part_name in &part_names {
        let mat_name = format!("{}_{}", prefix, part_name);
        update_material_emissive(world, &mat_name, emissive);
    }
}

pub fn set_agent_wounded_color(world: &mut World, body: &AgentBody, agent_index: usize) {
    set_agent_color(world, body, agent_index, WOUNDED_COLOR);
}

pub fn set_agent_healthy_color(world: &mut World, body: &AgentBody, agent_index: usize) {
    set_agent_color(world, body, agent_index, AGENT_COLOR);
}

fn update_material_color(world: &mut World, name: &str, color: [f32; 4]) {
    use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
    if let Some(material) =
        registry_entry_by_name_mut(&mut world.resources.material_registry.registry, name)
    {
        material.base_color = color;
    }
}

fn update_material_emissive(world: &mut World, name: &str, emissive: [f32; 3]) {
    use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
    if let Some(material) =
        registry_entry_by_name_mut(&mut world.resources.material_registry.registry, name)
    {
        material.emissive_factor = emissive;
    }
}

pub fn spawn_boulder(world: &mut World, position: Vec3) -> Entity {
    spawn_mesh_with_material(
        world,
        "Sphere",
        position,
        nalgebra_glm::vec3(1.2, 1.2, 1.2),
        "boulder_mat",
    )
}

pub fn spawn_fire_cluster(world: &mut World, position: Vec3, seed: u64) -> Vec<Entity> {
    let mut entities = Vec::new();
    let count = 3 + (seed % 3) as usize;

    for index in 0..count {
        let mat_name = format!("fire_color_{}", index % FIRE_COLORS.len());

        let offset = nalgebra_glm::vec3(
            ((seed.wrapping_mul(index as u64 + 1)) % 100) as f32 / 100.0 - 0.5,
            0.2 + (index as f32) * 0.15,
            ((seed.wrapping_mul(index as u64 + 3)) % 100) as f32 / 100.0 - 0.5,
        );

        let scale = 0.2 + (index as f32) * 0.05;
        let entity = spawn_mesh_with_material(
            world,
            "Sphere",
            position + offset,
            nalgebra_glm::vec3(scale, scale * 1.5, scale),
            &mat_name,
        );
        entities.push(entity);
    }

    entities
}

pub fn spawn_smoke_column(world: &mut World, position: Vec3) -> Entity {
    spawn_mesh_with_material(
        world,
        "Cylinder",
        position + nalgebra_glm::vec3(0.0, 2.0, 0.0),
        nalgebra_glm::vec3(0.3, 3.0, 0.3),
        "smoke_mat",
    )
}

pub fn spawn_fire_point_light(world: &mut World, position: Vec3) -> Entity {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
        1,
    )[0];
    world.set_light(
        entity,
        Light {
            light_type: LightType::Point,
            color: nalgebra_glm::vec3(1.0, 0.6, 0.2),
            intensity: 200.0,
            range: 8.0,
            ..Default::default()
        },
    );
    world.set_local_transform(
        entity,
        LocalTransform {
            translation: position + nalgebra_glm::vec3(0.0, 1.0, 0.0),
            ..Default::default()
        },
    );
    world.set_global_transform(entity, GlobalTransform::default());
    world.set_local_transform_dirty(entity, LocalTransformDirty);
    entity
}

pub fn spawn_impact_flash(world: &mut World, position: Vec3) -> Entity {
    spawn_mesh_with_material(
        world,
        "Sphere",
        position + nalgebra_glm::vec3(0.0, 0.3, 0.0),
        nalgebra_glm::vec3(0.5, 0.5, 0.5),
        "impact_flash",
    )
}

pub fn spawn_trail_particle(world: &mut World, position: Vec3) -> Entity {
    spawn_mesh_with_material(
        world,
        "Sphere",
        position,
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        "trail_particle",
    )
}

pub fn spawn_rubble_pieces(
    world: &mut World,
    position: Vec3,
    count: usize,
    seed: u64,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    for index in 0..count {
        let offset = nalgebra_glm::vec3(
            ((seed.wrapping_mul(index as u64 + 7)) % 200) as f32 / 100.0 - 1.0,
            0.1 + ((seed.wrapping_mul(index as u64 + 2)) % 30) as f32 / 100.0,
            ((seed.wrapping_mul(index as u64 + 13)) % 200) as f32 / 100.0 - 1.0,
        );

        let scale = 0.15 + ((seed.wrapping_mul(index as u64 + 5)) % 20) as f32 / 100.0;
        let entity = spawn_mesh_with_material(
            world,
            "Cube",
            position + offset,
            nalgebra_glm::vec3(scale, scale * 0.7, scale),
            "rubble_mat",
        );
        entities.push(entity);
    }

    entities
}

pub fn spawn_replan_ring(world: &mut World, position: Vec3) -> Entity {
    spawn_mesh_with_material(
        world,
        "Cylinder",
        position + nalgebra_glm::vec3(0.0, 0.05, 0.0),
        nalgebra_glm::vec3(0.5, 0.02, 0.5),
        "replan_ring_mat",
    )
}

pub fn spawn_carried_item(world: &mut World, position: Vec3, item: CarriedItem) -> Entity {
    let (mesh, material_name, scale) = match item {
        CarriedItem::Water => (
            "Cube",
            "carried_water",
            nalgebra_glm::vec3(0.15, 0.15, 0.15),
        ),
        CarriedItem::RepairMaterials => (
            "Cube",
            "carried_repair",
            nalgebra_glm::vec3(0.18, 0.18, 0.18),
        ),
        CarriedItem::Arrows => (
            "Cylinder",
            "carried_arrows",
            nalgebra_glm::vec3(0.06, 0.25, 0.06),
        ),
    };

    spawn_mesh_with_material(
        world,
        mesh,
        position + nalgebra_glm::vec3(0.35, 0.85, 0.0),
        scale,
        material_name,
    )
}

pub fn spawn_invader(world: &mut World, position: Vec3, index: usize) -> Entity {
    let mat_name = format!("invader_{}", index);
    create_material(world, &mat_name, INVADER_COLOR);
    spawn_mesh_with_material(
        world,
        "Cube",
        position,
        nalgebra_glm::vec3(0.6, 0.8, 0.6),
        &mat_name,
    )
}

pub fn update_wall_segment_color(
    world: &mut World,
    wall_index: usize,
    segment_index: usize,
    health_ratio: f32,
) {
    let mat_name = format!("wall_{}_{}", wall_index, segment_index);
    let healthy = [0.82f32, 0.71, 0.55, 1.0];
    let damaged = [0.45f32, 0.35, 0.25, 1.0];
    let color = [
        healthy[0] * health_ratio + damaged[0] * (1.0 - health_ratio),
        healthy[1] * health_ratio + damaged[1] * (1.0 - health_ratio),
        healthy[2] * health_ratio + damaged[2] * (1.0 - health_ratio),
        1.0,
    ];
    update_material_color(world, &mat_name, color);
}

pub fn update_gate_color(world: &mut World, health_ratio: f32) {
    let healthy = [0.45f32, 0.30, 0.15, 1.0];
    let damaged = [0.2f32, 0.1, 0.05, 1.0];
    let color = [
        healthy[0] * health_ratio + damaged[0] * (1.0 - health_ratio),
        healthy[1] * health_ratio + damaged[1] * (1.0 - health_ratio),
        healthy[2] * health_ratio + damaged[2] * (1.0 - health_ratio),
        1.0,
    ];
    update_material_color(world, "gate", color);
}

pub fn darken_structure(world: &mut World, material_name: &str) {
    update_material_color(world, material_name, CHARRED_COLOR);
}
