//! Game-start setup: the kickoff rule that begins both timers.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{Effect, Rule, Text, Trigger, Value};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_rule(
        ids::rule_kickoff(),
        Rule::on(
            Trigger::GameStart,
            vec![
                Effect::Say(Text::Ref(ids::text_storm_far())),
                Effect::StartTimer(ids::timer_storm()),
                Effect::StartTimer(ids::timer_stranger_arrival()),
                Effect::SetFlag(ids::flag_intro_shown(), Value::TRUE),
            ],
        )
        .once(),
    );

    area
}
