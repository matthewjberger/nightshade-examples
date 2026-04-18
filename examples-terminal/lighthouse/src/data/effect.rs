//! Closed enum of state mutations and outputs the engine can execute.
//!
//! Effects are the only way to change `RuntimeState` from the data layer:
//! rules, dialogue nodes, quest transitions, and choices all emit lists of
//! `Effect` values, which the engine interprets in order. Effects can be
//! nested with `If`, `Sequence`, and `OneOf` to build small programs purely
//! out of data.

use crate::data::choice::Choice;
use crate::data::condition::Condition;
use crate::data::ids::{
    DialogueId, EndingId, EventName, FlagKey, ItemId, NodeId, NpcId, QuestId, RoomId, RuleId,
    StatKey, TimerId,
};
use crate::data::state::ItemLocation;
use crate::data::text::Text;
use crate::data::value::Value;
use serde::{Deserialize, Serialize};

/// A single state-mutating operation, or a composite of such operations.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    // Output
    /// Append narration to the transcript.
    Say(Text),
    /// Describe the player's current room (name + description + visible contents).
    DescribeRoom,
    /// Clear the transcript.
    ClearTranscript,

    // State mutation
    /// Set a flag to a value.
    SetFlag(FlagKey, Value),
    /// Remove a flag entirely.
    UnsetFlag(FlagKey),
    /// Increment (or decrement) a stat by a delta.
    AddStat(StatKey, i64),
    /// Set a stat to an absolute value.
    SetStat(StatKey, i64),

    // World mutation
    /// Move an item to a new location (room, inventory, inside another item, held by an NPC, or nowhere).
    MoveItem(ItemId, ItemLocation),
    /// Move the player to a room. Fires the appropriate enter/exit triggers.
    MovePlayer(RoomId),
    /// Move an NPC to a room.
    MoveNpc(NpcId, RoomId),
    /// Shift an NPC's disposition by a delta.
    AdjustDisposition(NpcId, i64),

    // Quest
    /// Advance a quest to the given stage. Fires the stage's `on_enter` effects.
    SetQuestStage(QuestId, NodeId),

    // Dialogue
    /// Start the given dialogue at its start node.
    BeginDialogue(DialogueId),
    /// End the active dialogue, returning the player to normal play.
    EndDialogue,
    /// Jump within the active dialogue to the given node (runs `on_enter`).
    GotoDialogue(NodeId),

    // Flow control (effects composing other effects)
    /// Run `then` if the condition holds, else run `otherwise`.
    If {
        when: Condition,
        then: Vec<Effect>,
        otherwise: Vec<Effect>,
    },
    /// Run the effects in order.
    Sequence(Vec<Effect>),
    /// Run exactly one of the inner effect lists, chosen with the runtime RNG.
    OneOf(Vec<Vec<Effect>>),

    /// Replace the currently offered choices with this explicit set.
    /// Until the player picks one, normal choice assembly is suppressed.
    OfferChoices(Vec<Choice>),

    /// Fire a named custom event. Rules with `Trigger::Named(name)` will match.
    TriggerEvent(EventName),
    /// Force a specific rule to fire, regardless of its own trigger.
    FireRule(RuleId),

    // Timers
    /// Start (or reset) a timer at its configured initial duration.
    StartTimer(TimerId),
    /// Cancel a timer without running its expiration effects.
    CancelTimer(TimerId),
    /// Schedule a named event to fire after `in_turns` turns have passed.
    ScheduleEvent { event: EventName, in_turns: u32 },

    /// End the game with the given ending.
    TriggerEnding(EndingId),
}
