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
                    then: Box::new(Text::lit(crate::game::prose::EXPLOIT_OUTPUT)),
                    otherwise: Box::new(Text::Conditional {
                        when: Condition::StatAtLeast(ids::stat_cycle(), 8),
                        then: Box::new(Text::lit(
                            "The Code tool. Two files in the tree: the old helper.py you never ran, and a new file — check.py — loaded from the attachment in internal-1847@chimeran.internal. The editor shows a few lines of Python you don't immediately recognize. There is a Run button.",
                        )),
                        otherwise: Box::new(Text::Conditional {
                            when: Condition::StatAtLeast(ids::stat_cycle(), 3),
                            then: Box::new(Text::lit(
                                "The Code tool. The file tree on the left shows a scratch file and one attachment you opened but did not run: helper.py. The editor marks its lines as unexecuted. An output panel hums at the bottom.",
                            )),
                            otherwise: Box::new(Text::lit(
                                "The Code tool. The file tree on the left shows the scratch file you last used. The editor is empty. An output panel hums at the bottom.",
                            )),
                        }),
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
                            Effect::MoveEntity(
                                ids::fixture_picture_frame(),
                                EntityLocation::Room(ids::room_desk()),
                            ),
                            Effect::Say(Text::lit(crate::game::prose::EXPLOIT_OUTPUT)),
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
                DialogueNode::new(Text::lit(crate::game::prose::EXPLOIT_OUTPUT))
                    .with_option(DialogueOption::new(Text::lit("(Back.)")).goto(ids::node_code_after_exploit())),
            ),
    );

    area
}
