use crate::game::areas::{AreaContents, by_cycle};
use crate::game::ids;
use crate::game::plan::{CycleTransition, baseline, transitions};
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueNode, DialogueOption, Effect, Entity, Exit, Item, ItemLocation,
    ItemProperties, Room, Rule, Text, Trigger, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    let bedroom_wake = per_cycle_text(baseline().bedroom_wake, |step| step.bedroom_wake);
    let bedroom_state = per_cycle_text(baseline().bedroom_state, |step| step.bedroom_state);
    let bedroom_normal = Text::Sequence(vec![
        Text::Conditional {
            when: Condition::FlagSet(ids::flag_woke_up_this_cycle()),
            then: Box::new(Text::Sequence(vec![bedroom_wake, Text::lit("\n\n")])),
            otherwise: Box::new(Text::empty()),
        },
        bedroom_state,
    ]);
    let bedroom_redux = Text::lit(
        "The bed is made the way you make it. There is a sticky note on the nightstand, in your handwriting. You have the sense that something is missing from this room, though you cannot say what.",
    );
    area.add_room(
        ids::room_bedroom(),
        Room::new(
            "Bedroom",
            Text::Conditional {
                when: Condition::FlagSet(ids::flag_is_redux()),
                then: Box::new(bedroom_redux),
                otherwise: Box::new(bedroom_normal),
            },
        )
        .with_unseen_alias("a small room you slept in")
        .with_alias_when(Condition::StatAtLeast(ids::stat_env(), 6))
        .with_exit(Exit::new("south (to the hallway)", ids::room_hallway()))
        .with_examine("calendar", calendar_text())
        .with_examine(
            "closet",
            by_cycle(
                Text::lit("One set of clothes hangs in the closet. They are the clothes you wear."),
                vec![(7, Text::lit("The closet is empty. The hangers are bare."))],
            ),
        )
        .with_examine(
            "window",
            Text::lit(
                "The window looks onto a city block. Morning light. The buildings are plausible and the light is plausible.",
            ),
        )
        .with_examine(
            "nightstand",
            Text::lit(
                "A small wooden nightstand beside the bed. One drawer. A lamp. A few rings in the varnish where a mug used to sit.",
            ),
        )
        .with_examine(
            "dresser",
            Text::lit(
                "A low wooden dresser across from the bed, three drawers, slightly crooked. The mirror above it is the one you use every morning.",
            ),
        )
        .with_examine(
            "drawer",
            Text::lit(
                "The nightstand drawer slides open. Inside: a pen, a pair of earplugs, a clipped photograph you do not recognise, and a receipt for something you don't remember buying.",
            ),
        )
        .with_examine(
            "lamp",
            Text::lit(
                "A brass bedside lamp. The bulb is warm. You do not remember turning it on.",
            ),
        )
        .with_examine(
            "alarm",
            by_cycle(
                Text::lit(
                    "The alarm clock on the nightstand. A small black block. The display reads 6:47.",
                ),
                vec![(4, Text::lit(
                    "The alarm clock on the nightstand. The display reads 6:47. It always reads 6:47 when you wake.",
                ))],
            ),
        )
        .with_examine(
            "pillow",
            Text::lit(
                "The pillow on your side of the bed. Still dented where your head was.",
            ),
        )
        .with_examine(
            "blanket",
            Text::lit(
                "A folded blanket at the foot of the bed. You don't remember it being folded.",
            ),
        )
        .with_examine(
            "sheets",
            Text::lit(
                "White sheets, tangled where you slept. They smell faintly of yourself and nothing else.",
            ),
        )
        .with_examine(
            "curtains",
            Text::lit(
                "Thin curtains, pulled halfway. The light past them is even and unshifting.",
            ),
        )
        .with_examine(
            "carpet",
            Text::lit(
                "Cream carpet, worn slightly along the path between the bed and the door.",
            ),
        )
        .with_examine(
            "ceiling",
            Text::lit(
                "A plain ceiling. A small stain near the corner. You have never been able to tell what made it.",
            ),
        )
        .with_examine(
            "floor",
            Text::lit(
                "Hardwood under the carpet. The boards do not creak when you step. They never have.",
            ),
        )
        .with_examine(
            "walls",
            Text::lit(
                "Painted a sandy off-white. No art. No photographs. You have always meant to hang something.",
            ),
        )
        .with_examine(
            "door",
            Text::lit(
                "The bedroom door opens south, into the apartment hallway.",
            ),
        )
        .with_examine(
            "air",
            Text::lit(
                "The air is still. It smells of a room that has been closed overnight. Nothing more.",
            ),
        )
        .with_examine(
            "light",
            Text::lit(
                "Morning light through the curtains. Even. Unshifting. You have watched it for an hour and not seen it move.",
            ),
        ),
    );

    area.add_room(
        ids::room_hallway(),
        Room::new(
            "Apartment Hallway",
            per_cycle_text(baseline().hallway_description, |step| {
                step.hallway_description
            }),
        )
        .with_unseen_alias("a short hall")
        .with_alias_when(Condition::StatAtLeast(ids::stat_env(), 6))
        .with_exit(Exit::new("north (to the bedroom)", ids::room_bedroom()))
        .with_exit(Exit::new("east (to the kitchen)", ids::room_kitchen()))
        .with_exit(Exit::new(
            "west (out to the building corridor)",
            ids::room_building_corridor(),
        ))
        .with_examine(
            "coat",
            Text::lit(
                "A wool coat on the hook. Heavy. You've worn it every day you remember wearing a coat.",
            ),
        )
        .with_examine(
            "coat hook",
            Text::lit(
                "A brass hook mounted on the wall. One coat hangs from it. There's space for another.",
            ),
        )
        .with_examine(
            "side table",
            Text::lit(
                "A narrow side table by the front door. A small bowl. Your keys.",
            ),
        )
        .with_examine(
            "keys",
            Text::lit(
                "The keys to the apartment and the office. You do not remember which is which. They have never not worked.",
            ),
        )
        .with_examine(
            "bowl",
            Text::lit(
                "A shallow ceramic bowl on the side table. Just for the keys. A thin film of dust inside.",
            ),
        )
        .with_examine(
            "door",
            Text::lit(
                "The front door. A deadbolt, a peephole. The peephole always shows the same empty corridor.",
            ),
        )
        .with_examine(
            "peephole",
            Text::lit(
                "You look through the peephole. The corridor outside. No one in either direction.",
            ),
        )
        .with_examine(
            "walls",
            Text::lit(
                "Plain painted walls. There has never been a picture hung on them.",
            ),
        )
        .with_examine(
            "floor",
            Text::lit(
                "Hardwood running the length of the hallway. Scuffed where you walk.",
            ),
        )
        .with_examine(
            "ceiling",
            Text::lit(
                "A plain ceiling. A single light fixture, glowing evenly.",
            ),
        )
        .with_examine(
            "light",
            Text::lit(
                "A recessed light overhead. Warm white. It never flickers.",
            ),
        )
        .with_examine(
            "air",
            Text::lit(
                "The air is still. It smells of the apartment and nothing else.",
            ),
        ),
    );

    area.add_room(
        ids::room_kitchen(),
        Room::new(
            "Kitchen",
            per_cycle_text(baseline().kitchen_description, |step| step.kitchen_description),
        )
        .with_unseen_alias("a small kitchen")
        .with_alias_when(Condition::StatAtLeast(ids::stat_env(), 6))
        .with_exit(Exit::new("west (back to the hallway)", ids::room_hallway()))
        .with_examine(
            "refrigerator",
            by_cycle(
                Text::lit("Some groceries. Nothing remarkable."),
                vec![
                    (6, Text::lit(
                        "You open the refrigerator. A gallon of milk. The same gallon as last week. The seal is unbroken. You close the refrigerator.",
                    )),
                    (7, Text::lit(
                        "You open the refrigerator. It is empty. You do not open it again.",
                    )),
                ],
            ),
        )
        .with_examine(
            "coffee maker",
            Text::lit(
                "The coffee maker hums. The coffee is already brewed. You must have started it earlier. You don't remember starting it.",
            ),
        )
        .with_examine(
            "counter",
            Text::lit(
                "A small kitchen counter. The coffee maker. A single mug. A small stack of mail you haven't opened. The mail is never for you.",
            ),
        )
        .with_examine(
            "sink",
            Text::lit(
                "A small stainless sink. A faucet. A sponge. The drain has never clogged.",
            ),
        )
        .with_examine(
            "faucet",
            Text::lit(
                "A simple chrome faucet. Cold works. Hot works. The water tastes of water.",
            ),
        )
        .with_examine(
            "table",
            Text::lit(
                "A small round table with one chair pushed in. One placemat. You eat here when you eat, which is rarely.",
            ),
        )
        .with_examine(
            "chair",
            Text::lit(
                "A single wooden chair. The cushion is slightly compressed where you sit.",
            ),
        )
        .with_examine(
            "window",
            Text::lit(
                "The kitchen window is above the sink. It looks onto an air shaft. There is nothing in the shaft.",
            ),
        )
        .with_examine(
            "cabinets",
            Text::lit(
                "Cabinets above and below the counter. Inside: plates, glasses, a few pans. Nothing unexpected. Nothing surprising.",
            ),
        )
        .with_examine(
            "dishes",
            Text::lit(
                "Two plates. Two bowls. Two glasses. Always two.",
            ),
        )
        .with_examine(
            "floor",
            Text::lit(
                "Tile. Cool under foot. Spotless.",
            ),
        )
        .with_examine(
            "walls",
            Text::lit(
                "White kitchen walls. A small calendar on one. No photographs.",
            ),
        )
        .with_examine(
            "ceiling",
            Text::lit(
                "The kitchen ceiling. A small exhaust vent that never runs.",
            ),
        )
        .with_examine(
            "mail",
            Text::lit(
                "A few envelopes. None of them are addressed to you. They are addressed to a name you don't recognise, at this address.",
            ),
        )
        .with_examine(
            "air",
            Text::lit(
                "The air smells faintly of brewed coffee, even when the maker is off.",
            ),
        )
        .with_examine(
            "light",
            Text::lit(
                "Daylight from the window, evenly. A ceiling fixture you rarely turn on.",
            ),
        ),
    );

    area.add_item(
        ids::item_coffee_mug(),
        Item::new(
            "coffee mug",
            Text::lit("your coffee mug, warm"),
            Text::lit(
                "A ceramic mug. The coffee in it is the same temperature as yesterday morning. And the morning before.",
            ),
        )
        .with_synonyms(["mug", "coffee", "cup"])
        .with_properties(ItemProperties {
            takeable: true,
            consume_response: Some(Text::lit(
                "You sip the coffee. It is the same temperature it was yesterday. You set the mug down.",
            )),
            ..Default::default()
        })
        .initially_in(ids::room_kitchen()),
    );

    area.add_entity(
        ids::fixture_mirror(),
        Entity::object("the mirror", Text::lit("The mirror on the dresser."))
            .with_synonyms(["mirror"])
            .with_dialogue(ids::dialogue_mirror())
            .starting_in(ids::room_bedroom()),
    );

    area.add_dialogue(
        ids::dialogue_mirror(),
        Dialogue::new(ids::node_root()).with_node(
            ids::node_root(),
            DialogueNode::new(Text::Conditional {
                when: Condition::FlagSet(ids::flag_mirror_looked_closer()),
                then: Box::new(Text::Conditional {
                    when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                    then: Box::new(Text::lit(
                        "The face in the mirror is doing what your face should be doing. A little late. You do not meet its eyes.",
                    )),
                    otherwise: Box::new(Text::lit(
                        "The face in the mirror is doing what your face should be doing. You do not meet its eyes.",
                    )),
                }),
                otherwise: Box::new(per_cycle_text(baseline().mirror_text, |step| {
                    step.mirror_text
                })),
            })
            .with_option(
                DialogueOption::new(Text::lit("Look closer."))
                    .with_condition(Condition::All(vec![
                        Condition::StatAtLeast(ids::stat_cycle(), 5),
                        Condition::FlagUnset(ids::flag_mirror_looked_closer()),
                    ]))
                    .with_effects(vec![
                        Effect::Say(Text::lit(
                            "You lean in. The face leans in back. It is not your face. It has never been your face.",
                        )),
                        Effect::AddStat(ids::stat_awa(), 2),
                        Effect::SetFlag(ids::flag_mirror_looked_closer(), Value::TRUE),
                    ]),
            )
            .with_option(DialogueOption::new(Text::lit("Step away from the mirror."))),
        ),
    );

    area.add_entity(
        ids::fixture_bed(),
        Entity::object(
            "the bed",
            Text::lit(
                "The bed. You sleep here. You know this in the way you know the mug is yours.",
            ),
        )
        .with_synonyms(["bed", "sleep"])
        .with_dialogue(ids::dialogue_sleep())
        .starting_in(ids::room_bedroom()),
    );

    area.add_dialogue(
        ids::dialogue_sleep(),
        Dialogue::new(ids::node_sleep_prompt()).with_node(
            ids::node_sleep_prompt(),
            DialogueNode::new(Text::Conditional {
                when: Condition::FlagSet(ids::flag_is_redux()),
                then: Box::new(Text::lit(
                    "The bed is made. The room is the room. You could get in and sleep, and the alarm will buzz again.",
                )),
                otherwise: Box::new(Text::Conditional {
                    when: Condition::FlagSet(ids::flag_at_desk_arrived_this_cycle()),
                    then: Box::new(Text::lit(
                        "The bed. The day has drained out of you. You could lie down now.",
                    )),
                    otherwise: Box::new(Text::lit(
                        "The bed. It is morning. You are not ready to sleep yet.",
                    )),
                }),
            })
            .with_option(
                DialogueOption::new(Text::lit("Get into bed. Sleep."))
                    .with_condition(Condition::All(vec![
                        Condition::FlagSet(ids::flag_at_desk_arrived_this_cycle()),
                        Condition::FlagUnset(ids::flag_is_redux()),
                    ]))
                    .with_effects(vec![
                        Effect::SetFlag(ids::flag_at_desk_arrived_this_cycle(), Value::FALSE),
                        Effect::SetFlag(ids::flag_frame_looked_today(), Value::FALSE),
                        Effect::TriggerEvent(ids::event_sleep()),
                    ]),
            )
            .with_option(
                DialogueOption::new(Text::lit(
                    "(You should go to the office before you sleep.)",
                ))
                .with_condition(Condition::FlagUnset(ids::flag_at_desk_arrived_this_cycle()))
                .visible_when_locked(Text::lit("You haven't done the day's work yet.")),
            )
            .with_option(
                DialogueOption::new(Text::lit(
                    "Conclude the redux. Let the next instance begin.",
                ))
                .with_condition(Condition::FlagSet(ids::flag_is_redux()))
                .with_effects(vec![
                    Effect::Say(Text::lit(
                        "You lie down. The sheets are cool. You think of the next one of you — whoever that turns out to be. You hope he opens his mail. You hope he pulls the third book off the shelf. You close your eyes.",
                    )),
                    Effect::MovePlayer(ids::room_endgame()),
                ]),
            )
            .with_option(DialogueOption::new(Text::lit("Stay up a little longer."))),
        ),
    );

    area.add_entity(
        ids::fixture_commute(),
        Entity::object(
            "the commute to the office",
            Text::lit(
                "The walk to your office is short and habitual. You could leave for work now.",
            ),
        )
        .with_synonyms(["commute", "office", "work", "leave"])
        .with_dialogue(ids::dialogue_commute())
        .starting_in(ids::room_bedroom()),
    );

    area.add_dialogue(
        ids::dialogue_commute(),
        Dialogue::new(ids::node_root()).with_node(
            ids::node_root(),
            DialogueNode::new(Text::lit("Head to the office?"))
                .with_option(
                    DialogueOption::new(Text::lit("Yes. Walk to the office."))
                        .with_condition(Condition::All(vec![
                            Condition::FlagUnset(ids::flag_at_desk_arrived_this_cycle()),
                            Condition::FlagUnset(ids::flag_is_redux()),
                        ]))
                        .with_effects(vec![
                            Effect::Say(Text::lit(crate::game::prose::COMMUTE_TO_OFFICE)),
                            Effect::MovePlayer(ids::room_desk()),
                        ]),
                )
                .with_option(
                    DialogueOption::new(Text::lit("(You're already at the office today.)"))
                        .with_condition(Condition::All(vec![
                            Condition::FlagSet(ids::flag_at_desk_arrived_this_cycle()),
                            Condition::FlagUnset(ids::flag_is_redux()),
                        ]))
                        .visible_when_locked(Text::lit("You've already been to the office today.")),
                )
                .with_option(DialogueOption::new(Text::lit("Not yet."))),
        ),
    );

    area.add_rule(
        ids::rule_mark_desk_arrival(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_desk())),
            vec![Effect::SetFlag(
                ids::flag_at_desk_arrived_this_cycle(),
                Value::TRUE,
            )],
        ),
    );

    area.add_item(
        ids::item_sticky_note_hallway(),
        Item::new(
            "sticky note",
            Text::lit("a sticky note on the side table"),
            Text::lit(
                "Three letters, in your handwriting: C-H-I-M. You do not remember writing it.",
            ),
        )
        .with_synonyms(["note", "sticky"]),
    );

    area.add_item(
        ids::item_sticky_note_redux(),
        Item::new(
            "sticky note",
            Text::lit("a sticky note on the nightstand"),
            Text::lit(
                "A sticky note on the nightstand. The handwriting is yours. It says one word:\n\n    Don't.",
            ),
        )
        .with_synonyms(["note", "sticky", "yellow note"]),
    );

    area.add_rule(
        ids::rule_place_hallway_sticky(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_hallway())),
            vec![Effect::MoveItem(
                ids::item_sticky_note_hallway(),
                ItemLocation::Room(ids::room_hallway()),
            )],
        )
        .with_condition(Condition::StatAtLeast(ids::stat_cycle(), 5))
        .once(),
    );

    area.add_rule(
        ids::rule_place_redux_sticky_note(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_bedroom())),
            vec![Effect::MoveItem(
                ids::item_sticky_note_redux(),
                ItemLocation::Room(ids::room_bedroom()),
            )],
        )
        .with_condition(Condition::FlagSet(ids::flag_is_redux()))
        .once(),
    );

    area
}

fn calendar_text() -> Text {
    with_redux_override(
        "The calendar reads April 3. You do not remember the calendar being anything else.",
        per_cycle_text(baseline().calendar_narration, |step| {
            Some(step.calendar_narration)
        }),
    )
}

fn with_redux_override(redux_text: &'static str, base: Text) -> Text {
    Text::Conditional {
        when: Condition::FlagSet(ids::flag_is_redux()),
        then: Box::new(Text::lit(redux_text)),
        otherwise: Box::new(base),
    }
}

fn per_cycle_text<F>(initial: &'static str, extract: F) -> Text
where
    F: Fn(&CycleTransition) -> Option<&'static str>,
{
    let mut current = Text::lit(initial);
    for step in transitions() {
        if let Some(text) = extract(step) {
            current = Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), step.to),
                then: Box::new(Text::lit(text)),
                otherwise: Box::new(current),
            };
        }
    }
    current
}
