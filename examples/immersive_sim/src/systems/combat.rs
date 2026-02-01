use crate::data::enemies::{Enemy, EnemyState, get_enemy_definition};
use crate::data::items::ItemType;
use crate::data::player::PlayerProgress;
use crate::data::skills::{EffectType, Projectile, SkillType, get_skill_definition};
use crate::systems::particles::{
    ParticleSystem, spawn_damage_effect, spawn_explosion_effect, spawn_fireball_effect,
    spawn_heal_effect, spawn_ice_effect, spawn_lightning_effect, spawn_magic_effect,
};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

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

    if let Some(name) = world.get_name_mut(entity) {
        name.0 = format!("Projectile_{}", entity.id);
    }

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    if let Some(bounding_volume) = world.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    entity
}

fn mark_local_transform_dirty(world: &mut World, entity: Entity) {
    world.set_local_transform_dirty(entity, LocalTransformDirty);
}

#[derive(Default)]
pub struct CombatState {
    pub projectiles: Vec<Projectile>,
}

pub fn use_skill(
    skill_type: SkillType,
    player_progress: &mut PlayerProgress,
    combat_state: &mut CombatState,
    particle_system: &mut ParticleSystem,
    world: &mut World,
    camera_entity: Entity,
) -> bool {
    let def = match get_skill_definition(skill_type) {
        Some(d) => d,
        None => return false,
    };

    let skill_level = match player_progress.skills.skills.get(&skill_type) {
        Some(s) if s.unlocked && s.cooldown_remaining <= 0.0 => s.level,
        _ => return false,
    };

    if !player_progress.stats.use_mana(def.mana_cost) {
        return false;
    }

    if let Some(state) = player_progress.skills.skills.get_mut(&skill_type) {
        state.cooldown_remaining = def.cooldown;
    }

    let camera_pos = world
        .get_global_transform(camera_entity)
        .map(|t| t.translation())
        .unwrap_or(Vec3::zeros());

    let camera_forward = world
        .get_global_transform(camera_entity)
        .map(|t| t.forward_vector())
        .unwrap_or(Vec3::new(0.0, 0.0, -1.0));

    let level_multiplier = 1.0 + (skill_level - 1) as f32 * 0.15;

    match skill_type {
        SkillType::Fireball => {
            let spawn_pos = camera_pos + camera_forward * 1.0;
            spawn_projectile(
                combat_state,
                world,
                ProjectileParams {
                    position: spawn_pos,
                    velocity: camera_forward * 20.0,
                    skill_type,
                    damage: def.damage * level_multiplier,
                    aoe_radius: def.area_of_effect,
                    lifetime: 4.0,
                    owner_is_player: true,
                },
            );
            spawn_fireball_effect(particle_system, spawn_pos);
        }
        SkillType::IceBlast => {
            let spawn_pos = camera_pos + camera_forward * 0.8;
            for index in 0..5 {
                let angle = (index as f32 - 2.0) * 0.15;
                let rotated = rotate_vector_y(camera_forward, angle);
                spawn_projectile(
                    combat_state,
                    world,
                    ProjectileParams {
                        position: spawn_pos + rotated * 0.2 * index as f32,
                        velocity: rotated * 18.0,
                        skill_type,
                        damage: def.damage * level_multiplier * 0.5,
                        aoe_radius: 0.5,
                        lifetime: 2.5,
                        owner_is_player: true,
                    },
                );
            }
            spawn_ice_effect(particle_system, spawn_pos);
        }
        SkillType::LightningBolt => {
            let spawn_pos = camera_pos + camera_forward * 1.0;
            spawn_projectile(
                combat_state,
                world,
                ProjectileParams {
                    position: spawn_pos,
                    velocity: camera_forward * 40.0,
                    skill_type,
                    damage: def.damage * level_multiplier,
                    aoe_radius: 0.0,
                    lifetime: 3.0,
                    owner_is_player: true,
                },
            );
            spawn_lightning_effect(particle_system, spawn_pos);
        }
        SkillType::Dash => {
            spawn_magic_effect(particle_system, camera_pos);
        }
        SkillType::Shield => {
            player_progress
                .skills
                .active_effects
                .push(crate::data::skills::ActiveEffect {
                    effect_type: EffectType::Shield,
                    duration_remaining: 5.0 + skill_level as f32,
                    strength: 0.5,
                });
            spawn_magic_effect(particle_system, camera_pos);
        }
        SkillType::Heal => {
            let heal_amount = 30.0 * level_multiplier;
            player_progress.stats.heal(heal_amount);
            spawn_heal_effect(particle_system, camera_pos);
        }
        SkillType::Blink => {
            spawn_magic_effect(particle_system, camera_pos);
        }
        SkillType::Explosion => {
            let explosion_pos = camera_pos + camera_forward * 6.0;
            spawn_explosion_effect(particle_system, explosion_pos);

            combat_state.projectiles.push(Projectile {
                entity: Entity {
                    id: 0,
                    generation: 0,
                },
                skill_type: SkillType::Explosion,
                position: explosion_pos,
                velocity: Vec3::zeros(),
                damage: def.damage * level_multiplier,
                aoe_radius: def.area_of_effect,
                lifetime: 0.1,
                owner_is_player: true,
                is_aoe_explosion: true,
            });
        }
    }

    true
}

