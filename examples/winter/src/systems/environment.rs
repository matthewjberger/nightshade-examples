use crate::constants::{
    BAUBLE_COLORS, PRESENTS_MODEL, SNOWMAN_MODEL, SNOWMAN2_MODEL, SNOWY_HUT_MODEL,
};
use crate::ecs::{GameWorld, TerrainConfig};
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::particles::components::{
    ColorGradient, EmitterShape, EmitterType, ParticleEmitter,
};
use nightshade::ecs::physics::{create_textured_material, spawn_static_physics_cube_with_material};
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::terrain::spawn_terrain_with_material;
use nightshade::prelude::*;

pub fn spawn_environment(game_world: &mut GameWorld, world: &mut World) {
    spawn_terrain(game_world, world);
    spawn_huts(game_world, world);
    spawn_trees(game_world, world);
    spawn_rocks(game_world, world);
    spawn_campfire(game_world, world);
    spawn_snowmen(game_world, world);
}

pub fn sample_height(x: f32, z: f32, config: &TerrainConfig) -> f32 {
    nightshade::ecs::terrain::sample_terrain_height(x, z, &config.to_nightshade_config())
}

const HUT_POSITIONS: [(f32, f32, f32); 5] = [
    (18.0, 12.0, 0.3),
    (-20.0, 8.0, 2.5),
    (15.0, -18.0, 1.2),
    (-12.0, -22.0, 0.8),
    (-25.0, -5.0, 1.8),
];

const HUT_EXCLUSION_RADIUS: f32 = 5.0;

fn spawn_huts(game_world: &GameWorld, world: &mut World) {
    for (x, z, rotation) in HUT_POSITIONS {
        let terrain_y = sample_height(x, z, &game_world.resources.terrain_config);
        let hut_y = terrain_y - 0.3;

        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(SNOWY_HUT_MODEL);

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

                for prefab in result.prefabs {
                    let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                        world,
                        &prefab,
                        &result.animations,
                        &result.skins,
                        Vec3::new(x, hut_y, z),
                    );

                    if let Some(transform) = world.get_local_transform_mut(entity) {
                        transform.translation = Vec3::new(x, hut_y, z);
                        transform.rotation = nalgebra_glm::quat_angle_axis(rotation, &Vec3::y());
                        transform.scale = Vec3::new(1.0, 1.0, 1.0);
                    }
                    world.mark_local_transform_dirty(entity);
                }
            }
            Err(e) => {
                tracing::error!("Failed to load snowy hut model: {}", e);
            }
        }
    }
}

fn spawn_terrain(game_world: &GameWorld, world: &mut World) {
    let snow_material = Material {
        base_color: [0.95, 0.97, 1.0, 1.0],
        roughness: 0.85,
        metallic: 0.0,
        ..Default::default()
    };

    spawn_terrain_with_material(
        world,
        game_world.resources.terrain_config.to_nightshade_config(),
        Vec3::zeros(),
        snow_material,
    );
}

fn is_near_hut(x: f32, z: f32) -> bool {
    for (hut_x, hut_z, _) in HUT_POSITIONS {
        let dx = x - hut_x;
        let dz = z - hut_z;
        let dist = (dx * dx + dz * dz).sqrt();
        if dist < HUT_EXCLUSION_RADIUS {
            return true;
        }
    }
    false
}

