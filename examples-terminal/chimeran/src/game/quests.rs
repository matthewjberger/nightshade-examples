//! Chimeran has no formal quest graph — the cycle/ending logic is all rule-
//! and condition-driven. This module returns an empty map so the `World`
//! has a sensible default for the field.

use nightshade::interactive_fiction::data::{Quest, QuestId};
use std::collections::BTreeMap;

pub fn build() -> BTreeMap<QuestId, Quest> {
    BTreeMap::new()
}
