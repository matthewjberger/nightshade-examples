//! String-newtype IDs for every entity kind in the data layer.
//!
//! All IDs are thin wrappers around `String` so they serialize cleanly, are
//! easy to author inline (`RoomId::new("tower_base")`), and sort
//! deterministically inside `BTreeMap`. Collections in [`crate::data::World`]
//! are keyed on these types; references between entities (an exit's target, a
//! rule's effect moving an item to a room) also use them.

use serde::{Deserialize, Serialize};

macro_rules! newtype_id {
    ($($(#[$meta:meta])* $name:ident),* $(,)?) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
            pub struct $name(pub String);

            impl $name {
                pub fn new(value: impl Into<String>) -> Self {
                    Self(value.into())
                }

                pub fn as_str(&self) -> &str {
                    &self.0
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str(&self.0)
                }
            }

            impl From<&str> for $name {
                fn from(value: &str) -> Self {
                    Self(value.to_string())
                }
            }

            impl From<String> for $name {
                fn from(value: String) -> Self {
                    Self(value)
                }
            }
        )*
    };
}

newtype_id! {
    /// A room in the world graph.
    RoomId,
    /// An item that can live in a room, in inventory, inside a container, or held by an NPC.
    ItemId,
    /// A non-player character.
    NpcId,
    /// A conversation graph (keyed in `World.dialogues`).
    DialogueId,
    /// A node inside a dialogue or quest graph (keyed within that graph).
    NodeId,
    /// A rule in `World.rules`.
    RuleId,
    /// A quest in `World.quests`.
    QuestId,
    /// An ending in `World.endings`.
    EndingId,
    /// A text entry in the shared `World.texts` table, referenced from `Text::Ref`.
    TextId,
    /// A condition entry in the shared `World.conditions` table, referenced from `Condition::Ref`.
    ConditionId,
    /// A runtime flag key stored in `RuntimeState.flags`.
    FlagKey,
    /// A runtime numeric stat key stored in `RuntimeState.stats`.
    StatKey,
    /// A timer in `World.timers`.
    TimerId,
    /// A custom event name used by `Trigger::Named`, `Effect::TriggerEvent`, and `Effect::ScheduleEvent`.
    EventName,
}
