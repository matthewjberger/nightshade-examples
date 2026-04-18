//! Shared helper for aggregating per-area content contributions.
//!
//! Each top-level content table (`rooms`, `items`, `rules`, `npcs`,
//! `dialogues`, `timers`) is stitched together from per-area submodules.
//! Area authors accidentally reusing the same ID across two areas is a
//! class of bug that used to fail silently — later insertion simply
//! overwrote the earlier one. This helper panics on collision so the
//! problem surfaces at test or boot time, not hours into play.

use std::collections::BTreeMap;
use std::fmt::Display;

/// Merge `source` into `target`. Panics if any key in `source` already
/// exists in `target`.
pub fn merge<K, V>(target: &mut BTreeMap<K, V>, source: BTreeMap<K, V>)
where
    K: Ord + Display + Clone,
{
    for (key, value) in source {
        assert!(
            !target.contains_key(&key),
            "duplicate content id '{key}': the same id is registered by more than one area module"
        );
        target.insert(key, value);
    }
}
