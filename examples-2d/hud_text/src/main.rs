use nightshade::prelude::*;
use std::collections::VecDeque;

const LOG_MAX_ENTRIES: usize = 100;
const LOG_VISIBLE_ENTRIES: usize = 6;
const LOG_FONT_SIZE: f32 = 24.0;
const LOG_LINE_HEIGHT: f32 = 30.0;
const LOG_PADDING: f32 = 10.0;
const LOG_WIDTH: f32 = 400.0;
const LOG_HEIGHT: f32 = LOG_VISIBLE_ENTRIES as f32 * LOG_LINE_HEIGHT + LOG_PADDING * 2.0;

struct LogEntry {
    text: String,
    color: [f32; 4],
}

struct ScrollingLog {
    entries: VecDeque<LogEntry>,
    scroll_offset: usize,
    line_entities: Vec<Entity>,
    last_entry_time: f32,
    entry_counter: u32,
}

fn scrolling_log_new() -> ScrollingLog {
    ScrollingLog {
        entries: VecDeque::new(),
        scroll_offset: 0,
        line_entities: Vec::new(),
        last_entry_time: 0.0,
        entry_counter: 0,
    }
}

fn scrolling_log_add_entry(log: &mut ScrollingLog, text: String, color: [f32; 4]) {
    log.entries.push_back(LogEntry { text, color });
    if log.entries.len() > LOG_MAX_ENTRIES {
        log.entries.pop_front();
        if log.scroll_offset > 0 {
            log.scroll_offset -= 1;
        }
    }
    log.scroll_offset = log.entries.len().saturating_sub(LOG_VISIBLE_ENTRIES);
}

fn scrolling_log_spawn_ui(world: &mut World, log: &mut ScrollingLog) {
    let props = TextProperties {
        font_size: LOG_FONT_SIZE,
        color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 1.0),
        alignment: TextAlignment::Left,
        outline_width: 0.05,
        outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
        ..Default::default()
    };

    for index in 0..LOG_VISIBLE_ENTRIES {
        let y_offset = -(LOG_PADDING
            + (LOG_VISIBLE_ENTRIES - 1 - index) as f32 * LOG_LINE_HEIGHT
            + LOG_LINE_HEIGHT);

        let entity = spawn_hud_text_with_properties(
            world,
            "",
            HudAnchor::BottomLeft,
            nalgebra_glm::vec2(LOG_PADDING, y_offset),
            props.clone(),
        );

        log.line_entities.push(entity);
    }
}

fn scrolling_log_update_ui(world: &mut World, log: &ScrollingLog) {
    let start_index = log.scroll_offset;
    let entries_to_show: Vec<_> = log
        .entries
        .iter()
        .skip(start_index)
        .take(LOG_VISIBLE_ENTRIES)
        .collect();

    for (slot_index, entity) in log.line_entities.iter().enumerate() {
        if let Some(entry) = entries_to_show.get(slot_index) {
            if let Some(text_index) = world.core.get_hud_text(*entity).map(|t| t.text_index) {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, entry.text.clone());
            }
            if let Some(hud_text) = world.core.get_hud_text_mut(*entity) {
                hud_text.properties.color = nalgebra_glm::vec4(
                    entry.color[0],
                    entry.color[1],
                    entry.color[2],
                    entry.color[3],
                );
                hud_text.dirty = true;
            }
        } else {
            if let Some(text_index) = world.core.get_hud_text(*entity).map(|t| t.text_index) {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, String::new());
            }
            if let Some(hud_text) = world.core.get_hud_text_mut(*entity) {
                hud_text.dirty = true;
            }
        }
    }
}

fn scrolling_log_scroll_system(log: &mut ScrollingLog, world: &mut World) {
    let mouse_pos = world.resources.input.mouse.position;
    let screen_height = world
        .resources
        .window
        .handle
        .as_ref()
        .map(|h| h.inner_size().height as f32)
        .unwrap_or(600.0);

    let log_left = 0.0;
    let log_right = LOG_WIDTH + LOG_PADDING * 2.0;
    let log_bottom = screen_height;
    let log_top = screen_height - LOG_HEIGHT;

    let in_log_area = mouse_pos.x >= log_left
        && mouse_pos.x <= log_right
        && mouse_pos.y >= log_top
        && mouse_pos.y <= log_bottom;

    if !in_log_area {
        return;
    }

    if !world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::SCROLLED)
    {
        return;
    }

    let scroll_lines = -world.resources.input.mouse.wheel_delta.y.round() as i32;
    let max_scroll = log.entries.len().saturating_sub(LOG_VISIBLE_ENTRIES);

    if scroll_lines < 0 {
        log.scroll_offset = log
            .scroll_offset
            .saturating_sub(scroll_lines.unsigned_abs() as usize);
    } else {
        log.scroll_offset = (log.scroll_offset + scroll_lines as usize).min(max_scroll);
    }
}

