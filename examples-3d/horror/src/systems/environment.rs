use crate::constants::{
    CEILING_TEXTURE, DOOR_TEXTURE, FLOOR_TEXTURE, LEVER_TEXTURE, NOTE_TEXTURE, ROOM_HEIGHT,
    WALL_TEXTURE,
};
use crate::state::{HorrorDemo, LeverAction, NoteState};
use crate::systems::levers::init_lever;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub fn load_textures(world: &mut World) {
    let textures = [
        ("horror_floor", FLOOR_TEXTURE),
        ("horror_wall", WALL_TEXTURE),
        ("horror_ceiling", CEILING_TEXTURE),
        ("horror_door", DOOR_TEXTURE),
        ("horror_note", NOTE_TEXTURE),
        ("horror_lever", LEVER_TEXTURE),
    ];

    for (name, data) in textures {
        if let Ok(img) = image::load_from_memory(data) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            world.queue_command(WorldCommand::LoadTexture {
                name: name.to_string(),
                rgba_data: rgba.into_raw(),
                width,
                height,
            });
        }
    }
}

pub fn spawn_physics_props(demo: &mut HorrorDemo, world: &mut World) {
    let crate_material = create_textured_material(nalgebra_glm::vec3(0.45, 0.35, 0.22), 0.9, 0.0);
    let bottle_material = create_textured_material(nalgebra_glm::vec3(0.2, 0.35, 0.2), 0.3, 0.1);
    let sphere_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.55, 0.4), 0.7, 0.2);

    let storage_center_x = 9.0;
    let storage_center_z = -16.0;

    let bauble_colors = [
        nalgebra_glm::vec3(0.7, 0.2, 0.2),
        nalgebra_glm::vec3(0.2, 0.5, 0.7),
        nalgebra_glm::vec3(0.6, 0.5, 0.2),
        nalgebra_glm::vec3(0.3, 0.6, 0.3),
        nalgebra_glm::vec3(0.5, 0.3, 0.6),
        nalgebra_glm::vec3(0.7, 0.6, 0.5),
        nalgebra_glm::vec3(0.4, 0.4, 0.5),
        nalgebra_glm::vec3(0.6, 0.3, 0.4),
    ];

    let mut bauble_index = 0;
    for layer in 0..6 {
        let y = 0.1 + layer as f32 * 0.18;
        let grid_size = 6 - layer;
        let offset = (6 - grid_size) as f32 * 0.15;
        for row in 0..grid_size {
            for col in 0..grid_size {
                let x =
                    storage_center_x - 1.0 + offset + col as f32 * 0.3 + (layer % 2) as f32 * 0.08;
                let z =
                    storage_center_z - 1.0 + offset + row as f32 * 0.3 + (layer % 2) as f32 * 0.08;
                let color = bauble_colors[bauble_index % bauble_colors.len()];
                let bauble_material = create_textured_material(color, 0.3, 0.5);
                let radius = 0.08 + (bauble_index % 3) as f32 * 0.01;
                let entity = spawn_dynamic_physics_sphere_with_material(
                    world,
                    nalgebra_glm::vec3(x, y, z),
                    radius,
                    0.3,
                    bauble_material,
                );
                demo.physics_objects.push(entity);
                bauble_index += 1;
            }
        }
    }

    for layer in 0..4 {
        let y = 0.1 + layer as f32 * 0.18;
        let grid_size = 4 - layer;
        let offset = (4 - grid_size) as f32 * 0.15;
        for row in 0..grid_size {
            for col in 0..grid_size {
                let x = storage_center_x + 0.8 + offset + col as f32 * 0.3;
                let z = storage_center_z + 0.5 + offset + row as f32 * 0.3;
                let color = bauble_colors[bauble_index % bauble_colors.len()];
                let bauble_material = create_textured_material(color, 0.3, 0.5);
                let radius = 0.07 + (bauble_index % 3) as f32 * 0.015;
                let entity = spawn_dynamic_physics_sphere_with_material(
                    world,
                    nalgebra_glm::vec3(x, y, z),
                    radius,
                    0.25,
                    bauble_material,
                );
                demo.physics_objects.push(entity);
                bauble_index += 1;
            }
        }
    }

    for layer in 0..3 {
        let y = 0.1 + layer as f32 * 0.2;
        let count = 5 - layer;
        for index in 0..count {
            let x = storage_center_x - 2.2 + index as f32 * 0.25;
            let z = storage_center_z - 0.5 + (layer % 2) as f32 * 0.1;
            let color = bauble_colors[bauble_index % bauble_colors.len()];
            let bauble_material = create_textured_material(color, 0.3, 0.5);
            let entity = spawn_dynamic_physics_sphere_with_material(
                world,
                nalgebra_glm::vec3(x, y, z),
                0.09,
                0.35,
                bauble_material,
            );
            demo.physics_objects.push(entity);
            bauble_index += 1;
        }
    }

    let main_hall_z = -16.0;

    let entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(0.3, 0.85, main_hall_z + 0.2),
        0.08,
        0.5,
        sphere_material.clone(),
    );
    demo.physics_objects.push(entity);

    let entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(-0.1, 0.85, main_hall_z - 0.1),
        0.06,
        0.3,
        sphere_material.clone(),
    );
    demo.physics_objects.push(entity);

    let entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(-2.8, 0.78, main_hall_z - 2.8),
        0.07,
        0.4,
        sphere_material,
    );
    demo.physics_objects.push(entity);

    let entity = spawn_dynamic_physics_cylinder_with_material(
        world,
        nalgebra_glm::vec3(0.5, 0.93, main_hall_z),
        0.12,
        0.04,
        0.2,
        bottle_material.clone(),
    );
    demo.physics_objects.push(entity);

    let entity = spawn_dynamic_physics_cylinder_with_material(
        world,
        nalgebra_glm::vec3(-3.1, 0.78, main_hall_z - 3.2),
        0.1,
        0.035,
        0.15,
        bottle_material.clone(),
    );
    demo.physics_objects.push(entity);

    let generator_x = -9.0;
    let generator_z = -16.0;

    let entity = spawn_dynamic_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(generator_x + 1.5, 0.2, generator_z - 1.0),
        nalgebra_glm::vec3(0.35, 0.4, 0.35),
        2.0,
        crate_material.clone(),
    );
    demo.physics_objects.push(entity);

    let entity = spawn_dynamic_physics_cylinder_with_material(
        world,
        nalgebra_glm::vec3(generator_x + 2.0, 0.1, generator_z + 1.0),
        0.08,
        0.15,
        1.5,
        create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.4, 0.7),
    );
    demo.physics_objects.push(entity);

    let entity = spawn_dynamic_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(2.0, 0.15, 4.0),
        nalgebra_glm::vec3(0.3, 0.3, 0.3),
        1.5,
        crate_material,
    );
    demo.physics_objects.push(entity);

    let entity = spawn_dynamic_physics_cylinder_with_material(
        world,
        nalgebra_glm::vec3(2.5, 0.95, 5.5),
        0.1,
        0.03,
        0.1,
        bottle_material,
    );
    demo.physics_objects.push(entity);
}

