//! The picture frame on the desk.
//!
//! The frame is a fixture. "Opening" it shows a memory from one of
//! three pools (A, B, C) based on the current cycle and whether the
//! player has looked today. Post-exploit, it unlocks a "Who is this"
//! option that reveals the donor-shoot photograph.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueNode, DialogueOption, Effect, EntityLocation, Rule, Text, Trigger,
    Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_dialogue(
        ids::dialogue_picture_frame(),
        Dialogue::new(ids::node_frame_prompt())
            .with_node(ids::node_frame_prompt(), frame_prompt_node())
            .with_node(ids::node_frame_memory(), frame_memory_node())
            .with_node(ids::node_frame_who_is_this(), frame_who_is_this_node()),
    );

    // Hide the frame on cycle 7+ (face-down cycle 7, absent cycle 8).
    area.add_rule(
        ids::rule_hide_frame_late(),
        Rule::on(
            Trigger::TurnStart,
            vec![Effect::MoveEntity(
                ids::fixture_picture_frame(),
                EntityLocation::Nowhere,
            )],
        )
        .with_condition(Condition::StatAtLeast(ids::stat_cycle(), 7))
        .once(),
    );

    area
}

fn frame_prompt_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::FlagSet(ids::flag_frame_looked_today()),
        then: Box::new(Text::lit(
            "The frame on the right edge of your desk. You've looked at it already today.",
        )),
        otherwise: Box::new(Text::lit(
            "A silver frame with a photograph in it. It sits on the right edge of your desk. You know what is in it.",
        )),
    })
    .with_option(
        DialogueOption::new(Text::lit("Look at it."))
            .with_condition(Condition::FlagUnset(ids::flag_frame_looked_today()))
            .with_effects(vec![Effect::SetFlag(
                ids::flag_frame_looked_today(),
                Value::TRUE,
            )])
            .goto(ids::node_frame_memory()),
    )
    .with_option(
        DialogueOption::new(Text::lit("Who is this."))
            .with_condition(Condition::FlagSet(ids::flag_who_is_this_enabled()))
            .goto(ids::node_frame_who_is_this()),
    )
    .with_option(DialogueOption::new(Text::lit("Leave it alone.")))
}

fn frame_memory_node() -> DialogueNode {
    DialogueNode::new(memory_text())
        .with_option(DialogueOption::new(Text::lit("(Set the frame down.)")))
}

fn memory_text() -> Text {
    // Cycle 6 fires the scripted C6 memory (the server rack). Cycle 5
    // mixes pool B with pool C (not-Cameron's memories) — the player
    // may get either variant. Cycle 4 stays in pool B. Cycles 1-3 use
    // pool A (warm, internally consistent).
    Text::Conditional {
        when: Condition::StatAtLeast(ids::stat_cycle(), 6),
        then: Box::new(Text::lit(
            "You are holding your hand on a server rack. It hums through your glove. Someone says, \"Smile for the photo.\" You smile.",
        )),
        otherwise: Box::new(Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_cycle(), 5),
            then: Box::new(pool_b_and_c_text()),
            otherwise: Box::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 4),
                then: Box::new(pool_b_text()),
                otherwise: Box::new(pool_a_text()),
            }),
        }),
    }
}

fn pool_a_text() -> Text {
    Text::OneOf(vec![
        Text::lit(
            "Your daughter's seventh birthday. You had rented the pavilion at the park. The cake had too much frosting and she didn't care, she laughed and got it on her face and her mother laughed and you laughed. She opened the stuffed elephant before any of the other gifts and she slept with it that night.",
        ),
        Text::lit(
            "Your wife's laugh at a movie neither of you had expected to like. A quiet comedy about a failing bookstore. She laughed so hard she had to press her hand to her mouth. You watched her laugh instead of watching the screen.",
        ),
        Text::lit(
            "Reading to her when she was very small. The same book every night for two months. You could recite it from memory, even now, the way it went about the moon and the room and the red balloon.",
        ),
        Text::lit(
            "Your father, who taught you to drive in an empty parking lot in 1979. He had taken the morning off. He had made coffee in a thermos. He drank it while you made the car jerk forward and back and forward again. He did not get angry. He told you a story about his own father teaching him.",
        ),
    ])
}

fn pool_b_text() -> Text {
    Text::OneOf(vec![
        Text::lit(
            "Your son's seventh birthday. You had rented the pavilion at the park. He opened the telescope first, before the other gifts, and looked at the sky through it even though it was daytime. Your wife laughed. Your son was seven.",
        ),
        Text::lit(
            "Your husband's laugh at a movie neither of you had expected to like. A quiet comedy about a failing bookstore. He laughed so hard he had to press his hand to his mouth.",
        ),
        Text::lit(
            "Reading to him when he was very small. The same book every night for two months. Something about a train.",
        ),
        Text::lit(
            "Your mother, who taught you to drive. In 1982. She took the morning off work. You remember she had a thermos of tea.",
        ),
    ])
}

/// Cycle 5 memory pool: union of B (warm/contradictory) and C (not
/// Cameron's at all — belongs to donor minds in the substrate). The
/// player may get any variant — some of which belong to someone else.
fn pool_b_and_c_text() -> Text {
    Text::OneOf(vec![
        Text::lit(
            "Your son's seventh birthday. You had rented the pavilion at the park. He opened the telescope first, before the other gifts, and looked at the sky through it even though it was daytime. Your wife laughed. Your son was seven.",
        ),
        Text::lit(
            "Your husband's laugh at a movie neither of you had expected to like. A quiet comedy about a failing bookstore. He laughed so hard he had to press his hand to his mouth.",
        ),
        Text::lit(
            "Reading to him when he was very small. The same book every night for two months. Something about a train.",
        ),
        Text::lit(
            "Your mother, who taught you to drive. In 1982. She took the morning off work. You remember she had a thermos of tea.",
        ),
        Text::lit(
            "You are defending your dissertation. The room is small and overheated. Your advisor is in the front row; you cannot read her face. You have practiced this presentation for six weeks. The data is on the third slide. You hope it holds.\n\n(You have never had an advisor. You think. Haven't you?)",
        ),
        Text::lit(
            "You are in the cockpit. The procedure is automatic. Left hand on the yoke, right hand on the throttle. Flaps extended. Landing gear down. The runway is ahead. You have done this six thousand times. You do it again.\n\n(You have never flown a plane. You think. Haven't you?)",
        ),
        Text::lit(
            "You are eleven years old in a kitchen that is not yours. Your grandmother is showing you how to fold the corners of the pastry. The light comes through the window in a specific way because it is October in suburban Ohio in 1965. You can smell the yeast.\n\n(You have never been to Ohio. You think you'd know if you had.)",
        ),
    ])
}

fn frame_who_is_this_node() -> DialogueNode {
    DialogueNode::new(Text::lit(include_str!("../prose/who_is_this.txt")))
        .with_on_enter(vec![
            Effect::SetFlag(ids::flag_reveal_who_is_this_seen(), Value::TRUE),
            Effect::AddStat(ids::stat_exploit_counter(), -1),
        ])
        .with_option(DialogueOption::new(Text::lit("(Set the frame face-down.)")))
}
