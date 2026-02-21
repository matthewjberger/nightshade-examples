use nightshade::ecs::physics::{
    ColliderComponent, ColliderShape, RigidBodyComponent, spawn_first_person_player,
};
use nightshade::ecs::prefab::import_gltf_from_bytes;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::prefab::spawn_prefab_with_skins;
use nightshade::ecs::world::WorldCommand;
use nightshade::prelude::*;

use crate::CityDemo;
use crate::building::BuildingSpec;
use crate::city::CHUNK_SIZE;
use crate::interiors;
use crate::player_systems;

const VIEW_MODEL: &[u8] = include_bytes!("../../../assets/models/view_model.glb");

const GROUND_COLLIDER_THICKNESS: f32 = 1.0;
const COLLIDER_RADIUS: i32 = 5;
const COLLIDER_UNLOAD_RADIUS: i32 = COLLIDER_RADIUS + 2;
const MAX_COLLIDER_CHUNKS_PER_FRAME: usize = 2;

pub fn enter_first_person(demo: &mut CityDemo, world: &mut World) {
    let camera_pos = demo
        .camera_entity
        .and_then(|entity| world.get_local_transform(entity))
        .map(|t| t.translation)
        .unwrap_or(Vec3::new(0.0, 2.0, 0.0));

    let spawn_position = Vec3::new(camera_pos.x, 2.0, camera_pos.z);

    let (player_entity, player_camera) = spawn_first_person_player(world, spawn_position);
    demo.player_entity = Some(player_entity);
    demo.player_camera_entity = Some(player_camera);

    if let Some(transform) = world.get_local_transform_mut(player_camera) {
        transform.translation.y = crate::player_systems::STANDING_CAMERA_HEIGHT;
    }

    if let Some(camera_component) = world.get_camera_mut(player_camera) {
        camera_component.projection = Projection::Perspective(PerspectiveCamera {
            aspect_ratio: None,
            y_fov_rad: 60.0_f32.to_radians(),
            z_far: Some(2000.0),
            z_near: 0.1,
        });
    }

    world.resources.active_camera = Some(player_camera);

    spawn_player_hands(demo, world);

    let flashlight_entity = player_systems::spawn_flashlight(world);
    world.update_parent(flashlight_entity, Some(Parent(Some(player_camera))));
    if let Some(transform) = world.get_local_transform_mut(flashlight_entity) {
        transform.translation = Vec3::new(0.0, 0.0, 0.0);
    }
    world.mark_local_transform_dirty(flashlight_entity);
    demo.flashlight_entity = Some(flashlight_entity);

    spawn_ground_collider(demo, world);

    let player_chunk = (
        (spawn_position.x / CHUNK_SIZE).floor() as i32,
        (spawn_position.z / CHUNK_SIZE).floor() as i32,
    );
    stream_colliders_around(demo, world, player_chunk);

    demo.first_person_mode = true;
}

pub fn exit_first_person(demo: &mut CityDemo, world: &mut World) {
    let restore_position = demo
        .player_camera_entity
        .and_then(|entity| world.get_global_transform(entity))
        .map(|t| t.translation())
        .unwrap_or(Vec3::new(0.0, 30.0, 0.0));

    if let Some(entity) = demo.player_entity.take() {
        world.queue_command(WorldCommand::DespawnRecursive { entity });
    }
    demo.player_camera_entity = None;

    if let Some(entity) = demo.hands_entity.take() {
        world.queue_command(WorldCommand::DespawnRecursive { entity });
    }
    if let Some(entity) = demo.flashlight_entity.take() {
        world.queue_command(WorldCommand::DespawnRecursive { entity });
    }

    if let Some(entity) = demo.ground_collider_entity.take() {
        world.queue_command(WorldCommand::DespawnRecursive { entity });
    }
    for entities in demo.chunk_collision_entities.values() {
        for &entity in entities {
            world.queue_command(WorldCommand::DespawnRecursive { entity });
        }
    }
    demo.chunk_collision_entities.clear();

    if let Some(camera) = demo.camera_entity {
        world.resources.active_camera = Some(camera);
        if let Some(transform) = world.get_local_transform_mut(camera) {
            transform.translation = Vec3::new(
                restore_position.x,
                restore_position.y.max(2.0),
                restore_position.z,
            );
        }
        mark_local_transform_dirty(world, camera);
    }

    demo.first_person_mode = false;
}

