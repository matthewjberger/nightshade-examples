use nightshade::prelude::*;

use crate::data::{get_item_definition, get_shop_keeper_name};
use crate::ecs::{ShopMode, TreeState, World as GameWorld};
use crate::types::{ToolType, format_time};

fn get_short_item_name(name: &str) -> &str {
    if name.contains("Parsnip") {
        "Pars"
    } else if name.contains("Cauliflower") {
        "Caul"
    } else if name.contains("Potato") {
        "Pota"
    } else if name.contains("Tomato") {
        "Toma"
    } else if name.contains("Corn") {
        "Corn"
    } else if name.contains("Pumpkin") {
        "Pump"
    } else if name.len() > 4 {
        &name[..4]
    } else {
        name
    }
}

pub fn draw_main_menu(ui: &mut ImmediateUi) -> bool {
    let screen_size = ui.screen_size;

    ui.draw_rect(
        Vec2::new(0.0, 0.0),
        screen_size,
        Vec4::new(0.0, 0.0, 0.0, 0.7),
    );

    let panel_width = 400.0;
    let panel_x = (screen_size.x - panel_width) / 2.0;
    let panel_y = (screen_size.y - 400.0) / 2.0;
    ui.begin_vertical(Vec2::new(panel_x, panel_y.max(60.0)), panel_width);
    ui.set_alignment(LayoutAlignment::Center);

    ui.label_colored("MEADOW FIELDS", Vec4::new(0.4, 0.8, 0.4, 1.0));
    ui.spacing(10.0);
    ui.label_colored("A Farming Simulation Game", Vec4::new(0.5, 0.5, 0.5, 1.0));
    ui.spacing(40.0);

    let clicked = ui
        .button_with_color("START GAME", Vec4::new(0.24, 0.47, 0.31, 1.0))
        .clicked;

    ui.spacing(30.0);
    ui.set_alignment(LayoutAlignment::Start);
    ui.label("Controls:");
    ui.label_colored("WASD / Arrow Keys - Move", Vec4::new(0.5, 0.5, 0.5, 1.0));
    ui.label_colored("1-6 - Select Tool", Vec4::new(0.5, 0.5, 0.5, 1.0));
    ui.label_colored("7-0 - Select Seeds", Vec4::new(0.5, 0.5, 0.5, 1.0));
    ui.label_colored(
        "Left Click - Use Tool / Plant",
        Vec4::new(0.5, 0.5, 0.5, 1.0),
    );
    ui.label_colored("E - Interact / Talk", Vec4::new(0.5, 0.5, 0.5, 1.0));
    ui.label_colored("Tab - Toggle Camera", Vec4::new(0.5, 0.5, 0.5, 1.0));
    ui.label_colored("ESC - Pause", Vec4::new(0.5, 0.5, 0.5, 1.0));

    ui.spacing(15.0);
    ui.set_alignment(LayoutAlignment::Center);
    ui.label_colored(
        "Press Enter or click START to begin",
        Vec4::new(0.59, 0.59, 0.59, 1.0),
    );

    ui.end_vertical();

    clicked
}

pub fn draw_pause_menu(ui: &mut ImmediateUi) -> bool {
    let screen_size = ui.screen_size;

    ui.draw_rect(
        Vec2::new(0.0, 0.0),
        screen_size,
        Vec4::new(0.0, 0.0, 0.0, 0.78),
    );

    let panel_width = 300.0;
    let panel_x = (screen_size.x - panel_width) / 2.0;
    let panel_y = (screen_size.y - 200.0) / 2.0;
    ui.begin_vertical(Vec2::new(panel_x, panel_y.max(100.0)), panel_width);
    ui.set_alignment(LayoutAlignment::Center);

    ui.label_colored("PAUSED", Vec4::new(1.0, 1.0, 1.0, 1.0));
    ui.spacing(40.0);

    let clicked = ui
        .button_with_color("Resume", Vec4::new(0.24, 0.39, 0.24, 1.0))
        .clicked;

    ui.spacing(20.0);
    ui.label_colored("Press ESC to resume", Vec4::new(0.5, 0.5, 0.5, 1.0));

    ui.end_vertical();

    clicked
}

