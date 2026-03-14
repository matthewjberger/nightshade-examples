use nightshade::ecs::material::material_registry_insert;
use nightshade::prelude::*;
use rand::Rng;

pub struct Zone {
    pub center: Vec3,
    pub radius: f32,
}

impl Zone {
    pub fn contains_xz(&self, position: &Vec3) -> bool {
        let dx = position.x - self.center.x;
        let dz = position.z - self.center.z;
        (dx * dx + dz * dz).sqrt() <= self.radius
    }
}

pub struct Zones {
    pub food_a: Zone,
    pub food_b: Zone,
    pub rest: Zone,
}

impl Zones {
    pub fn new() -> Self {
        Self {
            food_a: Zone {
                center: Vec3::new(-10.0, 0.0, -8.0),
                radius: 3.0,
            },
            food_b: Zone {
                center: Vec3::new(8.0, 0.0, 6.0),
                radius: 3.0,
            },
            rest: Zone {
                center: Vec3::new(-5.0, 0.0, 6.0),
                radius: 3.0,
            },
        }
    }

    pub fn in_any_food(&self, position: &Vec3) -> bool {
        self.food_a.contains_xz(position) || self.food_b.contains_xz(position)
    }

    pub fn in_rest(&self, position: &Vec3) -> bool {
        self.rest.contains_xz(position)
    }