pub fn spawn_interactables(demo: &mut HorrorDemo, world: &mut World) {
    init_lever(
        demo,
        world,
        "Lever_RestorePower",
        nalgebra_glm::vec3(-8.0, 0.6, -14.5),
        LeverAction::RestorePower,
    );
    init_lever(
        demo,
        world,
        "Lever_UnlockExit",
        nalgebra_glm::vec3(3.0, 0.6, -18.0),
        LeverAction::UnlockExit,
    );

    spawn_chain_light(demo, world, nalgebra_glm::vec3(0.0, ROOM_HEIGHT, -5.0));

    spawn_note(
        demo,
        world,
        nalgebra_glm::vec3(0.0, 0.75, 5.0),
        "Engineer's Log - Day 1",
        "The power went out again. The generator is in the west wing.\n\n\
        I need to restore power before I can unlock the emergency exit.\n\n\
        The exit controls are in the main hall, but they won't work without power.",
    );

    spawn_note(
        demo,
        world,
        nalgebra_glm::vec3(0.0, 0.75, -5.0),
        "Warning",
        "I keep hearing things in the walls...\n\n\
        Something is down here with us.\n\n\
        Don't stay in the dark too long.",
    );

    spawn_note(
        demo,
        world,
        nalgebra_glm::vec3(-3.0, 0.75, -14.0),
        "Facility Notice",
        "EMERGENCY PROTOCOL:\n\n\
        1. Restore power via generator lever (West Wing)\n\
        2. Return to Main Hall\n\
        3. Pull exit lever to unlock emergency exit (South)\n\n\
        The exit lever requires power to function.",
    );

    spawn_note(
        demo,
        world,
        nalgebra_glm::vec3(9.0, 0.75, -16.0),
        "Final Entry",
        "Don't go to the lower levels. Don't follow the sounds.\n\n\
        If you find this note, get out while you still can.\n\n\
        - M. Richter",
    );

    spawn_note(
        demo,
        world,
        nalgebra_glm::vec3(-8.0, 0.75, -15.0),
        "Generator Instructions",
        "Pull the lever to restore emergency power.\n\n\
        Once power is restored, the exit controls in the main hall will function.\n\n\
        WARNING: Generator may attract unwanted attention.",
    );
}

