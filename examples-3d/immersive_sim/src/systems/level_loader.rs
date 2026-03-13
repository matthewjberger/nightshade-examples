use crate::data::enemies::{Enemy, get_enemy_definition};
use crate::data::items::{ItemType, WorldItem, get_item_definition};
use crate::data::levels::{LevelDefinition, LevelId, Portal, get_level};
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

#[derive(Default)]
pub struct LoadedLevel {
    pub geometry_entities: Vec<Entity>,
    pub light_entities: Vec<Entity>,
    pub portal_entities: Vec<(Entity, Portal)>,
    pub item_entities: Vec<WorldItem>,
    pub enemies: Vec<Enemy>,
}

fn spawn_mesh(world: &mut World, mesh_name: &str, position: Vec3, scale: Vec3) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = format!("Mesh_{}", entity.id);
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    entity
}

fn spawn_point_light(
    world: &mut World,
    position: Vec3,
    color: [f32; 3],
    intensity: f32,
    range: f32,
) -> Entity {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];

    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Point,
            color: nalgebra_glm::vec3(color[0], color[1], color[2]),
            intensity,
            range,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: false,
            shadow_bias: 0.001,
        },
    );

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    world.core.set_global_transform(entity, GlobalTransform::default());
    world.core.set_local_transform_dirty(entity, LocalTransformDirty);

    entity
}

struct SpotLightParams {
    position: Vec3,
    color: [f32; 3],
    intensity: f32,
    range: f32,
    direction: Vec3,
    inner_angle: f32,
    outer_angle: f32,
}

fn spawn_spot_light(world: &mut World, params: SpotLightParams) -> Entity {
    let entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];

    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Spot,
            color: nalgebra_glm::vec3(params.color[0], params.color[1], params.color[2]),
            intensity: params.intensity,
            range: params.range,
            inner_cone_angle: params.inner_angle,
            outer_cone_angle: params.outer_angle,
            cast_shadows: false,
            shadow_bias: 0.001,
        },
    );

    let rotation = nalgebra_glm::quat_look_at(&params.direction, &Vec3::y());

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: params.position,
            rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    world.core.set_global_transform(entity, GlobalTransform::default());
    world.core.set_local_transform_dirty(entity, LocalTransformDirty);

    entity
}

fn mark_local_transform_dirty(world: &mut World, entity: Entity) {
    world.core.set_local_transform_dirty(entity, LocalTransformDirty);
}

pub fn load_level(world: &mut World, level_id: LevelId) -> LoadedLevel {
    let level_def = get_level(level_id);

    if let Some(fog_settings) = create_fog_settings(&level_def) {
        world.resources.graphics.fog = Some(fog_settings);
    }

    let geometry_entities = spawn_level_geometry(world, &level_def);
    let light_entities = spawn_level_lights(world, &level_def);
    let portal_entities = spawn_portals(world, &level_def);
    let item_entities = spawn_items(world, &level_def);
    let enemies = spawn_enemies(world, &level_def);

    LoadedLevel {
        geometry_entities,
        light_entities,
        portal_entities,
        item_entities,
        enemies,
    }
}

pub fn unload_level(world: &mut World, loaded_level: &mut LoadedLevel) {
    for entity in loaded_level.geometry_entities.drain(..) {
        world.despawn_entities(&[entity]);
    }
    for entity in loaded_level.light_entities.drain(..) {
        world.despawn_entities(&[entity]);
    }
    for (entity, _) in loaded_level.portal_entities.drain(..) {
        world.despawn_entities(&[entity]);
    }
    for item in loaded_level.item_entities.drain(..) {
        world.despawn_entities(&[item.entity]);
    }
    for enemy in loaded_level.enemies.drain(..) {
        world.despawn_entities(&[enemy.entity]);
    }
}

fn create_fog_settings(level_def: &LevelDefinition) -> Option<Fog> {
    Some(Fog {
        start: level_def.fog_start,
        end: level_def.fog_end,
        color: level_def.fog_color,
    })
}

