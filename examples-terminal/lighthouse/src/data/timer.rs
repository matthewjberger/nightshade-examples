//! Timers.
//!
//! A timer decrements each turn while running. On each tick it runs
//! `on_tick` effects; when it reaches zero it runs `on_expire` and stops. A
//! timer can be cancelled mid-run by an effect or automatically when its
//! `cancel_on` condition holds.

use crate::data::condition::Condition;
use crate::data::effect::Effect;
use serde::{Deserialize, Serialize};

/// Content for a timer defined in `World.timers`. Actual per-run remaining
/// turns live in `RuntimeState.timers_remaining`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    /// Number of turns the timer runs for when started.
    pub initial_turns: u32,
    /// Effects run every turn the timer is still running (after decrement).
    pub on_tick: Vec<Effect>,
    /// Effects run the turn the timer hits zero. `on_tick` still runs that turn.
    pub on_expire: Vec<Effect>,
    /// Optional condition; when it holds, the timer is cancelled without
    /// running `on_expire`.
    pub cancel_on: Option<Condition>,
}

impl Timer {
    pub fn new(initial_turns: u32) -> Self {
        Self {
            initial_turns,
            on_tick: Vec::new(),
            on_expire: Vec::new(),
            cancel_on: None,
        }
    }

    pub fn with_on_tick(mut self, effects: Vec<Effect>) -> Self {
        self.on_tick = effects;
        self
    }

    pub fn with_on_expire(mut self, effects: Vec<Effect>) -> Self {
        self.on_expire = effects;
        self
    }

    pub fn cancel_on(mut self, condition: Condition) -> Self {
        self.cancel_on = Some(condition);
        self
    }
}