fn spawn_trees(game_world: &mut GameWorld, world: &mut World) {
    let tree_positions: Vec<(f32, f32, f32, f32)> = (0..60)
        .map(|index| {
            let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
            let theta = index as f32 * golden_angle;
            let r = (index as f32 / 60.0).sqrt() * 25.0 + 5.0;
            let x = theta.cos() * r;
            let z = theta.sin() * r;
            let dist_from_center = (x * x + z * z).sqrt();
            if dist_from_center < 4.0 || is_near_hut(x, z) {
                return (x * 2.0, z * 2.0, 0.0, 0.0);
            }
            let trunk_height = 0.8 + (index as f32 * 0.02) % 0.4;
            let trunk_radius = 0.12 + (index as f32 * 0.002) % 0.08;
            (x, z, trunk_height, trunk_radius)
        })
        .filter(|(_, _, trunk, _)| *trunk > 0.0)
        .collect();

    let bauble_colors: Vec<Vec3> = BAUBLE_COLORS
        .iter()
        .map(|c| Vec3::new(c[0], c[1], c[2]))
        .collect();

    let mut all_string_lights: Vec<Line> = Vec::new();

    for (tree_index, (x, z, trunk_height, trunk_radius)) in tree_positions.iter().enumerate() {
        let terrain_y = sample_height(*x, *z, &game_world.resources.terrain_config);

        let trunk_material = create_textured_material(Vec3::new(0.35, 0.2, 0.1), 0.9, 0.0);
        spawn_static_physics_cube_with_material(
            world,
            Vec3::new(*x, terrain_y + trunk_height / 2.0, *z),
            Vec3::new(*trunk_radius * 2.0, *trunk_height, *trunk_radius * 2.0),
            trunk_material,
        );

        let tree_scale = 0.8 + (tree_index as f32 * 0.015) % 0.6;
        let green_variation = (tree_index as f32 * 0.02) % 0.15;
        let num_tiers = 3;
        let tier_radii = [2.4 * tree_scale, 1.8 * tree_scale, 1.1 * tree_scale];
        let tier_heights = [1.6 * tree_scale, 1.4 * tree_scale, 1.2 * tree_scale];
        let tier_y_offsets = [0.0, 1.0 * tree_scale, 1.9 * tree_scale];

        for tier in 0..num_tiers {
            let radius = tier_radii[tier];
            let height = tier_heights[tier];
            let y_pos = terrain_y + *trunk_height + tier_y_offsets[tier] + height / 2.0;

            let cone = world.spawn_entities(
                LOCAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | GLOBAL_TRANSFORM
                    | RENDER_MESH
                    | MATERIAL_REF
                    | CASTS_SHADOW,
                1,
            )[0];
            world.set_local_transform(
                cone,
                LocalTransform {
                    translation: Vec3::new(*x, y_pos, *z),
                    rotation: Quat::identity(),
                    scale: Vec3::new(radius, height, radius),
                },
            );
            world.set_render_mesh(cone, RenderMesh::new("Cone"));

            let cone_material_name = format!("TreeCone_{}_{}", tree_index, tier);
            material_registry_insert(
                &mut world.resources.material_registry,
                cone_material_name.clone(),
                Material {
                    base_color: [0.08, 0.35 + green_variation, 0.05, 1.0],
                    roughness: 0.95,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
            if let Some(&mat_index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&cone_material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(mat_index);
            }
            world.set_material_ref(cone, MaterialRef::new(cone_material_name));
            world.set_casts_shadow(cone, CastsShadow);
        }

        let mut tree_bauble_positions: Vec<(Vec3, Vec3)> = Vec::new();

        let tree_base_y = terrain_y + *trunk_height;
        let total_visual_height = tier_y_offsets[2] + tier_heights[2];
        let num_spiral_baubles = 18;
        let num_spiral_rotations = 3.0;

        for bauble_index in 0..num_spiral_baubles {
            let progress = bauble_index as f32 / (num_spiral_baubles - 1) as f32;
            let angle =
                progress * num_spiral_rotations * std::f32::consts::TAU + (tree_index as f32 * 0.7);
            let height_in_tree = progress * total_visual_height * 0.9;
            let bauble_height = tree_base_y + height_in_tree;

            let mut best_radius = 0.0_f32;
            for tier in 0..num_tiers {
                let tier_base = tier_y_offsets[tier];
                let tier_top = tier_base + tier_heights[tier];
                if height_in_tree >= tier_base && height_in_tree <= tier_top {
                    let local_progress = (height_in_tree - tier_base) / tier_heights[tier];
                    let cone_radius = tier_radii[tier] * 0.5 * (1.0 - local_progress);
                    best_radius = best_radius.max(cone_radius);
                }
            }

            let radius_at_height = best_radius * 1.15;

            let bauble_x = *x + angle.cos() * radius_at_height;
            let bauble_z = *z + angle.sin() * radius_at_height;
            let bauble_size = 0.08 * tree_scale * (1.0 - progress * 0.3);
            let color = bauble_colors[(tree_index + bauble_index) % bauble_colors.len()];

            let bauble_pos = Vec3::new(bauble_x, bauble_height, bauble_z);
            tree_bauble_positions.push((bauble_pos, color));

            spawn_bauble(
                world,
                bauble_pos,
                bauble_size,
                color,
                tree_index,
                0,
                bauble_index,
            );
        }

        for index in 0..(tree_bauble_positions.len() - 1) {
            let (start_pos, start_color) = tree_bauble_positions[index];
            let (end_pos, end_color) = tree_bauble_positions[index + 1];
            let avg_color = (start_color + end_color) * 0.5;
            all_string_lights.push(Line {
                start: start_pos,
                end: end_pos,
                color: Vec4::new(avg_color.x, avg_color.y, avg_color.z, 1.0),
            });
        }

        spawn_tree_star(
            world,
            Vec3::new(*x, terrain_y, *z),
            *trunk_height,
            tree_scale,
            tree_index,
        );

        if tree_index % 3 == 0 {
            spawn_presents(game_world, world, *x, *z, tree_index);
        }
    }

    let string_lights_entity = world.spawn_entities(
        LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | LINES | VISIBILITY,
        1,
    )[0];
    world.set_local_transform(string_lights_entity, LocalTransform::default());
    world.set_global_transform(string_lights_entity, GlobalTransform::default());
    world.set_local_transform_dirty(string_lights_entity, LocalTransformDirty);
    world.set_lines(string_lights_entity, Lines::new(all_string_lights));
    world.set_visibility(string_lights_entity, Visibility { visible: true });
    game_world.resources.string_lights_entity = Some(freecs::Entity {
        id: string_lights_entity.id,
        generation: string_lights_entity.generation,
    });
}

fn spawn_bauble(
    world: &mut World,
    position: Vec3,
    size: f32,
    color: Vec3,
    tree_index: usize,
    tier: usize,
    bauble_index: usize,
) {
    let bauble = world.spawn_entities(
        LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | CASTS_SHADOW,
        1,
    )[0];
    world.set_local_transform(
        bauble,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(size, size, size),
        },
    );
    world.set_render_mesh(bauble, RenderMesh::new("Sphere"));

    let bauble_material_name = format!("Bauble_{}_{}_{}", tree_index, tier, bauble_index);
    material_registry_insert(
        &mut world.resources.material_registry,
        bauble_material_name.clone(),
        Material {
            base_color: [color.x, color.y, color.z, 1.0],
            roughness: 0.3,
            metallic: 0.5,
            emissive_factor: [color.x * 3.0, color.y * 3.0, color.z * 3.0],
            ..Default::default()
        },
    );
    if let Some(&mat_index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&bauble_material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(mat_index);
    }
    world.set_material_ref(bauble, MaterialRef::new(bauble_material_name));
    world.set_casts_shadow(bauble, CastsShadow);

    let sparkle_gradient = ColorGradient {
        colors: vec![
            (0.0, Vec4::new(color.x, color.y, color.z, 0.0)),
            (0.2, Vec4::new(color.x, color.y, color.z, 0.8)),
            (
                0.5,
                Vec4::new(color.x * 1.2, color.y * 1.2, color.z * 1.2, 0.6),
            ),
            (0.8, Vec4::new(color.x, color.y, color.z, 0.3)),
            (
                1.0,
                Vec4::new(color.x * 0.5, color.y * 0.5, color.z * 0.5, 0.0),
            ),
        ],
    };
    let sparkle_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
    world.set_particle_emitter(
        sparkle_entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.02 },
            position,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 2.0,
            burst_count: 0,
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.5,
            initial_velocity_min: 0.05,
            initial_velocity_max: 0.15,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 0.05, 0.0),
            drag: 0.5,
            size_start: 0.02,
            size_end: 0.005,
            color_gradient: sparkle_gradient,
            emissive_strength: 8.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.3,
            turbulence_frequency: 2.0,

            ..Default::default()
        },
    );
}

