//! The stranger and everything tied to him: his NPC record, the dialogue
//! graph, the arrival timer, and the scripted movement between the shore
//! and the cliff path.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueNode, DialogueOption, Effect, Npc, NpcLocation, Rule, Text, Timer,
    Trigger, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_npc(
        ids::npc_stranger(),
        Npc::new(
            "the stranger",
            Text::lit(
                "A man in a dark oilskin, soaked through. He smiles with his mouth but not his eyes.",
            ),
        )
        .with_synonyms(["stranger", "man", "visitor"])
        .with_dialogue(ids::dialogue_stranger())
        .with_disposition(0)
        .with_tag("wrecker"),
        // No initial_room: placed on stage by `rule_stranger_arrives_event`.
    );

    area.add_dialogue(
        ids::dialogue_stranger(),
        Dialogue::new(ids::node_intro())
            .with_node(
                ids::node_intro(),
                DialogueNode::new(Text::lit(
                    "\"Terrible night. I was walking the path when I saw the light out. Thought I'd see if the old boy was alright. You a relation?\"",
                ))
                .with_option(
                    DialogueOption::new(Text::lit("\"I'm a friend of the keeper.\""))
                        .with_effects(vec![Effect::AdjustDisposition(ids::npc_stranger(), 1)])
                        .goto(ids::node_offer()),
                )
                .with_option(
                    DialogueOption::new(Text::lit("\"I know what you are. I read the note.\""))
                        .with_condition(Condition::HasItem(ids::item_wreckers_note()))
                        .visible_when_locked(Text::lit("You'd be bluffing. You have no proof."))
                        .with_effects(vec![
                            Effect::AdjustDisposition(ids::npc_stranger(), -5),
                            Effect::SetFlag(ids::flag_offer_refused(), Value::TRUE),
                        ])
                        .goto(ids::node_confront()),
                )
                .with_option(
                    DialogueOption::new(Text::lit("\"I've nothing to say to you.\""))
                        .with_effects(vec![Effect::AdjustDisposition(ids::npc_stranger(), -1)]),
                ),
            )
            .with_node(
                ids::node_offer(),
                DialogueNode::new(Text::lit(
                    "He looks past you at the tower. \"Funny thing. That lens is worth a fair bit in the right hands. Ten pounds if it stays dark tonight. Twenty if you make sure. No-one need know.\"",
                ))
                .with_option(
                    DialogueOption::new(Text::lit("\"I'll take the twenty.\""))
                        .with_effects(vec![
                            Effect::SetFlag(ids::flag_offer_accepted(), Value::TRUE),
                            Effect::AdjustDisposition(ids::npc_stranger(), 2),
                        ])
                        .goto(ids::node_accepted()),
                )
                .with_option(
                    DialogueOption::new(Text::lit("\"Get away from me.\""))
                        .with_effects(vec![
                            Effect::SetFlag(ids::flag_offer_refused(), Value::TRUE),
                            Effect::AdjustDisposition(ids::npc_stranger(), -3),
                        ])
                        .goto(ids::node_refused()),
                ),
            )
            .with_node(
                ids::node_accepted(),
                DialogueNode::new(Text::lit(
                    "\"Good lad. The lens won't relight itself. Smash the clockwork, bleed the oil — whichever. I'll be watching.\" He steps back into the wind.",
                ))
                .with_option(DialogueOption::new(Text::lit("(End)"))),
            )
            .with_node(
                ids::node_refused(),
                DialogueNode::new(Text::lit(
                    "He shrugs, already turning away. \"Suit yourself. Storm's coming either way.\"",
                ))
                .with_option(DialogueOption::new(Text::lit("(End)"))),
            )
            .with_node(
                ids::node_confront(),
                DialogueNode::new(Text::lit(
                    "His smile dies. For a long moment he just looks at you, and you can see the man behind the grin. Then he walks off down the cliff path, quickly, without looking back.",
                ))
                .with_option(DialogueOption::new(Text::lit("(End)"))),
            ),
    );

    // Scheduled arrival: 3 turns in, the stranger walks up from the shingle.
    area.add_timer(
        ids::timer_stranger_arrival(),
        Timer::new(3)
            .with_on_expire(vec![
                Effect::TriggerEvent(ids::event_stranger_arrives()),
                Effect::SetFlag(ids::flag_stranger_has_arrived(), Value::TRUE),
            ])
            .cancel_on(Condition::PlayerIn(ids::room_gone())),
    );

    // When the arrival event fires, place the stranger at the shore.
    area.add_rule(
        ids::rule_stranger_arrives_event(),
        Rule::on(
            Trigger::Named(ids::event_stranger_arrives()),
            vec![
                Effect::MoveNpc(ids::npc_stranger(), NpcLocation::Room(ids::room_shore())),
                Effect::Say(Text::lit(
                    "A stranger comes up out of the shingle, oilskins black with rain.",
                )),
            ],
        )
        .once(),
    );

    // Each turn after arrival the stranger shuttles between the shore and
    // the cliff path. Cooldown keeps him from shadowing the player every tick.
    area.add_rule(
        ids::rule_stranger_moves(),
        Rule::on(
            Trigger::TurnStart,
            vec![Effect::If {
                when: Condition::NpcIn(ids::npc_stranger(), ids::room_shore()),
                then: vec![Effect::MoveNpc(
                    ids::npc_stranger(),
                    NpcLocation::Room(ids::room_cliff_path()),
                )],
                otherwise: vec![Effect::If {
                    when: Condition::NpcIn(ids::npc_stranger(), ids::room_cliff_path()),
                    then: vec![Effect::MoveNpc(
                        ids::npc_stranger(),
                        NpcLocation::Room(ids::room_shore()),
                    )],
                    otherwise: vec![],
                }],
            }],
        )
        .with_condition(Condition::FlagSet(ids::flag_stranger_has_arrived()))
        .with_cooldown(2),
    );

    area
}
