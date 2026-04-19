use crate::game::areas::AreaContents;
use crate::game::ids;
use crate::game::plan::INITIAL_CYCLE;
use nightshade::interactive_fiction::data::{Effect, Room, Rule, Text, Trigger, Value};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_room(
        ids::room_endgame(),
        Room::new(
            "Wrap",
            Text::lit(
                "The case file prints. Somewhere, something initializes. The work will continue.",
            ),
        ),
    );

    area.add_rule(
        ids::rule_kickoff(),
        Rule::on(
            Trigger::GameStart,
            vec![
                Effect::SetStat(ids::stat_cycle(), INITIAL_CYCLE),
                Effect::SetStat(ids::stat_env(), 0),
                Effect::SetStat(ids::stat_awa(), 0),
                Effect::SetStat(ids::stat_marisol_rel(), 0),
                Effect::SetStat(ids::stat_rachel_rel(), 0),
                Effect::SetStat(ids::stat_dmitri_rel(), 0),
                Effect::SetStat(ids::stat_winnie_rel(), 0),
                Effect::SetStat(ids::stat_stasis_loops(), 0),
                Effect::SetFlag(ids::flag_is_redux(), Value::FALSE),
            ],
        )
        .once(),
    );

    area
}