fn spawn_level_geometry(world: &mut World, level_def: &LevelDefinition) -> Vec<Entity> {
    let mut entities = Vec::new();

    for (index, geo) in level_def.geometry.iter().enumerate() {
        let entity = spawn_mesh(world, geo.mesh, geo.position, geo.scale);

        world.core.set_casts_shadow(entity, CastsShadow);

        let mat_name = format!("LevelGeo_{}_{}", level_def.id as u32, index);
        let emissive = if geo.emissive > 0.0 {
            [
                geo.color[0] * geo.emissive,
                geo.color[1] * geo.emissive,
                geo.color[2] * geo.emissive,
            ]
        } else {
            [0.0, 0.0, 0.0]
        };

        material_registry_insert(
            &mut world.resources.material_registry,
            mat_name.clone(),
            Material {
                base_color: geo.color,
                roughness: geo.roughness,
                metallic: geo.metallic,
                emissive_factor: emissive,
                ..Default::default()
            },
        );
        if let Some(&mat_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&mat_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(mat_index);
        }
        world.core.set_material_ref(entity, MaterialRef::new(mat_name));

        if geo.rotation != 0.0 {
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.rotation = nalgebra_glm::quat_angle_axis(geo.rotation, &Vec3::y());
            }
            mark_local_transform_dirty(world, entity);
        }

        entities.push(entity);
    }

    entities
}

fn spawn_level_lights(world: &mut World, level_def: &LevelDefinition) -> Vec<Entity> {
    let mut entities = Vec::new();

    for light_def in &level_def.lights {
        let light_entity = if light_def.is_spotlight {
            let direction = light_def.direction.unwrap_or(Vec3::new(0.0, -1.0, 0.0));
            spawn_spot_light(
                world,
                SpotLightParams {
                    position: light_def.position,
                    color: light_def.color,
                    intensity: light_def.intensity,
                    range: light_def.range,
                    direction,
                    inner_angle: 0.8,
                    outer_angle: 0.9,
                },
            )
        } else {
            spawn_point_light(
                world,
                light_def.position,
                light_def.color,
                light_def.intensity,
                light_def.range,
            )
        };

        entities.push(light_entity);
    }

    entities
}

fn spawn_portals(world: &mut World, level_def: &LevelDefinition) -> Vec<(Entity, Portal)> {
    let mut portal_entities = Vec::new();

    for portal in &level_def.portals {
        let entity = spawn_mesh(world, "Cylinder", portal.position, Vec3::new(1.5, 3.0, 1.5));

        let mat_name = format!("Portal_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            mat_name.clone(),
            Material {
                base_color: portal.color,
                roughness: 0.1,
                metallic: 0.8,
                emissive_factor: [
                    portal.color[0] * 2.0,
                    portal.color[1] * 2.0,
                    portal.color[2] * 2.0,
                ],
                ..Default::default()
            },
        );
        if let Some(&mat_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&mat_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(mat_index);
        }
        world.core.set_material_ref(entity, MaterialRef::new(mat_name));

        let _light_entity = spawn_point_light(
            world,
            portal.position + Vec3::new(0.0, 2.0, 0.0),
            [portal.color[0], portal.color[1], portal.color[2]],
            3.0,
            5.0,
        );

        portal_entities.push((entity, portal.clone()));
    }

    portal_entities
}

