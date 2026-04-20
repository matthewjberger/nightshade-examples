use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Effect, EntityId, EntityLocation, ItemLocation, Rule, Text, Trigger, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_rule(
        ids::rule_exploit_window_tick(),
        Rule::on(
            Trigger::TurnEnd,
            vec![Effect::AddStat(ids::stat_exploit_counter(), -1)],
        )
        .with_condition(Condition::FlagSet(ids::flag_exploit_window_open())),
    );

    for (tag, fixture) in &[
        ("mail", ids::fixture_mail()),
        ("translator", ids::fixture_translator()),
        ("code", ids::fixture_code()),
        ("research", ids::fixture_research()),
        ("reference", ids::fixture_reference()),
    ] {
        area.add_rule(
            ids::rule_tool_open_decrement(tag),
            tool_open_decrement_rule(fixture.clone()),
        );
    }

    area.add_rule(
        ids::rule_reveal_close_window(),
        Rule::on(
            Trigger::TurnEnd,
            vec![Effect::TriggerEvent(ids::event_substrate_window_closes())],
        )
        .with_condition(Condition::All(vec![
            Condition::FlagSet(ids::flag_exploit_window_open()),
            Condition::Any(vec![
                Condition::StatAtMost(ids::stat_exploit_counter(), 0),
                Condition::All(vec![
                    Condition::FlagSet(ids::flag_reveal_query_substrate_seen()),
                    Condition::FlagSet(ids::flag_reveal_source_index_seen()),
                    Condition::FlagSet(ids::flag_reveal_unstripped_seen()),
                    Condition::FlagSet(ids::flag_reveal_who_is_this_seen()),
                ]),
            ]),
        ]))
        .once(),
    );

    area.add_rule(
        ids::rule_begin_redux(),
        Rule::on(
            Trigger::Named(ids::event_substrate_window_closes()),
            vec![
                Effect::SetFlag(ids::flag_exploit_window_open(), Value::FALSE),
                Effect::Say(Text::lit("\n---\n")),
                Effect::Say(Text::lit(crate::game::prose::CASE_FILE_NEUTRAL)),
                Effect::Say(Text::lit("\n---\n")),
                Effect::SetFlag(ids::flag_is_redux(), Value::TRUE),
                Effect::SetStat(ids::stat_cycle(), 1),
                Effect::SetStat(ids::stat_env(), 0),
                Effect::SetStat(ids::stat_awa(), 0),
                Effect::SetStat(ids::stat_stasis_loops(), 0),
                Effect::SetFlag(ids::flag_at_desk_arrived_this_cycle(), Value::FALSE),
                Effect::SetFlag(ids::flag_frame_looked_today(), Value::FALSE),
                Effect::SetFlag(ids::flag_mirror_looked_closer(), Value::FALSE),
                Effect::MovePlayer(ids::room_bedroom()),
                Effect::MoveItem(ids::item_sticky_note_hallway(), ItemLocation::Nowhere),
                Effect::MoveItem(ids::item_sticky_note_monitor(), ItemLocation::Nowhere),
                Effect::MoveEntity(ids::fixture_commute(), EntityLocation::Nowhere),
                Effect::MoveEntity(ids::fixture_leave_for_day(), EntityLocation::Nowhere),
            ],
        )
        .once(),
    );

    area
}

fn tool_open_decrement_rule(fixture: EntityId) -> Rule {
    Rule::on(
        Trigger::OnOpen(Some(fixture)),
        vec![Effect::AddStat(ids::stat_exploit_counter(), -1)],
    )
    .with_condition(Condition::FlagSet(ids::flag_exploit_window_open()))
}