fn spawn_tree_star(
    world: &mut World,
    base_position: Vec3,
    trunk_height: f32,
    tree_scale: f32,
    tree_index: usize,
) {
    let tier_heights = [1.6 * tree_scale, 1.4 * tree_scale, 1.2 * tree_scale];
    let tier_y_offsets = [0.0, 1.0 * tree_scale, 1.9 * tree_scale];
    let num_tiers = 3;
    let top_cone_height = tier_heights[num_tiers - 1];
    let star_height =
        base_position.y + trunk_height + tier_y_offsets[num_tiers - 1] + top_cone_height + 0.08;
    let star_size = 0.15 * tree_scale;

    let star = world.spawn_entities(
        LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | CASTS_SHADOW,
        1,
    )[0];
    world.set_local_transform(
        star,
        LocalTransform {
            translation: Vec3::new(base_position.x, star_height, base_position.z),
            rotation: Quat::identity(),
            scale: Vec3::new(star_size, star_size * 1.5, star_size),
        },
    );
    world.set_render_mesh(star, RenderMesh::new("Sphere"));

    let star_material_name = format!("Star_{}", tree_index);
    material_registry_insert(
        &mut world.resources.material_registry,
        star_material_name.clone(),
        Material {
            base_color: [1.0, 0.9, 0.2, 1.0],
            roughness: 0.1,
            metallic: 0.9,
            emissive_factor: [2.0, 1.8, 0.4],
            ..Default::default()
        },
    );
    if let Some(&mat_index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&star_material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(mat_index);
    }
    world.set_material_ref(star, MaterialRef::new(star_material_name));
    world.set_casts_shadow(star, CastsShadow);
}

