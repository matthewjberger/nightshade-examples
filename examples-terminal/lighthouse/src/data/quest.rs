//! Quest graphs.
//!
//! A quest has a starting stage; each stage is either active, a success, or a
//! failure, and has a list of transitions to other stages guarded by
//! conditions. The engine advances the quest as transitions become satisfied.

use crate::data::condition::Condition;
use crate::data::effect::Effect;
use crate::data::ids::NodeId;
use crate::data::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A multi-stage objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub name: String,
    pub start: NodeId,
    pub stages: BTreeMap<NodeId, QuestStage>,
}

impl Quest {
    /// Begin a new quest with a display name and a starting stage id.
    pub fn new(name: impl Into<String>, start: NodeId) -> Self {
        Self {
            name: name.into(),
            start,
            stages: BTreeMap::new(),
        }
    }

    /// Attach a stage to the quest graph under its id.
    pub fn with_stage(mut self, id: NodeId, stage: QuestStage) -> Self {
        self.stages.insert(id, stage);
        self
    }
}

/// A single state in the quest graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestStage {
    pub kind: QuestStageKind,
    /// Short description shown in a "journal" view.
    pub description: Text,
    /// Effects run when the stage is first entered.
    pub on_enter: Vec<Effect>,
    /// Outgoing transitions. Evaluated in order; the first whose condition
    /// holds is taken.
    pub transitions: Vec<QuestTransition>,
}

impl QuestStage {
    pub fn active(description: Text) -> Self {
        Self {
            kind: QuestStageKind::Active,
            description,
            on_enter: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn success(description: Text) -> Self {
        Self {
            kind: QuestStageKind::Success,
            description,
            on_enter: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn failure(description: Text) -> Self {
        Self {
            kind: QuestStageKind::Failure,
            description,
            on_enter: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn with_on_enter(mut self, effects: Vec<Effect>) -> Self {
        self.on_enter = effects;
        self
    }

    pub fn with_transition(mut self, transition: QuestTransition) -> Self {
        self.transitions.push(transition);
        self
    }
}

/// Terminal classification for quest stages.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStageKind {
    /// In progress. Should have at least one outgoing transition.
    Active,
    /// Terminal success state.
    Success,
    /// Terminal failure state.
    Failure,
}

/// A guarded edge between two quest stages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestTransition {
    pub to: NodeId,
    pub condition: Condition,
    /// Effects to run when the transition fires, before the destination's
    /// `on_enter` runs.
    pub effects: Vec<Effect>,
}

impl QuestTransition {
    pub fn new(to: NodeId, condition: Condition) -> Self {
        Self {
            to,
            condition,
            effects: Vec::new(),
        }
    }

    pub fn with_effects(mut self, effects: Vec<Effect>) -> Self {
        self.effects = effects;
        self
    }
}
