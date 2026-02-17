use nightshade::ecs::text::components::TextProperties;
use nightshade::ecs::world::commands::despawn_entities_with_cache_cleanup;
use nightshade::prelude::*;

use crate::genome::Genome;
use crate::qlearning::{Action, QTable};

pub struct AgentBody {
    pub torso: Entity,
    pub head: Entity,
    pub left_arm: Entity,
    pub right_arm: Entity,
}

impl AgentBody {
    pub fn all_entities(&self) -> [Entity; 4] {
        [self.torso, self.head, self.left_arm, self.right_arm]
    }
}

#[derive(Clone)]
pub struct AgentNeeds {
    pub hunger: f32,
    pub energy: f32,
    pub loneliness: f32,
}

impl AgentNeeds {
    pub fn new() -> Self {
        Self {
            hunger: 0.0,
            energy: 0.0,
            loneliness: 0.0,
        }
    }

    pub fn worst(&self) -> f32 {
        self.hunger.max(self.energy).max(self.loneliness)
    }

    pub fn any_critical(&self) -> bool {
        self.hunger >= 1.0 || self.energy >= 1.0 || self.loneliness >= 1.0
    }
}

const AGENT_NAMES: &[&str] = &[
    "Ada", "Bjorn", "Cleo", "Dag", "Elva", "Finn", "Greta", "Hilde", "Ivar", "Jorun", "Kari",
    "Leif", "Marta", "Nils", "Olga", "Per", "Runa", "Sigrid", "Tor", "Ulla", "Vidar", "Ylva",
];

pub fn agent_name(index: usize) -> &'static str {
    AGENT_NAMES[index % AGENT_NAMES.len()]
}

pub struct Agent {
    pub body: AgentBody,
    pub material_name: String,
    pub name: String,
    pub name_entity: Entity,
    pub needs: AgentNeeds,
    pub genome: Genome,
    pub q_table: QTable,
    pub position: Vec3,
    pub target: Option<Vec3>,
    pub speed: f32,
    pub alive: bool,
    pub survival_time: f32,
    pub current_action: Action,
    pub action_cooldown: f32,
    pub flash_timer: f32,
    pub flash_color: [f32; 3],
    pub death_timer: f32,
    pub wolf_targeted: bool,
    pub was_in_food: bool,
    pub was_in_rest: bool,
    pub home_position: Vec3,
    pub home_entities: Vec<Entity>,
    pub home_level: u8,
    pub build_progress: f32,
    pub campfire_build_progress: f32,
    pub nearby_agent_count: usize,
    pub was_lonely: bool,
    pub was_grouped: bool,
}

impl Agent {
    pub fn new(
        body: AgentBody,
        material_name: String,
        name: String,
        name_entity: Entity,
        genome: Genome,
        q_table: QTable,
        position: Vec3,
    ) -> Self {
        let speed = 2.0 + genome.metabolism * 2.0;
        Self {
            body,
            material_name,
            name,
            name_entity,
            needs: AgentNeeds::new(),
            genome,
            q_table,
            position,
            target: None,
            speed,
            alive: true,
            survival_time: 0.0,
            current_action: Action::Wander,
            action_cooldown: 0.0,
            flash_timer: 0.0,
            flash_color: [0.0; 3],
            death_timer: 1.0,
            wolf_targeted: false,
            was_in_food: false,
            was_in_rest: false,
            home_position: position,
            home_entities: Vec::new(),
            home_level: 0,
            build_progress: 0.0,
            campfire_build_progress: 0.0,
            nearby_agent_count: 0,
            was_lonely: false,
            was_grouped: false,
        }
    }

    pub fn is_at_home(&self) -> bool {
        let dist = nalgebra_glm::distance(&self.position.xz(), &self.home_position.xz());
        dist < 1.5
    }

    pub fn energy_decay_multiplier(&self) -> f32 {
        if !self.is_at_home() {
            return 1.0;
        }
        match self.home_level {
            0 => 0.5,
            1 => 0.4,
            _ => 0.25,
        }
    }

