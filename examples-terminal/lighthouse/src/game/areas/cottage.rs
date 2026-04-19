//! The keeper's cottage — the hub room plus every item, rule, and flavour
//! line anchored to it.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Effect, Exit, Item, ItemLocation, ItemProperties, Room, Rule, Text, Trigger, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_room(
        ids::room_cottage(),
        Room::new(
            "The Keeper's Cottage",
            Text::lit(
                "A single room, meticulously kept: a narrow bed, a table laid for a meal that was never eaten, a kettle cold on the stove. A door in the north wall opens into the tower. A trapdoor set into the floor leads down; it is unbolted.",
            ),
        )
        .with_exit(Exit::new("south (back to the shore)", ids::room_shore()))
        .with_exit(Exit::new("north (into the tower)", ids::room_tower_base()))
        .with_exit(Exit::new("down (into the cellar)", ids::room_cellar()))
        .with_examine(
            "table",
            Text::lit(
                "A single plate, knife and fork set out. The bread on the plate has gone hard.",
            ),
        )
        .with_examine(
            "stove",
            Text::lit(
                "The iron is cold. The coals in the grate haven't been stirred in a day at least.",
            ),
        ),
    );

    area.add_item(
        ids::item_lantern(),
        Item::new(
            "storm lantern",
            Text::lit("a brass storm lantern, unlit"),
            Text::lit(
                "A heavy brass lantern with a clean wick and a reservoir half-full of oil. Unlit.",
            ),
        )
        .with_synonyms(["lantern", "light"])
        .with_properties(ItemProperties {
            takeable: true,
            light_source: true,
            lit_flag: Some(ids::flag_lantern_is_lit()),
            ..Default::default()
        })
        .initially_in(ids::room_cottage()),
    );

    area.add_item(
        ids::item_tinderbox(),
        Item::new(
            "tinderbox",
            Text::lit("a small brass tinderbox"),
            Text::lit("A tinderbox. Dry, well-kept. It will strike a flame."),
        )
        .with_synonyms(["tinder", "flint"])
        .takeable()
        .initially_in(ids::room_cottage()),
    );

    area.add_item(
        ids::item_keeper_log(),
        Item::new(
            "keeper's log",
            Text::lit("the keeper's log, open on the table"),
            Text::lit(
                "A leather-bound ledger of daily entries, weather and ship sightings. The last entry is dated yesterday.",
            ),
        )
        .with_synonyms(["log", "logbook", "journal"])
        .with_read(Text::lit(concat!(
            "The last entry reads: 'Third visitor this week. Asking after the lens. Told them I don't drink with strangers. ",
            "T--- won't stop smiling. I don't like it. If something happens to me, look to the cellar. The shelves are his doing, not mine.'",
        )))
        .with_tag("clue")
        .initially_in(ids::room_cottage()),
    );

    area.add_item(
        ids::item_oil_can(),
        Item::new(
            "oil can",
            Text::lit("a tin oil can"),
            Text::lit(
                "A narrow-necked can, three-quarters full of clean machine oil. Heavier than it looks.",
            ),
        )
        .with_synonyms(["oil", "can"])
        .takeable()
        .initially_in(ids::room_cottage()),
    );

    area.add_item(
        ids::item_rope(),
        Item::new(
            "coiled rope",
            Text::lit("a coil of tarred rope"),
            Text::lit(
                "A long coil of heavy rope, tarred against salt. The strands are in good condition.",
            ),
        )
        .with_synonyms(["rope", "cord", "line"])
        .takeable()
        .initially_in(ids::room_cottage()),
    );

    // Using the cottage key at the shore unlocks the cottage door.
    area.add_rule(
        ids::rule_cottage_unlocked(),
        Rule::on(
            Trigger::OnUse {
                item: Some(ids::item_cottage_key()),
                in_room: Some(ids::room_shore()),
            },
            vec![
                Effect::Say(Text::lit(
                    "The key turns stiffly. The cottage door swings inward.",
                )),
                Effect::SetFlag(ids::flag_cottage_unlocked(), Value::TRUE),
                Effect::MoveItem(ids::item_cottage_key(), ItemLocation::Nowhere),
            ],
        )
        .once(),
    );

    // Using the tinderbox while carrying the (unlit) storm lantern lights it.
    area.add_rule(
        ids::rule_light_lantern(),
        Rule::on(
            Trigger::OnUse {
                item: Some(ids::item_tinderbox()),
                in_room: None,
            },
            vec![Effect::If {
                when: Condition::All(vec![
                    Condition::HasItem(ids::item_lantern()),
                    Condition::FlagUnset(ids::flag_lantern_is_lit()),
                ]),
                then: vec![
                    Effect::Say(Text::lit(
                        "You light the wick of the storm lantern. It catches cleanly; the brass warms in your hand.",
                    )),
                    Effect::SetFlag(ids::flag_lantern_is_lit(), Value::TRUE),
                ],
                otherwise: vec![],
            }],
        ),
    );

    // Dropping the cottage key anywhere gets a flavour line.
    area.add_rule(
        ids::rule_drop_cottage_key(),
        Rule::on(
            Trigger::OnDrop(Some(ids::item_cottage_key())),
            vec![Effect::Say(Text::lit(
                "The key clinks against the shingle and settles.",
            ))],
        ),
    );

    // First read of the keeper's log nudges the player toward the cellar.
    area.add_rule(
        ids::rule_read_log_hint(),
        Rule::on(
            Trigger::OnExamine(Some(ids::item_keeper_log())),
            vec![Effect::Say(Text::lit(
                "You re-read the last line. 'Look to the cellar.' That settles it.",
            ))],
        )
        .once(),
    );

    area
}
