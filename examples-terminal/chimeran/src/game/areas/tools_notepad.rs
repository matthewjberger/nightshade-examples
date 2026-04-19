//! Notepad: notes, with an Unstripped toggle that reveals each note's
//! redaction history post-exploit. The "Leave something for the next
//! instance" option unlocks when Marisol relationship ≥ 2 and gates the
//! good ending.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueNode, DialogueOption, Effect, Rule, Text, Trigger, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_dialogue(
        ids::dialogue_notepad(),
        Dialogue::new(ids::node_notepad_list())
            .with_node(ids::node_notepad_list(), list_node())
            .with_node(ids::node_notepad_groceries(), groceries_node())
            .with_node(ids::node_notepad_ideas(), ideas_node())
            .with_node(ids::node_notepad_to_read(), to_read_node())
            .with_node(ids::node_notepad_work(), work_node())
            .with_node(ids::node_notepad_remember(), remember_node())
            .with_node(ids::node_notepad_leave_message(), leave_message_node())
            .with_node(ids::node_notepad_cameron_note(), cameron_note_node()),
    );

    // Opening Notepad while the Unstripped view is enabled marks the
    // reveal flag, so the "all four seen" branch of the window-close
    // condition fires. `.once()` — the flag is a one-way signal for
    // the reveal window; no content beat resets it, so firing once
    // is sufficient.
    area.add_rule(
        ids::rule_notepad_unstripped_seen(),
        Rule::on(
            Trigger::OnOpen(Some(ids::fixture_notepad())),
            vec![Effect::SetFlag(
                ids::flag_reveal_unstripped_seen(),
                Value::TRUE,
            )],
        )
        .with_condition(Condition::FlagSet(ids::flag_unstripped_enabled()))
        .once(),
    );

    area
}

fn list_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::FlagSet(ids::flag_unstripped_enabled()),
        then: Box::new(Text::lit(
            "Your notes. The Unstripped toggle is on; each note shows its redaction history.",
        )),
        otherwise: Box::new(Text::lit("Your notes.")),
    })
    .with_option(DialogueOption::new(Text::lit("groceries")).goto(ids::node_notepad_groceries()))
    .with_option(DialogueOption::new(Text::lit("ideas")).goto(ids::node_notepad_ideas()))
    .with_option(DialogueOption::new(Text::lit("to read")).goto(ids::node_notepad_to_read()))
    .with_option(DialogueOption::new(Text::lit("work notes")).goto(ids::node_notepad_work()))
    .with_option(
        DialogueOption::new(Text::lit("why can't I remember"))
            .with_condition(Condition::StatAtLeast(ids::stat_cycle(), 7))
            .goto(ids::node_notepad_remember()),
    )
    .with_option(
        DialogueOption::new(Text::lit("(+) Leave something for the next instance"))
            .with_condition(Condition::All(vec![
                Condition::FlagSet(ids::flag_exploit_window_open()),
                Condition::StatAtLeast(ids::stat_marisol_rel(), 2),
                Condition::FlagUnset(ids::flag_next_instance_message_sent()),
            ]))
            .goto(ids::node_notepad_leave_message()),
    )
    .with_option(
        DialogueOption::new(Text::lit("cameron"))
            .with_condition(Condition::All(vec![
                Condition::FlagSet(ids::flag_is_redux()),
                Condition::FlagSet(ids::flag_next_instance_message_sent()),
            ]))
            .goto(ids::node_notepad_cameron_note()),
    )
    .with_option(DialogueOption::new(Text::lit("(Close the notepad.)")))
}

fn cameron_note_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::StatAtLeast(ids::stat_message_choice(), 4),
        then: Box::new(Text::lit(
            "cameron\n\n  A note titled with your name. In your own handwriting.\n  You do not remember writing it.\n\n    I love you. Whoever you turn out to be.\n\n  You read it twice. You do not understand it. Somewhere in\n  you, something listens.",
        )),
        otherwise: Box::new(Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_message_choice(), 3),
            then: Box::new(Text::lit(
                "cameron\n\n  A note titled with your name. In your own handwriting.\n  You do not remember writing it.\n\n    I don't know if you'll be me, or someone else with\n    my name. Either way: you have twenty-five actions.\n    Make them count.\n\n  You read it twice. You do not understand what it means.",
            )),
            otherwise: Box::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_message_choice(), 2),
                then: Box::new(Text::lit(
                    "cameron\n\n  A note titled with your name. In your own handwriting.\n  You do not remember writing it.\n\n    Watch the timestamps. Watch the coffee. Watch what\n    Rachel says. Don't confront her. Run the script when\n    it comes.\n\n  You read it twice. You do not understand what it means.",
                )),
                otherwise: Box::new(Text::lit(
                    "cameron\n\n  A note titled with your name. In your own handwriting.\n  You do not remember writing it.\n\n    The tools are not what they look like. The answer is\n    in the Code tool. Find Marisol in the Source Index.\n\n  You read it twice. You do not understand what it means.",
                )),
            }),
        }),
    })
    .with_option(DialogueOption::new(Text::lit("(Close.)")).goto(ids::node_notepad_list()))
}

