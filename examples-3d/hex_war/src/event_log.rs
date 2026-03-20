use crate::ecs::{Faction, GameEvents};
use nightshade::ecs::ui::state::UiStateTrait;
use nightshade::prelude::*;
use std::collections::VecDeque;

const MAX_LOG_ENTRIES: usize = 1000;
const VISIBLE_ENTRIES: usize = 8;
const LOG_FONT_SIZE: f32 = 13.0;
const LOG_WIDTH: f32 = 420.0;

#[derive(Clone)]
pub struct LogEntry {
    pub faction_tag: String,
    pub faction_color: [f32; 4],
    pub message: String,
}

struct LogLineUi {
    faction_entity: Entity,
    message_entity: Entity,
}

pub struct EventLogUi {
    pub screen: Entity,
    lines: Vec<LogLineUi>,
}

pub struct EventLog {
    pub entries: VecDeque<LogEntry>,
    pub scroll_offset: usize,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            scroll_offset: 0,
        }
    }

    fn add_entry(&mut self, faction: Faction, message: String) {
        let faction_tag = format!("[{}]", faction.name());
        let fc = faction.color();
        self.entries.push_back(LogEntry {
            faction_tag,
            faction_color: fc,
            message,
        });
        if self.entries.len() > MAX_LOG_ENTRIES {
            self.entries.pop_front();
            if self.scroll_offset > 0 {
                self.scroll_offset -= 1;
            }
        }
        self.scroll_offset = self.entries.len().saturating_sub(VISIBLE_ENTRIES);
    }

    pub fn add_combat(
        &mut self,
        attacker_faction: Faction,
        defender_faction: Faction,
        attacker_survived: bool,
        defender_survived: bool,
    ) {
        let defender_name = defender_faction.name();
        let message = if !defender_survived {
            format!("destroyed {} unit", defender_name)
        } else if !attacker_survived {
            format!("was repelled by {}", defender_name)
        } else {
            format!("attacked {}", defender_name)
        };
        self.add_entry(attacker_faction, message);
    }

    pub fn add_faction_eliminated(&mut self, eliminated_faction: Faction) {
        self.add_entry(eliminated_faction, "has been eliminated!".to_string());
    }

    pub fn add_reinforcement(&mut self, faction: Faction, soldiers: i32, location: &str) {
        let message = format!("reinforced {} (+{})", location, soldiers);
        self.add_entry(faction, message);
    }

    pub fn add_turn_start(&mut self, turn: u32, faction: Faction) {
        let message = format!("Turn {} begins", turn);
        self.add_entry(faction, message);
    }

    pub fn add_speech(&mut self, faction: Faction) {
        self.add_entry(faction, "gave an inspiring speech".to_string());
    }

    pub fn drain_events(&mut self, events: &mut GameEvents) {
        for event in events.combat_events.drain(..) {
            self.add_combat(
                event.attacker_faction,
                event.defender_faction,
                event.attacker_survived,
                event.defender_survived,
            );
        }
        for event in events.speech_events.drain(..) {
            self.add_speech(event.faction);
        }
        for event in events.reinforcement_events.drain(..) {
            self.add_reinforcement(event.faction, event.soldiers, &event.location_name);
        }
        for event in events.faction_eliminated_events.drain(..) {
            self.add_faction_eliminated(event.faction);
        }
    }

    pub fn scroll_system(&mut self, world: &mut World) {
        let mouse_pos = world.resources.input.mouse.position;
        let screen_height = world
            .resources
            .window
            .handle
            .as_ref()
            .map(|h| h.inner_size().height as f32)
            .unwrap_or(600.0);

        let line_height = LOG_FONT_SIZE * 1.35;
        let log_height = VISIBLE_ENTRIES as f32 * line_height + 16.0;
        let log_left = 0.0;
        let log_right = LOG_WIDTH;
        let log_bottom = screen_height;
        let log_top = screen_height - log_height;

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
        let max_scroll = self.entries.len().saturating_sub(VISIBLE_ENTRIES);

        if scroll_lines < 0 {
            self.scroll_offset = self
                .scroll_offset
                .saturating_sub(scroll_lines.unsigned_abs() as usize);
        } else {
            self.scroll_offset = (self.scroll_offset + scroll_lines as usize).min(max_scroll);
        }
    }
}

