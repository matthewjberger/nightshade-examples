//! Cross-area ID deduplication helper.
//!
//! Each content table (rooms, items, rules, etc.) is stitched together
//! from per-area modules. If two areas happen to register the same ID,
//! the later insert would silently win — this helper panics so the
//! collision surfaces at boot or in the first test run.

use std::collections::BTreeMap;
use std::fmt::Display;

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
