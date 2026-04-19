//! The storm: the countdown timer that ends the run if the player dawdles,
//! and the ambient weather rules that fire while it runs.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{Condition, Effect, Rule, Text, Timer, Trigger};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_timer(
        ids::timer_storm(),
        Timer::new(18)
            .with_on_tick(vec![Effect::If {
                when: Condition::TimerRunning(ids::timer_storm()),
                then: vec![Effect::Say(Text::OneOf(vec![
                    Text::lit("Far off, thunder."),
                    Text::lit("The wind picks up another notch."),
                    Text::lit("A gust rattles the glass."),
                ]))],
                otherwise: vec![],
            }])
            .with_on_expire(vec![
                Effect::Say(Text::Ref(ids::text_storm_close())),
                Effect::TriggerEnding(ids::ending_lost_to_the_storm()),
            ])
            .cancel_on(Condition::Any(vec![
                Condition::FlagSet(ids::flag_lantern_restored()),
                Condition::FlagSet(ids::flag_lantern_sabotaged()),
                Condition::PlayerIn(ids::room_gone()),
            ])),
    );

    // Occasional whisper while the storm is still counting down.
    area.add_rule(
        ids::rule_storm_whisper(),
        Rule::on(
            Trigger::TurnEnd,
            vec![Effect::OneOf(vec![
                vec![Effect::Say(Text::lit("A gust pushes the cottage door."))],
                vec![Effect::Say(Text::lit(
                    "The sea makes a long, tired sound against the stones.",
                ))],
                vec![],
            ])],
        )
        .with_condition(Condition::All(vec![
            Condition::TimerRunning(ids::timer_storm()),
            Condition::TurnAtLeast(2),
        ]))
        .with_cooldown(3),
    );

    // After the stranger has arrived, an occasional louder gust.
    area.add_rule(
        ids::rule_storm_whisper_slow(),
        Rule::on(
            Trigger::TurnStart,
            vec![Effect::Say(Text::lit(
                "A stronger gust. The headland moans.",
            ))],
        )
        .with_condition(Condition::All(vec![
            Condition::FlagSet(ids::flag_stranger_has_arrived()),
            Condition::Chance(34),
        ]))
        .with_cooldown(2),
    );

    area
}
