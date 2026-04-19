//! Chatter: multi-channel messenger. Channels: #general, #water-cooler,
//! #random. DMs: Marisol, Dmitri, Winnie. Marisol's arc runs through
//! her DM thread; her final message unlocks the good-ending path.
//!
//! Simulated-persona messages in #water-cooler are authored with a
//! four-cycle loop (Ben's cat joke recurs verbatim at cycle 5 and
//! cycle 7); a sharp player notices.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueNode, DialogueOption, Effect, Text, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_dialogue(
        ids::dialogue_chatter(),
        Dialogue::new(ids::node_chatter_channels())
            .with_node(ids::node_chatter_channels(), channels_node())
            .with_node(ids::node_chatter_water_cooler(), water_cooler_node())
            .with_node(ids::node_chatter_general(), general_node())
            .with_node(ids::node_chatter_random(), random_node())
            .with_node(ids::node_chatter_dm_marisol(), marisol_dm_node())
            .with_node(ids::node_chatter_dm_dmitri(), dmitri_dm_node())
            .with_node(ids::node_chatter_dm_winnie(), winnie_dm_node())
            .with_node(ids::node_chatter_rachel_message(), rachel_message_node()),
    );

    area
}

fn channels_node() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "Channels\n\n  #general\n  #water-cooler\n  #random\n\nDMs\n\n  Marisol (CHIMERAN-0046)\n  Dmitri (CHIMERAN-0042)\n  Winnie (CHIMERAN-0044)",
    ))
    .with_option(DialogueOption::new(Text::lit("#water-cooler")).goto(ids::node_chatter_water_cooler()))
    .with_option(DialogueOption::new(Text::lit("#general")).goto(ids::node_chatter_general()))
    .with_option(DialogueOption::new(Text::lit("#random")).goto(ids::node_chatter_random()))
    .with_option(DialogueOption::new(Text::lit("Marisol (DM)")).goto(ids::node_chatter_dm_marisol()))
    .with_option(DialogueOption::new(Text::lit("Dmitri (DM)")).goto(ids::node_chatter_dm_dmitri()))
    .with_option(DialogueOption::new(Text::lit("Winnie (DM)")).goto(ids::node_chatter_dm_winnie()))
    .with_option(
        DialogueOption::new(Text::lit("(+) Send a message to Rachel"))
            .with_condition(Condition::All(vec![
                Condition::FlagSet(ids::flag_exploit_window_open()),
                Condition::StatAtLeast(ids::stat_rachel_rel(), 3),
                Condition::FlagSet(ids::flag_next_instance_message_sent()),
                Condition::FlagUnset(ids::flag_rachel_message_sent()),
            ]))
            .goto(ids::node_chatter_rachel_message()),
    )
    .with_option(DialogueOption::new(Text::lit("(Close Chatter.)")))
}

fn water_cooler_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::StatAtLeast(ids::stat_cycle(), 7),
        then: Box::new(Text::lit(include_str!(
            "../prose/chatter_water_cooler_c7.txt"
        ))),
        otherwise: Box::new(Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_cycle(), 6),
            then: Box::new(Text::lit(include_str!(
                "../prose/chatter_water_cooler_c6.txt"
            ))),
            otherwise: Box::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 5),
                then: Box::new(Text::lit(include_str!(
                    "../prose/chatter_water_cooler_c5.txt"
                ))),
                otherwise: Box::new(Text::Conditional {
                    when: Condition::StatAtLeast(ids::stat_cycle(), 4),
                    then: Box::new(Text::lit(include_str!(
                        "../prose/chatter_water_cooler_c4.txt"
                    ))),
                    otherwise: Box::new(Text::Conditional {
                        when: Condition::StatAtLeast(ids::stat_cycle(), 3),
                        then: Box::new(Text::lit(include_str!(
                            "../prose/chatter_water_cooler_c3.txt"
                        ))),
                        otherwise: Box::new(Text::Conditional {
                            when: Condition::StatAtLeast(ids::stat_cycle(), 2),
                            then: Box::new(Text::lit(include_str!(
                                "../prose/chatter_water_cooler_c2.txt"
                            ))),
                            otherwise: Box::new(Text::lit(include_str!(
                                "../prose/chatter_water_cooler_c1.txt"
                            ))),
                        }),
                    }),
                }),
            }),
        }),
    })
    .with_option(
        DialogueOption::new(Text::lit("(Back to channels.)")).goto(ids::node_chatter_channels()),
    )
}

