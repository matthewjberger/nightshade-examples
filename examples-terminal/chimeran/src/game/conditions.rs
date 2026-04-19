//! Shared condition table. Empty in the slice — every gate in the game
//! is a short inline `Condition::All(...)` and the deduplication `Ref`
//! pattern hasn't paid for itself yet. Kept as a module so the table
//! can grow without restructuring `build_world`.

use nightshade::interactive_fiction::data::{Condition, ConditionId};
use std::collections::BTreeMap;

pub fn build() -> BTreeMap<ConditionId, Condition> {
    BTreeMap::new()
}
