use crate::constants::INTERACT_RANGE;
use crate::ecs::{
    BUTTON, Button, DOOR, ENGINE_ENTITY, EngineEntity, GameWorld, INTERACTABLE, Interactable,
    InteractionKind, LEVER, LeverAction, NOTE, Note, OVERHEAD_LIGHT, OverheadLight,
};
use crate::systems::levers::init_lever;
use nightshade::ecs::world::commands::find_entity_by_name;
use nightshade::prelude::*;

pub fn discover_doors(game_world: &mut GameWorld, world: &mut World) {
    let door_configs: &[(&str, bool, bool, bool)] = &[
        ("Door_Entry", false, false, false),
        ("Door_Storage", false, true, false),
        ("Door_Generator", false, true, true),
        ("Door_Exit", true, false, false),
    ];

    for &(name, locked, side_door, swing_reversed) in door_configs {
        let entity = find_entity_by_name(world, name)
            .unwrap_or_else(|| panic!("Door entity '{}' not found", name));

        let position = world
            .core
            .get_local_transform(entity)
            .map(|transform| transform.translation)
            .unwrap_or(Vec3::zeros());

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | DOOR | INTERACTABLE, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
        game_world.set_interactable(
            game_entity,
            Interactable {
                kind: InteractionKind::Door,
                match_entity: entity,
                range: INTERACT_RANGE,
            },
        );

        crate::systems::doors::init_door(
            game_world,
            world,
            game_entity,
            &crate::systems::doors::DoorConfig {
                name,
                position,
                locked,
                side_door,
                swing_reversed,
            },
        );
    }

    let exit_door_entity = game_world.query_entities(DOOR).find(|&game_entity| {
        game_world
            .get_door(game_entity)
            .is_some_and(|door| door.locked)
    });
    if let Some(entity) = exit_door_entity {
        game_world.add_exit_door(entity);
        game_world.resources.exit_door = Some(entity);
    }
}

pub fn discover_levers(game_world: &mut GameWorld, world: &mut World) {
    let lever_configs: &[(&str, LeverAction)] = &[
        ("Lever_RestorePower", LeverAction::RestorePower),
        ("Lever_UnlockExit", LeverAction::UnlockExit),
    ];

    for (name, action) in lever_configs {
        let pivot_name = format!("{}_Pivot", name);
        let pivot_entity = find_entity_by_name(world, &pivot_name)
            .unwrap_or_else(|| panic!("Lever pivot '{}' not found", pivot_name));

        let position = world
            .core
            .get_local_transform(pivot_entity)
            .map(|transform| transform.translation)
            .unwrap_or(Vec3::zeros());

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | LEVER | INTERACTABLE, 1)[0];

        init_lever(game_world, world, game_entity, name, position, *action);
    }
}

pub fn discover_notes(game_world: &mut GameWorld, world: &mut World) {
    let note_data: &[(&str, &str, &str)] = &[
        (
            "Note_0",
            "Engineer's Log - Day 1",
            "The power went out again. The generator is in the west wing.\n\n\
             I need to restore power before I can unlock the emergency exit.\n\n\
             The exit controls are in the main hall, but they won't work without power.",
        ),
        (
            "Note_1",
            "Warning",
            "I keep hearing things in the walls...\n\n\
             Something is down here with us.\n\n\
             Don't stay in the dark too long.",
        ),
        (
            "Note_2",
            "Facility Notice",
            "EMERGENCY PROTOCOL:\n\n\
             1. Restore power via generator lever (West Wing)\n\
             2. Return to Main Hall\n\
             3. Pull exit lever to unlock emergency exit (South)\n\n\
             The exit lever requires power to function.",
        ),
        (
            "Note_3",
            "Final Entry",
            "Don't go to the lower levels. Don't follow the sounds.\n\n\
             If you find this note, get out while you still can.\n\n\
             - M. Richter",
        ),
        (
            "Note_4",
            "Generator Instructions",
            "Pull the lever to restore emergency power.\n\n\
             Once power is restored, the exit controls in the main hall will function.\n\n\
             WARNING: Generator may attract unwanted attention.",
        ),
    ];

    for &(name, title, content) in note_data {
        let entity = find_entity_by_name(world, name)
            .unwrap_or_else(|| panic!("Note entity '{}' not found", name));

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | NOTE | INTERACTABLE, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
        game_world.set_interactable(
            game_entity,
            Interactable {
                kind: InteractionKind::Note,
                match_entity: entity,
                range: INTERACT_RANGE,
            },
        );
        game_world.set_note(
            game_entity,
            Note {
                title: title.to_string(),
                content: content.to_string(),
            },
        );
    }
}

pub fn discover_physics_props(game_world: &mut GameWorld, world: &mut World) {
    let mut prop_index = 0;
    loop {
        let name = format!("Prop_{}", prop_index);
        let Some(entity) = find_entity_by_name(world, &name) else {
            break;
        };
        spawn_physics_prop(game_world, entity);
        prop_index += 1;
    }

    let mut link_index = 0;
    loop {
        let name = format!("ChainLink_{}", link_index);
        let Some(entity) = find_entity_by_name(world, &name) else {
            break;
        };
        spawn_physics_prop(game_world, entity);
        link_index += 1;
    }

    if let Some(entity) = find_entity_by_name(world, "Lantern") {
        spawn_physics_prop(game_world, entity);
    }
}