fn spawn_rocks(game_world: &GameWorld, world: &mut World) {
    let rock_data = [
        (Vec3::new(-3.0, 0.0, 2.0), 0.4),
        (Vec3::new(2.5, 0.0, -2.5), 0.3),
        (Vec3::new(-2.0, 0.0, -3.0), 0.35),
        (Vec3::new(4.0, 0.0, 1.5), 0.25),
        (Vec3::new(0.0, 0.0, 4.0), 0.3),
        (Vec3::new(-8.0, 0.0, 6.0), 0.6),
        (Vec3::new(10.0, 0.0, -8.0), 0.7),
        (Vec3::new(-6.0, 0.0, -10.0), 0.5),
        (Vec3::new(7.0, 0.0, 7.0), 0.6),
        (Vec3::new(-12.0, 0.0, -3.0), 0.45),
        (Vec3::new(5.0, 0.0, -12.0), 0.8),
        (Vec3::new(-4.0, 0.0, 11.0), 0.5),
        (Vec3::new(12.0, 0.0, 2.0), 0.55),
        (Vec3::new(-14.0, 0.0, 8.0), 0.65),
        (Vec3::new(3.0, 0.0, 14.0), 0.5),
    ];

    for (index, (pos, size)) in rock_data.iter().enumerate() {
        let terrain_y = sample_height(pos.x, pos.z, &game_world.resources.terrain_config);
        let gray = 0.4 + (index as f32 * 0.05) % 0.2;
        let rock_material = create_textured_material(Vec3::new(gray, gray, gray), 0.95, 0.0);

        let rock = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | CASTS_SHADOW
                | RIGID_BODY
                | COLLIDER
                | BOUNDING_VOLUME
                | VISIBILITY,
            1,
        )[0];

        let rock_y = terrain_y + size * 0.4;
        world.set_name(rock, Name(format!("Rock_{}", index)));
        world.set_local_transform(
            rock,
            LocalTransform {
                translation: Vec3::new(pos.x, rock_y, pos.z),
                rotation: Quat::identity(),
                scale: Vec3::new(*size, size * 0.7, *size),
            },
        );
        world.set_render_mesh(rock, RenderMesh::new("Sphere"));

        let material_name = format!("RockMat_{}", rock.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            rock_material,
        );
        if let Some(&mat_index) = world
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
                .add_reference(mat_index);
        }
        world.set_material_ref(rock, MaterialRef::new(material_name));

        if let Some(bv) = world.get_bounding_volume_mut(rock) {
            *bv = BoundingVolume::from_mesh_type("Sphere");
        }

        if let Some(rigid_body) = world.get_rigid_body_mut(rock) {
            *rigid_body = nightshade::ecs::physics::RigidBodyComponent::new_static()
                .with_translation(pos.x, rock_y, pos.z);
        }

        if let Some(collider) = world.get_collider_mut(rock) {
            *collider = nightshade::ecs::physics::ColliderComponent::new_ball(size * 0.5)
                .with_friction(0.8)
                .with_restitution(0.1);
        }

        world.set_casts_shadow(rock, CastsShadow);
        world.set_visibility(rock, Visibility { visible: true });
    }
}