fn draw_hotbar(ui: &mut ImmediateUi, game: &GameWorld) {
    let screen_size = ui.screen_size;
    let slot_size = 50.0;
    let slot_spacing = 5.0;
    let hotbar_slots = 10;
    let hotbar_width = hotbar_slots as f32 * (slot_size + slot_spacing) - slot_spacing;
    let hotbar_x = (screen_size.x - hotbar_width) / 2.0;
    let hotbar_y = screen_size.y - slot_size - 20.0;

    ui.draw_rect(
        Vec2::new(hotbar_x - 10.0, hotbar_y - 10.0),
        Vec2::new(hotbar_width + 20.0, slot_size + 20.0),
        Vec4::new(0.0, 0.0, 0.0, 0.5),
    );

    let tools = [
        Some(ToolType::Hoe),
        Some(ToolType::WateringCan),
        Some(ToolType::Axe),
        Some(ToolType::Pickaxe),
        Some(ToolType::Scythe),
        Some(ToolType::Sword),
        None,
        None,
        None,
        None,
    ];

    for (index, tool) in tools.iter().enumerate().take(hotbar_slots) {
        let slot_x = hotbar_x + index as f32 * (slot_size + slot_spacing);

        let is_selected = game.resources.inventory.selected_slot == index;
        let bg_color = if is_selected {
            Vec4::new(0.3, 0.5, 0.8, 0.8)
        } else {
            Vec4::new(0.2, 0.2, 0.2, 0.8)
        };

        ui.draw_rect(
            Vec2::new(slot_x, hotbar_y),
            Vec2::new(slot_size, slot_size),
            bg_color,
        );

        if is_selected {
            ui.draw_rect(
                Vec2::new(slot_x - 2.0, hotbar_y - 2.0),
                Vec2::new(slot_size + 4.0, slot_size + 4.0),
                Vec4::new(1.0, 1.0, 1.0, 0.5),
            );
        }

        let key_label = if index < 9 {
            format!("{}", index + 1)
        } else {
            "0".to_string()
        };

        ui.begin_vertical(Vec2::new(slot_x + 3.0, hotbar_y + 3.0), slot_size - 6.0);
        ui.label_colored(&key_label, Vec4::new(0.6, 0.6, 0.6, 1.0));

        if let Some(tool) = tool {
            let tool_name = match tool {
                ToolType::Hand => "",
                ToolType::Hoe => "Hoe",
                ToolType::WateringCan => "Can",
                ToolType::Axe => "Axe",
                ToolType::Pickaxe => "Pick",
                ToolType::Scythe => "Syth",
                ToolType::Sword => "Swrd",
            };
            ui.label_colored(tool_name, Vec4::new(0.9, 0.9, 0.9, 1.0));
        } else {
            let slot = &game.resources.inventory.hotbar[index];
            if let Some(item_id) = slot.item_id
                && let Some(definition) = get_item_definition(item_id)
            {
                let short_name = get_short_item_name(definition.name);
                ui.label_colored(short_name, Vec4::new(0.7, 0.9, 0.7, 1.0));
                if slot.quantity > 1 {
                    let qty_str = format!("x{}", slot.quantity);
                    ui.label_colored(&qty_str, Vec4::new(0.8, 0.8, 0.8, 1.0));
                }
            }
        }
        ui.end_vertical();
    }
}

fn draw_hud(ui: &mut ImmediateUi, game: &GameWorld) {
    let hud_x = 10.0;
    let hud_y = 10.0;
    let hud_width = 280.0;

    ui.draw_rect(
        Vec2::new(hud_x, hud_y),
        Vec2::new(hud_width, 180.0),
        Vec4::new(0.0, 0.0, 0.0, 0.6),
    );
    ui.begin_vertical(Vec2::new(hud_x + 10.0, hud_y + 10.0), hud_width - 20.0);

    let time_str = format_time(game.resources.hour);
    ui.label(&format!(
        "Day {} - {} - {}",
        game.resources.day,
        game.resources.season.name(),
        time_str
    ));
    ui.label_colored(
        &format!("Weather: {}", game.resources.weather.name()),
        Vec4::new(0.7, 0.8, 0.9, 1.0),
    );
    ui.spacing(5.0);

    let (stamina, max_stamina, equipped_tool) = game
        .resources
        .player_entity
        .and_then(|entity| game.get_player(entity))
        .map(|player| (player.stamina, player.max_stamina, player.equipped_tool))
        .unwrap_or((100.0, 100.0, ToolType::Hand));

    let stamina_pct = stamina / max_stamina;
    let stamina_color = if stamina_pct > 0.5 {
        Vec4::new(0.2, 0.6, 0.9, 1.0)
    } else if stamina_pct > 0.25 {
        Vec4::new(0.9, 0.7, 0.2, 1.0)
    } else {
        Vec4::new(0.9, 0.3, 0.2, 1.0)
    };
    ui.begin_horizontal_at_cursor();
    ui.label("Stamina:");
    ui.progress_bar_colored(stamina_pct, 150.0, stamina_color);
    ui.end_horizontal();
    ui.spacing(5.0);

    ui.label_colored(
        &format!("Gold: {} G", game.resources.money),
        Vec4::new(1.0, 0.84, 0.0, 1.0),
    );
    ui.spacing(5.0);
    ui.label(&format!("Tool: {}", equipped_tool.name()));
    ui.end_vertical();

    draw_hotbar(ui, game);
}

