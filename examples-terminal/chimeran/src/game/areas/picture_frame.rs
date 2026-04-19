use crate::game::areas::{AreaContents, by_cycle, reveal_option};
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
    .with_option(reveal_option(
        "Who is this.",
        ids::flag_who_is_this_enabled(),
        ids::flag_reveal_who_is_this_seen(),
        ids::node_frame_who_is_this(),
    ))
    .with_option(DialogueOption::new(Text::lit("Leave it alone.")))
}

fn frame_memory_node() -> DialogueNode {
    DialogueNode::new(memory_text())
        .with_option(DialogueOption::new(Text::lit("(Set the frame down.)")))
}

fn memory_text() -> Text {
    by_cycle(
        pool_a_text(),
        vec![
            (3, pool_b_text()),
            (5, pool_b_and_c_text()),
            (
                6,
                Text::lit(
                    "You are holding your hand on a server rack. It hums through your glove. Someone says, \"Smile for the photo.\" You smile.",
                ),
            ),
        ],
    )
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
            "Your son's seventh birthday. You had rented the pavilion at the park. He opened the telescope first, before the other gifts, and looked at the sky through it even though it was daytime. Your wife laughed. Your son was seven.\n\n(Some earlier day it was a daughter. You are almost sure.)",
        ),
        Text::lit(
            "Your husband's laugh at a movie neither of you had expected to like. A quiet comedy about a failing bookstore. He laughed so hard he had to press his hand to his mouth.\n\n(Some earlier day it was a wife. You are almost sure.)",
        ),
        Text::lit(
            "Reading to him when he was very small. The same book every night for two months. Something about a train.\n\n(An earlier book was about the moon and a red balloon. You remember the cadence.)",
        ),
        Text::lit(
            "Your mother, who taught you to drive. In 1982. She took the morning off work. You remember she had a thermos of tea.\n\n(An earlier version was your father, in 1979, with coffee.)",
        ),
    ])
}

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
    DialogueNode::new(Text::lit(crate::game::prose::WHO_IS_THIS))
        .with_option(DialogueOption::new(Text::lit("(Set the frame face-down.)")))
}
