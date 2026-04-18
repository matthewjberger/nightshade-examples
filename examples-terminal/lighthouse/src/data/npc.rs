//! Non-player characters.

use crate::data::ids::{DialogueId, RoomId};
use crate::data::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A character the player can encounter and optionally talk to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    pub name: String,
    pub synonyms: Vec<String>,
    pub description: Text,
    /// Dialogue graph triggered by "talk to" choices.
    pub dialogue: Option<DialogueId>,
    /// Where the NPC starts. `None` means the NPC begins offstage and must be placed by a rule.
    pub initial_room: Option<RoomId>,
    /// Starting disposition; higher values can unlock dialogue branches or
    /// quest transitions.
    pub initial_disposition: i64,
    /// Free-form tags.
    pub tags: BTreeSet<String>,
}

impl Npc {
    pub fn new(name: impl Into<String>, description: Text) -> Self {
        Self {
            name: name.into(),
            synonyms: Vec::new(),
            description,
            dialogue: None,
            initial_room: None,
            initial_disposition: 0,
            tags: BTreeSet::new(),
        }
    }

    pub fn with_synonyms(mut self, synonyms: impl IntoIterator<Item = &'static str>) -> Self {
        self.synonyms.extend(synonyms.into_iter().map(String::from));
        self
    }

    pub fn with_dialogue(mut self, dialogue: DialogueId) -> Self {
        self.dialogue = Some(dialogue);
        self
    }

    pub fn starting_in(mut self, room: RoomId) -> Self {
        self.initial_room = Some(room);
        self
    }

    pub fn with_disposition(mut self, disposition: i64) -> Self {
        self.initial_disposition = disposition;
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }
}
