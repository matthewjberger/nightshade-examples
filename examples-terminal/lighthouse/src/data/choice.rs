//! Player choices.
//!
//! Each turn, the engine assembles a menu of [`Choice`] values from:
//! - visible exits from the current room,
//! - visible items in the room (take, examine),
//! - items in inventory (drop, use, read, examine),
//! - NPCs in the room (talk),
//! - the active dialogue's options if one is running,
//! - any `Effect::OfferChoices` currently in effect.
//!
//! A choice, when picked, either enacts a structured [`ChoiceAction`] or runs
//! a free-form list of effects.

use crate::data::condition::Condition;
use crate::data::effect::Effect;
use crate::data::ids::{ItemId, NpcId, RoomId};
use crate::data::text::Text;
use serde::{Deserialize, Serialize};

/// A menu entry offered to the player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Player-facing label.
    pub label: Text,
    /// Optional gate. If unset, the choice is always selectable.
    pub condition: Option<Condition>,
    /// If true, the choice appears greyed out (with `locked_reason`) when its
    /// condition does not hold.
    pub visible_when_locked: bool,
    /// Message shown next to a greyed-out entry.
    pub locked_reason: Option<Text>,
    /// What picking the choice does.
    pub action: ChoiceAction,
}

impl Choice {
    pub fn new(label: Text, action: ChoiceAction) -> Self {
        Self {
            label,
            condition: None,
            visible_when_locked: false,
            locked_reason: None,
            action,
        }
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn visible_when_locked(mut self, reason: Text) -> Self {
        self.visible_when_locked = true;
        self.locked_reason = Some(reason);
        self
    }
}

/// What a [`Choice`] does when picked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChoiceAction {
    /// Move to the target room through a specific exit index.
    Go {
        to: RoomId,
        exit_index: usize,
    },
    /// Take the item from the current room into inventory.
    Take(ItemId),
    /// Drop the item from inventory into the current room.
    Drop(ItemId),
    /// Use the item in the current context (fires `Trigger::OnUse`).
    Use(ItemId),
    /// Examine the item (fires `Trigger::OnExamine`).
    Examine(ItemId),
    /// Examine a room feature by keyword.
    ExamineKeyword(String),
    /// Read the item's `read` text.
    Read(ItemId),
    /// Start talking to the NPC.
    TalkTo(NpcId),
    /// Pick option `index` inside the active dialogue node.
    DialogueOption(usize),
    /// End the active dialogue.
    LeaveDialogue,
    /// Inspect current room / inventory etc. without advancing a turn.
    Look,
    Inventory,
    /// Pass the turn.
    Wait,
    /// Run arbitrary effects.
    Effects(Vec<Effect>),
}
