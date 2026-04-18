//! Data layer for the text adventure.
//!
//! All types here are pure data: structs and closed enums with `Serialize +
//! Deserialize`. There is no game logic in this module. The engine layer
//! (`crate::engine`) is the only thing that interprets these values.
//!
//! Split into two kinds of state:
//!
//! - [`World`] holds authored, immutable content: rooms, items, rules, text.
//! - [`RuntimeState`] holds everything that changes during play: the
//!   player's location, inventory, flag values, timer remaining turns,
//!   transcript, etc. `RuntimeState` is the only thing that serializes for
//!   save files.

pub mod choice;
pub mod condition;
pub mod dialogue;
pub mod effect;
pub mod ending;
pub mod ids;
pub mod item;
pub mod npc;
pub mod quest;
pub mod room;
pub mod rule;
pub mod state;
pub mod text;
pub mod timer;
pub mod trigger;
pub mod value;
pub mod verb_responses;
pub mod world;

pub use choice::{Choice, ChoiceAction};
pub use condition::Condition;
pub use dialogue::{Dialogue, DialogueNode, DialogueOption};
pub use effect::Effect;
pub use ending::Ending;
pub use ids::{
    ConditionId, DialogueId, EndingId, EventName, FlagKey, ItemId, NodeId, NpcId, QuestId, RoomId,
    RuleId, StatKey, TextId, TimerId,
};
pub use item::{Item, ItemProperties};
pub use npc::Npc;
pub use quest::{Quest, QuestStage, QuestStageKind, QuestTransition};
pub use room::{Exit, Room};
pub use rule::Rule;
pub use state::{ItemLocation, RuntimeState, ScheduledEvent, TranscriptEntry};
pub use text::Text;
pub use timer::Timer;
pub use trigger::{Trigger, TriggerKind};
pub use value::Value;
pub use verb_responses::{Placeholder, VerbResponses};
pub use world::World;
