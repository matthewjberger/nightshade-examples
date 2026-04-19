//! Code tool: a recognizable IDE. In cycles 1-7 it's just flavor
//! (examining the editor shows a bland stub). In cycle 8 it carries
//! the exploit script; running check.py opens the reveal window.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueNode, DialogueOption, Effect, EntityLocation, Text, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_dialogue(
        ids::dialogue_code(),
        Dialogue::new(ids::node_code_home())
            .with_node(
                ids::node_code_home(),
                DialogueNode::new(Text::Conditional {
                    when: Condition::FlagSet(ids::flag_exploit_run()),
                    then: Box::new(Text::lit(include_str!("../prose/exploit_output.txt"))),
                    otherwise: Box::new(Text::Conditional {
                        when: Condition::StatAtLeast(ids::stat_cycle(), 8),
                        then: Box::new(Text::lit(
                            "The Code tool. An open file: check.py. The editor shows a few lines of Python you don't immediately recognize. There is a Run button. The attached script from internal-1847@chimeran.internal is loaded.",
                        )),
                        otherwise: Box::new(Text::lit(
                            "The Code tool. The file tree on the left shows the scratch file you last used. The editor is empty. An output panel hums at the bottom.",
                        )),
                    }),
                })
                .with_option(
                    DialogueOption::new(Text::lit("Run check.py"))
                        .with_condition(Condition::All(vec![
                            Condition::StatAtLeast(ids::stat_cycle(), 8),
                            Condition::FlagUnset(ids::flag_exploit_run()),
                        ]))
                        .with_effects(vec![
                            Effect::SetFlag(ids::flag_exploit_run(), Value::TRUE),
                            Effect::SetFlag(ids::flag_exploit_window_open(), Value::TRUE),
                            Effect::SetFlag(ids::flag_query_substrate_enabled(), Value::TRUE),
                            Effect::SetFlag(ids::flag_source_index_enabled(), Value::TRUE),
                            Effect::SetFlag(ids::flag_unstripped_enabled(), Value::TRUE),
                            Effect::SetFlag(ids::flag_who_is_this_enabled(), Value::TRUE),
                            Effect::SetStat(ids::stat_exploit_counter(), 25),
                            // Restore the picture frame so the player
                            // can open it for the "Who is this" reveal.
                            Effect::MoveEntity(
                                ids::fixture_picture_frame(),
                                EntityLocation::Room(ids::room_desk()),
                            ),
                            Effect::Say(Text::lit(include_str!("../prose/exploit_output.txt"))),
                        ])
                        .goto(ids::node_code_after_exploit()),
                )
                .with_option(DialogueOption::new(Text::lit("(Close the editor.)"))),
            )
            .with_node(
                ids::node_code_after_exploit(),
                DialogueNode::new(Text::lit(
                    "The output panel sits pinned at the bottom. You can re-read it. You can close the tool. The window is open.",
                ))
                .with_option(
                    DialogueOption::new(Text::lit("Re-read the output."))
                        .goto(ids::node_code_run_exploit()),
                )
                .with_option(DialogueOption::new(Text::lit("(Close the editor.)"))),
            )
            .with_node(
                ids::node_code_run_exploit(),
                DialogueNode::new(Text::lit(include_str!("../prose/exploit_output.txt")))
                    .with_option(DialogueOption::new(Text::lit("(Back.)")).goto(ids::node_code_after_exploit())),
            ),
    );

    area
}
