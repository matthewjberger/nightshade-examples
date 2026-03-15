use crate::ecs::{
    DOOR, ENGINE_ENTITY, EngineEntity, GameWorld, LEVER, LeverAction, NOTE, Note, OVERHEAD_LIGHT,
    OverheadLight,
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

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | DOOR, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));

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

    let exit_door_entity = game_world
        .query_entities(DOOR)
        .find(|&game_entity| {
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

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | LEVER, 1)[0];

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

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | NOTE, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
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
        let game_entity = game_world.spawn_entities(ENGINE_ENTITY, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
        game_world.add_physics_prop(game_entity);
        prop_index += 1;
    }

    let mut link_index = 0;
    loop {
        let name = format!("ChainLink_{}", link_index);
        let Some(entity) = find_entity_by_name(world, &name) else {
            break;
        };
        let game_entity = game_world.spawn_entities(ENGINE_ENTITY, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
        game_world.add_physics_prop(game_entity);
        link_index += 1;
    }

    if let Some(entity) = find_entity_by_name(world, "Lantern") {
        let game_entity = game_world.spawn_entities(ENGINE_ENTITY, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
        game_world.add_physics_prop(game_entity);
    }
}

pub fn discover_chain_light(game_world: &mut GameWorld, world: &mut World) {
    if let Some(lantern_entity) = find_entity_by_name(world, "Lantern") {
        game_world.resources.lantern_entity = Some(lantern_entity);
    }

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

    if let Some(lantern_entity) = find_entity_by_name(world, "Lantern")
        && let Some(rigid_body) = world.core.get_rigid_body(lantern_entity)
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
