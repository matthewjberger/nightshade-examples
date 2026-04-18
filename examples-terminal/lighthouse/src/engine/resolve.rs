//! Text resolution: `Text` -> `String`.
//!
//! Pure function of `World + RuntimeState` (except for `OneOf`, which consumes
//! a random number from the RNG and therefore takes `&mut RuntimeState`).
//!
//! The public [`resolve`] entry point takes an immutable state to stay cheap
//! for view-layer use; it resolves `OneOf` by always picking the first
//! branch. When the engine resolves text during effect execution, it uses
//! [`resolve_mut`] so `OneOf` is actually randomized.

use crate::data::{FlagKey, RuntimeState, StatKey, Text, TextId, Value};
use crate::engine::{Engine, eval};

/// Immutable resolver: `OneOf` picks its first variant.
pub fn resolve(engine: &Engine, state: &RuntimeState, text: &Text) -> String {
    resolve_inner(engine, state, text, None)
}

/// Mutable resolver: `OneOf` consumes RNG for random selection.
pub fn resolve_mut(engine: &Engine, state: &mut RuntimeState, text: &Text) -> String {
    let mut output = String::new();
    resolve_with_rng(engine, state, text, &mut output);
    output
}

fn resolve_inner(
    engine: &Engine,
    state: &RuntimeState,
    text: &Text,
    rng_override: Option<&mut RuntimeState>,
) -> String {
    match text {
        Text::Literal(value) => value.clone(),
        Text::Ref(id) => resolve_ref(engine, state, id, rng_override),
        Text::Flag(key) => flag_display(state, key),
        Text::Stat(key) => stat_display(state, key),
        Text::Conditional {
            when,
            then,
            otherwise,
        } => {
            let branch = if eval::evaluate(engine, state, when) {
                then
            } else {
                otherwise
            };
            resolve_inner(engine, state, branch, rng_override)
        }
        Text::OneOf(variants) => {
            if variants.is_empty() {
                String::new()
            } else {
                resolve_inner(engine, state, &variants[0], rng_override)
            }
        }
        Text::Sequence(parts) => {
            let mut out = String::new();
            for part in parts {
                out.push_str(&resolve_inner(engine, state, part, None));
            }
            out
        }
    }
}

fn resolve_ref(
    engine: &Engine,
    state: &RuntimeState,
    id: &TextId,
    rng_override: Option<&mut RuntimeState>,
) -> String {
    match engine.world().texts.get(id) {
        Some(inner) => resolve_inner(engine, state, inner, rng_override),
        None => format!("<missing text {id}>"),
    }
}

fn flag_display(state: &RuntimeState, key: &FlagKey) -> String {
    match state.flags.get(key) {
        Some(Value::Bool(value)) => if *value { "true" } else { "false" }.to_string(),
        Some(Value::Int(value)) => value.to_string(),
        Some(Value::Text(value)) => value.clone(),
        None => String::new(),
    }
}

fn stat_display(state: &RuntimeState, key: &StatKey) -> String {
    state.stats.get(key).copied().unwrap_or(0).to_string()
}

fn resolve_with_rng(engine: &Engine, state: &mut RuntimeState, text: &Text, out: &mut String) {
    match text {
        Text::Literal(value) => out.push_str(value),
        Text::Ref(id) => {
            if let Some(inner) = engine.world().texts.get(id).cloned() {
                resolve_with_rng(engine, state, &inner, out);
            } else {
                out.push_str("<missing text ");
                out.push_str(id.as_str());
                out.push('>');
            }
        }
        Text::Flag(key) => out.push_str(&flag_display(state, key)),
        Text::Stat(key) => out.push_str(&stat_display(state, key)),
        Text::Conditional {
            when,
            then,
            otherwise,
        } => {
            let branch = if eval::evaluate(engine, state, when) {
                then.as_ref()
            } else {
                otherwise.as_ref()
            };
            let owned = branch.clone();
            resolve_with_rng(engine, state, &owned, out);
        }
        Text::OneOf(variants) => {
            if !variants.is_empty() {
                let pick = state.random_index(variants.len());
                let chosen = variants[pick].clone();
                resolve_with_rng(engine, state, &chosen, out);
            }
        }
        Text::Sequence(parts) => {
            for part in parts {
                resolve_with_rng(engine, state, part, out);
            }
        }
    }
}