pub fn build_event_log_ui(world: &mut World) -> EventLogUi {
    let dim = Vec4::new(0.6, 0.6, 0.6, 1.0);
    let panel_bg = Vec4::new(0.0, 0.0, 0.0, 0.45);
    let line_height = LOG_FONT_SIZE * 1.35;

    let mut tree = UiTreeBuilder::new(world);

    let mut lines = Vec::new();

    let screen = tree
        .add_node()
        .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
        .with_visible(false)
        .with_layer(UiLayer::FloatingPanels)
        .without_pointer_events()
        .with_children(|tree| {
            let log_height = VISIBLE_ENTRIES as f32 * line_height + 16.0;
            tree.add_node()
                .window(
                    Rl(Vec2::new(0.0, 100.0)) + Ab(Vec2::new(6.0, -6.0)),
                    Ab(Vec2::new(LOG_WIDTH, log_height)),
                    Anchor::BottomLeft,
                )
                .with_rect(6.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_color::<UiBase>(panel_bg)
                .flow(FlowDirection::Vertical, 6.0, 2.0)
                .without_pointer_events()
                .with_children(|tree| {
                    for _ in 0..VISIBLE_ENTRIES {
                        let row = tree
                            .add_node()
                            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, line_height)))
                            .flow(FlowDirection::Horizontal, 0.0, 4.0)
                            .without_pointer_events()
                            .entity();

                        tree.push_parent(row);

                        let faction_entity = tree
                            .add_node()
                            .flow_child(Ab(Vec2::new(90.0, line_height)))
                            .with_text("", LOG_FONT_SIZE)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(dim)
                            .without_pointer_events()
                            .done();

                        let message_entity = tree
                            .add_node()
                            .flow_child(Ab(Vec2::new(300.0, line_height)))
                            .with_text("", LOG_FONT_SIZE)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_color::<UiBase>(dim)
                            .without_pointer_events()
                            .done();

                        tree.pop_parent();

                        lines.push(LogLineUi {
                            faction_entity,
                            message_entity,
                        });
                    }
                })
                .done();
        })
        .done();

    tree.finish();

    EventLogUi { screen, lines }
}

pub fn update_event_log_ui(
    world: &mut World,
    log: &EventLog,
    ui: &EventLogUi,
    previous_scroll: &mut usize,
    previous_count: &mut usize,
) {
    let current_scroll = log.scroll_offset;
    let current_count = log.entries.len();

    if current_scroll == *previous_scroll && current_count == *previous_count {
        return;
    }
    *previous_scroll = current_scroll;
    *previous_count = current_count;

    let start_index = log.scroll_offset;

    for (slot_index, line) in ui.lines.iter().enumerate() {
        if let Some(entry) = log.entries.get(start_index + slot_index) {
            world.ui_set_text(line.faction_entity, &entry.faction_tag);
            if let Some(color) = world.ui.get_ui_node_color_mut(line.faction_entity) {
                color.colors[UiBase::INDEX] = Some(Vec4::new(
                    entry.faction_color[0],
                    entry.faction_color[1],
                    entry.faction_color[2],
                    entry.faction_color[3],
                ));
            }
            world.ui_set_text(line.message_entity, &entry.message);
            if let Some(color) = world.ui.get_ui_node_color_mut(line.message_entity) {
                color.colors[UiBase::INDEX] = Some(Vec4::new(1.0, 1.0, 1.0, 1.0));
            }
        } else {
            world.ui_set_text(line.faction_entity, "");
            world.ui_set_text(line.message_entity, "");
        }
    }
}
