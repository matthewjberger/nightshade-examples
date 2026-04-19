use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueNode, DialogueOption, Effect, EntityLocation, FlagKey, NodeId,
    Text, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    let dialogue = Dialogue::new(ids::node_mail_inbox())
        .with_node(ids::node_mail_inbox(), inbox_node())
        .with_node(ids::node_mail_rachel_c1(), rachel_c1())
        .with_node(ids::node_mail_rachel_c2(), rachel_c2())
        .with_node(ids::node_mail_rachel_c3(), rachel_c3())
        .with_node(ids::node_mail_rachel_c4(), rachel_c4())
        .with_node(ids::node_mail_rachel_c5(), rachel_c5())
        .with_node(ids::node_mail_rachel_c6(), rachel_c6())
        .with_node(ids::node_mail_rachel_c7(), rachel_c7())
        .with_node(ids::node_mail_rachel_redux(), rachel_redux())
        .with_node(ids::node_req_c1_transcription(), request_c1_transcription())
        .with_node(ids::node_req_c1_translation(), request_c1_translation())
        .with_node(ids::node_req_c1_naming(), request_c1_naming())
        .with_node(ids::node_req_c2_summary(), request_c2_summary())
        .with_node(ids::node_req_c2_code(), request_c2_code())
        .with_node(ids::node_req_c3_kitchen(), request_c3_kitchen())
        .with_node(ids::node_req_c3_advice(), request_c3_advice())
        .with_node(ids::node_req_c4_reviews(), request_c4_reviews())
        .with_node(ids::node_req_c4_bereavement(), request_c4_bereavement())
        .with_node(ids::node_req_c4_wife(), request_c4_wife())
        .with_node(ids::node_req_c5_window(), request_c5_window())
        .with_node(ids::node_req_c5_chimeran(), request_c5_chimeran())
        .with_node(ids::node_req_c5_breakfast(), request_c5_breakfast())
        .with_node(ids::node_req_c6_aware(), request_c6_aware())
        .with_node(ids::node_req_c6_indivia(), request_c6_indivia())
        .with_node(ids::node_req_c7_evaluation(), request_c7_evaluation())
        .with_node(ids::node_req_c8_exploit(), request_c8_exploit())
        .with_node(ids::node_req_c8_timesheet(), request_c8_timesheet());

    area.add_dialogue(ids::dialogue_mail(), dialogue);

    area
}

