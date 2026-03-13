use nightshade::prelude::*;

use crate::ecs::{Popup, World as GameWorld};
use crate::systems::player::{get_equipped_tool, get_player_facing, get_player_position};
use crate::types::{PLAYER_RADIUS, ToolType};

struct ToolConfig {
    material: &'static str,
    scale: Vec3,
}

fn get_tool_config(tool: ToolType) -> ToolConfig {
    match tool {
        ToolType::Hand => ToolConfig {
            material: "PlayerBody",
            scale: Vec3::new(0.1, 0.1, 0.1),
        },
        ToolType::Hoe => ToolConfig {
            material: "ToolHoe",
            scale: Vec3::new(0.15, 0.6, 0.15),
        },
        ToolType::WateringCan => ToolConfig {
            material: "ToolWateringCan",
            scale: Vec3::new(0.25, 0.3, 0.15),
        },
        ToolType::Axe => ToolConfig {
            material: "ToolAxe",
            scale: Vec3::new(0.08, 0.5, 0.25),
        },
        ToolType::Pickaxe => ToolConfig {
            material: "ToolPickaxe",
            scale: Vec3::new(0.08, 0.5, 0.2),
        },
        ToolType::Scythe => ToolConfig {
            material: "ToolScythe",
            scale: Vec3::new(0.08, 0.6, 0.3),
        },
        ToolType::Sword => ToolConfig {
            material: "ToolSword",
            scale: Vec3::new(0.06, 0.7, 0.15),
        },
    }
}

pub fn update_tool(game: &GameWorld, world: &mut World) {
    let Some(tool_visual) = game.resources.visuals.tool_visual else {
        return;
    };

    let player_pos = get_player_position(game);
    let facing = get_player_facing(game);
    let equipped = get_equipped_tool(game);

    let config = get_tool_config(equipped);

    let tool_offset_forward = 0.5;
    let tool_offset_right = 0.3;

    let forward_dir = Vec3::new(-facing.z, 0.0, facing.x);
    let tool_position = Vec3::new(
        player_pos.x
            + facing.x.signum().abs() * tool_offset_forward * facing.x.abs().max(0.1)
            + forward_dir.x * tool_offset_right,
        player_pos.y + PLAYER_RADIUS * 0.3,
        player_pos.z
            + facing.z.signum().abs() * tool_offset_forward * facing.z.abs().max(0.1)
            + forward_dir.z * tool_offset_right,
    );

    let facing_angle = facing.x.atan2(facing.z);

    if let Some(transform) = world.core.get_local_transform_mut(tool_visual) {
        transform.translation = tool_position;
        transform.scale = config.scale;
        transform.rotation = nalgebra_glm::quat_angle_axis(facing_angle + 0.3, &Vec3::y());
    }
    mark_local_transform_dirty(world, tool_visual);

    crate::systems::init::apply_material_by_name(world, tool_visual, config.material);
}

pub fn update_popups(game: &mut GameWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let mut to_remove = Vec::new();

    for (index, popup) in game.resources.popups.popups.iter_mut().enumerate() {
        popup.lifetime -= delta;

        if popup.lifetime <= 0.0 {
            if let Some(entity) = popup.entity {
                world.queue_despawn_entity(entity);
            }
            to_remove.push(index);
            continue;
        }

        let entity = match popup.entity {
            Some(e) => e,
            None => {
                let e = create_popup_visual(world, &popup.text, popup.start_position);
                popup.entity = Some(e);
                e
            }
        };

        let progress = 1.0 - (popup.lifetime / 2.0);
        let offset_y = progress * 1.5;
        let alpha = popup.lifetime.min(1.0);

        if let Some(text) = world.core.get_text_mut(entity) {
            text.properties.color.w = alpha;
        }

        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = popup.start_position + Vec3::new(0.0, offset_y, 0.0);
        }
        mark_local_transform_dirty(world, entity);
    }

    for index in to_remove.into_iter().rev() {
        game.resources.popups.popups.remove(index);
    }
}

fn create_popup_visual(world: &mut World, text: &str, position: Vec3) -> Entity {
    let entity = world.spawn_entities(
        LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | TEXT,
        1,
    )[0];

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    mark_local_transform_dirty(world, entity);

    let text_index = world.resources.text_cache.add_text(text);
    let mut text_component = Text::new(text_index);
    text_component.properties.color = Vec4::new(0.8, 0.9, 0.7, 1.0);
    text_component.properties.font_size = 24.0;
    text_component.billboard = true;
    world.core.set_text(entity, text_component);

    entity
}

pub fn spawn_popup(game: &mut GameWorld, world: &mut World, position: Vec3, text: &str) {
    let adjusted_position = position + Vec3::new(0.0, 2.0, 0.0);
    let entity = create_popup_visual(world, text, adjusted_position);

    game.resources.popups.popups.push(Popup {
        entity: Some(entity),
        text: text.to_string(),
        lifetime: 2.0,
        start_position: adjusted_position,
    });
}
