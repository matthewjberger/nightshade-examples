use crate::ecs::{GameState, GameWorld, POSITION, RENDERABLE, TileType};
use nightshade::prelude::*;

const MAP_WIDTH: i32 = 60;
const MAP_HEIGHT: i32 = 30;

#[derive(Default)]
pub struct DisplayState {
    pub map_text_entity: Option<Entity>,
    pub stats_text_entity: Option<Entity>,
    pub messages_text_entity: Option<Entity>,
    pub header_text_entity: Option<Entity>,
    pub title_text_entity: Option<Entity>,
    pub subtitle_text_entity: Option<Entity>,
}

fn get_scale_factor(world: &World) -> f32 {
    let (width, height) = world
        .resources
        .window
        .handle
        .as_ref()
        .map(|h| {
            let size = h.inner_size();
            (size.width as f32, size.height as f32)
        })
        .unwrap_or((800.0, 600.0));

    let char_width = 8.5;
    let line_height = 16.0;

    let needed_width = (MAP_WIDTH as f32 + 4.0) * char_width + 40.0;
    let needed_height = (MAP_HEIGHT as f32 + 10.0) * line_height + 160.0;

    let scale_x = width / needed_width;
    let scale_y = height / needed_height;

    scale_x.min(scale_y).clamp(0.5, 2.0)
}

pub fn spawn_display(world: &mut World) -> DisplayState {
    let scale = get_scale_factor(world);

    let map_props = TextProperties {
        font_size: 14.0 * scale,
        color: Vec4::new(0.0, 1.0, 0.85, 1.0),
        alignment: TextAlignment::Left,
        line_height: 1.0,
        monospace_width: Some(8.5 * scale),
        ..Default::default()
    };

    let map_entity = spawn_hud_text_with_properties(
        world,
        "",
        HudAnchor::TopLeft,
        Vec2::new(20.0 * scale, 70.0 * scale),
        map_props,
    );

    let title_props = TextProperties {
        font_size: 24.0 * scale,
        color: Vec4::new(1.0, 0.0, 0.7, 1.0),
        alignment: TextAlignment::Left,
        ..Default::default()
    };

    let title_entity = spawn_hud_text_with_properties(
        world,
        "CYBERDUST",
        HudAnchor::TopLeft,
        Vec2::new(20.0 * scale, 10.0 * scale),
        title_props,
    );

    let subtitle_props = TextProperties {
        font_size: 12.0 * scale,
        color: Vec4::new(0.5, 0.5, 0.6, 1.0),
        alignment: TextAlignment::Left,
        ..Default::default()
    };

    let subtitle_entity = spawn_hud_text_with_properties(
        world,
        "// NEON RUNNER v2.0",
        HudAnchor::TopLeft,
        Vec2::new(130.0 * scale, 14.0 * scale),
        subtitle_props,
    );

    let header_props = TextProperties {
        font_size: 14.0 * scale,
        color: Vec4::new(0.0, 0.9, 0.7, 1.0),
        alignment: TextAlignment::Left,
        monospace_width: Some(8.5 * scale),
        ..Default::default()
    };

    let header_entity = spawn_hud_text_with_properties(
        world,
        "",
        HudAnchor::TopLeft,
        Vec2::new(20.0 * scale, 40.0 * scale),
        header_props,
    );

    let stats_props = TextProperties {
        font_size: 14.0 * scale,
        color: Vec4::new(0.0, 1.0, 0.8, 1.0),
        alignment: TextAlignment::Left,
        monospace_width: Some(8.5 * scale),
        ..Default::default()
    };

    let stats_entity = spawn_hud_text_with_properties(
        world,
        "",
        HudAnchor::BottomLeft,
        Vec2::new(20.0 * scale, -90.0 * scale),
        stats_props,
    );

    let messages_props = TextProperties {
        font_size: 13.0 * scale,
        color: Vec4::new(0.9, 0.3, 0.9, 1.0),
        alignment: TextAlignment::Left,
        line_height: 1.3,
        ..Default::default()
    };

    let messages_entity = spawn_hud_text_with_properties(
        world,
        "",
        HudAnchor::BottomLeft,
        Vec2::new(20.0 * scale, -20.0 * scale),
        messages_props,
    );

    DisplayState {
        map_text_entity: Some(map_entity),
        stats_text_entity: Some(stats_entity),
        messages_text_entity: Some(messages_entity),
        header_text_entity: Some(header_entity),
        title_text_entity: Some(title_entity),
        subtitle_text_entity: Some(subtitle_entity),
    }
}

