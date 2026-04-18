//! Rule indexing and dispatch.
//!
//! At `Engine::new` we build a [`RuleIndex`] bucketing rule IDs by their
//! [`TriggerKind`]. At runtime, when an event fires, we look up the bucket,
//! filter by matching the full trigger, evaluate each candidate's condition,
//! check once/cooldown gating, and run effects of the matching rules in
//! priority order (higher first, ties broken by rule ID).

use crate::data::{
    EventName, FlagKey, ItemId, NpcId, RoomId, Rule, RuleId, Trigger, TriggerKind, World,
};
use crate::engine::{Engine, eval};
use std::collections::HashMap;

/// Derived index built from `World.rules` for fast lookup. Never serialized.
pub struct RuleIndex {
    by_kind: HashMap<TriggerKind, Vec<RuleId>>,
}

impl RuleIndex {
    pub fn build(world: &World) -> Self {
        let mut by_kind: HashMap<TriggerKind, Vec<RuleId>> = HashMap::new();
        for (id, rule) in &world.rules {
            by_kind
                .entry(rule.trigger.kind())
                .or_default()
                .push(id.clone());
        }
        for bucket in by_kind.values_mut() {
            bucket.sort();
        }
        Self { by_kind }
    }

    pub fn rules_for(&self, kind: TriggerKind) -> &[RuleId] {
        self.by_kind.get(&kind).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// A concrete event the engine raises. Rules are matched against this.
#[derive(Debug, Clone)]
pub enum Event {
    GameStart,
    TurnStart,
    TurnEnd,
    PlayerEntered(RoomId),
    PlayerExited(RoomId),
    TakeItem(ItemId),
    DropItem(ItemId),
    UseItem { item: ItemId, room: RoomId },
    ExamineItem(ItemId),
    TalkTo(NpcId),
    FlagSet(FlagKey),
    FlagUnset(FlagKey),
    Named(EventName),
}

impl Event {
    pub fn kind(&self) -> TriggerKind {
        match self {
            Event::GameStart => TriggerKind::GameStart,
            Event::TurnStart => TriggerKind::TurnStart,
            Event::TurnEnd => TriggerKind::TurnEnd,
            Event::PlayerEntered(_) => TriggerKind::OnEnter,
            Event::PlayerExited(_) => TriggerKind::OnExit,
            Event::TakeItem(_) => TriggerKind::OnTake,
            Event::DropItem(_) => TriggerKind::OnDrop,
            Event::UseItem { .. } => TriggerKind::OnUse,
            Event::ExamineItem(_) => TriggerKind::OnExamine,
            Event::TalkTo(_) => TriggerKind::OnTalk,
            Event::FlagSet(_) => TriggerKind::OnFlagSet,
            Event::FlagUnset(_) => TriggerKind::OnFlagUnset,
            Event::Named(_) => TriggerKind::Named,
        }
    }
}

/// Check whether the given rule trigger matches the fired event.
///
/// Matches on the `Trigger` side exhaustively so that adding a new
/// `Trigger` variant forces a compile error here rather than silently
/// never firing. The inner event match may fall through — we only reach
/// this function for events whose `TriggerKind` already equals the
/// trigger's, so per-variant filtering handles the payload only.
fn trigger_matches(trigger: &Trigger, event: &Event) -> bool {
    match trigger {
        Trigger::GameStart => matches!(event, Event::GameStart),
        Trigger::TurnStart => matches!(event, Event::TurnStart),
        Trigger::TurnEnd => matches!(event, Event::TurnEnd),
        Trigger::OnEnter(target) => match event {
            Event::PlayerEntered(room) => target.as_ref().is_none_or(|id| id == room),
            _ => false,
        },
        Trigger::OnExit(target) => match event {
            Event::PlayerExited(room) => target.as_ref().is_none_or(|id| id == room),
            _ => false,
        },
        Trigger::OnTake(target) => match event {
            Event::TakeItem(item) => target.as_ref().is_none_or(|id| id == item),
            _ => false,
        },
        Trigger::OnDrop(target) => match event {
            Event::DropItem(item) => target.as_ref().is_none_or(|id| id == item),
            _ => false,
        },
        Trigger::OnUse { item, in_room } => match event {
            Event::UseItem {
                item: ev_item,
                room: ev_room,
            } => {
                item.as_ref().is_none_or(|id| id == ev_item)
                    && in_room.as_ref().is_none_or(|id| id == ev_room)
            }
            _ => false,
        },
        Trigger::OnExamine(target) => match event {
            Event::ExamineItem(item) => target.as_ref().is_none_or(|id| id == item),
            _ => false,
        },
        Trigger::OnTalk(target) => match event {
            Event::TalkTo(npc) => target.as_ref().is_none_or(|id| id == npc),
            _ => false,
        },
        Trigger::OnFlagSet(key) => match event {
            Event::FlagSet(ev_key) => key == ev_key,
            _ => false,
        },
        Trigger::OnFlagUnset(key) => match event {
            Event::FlagUnset(ev_key) => key == ev_key,
            _ => false,
        },
        Trigger::Named(name) => match event {
            Event::Named(ev_name) => name == ev_name,
            _ => false,
        },
    }
}

/// Return the subset of rules for this event kind that actually match this
/// event, pass their condition, and satisfy once/cooldown gating — sorted
/// highest priority first, with RuleId as tiebreaker.
pub fn candidates(
    engine: &Engine,
    state: &crate::data::RuntimeState,
    event: &Event,
) -> Vec<RuleId> {
    let mut chosen: Vec<(&Rule, RuleId)> = Vec::new();
    for rule_id in engine.rule_index().rules_for(event.kind()) {
        let Some(rule) = engine.world().rules.get(rule_id) else {
            continue;
        };
        if !trigger_matches(&rule.trigger, event) {
            continue;
        }
        if rule.once && state.rules_fired.contains(rule_id) {
            continue;
        }
        if rule.cooldown_turns > 0
            && let Some(last) = state.rule_last_fired.get(rule_id)
            && state.turn.saturating_sub(*last) < rule.cooldown_turns
        {
            continue;
        }
        if !eval::evaluate(engine, state, &rule.condition) {
            continue;
        }
        chosen.push((rule, rule_id.clone()));
    }
    chosen.sort_by(|a, b| b.0.priority.cmp(&a.0.priority).then_with(|| a.1.cmp(&b.1)));
    chosen.into_iter().map(|(_, id)| id).collect()
}