    pub fn hunger_decay_multiplier(&self) -> f32 {
        if self.is_at_home() && self.home_level >= 2 {
            0.8
        } else {
            1.0
        }
    }

    pub fn trigger_flash(&mut self, color: [f32; 3]) {
        self.flash_timer = 0.3;
        self.flash_color = color;
    }

    pub fn despawn_home(&mut self, world: &mut World) {
        if !self.home_entities.is_empty() {
            despawn_entities_with_cache_cleanup(world, &self.home_entities);
            self.home_entities.clear();
        }
    }
}

fn spawn_mesh_with_named_material(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material_name: &str,
) -> Entity {
    let entity = spawn_mesh(world, mesh_name, position, scale);
    world.set_material_ref(entity, MaterialRef::new(material_name.to_string()));
    entity
}

pub fn spawn_home(world: &mut World, position: Vec3, level: u8) -> Vec<Entity> {
    let mut entities = Vec::new();

    match level {
        0 => {
            let plot = spawn_mesh_with_named_material(
                world,
                "Cylinder",
                Vec3::new(position.x, 0.03, position.z),
                Vec3::new(1.5, 0.02, 1.5),
                "home_plot",
            );
            entities.push(plot);
        }
        1 => {
            let wall_positions = [
                Vec3::new(position.x - 0.4, 0.2, position.z),
                Vec3::new(position.x + 0.4, 0.2, position.z),
                Vec3::new(position.x, 0.2, position.z - 0.4),
                Vec3::new(position.x, 0.2, position.z + 0.4),
            ];
            for wall_pos in &wall_positions {
                let wall = spawn_mesh_with_named_material(
                    world,
                    "Cube",
                    *wall_pos,
                    Vec3::new(0.15, 0.4, 0.15),
                    "home_wall",
                );
                entities.push(wall);
            }
            let roof = spawn_mesh_with_named_material(
                world,
                "Cube",
                Vec3::new(position.x, 0.45, position.z),
                Vec3::new(1.0, 0.08, 1.0),
                "home_roof",
            );
            entities.push(roof);
        }
        _ => {
            let wall_positions = [
                Vec3::new(position.x - 0.4, 0.35, position.z),
                Vec3::new(position.x + 0.4, 0.35, position.z),
                Vec3::new(position.x, 0.35, position.z - 0.4),
                Vec3::new(position.x, 0.35, position.z + 0.4),
            ];
            for wall_pos in &wall_positions {
                let wall = spawn_mesh_with_named_material(
                    world,
                    "Cube",
                    *wall_pos,
                    Vec3::new(0.18, 0.7, 0.18),
                    "home_upgraded",
                );
                entities.push(wall);
            }
            let roof = spawn_mesh_with_named_material(
                world,
                "Cube",
                Vec3::new(position.x, 0.75, position.z),
                Vec3::new(1.1, 0.1, 1.1),
                "home_upgraded",
            );
            entities.push(roof);
        }
    }

    entities
}

pub fn upgrade_home(world: &mut World, agent: &mut Agent) {
    agent.despawn_home(world);
    agent.home_level += 1;
    agent.home_entities = spawn_home(world, agent.home_position, agent.home_level);
}

pub fn spawn_agent_body(world: &mut World, position: Vec3, material_name: &str) -> AgentBody {
    let torso = spawn_mesh(world, "Cube", position, Vec3::new(0.4, 0.5, 0.25));
    world.set_material_ref(torso, MaterialRef::new(material_name.to_string()));

    let head_pos = position + Vec3::new(0.0, 0.45, 0.0);
    let head = spawn_mesh(world, "Sphere", head_pos, Vec3::new(0.2, 0.2, 0.2));
    world.set_material_ref(head, MaterialRef::new(material_name.to_string()));

    let left_arm_pos = position + Vec3::new(-0.28, 0.15, 0.0);
    let left_arm = spawn_mesh(world, "Cube", left_arm_pos, Vec3::new(0.08, 0.35, 0.08));
    world.set_material_ref(left_arm, MaterialRef::new(material_name.to_string()));

    let right_arm_pos = position + Vec3::new(0.28, 0.15, 0.0);
    let right_arm = spawn_mesh(world, "Cube", right_arm_pos, Vec3::new(0.08, 0.35, 0.08));
    world.set_material_ref(right_arm, MaterialRef::new(material_name.to_string()));

    AgentBody {
        torso,
        head,
        left_arm,
        right_arm,
    }
}