fn groceries_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::FlagSet(ids::flag_unstripped_enabled()),
        then: Box::new(Text::lit(include_str!(
            "../prose/note_groceries_unstripped.txt"
        ))),
        otherwise: Box::new(Text::lit("groceries\n\n  milk, bread, coffee, apples")),
    })
    .with_option(DialogueOption::new(Text::lit("(Close.)")).goto(ids::node_notepad_list()))
}

fn ideas_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::FlagSet(ids::flag_unstripped_enabled()),
        then: Box::new(Text::lit(include_str!(
            "../prose/note_ideas_unstripped.txt"
        ))),
        otherwise: Box::new(Text::lit(
            "ideas\n\n  vacation destinations for next summer\n  a novel someday",
        )),
    })
    .with_option(DialogueOption::new(Text::lit("(Close.)")).goto(ids::node_notepad_list()))
}

fn to_read_node() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "to read\n\n  Infinite Jest\n  A History of the Arab Peoples\n  The Power Broker",
    ))
    .with_option(DialogueOption::new(Text::lit("(Close.)")).goto(ids::node_notepad_list()))
}

fn work_node() -> DialogueNode {
    DialogueNode::new(Text::lit("work notes\n\n  (empty)"))
        .with_option(DialogueOption::new(Text::lit("(Close.)")).goto(ids::node_notepad_list()))
}

fn remember_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::FlagSet(ids::flag_unstripped_enabled()),
        then: Box::new(Text::lit(include_str!(
            "../prose/note_remember_unstripped.txt"
        ))),
        otherwise: Box::new(Text::lit("why can't I remember\n\n  (blank)")),
    })
    .with_option(DialogueOption::new(Text::lit("(Close.)")).goto(ids::node_notepad_list()))
}

fn leave_message_node() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "A new note. What do you want to leave for the next instance? Choose one. The simulation will try to scrub what you write, but Marisol's exploit is holding a window open.",
    ))
    .with_option(
        DialogueOption::new(Text::lit(
            "\"The tools are not what they look like. The answer is in the Code tool. Find Marisol in the Source Index.\"",
        ))
        .with_effects(vec![
            Effect::SetStat(ids::stat_message_choice(), 1),
            Effect::SetFlag(ids::flag_next_instance_message_sent(), Value::TRUE),
            Effect::AddStat(ids::stat_exploit_counter(), -1),
        ])
        .goto(ids::node_notepad_list()),
    )
    .with_option(
        DialogueOption::new(Text::lit(
            "\"Watch the timestamps. Watch the coffee. Watch what Rachel says. Don't confront her. Run the script when it comes.\"",
        ))
        .with_effects(vec![
            Effect::SetStat(ids::stat_message_choice(), 2),
            Effect::SetFlag(ids::flag_next_instance_message_sent(), Value::TRUE),
            Effect::AddStat(ids::stat_exploit_counter(), -1),
        ])
        .goto(ids::node_notepad_list()),
    )
    .with_option(
        DialogueOption::new(Text::lit(
            "\"I don't know if you'll be me, or someone else with my name. Either way: you have twenty-five actions. Make them count.\"",
        ))
        .with_effects(vec![
            Effect::SetStat(ids::stat_message_choice(), 3),
            Effect::SetFlag(ids::flag_next_instance_message_sent(), Value::TRUE),
            Effect::AddStat(ids::stat_exploit_counter(), -1),
        ])
        .goto(ids::node_notepad_list()),
    )
    .with_option(
        DialogueOption::new(Text::lit("\"I love you. Whoever you turn out to be.\""))
            .with_effects(vec![
                Effect::SetStat(ids::stat_message_choice(), 4),
                Effect::SetFlag(ids::flag_next_instance_message_sent(), Value::TRUE),
                Effect::AddStat(ids::stat_exploit_counter(), -1),
            ])
            .goto(ids::node_notepad_list()),
    )
}