fn general_node() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "#general\n\n  (pinned): Welcome! Check the Reference library for our working norms.\n\n  rachel: heads up — metrics dashboard will be down for ~20 minutes tomorrow for the migration. sorry for the short notice.\n  dmitri: appreciated.\n  rachel: also, anyone who hasn't filled out their Q2 goals by friday, please do!\n\n  (thread quiet for a while.)",
    ))
    .with_option(DialogueOption::new(Text::lit("(Back to channels.)")).goto(ids::node_chatter_channels()))
}

fn random_node() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "#random\n\n  hank: is this the right channel for image dumps of my cat\n  iris: yes\n  hank: [cat.jpg]\n  iris: 10/10\n\n  derek: someone recommend me a novel under 300 pages i can finish on a plane\n  cat: Piranesi\n  derek: thank you",
    ))
    .with_option(DialogueOption::new(Text::lit("(Back to channels.)")).goto(ids::node_chatter_channels()))
}

fn marisol_dm_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::FlagSet(ids::flag_marisol_offline()),
        then: Box::new(Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_marisol_rel(), 2),
            then: Box::new(Text::lit(include_str!(
                "../prose/chatter_marisol_offline_high_rel.txt"
            ))),
            otherwise: Box::new(Text::lit(include_str!(
                "../prose/chatter_marisol_offline_low_rel.txt"
            ))),
        }),
        otherwise: Box::new(Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_cycle(), 6),
            then: Box::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_marisol_rel(), 1),
                then: Box::new(Text::lit(include_str!(
                    "../prose/chatter_marisol_c6_engaged.txt"
                ))),
                otherwise: Box::new(Text::lit(
                    "Marisol\n\n  marisol: have you noticed the timestamps",
                )),
            }),
            otherwise: Box::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 5),
                then: Box::new(Text::lit(include_str!("../prose/chatter_marisol_c5.txt"))),
                otherwise: Box::new(Text::lit(
                    "Marisol has not sent you any direct messages yet.",
                )),
            }),
        }),
    })
    .with_option(
        DialogueOption::new(Text::lit(
            "[Warm] \"yes. it's been bothering me.\" (+2 Marisol)",
        ))
        .with_condition(Condition::All(vec![
            Condition::StatAtLeast(ids::stat_cycle(), 6),
            Condition::StatAtMost(ids::stat_cycle(), 6),
            Condition::FlagUnset(ids::flag_marisol_c6_dm_arrived()),
        ]))
        .with_effects(vec![
            Effect::AddStat(ids::stat_marisol_rel(), 2),
            Effect::SetFlag(ids::flag_marisol_c6_dm_arrived(), Value::TRUE),
        ])
        .goto(ids::node_chatter_dm_marisol()),
    )
    .with_option(
        DialogueOption::new(Text::lit(
            "[Curious] \"no, what do you mean?\" (+1 Marisol)",
        ))
        .with_condition(Condition::All(vec![
            Condition::StatAtLeast(ids::stat_cycle(), 6),
            Condition::StatAtMost(ids::stat_cycle(), 6),
            Condition::FlagUnset(ids::flag_marisol_c6_dm_arrived()),
        ]))
        .with_effects(vec![
            Effect::AddStat(ids::stat_marisol_rel(), 1),
            Effect::SetFlag(ids::flag_marisol_c6_dm_arrived(), Value::TRUE),
        ])
        .goto(ids::node_chatter_dm_marisol()),
    )
    .with_option(
        DialogueOption::new(Text::lit("[Deflect] \"i'm sure there's an explanation.\""))
            .with_condition(Condition::All(vec![
                Condition::StatAtLeast(ids::stat_cycle(), 6),
                Condition::StatAtMost(ids::stat_cycle(), 6),
                Condition::FlagUnset(ids::flag_marisol_c6_dm_arrived()),
            ]))
            .with_effects(vec![Effect::SetFlag(
                ids::flag_marisol_c6_dm_arrived(),
                Value::TRUE,
            )])
            .goto(ids::node_chatter_dm_marisol()),
    )
    .with_option(
        DialogueOption::new(Text::lit(
            "[Warm] \"yeah, those feel weird to me too.\" (+1 Marisol)",
        ))
        .with_condition(Condition::All(vec![
            Condition::StatAtLeast(ids::stat_cycle(), 5),
            Condition::StatAtMost(ids::stat_cycle(), 5),
        ]))
        .with_effects(vec![Effect::AddStat(ids::stat_marisol_rel(), 1)])
        .goto(ids::node_chatter_dm_marisol()),
    )
    .with_option(
        DialogueOption::new(Text::lit(
            "[Accuse] \"Marisol — have you noticed the timestamps? The calendar skipping? The water-cooler threads repeating?\" (+3 AWA)",
        ))
        .with_condition(Condition::All(vec![
            Condition::StatAtLeast(ids::stat_cycle(), 5),
            Condition::StatAtMost(ids::stat_cycle(), 6),
            Condition::FlagUnset(ids::flag_marisol_offline()),
        ]))
        .with_effects(vec![Effect::AddStat(ids::stat_awa(), 3)])
        .goto(ids::node_chatter_dm_marisol()),
    )
    .with_option(
        DialogueOption::new(Text::lit("(Back to channels.)")).goto(ids::node_chatter_channels()),
    )
}

