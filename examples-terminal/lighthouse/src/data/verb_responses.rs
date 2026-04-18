//! Every player-facing string the engine emits on its own initiative.
//!
//! The engine never hard-codes flavour text. When it needs to narrate a
//! take, a drop, a dialogue speaker fallback, or a choice-menu label, it
//! reads the relevant field from `World.verb_responses` and substitutes
//! any `{placeholder}` tokens via [`VerbResponses::render`].
//!
//! Authors can override any field when building a world; the defaults are
//! plain English and match what the engine used to hard-code. Tokens per
//! field are documented inline.

use serde::{Deserialize, Serialize};

/// The full set of templates the engine draws on. Placeholders are
/// `{item}`, `{npc}`, `{dir}`, `{keyword}`, `{room}`, `{rule}`, depending
/// on the field. Fields without any placeholder are literal strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbResponses {
    // ---- Choice-menu labels --------------------------------------------
    /// Template: `{dir}`.
    pub choice_go: String,
    /// Template: `{item}`.
    pub choice_take: String,
    /// Template: `{item}`.
    pub choice_examine_item: String,
    /// Template: `{item}`.
    pub choice_use: String,
    /// Template: `{item}`.
    pub choice_read: String,
    /// Template: `{item}`.
    pub choice_drop: String,
    /// Template: `{npc}`.
    pub choice_talk: String,
    /// Template: `{keyword}`.
    pub choice_examine_feature: String,
    pub choice_look: String,
    pub choice_inventory: String,
    pub choice_wait: String,
    pub choice_leave_dialogue: String,
    /// Fallback when a passable-when-locked exit has no `locked_message`.
    pub exit_locked_default: String,

    // ---- Room description ----------------------------------------------
    /// Template: `{name}`.
    pub room_header: String,
    /// Prefix preceding a comma-joined list of visible entities.
    /// Combined string shape: `"{visible_listing_prefix}X, Y, Z."`
    pub visible_listing_prefix: String,

    // ---- Verb responses ------------------------------------------------
    pub inventory_empty: String,
    /// Prefix preceding a comma-joined list of inventory items.
    pub inventory_listing_prefix: String,
    /// Template: `{item}`.
    pub take_success: String,
    /// Template: `{item}`.
    pub take_already_carrying: String,
    /// Template: `{item}`.
    pub take_not_takeable: String,
    /// Template: `{item}`.
    pub drop_success: String,
    /// Template: `{item}`.
    pub use_not_carrying: String,
    /// Template: `{item}`.
    pub read_nothing_written: String,
    pub examine_unknown: String,
    /// Template: `{npc}`.
    pub npc_silent: String,
    pub leave_dialogue: String,
    pub wait: String,
    /// Fallback dialogue speaker label if no NPC references the dialogue.
    pub dialogue_default_speaker: String,

    // ---- Meta ----------------------------------------------------------
    pub option_unavailable: String,
    pub action_forbidden: String,
    /// Template: `{rule}`. Emitted only when rule tracing is enabled.
    pub trace_prefix: String,
}

impl Default for VerbResponses {
    fn default() -> Self {
        Self {
            choice_go: "Go {dir}".to_string(),
            choice_take: "Take the {item}".to_string(),
            choice_examine_item: "Examine the {item}".to_string(),
            choice_use: "Use the {item}".to_string(),
            choice_read: "Read the {item}".to_string(),
            choice_drop: "Drop the {item}".to_string(),
            choice_talk: "Talk to {npc}".to_string(),
            choice_examine_feature: "Look at the {keyword}".to_string(),
            choice_look: "Look".to_string(),
            choice_inventory: "Check inventory".to_string(),
            choice_wait: "Wait".to_string(),
            choice_leave_dialogue: "Leave the conversation".to_string(),
            exit_locked_default: "locked.".to_string(),

            room_header: "--- {name} ---".to_string(),
            visible_listing_prefix: "You see: ".to_string(),

            inventory_empty: "You are carrying nothing.".to_string(),
            inventory_listing_prefix: "You are carrying: ".to_string(),
            take_success: "You take the {item}.".to_string(),
            take_already_carrying: "You are already carrying the {item}.".to_string(),
            take_not_takeable: "The {item} won't move.".to_string(),
            drop_success: "You drop the {item}.".to_string(),
            use_not_carrying: "You aren't carrying the {item}.".to_string(),
            read_nothing_written: "There is nothing to read on the {item}.".to_string(),
            examine_unknown: "You see nothing special about that.".to_string(),
            npc_silent: "{npc} has nothing to say.".to_string(),
            leave_dialogue: "You step away from the conversation.".to_string(),
            wait: "Time drifts past.".to_string(),
            dialogue_default_speaker: "Voice".to_string(),

            option_unavailable: "That option is not available.".to_string(),
            action_forbidden: "You cannot do that.".to_string(),
            trace_prefix: "[trace] rule '{rule}' fired".to_string(),
        }
    }
}

impl VerbResponses {
    /// Substitute every occurrence of each [`Placeholder`] token in
    /// `template` with the paired value. Missing placeholders are left
    /// alone — useful if an author wants to embed literal braces.
    pub fn render(template: &str, replacements: &[(Placeholder, &str)]) -> String {
        let mut output = template.to_string();
        for (placeholder, value) in replacements {
            output = output.replace(placeholder.token(), value);
        }
        output
    }
}

/// Every placeholder name a [`VerbResponses`] template may contain. Using
/// this enum at call sites guarantees we can't mistype a token like
/// `"itme"` or use one that the struct docs don't advertise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placeholder {
    /// `{item}` — an item's display name.
    Item,
    /// `{npc}` — an NPC's display name.
    Npc,
    /// `{dir}` — an exit's direction label.
    Dir,
    /// `{keyword}` — a room's examine-feature keyword.
    Keyword,
    /// `{name}` — a room's display name.
    Name,
    /// `{rule}` — a rule's ID.
    Rule,
    /// `{items}` — a comma-joined list of item names.
    Items,
    /// `{things}` — a comma-joined list of visible entities.
    Things,
}

impl Placeholder {
    /// The literal `{name}` token this placeholder replaces.
    pub const fn token(self) -> &'static str {
        match self {
            Placeholder::Item => "{item}",
            Placeholder::Npc => "{npc}",
            Placeholder::Dir => "{dir}",
            Placeholder::Keyword => "{keyword}",
            Placeholder::Name => "{name}",
            Placeholder::Rule => "{rule}",
            Placeholder::Items => "{items}",
            Placeholder::Things => "{things}",
        }
    }
}