pub fn spawn_chain_light(demo: &mut HorrorDemo, world: &mut World, anchor_pos: Vec3) {
    use rapier3d::prelude::*;

    let anchor_height = anchor_pos.y;
    let anchor_position = anchor_pos;

    let beam_material = create_textured_material(nalgebra_glm::vec3(0.25, 0.2, 0.15), 0.9, 0.0);
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(anchor_pos.x, anchor_height + 0.1, anchor_pos.z),
        nalgebra_glm::vec3(0.3, 0.15, 0.3),
        beam_material,
    );

    let anchor_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::RIGID_BODY,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(anchor_entity) {
        name.0 = "Chain Anchor".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(anchor_entity) {
        transform.translation = anchor_position;
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(anchor_entity) {
        *rigid_body = RigidBodyComponent::new_static().with_translation(
            anchor_position.x,
            anchor_position.y,
            anchor_position.z,
        );
    }

    let anchor_handle = {
        let rigid_body_comp = world.core.get_rigid_body(anchor_entity).cloned().unwrap();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(anchor_entity) {
            rigid_body_mut.handle = Some(handle.into());
        }
        handle
    };

    let chain_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.3, 0.32), 0.4, 0.8);

    let num_links = 6;
    let link_length = 0.12;
    let link_radius = 0.015;

    let mut prev_handle: Option<RigidBodyHandle> = Some(anchor_handle);

    for link_index in 0..num_links {
        let link_y = anchor_height - (link_index as f32 + 0.5) * link_length;
        let link_position = nalgebra_glm::vec3(anchor_pos.x, link_y, anchor_pos.z);

        let entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM
                | GLOBAL_TRANSFORM
                | LOCAL_TRANSFORM_DIRTY
                | RENDER_MESH
                | MATERIAL_REF
                | BOUNDING_VOLUME
                | CASTS_SHADOW
                | VISIBILITY
                | nightshade::ecs::world::RIGID_BODY
                | nightshade::ecs::world::COLLIDER
                | nightshade::ecs::world::PHYSICS_INTERPOLATION,
            1,
        )[0];

        if let Some(name) = world.core.get_name_mut(entity) {
            name.0 = format!("Chain Link {}", link_index + 1);
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = link_position;
            transform.scale = nalgebra_glm::vec3(link_radius * 2.0, link_length, link_radius * 2.0);
        }

        if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
            mesh.name = "Cylinder".to_string();
        }

        let material_name = format!("ChainLink_{}", entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            chain_material.clone(),
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
        world.core.set_material_ref(entity, MaterialRef::new(material_name));

        if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
            *bounding_volume =
                nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
        }

        if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
            *rigid_body = RigidBodyComponent::new_dynamic()
                .with_translation(link_position.x, link_position.y, link_position.z)
                .with_mass(0.05);
        }

        if let Some(collider) = world.core.get_collider_mut(entity) {
            *collider =
                ColliderComponent::new_capsule(link_length / 2.0 - link_radius, link_radius)
                    .with_friction(0.3);
        }

        let handle = {
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
            if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(handle) {
                rb.set_linear_damping(0.5);
                rb.set_angular_damping(0.5);
            }
            handle
        };

        if let Some(prev) = prev_handle {
            let local_anchor1 = if link_index == 0 {
                point![0.0, 0.0, 0.0]
            } else {
                point![0.0, -link_length / 2.0, 0.0]
            };
            let joint = SphericalJointBuilder::new()
                .local_anchor1(local_anchor1)
                .local_anchor2(point![0.0, link_length / 2.0, 0.0]);
            world.resources.physics.add_joint(prev, handle, joint);
        }

        prev_handle = Some(handle);
        demo.physics_objects.push(entity);
    }

    let mut lantern_material =
        create_textured_material(nalgebra_glm::vec3(0.8, 0.6, 0.3), 0.3, 0.7);
    lantern_material.emissive_factor = [4.0, 3.2, 1.6];

    let lantern_y = anchor_height - (num_links as f32 * link_length) - 0.12;
    let lantern_position = nalgebra_glm::vec3(anchor_pos.x, lantern_y, anchor_pos.z);

    let lantern_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER
            | nightshade::ecs::world::PHYSICS_INTERPOLATION,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(lantern_entity) {
        name.0 = "Lantern".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(lantern_entity) {
        transform.translation = lantern_position;
        transform.scale = nalgebra_glm::vec3(0.2, 0.3, 0.2);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(lantern_entity) {
        mesh.name = "Cube".to_string();
    }

    let material_name = format!("Lantern_{}", lantern_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        lantern_material,
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
    world.core.set_material_ref(lantern_entity, MaterialRef::new(material_name));

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(lantern_entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(lantern_entity) {
        *rigid_body = RigidBodyComponent::new_dynamic()
            .with_translation(lantern_position.x, lantern_position.y, lantern_position.z)
            .with_mass(0.3);
    }

    if let Some(collider) = world.core.get_collider_mut(lantern_entity) {
        *collider = ColliderComponent::new_cuboid(0.1, 0.15, 0.1).with_friction(0.5);
    }

    let lantern_handle = {
        let rigid_body_comp = world.core.get_rigid_body(lantern_entity).cloned().unwrap();
        let collider_comp = world.core.get_collider(lantern_entity).cloned();
        let rigid_body = rigid_body_comp.to_rapier_rigid_body();
        let handle = world.resources.physics.add_rigid_body(rigid_body);
        if let Some(collider_comp) = collider_comp {
            let collider = collider_comp.to_rapier_collider();
            world.resources.physics.add_collider(collider, handle);
        }
        if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(lantern_entity) {
            rigid_body_mut.handle = Some(handle.into());
        }
        if let Some(rb) = world.resources.physics.rigid_body_set.get_mut(handle) {
            rb.set_linear_damping(0.5);
            rb.set_angular_damping(0.5);
        }
        handle
    };

    if let Some(last_link_handle) = prev_handle {
        let joint = SphericalJointBuilder::new()
            .local_anchor1(point![0.0, -link_length / 2.0, 0.0])
            .local_anchor2(point![0.0, 0.15, 0.0]);
        world
            .resources
            .physics
            .add_joint(last_link_handle, lantern_handle, joint);
    }

    let light_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | LIGHT,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(light_entity) {
        name.0 = "Lantern Light".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(light_entity) {
        transform.translation = lantern_position;
    }

    if let Some(light) = world.core.get_light_mut(light_entity) {
        *light = Light {
            light_type: LightType::Point,
            color: nalgebra_glm::vec3(1.0, 0.85, 0.6),
            intensity: 12.0,
            range: 15.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: true,
            shadow_bias: 0.005,
        };
    }

    demo.lantern_entity = Some(lantern_entity);
    demo.lantern_light_entity = Some(light_entity);
    demo.physics_objects.push(lantern_entity);
}

pub fn spawn_note(
    demo: &mut HorrorDemo,
    world: &mut World,
    position: Vec3,
    title: &str,
    content: &str,
) {
    let table_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.22, 0.15), 0.85, 0.0);

    let table_width = 0.8;
    let table_depth = 0.6;
    let table_thickness = 0.05;
    let leg_size = 0.06;
    let leg_offset_x = (table_width - leg_size) / 2.0 - 0.02;
    let leg_offset_z = (table_depth - leg_size) / 2.0 - 0.02;
    let leg_height = position.y - table_thickness / 2.0;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(position.x, position.y - table_thickness / 2.0, position.z),
        nalgebra_glm::vec3(table_width, table_thickness, table_depth),
        table_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(
            position.x - leg_offset_x,
            leg_height / 2.0,
            position.z - leg_offset_z,
        ),
        nalgebra_glm::vec3(leg_size, leg_height, leg_size),
        table_material.clone(),
    );
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(
            position.x + leg_offset_x,
            leg_height / 2.0,
            position.z - leg_offset_z,
        ),
        nalgebra_glm::vec3(leg_size, leg_height, leg_size),
        table_material.clone(),
    );
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(
            position.x - leg_offset_x,
            leg_height / 2.0,
            position.z + leg_offset_z,
        ),
        nalgebra_glm::vec3(leg_size, leg_height, leg_size),
        table_material.clone(),
    );
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(
            position.x + leg_offset_x,
            leg_height / 2.0,
            position.z + leg_offset_z,
        ),
        nalgebra_glm::vec3(leg_size, leg_height, leg_size),
        table_material,
    );

    let note_material = create_textured_material(nalgebra_glm::vec3(0.85, 0.8, 0.65), 0.95, 0.0);

    let note_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(note_entity) {
        name.0 = format!("Note_{}", demo.notes.len());
    }

    if let Some(transform) = world.core.get_local_transform_mut(note_entity) {
        transform.translation = nalgebra_glm::vec3(position.x, position.y + 0.003, position.z);
        transform.scale = nalgebra_glm::vec3(0.12, 0.002, 0.16);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(note_entity) {
        mesh.name = "Cube".to_string();
    }

    let material_name = format!("Note_{}", note_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        note_material,
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
    world.core.set_material_ref(note_entity, MaterialRef::new(material_name));

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(note_entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(note_entity) {
        *rigid_body = RigidBodyComponent::new_static().with_translation(
            position.x,
            position.y + 0.003,
            position.z,
        );
    }

    if let Some(collider) = world.core.get_collider_mut(note_entity) {
        *collider = ColliderComponent::new_cuboid(0.06, 0.001, 0.08).with_friction(0.5);
    }

    let rigid_body_comp = world.core.get_rigid_body(note_entity).cloned().unwrap();
    let collider_comp = world.core.get_collider(note_entity).cloned();
    let rigid_body = rigid_body_comp.to_rapier_rigid_body();
    let handle = world.resources.physics.add_rigid_body(rigid_body);
    if let Some(collider_comp) = collider_comp {
        let collider = collider_comp.to_rapier_collider();
        world.resources.physics.add_collider(collider, handle);
    }
    if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(note_entity) {
        rigid_body_mut.handle = Some(handle.into());
    }

    demo.notes.push(NoteState {
        entity: note_entity,
        title: title.to_string(),
        content: content.to_string(),
    });
}
