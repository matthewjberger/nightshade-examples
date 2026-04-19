//! The central quest graph.

use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Quest, QuestId, QuestStage, QuestTransition, Text,
};
use std::collections::BTreeMap;

pub fn build() -> BTreeMap<QuestId, Quest> {
    let mut quests = BTreeMap::new();

    let quest = Quest::new("The Lantern at Dunmere Point", ids::stage_begin())
        .with_stage(
            ids::stage_begin(),
            QuestStage::active(Text::lit("Find out what has happened at the lighthouse."))
                .with_transition(QuestTransition::new(
                    ids::stage_unlocked_cottage(),
                    Condition::PlayerIn(ids::room_cottage()),
                )),
        )
        .with_stage(
            ids::stage_unlocked_cottage(),
            QuestStage::active(Text::lit(
                "The keeper is missing. The cellar and tower both hold secrets.",
            ))
            .with_transition(QuestTransition::new(
                ids::stage_found_keeper(),
                Condition::FlagSet(ids::flag_found_keeper()),
            )),
        )
        .with_stage(
            ids::stage_found_keeper(),
            QuestStage::active(Text::lit(
                "The keeper is dead. Decide whether to light the lantern, sabotage it, or flee.",
            ))
            .with_transition(QuestTransition::new(
                ids::stage_restored(),
                Condition::FlagSet(ids::flag_lantern_restored()),
            ))
            .with_transition(QuestTransition::new(
                ids::stage_sabotaged(),
                Condition::FlagSet(ids::flag_lantern_sabotaged()),
            ))
            .with_transition(QuestTransition::new(
                ids::stage_abandoned(),
                Condition::PlayerIn(ids::room_gone()),
            )),
        )
        .with_stage(
            ids::stage_restored(),
            QuestStage::success(Text::lit("You relit the lantern. The ships came home.")),
        )
        .with_stage(
            ids::stage_sabotaged(),
            QuestStage::success(Text::lit(
                "You kept the light dark. The wreckers paid their thirty pounds.",
            )),
        )
        .with_stage(
            ids::stage_abandoned(),
            QuestStage::failure(Text::lit(
                "You walked away from the headland before the storm broke.",
            )),
        );

    quests.insert(ids::quest_lantern(), quest);
    quests
}
