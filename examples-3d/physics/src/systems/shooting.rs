use crate::constants::{BAUBLE_LIFETIME_MS, BAUBLE_SHRINK_DURATION_MS, MAX_SHOT_BAUBLES};
use crate::ecs::{GameWorld, ShotBauble, SHOT_BAUBLE};
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub fn shoot_bauble(game_world: &mut GameWorld, world: &mut World, position: Vec3, direction: Vec3) {
    let bauble_radius = 0.05;
    let bauble_colors = [
        nalgebra_glm::vec3(0.9, 0.2, 0.2),
        nalgebra_glm::vec3(0.2, 0.8, 0.3),
        nalgebra_glm::vec3(0.2, 0.4, 0.9),
        nalgebra_glm::vec3(0.9, 0.8, 0.1),
    ];

    let color_index = (world.resources.window.timing.uptime_milliseconds / 100) as usize
        % bauble_colors.len();
    let color = bauble_colors[color_index];

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY
            | RIGID_BODY
            | COLLIDER
            | COLLISION_LISTENER
            | PHYSICS_INTERPOLATION,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = "Shot Bauble".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = nalgebra_glm::vec3(bauble_radius, bauble_radius, bauble_radius);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = "Sphere".to_string();
    }

    let material_name = format!("ShotBauble_{}", entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        create_textured_material(color, 0.2, 0.8),
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
        *bv = BoundingVolume::from_mesh_type("Sphere");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
        *rigid_body = RigidBodyComponent::new_dynamic()
            .with_translation(position.x, position.y, position.z)
            .with_mass(0.05);
    }

    if let Some(collider) = world.core.get_collider_mut(entity) {
        *collider = ColliderComponent::new_ball(bauble_radius)
            .with_friction(0.5)
            .with_restitution(0.5);
    }

    let rigid_body_comp = world.core.get_rigid_body(entity).cloned().unwrap();
    let collider_comp = world.core.get_collider(entity).cloned();
    let rigid_body = rigid_body_comp.to_rapier_rigid_body();
    let handle = world.resources.physics.add_rigid_body(rigid_body);
    if let Some(collider_comp) = collider_comp {
        let collider = collider_comp.to_rapier_collider();
        world.resources.physics.add_collider(collider, handle);
    }
    if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(entity) {
        rigid_body_mut.handle = Some(handle.into());
    }
    world
        .resources
        .physics
        .handle_to_entity
        .insert(handle, entity);
    world
        .resources
        .physics
        .entity_to_handle
        .insert(entity, handle);

    world.resources.mesh_render_state.mark_entity_added(entity);
    game_world.resources.physics_objects.push(entity);

    if let Some(interpolation) = world.core.get_physics_interpolation_mut(entity) {
        interpolation.previous_translation = position;
        interpolation.previous_rotation = nalgebra_glm::quat_identity();
        interpolation.current_translation = position;
        interpolation.current_rotation = nalgebra_glm::quat_identity();
        interpolation.enabled = true;
    }

    let shoot_speed = 15.0;
    let velocity = direction * shoot_speed;
    if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(handle) {
        rb.set_linvel(
            rapier3d::math::Vector::new(velocity.x, velocity.y, velocity.z),
            true,
        );
    }

    let game_entity = game_world.spawn_entities(SHOT_BAUBLE, 1)[0];
    game_world.set_shot_bauble(
        game_entity,
        ShotBauble {
            entity,
            spawn_time_ms: world.resources.window.timing.uptime_milliseconds,
            original_scale: bauble_radius,
            landed: false,
        },
    );
}

pub fn update_shot_baubles(game_world: &mut GameWorld, world: &mut World) {
    let current_time = world.resources.window.timing.uptime_milliseconds;

    let mut shot_bauble_entities: Vec<freecs::Entity> =
        game_world.query_entities(SHOT_BAUBLE).collect();

    while shot_bauble_entities.len() > MAX_SHOT_BAUBLES {
        let oldest = shot_bauble_entities
            .iter()
            .copied()
            .min_by_key(|&game_entity| {
                game_world
                    .get_shot_bauble(game_entity)
                    .map(|bauble| bauble.spawn_time_ms)
                    .unwrap_or(u64::MAX)
            });

        if let Some(oldest_entity) = oldest {
            if let Some(bauble) = game_world.get_shot_bauble(oldest_entity) {
                let engine_entity = bauble.entity;
                despawn_bauble(game_world, world, engine_entity);
            }
            game_world.despawn_entities(&[oldest_entity]);
            shot_bauble_entities.retain(|&entity| entity != oldest_entity);
        } else {
            break;
        }
    }

    let shot_bauble_entities: Vec<freecs::Entity> =
        game_world.query_entities(SHOT_BAUBLE).collect();

    let mut collided_entities = std::collections::HashSet::new();
    for event in world.resources.physics.collision_events() {
        if event.kind == CollisionEventKind::Started {
            collided_entities.insert(event.entity_a);
            collided_entities.insert(event.entity_b);
        }
    }

    let mut baubles_to_remove = Vec::new();
    let mut baubles_just_landed = Vec::new();

    for &game_entity in &shot_bauble_entities {
        let Some(bauble) = game_world.get_shot_bauble(game_entity) else {
            continue;
        };
        let age_ms = current_time.saturating_sub(bauble.spawn_time_ms);

        if !bauble.landed && collided_entities.contains(&bauble.entity) {
            baubles_just_landed.push((game_entity, bauble.entity));
        }

        if age_ms >= BAUBLE_LIFETIME_MS {
            let shrink_progress_ms = age_ms - BAUBLE_LIFETIME_MS;
            let shrink_factor =
                1.0 - (shrink_progress_ms as f32 / BAUBLE_SHRINK_DURATION_MS as f32);

            if shrink_factor <= 0.0 {
                baubles_to_remove.push((game_entity, bauble.entity));
            } else {
                let new_scale = bauble.original_scale * shrink_factor;
                let entity = bauble.entity;
                if let Some(transform) = world.core.get_local_transform_mut(entity) {
                    transform.scale = nalgebra_glm::vec3(new_scale, new_scale, new_scale);
                }
            }
        }
    }

    for (game_entity, engine_entity) in baubles_just_landed {
        if let Some(bauble) = game_world.get_shot_bauble_mut(game_entity) {
            bauble.landed = true;
        }
        game_world.resources.physics_objects.push(engine_entity);
    }

    for (game_entity, engine_entity) in baubles_to_remove.into_iter().rev() {
        despawn_bauble(game_world, world, engine_entity);
        game_world.despawn_entities(&[game_entity]);
    }
}

fn despawn_bauble(game_world: &mut GameWorld, world: &mut World, entity: Entity) {
    if let Some(rigid_body) = world.core.get_rigid_body(entity)
        && let Some(handle) = rigid_body.handle
    {
        world.resources.physics.remove_rigid_body(handle.into());
    }

    game_world.resources.physics_objects.retain(|e| *e != entity);
    despawn_entities_with_cache_cleanup(world, &[entity]);
}
