//! Per-area content bags.
//!
//! Each area module contributes rooms, items, rules, entities, dialogues,
//! and timers anchored to one part of the game. The top-level
//! `build_world` merges the bags into the final `World`. IDs must be
//! unique within an area (enforced by `add_*`) and across areas
//! (enforced by the top-level `merge::merge`).

pub mod ambient;
pub mod apartment;
pub mod chatter;
pub mod cycles;
pub mod mail;
pub mod office;
pub mod picture_frame;
pub mod redux;
pub mod reveal;
pub mod setup;
pub mod tools_code;
pub mod tools_notepad;
pub mod tools_reference;
pub mod tools_research;
pub mod tools_translator;
pub mod walk;

use nightshade::interactive_fiction::data::{
    Dialogue, DialogueId, Entity, EntityId, Item, ItemId, Room, RoomId, Rule, RuleId, Timer,
    TimerId,
};
use std::collections::BTreeMap;

/// Content contributed by a single area. Fields default to empty so
/// individual areas only fill what they own.
#[derive(Default)]
pub struct AreaContents {
    pub rooms: BTreeMap<RoomId, Room>,
    pub items: BTreeMap<ItemId, Item>,
    pub rules: BTreeMap<RuleId, Rule>,
    pub entities: BTreeMap<EntityId, Entity>,
    pub dialogues: BTreeMap<DialogueId, Dialogue>,
    pub timers: BTreeMap<TimerId, Timer>,
}

impl AreaContents {
    pub fn add_room(&mut self, id: RoomId, room: Room) {
        insert_unique(&mut self.rooms, id, room, "room");
    }
    pub fn add_item(&mut self, id: ItemId, item: Item) {
        insert_unique(&mut self.items, id, item, "item");
    }
    pub fn add_rule(&mut self, id: RuleId, rule: Rule) {
        insert_unique(&mut self.rules, id, rule, "rule");
    }
    pub fn add_entity(&mut self, id: EntityId, entity: impl Into<Entity>) {
        insert_unique(&mut self.entities, id, entity.into(), "entity");
    }
    pub fn add_dialogue(&mut self, id: DialogueId, dialogue: Dialogue) {
        insert_unique(&mut self.dialogues, id, dialogue, "dialogue");
    }
    pub fn add_timer(&mut self, id: TimerId, timer: Timer) {
        insert_unique(&mut self.timers, id, timer, "timer");
    }
}

fn insert_unique<K, V>(target: &mut BTreeMap<K, V>, key: K, value: V, kind: &str)
where
    K: Ord + std::fmt::Display + Clone,
{
    assert!(
        !target.contains_key(&key),
        "duplicate {kind} id '{key}' registered twice within the same area"
    );
    target.insert(key, value);
}

pub fn all() -> Vec<fn() -> AreaContents> {
    vec![
        setup::build,
        apartment::build,
        walk::build,
        office::build,
        mail::build,
        tools_notepad::build,
        tools_research::build,
        tools_translator::build,
        tools_code::build,
        tools_reference::build,
        chatter::build,
        picture_frame::build,
        cycles::build,
        ambient::build,
        reveal::build,
        redux::build,
    ]
}