fn spawn_campfire(game_world: &mut GameWorld, world: &mut World) {
    let campfire_x = 8.0;
    let campfire_z = 5.0;
    let terrain_y = sample_height(campfire_x, campfire_z, &game_world.resources.terrain_config);

    spawn_campfire_logs(world, campfire_x, campfire_z, terrain_y);
    spawn_campfire_particles(world, campfire_x, campfire_z, terrain_y);

    let light_entity = spawn_campfire_light(world, campfire_x, campfire_z, terrain_y);
    game_world.resources.campfire_light = Some(freecs::Entity {
        id: light_entity.id,
        generation: light_entity.generation,
    });
}

fn spawn_campfire_logs(world: &mut World, campfire_x: f32, campfire_z: f32, terrain_y: f32) {
    let log_material = create_textured_material(Vec3::new(0.35, 0.2, 0.1), 0.95, 0.0);
    let log_radius = 0.25;
    let log_length = 1.8;

    let logs: [(f32, f32, f32, f32, f32); 6] = [
        (0.0, 0.2, 0.65, 0.0, std::f32::consts::FRAC_PI_2),
        (0.0, 0.2, -0.65, 0.0, std::f32::consts::FRAC_PI_2),
        (-0.55, 0.45, 0.0, std::f32::consts::FRAC_PI_4, 0.45),
        (0.55, 0.45, 0.0, -std::f32::consts::FRAC_PI_4, 0.45),
        (-0.3, 0.65, 0.3, std::f32::consts::FRAC_PI_6, 0.6),
        (0.3, 0.65, -0.3, -std::f32::consts::FRAC_PI_6, 0.6),
    ];

    for (index, (offset_x, offset_y, offset_z, yaw, pitch)) in logs.iter().enumerate() {
        let log = world.spawn_entities(
            LOCAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | GLOBAL_TRANSFORM
                | RENDER_MESH
                | MATERIAL_REF
                | CASTS_SHADOW,
            1,
        )[0];

        let pitch_rotation = nalgebra_glm::quat_angle_axis(*pitch, &Vec3::new(1.0, 0.0, 0.0));
        let yaw_rotation = nalgebra_glm::quat_angle_axis(*yaw, &Vec3::new(0.0, 1.0, 0.0));
        let roll_rotation =
            nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::new(0.0, 0.0, 1.0));

        world.set_local_transform(
            log,
            LocalTransform {
                translation: Vec3::new(
                    campfire_x + offset_x,
                    terrain_y + offset_y,
                    campfire_z + offset_z,
                ),
                rotation: yaw_rotation * pitch_rotation * roll_rotation,
                scale: Vec3::new(log_radius, log_length, log_radius),
            },
        );
        world.set_render_mesh(log, RenderMesh::new("Cylinder"));

        let log_material_name = format!("CampfireLog_{}", index);
        material_registry_insert(
            &mut world.resources.material_registry,
            log_material_name.clone(),
            log_material.clone(),
        );
        if let Some(&mat_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&log_material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(mat_index);
        }
        world.set_material_ref(log, MaterialRef::new(log_material_name));
        world.set_casts_shadow(log, CastsShadow);
    }
}

