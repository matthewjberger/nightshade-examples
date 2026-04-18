//! Dialogue graphs.
//!
//! A dialogue is a small graph of nodes (with narrative text and entry
//! effects) connected by labelled options. The engine tracks an active
//! dialogue in `RuntimeState.active_dialogue` and offers its current node's
//! options as the turn's choices until an option ends the dialogue.

use crate::data::condition::Condition;
use crate::data::effect::Effect;
use crate::data::ids::NodeId;
use crate::data::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A complete conversation graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    /// The node the dialogue begins at.
    pub start: NodeId,
    /// All nodes in the graph keyed by id. Nodes point at each other via the
    /// `goto` field on their options.
    pub nodes: BTreeMap<NodeId, DialogueNode>,
}

impl Dialogue {
    /// Start a new dialogue whose first node is `start`.
    pub fn new(start: NodeId) -> Self {
        Self {
            start,
            nodes: BTreeMap::new(),
        }
    }

    /// Insert a node into the dialogue graph keyed by its id.
    pub fn with_node(mut self, id: NodeId, node: DialogueNode) -> Self {
        self.nodes.insert(id, node);
        self
    }
}

/// A single beat of conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    /// The NPC's line at this point.
    pub text: Text,
    /// Effects run when the engine first enters this node.
    pub on_enter: Vec<Effect>,
    /// Player responses.
    pub options: Vec<DialogueOption>,
}

impl DialogueNode {
    pub fn new(text: Text) -> Self {
        Self {
            text,
            on_enter: Vec::new(),
            options: Vec::new(),
        }
    }

    pub fn with_on_enter(mut self, effects: Vec<Effect>) -> Self {
        self.on_enter = effects;
        self
    }

    pub fn with_option(mut self, option: DialogueOption) -> Self {
        self.options.push(option);
        self
    }
}

/// A choice the player can pick inside a dialogue node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueOption {
    /// Player-facing label.
    pub label: Text,
    /// If present, the option is only selectable when the condition holds.
    pub condition: Option<Condition>,
    /// If true, the option is shown greyed out with `locked_reason` when its
    /// condition does not hold; otherwise it is omitted from the menu.
    pub visible_when_locked: bool,
    /// Message shown next to a greyed-out option.
    pub locked_reason: Option<Text>,
    /// Effects to run when the player picks this option.
    pub effects: Vec<Effect>,
    /// If `Some`, the dialogue jumps to that node after running effects.
    /// If `None`, the dialogue ends.
    pub goto: Option<NodeId>,
}

impl DialogueOption {
    pub fn new(label: Text) -> Self {
        Self {
            label,
            condition: None,
            visible_when_locked: false,
            locked_reason: None,
            effects: Vec::new(),
            goto: None,
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

    pub fn with_effects(mut self, effects: Vec<Effect>) -> Self {
        self.effects = effects;
        self
    }

    pub fn goto(mut self, node: NodeId) -> Self {
        self.goto = Some(node);
        self
    }
}
