use nightshade::ecs::text::components::TextProperties;
use nightshade::prelude::*;

pub struct Popup {
    pub text_entity: Entity,
    pub lifetime: f32,
}

pub fn spawn_popup(world: &mut World, text: &str, position: Vec3, color: Vec4) -> Popup {
    let text_position = position + Vec3::new(0.0, 1.2, 0.0);

    let text_entity = spawn_3d_billboard_text_with_properties(
        world,
        text,
        text_position,
        TextProperties {
            font_size: 18.0,
            color,
            alignment: nightshade::ecs::text::components::TextAlignment::Center,
            outline_width: 0.1,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            smoothing: 0.15,
            ..Default::default()
        },
    );

    Popup {
        text_entity,
        lifetime: 0.0,
    }
}

const POPUP_DURATION: f32 = 1.5;
const POPUP_RISE_SPEED: f32 = 0.5;

pub fn update_popups(popups: &mut Vec<Popup>, world: &mut World, delta_time: f32) {
    let mut indices_to_remove = Vec::new();

    for (index, popup) in popups.iter_mut().enumerate() {
        popup.lifetime += delta_time;

        if popup.lifetime > POPUP_DURATION {
            indices_to_remove.push(index);
            continue;
        }

        if let Some(transform) = world.core.get_local_transform_mut(popup.text_entity) {
            transform.translation.y += delta_time * POPUP_RISE_SPEED;
        }
        world.core.set_local_transform_dirty(popup.text_entity, LocalTransformDirty);

        if let Some(text_component) = world.core.get_text_mut(popup.text_entity) {
            let alpha = (1.0 - (popup.lifetime / POPUP_DURATION)).max(0.0);
            text_component.properties.color.w = alpha;
            text_component.dirty = true;
        }
    }

    for index in indices_to_remove.into_iter().rev() {
        let popup = popups.swap_remove(index);
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive {
                entity: popup.text_entity,
            });
    }
}

pub fn despawn_all_popups(popups: &mut Vec<Popup>, world: &mut World) {
    for popup in popups.drain(..) {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive {
                entity: popup.text_entity,
            });
    }
}
