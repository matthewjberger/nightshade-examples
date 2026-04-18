//! Small pure helpers shared across engine submodules.

use crate::data::{ItemLocation, RuntimeState, Value};
use crate::engine::Engine;

/// True when the player's inventory contains a lit light source.
///
/// A light source is considered currently lit when the item has
/// `properties.light_source == true` and its configured `lit_flag` is
/// currently set to `Value::Bool(true)`. Items whose `lit_flag` is `None`
/// are treated as non-functional as light sources.
pub fn player_has_light(engine: &Engine, state: &RuntimeState) -> bool {
    state.item_locations.iter().any(|(item_id, location)| {
        if !matches!(location, ItemLocation::Inventory) {
            return false;
        }
        let Some(item) = engine.world().items.get(item_id) else {
            return false;
        };
        if !item.properties.light_source {
            return false;
        }
        let Some(flag) = item.properties.lit_flag.as_ref() else {
            return false;
        };
        matches!(state.flags.get(flag), Some(Value::Bool(true)))
    })
}