fn spawn_player_hands(demo: &mut CityDemo, world: &mut World) {
    let Some(camera_entity) = demo.player_camera_entity else {
        return;
    };

    let load_result = import_gltf_from_bytes(VIEW_MODEL);

    match load_result {
        Ok(result) => {
            for (name, (rgba_data, width, height)) in result.textures {
                world.queue_command(WorldCommand::LoadTexture {
                    name,
                    rgba_data,
                    width,
                    height,
                });
            }

            for (name, mesh) in result.meshes {
                mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
            }

            if let Some(prefab) = result.prefabs.into_iter().next() {
                let hands_entity = spawn_prefab_with_skins(
                    world,
                    &prefab,
                    &result.animations,
                    &result.skins,
                    Vec3::zeros(),
                );

                world.update_parent(hands_entity, Some(Parent(Some(camera_entity))));

                if let Some(transform) = world.get_local_transform_mut(hands_entity) {
                    let view_model_scale = 0.4;
                    transform.translation = Vec3::new(0.0, -0.02, -0.06);
                    transform.rotation = nalgebra_glm::quat_angle_axis(
                        std::f32::consts::PI,
                        &Vec3::new(0.0, 1.0, 0.0),
                    );
                    transform.scale =
                        Vec3::new(view_model_scale, view_model_scale, view_model_scale);
                }
                world.mark_local_transform_dirty(hands_entity);

                if let Some(player) = world.get_animation_player_mut(hands_entity)
                    && player.clips.len() > 9
                {
                    player.blend_to(9, 0.0);
                    player.looping = true;
                }

                demo.hands_entity = Some(hands_entity);
            }
        }
        Err(_error) => {}
    }
}

fn spawn_ground_collider(demo: &mut CityDemo, world: &mut World) {
    let half_extent = demo.city_half as f32 * CHUNK_SIZE + 100.0;
    let entity = spawn_static_cuboid(
        world,
        Vec3::new(0.0, -GROUND_COLLIDER_THICKNESS, 0.0),
        half_extent,
        GROUND_COLLIDER_THICKNESS,
        half_extent,
    );

    if let Some(collider) = world.get_collider_mut(entity) {
        collider.friction = 0.7;
    }

    demo.ground_collider_entity = Some(entity);
}

pub fn update_collider_streaming(demo: &mut CityDemo, world: &mut World) {
    let camera_pos = demo.active_camera_position(world);
    let player_chunk = (
        (camera_pos.x / CHUNK_SIZE).floor() as i32,
        (camera_pos.z / CHUNK_SIZE).floor() as i32,
    );

    let chunks_to_remove: Vec<(i32, i32)> = demo
        .chunk_collision_entities
        .keys()
        .copied()
        .filter(|coords| {
            let dx = (coords.0 - player_chunk.0).abs();
            let dz = (coords.1 - player_chunk.1).abs();
            dx.max(dz) > COLLIDER_UNLOAD_RADIUS
        })
        .collect();

    for coords in chunks_to_remove {
        if let Some(entities) = demo.chunk_collision_entities.remove(&coords) {
            for &entity in &entities {
                world.queue_command(WorldCommand::DespawnRecursive { entity });
            }
        }
    }

    stream_colliders_around(demo, world, player_chunk);
}

fn stream_colliders_around(demo: &mut CityDemo, world: &mut World, center: (i32, i32)) {
    let layout_data: Vec<((i32, i32), Vec<BuildingSpec>)> = demo
        .chunk_streamer
        .as_ref()
        .map(|streamer| {
            let mut result = Vec::new();
            for x in (center.0 - COLLIDER_RADIUS)..=(center.0 + COLLIDER_RADIUS) {
                for z in (center.1 - COLLIDER_RADIUS)..=(center.1 + COLLIDER_RADIUS) {
                    let coords = (x, z);
                    if demo.chunk_collision_entities.contains_key(&coords) {
                        continue;
                    }
                    if let Some(layout) = streamer.layouts().get(&coords) {
                        let dx = coords.0 - center.0;
                        let dz = coords.1 - center.1;
                        let distance_sq = dx * dx + dz * dz;
                        result.push((distance_sq, coords, layout.buildings.clone()));
                    }
                }
            }
            result.sort_by_key(|(distance_sq, _, _)| *distance_sq);
            result
                .into_iter()
                .take(MAX_COLLIDER_CHUNKS_PER_FRAME)
                .map(|(_, coords, buildings)| (coords, buildings))
                .collect()
        })
        .unwrap_or_default();

    for (coords, buildings) in &layout_data {
        generate_chunk_colliders(demo, world, *coords, buildings);
    }
}

pub fn generate_chunk_colliders(
    demo: &mut CityDemo,
    world: &mut World,
    coords: (i32, i32),
    buildings: &[BuildingSpec],
) {
    if demo.chunk_collision_entities.contains_key(&coords) {
        return;
    }

    let mut chunk_colliders = Vec::new();
    for spec in buildings {
        if interiors::building_has_interior(spec) {
            let entities = spawn_interior_building_colliders(world, spec);
            chunk_colliders.extend(entities);
        } else if let Some(entity) = spawn_building_box_collider(world, spec) {
            chunk_colliders.push(entity);
        }
    }
    if !chunk_colliders.is_empty() {
        demo.chunk_collision_entities
            .insert(coords, chunk_colliders);
    }
}