fn spawn_campfire_particles(world: &mut World, campfire_x: f32, campfire_z: f32, terrain_y: f32) {
    let fire_core_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
    let fire_core_gradient = ColorGradient {
        colors: vec![
            (0.0, Vec4::new(1.0, 1.0, 0.9, 0.0)),
            (0.1, Vec4::new(1.0, 0.95, 0.7, 1.0)),
            (0.3, Vec4::new(1.0, 0.8, 0.4, 0.9)),
            (0.6, Vec4::new(1.0, 0.5, 0.1, 0.6)),
            (0.85, Vec4::new(0.9, 0.2, 0.02, 0.2)),
            (1.0, Vec4::new(0.5, 0.05, 0.0, 0.0)),
        ],
    };
    world.set_particle_emitter(
        fire_core_entity,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Sphere { radius: 0.04 },
            position: Vec3::new(campfire_x, terrain_y + 0.35, campfire_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 50.0,
            burst_count: 0,
            particle_lifetime_min: 0.25,
            particle_lifetime_max: 0.5,
            initial_velocity_min: 0.8,
            initial_velocity_max: 1.5,
            velocity_spread: 0.12,
            gravity: Vec3::new(0.0, 2.5, 0.0),
            drag: 1.2,
            size_start: 0.12,
            size_end: 0.04,
            color_gradient: fire_core_gradient,
            emissive_strength: 18.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 1.2,
            turbulence_frequency: 4.5,

            ..Default::default()
        },
    );

    let fire_outer_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
    let fire_outer_gradient = ColorGradient {
        colors: vec![
            (0.0, Vec4::new(1.0, 0.7, 0.2, 0.0)),
            (0.15, Vec4::new(1.0, 0.5, 0.1, 0.85)),
            (0.4, Vec4::new(1.0, 0.3, 0.05, 0.7)),
            (0.7, Vec4::new(0.8, 0.15, 0.02, 0.35)),
            (0.9, Vec4::new(0.4, 0.05, 0.0, 0.1)),
            (1.0, Vec4::new(0.2, 0.02, 0.0, 0.0)),
        ],
    };
    world.set_particle_emitter(
        fire_outer_entity,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Sphere { radius: 0.08 },
            position: Vec3::new(campfire_x, terrain_y + 0.3, campfire_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 70.0,
            burst_count: 0,
            particle_lifetime_min: 0.3,
            particle_lifetime_max: 0.7,
            initial_velocity_min: 0.6,
            initial_velocity_max: 1.4,
            velocity_spread: 0.25,
            gravity: Vec3::new(0.0, 2.0, 0.0),
            drag: 0.9,
            size_start: 0.15,
            size_end: 0.03,
            color_gradient: fire_outer_gradient,
            emissive_strength: 12.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 1.5,
            turbulence_frequency: 4.0,

            ..Default::default()
        },
    );

    let fire_flicker_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
    let fire_flicker_gradient = ColorGradient {
        colors: vec![
            (0.0, Vec4::new(1.0, 0.6, 0.15, 0.0)),
            (0.2, Vec4::new(1.0, 0.45, 0.08, 0.7)),
            (0.5, Vec4::new(0.9, 0.25, 0.03, 0.5)),
            (0.8, Vec4::new(0.6, 0.1, 0.01, 0.2)),
            (1.0, Vec4::new(0.3, 0.03, 0.0, 0.0)),
        ],
    };
    world.set_particle_emitter(
        fire_flicker_entity,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Sphere { radius: 0.1 },
            position: Vec3::new(campfire_x, terrain_y + 0.25, campfire_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 30.0,
            burst_count: 0,
            particle_lifetime_min: 0.4,
            particle_lifetime_max: 0.9,
            initial_velocity_min: 0.4,
            initial_velocity_max: 1.0,
            velocity_spread: 0.35,
            gravity: Vec3::new(0.0, 1.2, 0.0),
            drag: 0.7,
            size_start: 0.08,
            size_end: 0.01,
            color_gradient: fire_flicker_gradient,
            emissive_strength: 10.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 2.5,
            turbulence_frequency: 5.5,

            ..Default::default()
        },
    );

    let smoke_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
    let smoke_gradient = ColorGradient {
        colors: vec![
            (0.0, Vec4::new(0.15, 0.12, 0.1, 0.0)),
            (0.05, Vec4::new(0.2, 0.18, 0.15, 0.25)),
            (0.15, Vec4::new(0.3, 0.28, 0.25, 0.4)),
            (0.4, Vec4::new(0.4, 0.38, 0.36, 0.35)),
            (0.7, Vec4::new(0.5, 0.49, 0.47, 0.2)),
            (0.9, Vec4::new(0.6, 0.58, 0.56, 0.08)),
            (1.0, Vec4::new(0.65, 0.63, 0.6, 0.0)),
        ],
    };
    world.set_particle_emitter(
        smoke_entity,
        ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Sphere { radius: 0.1 },
            position: Vec3::new(campfire_x, terrain_y + 1.0, campfire_z),
            direction: Vec3::new(0.1, 1.0, 0.05).normalize(),
            spawn_rate: 25.0,
            burst_count: 0,
            particle_lifetime_min: 5.0,
            particle_lifetime_max: 10.0,
            initial_velocity_min: 0.3,
            initial_velocity_max: 0.7,
            velocity_spread: 0.12,
            gravity: Vec3::new(0.05, 0.2, 0.02),
            drag: 0.05,
            size_start: 0.2,
            size_end: 2.5,
            color_gradient: smoke_gradient,
            emissive_strength: 0.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 0.6,
            turbulence_frequency: 0.2,

            ..Default::default()
        },
    );

    let ember_entity = world.spawn_entities(PARTICLE_EMITTER, 1)[0];
    let ember_gradient = ColorGradient {
        colors: vec![
            (0.0, Vec4::new(1.0, 0.7, 0.2, 0.0)),
            (0.1, Vec4::new(1.0, 0.6, 0.15, 1.0)),
            (0.4, Vec4::new(1.0, 0.4, 0.08, 0.9)),
            (0.7, Vec4::new(0.9, 0.25, 0.03, 0.6)),
            (0.9, Vec4::new(0.6, 0.1, 0.01, 0.2)),
            (1.0, Vec4::new(0.3, 0.03, 0.0, 0.0)),
        ],
    };
    world.set_particle_emitter(
        ember_entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.15 },
            position: Vec3::new(campfire_x, terrain_y + 0.45, campfire_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 6.0,
            burst_count: 0,
            particle_lifetime_min: 2.0,
            particle_lifetime_max: 4.0,
            initial_velocity_min: 0.6,
            initial_velocity_max: 2.0,
            velocity_spread: 0.35,
            gravity: Vec3::new(0.02, 0.3, 0.01),
            drag: 0.15,
            size_start: 0.025,
            size_end: 0.006,
            color_gradient: ember_gradient,
            emissive_strength: 15.0,
            enabled: true,
            accumulated_spawn: 0.0,
            one_shot: false,
            has_fired: false,
            turbulence_strength: 1.0,
            turbulence_frequency: 1.0,

            ..Default::default()
        },
    );
}

