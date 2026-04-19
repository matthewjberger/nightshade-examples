//! Endings.

use crate::game::ids;
use nightshade::interactive_fiction::data::{Condition, Ending, EndingId, Text};
use std::collections::BTreeMap;

pub fn build() -> BTreeMap<EndingId, Ending> {
    let mut endings = BTreeMap::new();

    // Canonical success: relit the lantern.
    endings.insert(
        ids::ending_the_lantern_burns(),
        Ending::new(
            "The Lantern Burns",
            Text::lit(
                "Light leaps out from the prisms in a clean white beam. Below, the fishing ketch corrects its course and finds the channel. The storm breaks, and the beam turns, and turns, all night.",
            ),
            Text::lit(
                "You tell them in the village what you found in the cellar, and what was written in the second ledger. Dunmere Light burns on through a new keeper and the next, and the names in that ledger never come to port again.",
            ),
            Condition::FlagSet(ids::flag_lantern_restored()),
        )
        .with_priority(10)
        .with_tag("success"),
    );

    // Grim success: took the bribe.
    endings.insert(
        ids::ending_the_wreckers_gold(),
        Ending::new(
            "The Wreckers' Gold",
            Text::lit(
                "The ketch breaks on the headland at three in the morning. You count the pieces of eight by the light of the stranger's pipe. The money is good. It buys a boat of your own, in time, and a small house, and a silence.",
            ),
            Text::lit(
                "You never speak of that night. No-one asks. Dunmere Light is repaired the following spring, and a new keeper comes, and you are careful never to walk the cliff path when the wind is from the east.",
            ),
            Condition::FlagSet(ids::flag_lantern_sabotaged()),
        )
        .with_priority(10)
        .with_tag("grim"),
    );

    // Cautious flight: walked away before anything broke.
    endings.insert(
        ids::ending_safe_ashore(),
        Ending::new(
            "Safe Ashore",
            Text::lit(
                "You walk inland. The storm passes behind you. Behind you, too, the sound of a ship breaking up against the headland, very far away.",
            ),
            Text::lit(
                "You read about the wreck in the newspaper three days later. Seven crew lost. You do not talk about the lighthouse again, but you do not sleep as well as you used to.",
            ),
            Condition::PlayerIn(ids::room_gone()),
        )
        .with_priority(5)
        .with_tag("neutral"),
    );

    // Failure: storm timer expired without a decision.
    endings.insert(
        ids::ending_lost_to_the_storm(),
        Ending::new(
            "Lost to the Storm",
            Text::lit(
                "You are still inside the tower when the storm comes over the headland in full. The cliff shakes; the glass in the lantern room cracks; the sea comes up like a hand and takes the rocks where the ketch went down.",
            ),
            Text::lit(
                "No-one finds you. Dunmere Light is dark for a season until a new keeper is appointed, and the shingle below the headland is heavy with timbers all that year.",
            ),
            Condition::TimerExpired(ids::timer_storm()),
        )
        .with_priority(20)
        .with_tag("failure"),
    );

    endings
}