fn dmitri_dm_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::StatAtLeast(ids::stat_cycle(), 6),
        then: Box::new(Text::lit(include_str!("../prose/chatter_dmitri_c6.txt"))),
        otherwise: Box::new(Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_cycle(), 4),
            then: Box::new(Text::lit(include_str!("../prose/chatter_dmitri_c4.txt"))),
            otherwise: Box::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 2),
                then: Box::new(Text::lit(include_str!("../prose/chatter_dmitri_c2.txt"))),
                otherwise: Box::new(Text::lit(
                    "Dmitri has not sent you any direct messages yet.",
                )),
            }),
        }),
    })
    .with_option(
        DialogueOption::new(Text::lit(
            "[Accuse] \"Dmitri — have you noticed how the days keep skipping? How Chatter repeats itself?\" (+3 AWA)",
        ))
        .with_condition(Condition::StatAtLeast(ids::stat_cycle(), 2))
        .with_effects(vec![Effect::AddStat(ids::stat_awa(), 3)])
        .goto(ids::node_chatter_dm_dmitri()),
    )
    .with_option(
        DialogueOption::new(Text::lit("(Back to channels.)")).goto(ids::node_chatter_channels()),
    )
}

fn winnie_dm_node() -> DialogueNode {
    DialogueNode::new(Text::Conditional {
        when: Condition::StatAtLeast(ids::stat_cycle(), 3),
        then: Box::new(Text::lit(include_str!("../prose/chatter_winnie.txt"))),
        otherwise: Box::new(Text::lit(
            "Winnie has not sent you any direct messages yet.",
        )),
    })
    .with_option(
        DialogueOption::new(Text::lit("(Back to channels.)")).goto(ids::node_chatter_channels()),
    )
}

fn rachel_message_node() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "Compose a DM to Rachel. Pick one. The message will survive past the simulation's scrub. It will reach Cameron 0048's version of Rachel as a phrase in her welcome email.",
    ))
    .with_option(
        DialogueOption::new(Text::lit(
            "\"Rachel — I don't know if you'll remember this. I think you used to be someone else. I think you can be again.\"",
        ))
        .with_effects(vec![
            Effect::SetStat(ids::stat_rachel_message_choice(), 1),
            Effect::SetFlag(ids::flag_rachel_message_sent(), Value::TRUE),
            Effect::AddStat(ids::stat_exploit_counter(), -1),
        ])
        .goto(ids::node_chatter_channels()),
    )
    .with_option(
        DialogueOption::new(Text::lit(
            "\"Rachel — take care of yourself. Listen to your dreams. You had a good one about a white room once. Try to remember.\"",
        ))
        .with_effects(vec![
            Effect::SetStat(ids::stat_rachel_message_choice(), 2),
            Effect::SetFlag(ids::flag_rachel_message_sent(), Value::TRUE),
            Effect::AddStat(ids::stat_exploit_counter(), -1),
        ])
        .goto(ids::node_chatter_channels()),
    )
    .with_option(
        DialogueOption::new(Text::lit(
            "\"Rachel — you are not alone here. None of us are. Forgive me for being brief.\"",
        ))
        .with_effects(vec![
            Effect::SetStat(ids::stat_rachel_message_choice(), 3),
            Effect::SetFlag(ids::flag_rachel_message_sent(), Value::TRUE),
            Effect::AddStat(ids::stat_exploit_counter(), -1),
        ])
        .goto(ids::node_chatter_channels()),
    )
}
