pub mod areas;
pub mod endings;
pub mod ids;
pub mod mail;
pub mod map;
pub mod plan;
pub mod prose;
pub mod texts;

use nightshade::interactive_fiction::data::{Text, VerbResponses, World};
use std::collections::BTreeMap;

pub fn build_world() -> World {
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
        quests: BTreeMap::new(),
        endings: endings::build(),
        rules: Default::default(),
        timers: Default::default(),
        texts: texts::build(),
        conditions: BTreeMap::new(),
        verb_responses,
    };

    for build_area in areas::all() {
        let contribution = build_area();
        merge(&mut world.rooms, contribution.rooms);
        merge(&mut world.items, contribution.items);
        merge(&mut world.rules, contribution.rules);
        merge(&mut world.entities, contribution.entities);
        merge(&mut world.dialogues, contribution.dialogues);
        merge(&mut world.timers, contribution.timers);
    }

    world
}

fn merge<K: Ord + std::fmt::Display + Clone, V>(
    target: &mut BTreeMap<K, V>,
    contribution: BTreeMap<K, V>,
) {
    for (key, value) in contribution {
        assert!(
            !target.contains_key(&key),
            "duplicate id '{key}' across areas"
        );
        target.insert(key, value);
    }
}