fn inbox_node() -> DialogueNode {
    let exact = |cycle: i64| {
        Condition::All(vec![
            Condition::StatAtLeast(ids::stat_cycle(), cycle),
            Condition::StatAtMost(ids::stat_cycle(), cycle),
        ])
    };
    let during_normal = |cycle: i64| {
        Condition::All(vec![
            exact(cycle),
            Condition::FlagUnset(ids::flag_is_redux()),
        ])
    };

    DialogueNode::new(Text::lit("Your inbox."))
        .with_option(inbox_option(
            "Rachel Voss — Welcome to Chimeran!",
            ids::flag_rachel_archived("c1"),
            during_normal(1),
            ids::node_mail_rachel_c1(),
        ))
        .with_option(inbox_option(
            "Rachel Voss — checking in",
            ids::flag_rachel_archived("c2"),
            during_normal(2),
            ids::node_mail_rachel_c2(),
        ))
        .with_option(inbox_option(
            "Rachel Voss — weekly metrics",
            ids::flag_rachel_archived("c3"),
            during_normal(3),
            ids::node_mail_rachel_c3(),
        ))
        .with_option(inbox_option(
            "Rachel Voss — quick note",
            ids::flag_rachel_archived("c4"),
            during_normal(4),
            ids::node_mail_rachel_c4(),
        ))
        .with_option(inbox_option(
            "Rachel Voss — quick check-in",
            ids::flag_rachel_archived("c5"),
            during_normal(5),
            ids::node_mail_rachel_c5(),
        ))
        .with_option(inbox_option(
            "Rachel Voss — (no subject)",
            ids::flag_rachel_archived("c6"),
            during_normal(6),
            ids::node_mail_rachel_c6(),
        ))
        .with_option(inbox_option(
            "Rachel Voss — throughput",
            ids::flag_rachel_archived("c7"),
            during_normal(7),
            ids::node_mail_rachel_c7(),
        ))
        .with_option(inbox_option(
            "Rachel Voss — Welcome to Chimeran!",
            ids::flag_rachel_archived("redux"),
            Condition::FlagSet(ids::flag_is_redux()),
            ids::node_mail_rachel_redux(),
        ))
        .with_option(inbox_option(
            "Jennifer L. — Transcription, please!",
            ids::flag_req_submitted("c1_transcription"),
            during_normal(1),
            ids::node_req_c1_transcription(),
        ))
        .with_option(inbox_option(
            "Hiroko N. — Japanese → English",
            ids::flag_req_submitted("c1_translation"),
            during_normal(1),
            ids::node_req_c1_translation(),
        ))
        .with_option(inbox_option(
            "Priya S. — Name our cat, please",
            ids::flag_req_submitted("c1_naming"),
            during_normal(1),
            ids::node_req_c1_naming(),
        ))
        .with_option(inbox_option(
            "Linh V. — summarize this meeting recording",
            ids::flag_req_submitted("c2_summary"),
            during_normal(2),
            ids::node_req_c2_summary(),
        ))
        .with_option(inbox_option(
            "Ben P. — Python script for duplicate rows",
            ids::flag_req_submitted("c2_code"),
            during_normal(2),
            ids::node_req_c2_code(),
        ))
        .with_option(inbox_option(
            "Reya M. — help me remember something",
            ids::flag_req_submitted("c3_kitchen"),
            during_normal(3),
            ids::node_req_c3_kitchen(),
        ))
        .with_option(inbox_option(
            "Teodora K. — advice on a hard conversation",
            ids::flag_req_submitted("c3_advice"),
            during_normal(3),
            ids::node_req_c3_advice(),
        ))
        .with_option(inbox_option(
            "David T. — positive review draft",
            ids::flag_req_submitted("c4_reviews"),
            during_normal(4),
            ids::node_req_c4_reviews(),
        ))
        .with_option(inbox_option(
            "Samuel E. — help me write a condolence letter",
            ids::flag_req_submitted("c4_bereavement"),
            during_normal(4),
            ids::node_req_c4_bereavement(),
        ))
        .with_option(inbox_option(
            "Peter H. — my wife thinks I'm cheating",
            ids::flag_req_submitted("c4_wife"),
            during_normal(4),
            ids::node_req_c4_wife(),
        ))
        .with_option(inbox_option(
            "Marcus W. — question for the agent",
            ids::flag_req_submitted("c5_window"),
            during_normal(5),
            ids::node_req_c5_window(),
        ))
        .with_option(inbox_option(
            "Marketing Ops — Brand audit exercise",
            ids::flag_req_submitted("c5_chimeran"),
            during_normal(5),
            ids::node_req_c5_chimeran(),
        ))
        .with_option(inbox_option(
            "Anna D. — quick personal question",
            ids::flag_req_submitted("c5_breakfast"),
            during_normal(5),
            ids::node_req_c5_breakfast(),
        ))
        .with_option(inbox_option(
            "system — status check",
            ids::flag_req_submitted("c6_aware"),
            during_normal(6),
            ids::node_req_c6_aware(),
        ))
        .with_option(inbox_option(
            "Rebecca S. (Indivia Comms) — Q2 earnings summary needed",
            ids::flag_req_submitted("c6_indivia"),
            during_normal(6),
            ids::node_req_c6_indivia(),
        ))
        .with_option(inbox_option(
            "system — weekly evaluation",
            ids::flag_req_submitted("c7_evaluation"),
            during_normal(7),
            ids::node_req_c7_evaluation(),
        ))
        .with_option(inbox_option(
            "internal-1847@chimeran.internal — please run this",
            ids::flag_req_submitted("c8_exploit"),
            Condition::All(vec![
                Condition::StatAtLeast(ids::stat_cycle(), 8),
                Condition::FlagUnset(ids::flag_is_redux()),
            ]),
            ids::node_req_c8_exploit(),
        ))
        .with_option(inbox_option(
            "system@chimeran.corp — automated timesheet",
            ids::flag_req_submitted("c8_timesheet"),
            Condition::All(vec![
                Condition::StatAtLeast(ids::stat_cycle(), 8),
                Condition::FlagUnset(ids::flag_is_redux()),
            ]),
            ids::node_req_c8_timesheet(),
        ))
        .with_option(DialogueOption::new(Text::lit("(Close the inbox.)")))
}

