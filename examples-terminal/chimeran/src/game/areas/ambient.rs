//! Ambient observation rules. Each `OnEnter` rule picks one short
//! flavor line from a `Text::OneOf` pool, with pools varying by the
//! ENV anomaly stat. Nameplate-surfacing lines are intentionally mixed
//! into each anomaly tier of the desk pool so the player encounters
//! the changing nameplate without having to examine it explicitly.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{Condition, Effect, Rule, Text, Trigger};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_rule(
        ids::rule_ambient_desk(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_desk())),
            vec![Effect::Say(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_env(), 8),
                then: Box::new(Text::OneOf(vec![
                    Text::lit("The monitor is the only sound. The monitor has always been the only sound."),
                    Text::lit("The mug is warm. It has been warm for some time."),
                    Text::lit("The pens. The papers. The nameplate. These are the same objects."),
                    Text::lit("The nameplate in your peripheral vision reads something. You do not check what."),
                ])),
                otherwise: Box::new(Text::Conditional {
                    when: Condition::StatAtLeast(ids::stat_env(), 4),
                    then: Box::new(Text::OneOf(vec![
                        Text::lit("The monitor hums. You do not remember the sound of the monitor being off."),
                        Text::lit("The pen cup has three pens. You have never used any of them. They have always been there."),
                        Text::lit("The nameplate in your peripheral vision — the letters look different from yesterday. You are not certain how."),
                    ])),
                    otherwise: Box::new(Text::OneOf(vec![
                        Text::lit("The monitor hums. The mug is where you left it. The pen cup has three pens."),
                        Text::lit("The chair is comfortable. You have spent a great deal of time in this chair."),
                        Text::lit("The nameplate reads CAMERON HALE. You've never looked closely at it before."),
                    ])),
                }),
            })],
        )
        .with_cooldown(2),
    );

    area.add_rule(
        ids::rule_ambient_bedroom(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_bedroom())),
            vec![Effect::Say(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_env(), 7),
                then: Box::new(Text::lit(
                    "The bedroom. It is the bedroom. You have the sense of having been here a long time.",
                )),
                otherwise: Box::new(Text::lit(
                    "The bedroom is still. Morning light through the window.",
                )),
            })],
        )
        .with_cooldown(3),
    );

    area.add_rule(
        ids::rule_ambient_kitchen(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_kitchen())),
            vec![Effect::Say(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_env(), 6),
                then: Box::new(Text::lit(
                    "The kitchen is small. You have never used all of it.",
                )),
                otherwise: Box::new(Text::lit("The coffee is already brewed. The mug is clean.")),
            })],
        )
        .with_cooldown(2),
    );

    area.add_rule(
        ids::rule_ambient_street(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_street())),
            vec![Effect::Say(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_env(), 7),
                then: Box::new(Text::lit("The street is empty of everyone.")),
                otherwise: Box::new(Text::lit("The street is empty of people. Early morning.")),
            })],
        )
        .with_cooldown(3),
    );

    area.add_rule(
        ids::rule_ambient_elevator(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_elevator())),
            vec![Effect::Say(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_env(), 7),
                then: Box::new(Text::lit(
                    "It descends. It descends. It descends. The doors open.",
                )),
                otherwise: Box::new(Text::lit(
                    "The elevator descends at the rate elevators descend.",
                )),
            })],
        )
        .with_cooldown(3),
    );

    area
}