fn rotate_vector_y(vec: Vec3, angle: f32) -> Vec3 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    Vec3::new(
        vec.x * cos_a + vec.z * sin_a,
        vec.y,
        -vec.x * sin_a + vec.z * cos_a,
    )
}

struct ProjectileParams {
    position: Vec3,
    velocity: Vec3,
    skill_type: SkillType,
    damage: f32,
    aoe_radius: f32,
    lifetime: f32,
    owner_is_player: bool,
}

fn spawn_projectile(combat_state: &mut CombatState, world: &mut World, params: ProjectileParams) {
    let def = get_skill_definition(params.skill_type).unwrap();

    let projectile_scale = match params.skill_type {
        SkillType::Fireball => 0.4,
        SkillType::IceBlast => 0.25,
        SkillType::LightningBolt => 0.3,
        _ => 0.3,
    };

    let entity = spawn_mesh(
        world,
        "Sphere",
        params.position,
        Vec3::new(projectile_scale, projectile_scale, projectile_scale),
    );

    let mat_name = format!("Projectile_{}", entity.id);
    let emissive_strength = 5.0;
    material_registry_insert(
        &mut world.resources.material_registry,
        mat_name.clone(),
        Material {
            base_color: def.color,
            roughness: 0.1,
            metallic: 0.8,
            emissive_factor: [
                def.color[0] * emissive_strength,
                def.color[1] * emissive_strength,
                def.color[2] * emissive_strength,
            ],
            unlit: true,
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
    world.set_material_ref(entity, MaterialRef::new(mat_name));

    combat_state.projectiles.push(Projectile {
        entity,
        skill_type: params.skill_type,
        position: params.position,
        velocity: params.velocity,
        damage: params.damage,
        aoe_radius: params.aoe_radius,
        lifetime: params.lifetime,
        owner_is_player: params.owner_is_player,
        is_aoe_explosion: false,
    });
}

pub fn spawn_enemy_projectile(
    combat_state: &mut CombatState,
    world: &mut World,
    position: Vec3,
    direction: Vec3,
    damage: f32,
) {
    let entity = spawn_mesh(world, "Sphere", position, Vec3::new(0.3, 0.3, 0.3));

    let mat_name = format!("EnemyProjectile_{}", entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        mat_name.clone(),
        Material {
            base_color: [0.8, 0.2, 0.2, 1.0],
            roughness: 0.1,
            metallic: 0.5,
            emissive_factor: [4.0, 1.0, 1.0],
            unlit: true,
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
    world.set_material_ref(entity, MaterialRef::new(mat_name));

    combat_state.projectiles.push(Projectile {
        entity,
        skill_type: SkillType::Fireball,
        position,
        velocity: direction * 12.0,
        damage,
        aoe_radius: 0.0,
        lifetime: 4.0,
        owner_is_player: false,
        is_aoe_explosion: false,
    });
}

pub fn update_projectiles(
    combat_state: &mut CombatState,
    enemies: &mut [Enemy],
    player_progress: &mut PlayerProgress,
    particle_system: &mut ParticleSystem,
    world: &mut World,
    player_pos: Vec3,
    delta_time: f32,
) {
    let mut to_remove = Vec::new();
    let mut enemy_damage_events: Vec<(usize, f32, Vec3, SkillType)> = Vec::new();
    let mut player_damage = 0.0;

    for (proj_index, projectile) in combat_state.projectiles.iter_mut().enumerate() {
        if projectile.is_aoe_explosion {
            for (enemy_index, enemy) in enemies.iter().enumerate() {
                if enemy.is_dead() {
                    continue;
                }

                let enemy_pos = world
                    .get_local_transform(enemy.entity)
                    .map(|t| t.translation)
                    .unwrap_or(Vec3::zeros());

                let distance = nalgebra_glm::length(&(projectile.position - enemy_pos));
                if distance < projectile.aoe_radius {
                    let falloff = 1.0 - (distance / projectile.aoe_radius).min(1.0);
                    let actual_damage = projectile.damage * falloff;
                    enemy_damage_events.push((
                        enemy_index,
                        actual_damage,
                        enemy_pos,
                        projectile.skill_type,
                    ));
                }
            }
            to_remove.push(proj_index);
            continue;
        }

        projectile.position += projectile.velocity * delta_time;
        projectile.lifetime -= delta_time;

        if !projectile.is_aoe_explosion {
            if let Some(transform) = world.get_local_transform_mut(projectile.entity) {
                transform.translation = projectile.position;
            }
            mark_local_transform_dirty(world, projectile.entity);
        }

        let hit_radius = match projectile.skill_type {
            SkillType::Fireball => 1.8,
            SkillType::IceBlast => 1.2,
            SkillType::LightningBolt => 1.5,
            _ => 1.5,
        };

        if projectile.owner_is_player {
            for (enemy_index, enemy) in enemies.iter().enumerate() {
                if enemy.is_dead() {
                    continue;
                }

                let enemy_pos = world
                    .get_local_transform(enemy.entity)
                    .map(|t| t.translation)
                    .unwrap_or(Vec3::zeros());

                let distance = nalgebra_glm::length(&(projectile.position - enemy_pos));
                if distance < hit_radius {
                    if projectile.aoe_radius > 0.0 {
                        for (other_enemy_index, other_enemy) in enemies.iter().enumerate() {
                            if other_enemy.is_dead() {
                                continue;
                            }
                            let other_pos = world
                                .get_local_transform(other_enemy.entity)
                                .map(|t| t.translation)
                                .unwrap_or(Vec3::zeros());
                            let dist_to_impact =
                                nalgebra_glm::length(&(projectile.position - other_pos));
                            if dist_to_impact < projectile.aoe_radius {
                                let falloff =
                                    1.0 - (dist_to_impact / projectile.aoe_radius).min(1.0);
                                let aoe_damage = projectile.damage * falloff;
                                enemy_damage_events.push((
                                    other_enemy_index,
                                    aoe_damage,
                                    other_pos,
                                    projectile.skill_type,
                                ));
                            }
                        }
                    } else {
                        enemy_damage_events.push((
                            enemy_index,
                            projectile.damage,
                            enemy_pos,
                            projectile.skill_type,
                        ));
                    }

                    to_remove.push(proj_index);
                    spawn_impact_effect(particle_system, enemy_pos, projectile.skill_type);
                    break;
                }
            }
        } else {
            let distance = nalgebra_glm::length(&(projectile.position - player_pos));
            if distance < 1.2 {
                player_damage += projectile.damage;
                to_remove.push(proj_index);
                spawn_damage_effect(particle_system, player_pos);
            }
        }

        if projectile.lifetime <= 0.0 {
            to_remove.push(proj_index);
        }
    }

    for (enemy_index, damage, _pos, _skill) in enemy_damage_events {
        if let Some(enemy) = enemies.get_mut(enemy_index)
            && !enemy.is_dead()
        {
            enemy.take_damage(damage);
            player_progress.total_damage_dealt += damage;
        }
    }

    if player_damage > 0.0 {
        let shield_reduction = get_shield_reduction(player_progress);
        let reduced_damage = player_damage * (1.0 - shield_reduction);
        let actual = player_progress.stats.take_damage(reduced_damage);
        player_progress.total_damage_taken += actual;
    }

    to_remove.sort();
    to_remove.dedup();
    for index in to_remove.into_iter().rev() {
        if index < combat_state.projectiles.len() {
            let projectile = combat_state.projectiles.remove(index);
            if !projectile.is_aoe_explosion {
                world.despawn_entities(&[projectile.entity]);
            }
        }
    }
}

fn spawn_impact_effect(
    particle_system: &mut ParticleSystem,
    position: Vec3,
    skill_type: SkillType,
) {
    match skill_type {
        SkillType::Fireball => {
            spawn_fireball_effect(particle_system, position);
            spawn_explosion_effect(particle_system, position);
        }
        SkillType::IceBlast => spawn_ice_effect(particle_system, position),
        SkillType::LightningBolt => spawn_lightning_effect(particle_system, position),
        SkillType::Explosion => spawn_explosion_effect(particle_system, position),
        _ => spawn_damage_effect(particle_system, position),
    }
}

fn get_shield_reduction(player_progress: &PlayerProgress) -> f32 {
    for effect in &player_progress.skills.active_effects {
        if effect.effect_type == EffectType::Shield {
            return effect.strength;
        }
    }
    0.0
}

pub fn update_enemy_ai(
    enemies: &mut [Enemy],
    combat_state: &mut CombatState,
    particle_system: &mut ParticleSystem,
    world: &mut World,
    player_pos: Vec3,
    delta_time: f32,
) {
    for enemy in enemies.iter_mut() {
        if enemy.is_dead() {
            continue;
        }

        enemy.attack_cooldown = (enemy.attack_cooldown - delta_time).max(0.0);
        enemy.stun_duration = (enemy.stun_duration - delta_time).max(0.0);
        enemy.damage_flash_timer = (enemy.damage_flash_timer - delta_time).max(0.0);

        if enemy.stun_duration > 0.0 {
            enemy.state = EnemyState::Stunned;
            continue;
        }

        let def = get_enemy_definition(enemy.enemy_type).unwrap();
        let enemy_pos = world
            .get_local_transform(enemy.entity)
            .map(|t| t.translation)
            .unwrap_or(enemy.home_position);

        let to_player = player_pos - enemy_pos;
        let distance_to_player = nalgebra_glm::length(&to_player);

        if distance_to_player < def.detection_range {
            enemy.last_known_player_pos = Some(player_pos);

            if distance_to_player <= def.attack_range {
                enemy.state = EnemyState::Attack;

                if enemy.attack_cooldown <= 0.0 && def.attack_range > 3.0 {
                    enemy.attack_cooldown = def.attack_cooldown;
                    let direction = nalgebra_glm::normalize(&to_player);
                    let spawn_pos = enemy_pos + Vec3::new(0.0, 1.0, 0.0) + direction * 0.5;
                    spawn_enemy_projectile(combat_state, world, spawn_pos, direction, def.damage);
                    spawn_magic_effect(particle_system, spawn_pos);
                }
            } else {
                enemy.state = EnemyState::Chase;
                let direction = nalgebra_glm::normalize(&to_player);
                let new_pos = enemy_pos + direction * def.speed * delta_time;

                if let Some(transform) = world.get_local_transform_mut(enemy.entity) {
                    transform.translation = new_pos;

                    let look_direction = Vec3::new(direction.x, 0.0, direction.z);
                    if nalgebra_glm::length(&look_direction) > 0.01 {
                        let angle = look_direction.x.atan2(look_direction.z);
                        transform.rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::y());
                    }
                }
                mark_local_transform_dirty(world, enemy.entity);
            }
        } else {
            enemy.state = EnemyState::Patrol;

            if enemy.patrol_target.is_none() || rand::random::<f32>() < 0.005 {
                let angle = rand::random::<f32>() * std::f32::consts::TAU;
                let radius = rand::random::<f32>() * 5.0;
                enemy.patrol_target = Some(
                    enemy.home_position
                        + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius),
                );
            }

            if let Some(target) = enemy.patrol_target {
                let to_target = target - enemy_pos;
                let dist = nalgebra_glm::length(&to_target);

                if dist > 0.5 {
                    let direction = nalgebra_glm::normalize(&to_target);
                    let new_pos = enemy_pos + direction * def.speed * 0.4 * delta_time;

                    if let Some(transform) = world.get_local_transform_mut(enemy.entity) {
                        transform.translation = new_pos;
                    }
                    mark_local_transform_dirty(world, enemy.entity);
                } else {
                    enemy.patrol_target = None;
                }
            }
        }
    }
}

pub fn check_melee_combat(
    enemies: &mut [Enemy],
    player_progress: &mut PlayerProgress,
    particle_system: &mut ParticleSystem,
    world: &mut World,
    player_pos: Vec3,
    _delta_time: f32,
) {
    for enemy in enemies.iter_mut() {
        if enemy.is_dead() {
            continue;
        }

        let def = get_enemy_definition(enemy.enemy_type).unwrap();

        if def.attack_range <= 3.0
            && enemy.state == EnemyState::Attack
            && enemy.attack_cooldown <= 0.0
        {
            let enemy_pos = world
                .get_local_transform(enemy.entity)
                .map(|t| t.translation)
                .unwrap_or(enemy.home_position);

            let distance = nalgebra_glm::length(&(player_pos - enemy_pos));

            if distance <= def.attack_range + 0.5 {
                enemy.attack_cooldown = def.attack_cooldown;

                let shield_reduction = get_shield_reduction(player_progress);
                let reduced_damage = def.damage * (1.0 - shield_reduction);
                let actual = player_progress.stats.take_damage(reduced_damage);
                player_progress.total_damage_taken += actual;
                spawn_damage_effect(particle_system, player_pos);
            }
        }
    }
}

pub fn get_loot_drop(enemy_type: crate::data::enemies::EnemyType) -> Option<(ItemType, usize)> {
    let def = get_enemy_definition(enemy_type)?;

    if rand::random::<f32>() > def.loot_chance {
        return None;
    }

    let roll = rand::random::<f32>();
    Some(if roll < 0.4 {
        (ItemType::Coin, (rand::random::<f32>() * 10.0) as usize + 5)
    } else if roll < 0.6 {
        (ItemType::HealthPotion, 1)
    } else if roll < 0.75 {
        (ItemType::ManaPotion, 1)
    } else if roll < 0.85 {
        (ItemType::Gem, (rand::random::<f32>() * 3.0) as usize + 1)
    } else if roll < 0.95 {
        (ItemType::SpeedPotion, 1)
    } else {
        (ItemType::Scroll, 1)
    })
}
