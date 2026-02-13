use crate::ecs::{GameWorld, POSITION, RENDERABLE, TileType};
use nightshade::prelude::*;

fn rgb(r: u8, g: u8, b: u8) -> Vec4 {
    Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

const COLOR_VISIBLE_WALL: (u8, u8, u8) = (139, 119, 101);
const COLOR_VISIBLE_FLOOR: (u8, u8, u8) = (50, 50, 80);
const COLOR_VISIBLE_STAIRS: (u8, u8, u8) = (255, 255, 100);
const COLOR_EXPLORED_WALL: (u8, u8, u8) = (50, 45, 40);
const COLOR_EXPLORED_FLOOR: (u8, u8, u8) = (25, 25, 35);
const COLOR_EXPLORED_STAIRS: (u8, u8, u8) = (80, 80, 50);
const COLOR_PLAYER: (u8, u8, u8) = (0, 255, 0);
const COLOR_ENEMY: (u8, u8, u8) = (255, 80, 80);
const COLOR_ITEM: (u8, u8, u8) = (255, 255, 0);
const COLOR_BORDER: (u8, u8, u8) = (100, 100, 100);

#[derive(Default)]
pub struct DisplayState {
    pub map_text_entity: Option<Entity>,
    pub stats_text_entity: Option<Entity>,
    pub messages_text_entity: Option<Entity>,
}

pub fn spawn_display(world: &mut World) -> DisplayState {
    let map_props = TextProperties {
        font_size: 14.0,
        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        alignment: TextAlignment::Left,
        line_height: 1.0,
        monospace_width: Some(8.5),
        ..Default::default()
    };

    let map_entity = spawn_hud_text_with_properties(
        world,
        "",
        HudAnchor::TopLeft,
        Vec2::new(20.0, 40.0),
        map_props,
    );

    world.queue_add_components(map_entity, TEXT_CHARACTER_COLORS);
    world.apply_commands();

    let title_props = TextProperties {
        font_size: 20.0,
        color: Vec4::new(1.0, 0.8, 0.0, 1.0),
        alignment: TextAlignment::Left,
        ..Default::default()
    };

    spawn_hud_text_with_properties(
        world,
        "ROGUELIKE DUNGEON",
        HudAnchor::TopLeft,
        Vec2::new(20.0, 10.0),
        title_props,
    );

    let stats_props = TextProperties {
        font_size: 16.0,
        color: Vec4::new(0.8, 0.8, 0.8, 1.0),
        alignment: TextAlignment::Left,
        ..Default::default()
    };

    let stats_entity = spawn_hud_text_with_properties(
        world,
        "",
        HudAnchor::BottomLeft,
        Vec2::new(20.0, -80.0),
        stats_props,
    );

    let messages_props = TextProperties {
        font_size: 14.0,
        color: Vec4::new(0.7, 0.7, 1.0, 1.0),
        alignment: TextAlignment::Left,
        line_height: 1.2,
        ..Default::default()
    };

    let messages_entity = spawn_hud_text_with_properties(
        world,
        "",
        HudAnchor::BottomLeft,
        Vec2::new(20.0, -20.0),
        messages_props,
    );

    DisplayState {
        map_text_entity: Some(map_entity),
        stats_text_entity: Some(stats_entity),
        messages_text_entity: Some(messages_entity),
    }
}

pub fn update_display(display: &DisplayState, game_world: &GameWorld, world: &mut World) {
    update_map_display(display, game_world, world);
    update_stats_display(display, game_world, world);
    update_messages_display(display, game_world, world);
}

#[derive(Clone, Copy)]
struct ColoredCell {
    glyph: char,
    color: (u8, u8, u8),
}

fn update_map_display(display: &DisplayState, game_world: &GameWorld, world: &mut World) {
    let Some(entity) = display.map_text_entity else {
        return;
    };

    let map = &game_world.resources.map;
    let fov = &game_world.resources.fov_map;

    let mut render_grid: Vec<Vec<ColoredCell>> = vec![
        vec![
            ColoredCell {
                glyph: ' ',
                color: (0, 0, 0)
            };
            map.width as usize
        ];
        map.height as usize
    ];

    for y in 0..map.height {
        for x in 0..map.width {
            let visible = fov.is_visible(x, y);
            let explored = fov.is_explored(x, y);

            let tile = map.get_tile(x, y);

            let (glyph, color) = if visible {
                match tile {
                    TileType::Wall => ('#', COLOR_VISIBLE_WALL),
                    TileType::Floor => ('.', COLOR_VISIBLE_FLOOR),
                    TileType::StairsDown => ('>', COLOR_VISIBLE_STAIRS),
                }
            } else if explored {
                match tile {
                    TileType::Wall => ('#', COLOR_EXPLORED_WALL),
                    TileType::Floor => ('.', COLOR_EXPLORED_FLOOR),
                    TileType::StairsDown => ('>', COLOR_EXPLORED_STAIRS),
                }
            } else {
                (' ', (0, 0, 0))
            };

            render_grid[y as usize][x as usize] = ColoredCell { glyph, color };
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
            let is_item = game_world.get_item(entity).is_some();

            let (glyph, color) = if is_player {
                ('@', COLOR_PLAYER)
            } else if is_enemy {
                (renderable.glyph.to_ascii_uppercase(), COLOR_ENEMY)
            } else if is_item {
                (renderable.glyph, COLOR_ITEM)
            } else {
                (renderable.glyph, COLOR_VISIBLE_FLOOR)
            };

            render_grid[y as usize][x as usize] = ColoredCell { glyph, color };
        }
    }

    let border_color = rgb(COLOR_BORDER.0, COLOR_BORDER.1, COLOR_BORDER.2);

    let mut text = String::new();
    let mut colors: Vec<Option<Vec4>> = Vec::new();

    text.push('+');
    colors.push(Some(border_color));
    for _ in 0..map.width {
        text.push('-');
        colors.push(Some(border_color));
    }
    text.push('+');
    colors.push(Some(border_color));
    text.push('\n');
    colors.push(None);

    for row in render_grid.iter() {
        text.push('|');
        colors.push(Some(border_color));
        for cell in row.iter() {
            text.push(cell.glyph);
            colors.push(Some(rgb(cell.color.0, cell.color.1, cell.color.2)));
        }
        text.push('|');
        colors.push(Some(border_color));
        text.push('\n');
        colors.push(None);
    }

    text.push('+');
    colors.push(Some(border_color));
    for _ in 0..map.width {
        text.push('-');
        colors.push(Some(border_color));
    }
    text.push('+');
    colors.push(Some(border_color));

    if let Some(hud_text) = world.get_hud_text(entity) {
        let text_index = hud_text.text_index;
        world.resources.text_cache.set_text(text_index, &text);
        if let Some(hud_text) = world.get_hud_text_mut(entity) {
            hud_text.dirty = true;
        }
    }

    if let Some(char_colors) = world.get_text_character_colors_mut(entity) {
        char_colors.colors = colors;
        char_colors.dirty = true;
    }
}

fn update_stats_display(display: &DisplayState, game_world: &GameWorld, world: &mut World) {
    let Some(entity) = display.stats_text_entity else {
        return;
    };

    let mut stats_text = String::new();

    if let Some(player_entity) = game_world.resources.player_entity {
        let player_entity = freecs::Entity {
            id: player_entity.id,
            generation: player_entity.generation,
        };

        if let Some(stats) = game_world.get_combat_stats(player_entity) {
            let attack = stats.attack + game_world.resources.inventory.attack_bonus();
            let defense = stats.defense + game_world.resources.inventory.defense_bonus();

            stats_text = format!(
                "HP: {}/{}  ATK: {}  DEF: {}  Depth: {}",
                stats.hp, stats.max_hp, attack, defense, game_world.resources.current_depth
            );
        }
    }

    let inventory = &game_world.resources.inventory;
    if !inventory.items.is_empty()
        || inventory.equipped_weapon.is_some()
        || inventory.equipped_armor.is_some()
    {
        stats_text.push_str("  |  ");

        if let Some(weapon) = &inventory.equipped_weapon {
            stats_text.push_str(&format!("[{}] ", weapon.name()));
        }
        if let Some(armor) = &inventory.equipped_armor {
            stats_text.push_str(&format!("[{}] ", armor.name()));
        }

        let potion_count = inventory
            .items
            .iter()
            .filter(|item| matches!(item, crate::ecs::ItemType::HealthPotion))
            .count();
        if potion_count > 0 {
            stats_text.push_str(&format!("Potions: {}", potion_count));
        }
    }

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
    let display_count = 3.min(messages.len());
    let start = messages.len().saturating_sub(display_count);

    let messages_text: String = messages[start..].to_vec().join("\n");

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