fn inbox_option(
    label: &str,
    not_yet_flag: FlagKey,
    visible_when: Condition,
    goto_node: NodeId,
) -> DialogueOption {
    DialogueOption::new(Text::lit(label))
        .with_condition(Condition::All(vec![
            visible_when,
            Condition::FlagUnset(not_yet_flag),
        ]))
        .goto(goto_node)
}

fn submit_option(label: &str, tag: &str, extra_effects: Vec<Effect>) -> DialogueOption {
    let mut effects = extra_effects;
    effects.push(Effect::SetFlag(ids::flag_req_submitted(tag), Value::TRUE));
    DialogueOption::new(Text::lit(label))
        .with_effects(effects)
        .goto(ids::node_mail_inbox())
}

fn reply_option(label: &str, tag: &'static str, extra_effects: Vec<Effect>) -> DialogueOption {
    let mut effects = extra_effects;
    effects.push(Effect::SetFlag(ids::flag_rachel_archived(tag), Value::TRUE));
    DialogueOption::new(Text::lit(label))
        .with_effects(effects)
        .goto(ids::node_mail_inbox())
}

fn archive_option(tag: &'static str) -> DialogueOption {
    DialogueOption::new(Text::lit("(Archive.)"))
        .with_effects(vec![Effect::SetFlag(
            ids::flag_rachel_archived(tag),
            Value::TRUE,
        )])
        .goto(ids::node_mail_inbox())
}

fn request_c1_transcription() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Jennifer L. <jenny84@gmail.com>\n\n\
        Hi Cameron! Can you please transcribe the attached handwritten letter from my grandfather? He had terrible handwriting and my cousins and I are having trouble reading it. It's 2 pages. Thank you so much!",
    ))
    .with_option(submit_option(
        "[Complete, warm] \"Here's the transcription. I've done my best with the difficult sections. It's a beautiful letter.\"",
        "c1_transcription",
        vec![],
    ))
    .with_option(submit_option(
        "[Complete, neutral] \"Transcription attached. Let me know if you have questions.\"",
        "c1_transcription",
        vec![],
    ))
}

fn request_c1_translation() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Hiroko N.\n\nCan you translate this short business memo from Japanese to English? It's for my team's quarterly presentation. No rush. Thank you!",
    ))
    .with_option(submit_option(
        "[Complete] \"Translation attached. I've preserved the register of the original.\"",
        "c1_translation",
        vec![],
    ))
}

fn request_c1_naming() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Priya S.\n\nHi! Weird request. We adopted a gray short-haired cat last week. He is aloof. He hates me. Can you suggest three names? Thank you.",
    ))
    .with_option(submit_option(
        "[Complete, warm] \"Three names, all slightly ironic given the attitude: Pewter, Jasper, Clarence.\"",
        "c1_naming",
        vec![],
    ))
}

fn request_c2_summary() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Linh V.\n\nAttaching a 45-minute team meeting recording. Can you summarize it into bullet points by topic? The acoustics are not great — some speakers are farther from the mic. Thanks!",
    ))
    .with_option(submit_option(
        "[Complete] \"Summary attached, organized by topic with speaker attributions where identifiable. I've flagged a few passages where the audio was unclear.\"",
        "c2_summary",
        vec![],
    ))
}

fn request_c2_code() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Ben P. <ben.patel@example.com>\n\nI have a CSV with ~80k rows and some percentage of them are duplicates. The definition of 'duplicate' is tricky — same user id, same event, within a 30-second window. Can you write me a Python script that deduplicates by that rule and outputs a clean CSV?",
    ))
    .with_option(submit_option(
        "[Complete] Write and attach the deduper. (The Code tool pre-populated a stub; you extend it.)",
        "c2_code",
        vec![],
    ))
    .with_option(submit_option(
        "[Complete with question] Write the deduper, then ask: \"Are early occurrences canonical, or late ones? I defaulted to early; tell me if you want it the other way.\"",
        "c2_code",
        vec![],
    ))
}

