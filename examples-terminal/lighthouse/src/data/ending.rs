//! Endings.
//!
//! An ending is the final state of a run. The engine evaluates endings after
//! every turn (when none is already active) and picks the highest-priority
//! ending whose `trigger` condition holds. Endings can also be forced
//! explicitly via `Effect::TriggerEnding`.

use crate::data::condition::Condition;
use crate::data::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A terminal conclusion for a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ending {
    /// Short title shown on the game-over screen.
    pub title: String,
    /// Paragraph shown as the immediate outcome.
    pub description: Text,
    /// Epilogue shown after the description.
    pub epilogue: Text,
    /// Condition that causes the ending to fire when evaluated in-line.
    /// When the ending is forced via `Effect::TriggerEnding`, this is ignored.
    pub trigger: Condition,
    /// Higher priority endings override lower ones when multiple trigger the same turn.
    pub priority: i32,
    /// Free-form tags used when listing unlocked endings.
    pub tags: BTreeSet<String>,
}

impl Ending {
    pub fn new(
        title: impl Into<String>,
        description: Text,
        epilogue: Text,
        trigger: Condition,
    ) -> Self {
        Self {
            title: title.into(),
            description,
            epilogue,
            trigger,
            priority: 0,
            tags: BTreeSet::new(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }
}