    pub fn nearest_food_center(&self, position: &Vec3) -> Vec3 {
        let centers = [self.food_a.center, self.food_b.center];
        *centers
            .iter()
            .min_by(|a, b| {
                let dist_a = nalgebra_glm::distance(&a.xz(), &position.xz());
                let dist_b = nalgebra_glm::distance(&b.xz(), &position.xz());
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .unwrap()
    }
}

pub struct Wolf {
    pub body: Entity,
    pub hunt_radius_entity: Entity,
    pub position: Vec3,
    pub waypoints: Vec<Vec3>,
    pub current_waypoint: usize,
    pub speed: f32,
    pub hunt_target: Option<usize>,
}

impl Wolf {
    pub fn hunt_radius(&self, is_night: bool) -> f32 {
        if is_night { 12.0 } else { 8.0 }
    }
}

pub struct DayNight {
    pub time_of_day: f32,
    pub cycle_length: f32,
}

impl DayNight {
    pub fn new() -> Self {
        Self {
            time_of_day: 8.0 / 24.0,
            cycle_length: 60.0,
        }
    }

    pub fn is_night(&self) -> bool {
        self.time_of_day > 0.5
    }

    pub fn to_hour(&self) -> f32 {
        self.time_of_day * 24.0
    }

    pub fn advance(&mut self, tick_interval: f32) {
        self.time_of_day += tick_interval / self.cycle_length;
        if self.time_of_day >= 1.0 {
            self.time_of_day -= 1.0;
        }
    }
}

pub struct Campfire {
    pub position: Vec3,
    pub entities: Vec<Entity>,
    pub fuel: f32,
}

pub struct Environment {
    pub zones: Zones,
    pub day_night: DayNight,
    pub wolf: Wolf,
    pub campfires: Vec<Campfire>,
    _ground_entity: Entity,
    _decoration_entities: Vec<Entity>,
}

fn register_material(world: &mut World, name: &str, material: Material) {
    material_registry_insert(
        &mut world.resources.material_registry,
        name.to_string(),
        material,
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

fn spawn_mesh_with_named_material(
    world: &mut World,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material_name: &str,
) -> Entity {
    let entity = spawn_mesh(world, mesh_name, position, scale);
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name.to_string()));
    entity
}

fn spawn_zone_disc(
    world: &mut World,
    center: Vec3,
    radius: f32,
    y_height: f32,
    material_name: &str,
) -> Entity {
    let diameter = radius * 2.0;
    spawn_mesh_with_named_material(
        world,
        "Cylinder",
        Vec3::new(center.x, y_height, center.z),
        Vec3::new(diameter, 0.02, diameter),
        material_name,
    )
}

impl Environment {
    pub fn initialize(world: &mut World, rng: &mut impl Rng) -> Self {
        register_material(
            world,
            "ground",
            Material {
                base_color: [0.25, 0.35, 0.15, 1.0],
                roughness: 0.9,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "food_tree",
            Material {
                base_color: [0.8, 0.15, 0.1, 1.0],
                emissive_factor: [0.3, 0.05, 0.02],
                emissive_strength: 1.0,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "premium_food",
            Material {
                base_color: [1.0, 0.2, 0.1, 1.0],
                emissive_factor: [0.5, 0.1, 0.05],
                emissive_strength: 2.0,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "rest_mat",
            Material {
                base_color: [0.2, 0.3, 0.7, 1.0],
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "social_lantern",
            Material {
                base_color: [1.0, 0.9, 0.3, 1.0],
                emissive_factor: [1.0, 0.8, 0.2],
                emissive_strength: 3.0,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "social_post",
            Material {
                base_color: [0.4, 0.25, 0.1, 1.0],
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "home_wall",
            Material {
                base_color: [0.4, 0.25, 0.1, 1.0],
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "home_roof",
            Material {
                base_color: [0.3, 0.15, 0.05, 1.0],
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "home_upgraded",
            Material {
                base_color: [0.45, 0.3, 0.15, 1.0],
                emissive_factor: [0.4, 0.25, 0.1],
                emissive_strength: 1.5,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "home_plot",
            Material {
                base_color: [0.3, 0.2, 0.1, 1.0],
                emissive_factor: [0.1, 0.05, 0.02],
                emissive_strength: 0.3,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "campfire_base",
            Material {
                base_color: [0.35, 0.2, 0.1, 1.0],
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "campfire_flame",
            Material {
                base_color: [1.0, 0.6, 0.1, 1.0],
                emissive_factor: [1.0, 0.5, 0.1],
                emissive_strength: 3.0,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "wolf_material",
            Material {
                base_color: [0.15, 0.1, 0.1, 1.0],
                emissive_factor: [0.8, 0.1, 0.0],
                emissive_strength: 2.0,
                roughness: 0.3,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "wolf_radius",
            Material {
                base_color: [0.3, 0.04, 0.02, 1.0],
                emissive_factor: [0.2, 0.02, 0.0],
                emissive_strength: 0.5,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "boulder",
            Material {
                base_color: [0.5, 0.48, 0.45, 1.0],
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "tree_trunk",
            Material {
                base_color: [0.35, 0.2, 0.1, 1.0],
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "tree_canopy",
            Material {
                base_color: [0.1, 0.35, 0.1, 1.0],
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "zone_food",
            Material {
                base_color: [0.35, 0.15, 0.05, 1.0],
                emissive_factor: [0.5, 0.2, 0.05],
                emissive_strength: 0.8,
                unlit: false,
                ..Default::default()
            },
        );

        register_material(
            world,
            "zone_rest",
            Material {
                base_color: [0.05, 0.1, 0.3, 1.0],
                emissive_factor: [0.1, 0.15, 0.5],
                emissive_strength: 0.8,
                unlit: false,
                ..Default::default()
            },
        );

        let ground_entity = spawn_mesh_with_named_material(
            world,
            "Cube",
            Vec3::new(0.0, -0.25, 0.0),
            Vec3::new(40.0, 0.5, 40.0),
            "ground",
        );

        let mut decoration_entities = Vec::new();

        let zones = Zones::new();

        decoration_entities.push(spawn_zone_disc(
            world,
            zones.food_a.center,
            zones.food_a.radius,
            0.03,
            "zone_food",
        ));
        decoration_entities.push(spawn_zone_disc(
            world,
            zones.food_b.center,
            zones.food_b.radius,
            0.03,
            "zone_food",
        ));
        decoration_entities.push(spawn_zone_disc(
            world,
            zones.rest.center,
            zones.rest.radius,
            0.03,
            "zone_rest",
        ));

        for _ in 0..3 {
            let offset_x = rng.random_range(-2.0..2.0f32);
            let offset_z = rng.random_range(-2.0..2.0f32);
            let entity = spawn_mesh_with_named_material(
                world,
                "Cylinder",
                Vec3::new(-10.0 + offset_x, 0.75, -8.0 + offset_z),
                Vec3::new(0.3, 1.5, 0.3),
                "food_tree",
            );
            decoration_entities.push(entity);
        }

        for _ in 0..2 {
            let offset_x = rng.random_range(-2.0..2.0f32);
            let offset_z = rng.random_range(-2.0..2.0f32);
            let entity = spawn_mesh_with_named_material(
                world,
                "Cylinder",
                Vec3::new(8.0 + offset_x, 0.75, 6.0 + offset_z),
                Vec3::new(0.3, 1.5, 0.3),
                "food_tree",
            );
            decoration_entities.push(entity);
        }

        for offset_index in 0..3 {
            let offset_x = offset_index as f32 * 1.5 - 1.5;
            let entity = spawn_mesh_with_named_material(
                world,
                "Cube",
                Vec3::new(-5.0 + offset_x, 0.075, 6.0),
                Vec3::new(1.2, 0.15, 0.8),
                "rest_mat",
            );
            decoration_entities.push(entity);
        }

        let lantern_post = spawn_mesh_with_named_material(
            world,
            "Cylinder",
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(0.1, 1.0, 0.1),
            "social_post",
        );
        decoration_entities.push(lantern_post);

        let lantern_globe = spawn_mesh_with_named_material(
            world,
            "Sphere",
            Vec3::new(0.0, 1.2, 0.0),
            Vec3::new(0.3, 0.3, 0.3),
            "social_lantern",
        );
        decoration_entities.push(lantern_globe);

        for _ in 0..8 {
            let boulder_x = rng.random_range(-18.0..18.0f32);
            let boulder_z = rng.random_range(-18.0..18.0f32);
            let boulder_scale = rng.random_range(0.3..0.7f32);
            let entity = spawn_mesh_with_named_material(
                world,
                "Sphere",
                Vec3::new(boulder_x, boulder_scale * 0.5, boulder_z),
                Vec3::new(boulder_scale, boulder_scale, boulder_scale),
                "boulder",
            );
            decoration_entities.push(entity);
        }

        for _ in 0..6 {
            let tree_x = rng.random_range(-17.0..17.0f32);
            let tree_z = rng.random_range(-17.0..17.0f32);
            let trunk = spawn_mesh_with_named_material(
                world,
                "Cylinder",
                Vec3::new(tree_x, 0.3, tree_z),
                Vec3::new(0.15, 0.6, 0.15),
                "tree_trunk",
            );
            decoration_entities.push(trunk);
            let canopy = spawn_mesh_with_named_material(
                world,
                "Sphere",
                Vec3::new(tree_x, 0.85, tree_z),
                Vec3::new(0.5, 0.5, 0.5),
                "tree_canopy",
            );
            decoration_entities.push(canopy);
        }

        let wolf_waypoints = vec![
            Vec3::new(-10.0, 0.0, -8.0),
            Vec3::new(-3.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 4.0),
            Vec3::new(10.0, 0.0, 8.0),
            Vec3::new(5.0, 0.0, 4.0),
            Vec3::new(-3.0, 0.0, 0.0),
        ];

        let wolf_pos = Vec3::new(wolf_waypoints[0].x, 0.15, wolf_waypoints[0].z);
        let wolf_body = spawn_mesh_with_named_material(
            world,
            "Cube",
            wolf_pos,
            Vec3::new(0.6, 0.3, 0.9),
            "wolf_material",
        );

        let initial_hunt_radius = 8.0;
        let hunt_radius_entity = spawn_mesh_with_named_material(
            world,
            "Cylinder",
            Vec3::new(wolf_pos.x, 0.005, wolf_pos.z),
            Vec3::new(initial_hunt_radius * 2.0, 0.01, initial_hunt_radius * 2.0),
            "wolf_radius",
        );

        let wolf = Wolf {
            body: wolf_body,
            hunt_radius_entity,
            position: wolf_pos,
            waypoints: wolf_waypoints,
            current_waypoint: 0,
            speed: 3.0,
            hunt_target: None,
        };

        Environment {
            zones,
            day_night: DayNight::new(),
            wolf,
            campfires: Vec::new(),
            _ground_entity: ground_entity,
            _decoration_entities: decoration_entities,
        }
    }

    pub fn wolf_tick(&mut self, agents: &mut [crate::agent::Agent], tick_interval: f32) {
        let is_night = self.day_night.is_night();
        let hunt_radius = self.wolf.hunt_radius(is_night);

        let near_campfire = self.campfires.iter().any(|campfire| {
            let dist = nalgebra_glm::distance(&self.wolf.position.xz(), &campfire.position.xz());
            dist < 3.0
        });

        if near_campfire {
            self.wolf.hunt_target = None;
            let waypoint = self.wolf.waypoints[self.wolf.current_waypoint];
            let direction = Vec3::new(waypoint.x, 0.0, waypoint.z)
                - Vec3::new(self.wolf.position.x, 0.0, self.wolf.position.z);
            let distance = nalgebra_glm::length(&direction.xz());
            if distance > 0.1 {
                let normalized = direction.normalize();
                let move_speed = self.wolf.speed * 2.0 * tick_interval;
                self.wolf.position += normalized * move_speed;
                self.wolf.position.y = 0.15;
            }
            return;
        }

        if let Some(target_index) = self.wolf.hunt_target {
            if target_index < agents.len() && agents[target_index].alive {
                let target_pos = agents[target_index].position;
                let distance = nalgebra_glm::distance(&self.wolf.position.xz(), &target_pos.xz());

                if distance < 1.0 {
                    agents[target_index].needs.hunger = 1.0;
                    agents[target_index].needs.energy = 1.0;
                    agents[target_index].needs.loneliness = 1.0;
                    self.wolf.hunt_target = None;
                } else if distance > 14.0 {
                    self.wolf.hunt_target = None;
                } else {
                    let direction = (target_pos - self.wolf.position).normalize();
                    let move_speed = self.wolf.speed * 1.8 * tick_interval;
                    self.wolf.position += direction * move_speed;
                    self.wolf.position.y = 0.15;
                }
            } else {
                self.wolf.hunt_target = None;
            }
        }

        if self.wolf.hunt_target.is_none() {
            let mut closest_index = None;
            let mut closest_distance = hunt_radius;

            for (index, agent) in agents.iter().enumerate() {
                if !agent.alive {
                    continue;
                }
                let distance =
                    nalgebra_glm::distance(&self.wolf.position.xz(), &agent.position.xz());
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_index = Some(index);
                }
            }

            if let Some(index) = closest_index {
                self.wolf.hunt_target = Some(index);
            } else {
                let waypoint = self.wolf.waypoints[self.wolf.current_waypoint];
                let direction = Vec3::new(waypoint.x, 0.0, waypoint.z)
                    - Vec3::new(self.wolf.position.x, 0.0, self.wolf.position.z);
                let distance = nalgebra_glm::length(&direction.xz());
                if distance < 2.0 {
                    self.wolf.current_waypoint =
                        (self.wolf.current_waypoint + 1) % self.wolf.waypoints.len();
                } else {
                    let normalized = direction.normalize();
                    let move_speed = self.wolf.speed * tick_interval;
                    self.wolf.position += normalized * move_speed;
                    self.wolf.position.y = 0.15;
                }
            }
        }
    }

    pub fn sync_wolf_transform(&self, world: &mut World) {
        if let Some(transform) = world.core.get_local_transform_mut(self.wolf.body) {
            transform.translation = self.wolf.position;
        }
        world
            .core
            .set_local_transform_dirty(self.wolf.body, LocalTransformDirty);

        let is_night = self.day_night.is_night();
        let hunt_radius = self.wolf.hunt_radius(is_night);
        let diameter = hunt_radius * 2.0;
        if let Some(transform) = world
            .core
            .get_local_transform_mut(self.wolf.hunt_radius_entity)
        {
            transform.translation = Vec3::new(self.wolf.position.x, 0.005, self.wolf.position.z);
            transform.scale = Vec3::new(diameter, 0.01, diameter);
        }
        world
            .core
            .set_local_transform_dirty(self.wolf.hunt_radius_entity, LocalTransformDirty);

        let hunting = self.wolf.hunt_target.is_some();
        if let Some(material) = nightshade::ecs::generational_registry::registry_entry_by_name_mut(
            &mut world.resources.material_registry.registry,
            "wolf_material",
        ) {
            if hunting {
                material.emissive_factor = [1.0, 0.15, 0.0];
                material.emissive_strength = 4.0;
            } else {
                material.emissive_factor = [0.4, 0.05, 0.0];
                material.emissive_strength = 1.5;
            }
        }
    }

    pub fn reset_wolf(&mut self) {
        self.wolf.position = Vec3::new(self.wolf.waypoints[0].x, 0.15, self.wolf.waypoints[0].z);
        self.wolf.current_waypoint = 0;
        self.wolf.hunt_target = None;
    }

    pub fn spawn_campfire(&mut self, world: &mut World, position: Vec3) {
        if self.campfires.len() >= 6 {
            return;
        }

        let mut entities = Vec::new();

        let base = spawn_mesh_with_named_material(
            world,
            "Cylinder",
            Vec3::new(position.x, 0.1, position.z),
            Vec3::new(0.4, 0.2, 0.4),
            "campfire_base",
        );
        entities.push(base);

        let flame = spawn_mesh_with_named_material(
            world,
            "Sphere",
            Vec3::new(position.x, 0.35, position.z),
            Vec3::new(0.25, 0.3, 0.25),
            "campfire_flame",
        );
        entities.push(flame);

        self.campfires.push(Campfire {
            position,
            entities,
            fuel: 1.0,
        });
    }

    pub fn despawn_campfires(&mut self, world: &mut World) {
        for campfire in self.campfires.drain(..) {
            nightshade::ecs::world::commands::despawn_entities_with_cache_cleanup(
                world,
                &campfire.entities,
            );
        }
    }

    pub fn tick_campfires(&mut self, world: &mut World, tick_interval: f32) {
        let mut indices_to_remove = Vec::new();

        for (index, campfire) in self.campfires.iter_mut().enumerate() {
            campfire.fuel -= 0.01 * tick_interval * 10.0;
            if campfire.fuel <= 0.0 {
                indices_to_remove.push(index);
            }
        }

        for index in indices_to_remove.into_iter().rev() {
            let campfire = self.campfires.swap_remove(index);
            nightshade::ecs::world::commands::despawn_entities_with_cache_cleanup(
                world,
                &campfire.entities,
            );
        }
    }

    pub fn agent_near_campfire(&self, position: &Vec3) -> bool {
        self.campfires.iter().any(|campfire| {
            let dist = nalgebra_glm::distance(&position.xz(), &campfire.position.xz());
            dist < 3.0
        })
    }
}
