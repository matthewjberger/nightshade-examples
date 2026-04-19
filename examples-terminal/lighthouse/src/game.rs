//! The specific adventure: *The Lantern at Dunmere Point*.
//!
//! Content is split **by area**, not by entity kind. Each submodule under
//! `game/areas/` contributes everything anchored to one place or system:
//! its rooms, items that live there, rules that fire against it, NPCs
//! stationed there, dialogues, and timers. Adding a new room, puzzle, or
//! piece of flavour lands in exactly one file — the area file for the
//! place it belongs to.
//!
//! Cross-area content (endings, the quest graph, the shared text and
//! condition tables) stays centralized since it spans every area by
//! design.

pub mod areas;
pub mod conditions;
pub mod endings;
pub mod ids;
pub mod merge;
pub mod quests;
pub mod texts;

use nightshade::interactive_fiction::data::{Text, World};

/// Assemble the complete `World` from every area's contribution plus the
/// cross-area central content. Duplicate IDs across areas panic via
/// [`merge::merge`].
pub fn build_world() -> World {
    let mut world = World {
        title: "The Lantern at Dunmere Point".to_string(),
        intro: Text::Ref(ids::text_intro()),
        start_room: ids::room_shore(),
        rooms: Default::default(),
        items: Default::default(),
        npcs: Default::default(),
        dialogues: Default::default(),
        quests: quests::build(),
        endings: endings::build(),
        rules: Default::default(),
        timers: Default::default(),
        texts: texts::build(),
        conditions: conditions::build(),
        verb_responses: Default::default(),
    };

    for build_area in areas::all() {
        let contribution = build_area();
        merge::merge(&mut world.rooms, contribution.rooms);
        merge::merge(&mut world.items, contribution.items);
        merge::merge(&mut world.rules, contribution.rules);
        merge::merge(&mut world.npcs, contribution.npcs);
        merge::merge(&mut world.dialogues, contribution.dialogues);
        merge::merge(&mut world.timers, contribution.timers);
    }

    world
}
