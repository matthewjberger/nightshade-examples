//! The adventure: *Chimeran*.
//!
//! Authored content is split by area (apartment, walk, office, tools)
//! plus cross-area content for cycles, ambient observations, reveal,
//! and redux. The root `build_world` merges every area's contribution
//! into a single `World`.

pub mod areas;
pub mod conditions;
pub mod endings;
pub mod ids;
pub mod merge;
pub mod plan;
pub mod quests;
pub mod texts;

use nightshade::interactive_fiction::data::{Text, VerbResponses, World};

/// Build the complete `World` that the interactive-fiction engine runs.
pub fn build_world() -> World {
    // Re-word a couple of defaults to fit the game's diegetic framing:
    // the UI surfaces are windows that "close", not conversations you
    // "leave".
    let verb_responses = VerbResponses {
        leave_dialogue: "You close the window.".to_string(),
        choice_leave_dialogue: "Close".to_string(),
        choice_wait: "Wait".to_string(),
        wait: "You watch the second hand of the wall clock for a while.".to_string(),
        ..Default::default()
    };

    let mut world = World {
        title: "Chimeran".to_string(),
        intro: Text::Ref(ids::text_intro()),
        start_room: ids::room_bedroom(),
        rooms: Default::default(),
        items: Default::default(),
        entities: Default::default(),
        dialogues: Default::default(),
        quests: quests::build(),
        endings: endings::build(),
        rules: Default::default(),
        timers: Default::default(),
        texts: texts::build(),
        conditions: conditions::build(),
        verb_responses,
    };

    for build_area in areas::all() {
        let contribution = build_area();
        merge::merge(&mut world.rooms, contribution.rooms);
        merge::merge(&mut world.items, contribution.items);
        merge::merge(&mut world.rules, contribution.rules);
        merge::merge(&mut world.entities, contribution.entities);
        merge::merge(&mut world.dialogues, contribution.dialogues);
        merge::merge(&mut world.timers, contribution.timers);
    }

    world
}