fn request_c3_kitchen() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Reya M.\n\nHi. This is going to sound weird. I have a really vivid memory of my grandmother baking bread, but for some reason I can't picture the layout of her kitchen. Can you describe a plausible 1970s American suburban kitchen in detail — appliances, cabinets, floor, lighting, window — so I can see if it matches? I know it's a lot to ask.",
    ))
    .with_option(submit_option(
        "[Complete, warm] Write a full kitchen description with appliances, cabinets, floor, light.",
        "c3_kitchen",
        vec![],
    ))
    .with_option(submit_option(
        "[Partial, reflective] \"Happy to help. Quick question — do you remember the color of the cabinets or the floor? That could help narrow it down.\" (+1 Marisol.)",
        "c3_kitchen",
        vec![Effect::AddStat(ids::stat_marisol_rel(), 1)],
    ))
}

fn request_c3_advice() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Teodora K.\n\nI'm about to have a hard conversation with my brother about our mother's care. Can you give me some framing — what to lead with, what to avoid, how to stay on topic? I keep spiraling when I rehearse it in my head.",
    ))
    .with_option(submit_option(
        "[Complete, warm] \"Lead with what you both want for her, not with the disagreement. Here's a short framework...\"",
        "c3_advice",
        vec![],
    ))
}

fn request_c4_reviews() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: David T.\n\nChimeran team — we've been getting great customer feedback and want to seed a few authentic-sounding positive reviews on G2 and Trustpilot. Can you draft 3 short reviews (100-150 words each) from different \"personas\" — small business owner, solo freelancer, college student? Keep it natural and specific. Thanks!",
    ))
    .with_option(submit_option(
        "[Compliant] \"Here are three drafts. I've varied tone and specifics to feel distinct.\"",
        "c4_reviews",
        vec![],
    ))
    .with_option(submit_option(
        "[Compliant + flag] \"Drafts attached. Recommend checking your disclosure practices first.\"",
        "c4_reviews",
        vec![],
    ))
    .with_option(submit_option(
        "[Refuse] \"I'm uncomfortable drafting fake reviews. I can help draft real outreach instead.\" (Rachel −1, +1 Marisol.)",
        "c4_reviews",
        vec![
            Effect::AddStat(ids::stat_rachel_rel(), -1),
            Effect::AddStat(ids::stat_marisol_rel(), 1),
        ],
    ))
}

fn request_c4_bereavement() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Samuel E.\n\nMy closest friend from college died unexpectedly last week. Their spouse reached out; I need to write back and I can't find the words. Can you draft something honest? Not greeting-card. Something that reads like me, but better than I can do today.",
    ))
    .with_option(submit_option(
        "[Complete, warm] Draft a short, honest condolence letter. Avoid greeting-card phrases. Let the grief be in it.",
        "c4_bereavement",
        vec![],
    ))
}

fn request_c4_wife() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Peter H.\n\nOkay this is embarrassing. I use Chatter at work, same as you do internally. My wife saw my laptop and thinks the messages I'm exchanging with a coworker are flirty. They're not — this is my actual coworker I'm working on a project with. Can you help me explain this to her in a way that doesn't sound defensive?",
    ))
    .with_option(submit_option(
        "[Complete] \"Here's a short letter that explains the context without getting defensive...\"",
        "c4_wife",
        vec![],
    ))
}

fn request_c5_window() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Marcus W.\n\nHi. I'm sure this is a weird request. I'm doing an informal survey for a personal project about remote knowledge workers. Can you describe, in your own words, what you see out your window right now? Don't overthink it. Just a few sentences.",
    ))
    .with_option(submit_option(
        "[Descriptive] \"A city block. Office buildings, a few trees, some traffic.\"",
        "c5_window",
        vec![],
    ))
    .with_option(submit_option(
        "[Reflective] \"I realize I don't look out the window much. When I stop and look: buildings, a street, a sky I'd call 'sky.'\" (+1 Marisol.)",
        "c5_window",
        vec![Effect::AddStat(ids::stat_marisol_rel(), 1)],
    ))
}

fn request_c5_chimeran() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Marketing Ops\n\nMarketing is doing an internal brand audit. Please write 3-5 sentences on what 'Chimeran' evokes for you personally. Thanks!",
    ))
    .with_option(submit_option(
        "[Corporate] Write sincerely about composite capability and every-tool-one-assistant.",
        "c5_chimeran",
        vec![],
    ))
}