fn spawn_campfire_light(
    world: &mut World,
    campfire_x: f32,
    campfire_z: f32,
    terrain_y: f32,
) -> nightshade::prelude::Entity {
    let light_entity = world.spawn_entities(
        LIGHT | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM,
        1,
    )[0];
    world.set_local_transform(
        light_entity,
        LocalTransform {
            translation: Vec3::new(campfire_x, terrain_y + 0.7, campfire_z),
            ..Default::default()
        },
    );
    world.set_light(
        light_entity,
        Light {
            light_type: LightType::Point,
            color: Vec3::new(1.0, 0.7, 0.3),
            intensity: 3.5,
            range: 15.0,
            cast_shadows: false,
            ..Default::default()
        },
    );
    light_entity
}

pub fn update_campfire_light(game_world: &GameWorld, world: &mut World) {
    let Some(light_entity) = game_world.resources.campfire_light else {
        return;
    };

    let engine_entity = nightshade::prelude::Entity {
        id: light_entity.id,
        generation: light_entity.generation,
    };

    let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
    if let Some(light) = world.get_light_mut(engine_entity) {
        let flicker1 = (time * 8.0).sin() * 0.15;
        let flicker2 = (time * 12.5).sin() * 0.1;
        let flicker3 = (time * 23.0).sin() * 0.08;
        let flicker4 = (time * 3.5).sin() * 0.05;
        let base_intensity = 3.5;
        light.intensity = base_intensity + flicker1 + flicker2 + flicker3 + flicker4;
    }
}

