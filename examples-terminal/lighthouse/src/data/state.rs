//! Mutable runtime state.
//!
//! `RuntimeState` holds everything that changes turn-to-turn. It is the only
//! thing that serializes to a save file. The authored content in
//! [`crate::data::World`] is never mutated at runtime.

use crate::data::choice::Choice;
use crate::data::ids::{
    DialogueId, EndingId, EventName, FlagKey, ItemId, NodeId, NpcId, QuestId, RoomId, RuleId,
    StatKey, TimerId,
};
use crate::data::value::Value;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Where a single item currently is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemLocation {
    /// On the floor in a specific room.
    Room(RoomId),
    /// In the player's inventory.
    Inventory,
    /// Carried by an NPC.
    HeldBy(NpcId),
    /// Removed from play entirely.
    Nowhere,
}

/// A single entry in the scrolling transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptEntry {
    /// Narrative description or rule-emitted text.
    Narration(String),
    /// Echo of the player's picked choice.
    PlayerAction(String),
    /// Dialogue line attributed to a speaker.
    Dialogue { speaker: String, text: String },
    /// System/meta message ("saved.", "cannot go that way.").
    System(String),
    /// A horizontal rule / separator for room transitions.
    Separator,
}

/// The entire mutable state of a single run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeState {
    /// Where the player currently is.
    pub current_room: RoomId,
    /// Where the player was before the most recent move (for "go back" UX).
    pub previous_room: Option<RoomId>,

    /// Location of every item that has a location.
    pub item_locations: BTreeMap<ItemId, ItemLocation>,
    /// Current room of each NPC that is on stage.
    pub npc_locations: BTreeMap<NpcId, RoomId>,
    /// Current disposition of each NPC.
    pub dispositions: BTreeMap<NpcId, i64>,

    /// Flags set by effects.
    pub flags: BTreeMap<FlagKey, Value>,
    /// Numeric stats.
    pub stats: BTreeMap<StatKey, i64>,

    /// Monotonic turn counter.
    pub turn: u32,
    /// Set of rooms the player has ever occupied.
    pub visited: BTreeSet<RoomId>,

    /// Current stage of each quest.
    pub quest_stages: BTreeMap<QuestId, NodeId>,
    /// Every stage each quest has ever been at (for `Condition::QuestReached`).
    pub quest_history: BTreeMap<QuestId, BTreeSet<NodeId>>,

    /// If `Some`, the player is currently inside a dialogue at this node.
    pub active_dialogue: Option<(DialogueId, NodeId)>,

    /// Remaining turns for each timer that is currently running.
    /// Timers not present in this map are not running.
    pub timers_remaining: BTreeMap<TimerId, u32>,
    /// Timers that have expired this run.
    pub timers_expired: BTreeSet<TimerId>,
    /// Timers that were cancelled (not expired) this run.
    pub timers_cancelled: BTreeSet<TimerId>,

    /// Events scheduled to fire on a specific turn.
    pub scheduled_events: Vec<ScheduledEvent>,

    /// Rules that have ever fired.
    pub rules_fired: BTreeSet<RuleId>,
    /// Most recent turn on which each rule fired (for cooldowns).
    pub rule_last_fired: BTreeMap<RuleId, u32>,

    /// Endings that have been unlocked across this run.
    pub unlocked_endings: BTreeSet<EndingId>,

    /// Scrolling transcript.
    pub transcript: VecDeque<TranscriptEntry>,

    /// If non-empty, these override normal choice assembly until one is picked.
    pub pending_choices: Vec<Choice>,

    /// If set, the game is over and this is the ending that fired.
    pub game_over: Option<EndingId>,

    /// Deterministic RNG state (xorshift64).
    pub rng_state: u64,
}

/// Maximum transcript length kept in state. Older entries are dropped.
pub const TRANSCRIPT_CAPACITY: usize = 512;

/// A pending scheduled event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub event: EventName,
    /// Turn on which the event should fire (compared against `RuntimeState.turn`).
    pub fires_on_turn: u32,
}

impl RuntimeState {
    /// Push a transcript entry, trimming the front if the buffer is full.
    pub fn push_transcript(&mut self, entry: TranscriptEntry) {
        if self.transcript.len() >= TRANSCRIPT_CAPACITY {
            self.transcript.pop_front();
        }
        self.transcript.push_back(entry);
    }

    /// Advance the deterministic RNG and return the raw 64-bit output.
    pub fn advance_rng(&mut self) -> u64 {
        let mut state = self.rng_state;
        if state == 0 {
            state = 0x9E3779B97F4A7C15;
        }
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.rng_state = state;
        state
    }

    /// Return a value in `[0, bound)` using the deterministic RNG.
    pub fn random_index(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.advance_rng() as usize) % bound
        }
    }

    /// Return a value in `[0, 100)` for percentage checks. Combined with
    /// `roll < percent`, this makes `Condition::Chance(100)` always fire and
    /// `Condition::Chance(0)` never fire.
    pub fn random_percent(&mut self) -> u8 {
        (self.advance_rng() % 100) as u8
    }
}