fn scrolling_log_auto_add_entries(log: &mut ScrollingLog, elapsed: f32) {
    if elapsed - log.last_entry_time >= 2.0 {
        log.last_entry_time = elapsed;
        log.entry_counter += 1;

        let colors = [
            [1.0, 0.3, 0.3, 1.0],
            [0.3, 1.0, 0.3, 1.0],
            [0.3, 0.5, 1.0, 1.0],
            [1.0, 1.0, 0.3, 1.0],
            [1.0, 0.5, 1.0, 1.0],
        ];
        let color = colors[(log.entry_counter as usize) % colors.len()];

        let messages = [
            "Player joined the game",
            "Enemy spotted!",
            "Resource collected",
            "Building complete",
            "Unit ready",
            "Quest updated",
            "Achievement unlocked!",
        ];
        let message = messages[(log.entry_counter as usize) % messages.len()];

        scrolling_log_add_entry(log, format!("[{}] {}", log.entry_counter, message), color);
    }
}

#[derive(Default)]
struct HudTextDemoState {
    fps_text: Option<Entity>,
    timer_text: Option<Entity>,
    start_time: f32,
    scrolling_log: Option<ScrollingLog>,
}

impl State for HudTextDemoState {
    fn title(&self) -> &str {
        "HUD Text Demo - Nightshade"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;

        let camera_position = Vec3::new(0.0, 4.0, 10.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);

        let fps_props = TextProperties {
            font_size: 24.0,
            color: nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0),
            outline_width: 0.0,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        };

        self.fps_text = Some(spawn_hud_text_with_properties(
            world,
            "FPS: 0",
            HudAnchor::TopLeft,
            nalgebra_glm::vec2(10.0, 10.0),
            fps_props,
        ));

        let title_props = TextProperties {
            font_size: 48.0,
            color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.03,
            outline_color: nalgebra_glm::vec4(0.2, 0.2, 0.8, 1.0),
            ..Default::default()
        };

        spawn_hud_text_with_properties(
            world,
            "HUD Text Demo",
            HudAnchor::TopCenter,
            nalgebra_glm::vec2(0.0, 20.0),
            title_props,
        );

        let timer_props = TextProperties {
            font_size: 32.0,
            color: nalgebra_glm::vec4(1.0, 0.5, 0.0, 1.0),
            alignment: TextAlignment::Center,
            ..Default::default()
        };

        self.timer_text = Some(spawn_hud_text_with_properties(
            world,
            "Time: 0.0s",
            HudAnchor::Center,
            nalgebra_glm::vec2(0.0, 0.0),
            timer_props,
        ));

        spawn_hud_text(
            world,
            "Bottom Left",
            HudAnchor::BottomLeft,
            nalgebra_glm::vec2(10.0, -10.0),
        );

        spawn_hud_text(
            world,
            "Bottom Center",
            HudAnchor::BottomCenter,
            nalgebra_glm::vec2(0.0, -10.0),
        );

        spawn_hud_text(
            world,
            "Bottom Right",
            HudAnchor::BottomRight,
            nalgebra_glm::vec2(-10.0, -10.0),
        );

        let corner_props = TextProperties {
            font_size: 16.0,
            color: nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0),
            ..Default::default()
        };

        spawn_hud_text_with_properties(
            world,
            "Top Right",
            HudAnchor::TopRight,
            nalgebra_glm::vec2(-10.0, 10.0),
            corner_props.clone(),
        );

        spawn_hud_text_with_properties(
            world,
            "Center Left",
            HudAnchor::CenterLeft,
            nalgebra_glm::vec2(10.0, 0.0),
            corner_props.clone(),
        );

        spawn_hud_text_with_properties(
            world,
            "Center Right",
            HudAnchor::CenterRight,
            nalgebra_glm::vec2(-10.0, 0.0),
            corner_props,
        );

        let mut log = scrolling_log_new();
        scrolling_log_spawn_ui(world, &mut log);
        self.scrolling_log = Some(log);

        self.start_time = (world.resources.window.timing.uptime_milliseconds as f32) / 1000.0;
    }

    fn run_systems(&mut self, world: &mut World) {
        fly_camera_system(world);

        let fps = world.resources.window.timing.frames_per_second;
        if let Some(fps_entity) = self.fps_text {
            let text_index = world.core.get_hud_text(fps_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("FPS: {:.0}", fps));
                if let Some(hud_text) = world.core.get_hud_text_mut(fps_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        let elapsed =
            (world.resources.window.timing.uptime_milliseconds as f32) / 1000.0 - self.start_time;
        if let Some(timer_entity) = self.timer_text {
            let text_index = world.core.get_hud_text(timer_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("Time: {:.1}s", elapsed));
                if let Some(hud_text) = world.core.get_hud_text_mut(timer_entity) {
                    hud_text.dirty = true;
                    let r = (elapsed * 2.0).sin() * 0.5 + 0.5;
                    let g = (elapsed * 3.0).sin() * 0.5 + 0.5;
                    hud_text.properties.color = nalgebra_glm::vec4(r, g, 0.2, 1.0);
                }
            }
        }

        if let Some(ref mut log) = self.scrolling_log {
            scrolling_log_auto_add_entries(log, elapsed);
            scrolling_log_scroll_system(log, world);
            scrolling_log_update_ui(world, log);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(HudTextDemoState::default())
}
