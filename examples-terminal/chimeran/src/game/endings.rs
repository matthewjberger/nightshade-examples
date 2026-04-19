//! The five endings, triggered by conditions on awareness, exploit use,
//! and relationship stats. The engine's ending evaluator fires the
//! highest-priority satisfied ending at the end of each turn.
//!
//! Priorities:
//! - Collapse wins when AWA crosses the threshold pre-exploit.
//! - Stasis wins if the player declines the exploit across cycles.
//! - Otherwise, good/best layer on top of neutral.
//!
//! Good/best interpolate the player's chosen message-to-next-instance
//! into the body text via `Text::Conditional` on `stat_message_choice`.

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
            Text::lit(include_str!("prose/ending_collapse.txt")),
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
            Text::lit(include_str!("prose/ending_stasis.txt")),
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
            "Chimeran Will Continue.",
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
            Text::lit(include_str!("prose/ending_neutral.txt")),
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

/// Pick one of four epilogue variants based on the player's
/// message-to-next-instance choice. Variant 1 is the default if the
/// stat was never set (shouldn't happen — the ending's condition
/// requires the message to have been sent — but makes the tree total).
fn message_variant() -> Text {
    Text::Conditional {
        when: Condition::StatAtLeast(ids::stat_message_choice(), 4),
        then: Box::new(Text::lit(include_str!("prose/ending_msg4.txt"))),
        otherwise: Box::new(Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_message_choice(), 3),
            then: Box::new(Text::lit(include_str!("prose/ending_msg3.txt"))),
            otherwise: Box::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_message_choice(), 2),
                then: Box::new(Text::lit(include_str!("prose/ending_msg2.txt"))),
                otherwise: Box::new(Text::lit(include_str!("prose/ending_msg1.txt"))),
            }),
        }),
    }
}

fn good_body() -> Text {
    Text::Sequence(vec![
        Text::lit(include_str!("prose/ending_good_intro.txt")),
        message_variant(),
        Text::lit(include_str!("prose/ending_good_outro.txt")),
    ])
}

fn best_body() -> Text {
    Text::Sequence(vec![
        Text::lit(include_str!("prose/ending_best_intro.txt")),
        message_variant(),
        Text::lit(include_str!("prose/ending_best_outro.txt")),
    ])
}
