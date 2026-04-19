//! Per-area content.
//!
//! Each submodule in this directory owns every entity anchored to one
//! conceptual place or system in the adventure: its rooms, the items that
//! live in those rooms, the rules that fire against them, NPCs stationed
//! there, dialogues, and any timers driving that area's behaviour.
//!
//! An area module returns an [`AreaContents`] bag. The top-level
//! [`crate::game::build_world`] walks the list of areas, merging every
//! bag into the single `World` that the engine consumes. Merging is done
//! via [`crate::game::merge::merge`], which panics on duplicate IDs so
//! cross-area name collisions surface immediately.

pub mod cellar;
pub mod cottage;
pub mod setup;
pub mod shore;
pub mod storm;
pub mod stranger;
pub mod tower;

use nightshade::interactive_fiction::data::{
    Dialogue, DialogueId, Item, ItemId, Npc, NpcId, Room, RoomId, Rule, RuleId, Timer, TimerId,
};
use std::collections::BTreeMap;

/// Content contributed by a single area to the world.
///
/// Every collection defaults to empty — an area only fills the entity
/// kinds it actually owns. E.g. the shore area contributes rooms and
/// items but no NPCs or timers.
///
/// Area modules should use the `add_*` methods below instead of touching
/// the public maps directly — they panic on duplicate IDs, catching a
/// class of silent-later-wins bug that `BTreeMap::insert` would otherwise
/// hide.
#[derive(Default)]
pub struct AreaContents {
    pub rooms: BTreeMap<RoomId, Room>,
    pub items: BTreeMap<ItemId, Item>,
    pub rules: BTreeMap<RuleId, Rule>,
    pub npcs: BTreeMap<NpcId, Npc>,
    pub dialogues: BTreeMap<DialogueId, Dialogue>,
    pub timers: BTreeMap<TimerId, Timer>,
}

impl AreaContents {
    /// Register a room. Panics if an entry for `id` already exists in
    /// this area's contribution.
    pub fn add_room(&mut self, id: RoomId, room: Room) {
        insert_unique(&mut self.rooms, id, room, "room");
    }

    /// Register an item. Panics on duplicate id within the area.
    pub fn add_item(&mut self, id: ItemId, item: Item) {
        insert_unique(&mut self.items, id, item, "item");
    }

    /// Register a rule. Panics on duplicate id within the area.
    pub fn add_rule(&mut self, id: RuleId, rule: Rule) {
        insert_unique(&mut self.rules, id, rule, "rule");
    }

    /// Register an NPC. Panics on duplicate id within the area.
    pub fn add_npc(&mut self, id: NpcId, npc: Npc) {
        insert_unique(&mut self.npcs, id, npc, "NPC");
    }

    /// Register a dialogue. Panics on duplicate id within the area.
    pub fn add_dialogue(&mut self, id: DialogueId, dialogue: Dialogue) {
        insert_unique(&mut self.dialogues, id, dialogue, "dialogue");
    }

    /// Register a timer. Panics on duplicate id within the area.
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

/// The fixed list of area builders. `build_world` walks this list.
pub fn all() -> Vec<fn() -> AreaContents> {
    vec![
        setup::build,
        shore::build,
        cottage::build,
        tower::build,
        cellar::build,
        stranger::build,
        storm::build,
    ]
}