pub fn update_display(display: &DisplayState, game_world: &GameWorld, world: &mut World) {
    let scale = get_scale_factor(world);

    update_text_properties(display, world, scale);
    update_header_display(display, game_world, world);
    update_map_display(display, game_world, world);
    update_stats_display(display, game_world, world);
    update_messages_display(display, game_world, world);
}

fn update_text_properties(display: &DisplayState, world: &mut World, scale: f32) {
    if let Some(entity) = display.map_text_entity
        && let Some(hud_text) = world.get_hud_text_mut(entity)
    {
        hud_text.properties.font_size = 14.0 * scale;
        hud_text.properties.monospace_width = Some(8.5 * scale);
        hud_text.position = Vec2::new(20.0 * scale, 70.0 * scale);
    }

    if let Some(entity) = display.title_text_entity
        && let Some(hud_text) = world.get_hud_text_mut(entity)
    {
        hud_text.properties.font_size = 24.0 * scale;
        hud_text.position = Vec2::new(20.0 * scale, 10.0 * scale);
    }

    if let Some(entity) = display.subtitle_text_entity
        && let Some(hud_text) = world.get_hud_text_mut(entity)
    {
        hud_text.properties.font_size = 12.0 * scale;
        hud_text.position = Vec2::new(130.0 * scale, 14.0 * scale);
    }

    if let Some(entity) = display.header_text_entity
        && let Some(hud_text) = world.get_hud_text_mut(entity)
    {
        hud_text.properties.font_size = 14.0 * scale;
        hud_text.properties.monospace_width = Some(8.5 * scale);
        hud_text.position = Vec2::new(20.0 * scale, 40.0 * scale);
    }

    if let Some(entity) = display.stats_text_entity
        && let Some(hud_text) = world.get_hud_text_mut(entity)
    {
        hud_text.properties.font_size = 14.0 * scale;
        hud_text.properties.monospace_width = Some(8.5 * scale);
        hud_text.position = Vec2::new(20.0 * scale, -90.0 * scale);
    }

    if let Some(entity) = display.messages_text_entity
        && let Some(hud_text) = world.get_hud_text_mut(entity)
    {
        hud_text.properties.font_size = 13.0 * scale;
        hud_text.position = Vec2::new(20.0 * scale, -20.0 * scale);
    }
}

fn update_header_display(display: &DisplayState, game_world: &GameWorld, world: &mut World) {
    let Some(entity) = display.header_text_entity else {
        return;
    };

    let depth = game_world.resources.current_depth;
    let kills = game_world.resources.stats.kills;
    let credits = game_world.resources.inventory.credits;
    let enemy_count = game_world.query_entities(crate::ecs::ENEMY).count();

    let header = format!(
        "LAYER {:02}  |  KILLS {:03}  |  CREDS {:05}  |  HOSTILES {:02}",
        depth, kills, credits, enemy_count
    );

    if let Some(hud_text) = world.get_hud_text(entity) {
        let text_index = hud_text.text_index;
        world.resources.text_cache.set_text(text_index, &header);
        if let Some(hud_text) = world.get_hud_text_mut(entity) {
            hud_text.dirty = true;
        }
    }
}

fn update_map_display(display: &DisplayState, game_world: &GameWorld, world: &mut World) {
    let Some(entity) = display.map_text_entity else {
        return;
    };

    let map = &game_world.resources.map;
    let fov = &game_world.resources.fov_map;

    let mut render_grid: Vec<Vec<char>> = vec![vec![' '; map.width as usize]; map.height as usize];

    for y in 0..map.height {
        for x in 0..map.width {
            let visible = fov.is_visible(x, y);
            let explored = fov.is_explored(x, y);

            let tile = map.get_tile(x, y);

            let glyph = if visible {
                match tile {
                    TileType::Wall => '#',
                    TileType::Floor => '.',
                    TileType::DataPort => '>',
                }
            } else if explored {
                match tile {
                    TileType::Wall => '+',
                    TileType::Floor => ' ',
                    TileType::DataPort => '>',
                }
            } else {
                ' '
            };

            render_grid[y as usize][x as usize] = glyph;
        }
    }

    let entities_with_renderables: Vec<_> = game_world
        .query_entities(POSITION | RENDERABLE)
        .filter_map(|entity| {
            let position = game_world.get_position(entity)?;
            let renderable = game_world.get_renderable(entity)?;
            Some((entity, position, renderable))
        })
        .collect();

    for (entity, position, renderable) in entities_with_renderables {
        let x = position.x;
        let y = position.y;

        if !fov.is_visible(x, y) {
            continue;
        }

        if map.in_bounds(x, y) {
            let is_player = game_world.get_player(entity).is_some();
            let is_enemy = game_world.get_enemy(entity).is_some();

            let glyph = if is_player {
                '@'
            } else if is_enemy {
                renderable.glyph.to_ascii_uppercase()
            } else {
                renderable.glyph
            };

            render_grid[y as usize][x as usize] = glyph;
        }
    }

    let border_top = format!("/{}\\", "=".repeat(map.width as usize));
    let border_bottom = format!("\\{}/", "=".repeat(map.width as usize));

    let mut lines: Vec<String> = Vec::with_capacity((map.height + 2) as usize);
    lines.push(border_top);

    for row in render_grid.iter() {
        let row_str: String = row.iter().collect();
        lines.push(format!("|{}|", row_str));
    }
    lines.push(border_bottom);
    let map_string = lines.join("\n");

    if let Some(hud_text) = world.get_hud_text(entity) {
        let text_index = hud_text.text_index;
        world.resources.text_cache.set_text(text_index, &map_string);
        if let Some(hud_text) = world.get_hud_text_mut(entity) {
            hud_text.dirty = true;
        }
    }
}