fn draw_tree_health_bar(ui: &mut ImmediateUi, game: &GameWorld) {
    let Some(tree_entity) = game.resources.targeted_tree else {
        return;
    };

    let Some(tree) = game.get_tree(tree_entity) else {
        return;
    };

    if tree.state != TreeState::Standing {
        return;
    }

    let health_pct = tree.health / tree.max_health;

    let health_color = if health_pct > 0.66 {
        Vec4::new(0.2, 0.8, 0.2, 1.0)
    } else if health_pct > 0.33 {
        Vec4::new(0.9, 0.7, 0.1, 1.0)
    } else {
        Vec4::new(0.9, 0.2, 0.1, 1.0)
    };

    let bar_width = 200.0;
    let bar_height = 20.0;
    let screen_size = ui.screen_size;
    let bar_x = (screen_size.x - bar_width) / 2.0;
    let bar_y = 60.0;

    ui.draw_rect(
        Vec2::new(bar_x - 4.0, bar_y - 4.0),
        Vec2::new(bar_width + 8.0, bar_height + 8.0),
        Vec4::new(0.0, 0.0, 0.0, 0.9),
    );

    ui.draw_rect(
        Vec2::new(bar_x - 2.0, bar_y - 2.0),
        Vec2::new(bar_width + 4.0, bar_height + 4.0),
        Vec4::new(0.3, 0.2, 0.1, 1.0),
    );

    ui.draw_rect(
        Vec2::new(bar_x, bar_y),
        Vec2::new(bar_width * health_pct, bar_height),
        health_color,
    );

    ui.begin_vertical(Vec2::new(bar_x, bar_y + bar_height + 5.0), bar_width);
    ui.set_alignment(LayoutAlignment::Center);
    ui.label_colored("Tree", Vec4::new(0.8, 0.6, 0.4, 1.0));
    ui.end_vertical();
}

