//! A rule: "when this event fires and this condition holds, run these effects."

use crate::data::condition::Condition;
use crate::data::effect::Effect;
use crate::data::trigger::Trigger;
use serde::{Deserialize, Serialize};

/// A data-driven production rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// The event that may cause this rule to fire.
    pub trigger: Trigger,
    /// Higher priority rules fire first when multiple rules match the same event.
    /// Ties are broken deterministically by the rule's ID.
    pub priority: i32,
    /// Extra gate. Evaluated after the trigger matches. `Condition::Always` to always fire.
    pub condition: Condition,
    /// Effects to run when the rule fires, in order.
    pub effects: Vec<Effect>,
    /// If true, the rule fires at most once per game.
    pub once: bool,
    /// Minimum number of turns between consecutive firings. `0` means no cooldown.
    pub cooldown_turns: u32,
}

impl Rule {
    /// Convenience constructor for the common case of "always-on rule with no
    /// special conditions."
    pub fn on(trigger: Trigger, effects: Vec<Effect>) -> Self {
        Self {
            trigger,
            priority: 0,
            condition: Condition::Always,
            effects,
            once: false,
            cooldown_turns: 0,
        }
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = condition;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn once(mut self) -> Self {
        self.once = true;
        self
    }

    pub fn with_cooldown(mut self, turns: u32) -> Self {
        self.cooldown_turns = turns;
        self
    }
}
