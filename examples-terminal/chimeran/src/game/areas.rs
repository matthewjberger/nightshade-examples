pub mod ambient;
pub mod apartment;
pub mod chatter;
pub mod cycles;
pub mod mail;
pub mod office;
pub mod picture_frame;
pub mod reveal;
pub mod setup;
pub mod tools_code;
pub mod tools_notepad;
pub mod tools_reference;
pub mod tools_research;
pub mod tools_translator;
pub mod walk;

use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueId, DialogueOption, Effect, Entity, EntityId, FlagKey, Item,
    ItemId, NodeId, Room, RoomId, Rule, RuleId, Text, Timer, TimerId, Value,
};
use std::collections::BTreeMap;

pub fn by_cycle(base: Text, variants: Vec<(i64, Text)>) -> Text {
    let mut sorted = variants;
    sorted.sort_by_key(|(cycle, _)| *cycle);
    let mut current = base;
    for (cycle, text) in sorted {
        current = Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_cycle(), cycle),
            then: Box::new(text),
            otherwise: Box::new(current),
        };
    }
    current
}

pub fn reveal_option(
    label: &str,
    enabled_flag: FlagKey,
    seen_flag: FlagKey,
    body_node: NodeId,
) -> DialogueOption {
    DialogueOption::new(Text::lit(label))
        .with_condition(Condition::FlagSet(enabled_flag))
        .with_effects(vec![
            Effect::SetFlag(seen_flag, Value::TRUE),
            Effect::AddStat(ids::stat_exploit_counter(), -1),
        ])
        .goto(body_node)
}

pub fn body_with_acks(body: Text, acks: Vec<(FlagKey, Text)>) -> Text {
    let mut parts = vec![body];
    for (flag, ack) in acks {
        parts.push(Text::Conditional {
            when: Condition::FlagSet(flag),
            then: Box::new(ack),
            otherwise: Box::new(Text::empty()),
        });
    }
    Text::Sequence(parts)
}

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
    ]
}
