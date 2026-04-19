//! The exploit window.
//!
//! When the player runs the script (via the Code tool), a flag is set
//! that opens the reveal window: stat_exploit_counter starts at 25 and
//! decrements on every turn-end and on specific dialogue options that
//! count as "meaningful." When the counter reaches 0 or all four
//! reveal options have been opened, the window closes, the case file
//! prints, and the player transitions into the redux cycle.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Effect, EntityId, ItemLocation, Rule, Text, Trigger, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    // Decrement the counter on every turn-end while the window is open.
    // Turn-advancing actions (moving between rooms, Wait) count;
    // non-turn-advancing interactions do not. Specific dialogue options
    // also decrement explicitly via AddStat(-1).
    area.add_rule(
        ids::rule_exploit_window_tick(),
        Rule::on(
            Trigger::TurnEnd,
            vec![Effect::AddStat(ids::stat_exploit_counter(), -1)],
        )
        .with_condition(Condition::FlagSet(ids::flag_exploit_window_open())),
    );

    // Opening Mail, Translator, Code, unaltered-Research or
    // unaltered-Reference during the exploit window is a "meaningful
    // action" per the spec — each OnOpen event decrements the
    // counter by one. Notepad and Chatter are not on that list
    // (Notepad's Unstripped view is already reveal content; Chatter's
    // Rachel-message option decrements explicitly when chosen).
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

    // The window closes when the counter hits zero OR all four reveal
    // items have been opened.
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

    // On window close: print the case file, reset fresh for Cameron
    // 0048 (awa/cycle/env/stasis_loops all back to zero; per-day flags
    // cleared), set the redux flag, move player to the bedroom. All
    // cycle-gated content now reads through the redux branch.
    area.add_rule(
        ids::rule_begin_redux(),
        Rule::on(
            Trigger::Named(ids::event_substrate_window_closes()),
            vec![
                Effect::SetFlag(ids::flag_exploit_window_open(), Value::FALSE),
                Effect::Say(Text::lit("\n---\n")),
                Effect::Say(Text::lit(include_str!("../prose/case_file_neutral.txt"))),
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
                // Pull the pre-reveal sticky notes out of play; the
                // redux uses the nightstand sticky only.
                Effect::MoveItem(ids::item_sticky_note_hallway(), ItemLocation::Nowhere),
                Effect::MoveItem(ids::item_sticky_note_monitor(), ItemLocation::Nowhere),
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
