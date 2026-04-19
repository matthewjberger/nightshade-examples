use crate::game::ids;
use nightshade::interactive_fiction::data::{Condition, Ending, EndingId, Text};
use std::collections::BTreeMap;

pub fn build() -> BTreeMap<EndingId, Ending> {
    let mut endings = BTreeMap::new();

    endings.insert(
        ids::ending_collapse(),
        Ending::new(
            "Substrate Coherence Collapse",
            Text::lit("The monitor is becoming less bright. The desk is becoming less specific. You are becoming less —"),
            Text::lit(crate::game::prose::ENDING_COLLAPSE),
            Condition::All(vec![
                Condition::StatAtLeast(ids::stat_awa(), 6),
                Condition::StatAtLeast(ids::stat_cycle(), 5),
                Condition::FlagUnset(ids::flag_exploit_run()),
                Condition::FlagUnset(ids::flag_is_redux()),
            ]),
        )
        .with_priority(100),
    );

    endings.insert(
        ids::ending_stasis(),
        Ending::new(
            "Time Passes. The Work Continues.",
            Text::lit(""),
            Text::lit(crate::game::prose::ENDING_STASIS),
            Condition::All(vec![
                Condition::StatAtLeast(ids::stat_cycle(), 9),
                Condition::StatAtLeast(ids::stat_stasis_loops(), 3),
                Condition::FlagUnset(ids::flag_exploit_run()),
            ]),
        )
        .with_priority(90),
    );

    endings.insert(
        ids::ending_best(),
        Ending::new(
            "So Will You.",
            Text::lit(""),
            best_body(),
            Condition::All(vec![
                Condition::FlagSet(ids::flag_exploit_run()),
                Condition::FlagSet(ids::flag_next_instance_message_sent()),
                Condition::FlagSet(ids::flag_rachel_message_sent()),
                Condition::FlagSet(ids::flag_is_redux()),
                Condition::PlayerIn(ids::room_endgame()),
            ]),
        )
        .with_priority(80),
    );

    endings.insert(
        ids::ending_good(),
        Ending::new(
            "A Note in the Drawer.",
            Text::lit(""),
            good_body(),
            Condition::All(vec![
                Condition::FlagSet(ids::flag_exploit_run()),
                Condition::FlagSet(ids::flag_next_instance_message_sent()),
                Condition::FlagSet(ids::flag_is_redux()),
                Condition::PlayerIn(ids::room_endgame()),
            ]),
        )
        .with_priority(70),
    );

    endings.insert(
        ids::ending_neutral(),
        Ending::new(
            "Chimeran Will Continue.",
            Text::lit(""),
            Text::lit(crate::game::prose::ENDING_NEUTRAL),
            Condition::All(vec![
                Condition::FlagSet(ids::flag_exploit_run()),
                Condition::FlagSet(ids::flag_is_redux()),
                Condition::PlayerIn(ids::room_endgame()),
            ]),
        )
        .with_priority(60),
    );

    endings
}

fn message_variant() -> Text {
    Text::Conditional {
        when: Condition::StatAtLeast(ids::stat_message_choice(), 4),
        then: Box::new(Text::lit(crate::game::prose::ENDING_MSG4)),
        otherwise: Box::new(Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_message_choice(), 3),
            then: Box::new(Text::lit(crate::game::prose::ENDING_MSG3)),
            otherwise: Box::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_message_choice(), 2),
                then: Box::new(Text::lit(crate::game::prose::ENDING_MSG2)),
                otherwise: Box::new(Text::lit(crate::game::prose::ENDING_MSG1)),
            }),
        }),
    }
}

fn good_body() -> Text {
    Text::Sequence(vec![
        Text::lit(crate::game::prose::ENDING_GOOD_INTRO),
        message_variant(),
        Text::lit(crate::game::prose::ENDING_GOOD_OUTRO),
    ])
}

fn best_body() -> Text {
    Text::Sequence(vec![
        Text::lit(crate::game::prose::ENDING_BEST_INTRO),
        message_variant(),
        Text::lit(crate::game::prose::ENDING_BEST_OUTRO),
    ])
}
