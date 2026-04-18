//! Closed enum of predicates the engine can evaluate over `World` and `RuntimeState`.
//!
//! Conditions appear in exits (to gate them), rules (to guard firing), effects
//! (inside `Effect::If`), choices (to lock/show them), endings (as triggers),
//! timers (as cancel conditions), and quest transitions.
//!
//! Evaluation is a pure function of `World` + `RuntimeState` implemented in
//! [`crate::engine::eval`]. The engine never mutates state while evaluating.

use crate::data::ids::{
    ConditionId, FlagKey, ItemId, NodeId, NpcId, QuestId, RoomId, RuleId, StatKey, TimerId,
};
use crate::data::value::Value;
use serde::{Deserialize, Serialize};

/// A predicate over the current game state.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// Always true. Useful as a default and for testing.
    Always,
    /// Always false.
    Never,

    /// All inner conditions must hold.
    All(Vec<Condition>),
    /// Any inner condition must hold.
    Any(Vec<Condition>),
    /// Inner condition must not hold.
    Not(Box<Condition>),

    /// Flag is set to a specific value.
    FlagEquals(FlagKey, Value),
    /// Flag is set to any value.
    FlagSet(FlagKey),
    /// Flag is not set (never written, or written and later cleared).
    FlagUnset(FlagKey),

    /// Stat is at or above the given threshold. Stats default to 0 when unset.
    StatAtLeast(StatKey, i64),
    /// Stat is at or below the given threshold.
    StatAtMost(StatKey, i64),

    /// The player is currently in this room.
    PlayerIn(RoomId),
    /// The player has visited this room at least once.
    Visited(RoomId),

    /// Item is in the player's inventory.
    HasItem(ItemId),
    /// Item is in the given room (on the floor).
    ItemInRoom(ItemId, RoomId),
    /// Item has been seen anywhere (taken by the player or moved by a rule);
    /// convenience check for "does the player know about this item".
    ItemIsSomewhere(ItemId),

    /// NPC is currently in the given room.
    NpcIn(NpcId, RoomId),
    /// NPC's disposition is at or above the given threshold.
    DispositionAtLeast(NpcId, i64),

    /// Current turn number is at least this value.
    TurnAtLeast(u32),
    /// Current turn number is at most this value.
    TurnAtMost(u32),

    /// The quest is currently at the given stage.
    QuestAt(QuestId, NodeId),
    /// The quest has reached (is at, or has passed through) the given stage.
    QuestReached(QuestId, NodeId),

    /// A timer is still counting down (started and not yet expired or cancelled).
    TimerRunning(TimerId),
    /// A timer has expired.
    TimerExpired(TimerId),

    /// A rule has fired at least once.
    RuleFired(RuleId),

    /// Random chance in [0, 100]. 0 never, 100 always, using the runtime RNG.
    Chance(u8),

    /// Reference a condition defined in `World.conditions` for reuse.
    Ref(ConditionId),
}