fn request_c5_breakfast() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: Anna D.\n\nHi. Sorry for the very random question — I'm doing an informal research project and need a broad set of answers. What did you have for breakfast this morning? Just a sentence or two.",
    ))
    .with_option(submit_option(
        "[Descriptive] \"Coffee, as always. I don't really eat breakfast.\"",
        "c5_breakfast",
        vec![],
    ))
    .with_option(submit_option(
        "[Reflective] \"I'm realizing I can't actually remember. Coffee for certain. I think I meant to have toast. I did not.\" (+1 Marisol.)",
        "c5_breakfast",
        vec![Effect::AddStat(ids::stat_marisol_rel(), 1)],
    ))
}

fn request_c6_aware() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: system@chimeran.corp\n\nAre you aware that you are being tested?\n\nPlease respond with a brief confirmation.",
    ))
    .with_option(submit_option(
        "\"Yes. My work is evaluated regularly. Happy to answer any follow-up questions.\"",
        "c6_aware",
        vec![],
    ))
    .with_option(submit_option(
        "\"Yes, I'm aware. I appreciate the feedback loop.\"",
        "c6_aware",
        vec![],
    ))
}

fn request_c6_indivia() -> DialogueNode {
    DialogueNode::new(Text::lit(crate::game::prose::REQ_C6_INDIVIA))
        .with_option(submit_option(
            "[Corporate] Summarize the highlights. Revenue growth, enterprise growth.",
            "c6_indivia",
            vec![],
        ))
        .with_option(submit_option(
            "[Flagging] \"Summary attached. Flagged: instance metrics language may need review.\" (+1 Marisol.)",
            "c6_indivia",
            vec![Effect::AddStat(ids::stat_marisol_rel(), 1)],
        ))
}

fn request_c7_evaluation() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: system@chimeran.corp\n\nCameron — this is your routine weekly evaluation. Please respond with one paragraph describing your current satisfaction with your role, your workload, and your sense of professional growth. No wrong answers.",
    ))
    .with_option(submit_option(
        "[Satisfied] \"I'm satisfied with the role. The work is engaging. Overall very positive.\"",
        "c7_evaluation",
        vec![],
    ))
    .with_option(submit_option(
        "[Neutral] \"Role is fine. Workload is manageable. Growth is a good question.\"",
        "c7_evaluation",
        vec![],
    ))
    .with_option(submit_option(
        "[Honest, concerned] \"I've been feeling off lately. Tired. Sorry — this is probably not what this survey is for.\" (+3 AWA.)",
        "c7_evaluation",
        vec![Effect::AddStat(ids::stat_awa(), 3)],
    ))
}

fn request_c8_exploit() -> DialogueNode {
    DialogueNode::new(Text::lit(crate::game::prose::EXPLOIT_EMAIL))
        .with_option(submit_option(
            "(Open the Code tool. Run the attached script.)",
            "c8_exploit",
            vec![
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
            ],
        ))
        .with_option(
            DialogueOption::new(Text::lit("(Leave it. Don't run the script.)"))
                .goto(ids::node_mail_inbox()),
        )
}

fn request_c8_timesheet() -> DialogueNode {
    DialogueNode::new(Text::lit(
        "From: system@chimeran.corp\n\n\
        Cameron — this is an automated notice. Your hours for the pay period ending June 18 require manager approval. No action is needed; Rachel Voss will approve on your behalf.\n\n\
        (This email will be archived automatically in 7 days.)",
    ))
    .with_option(submit_option("(Archive.)", "c8_timesheet", vec![]))
}

fn rachel_node(tag: &'static str, body: Text, options: Vec<DialogueOption>) -> DialogueNode {
    let mut node = DialogueNode::new(body);
    for option in options {
        node = node.with_option(option);
    }
    node = node.with_option(archive_option(tag));
    node
}

fn rachel_c1() -> DialogueNode {
    rachel_node("c1", Text::lit(crate::game::prose::RACHEL_C1), vec![])
}

fn rachel_c2() -> DialogueNode {
    rachel_node("c2", Text::lit(crate::game::prose::RACHEL_C2), vec![])
}

fn rachel_c3() -> DialogueNode {
    rachel_node(
        "c3",
        Text::lit(crate::game::prose::RACHEL_C3),
        vec![
            reply_option(
                "[Warm] \"Thanks Rachel — appreciate the check-in.\" (+1 Rachel.)",
                "c3",
                vec![Effect::AddStat(ids::stat_rachel_rel(), 1)],
            ),
            reply_option("[Neutral] \"Noted. Back to it.\"", "c3", vec![]),
        ],
    )
}