pub fn spawn_agent_name_label(world: &mut World, name: &str, position: Vec3) -> Entity {
    let text_position = position + Vec3::new(0.0, 0.85, 0.0);
    spawn_3d_billboard_text_with_properties(
        world,
        name,
        text_position,
        TextProperties {
            font_size: 14.0,
            color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 1.0),
            alignment: nightshade::ecs::text::components::TextAlignment::Center,
            outline_width: 0.15,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            smoothing: 0.15,
            ..Default::default()
        },
    )
}

pub fn sync_agent_body_transforms(world: &mut World, agent: &Agent) {
    let position = agent.position;

    if let Some(transform) = world.get_local_transform_mut(agent.body.torso) {
        transform.translation = position;
    }
    world.set_local_transform_dirty(agent.body.torso, LocalTransformDirty);

    if let Some(transform) = world.get_local_transform_mut(agent.body.head) {
        transform.translation = position + Vec3::new(0.0, 0.45, 0.0);
    }
    world.set_local_transform_dirty(agent.body.head, LocalTransformDirty);

    if let Some(transform) = world.get_local_transform_mut(agent.body.left_arm) {
        transform.translation = position + Vec3::new(-0.28, 0.15, 0.0);
    }
    world.set_local_transform_dirty(agent.body.left_arm, LocalTransformDirty);

    if let Some(transform) = world.get_local_transform_mut(agent.body.right_arm) {
        transform.translation = position + Vec3::new(0.28, 0.15, 0.0);
    }
    world.set_local_transform_dirty(agent.body.right_arm, LocalTransformDirty);

    if let Some(transform) = world.get_local_transform_mut(agent.name_entity) {
        transform.translation = position + Vec3::new(0.0, 0.85, 0.0);
    }
    world.set_local_transform_dirty(agent.name_entity, LocalTransformDirty);
}

pub fn update_agent_material(world: &mut World, agent: &Agent) {
    let worst = agent.needs.worst();
    let red_factor = worst;
    let base_r = 1.0;
    let base_g = 1.0 - red_factor * 0.8;
    let base_b = 1.0 - red_factor * 0.8;

    let (emissive, emissive_strength) = if agent.wolf_targeted && agent.alive {
        ([1.0, 0.0, 0.0], 5.0)
    } else if agent.flash_timer > 0.0 {
        (agent.flash_color, 3.0)
    } else {
        ([0.0, 0.0, 0.0], 0.0)
    };

    let (final_r, final_g, final_b) = if !agent.alive {
        (0.4, 0.4, 0.4)
    } else {
        (base_r, base_g, base_b)
    };

    if let Some(material) = nightshade::ecs::generational_registry::registry_entry_by_name_mut(
        &mut world.resources.material_registry.registry,
        &agent.material_name,
    ) {
        material.base_color = [final_r, final_g, final_b, 1.0];
        material.emissive_factor = emissive;
        material.emissive_strength = emissive_strength;
    }
}

pub fn apply_death_animation(world: &mut World, agent: &Agent) {
    let collapse_factor = agent.death_timer.max(0.0);
    if let Some(transform) = world.get_local_transform_mut(agent.body.torso) {
        transform.scale = Vec3::new(0.4, 0.5 * collapse_factor, 0.25);
        transform.translation.y = agent.position.y * collapse_factor;
    }
    world.set_local_transform_dirty(agent.body.torso, LocalTransformDirty);
}

pub fn collect_agent_entities(agents: &[Agent]) -> Vec<Entity> {
    let mut entities = Vec::new();
    for agent in agents {
        entities.extend_from_slice(&agent.body.all_entities());
        entities.push(agent.name_entity);
        entities.extend_from_slice(&agent.home_entities);
    }
    entities
}