fn spawn_items(world: &mut World, level_def: &LevelDefinition) -> Vec<WorldItem> {
    let mut items = Vec::new();

    for item_spawn in &level_def.item_spawns {
        if let Some(def) = get_item_definition(item_spawn.item_type) {
            let entity = spawn_mesh(
                world,
                def.mesh,
                item_spawn.position,
                Vec3::new(def.scale, def.scale, def.scale),
            );

            let mat_name = format!("Item_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                mat_name.clone(),
                Material {
                    base_color: def.color,
                    roughness: 0.3,
                    metallic: 0.5,
                    emissive_factor: [def.color[0] * 0.5, def.color[1] * 0.5, def.color[2] * 0.5],
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&mat_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(mat_index);
            }
            world.core.set_material_ref(entity, MaterialRef::new(mat_name));

            items.push(WorldItem {
                entity,
                item_type: item_spawn.item_type,
                quantity: item_spawn.quantity,
                spawn_time: 0.0,
            });
        }
    }

    items
}

fn spawn_enemies(world: &mut World, level_def: &LevelDefinition) -> Vec<Enemy> {
    let mut enemies = Vec::new();

    for enemy_spawn in &level_def.enemy_spawns {
        if let Some(def) = get_enemy_definition(enemy_spawn.enemy_type) {
            let entity = spawn_mesh(
                world,
                "Cylinder",
                enemy_spawn.position,
                Vec3::new(def.scale * 0.5, def.scale * 2.0, def.scale * 0.5),
            );

            world.core.set_casts_shadow(entity, CastsShadow);

            let mat_name = format!("Enemy_{}", entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                mat_name.clone(),
                Material {
                    base_color: def.color,
                    roughness: 0.6,
                    metallic: 0.2,
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&mat_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(mat_index);
            }
            world.core.set_material_ref(entity, MaterialRef::new(mat_name));

            enemies.push(Enemy::new(
                entity,
                enemy_spawn.enemy_type,
                enemy_spawn.position,
            ));
        }
    }

    enemies
}

pub fn check_portal_collision(
    loaded_level: &LoadedLevel,
    world: &World,
    player_pos: Vec3,
    has_key: bool,
) -> Option<LevelId> {
    for (entity, portal) in &loaded_level.portal_entities {
        let portal_pos = world
            .core.get_local_transform(*entity)
            .map(|t| t.translation)
            .unwrap_or(portal.position);

        let distance = nalgebra_glm::length(&(player_pos - portal_pos));

        if distance < 2.0 {
            if portal.requires_key && !has_key {
                continue;
            }
            return Some(portal.target_level);
        }
    }

    None
}

pub fn check_item_pickup(
    loaded_level: &mut LoadedLevel,
    world: &mut World,
    player_pos: Vec3,
) -> Option<(ItemType, usize)> {
    let mut picked_up = None;
    let mut index_to_remove = None;

    for (index, item) in loaded_level.item_entities.iter().enumerate() {
        let item_pos = world
            .core.get_local_transform(item.entity)
            .map(|t| t.translation)
            .unwrap_or(Vec3::zeros());

        let distance = nalgebra_glm::length(&(player_pos - item_pos));

        if distance < 1.5 {
            picked_up = Some((item.item_type, item.quantity));
            index_to_remove = Some(index);
            break;
        }
    }

    if let Some(index) = index_to_remove {
        let item = loaded_level.item_entities.remove(index);
        world.despawn_entities(&[item.entity]);
    }

    picked_up
}

pub fn update_item_bobbing(loaded_level: &mut LoadedLevel, world: &mut World, time: f32) {
    for item in &loaded_level.item_entities {
        if let Some(transform) = world.core.get_local_transform_mut(item.entity) {
            let bob_offset = (time * 2.0 + item.spawn_time).sin() * 0.1;
            transform.translation.y += bob_offset * 0.016;
            transform.rotation = nalgebra_glm::quat_angle_axis(time * 1.5, &Vec3::y());
        }
        mark_local_transform_dirty(world, item.entity);
    }
}

pub fn spawn_loot_item(
    loaded_level: &mut LoadedLevel,
    world: &mut World,
    position: Vec3,
    item_type: ItemType,
    quantity: usize,
) {
    if let Some(def) = get_item_definition(item_type) {
        let entity = spawn_mesh(
            world,
            def.mesh,
            position + Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(def.scale, def.scale, def.scale),
        );

        let mat_name = format!("LootItem_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            mat_name.clone(),
            Material {
                base_color: def.color,
                roughness: 0.3,
                metallic: 0.5,
                emissive_factor: [def.color[0] * 0.5, def.color[1] * 0.5, def.color[2] * 0.5],
                ..Default::default()
            },
        );
        if let Some(&mat_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&mat_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(mat_index);
        }
        world.core.set_material_ref(entity, MaterialRef::new(mat_name));

        loaded_level.item_entities.push(WorldItem {
            entity,
            item_type,
            quantity,
            spawn_time: 0.0,
        });
    }
}