fn update_stats_display(display: &DisplayState, game_world: &GameWorld, world: &mut World) {
    let Some(entity) = display.stats_text_entity else {
        return;
    };

    let mut stats_lines: Vec<String> = Vec::new();

    if let Some(player_entity) = game_world.resources.player_entity {
        let player_entity = freecs::Entity {
            id: player_entity.id,
            generation: player_entity.generation,
        };

        if let Some(stats) = game_world.get_combat_stats(player_entity) {
            let attack = stats.attack + game_world.resources.inventory.attack_bonus();
            let defense = stats.defense + game_world.resources.inventory.defense_bonus();

            let hp_percent = (stats.hp as f32 / stats.max_hp as f32 * 100.0) as i32;
            let bar_width = 20;
            let filled = (stats.hp as f32 / stats.max_hp as f32 * bar_width as f32) as usize;
            let empty = bar_width - filled;

            let hp_bar = format!(
                "[{}{}] {:3}/{:3}",
                "#".repeat(filled),
                "-".repeat(empty),
                stats.hp,
                stats.max_hp
            );

            let status = match hp_percent {
                0..=25 => "CRITICAL",
                26..=50 => "DAMAGED",
                51..=75 => "STABLE",
                _ => "OPTIMAL",
            };

            stats_lines.push(format!("INTEGRITY {} {}", hp_bar, status));
            stats_lines.push(format!("DMG {:02}  ARMOR {:02}", attack, defense));
        }
    }

    let inventory = &game_world.resources.inventory;
    let mut inv_parts: Vec<String> = Vec::new();

    if let Some(weapon) = &inventory.equipped_weapon {
        inv_parts.push(format!("<{}>", weapon.name()));
    }
    if let Some(armor) = &inventory.equipped_armor {
        inv_parts.push(format!("[{}]", armor.name()));
    }

    let stim_count = inventory
        .items
        .iter()
        .filter(|item| matches!(item, crate::ecs::ItemType::StimPack))
        .count();

    if stim_count > 0 {
        inv_parts.push(format!("STIM:{}x", stim_count));
    }
    if inventory.emp_grenades > 0 {
        inv_parts.push(format!("EMP:{}x", inventory.emp_grenades));
    }

    if !inv_parts.is_empty() {
        stats_lines.push(inv_parts.join("  "));
    }

    let state = game_world.resources.game_state;
    if matches!(state, GameState::PlayerDead) {
        stats_lines.push(">>> FLATLINED - PRESS R TO REBOOT <<<".to_string());
    } else if matches!(state, GameState::Victory) {
        stats_lines.push(">>> NETRUN COMPLETE - PRESS R TO RESTART <<<".to_string());
    }

    let stats_text = stats_lines.join("\n");

    if let Some(hud_text) = world.get_hud_text(entity) {
        let text_index = hud_text.text_index;
        world.resources.text_cache.set_text(text_index, &stats_text);
        if let Some(hud_text) = world.get_hud_text_mut(entity) {
            hud_text.dirty = true;
        }
    }
}

fn update_messages_display(display: &DisplayState, game_world: &GameWorld, world: &mut World) {
    let Some(entity) = display.messages_text_entity else {
        return;
    };

    let messages = &game_world.resources.message_log;
    let display_count = 4.min(messages.len());
    let start = messages.len().saturating_sub(display_count);

    let messages_text: String = messages[start..]
        .iter()
        .map(|msg| format!("> {}", msg))
        .collect::<Vec<_>>()
        .join("\n");

    if let Some(hud_text) = world.get_hud_text(entity) {
        let text_index = hud_text.text_index;
        world
            .resources
            .text_cache
            .set_text(text_index, &messages_text);
        if let Some(hud_text) = world.get_hud_text_mut(entity) {
            hud_text.dirty = true;
        }
    }
}
