//! Events that can cause a rule to fire.
//!
//! Each [`crate::data::Rule`] has a single [`Trigger`]. The engine indexes
//! rules by [`TriggerKind`] (the kind tag without the specific ID payload) so
//! it can quickly find candidate rules when an event fires, then filters the
//! candidates by matching the full trigger and evaluating the rule's
//! condition.
//!
//! `Option<Id>` payloads mean "match any target of that kind." A rule with
//! `Trigger::OnEnter(None)` fires whenever the player enters any room; a rule
//! with `Trigger::OnEnter(Some(room))` fires only for that specific room.

use crate::data::ids::{EventName, FlagKey, ItemId, NpcId, RoomId};
use serde::{Deserialize, Serialize};

/// An event the engine raises that rules can match against.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
    /// Fires once when the game starts, before any player input.
    GameStart,
    /// Fires at the start of every turn (before the player picks a choice).
    TurnStart,
    /// Fires at the end of every turn (after effects of the player's choice have run).
    TurnEnd,

    /// Fires when the player enters a room. `None` matches any room.
    OnEnter(Option<RoomId>),
    /// Fires when the player leaves a room. `None` matches any room.
    OnExit(Option<RoomId>),

    /// Fires when the player takes an item. `None` matches any item.
    OnTake(Option<ItemId>),
    /// Fires when the player drops an item.
    OnDrop(Option<ItemId>),
    /// Fires when the player uses an item (optionally in a specific room).
    OnUse {
        item: Option<ItemId>,
        in_room: Option<RoomId>,
    },
    /// Fires when the player examines an item.
    OnExamine(Option<ItemId>),

    /// Fires when the player starts talking to an NPC.
    OnTalk(Option<NpcId>),

    /// Fires when a flag is set to any value.
    OnFlagSet(FlagKey),
    /// Fires when a flag is unset.
    OnFlagUnset(FlagKey),

    /// Fires when a custom named event is raised via `Effect::TriggerEvent`
    /// or a scheduled event's due turn is reached.
    Named(EventName),
}

/// The tag portion of a [`Trigger`], used by the engine's rule dispatch index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriggerKind {
    GameStart,
    TurnStart,
    TurnEnd,
    OnEnter,
    OnExit,
    OnTake,
    OnDrop,
    OnUse,
    OnExamine,
    OnTalk,
    OnFlagSet,
    OnFlagUnset,
    Named,
}

impl Trigger {
    pub fn kind(&self) -> TriggerKind {
        match self {
            Trigger::GameStart => TriggerKind::GameStart,
            Trigger::TurnStart => TriggerKind::TurnStart,
            Trigger::TurnEnd => TriggerKind::TurnEnd,
            Trigger::OnEnter(_) => TriggerKind::OnEnter,
            Trigger::OnExit(_) => TriggerKind::OnExit,
            Trigger::OnTake(_) => TriggerKind::OnTake,
            Trigger::OnDrop(_) => TriggerKind::OnDrop,
            Trigger::OnUse { .. } => TriggerKind::OnUse,
            Trigger::OnExamine(_) => TriggerKind::OnExamine,
            Trigger::OnTalk(_) => TriggerKind::OnTalk,
            Trigger::OnFlagSet(_) => TriggerKind::OnFlagSet,
            Trigger::OnFlagUnset(_) => TriggerKind::OnFlagUnset,
            Trigger::Named(_) => TriggerKind::Named,
        }
    }
}