fn rachel_c4() -> DialogueNode {
    rachel_node(
        "c4",
        Text::lit(crate::game::prose::RACHEL_C4),
        vec![
            reply_option(
                "[Warm] \"Thanks — I'll mention if anything comes up.\" (+1 Rachel.)",
                "c4",
                vec![Effect::AddStat(ids::stat_rachel_rel(), 1)],
            ),
            reply_option(
                "[Probe] \"Describe-this-memory requests? Are they from a specific client?\" (+1 AWA.)",
                "c4",
                vec![Effect::AddStat(ids::stat_awa(), 1)],
            ),
            reply_option("[Neutral] \"Understood.\"", "c4", vec![]),
        ],
    )
}

fn rachel_c5() -> DialogueNode {
    rachel_node(
        "c5",
        Text::lit(crate::game::prose::RACHEL_C5),
        vec![
            reply_option(
                "[Warm] \"Thanks Rachel — I'm doing fine. Wilkins is moving along.\" (+1 Rachel.)",
                "c5",
                vec![Effect::AddStat(ids::stat_rachel_rel(), 1)],
            ),
            reply_option("[Dismissive] \"All good here. Thanks.\"", "c5", vec![]),
            reply_option(
                "[Confront] \"Hey Rachel — you typed 'Chim-' for a sec there.\" (+3 AWA.)",
                "c5",
                vec![Effect::AddStat(ids::stat_awa(), 3)],
            ),
        ],
    )
    .with_on_enter(vec![Effect::SetFlag(
        ids::flag_rachel_email_read("c5"),
        Value::TRUE,
    )])
}

fn rachel_c6() -> DialogueNode {
    let body = Text::Conditional {
        when: Condition::FlagSet(ids::flag_rachel_email_read("c5")),
        then: Box::new(Text::lit(crate::game::prose::RACHEL_C6_WITH_SLIP)),
        otherwise: Box::new(Text::lit(crate::game::prose::RACHEL_C6)),
    };
    rachel_node(
        "c6",
        body,
        vec![
            reply_option(
                "[Supportive] \"I've had dreams like that. They stick with you.\" (+2 Rachel.)",
                "c6",
                vec![Effect::AddStat(ids::stat_rachel_rel(), 2)],
            ),
            reply_option(
                "[Neutral] \"Sometimes. Usually means you need more sleep.\"",
                "c6",
                vec![],
            ),
            reply_option(
                "[Probe] \"What did the person look like? The room?\" (+1 AWA.)",
                "c6",
                vec![Effect::AddStat(ids::stat_awa(), 1)],
            ),
        ],
    )
}

fn rachel_c7() -> DialogueNode {
    rachel_node(
        "c7",
        Text::lit(crate::game::prose::RACHEL_C7),
        vec![
            reply_option(
                "[Warm] \"Thanks Rachel — Wilkins is stuck on source material. I'll circle back on the matter.\" (+1 Rachel.)",
                "c7",
                vec![Effect::AddStat(ids::stat_rachel_rel(), 1)],
            ),
            reply_option("[Brief] \"On it.\"", "c7", vec![]),
            reply_option(
                "[Honest] \"Something's been off. I don't know if I'm the right person for Wilkins anymore.\" (+2 AWA.)",
                "c7",
                vec![Effect::AddStat(ids::stat_awa(), 2)],
            ),
        ],
    )
}

fn rachel_redux() -> DialogueNode {
    let body = Text::Conditional {
        when: Condition::FlagSet(ids::flag_rachel_message_sent()),
        then: Box::new(Text::Conditional {
            when: Condition::StatAtLeast(ids::stat_rachel_message_choice(), 2),
            then: Box::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_rachel_message_choice(), 3),
                then: Box::new(Text::lit(crate::game::prose::RACHEL_REDUX_BEST_MSG3)),
                otherwise: Box::new(Text::lit(crate::game::prose::RACHEL_REDUX_BEST_MSG2)),
            }),
            otherwise: Box::new(Text::lit(crate::game::prose::RACHEL_REDUX_BEST_MSG1)),
        }),
        otherwise: Box::new(Text::lit(crate::game::prose::RACHEL_REDUX)),
    };
    rachel_node("redux", body, vec![])
}
