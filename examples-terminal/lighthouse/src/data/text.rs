//! Narrative text with light composition.
//!
//! Every player-facing string in the data layer is a [`Text`] value. Resolving
//! a `Text` into a final `String` requires `RuntimeState` context and happens
//! in [`crate::engine::resolve`].

use crate::data::condition::Condition;
use crate::data::ids::{FlagKey, StatKey, TextId};
use serde::{Deserialize, Serialize};

/// A composable piece of narrative text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Text {
    /// A literal string, written as-is.
    Literal(String),
    /// Look up another `Text` by ID from `World.texts`.
    Ref(TextId),
    /// Inline the current textual representation of a flag's value.
    Flag(FlagKey),
    /// Inline the current value of a stat.
    Stat(StatKey),
    /// Pick one of two texts based on a condition.
    Conditional {
        when: Condition,
        then: Box<Text>,
        otherwise: Box<Text>,
    },
    /// Pick one of several variants using the runtime RNG.
    OneOf(Vec<Text>),
    /// Concatenate several texts in order.
    Sequence(Vec<Text>),
}

impl Text {
    /// Convenience constructor for literal strings.
    pub fn lit(value: impl Into<String>) -> Self {
        Text::Literal(value.into())
    }

    /// Empty literal text, useful as a default.
    pub fn empty() -> Self {
        Text::Literal(String::new())
    }
}

impl Default for Text {
    fn default() -> Self {
        Text::empty()
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Text::Literal(value.to_string())
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Text::Literal(value)
    }
}