fn spawn_interior_building_colliders(world: &mut World, spec: &BuildingSpec) -> Vec<Entity> {
    let mut entities = Vec::new();

    let half_width = spec.width / 2.0;
    let half_depth = spec.depth / 2.0;
    let wall_height = spec.height;
    let wall_thickness: f32 = 0.3;
    let door_width: f32 = 2.0;
    let door_height: f32 = 2.5;
    let ceiling_thickness: f32 = 0.2;

    let door_face = interiors::door_face_for_building(spec);

    entities.push(spawn_static_cuboid(
        world,
        Vec3::new(spec.x, wall_height - ceiling_thickness / 2.0, spec.z),
        spec.width / 2.0,
        ceiling_thickness / 2.0,
        spec.depth / 2.0,
    ));

    let walls: [(Vec3, f32, bool); 4] = [
        (
            Vec3::new(
                spec.x,
                wall_height / 2.0,
                spec.z + half_depth - wall_thickness / 2.0,
            ),
            spec.width,
            true,
        ),
        (
            Vec3::new(
                spec.x,
                wall_height / 2.0,
                spec.z - half_depth + wall_thickness / 2.0,
            ),
            spec.width,
            true,
        ),
        (
            Vec3::new(
                spec.x + half_width - wall_thickness / 2.0,
                wall_height / 2.0,
                spec.z,
            ),
            spec.depth,
            false,
        ),
        (
            Vec3::new(
                spec.x - half_width + wall_thickness / 2.0,
                wall_height / 2.0,
                spec.z,
            ),
            spec.depth,
            false,
        ),
    ];

    for (face_index, (center, wall_length, is_z_facing)) in walls.iter().enumerate() {
        let has_door = face_index as u32 == door_face;

        if !has_door {
            let (hx, hz) = if *is_z_facing {
                (wall_length / 2.0, wall_thickness / 2.0)
            } else {
                (wall_thickness / 2.0, wall_length / 2.0)
            };
            entities.push(spawn_static_cuboid(
                world,
                *center,
                hx,
                wall_height / 2.0,
                hz,
            ));
        } else {
            let half_length = wall_length / 2.0;
            let section_length = (half_length - door_width / 2.0).max(0.5);
            let above_door_height = (wall_height - door_height).max(0.1);

            if *is_z_facing {
                let left_x = center.x - half_length + section_length / 2.0;
                entities.push(spawn_static_cuboid(
                    world,
                    Vec3::new(left_x, center.y, center.z),
                    section_length / 2.0,
                    wall_height / 2.0,
                    wall_thickness / 2.0,
                ));

                let right_x = center.x + half_length - section_length / 2.0;
                entities.push(spawn_static_cuboid(
                    world,
                    Vec3::new(right_x, center.y, center.z),
                    section_length / 2.0,
                    wall_height / 2.0,
                    wall_thickness / 2.0,
                ));

                if above_door_height > 0.1 {
                    entities.push(spawn_static_cuboid(
                        world,
                        Vec3::new(center.x, door_height + above_door_height / 2.0, center.z),
                        door_width / 2.0,
                        above_door_height / 2.0,
                        wall_thickness / 2.0,
                    ));
                }
            } else {
                let left_z = center.z - half_length + section_length / 2.0;
                entities.push(spawn_static_cuboid(
                    world,
                    Vec3::new(center.x, center.y, left_z),
                    wall_thickness / 2.0,
                    wall_height / 2.0,
                    section_length / 2.0,
                ));

                let right_z = center.z + half_length - section_length / 2.0;
                entities.push(spawn_static_cuboid(
                    world,
                    Vec3::new(center.x, center.y, right_z),
                    wall_thickness / 2.0,
                    wall_height / 2.0,
                    section_length / 2.0,
                ));

                if above_door_height > 0.1 {
                    entities.push(spawn_static_cuboid(
                        world,
                        Vec3::new(center.x, door_height + above_door_height / 2.0, center.z),
                        wall_thickness / 2.0,
                        above_door_height / 2.0,
                        door_width / 2.0,
                    ));
                }
            }
        }
    }

    entities
}

fn spawn_static_cuboid(world: &mut World, position: Vec3, hx: f32, hy: f32, hz: f32) -> Entity {
    let entity = world.spawn_entities(
        LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | RIGID_BODY | COLLIDER,
        1,
    )[0];

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    if let Some(rigid_body) = world.get_rigid_body_mut(entity) {
        *rigid_body =
            RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z);
    }

    if let Some(collider) = world.get_collider_mut(entity) {
        *collider = ColliderComponent {
            shape: ColliderShape::Cuboid { hx, hy, hz },
            friction: 0.5,
            restitution: 0.0,
            ..Default::default()
        };
    }

    entity
}

fn spawn_building_box_collider(world: &mut World, spec: &BuildingSpec) -> Option<Entity> {
    if spec.height < 0.5 {
        return None;
    }

    Some(spawn_static_cuboid(
        world,
        Vec3::new(spec.x, spec.height / 2.0, spec.z),
        spec.width / 2.0,
        spec.height / 2.0,
        spec.depth / 2.0,
    ))
}
