//! The shore, the cliff path, and the "gone" sink — plus the driftwood and
//! cottage key the player finds on the shingle.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{Condition, Exit, Item, ItemProperties, Room, Text};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_room(
        ids::room_shore(),
        Room::new(
            "The Shingle Shore",
            Text::lit(
                "Grey waves crash against a slope of wet black stones. The lighthouse cottage huddles in the lee of the cliff, its single dark window watching the sea. The path north climbs to the cottage door; the cliff path rises to the east.",
            ),
        )
        .with_exit(
            Exit::new("north (to the cottage)", ids::room_cottage()).gated(
                Condition::FlagSet(ids::flag_cottage_unlocked()),
                Text::lit("The cottage door is locked. You'll need a key."),
            ),
        )
        .with_exit(Exit::new("east (up the cliff path)", ids::room_cliff_path()))
        .with_examine(
            "sea",
            Text::lit(
                "The water is rough, restless. Far out, you can just make out the white bow of a fishing ketch fighting its way towards the headland.",
            ),
        )
        .with_examine(
            "cottage",
            Text::lit("A low stone cottage with a slate roof and a heavy iron-banded door."),
        ),
    );

    area.add_room(
        ids::room_cliff_path(),
        Room::new(
            "The Cliff Path",
            Text::lit(
                "A narrow path along the top of the headland. Far below, the sea. To the west the path winds inland and away from the point.",
            ),
        )
        .with_exit(Exit::new("west (away from the lighthouse)", ids::room_gone()))
        .with_exit(Exit::new("down (back to the shore)", ids::room_shore()))
        .with_examine(
            "sea",
            Text::lit(
                "A fishing ketch is visible now, much closer, its sails half-down, pitching badly in the swell.",
            ),
        ),
    );

    // Terminal sink: entering this room triggers the "safe ashore" ending
    // via that ending's `PlayerIn(room_gone)` condition.
    area.add_room(
        ids::room_gone(),
        Room::new(
            "Away from the Headland",
            Text::lit("You walk away from the point. The wind eases as the cliff shelters you."),
        ),
    );

    area.add_item(
        ids::item_driftwood(),
        Item::new(
            "piece of driftwood",
            Text::lit("a salt-bleached piece of driftwood"),
            Text::lit("A forearm-length of driftwood, bleached pale. Heavy enough to swing."),
        )
        .with_synonyms(["wood", "stick"])
        .takeable()
        .initially_in(ids::room_shore()),
    );

    area.add_item(
        ids::item_cottage_key(),
        Item::new(
            "iron key",
            Text::lit("a small iron key, half-buried in the shingle"),
            Text::lit("A stubby iron key. The metal is cold even for this weather."),
        )
        .with_synonyms(["cottage key", "key"])
        .with_properties(ItemProperties {
            takeable: true,
            ..Default::default()
        })
        .with_tag("opens_cottage")
        .initially_in(ids::room_shore()),
    );

    area
}
