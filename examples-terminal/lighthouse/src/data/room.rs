//! Rooms and exits.
//!
//! Rooms are kept in `World.rooms` keyed by [`crate::data::RoomId`]. Each
//! room has a list of [`Exit`]s whose directions become menu entries when the
//! engine assembles choices for a turn.

use crate::data::condition::Condition;
use crate::data::ids::RoomId;
use crate::data::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A location the player can occupy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// Short room name shown as a header.
    pub name: String,
    /// Long-form narrative description.
    pub description: Text,
    /// Available exits. Hidden and locked exits still live here; visibility
    /// and lock state are expressed through conditions and messages.
    pub exits: Vec<Exit>,
    /// If true, the room is in darkness unless the player carries a lit light source.
    pub dark: bool,
    /// Text shown when in the dark (no description, just this).
    pub dark_description: Option<Text>,
    /// Keyword -> description lookup for "examine X" when X is a feature of
    /// the room rather than an item. Keywords are matched case-insensitively
    /// against substrings of the player's input.
    pub examine: BTreeMap<String, Text>,
}

impl Room {
    pub fn new(name: impl Into<String>, description: Text) -> Self {
        Self {
            name: name.into(),
            description,
            exits: Vec::new(),
            dark: false,
            dark_description: None,
            examine: BTreeMap::new(),
        }
    }

    pub fn with_exit(mut self, exit: Exit) -> Self {
        self.exits.push(exit);
        self
    }

    pub fn dark(mut self, description: Text) -> Self {
        self.dark = true;
        self.dark_description = Some(description);
        self
    }

    pub fn with_examine(mut self, keyword: impl Into<String>, text: Text) -> Self {
        self.examine.insert(keyword.into().to_lowercase(), text);
        self
    }
}

/// A one-way passage between rooms. Bidirectional connectivity is expressed by
/// giving both rooms an exit that targets the other.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exit {
    /// Label shown in the menu: "north", "up", "to the cellar".
    pub direction: String,
    /// Destination room.
    pub to: RoomId,
    /// If present and false, the exit appears in the menu but picking it
    /// prints `locked_message` instead of moving. `None` means always passable.
    pub passable_when: Option<Condition>,
    /// Message shown when the player tries to pass a gated exit whose
    /// condition does not hold.
    pub locked_message: Option<Text>,
    /// If present and false, the exit is hidden from the menu entirely.
    /// Used for secret passages revealed by rules.
    pub visible_when: Option<Condition>,
}

impl Exit {
    pub fn new(direction: impl Into<String>, to: RoomId) -> Self {
        Self {
            direction: direction.into(),
            to,
            passable_when: None,
            locked_message: None,
            visible_when: None,
        }
    }

    pub fn gated(mut self, condition: Condition, locked_message: Text) -> Self {
        self.passable_when = Some(condition);
        self.locked_message = Some(locked_message);
        self
    }

    pub fn hidden_until(mut self, visible_when: Condition) -> Self {
        self.visible_when = Some(visible_when);
        self
    }
}
