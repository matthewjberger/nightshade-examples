//! Runtime layer. Interprets a [`crate::data::World`] and mutates a
//! [`crate::data::RuntimeState`].
//!
//! The engine has no knowledge of any specific game. The same `Engine` can
//! run any `World`. Public surface is deliberately small:
//!
//! ```ignore
//! let engine = Engine::new(world)?;         // builds indices + validates
//! let mut state = engine.start_state();      // seeds RuntimeState from World
//! engine.start(&mut state);                  // fires GameStart, DescribeRoom
//! for choice in engine.available_choices(&state) { /* render */ }
//! engine.pick(&mut state, index);            // apply input, advance a turn
//! ```

pub mod choices;
pub mod dispatch;
pub mod eval;
pub mod exec;
pub mod helpers;
pub mod resolve;
pub mod save;
pub mod turn;
pub mod validate;

use crate::data::{Choice, RuntimeState, World};
use std::sync::Arc;

pub use validate::{ValidationError, validate};

/// The text-adventure interpreter.
///
/// Wraps a `World` behind an `Arc` so the view layer (and undo snapshots) can
/// cheaply share ownership. The engine itself is stateless across calls; all
/// mutable state lives in the `RuntimeState` passed in by the caller.
pub struct Engine {
    world: Arc<World>,
    /// Rule IDs bucketed by their trigger kind for fast dispatch.
    /// Derived from `world.rules`; never serialized.
    rule_index: dispatch::RuleIndex,
    /// When true, every rule fire emits a system-channel transcript entry
    /// with the rule ID. Off by default; flip on during authoring to trace
    /// why a particular behaviour did or did not fire.
    trace_rules: bool,
}

impl Engine {
    /// Build an engine from a world. Runs load-time validation; if any
    /// content-level error is found, returns the collected list.
    pub fn new(world: World) -> Result<Self, Vec<ValidationError>> {
        validate(&world)?;
        let world = Arc::new(world);
        let rule_index = dispatch::RuleIndex::build(&world);
        Ok(Self {
            world,
            rule_index,
            trace_rules: false,
        })
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// Enable or disable per-rule-fire transcript tracing. Intended for
    /// authoring. Each fired rule appends a `TranscriptEntry::System` line
    /// like `"[trace] rule 'cottage_unlocked' fired"`.
    pub fn set_rule_tracing(&mut self, enabled: bool) {
        self.trace_rules = enabled;
    }

    /// Whether rule tracing is currently enabled.
    pub fn rule_tracing_enabled(&self) -> bool {
        self.trace_rules
    }

    /// Construct a fresh `RuntimeState` seeded from the world's initial values.
    pub fn start_state(&self) -> RuntimeState {
        let mut state = RuntimeState {
            current_room: self.world.start_room.clone(),
            rng_state: 0x9E3779B97F4A7C15,
            ..RuntimeState::default()
        };

        for (item_id, item) in &self.world.items {
            let location = item
                .initial_location
                .clone()
                .unwrap_or(crate::data::ItemLocation::Nowhere);
            state.item_locations.insert(item_id.clone(), location);
        }
        for (npc_id, npc) in &self.world.npcs {
            if let Some(room) = &npc.initial_room {
                state.npc_locations.insert(npc_id.clone(), room.clone());
            }
            state
                .dispositions
                .insert(npc_id.clone(), npc.initial_disposition);
        }
        for (quest_id, quest) in &self.world.quests {
            state
                .quest_stages
                .insert(quest_id.clone(), quest.start.clone());
            let mut seen = std::collections::BTreeSet::new();
            seen.insert(quest.start.clone());
            state.quest_history.insert(quest_id.clone(), seen);
        }

        state
    }

    /// Fire `Trigger::GameStart`, print the intro and start room, and run
    /// quest start-stage effects.
    pub fn start(&self, state: &mut RuntimeState) {
        turn::start(self, state);
    }

    /// Apply the player's picked choice (by index into
    /// `available_choices`). Advances one turn if the action is a turn-taker.
    pub fn pick(&self, state: &mut RuntimeState, index: usize) {
        turn::pick(self, state, index);
    }

    /// Build the menu of choices currently available to the player.
    pub fn available_choices(&self, state: &RuntimeState) -> Vec<Choice> {
        choices::assemble(self, state)
    }

    /// Resolve a [`crate::data::Text`] value to a concrete `String` in the
    /// current state. Useful for the view layer.
    pub fn resolve_text(&self, state: &RuntimeState, text: &crate::data::Text) -> String {
        resolve::resolve(self, state, text)
    }

    /// Internal access for submodules that need both world and rule index.
    pub(crate) fn rule_index(&self) -> &dispatch::RuleIndex {
        &self.rule_index
    }
}
