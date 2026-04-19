//! Cross-cycle plumbing. Sleep rules are generated from the cycle
//! transitions in `crate::game::plan` so cycle progression lives in
//! data. The code here is just the transitions-to-rules dispatcher,
//! the post-exploit stasis loop, and the Marisol-goes-offline rule.
//!
//! All sleep rules listen for `event_sleep`. Rule dispatch filters
//! candidates and evaluates conditions *before* any effects run, so
//! each rule's `from` condition reads the pre-mutation cycle value
//! and only the matching rule fires.

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

    // Post-planned stasis loop. Fires when the player sleeps at or past
    // the last scripted cycle — i.e. cycle 8 onward without having run
    // the exploit. Each loop bumps both `stat_cycle` (past 8) and
    // `stat_stasis_loops`; the stasis ending triggers when loops ≥ 3.
    area.add_rule(
        ids::rule_sleep_post_exploit(),
        Rule::on(
            Trigger::Named(ids::event_sleep()),
            vec![
                Effect::AddStat(ids::stat_cycle(), 1),
                Effect::AddStat(ids::stat_stasis_loops(), 1),
                Effect::Say(Text::lit("You sleep.\n\nTime passes.")),
                Effect::DescribeRoom,
            ],
        )
        .with_condition(Condition::StatAtLeast(
            ids::stat_cycle(),
            last_planned_cycle(),
        )),
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

    area
}

fn sleep_rule(transition: &CycleTransition) -> Rule {
    let mut effects = vec![Effect::SetStat(ids::stat_cycle(), transition.to)];
    // Auto-submit every request belonging to the cycle we're leaving.
    // Already-submitted flags are overwritten with the same value (no-
    // op); unsubmitted ones now count as Escalated.
    for tag in transition.requests {
        effects.push(Effect::SetFlag(ids::flag_req_submitted(tag), Value::TRUE));
    }
    effects.push(Effect::AddStat(ids::stat_env(), transition.env_bump));
    effects.push(Effect::Say(Text::lit(
        transition.sleep_narration.to_string(),
    )));
    effects.push(Effect::DescribeRoom);
    Rule::on(Trigger::Named(ids::event_sleep()), effects).with_condition(Condition::All(vec![
        Condition::StatAtLeast(ids::stat_cycle(), transition.from),
        Condition::StatAtMost(ids::stat_cycle(), transition.from),
    ]))
}