fn draw_shop(ui: &mut ImmediateUi, game: &GameWorld) {
    let Some(shop) = &game.resources.shop else {
        return;
    };

    let screen_size = ui.screen_size;
    let panel_width = 400.0;
    let panel_height = 350.0;
    let panel_x = (screen_size.x - panel_width) / 2.0;
    let panel_y = (screen_size.y - panel_height) / 2.0;

    ui.draw_rect(
        Vec2::new(panel_x, panel_y),
        Vec2::new(panel_width, panel_height),
        Vec4::new(0.1, 0.1, 0.1, 0.95),
    );

    ui.draw_rect(
        Vec2::new(panel_x, panel_y),
        Vec2::new(panel_width, 40.0),
        Vec4::new(0.2, 0.4, 0.3, 1.0),
    );

    let shop_title = format!("{}'s Shop", get_shop_keeper_name());
    ui.begin_vertical(
        Vec2::new(panel_x + 15.0, panel_y + 10.0),
        panel_width - 30.0,
    );
    ui.label_colored(&shop_title, Vec4::new(1.0, 1.0, 1.0, 1.0));
    ui.end_vertical();

    let tab_y = panel_y + 50.0;
    let tab_width = (panel_width - 20.0) / 2.0;

    let buy_selected = shop.mode == ShopMode::Buy;
    let buy_color = if buy_selected {
        Vec4::new(0.3, 0.5, 0.3, 1.0)
    } else {
        Vec4::new(0.2, 0.2, 0.2, 1.0)
    };
    let sell_color = if !buy_selected {
        Vec4::new(0.3, 0.5, 0.3, 1.0)
    } else {
        Vec4::new(0.2, 0.2, 0.2, 1.0)
    };

    ui.draw_rect(
        Vec2::new(panel_x + 10.0, tab_y),
        Vec2::new(tab_width, 30.0),
        buy_color,
    );
    ui.draw_rect(
        Vec2::new(panel_x + 10.0 + tab_width, tab_y),
        Vec2::new(tab_width, 30.0),
        sell_color,
    );

    let buy_text_color = if buy_selected {
        Vec4::new(1.0, 1.0, 1.0, 1.0)
    } else {
        Vec4::new(0.6, 0.6, 0.6, 1.0)
    };
    let sell_text_color = if !buy_selected {
        Vec4::new(1.0, 1.0, 1.0, 1.0)
    } else {
        Vec4::new(0.6, 0.6, 0.6, 1.0)
    };

    ui.begin_vertical(Vec2::new(panel_x + 10.0, tab_y + 5.0), tab_width);
    ui.set_alignment(LayoutAlignment::Center);
    ui.label_colored("BUY", buy_text_color);
    ui.end_vertical();

    ui.begin_vertical(
        Vec2::new(panel_x + 10.0 + tab_width, tab_y + 5.0),
        tab_width,
    );
    ui.set_alignment(LayoutAlignment::Center);
    ui.label_colored("SELL", sell_text_color);
    ui.end_vertical();

    let list_y = tab_y + 40.0;
    let list_height = panel_height - 140.0;

    ui.draw_rect(
        Vec2::new(panel_x + 10.0, list_y),
        Vec2::new(panel_width - 20.0, list_height),
        Vec4::new(0.15, 0.15, 0.15, 1.0),
    );

    let item_height = 35.0;
    for (index, shop_item) in game.resources.shop_items.iter().enumerate() {
        let item_y = list_y + 5.0 + index as f32 * item_height;

        if index == shop.selected {
            ui.draw_rect(
                Vec2::new(panel_x + 12.0, item_y),
                Vec2::new(panel_width - 24.0, item_height - 2.0),
                Vec4::new(0.3, 0.4, 0.3, 1.0),
            );
        }

        let name = get_item_definition(shop_item.item_id)
            .map(|definition| definition.name)
            .unwrap_or("Unknown");

        let price = if shop.mode == ShopMode::Buy {
            shop_item.buy_price
        } else {
            shop_item.sell_price
        };

        let owned = game.resources.inventory.count_item(shop_item.item_id);

        ui.begin_vertical(Vec2::new(panel_x + 20.0, item_y + 8.0), panel_width - 40.0);
        ui.begin_horizontal_at_cursor();
        ui.label(name);
        ui.end_horizontal();
        ui.end_vertical();

        let price_str = format!("{} G", price);
        let owned_str = format!("x{}", owned);

        ui.begin_vertical(Vec2::new(panel_x + panel_width - 130.0, item_y + 8.0), 40.0);
        ui.set_alignment(LayoutAlignment::End);
        ui.label_colored(&owned_str, Vec4::new(0.7, 0.7, 0.7, 1.0));
        ui.end_vertical();

        ui.begin_vertical(Vec2::new(panel_x + panel_width - 80.0, item_y + 8.0), 60.0);
        ui.set_alignment(LayoutAlignment::End);
        ui.label_colored(&price_str, Vec4::new(1.0, 0.9, 0.4, 1.0));
        ui.end_vertical();
    }

    let footer_y = panel_y + panel_height - 50.0;
    ui.draw_rect(
        Vec2::new(panel_x, footer_y),
        Vec2::new(panel_width, 50.0),
        Vec4::new(0.15, 0.15, 0.15, 1.0),
    );

    let gold_str = format!("Your Gold: {} G", game.resources.money);
    let action_str = if shop.mode == ShopMode::Buy {
        "[E] Buy"
    } else {
        "[E] Sell"
    };
    let controls_str = format!("[W/S] Select  {}  [Q] Switch Tab  [ESC] Close", action_str);

    ui.begin_vertical(
        Vec2::new(panel_x + 15.0, footer_y + 10.0),
        panel_width - 30.0,
    );
    ui.label_colored(&gold_str, Vec4::new(1.0, 0.84, 0.0, 1.0));
    ui.spacing(5.0);
    ui.label_colored(&controls_str, Vec4::new(0.6, 0.6, 0.6, 1.0));
    ui.end_vertical();
}

pub fn draw_playing(game: &GameWorld, ui: &mut ImmediateUi) {
    draw_hud(ui, game);
    draw_tree_health_bar(ui, game);

    if game.resources.shop.is_some() {
        draw_shop(ui, game);
    } else if game.resources.dialogue.is_some() {
        crate::systems::social::draw_dialogue(ui, game);
    } else if crate::systems::social::should_show_shop_hint(game) {
        crate::systems::social::draw_shop_hint(ui);
    } else {
        crate::systems::social::draw_npc_hint(ui, game);
    }
}
