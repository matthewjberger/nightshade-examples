use crate::constants::INTERACT_RANGE;
use crate::ecs::{
    BUTTON, Button, DOOR, ENGINE_ENTITY, EngineEntity, GameWorld, INTERACTABLE, Interactable,
    InteractionKind, LEVER, LeverAction, NOTE, Note, OVERHEAD_LIGHT, OverheadLight,
};
use crate::systems::levers::init_lever;
use nightshade::ecs::scene::{MetadataValue, Scene};
use nightshade::ecs::world::commands::find_entity_by_name;
use nightshade::prelude::*;

pub fn discover_doors(game_world: &mut GameWorld, world: &mut World, scene: &Scene) {
    for scene_entity in &scene.entities {
        let Some(name) = &scene_entity.name else {
            continue;
        };
        if !name.starts_with("Door_") {
            continue;
        }

        let tags = &scene_entity.components.tags;
        let locked = tags.iter().any(|tag| tag == "locked") || name.contains("Exit");
        let side_door = tags.iter().any(|tag| tag == "side_door")
            || name.contains("Storage")
            || name.contains("Generator");
        let swing_reversed =
            tags.iter().any(|tag| tag == "swing_reversed") || name.contains("Generator");

        let Some(entity) = find_entity_by_name(world, name) else {
            continue;
        };

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
        if let Some(engine_entity) = game_world.get_engine_entity(game_entity) {
            let entity = engine_entity.0;
            scene.entities.iter().any(|scene_entity| {
                scene_entity.name.as_deref() == world.core.get_name(entity).map(|n| n.0.as_str())
                    && (scene_entity
                        .components
                        .tags
                        .iter()
                        .any(|tag| tag == "exit_door")
                        || scene_entity
                            .name
                            .as_ref()
                            .is_some_and(|n| n.contains("Exit")))
            })
        } else {
            game_world
                .get_door(game_entity)
                .is_some_and(|door| door.locked)
        }
    });
    if let Some(entity) = exit_door_entity {
        game_world.add_exit_door(entity);
        game_world.resources.exit_door = Some(entity);
    }
}

pub fn discover_levers(game_world: &mut GameWorld, world: &mut World, scene: &Scene) {
    for scene_entity in &scene.entities {
        let Some(name) = &scene_entity.name else {
            continue;
        };
        if !name.starts_with("Lever_") || !name.ends_with("_Pivot") {
            continue;
        }

        let lever_name = name.trim_end_matches("_Pivot");

        let action = if lever_name.contains("UnlockExit") {
            LeverAction::UnlockExit
        } else {
            LeverAction::RestorePower
        };

        let pivot_entity = find_entity_by_name(world, name)
            .unwrap_or_else(|| panic!("Lever pivot '{}' not found", name));

        let position = world
            .core
            .get_local_transform(pivot_entity)
            .map(|transform| transform.translation)
            .unwrap_or(Vec3::zeros());

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | LEVER | INTERACTABLE, 1)[0];

        init_lever(game_world, world, game_entity, lever_name, position, action);
    }
}

pub fn discover_notes(game_world: &mut GameWorld, world: &mut World, scene: &Scene) {
    for scene_entity in &scene.entities {
        let Some(name) = &scene_entity.name else {
            continue;
        };

        let has_note_tag = scene_entity.components.tags.iter().any(|tag| tag == "note");
        if !has_note_tag {
            continue;
        }

        let Some(entity) = find_entity_by_name(world, name) else {
            continue;
        };

        let title = match scene_entity.components.metadata.get("title") {
            Some(MetadataValue::String(value)) => value.clone(),
            _ => name.clone(),
        };

        let content = match scene_entity.components.metadata.get("content") {
            Some(MetadataValue::String(value)) => value.clone(),
            _ => String::new(),
        };

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
        game_world.set_note(game_entity, Note { title, content });
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

pub fn discover_buttons(game_world: &mut GameWorld, world: &mut World, scene: &Scene) {
    for scene_entity in &scene.entities {
        let Some(name) = &scene_entity.name else {
            continue;
        };
        if !name.starts_with("Button_") {
            continue;
        }

        let Some(entity) = find_entity_by_name(world, name) else {
            continue;
        };

        let position = world
            .core
            .get_local_transform(entity)
            .map(|transform| transform.translation)
            .unwrap_or(Vec3::zeros());

        let game_entity = game_world.spawn_entities(ENGINE_ENTITY | BUTTON | INTERACTABLE, 1)[0];
        game_world.set_engine_entity(game_entity, EngineEntity(entity));
        game_world.set_button(
            game_entity,
            Button {
                base_position: position,
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
