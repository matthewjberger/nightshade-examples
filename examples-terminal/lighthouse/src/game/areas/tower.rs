//! The tower: its three rooms, the tower key the player finds inside,
//! every puzzle rule tied to the lens, and the hidden-passage backup path.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Effect, Exit, Item, Room, Rule, Text, Trigger, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_room(
        ids::room_tower_base(),
        Room::new(
            "The Base of the Tower",
            Text::lit(
                "A circular stone room. The door north is heavy oak bound in iron. A narrow spiral staircase hugs the wall to your right.",
            ),
        )
        .with_exit(Exit::new(
            "south (back to the cottage)",
            ids::room_cottage(),
        ))
        .with_exit(
            Exit::new("up (the staircase)", ids::room_tower_stairs()).gated(
                Condition::FlagSet(ids::flag_tower_unlocked()),
                Text::lit(
                    "The stairwell door is bolted from the inside. You need the tower key, or another way.",
                ),
            ),
        )
        .with_exit(
            Exit::new(
                "through the narrow crack in the wall",
                ids::room_tower_stairs(),
            )
            .hidden_until(Condition::FlagSet(ids::flag_hidden_passage())),
        )
        .with_examine(
            "stairs",
            Text::lit("The iron treads are cold. They climb into darkness."),
        )
        .with_examine(
            "wall",
            Text::lit(
                "Rough granite, damp. There is a darker seam near the floor that looks almost like a crack.",
            ),
        ),
    );

    area.add_room(
        ids::room_tower_stairs(),
        Room::new(
            "The Stairwell",
            Text::lit(
                "Iron stairs climb a stone throat. The wind moans through gaps in the masonry.",
            ),
        )
        .with_exit(Exit::new(
            "down (to the tower base)",
            ids::room_tower_base(),
        ))
        .with_exit(Exit::new(
            "up (to the lantern room)",
            ids::room_lantern_room(),
        )),
    );

    area.add_room(
        ids::room_lantern_room(),
        Room::new(
            "The Lantern Room",
            Text::lit(
                "Glass on all sides, the horizon wheeling around you. At the centre stands the great Fresnel lens, all prisms and brass, cold and dark. A bronze mechanism lies frozen at its base, stiff with salt.",
            ),
        )
        .with_exit(Exit::new(
            "down (back to the stairwell)",
            ids::room_tower_stairs(),
        ))
        .with_examine(
            "lens",
            Text::lit(
                "Each prism is a palm-sized block of glass. Dust and brine have dimmed them, but the mechanism is intact.",
            ),
        )
        .with_examine(
            "mechanism",
            Text::lit(
                "The clockwork that rotates the lens. It is seized tight; you can see where salt has welded the joints.",
            ),
        ),
    );

    area.add_item(
        ids::item_tower_key(),
        Item::new(
            "tower key",
            Text::lit("a long brass key on a leather thong"),
            Text::lit("The keeper's tower key. The leather is dark with handling."),
        )
        .with_synonyms(["brass key"])
        .takeable()
        .with_tag("opens_tower")
        .initially_in(ids::room_tower_base()),
    );

    // Using the tower key at the tower base unlocks the stairwell.
    area.add_rule(
        ids::rule_tower_unlocked(),
        Rule::on(
            Trigger::OnUse {
                item: Some(ids::item_tower_key()),
                in_room: Some(ids::room_tower_base()),
            },
            vec![
                Effect::Say(Text::lit(
                    "You draw the bolt and the stairwell door scrapes open. Warm dust falls from the lintel.",
                )),
                Effect::SetFlag(ids::flag_tower_unlocked(), Value::TRUE),
            ],
        )
        .once(),
    );

    // Using the oil can in the lantern room frees the lens mechanism.
    area.add_rule(
        ids::rule_oil_applied(),
        Rule::on(
            Trigger::OnUse {
                item: Some(ids::item_oil_can()),
                in_room: Some(ids::room_lantern_room()),
            },
            vec![
                Effect::Say(Text::lit(
                    "You work oil into the joints. The clockwork frees with a small, willing click.",
                )),
                Effect::SetFlag(ids::flag_lens_oiled(), Value::TRUE),
            ],
        )
        .once(),
    );

    // Using the tinderbox in the lantern room relights the great lantern —
    // but only if the lens is oiled and the player carries the storm lantern.
    area.add_rule(
        ids::rule_relight_lantern(),
        Rule::on(
            Trigger::OnUse {
                item: Some(ids::item_tinderbox()),
                in_room: Some(ids::room_lantern_room()),
            },
            vec![Effect::If {
                when: Condition::All(vec![
                    Condition::HasItem(ids::item_lantern()),
                    Condition::FlagSet(ids::flag_lens_oiled()),
                ]),
                then: vec![
                    Effect::Say(Text::lit(
                        "Flame catches in the reservoir. Through the prisms the light shatters and throws itself out across the dark sea.",
                    )),
                    Effect::SetFlag(ids::flag_lantern_restored(), Value::TRUE),
                    Effect::TriggerEvent(ids::event_lantern_restored()),
                ],
                otherwise: vec![Effect::Say(Text::Conditional {
                    when: Condition::HasItem(ids::item_lantern()),
                    then: Box::new(Text::lit(
                        "You strike sparks. Nothing catches. The lens is still frozen; you need to free the mechanism first.",
                    )),
                    otherwise: Box::new(Text::lit(
                        "You strike sparks with nothing to light. You need the storm lantern.",
                    )),
                })],
            }],
        ),
    );

    // Using the driftwood on the lens mechanism sabotages it.
    area.add_rule(
        ids::rule_sabotage_lantern(),
        Rule::on(
            Trigger::OnUse {
                item: Some(ids::item_driftwood()),
                in_room: Some(ids::room_lantern_room()),
            },
            vec![
                Effect::Say(Text::lit(
                    "You bring the driftwood down on the prisms. Glass sprays. The mechanism seizes, this time for good.",
                )),
                Effect::SetFlag(ids::flag_lantern_sabotaged(), Value::TRUE),
            ],
        )
        .once(),
    );

    // Hidden crack in the tower base: revealed on dwelling there after the
    // cellar has been visited (backup path if the player never finds the key).
    area.add_rule(
        ids::rule_hidden_passage(),
        Rule::on(
            Trigger::TurnEnd,
            vec![
                Effect::Say(Text::lit(
                    "You notice the crack in the wall widens in the damp: you could slip through.",
                )),
                Effect::SetFlag(ids::flag_hidden_passage(), Value::TRUE),
            ],
        )
        .with_condition(Condition::All(vec![
            Condition::PlayerIn(ids::room_tower_base()),
            Condition::FlagUnset(ids::flag_tower_unlocked()),
            Condition::Visited(ids::room_cellar()),
            Condition::TurnAtLeast(4),
        ]))
        .once(),
    );

    // First step onto the stairs gets a flavour line.
    area.add_rule(
        ids::rule_first_stairs(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_tower_stairs())),
            vec![Effect::Say(Text::lit(
                "The stairwell answers your footsteps with a long iron note.",
            ))],
        )
        .once(),
    );

    area
}
