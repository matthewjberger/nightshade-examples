//! Items.
//!
//! An item's *location* is not stored on the item itself. It lives in
//! [`crate::data::RuntimeState::item_locations`] as an [`crate::data::ItemLocation`].
//! Querying "what's in this room" or "what does the player carry" is a scan
//! over that map.

use crate::data::ids::{FlagKey, NpcId, RoomId};
use crate::data::state::ItemLocation;
use crate::data::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A carryable or interactable object in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Short display name ("rusty key").
    pub name: String,
    /// Alternative nouns the player might type ("key", "old key").
    pub synonyms: Vec<String>,
    /// One-line description shown in room listings.
    pub short: Text,
    /// Longer description shown on examine.
    pub long: Text,
    /// Text shown on "read" for readable items.
    pub read: Option<Text>,
    /// Behavioral flags.
    pub properties: ItemProperties,
    /// Where the item is placed at the start of a run. `None` means the item
    /// begins in `ItemLocation::Nowhere` and must be moved onto the stage by
    /// a rule (e.g. the keeper's body, revealed by the `found_keeper` rule).
    pub initial_location: Option<ItemLocation>,
    /// Free-form tags for rule matching.
    pub tags: BTreeSet<String>,
}

impl Item {
    pub fn new(name: impl Into<String>, short: Text, long: Text) -> Self {
        Self {
            name: name.into(),
            synonyms: Vec::new(),
            short,
            long,
            read: None,
            properties: ItemProperties::default(),
            initial_location: None,
            tags: BTreeSet::new(),
        }
    }

    /// Place this item in the given room at the start of play.
    pub fn initially_in(mut self, room: RoomId) -> Self {
        self.initial_location = Some(ItemLocation::Room(room));
        self
    }

    /// Place this item in the player's inventory at the start of play.
    pub fn initially_carried(mut self) -> Self {
        self.initial_location = Some(ItemLocation::Inventory);
        self
    }

    /// Start the item held by an NPC.
    pub fn initially_held_by(mut self, npc: NpcId) -> Self {
        self.initial_location = Some(ItemLocation::HeldBy(npc));
        self
    }

    pub fn with_synonyms(mut self, synonyms: impl IntoIterator<Item = &'static str>) -> Self {
        self.synonyms.extend(synonyms.into_iter().map(String::from));
        self
    }

    pub fn with_read(mut self, text: Text) -> Self {
        self.read = Some(text);
        self.properties.readable = true;
        self
    }

    pub fn takeable(mut self) -> Self {
        self.properties.takeable = true;
        self
    }

    pub fn with_properties(mut self, properties: ItemProperties) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }
}

/// Static behavioral flags describing what the player can do with an item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemProperties {
    /// Can be picked up into inventory.
    pub takeable: bool,
    /// Can be worn.
    pub wearable: bool,
    /// Has `read` text.
    pub readable: bool,
    /// Can be opened/closed.
    pub openable: bool,
    /// Has a lock.
    pub lockable: bool,
    /// Emits light when its `lit_flag` is currently set to `Value::Bool(true)`.
    pub light_source: bool,
    /// The flag that tracks whether this light source is currently lit.
    /// Only meaningful when `light_source` is true. `None` disables the item
    /// as a functional light source even if `light_source` is set.
    pub lit_flag: Option<FlagKey>,
    /// Relative weight for future inventory-limit rules.
    pub weight: u32,
    /// Relative size for future container-fit rules.
    pub size: u32,
}
