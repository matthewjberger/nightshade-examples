//! Shared predicate table referenced via `Condition::Ref`.
//!
//! Stand-alone conditions that appear in more than one rule, quest, or
//! timer live here so they can be authored once and reused by ID.

use crate::data::{Condition, ConditionId};
use crate::game::ids;
use std::collections::BTreeMap;

pub fn build() -> BTreeMap<ConditionId, Condition> {
    let mut conditions = BTreeMap::new();
    conditions.insert(
        ids::cond_has_lit_lantern(),
        Condition::All(vec![
            Condition::HasItem(ids::item_lantern()),
            Condition::FlagSet(ids::flag_lantern_is_lit()),
        ]),
    );
    conditions
}