fn spawn_snowmen(game_world: &GameWorld, world: &mut World) {
    let snowman_positions = [
        (5.0_f32, -8.0_f32, 0.0_f32),
        (-8.0, 5.0, 0.5),
        (12.0, 3.0, -0.3),
        (-5.0, -15.0, 0.2),
        (10.0, -12.0, 0.7),
    ];

    for (index, (x, z, rotation)) in snowman_positions.iter().enumerate() {
        let terrain_y = sample_height(*x, *z, &game_world.resources.terrain_config);
        let (model, y_offset) = if index % 2 == 0 {
            (SNOWMAN_MODEL, 0.0)
        } else {
            (SNOWMAN2_MODEL, 0.0)
        };
        let snowman_y = terrain_y + y_offset;

        let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(model);

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

                for prefab in result.prefabs {
                    let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                        world,
                        &prefab,
                        &result.animations,
                        &result.skins,
                        Vec3::new(*x, snowman_y, *z),
                    );

                    if let Some(transform) = world.get_local_transform_mut(entity) {
                        transform.translation = Vec3::new(*x, snowman_y, *z);
                        transform.rotation = nalgebra_glm::quat_angle_axis(*rotation, &Vec3::y());
                        transform.scale = Vec3::new(1.0, 1.0, 1.0);
                    }
                    world.mark_local_transform_dirty(entity);
                }
            }
            Err(e) => {
                tracing::error!("Failed to load snowman model: {}", e);
            }
        }
    }
}

fn spawn_presents(
    game_world: &GameWorld,
    world: &mut World,
    tree_x: f32,
    tree_z: f32,
    tree_index: usize,
) {
    let base_scale = 0.02;
    let scale_variation = 0.005 + (tree_index as f32 * 0.003) % 0.015;
    let scale = base_scale + scale_variation;

    let offset_angle = (tree_index as f32 * 1.3) % std::f32::consts::TAU;
    let offset_distance = 0.8 + (tree_index as f32 * 0.1) % 0.5;
    let present_x = tree_x + offset_angle.cos() * offset_distance;
    let present_z = tree_z + offset_angle.sin() * offset_distance;
    let present_y = sample_height(present_x, present_z, &game_world.resources.terrain_config) + 0.2;
    let rotation = (tree_index as f32 * 0.7) % std::f32::consts::TAU;

    let load_result = nightshade::ecs::prefab::import_gltf_from_bytes(PRESENTS_MODEL);

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

            for prefab in result.prefabs {
                let entity = nightshade::ecs::prefab::spawn_prefab_with_skins(
                    world,
                    &prefab,
                    &result.animations,
                    &result.skins,
                    Vec3::new(present_x, present_y, present_z),
                );

                if let Some(transform) = world.get_local_transform_mut(entity) {
                    transform.translation = Vec3::new(present_x, present_y, present_z);
                    transform.rotation = nalgebra_glm::quat_angle_axis(rotation, &Vec3::y());
                    transform.scale = Vec3::new(scale, scale, scale);
                }
                world.mark_local_transform_dirty(entity);
            }
        }
        Err(e) => {
            tracing::error!("Failed to load presents model: {}", e);
        }
    }
}
