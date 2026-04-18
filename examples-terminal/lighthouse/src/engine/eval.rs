//! Condition evaluation. Pure function of `World + RuntimeState` (no mutation).

use crate::data::{Condition, ItemLocation, RuntimeState, Value};
use crate::engine::Engine;

/// Evaluate a condition against the current state.
pub fn evaluate(engine: &Engine, state: &RuntimeState, condition: &Condition) -> bool {
    match condition {
        Condition::Always => true,
        Condition::Never => false,

        Condition::All(inner) => inner.iter().all(|c| evaluate(engine, state, c)),
        Condition::Any(inner) => inner.iter().any(|c| evaluate(engine, state, c)),
        Condition::Not(inner) => !evaluate(engine, state, inner),

        Condition::FlagEquals(key, value) => state.flags.get(key) == Some(value),
        Condition::FlagSet(key) => match state.flags.get(key) {
            Some(Value::Bool(false)) => false,
            Some(_) => true,
            None => false,
        },
        Condition::FlagUnset(key) => match state.flags.get(key) {
            Some(Value::Bool(false)) => true,
            Some(_) => false,
            None => true,
        },

        Condition::StatAtLeast(key, threshold) => {
            state.stats.get(key).copied().unwrap_or(0) >= *threshold
        }
        Condition::StatAtMost(key, threshold) => {
            state.stats.get(key).copied().unwrap_or(0) <= *threshold
        }

        Condition::PlayerIn(room) => &state.current_room == room,
        Condition::Visited(room) => state.visited.contains(room),

        Condition::HasItem(item) => {
            matches!(
                state.item_locations.get(item),
                Some(ItemLocation::Inventory)
            )
        }
        Condition::ItemInRoom(item, room) => {
            matches!(
                state.item_locations.get(item),
                Some(ItemLocation::Room(current)) if current == room
            )
        }
        Condition::ItemIsSomewhere(item) => !matches!(
            state.item_locations.get(item),
            None | Some(ItemLocation::Nowhere)
        ),

        Condition::NpcIn(npc, room) => state.npc_locations.get(npc) == Some(room),
        Condition::DispositionAtLeast(npc, threshold) => {
            state.dispositions.get(npc).copied().unwrap_or(0) >= *threshold
        }

        Condition::TurnAtLeast(turn) => state.turn >= *turn,
        Condition::TurnAtMost(turn) => state.turn <= *turn,

        Condition::QuestAt(quest, stage) => state.quest_stages.get(quest) == Some(stage),
        Condition::QuestReached(quest, stage) => state
            .quest_history
            .get(quest)
            .map(|seen| seen.contains(stage))
            .unwrap_or(false),

        Condition::TimerRunning(timer) => state.timers_remaining.contains_key(timer),
        Condition::TimerExpired(timer) => state.timers_expired.contains(timer),

        Condition::RuleFired(rule) => state.rules_fired.contains(rule),

        Condition::Chance(_) => {
            // Chance needs mutable state to advance the RNG; use `evaluate_mut`
            // from an effect-executor context. When evaluated via the
            // immutable path (e.g. from text resolution or choice assembly),
            // a chance condition is treated as false so the UI stays stable.
            false
        }

        Condition::Ref(id) => match engine.world().conditions.get(id) {
            Some(inner) => evaluate(engine, state, inner),
            None => false,
        },
    }
}

/// Evaluate a condition with access to the RNG. Behaves identically to
/// [`evaluate`] for every variant except `Condition::Chance`, which reads
/// the RNG to randomize.
pub fn evaluate_mut(engine: &Engine, state: &mut RuntimeState, condition: &Condition) -> bool {
    match condition {
        // Combinators recurse through the mut-aware evaluator so any
        // nested Chance is still randomized.
        Condition::All(inner) => inner.iter().all(|c| evaluate_mut(engine, state, c)),
        Condition::Any(inner) => inner.iter().any(|c| evaluate_mut(engine, state, c)),
        Condition::Not(inner) => !evaluate_mut(engine, state, inner),
        Condition::Ref(id) => match engine.world().conditions.get(id).cloned() {
            Some(inner) => evaluate_mut(engine, state, &inner),
            None => false,
        },

        Condition::Chance(percent) => state.random_percent() < *percent,

        // Everything else is deterministic; defer to the immutable evaluator
        // so there is exactly one implementation per variant.
        Condition::Always
        | Condition::Never
        | Condition::FlagEquals(_, _)
        | Condition::FlagSet(_)
        | Condition::FlagUnset(_)
        | Condition::StatAtLeast(_, _)
        | Condition::StatAtMost(_, _)
        | Condition::PlayerIn(_)
        | Condition::Visited(_)
        | Condition::HasItem(_)
        | Condition::ItemInRoom(_, _)
        | Condition::ItemIsSomewhere(_)
        | Condition::NpcIn(_, _)
        | Condition::DispositionAtLeast(_, _)
        | Condition::TurnAtLeast(_)
        | Condition::TurnAtMost(_)
        | Condition::QuestAt(_, _)
        | Condition::QuestReached(_, _)
        | Condition::TimerRunning(_)
        | Condition::TimerExpired(_)
        | Condition::RuleFired(_) => evaluate(engine, state, condition),
    }
}
