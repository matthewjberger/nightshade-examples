use crate::game::areas::AreaContents;
use crate::game::ids;
use crate::game::plan::{CycleTransition, last_planned_cycle, transitions};
use nightshade::interactive_fiction::data::{Condition, Effect, Rule, Text, Trigger, Value};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    for transition in transitions() {
        area.add_rule(
            ids::rule_sleep_from(transition.from),
            sleep_rule(transition),
        );
    }

    area.add_rule(
        ids::rule_sleep_post_exploit(),
        Rule::on(
            Trigger::Named(ids::event_sleep()),
            vec![
                Effect::AddStat(ids::stat_cycle(), 1),
                Effect::AddStat(ids::stat_stasis_loops(), 1),
                Effect::SetFlag(ids::flag_woke_up_this_cycle(), Value::TRUE),
                Effect::Say(Text::lit("You sleep.\n\nTime passes.")),
                Effect::MovePlayer(ids::room_bedroom()),
            ],
        )
        .with_condition(Condition::All(vec![
            Condition::StatAtLeast(ids::stat_cycle(), last_planned_cycle()),
            Condition::FlagUnset(ids::flag_is_redux()),
        ])),
    );

    area.add_rule(
        ids::rule_marisol_goes_offline(),
        Rule::on(
            Trigger::TurnStart,
            vec![Effect::SetFlag(ids::flag_marisol_offline(), Value::TRUE)],
        )
        .with_condition(Condition::StatAtLeast(ids::stat_cycle(), 7))
        .once(),
    );

    area.add_rule(
        ids::rule_clear_wake_flag(),
        Rule::on(
            Trigger::OnExit(Some(ids::room_bedroom())),
            vec![Effect::SetFlag(
                ids::flag_woke_up_this_cycle(),
                Value::FALSE,
            )],
        ),
    );

    area
}

fn sleep_rule(transition: &CycleTransition) -> Rule {
    let mut effects = vec![Effect::SetStat(ids::stat_cycle(), transition.to)];
    for tag in transition.requests {
        effects.push(Effect::SetFlag(ids::flag_req_submitted(tag), Value::TRUE));
    }
    effects.push(Effect::AddStat(ids::stat_env(), transition.env_bump));
    effects.push(Effect::SetFlag(ids::flag_woke_up_this_cycle(), Value::TRUE));
    effects.push(Effect::Say(Text::lit(
        transition.sleep_narration.to_string(),
    )));
    effects.push(Effect::MovePlayer(ids::room_bedroom()));
    Rule::on(Trigger::Named(ids::event_sleep()), effects).with_condition(Condition::All(vec![
        Condition::StatAtLeast(ids::stat_cycle(), transition.from),
        Condition::StatAtMost(ids::stat_cycle(), transition.from),
    ]))
}
