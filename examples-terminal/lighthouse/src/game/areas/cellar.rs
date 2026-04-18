//! The cellar: the dark room itself, the items it conceals (the second
//! ledger, the wreckers' note, and — once found — the keeper's remains),
//! and the rules that fire when the player arrives with a light.

use crate::data::{
    Condition, Effect, Exit, Item, ItemLocation, ItemProperties, Room, Rule, Text, Trigger, Value,
};
use crate::game::areas::AreaContents;
use crate::game::ids;

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_room(
        ids::room_cellar(),
        Room::new(
            "The Cellar",
            Text::lit(
                "A low-ceilinged stone room, smelling of damp and kelp. Shelves along one wall hold preserves and a careful log of deliveries.",
            ),
        )
        .dark(Text::lit(
            "It is pitch black. You can hear something dripping, but you can see nothing. You need a light.",
        ))
        .with_exit(Exit::new("up (back to the cottage)", ids::room_cottage()))
        .with_examine(
            "shelves",
            Text::lit(
                "Jars of pickled things, labelled in a careful hand. Everything is tidy until the third shelf, where a row of jars has been swept onto the floor.",
            ),
        )
        .with_examine(
            "drip",
            Text::lit(
                "A slow trickle from the ceiling. You follow it with your eyes and see it pooling against the far wall, under something.",
            ),
        ),
    );

    area.add_item(
        ids::item_ledger(),
        Item::new(
            "second ledger",
            Text::lit("a second, thinner ledger, tucked behind a crate"),
            Text::lit(
                "A thin ledger in another hand. Columns of dates and shares, with the word 'salvage' written at the top of every page.",
            ),
        )
        .with_synonyms(["ledger", "accounts", "book"])
        .with_read(Text::lit(concat!(
            "Thirty-eight ships over eleven years. Each entry lists a date, weather, the name of a vessel, and a column of initials with a share ",
            "marked next to each. Your eye keeps returning to one set of initials, over and over: T.V. Below the last entry, in a different pen: 'Keeper asks too many questions. Attend to it.'",
        )))
        .with_tag("clue")
        .initially_in(ids::room_cellar()),
    );

    area.add_item(
        ids::item_wreckers_note(),
        Item::new(
            "folded note",
            Text::lit("a folded paper, slipped into the ledger"),
            Text::lit("A folded paper. The crease is still sharp; it was hidden recently."),
        )
        .with_synonyms(["note", "paper", "letter"])
        .takeable()
        .with_read(Text::lit(concat!(
            "A short note: 'Dunmere to be dark on the fifteenth. Boat due from Padstow. Share is forty pounds if the headland takes her clean. ",
            "Thirty pounds if only half. Burn this.'",
        )))
        .with_tag("evidence")
        .initially_in(ids::room_cellar()),
    );

    // The keeper's remains start offstage; `rule_found_keeper` reveals them
    // when the player first enters the cellar with a lit lantern.
    area.add_item(
        ids::item_keeper_remains(),
        Item::new(
            "keeper",
            Text::lit("the keeper, lying beneath the shelves"),
            Text::lit(
                "The old keeper lies half-covered in fallen stone. A blow to the head; done quickly. His hand is still curled around the thong of his tower key.",
            ),
        )
        .with_synonyms(["body", "corpse"])
        .with_properties(ItemProperties {
            takeable: false,
            ..Default::default()
        })
        .with_tag("evidence"),
    );

    // Entering the cellar with a lit lantern finds the keeper's body.
    area.add_rule(
        ids::rule_found_keeper(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_cellar())),
            vec![Effect::If {
                when: Condition::Ref(ids::cond_has_lit_lantern()),
                then: vec![
                    Effect::Say(Text::lit(
                        "Your light falls on the far wall. The keeper is there, under the fallen shelves. He has been dead most of a day.",
                    )),
                    Effect::MoveItem(
                        ids::item_keeper_remains(),
                        ItemLocation::Room(ids::room_cellar()),
                    ),
                    Effect::SetFlag(ids::flag_found_keeper(), Value::TRUE),
                ],
                otherwise: vec![],
            }],
        )
        .once(),
    );

    // Examining the second ledger records that the player has seen it.
    area.add_rule(
        ids::rule_log_examined(),
        Rule::on(
            Trigger::OnExamine(Some(ids::item_ledger())),
            vec![Effect::SetFlag(ids::flag_read_second_ledger(), Value::TRUE)],
        )
        .once(),
    );

    // Picking up the wreckers' note.
    area.add_rule(
        ids::rule_take_note_flavor(),
        Rule::on(
            Trigger::OnTake(Some(ids::item_wreckers_note())),
            vec![Effect::Say(Text::lit(
                "You pocket the note. Your hand is not quite steady.",
            ))],
        )
        .once(),
    );

    area
}
