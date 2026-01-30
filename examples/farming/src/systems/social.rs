use nightshade::prelude::*;

use crate::data::{get_npc_definition, get_shop_keeper_position};
use crate::ecs::{DialogueState, GameEntity, ShopMode, ShopState, World as GameWorld};
use crate::systems::player::get_player_position;
use crate::types::INTERACTION_RANGE;

fn get_nearest_npc(game: &GameWorld) -> Option<GameEntity> {
    let player_pos = get_player_position(game);

    let mut nearest_entity = None;
    let mut nearest_dist = INTERACTION_RANGE;

    for &npc_entity in &game.resources.npcs {
        let Some(npc_pos) = game.get_position(npc_entity) else {
            continue;
        };

        let dist = nalgebra_glm::distance(&player_pos, &npc_pos.0);
        if dist < nearest_dist {
            nearest_dist = dist;
            nearest_entity = Some(npc_entity);
        }
    }

    nearest_entity
}

fn is_near_shop(game: &GameWorld) -> bool {
    let Some(shop_position) = get_shop_keeper_position() else {
        return false;
    };

    let player_pos = get_player_position(game);
    nalgebra_glm::distance(&player_pos, &shop_position) < 2.5
}

pub fn try_interact(game: &mut GameWorld) -> bool {
    if is_near_shop(game) && game.resources.shop.is_none() {
        game.resources.shop = Some(ShopState::default());
        return true;
    }

    let Some(npc_entity) = get_nearest_npc(game) else {
        return false;
    };

    if game.resources.dialogue.is_some() {
        return false;
    }

    game.resources.dialogue = Some(DialogueState {
        npc: npc_entity,
        line_index: 0,
    });

    if let Some(npc) = game.get_npc_mut(npc_entity)
        && !npc.talked_today
    {
        npc.talked_today = true;
        npc.friendship += 10;
    }

    true
}

pub fn advance_dialogue(game: &mut GameWorld) {
    let Some(dialogue) = &game.resources.dialogue else {
        return;
    };

    let npc_entity = dialogue.npc;
    let line_index = dialogue.line_index;

    let Some(npc) = game.get_npc(npc_entity) else {
        game.resources.dialogue = None;
        return;
    };

    let Some(definition) = get_npc_definition(npc.npc_type) else {
        game.resources.dialogue = None;
        return;
    };

    if line_index + 1 >= definition.dialogue.len() {
        game.resources.dialogue = None;
    } else {
        game.resources.dialogue = Some(DialogueState {
            npc: npc_entity,
            line_index: line_index + 1,
        });
    }
}

pub fn reset_daily_flags(game: &mut GameWorld) {
    let npc_entities: Vec<GameEntity> = game.resources.npcs.clone();
    for npc_entity in npc_entities {
        if let Some(npc) = game.get_npc_mut(npc_entity) {
            npc.talked_today = false;
        }
    }
}

pub fn try_shop_transaction(game: &mut GameWorld) {
    let Some(shop) = &game.resources.shop else {
        return;
    };

    let selected = shop.selected;
    let mode = shop.mode;

    if selected >= game.resources.shop_items.len() {
        return;
    }

    let item = &game.resources.shop_items[selected];
    let item_id = item.item_id;
    let buy_price = item.buy_price;
    let sell_price = item.sell_price;

    match mode {
        ShopMode::Buy => {
            if game.resources.money >= buy_price {
                game.resources.money -= buy_price;
                game.resources.inventory.add_item(item_id, 1);
            }
        }
        ShopMode::Sell => {
            if game.resources.inventory.count_item(item_id) > 0 {
                game.resources.inventory.remove_item(item_id, 1);
                game.resources.money += sell_price;
            }
        }
    }
}

pub fn draw_dialogue(ui: &mut ImmediateUi, game: &GameWorld) {
    let Some(dialogue) = &game.resources.dialogue else {
        return;
    };

    let Some(npc) = game.get_npc(dialogue.npc) else {
        return;
    };

    let Some(definition) = get_npc_definition(npc.npc_type) else {
        return;
    };

    let screen_size = ui.screen_size;
    let box_height = 120.0;
    let box_width = screen_size.x * 0.8;
    let box_x = (screen_size.x - box_width) / 2.0;
    let box_y = screen_size.y - box_height - 80.0;

    ui.draw_rect(
        Vec2::new(box_x, box_y),
        Vec2::new(box_width, box_height),
        Vec4::new(0.0, 0.0, 0.0, 0.85),
    );

    ui.draw_rect(
        Vec2::new(box_x, box_y),
        Vec2::new(box_width, 30.0),
        Vec4::new(
            definition.color[0] * 0.5,
            definition.color[1] * 0.5,
            definition.color[2] * 0.5,
            0.9,
        ),
    );

    ui.begin_vertical(Vec2::new(box_x + 15.0, box_y + 5.0), box_width - 30.0);
    ui.label_colored(
        definition.name,
        Vec4::new(
            definition.color[0],
            definition.color[1],
            definition.color[2],
            1.0,
        ),
    );
    ui.end_vertical();

    if dialogue.line_index < definition.dialogue.len() {
        let dialogue_text = definition.dialogue[dialogue.line_index];
        ui.begin_vertical(Vec2::new(box_x + 15.0, box_y + 40.0), box_width - 30.0);
        ui.label(dialogue_text);
        ui.spacing(10.0);
        ui.label_colored("[Press E to continue]", Vec4::new(0.6, 0.6, 0.6, 1.0));
        ui.end_vertical();
    }
}

pub fn draw_shop_hint(ui: &mut ImmediateUi) {
    let screen_size = ui.screen_size;
    let hint_width = 200.0;
    let hint_x = (screen_size.x - hint_width) / 2.0;
    let hint_y = screen_size.y - 200.0;

    ui.draw_rect(
        Vec2::new(hint_x - 10.0, hint_y - 5.0),
        Vec2::new(hint_width + 20.0, 30.0),
        Vec4::new(0.0, 0.0, 0.0, 0.7),
    );

    ui.begin_vertical(Vec2::new(hint_x, hint_y), hint_width);
    ui.set_alignment(LayoutAlignment::Center);
    ui.label_colored("[E] Open Shop", Vec4::new(1.0, 1.0, 0.8, 1.0));
    ui.end_vertical();
}

pub fn draw_npc_hint(ui: &mut ImmediateUi, game: &GameWorld) {
    if game.resources.dialogue.is_some() {
        return;
    }

    let Some(npc_entity) = get_nearest_npc(game) else {
        return;
    };

    let Some(npc) = game.get_npc(npc_entity) else {
        return;
    };

    let Some(definition) = get_npc_definition(npc.npc_type) else {
        return;
    };

    let screen_size = ui.screen_size;
    let hint_text = format!("[E] Talk to {}", definition.name);

    let hint_width = 200.0;
    let hint_x = (screen_size.x - hint_width) / 2.0;
    let hint_y = screen_size.y - 200.0;

    ui.draw_rect(
        Vec2::new(hint_x - 10.0, hint_y - 5.0),
        Vec2::new(hint_width + 20.0, 30.0),
        Vec4::new(0.0, 0.0, 0.0, 0.7),
    );

    ui.begin_vertical(Vec2::new(hint_x, hint_y), hint_width);
    ui.set_alignment(LayoutAlignment::Center);
    ui.label_colored(&hint_text, Vec4::new(1.0, 1.0, 0.8, 1.0));
    ui.end_vertical();
}

pub fn should_show_shop_hint(game: &GameWorld) -> bool {
    is_near_shop(game) && game.resources.shop.is_none()
}