pub fn discover_buttons(game_world: &mut GameWorld, world: &mut World) {
    let button_position = nalgebra_glm::vec3(-7.2, 1.0, -14.0);

    let entity = world.spawn_entities(
        nightshade::prelude::NAME
            | nightshade::prelude::LOCAL_TRANSFORM
            | nightshade::prelude::GLOBAL_TRANSFORM
            | nightshade::prelude::LOCAL_TRANSFORM_DIRTY
            | nightshade::prelude::RENDER_MESH
            | nightshade::prelude::MATERIAL_REF
            | nightshade::prelude::BOUNDING_VOLUME
            | nightshade::prelude::CASTS_SHADOW
            | nightshade::prelude::VISIBILITY
            | nightshade::ecs::world::RIGID_BODY
            | nightshade::ecs::world::COLLIDER,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(entity) {
        name.0 = "Generator Button".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = button_position;
        transform.scale = nalgebra_glm::vec3(0.12, 0.06, 0.12);
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = "Cylinder".to_string();
    }

    let material = nightshade::prelude::Material {
        base_color: [0.8, 0.15, 0.1, 1.0],
        roughness: 0.4,
        metallic: 0.6,
        ..Default::default()
    };
    nightshade::ecs::world::commands::spawn_material(
        world,
        entity,
        "generator_button".to_string(),
        material,
    );

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cylinder");
    }

    if let Some(rigid_body) = world.core.get_rigid_body_mut(entity) {
        *rigid_body = nightshade::ecs::physics::RigidBodyComponent::new_kinematic()
            .with_translation(button_position.x, button_position.y, button_position.z);
    }

    if let Some(collider) = world.core.get_collider_mut(entity) {
        *collider = nightshade::ecs::physics::ColliderComponent::new_cylinder(0.03, 0.06)
            .with_friction(0.5);
    }

    let rigid_body_comp = world.core.get_rigid_body(entity).cloned().unwrap();
    let collider_comp = world.core.get_collider(entity).cloned();
    let rapier_body = rigid_body_comp.to_rapier_rigid_body();
    let rb_handle = world.resources.physics.add_rigid_body(rapier_body);
    if let Some(collider_comp) = collider_comp {
        let rapier_collider = collider_comp.to_rapier_collider();
        world
            .resources
            .physics
            .add_collider(rapier_collider, rb_handle);
    }
    if let Some(rigid_body_mut) = world.core.get_rigid_body_mut(entity) {
        rigid_body_mut.handle = Some(rb_handle.into());
    }
    world
        .resources
        .physics
        .handle_to_entity
        .insert(rb_handle, entity);

    let game_entity = game_world.spawn_entities(ENGINE_ENTITY | BUTTON | INTERACTABLE, 1)[0];
    game_world.set_engine_entity(game_entity, EngineEntity(entity));
    game_world.set_button(
        game_entity,
        Button {
            base_position: button_position,
            current_press: 0.0,
            is_pressed: false,
        },
    );
    game_world.set_interactable(
        game_entity,
        Interactable {
            kind: InteractionKind::Button,
            match_entity: entity,
            range: INTERACT_RANGE,
        },
    );
}

fn spawn_physics_prop(game_world: &mut GameWorld, engine_entity: Entity) {
    let game_entity = game_world.spawn_entities(ENGINE_ENTITY | INTERACTABLE, 1)[0];
    game_world.set_engine_entity(game_entity, EngineEntity(engine_entity));
    game_world.set_interactable(
        game_entity,
        Interactable {
            kind: InteractionKind::Grab,
            match_entity: engine_entity,
            range: 0.0,
        },
    );
    game_world.add_physics_prop(game_entity);
}

pub fn discover_chain_light(game_world: &mut GameWorld, world: &mut World) {
    let lantern_entity = find_entity_by_name(world, "Lantern");
    game_world.resources.lantern_entity = lantern_entity;

    if let Some(light_entity) = find_entity_by_name(world, "LanternLight") {
        game_world.resources.lantern_light_entity = Some(light_entity);
    }

    let mut link_index = 0;
    loop {
        let name = format!("ChainLink_{}", link_index);
        let Some(entity) = find_entity_by_name(world, &name) else {
            break;
        };
        if let Some(rigid_body) = world.core.get_rigid_body(entity)
            && let Some(handle) = rigid_body.handle
            && let Some(rb) = world
                .resources
                .physics
                .rigid_body_set
                .get_mut(handle.into())
        {
            rb.set_linear_damping(0.5);
            rb.set_angular_damping(0.5);
        }
        link_index += 1;
    }

    if let Some(lantern) = lantern_entity
        && let Some(rigid_body) = world.core.get_rigid_body(lantern)
        && let Some(handle) = rigid_body.handle
        && let Some(rb) = world
            .resources
            .physics
            .rigid_body_set
            .get_mut(handle.into())
    {
        rb.set_linear_damping(0.5);
        rb.set_angular_damping(0.5);
    }
}

pub fn discover_overhead_lights(game_world: &mut GameWorld, world: &mut World) {
    for index in 0..9 {
        let fixture_name = format!("LightFixture_{}", index);
        let light_name = format!("OverheadLight_{}", index);

        let Some(fixture_entity) = find_entity_by_name(world, &fixture_name) else {
            continue;
        };
        let Some(light_entity) = find_entity_by_name(world, &light_name) else {
            continue;
        };

        let base_intensity = 1.5 + (index % 3) as f32 * 0.3;

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | OVERHEAD_LIGHT, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(fixture_entity));
        game_world.set_overhead_light(
            game_entity,
            OverheadLight {
                light_entity,
                base_intensity,
                spark_timer: 0.0,
                next_spark_time: 2.0 + (index as f32 * 1.7) % 5.0,
                is_sparking: false,
            },
        );
    }
}
