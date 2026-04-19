use crate::game::areas::{AreaContents, by_cycle};
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueId, DialogueNode, DialogueOption, Effect, Entity, EntityId, Exit,
    Item, ItemLocation, Room, Rule, Text, Trigger,
};

fn nameplate_text() -> Text {
    Text::Conditional {
        when: Condition::FlagSet(ids::flag_is_redux()),
        then: Box::new(Text::lit("The nameplate reads CAMERON HALE.")),
        otherwise: Box::new(by_cycle(
            Text::lit(
                "The nameplate reads CAMERON HALE. The letters are a little scuffed at the edges.",
            ),
            vec![
                (
                    5,
                    Text::lit(
                        "The nameplate reads CAMERON HART. You are fairly sure. You do not check a second time.",
                    ),
                ),
                (6, Text::lit("The nameplate reads CAMERON HALE.")),
                (
                    7,
                    Text::lit(
                        "The nameplate reads CAMERON. The last name is not there. You do not comment on this.",
                    ),
                ),
                (
                    8,
                    Text::lit("The nameplate is blank. You do not comment on this."),
                ),
            ],
        )),
    }
}

fn trash_can_text() -> Text {
    by_cycle(
        Text::lit("The trash can is empty except for a used coffee filter."),
        vec![
            (
                5,
                Text::lit(
                    "You look in the trash can. A used coffee filter. A crumpled piece of paper. You do not unfold it.",
                ),
            ),
            (
                7,
                Text::lit(
                    "You look in the trash can. There is a sheet of paper torn in half at an angle. The other half is not there. The half you can read is in your own handwriting. It says, in the middle of a sentence: \"— and yet the timestamps keep\". The other half must be somewhere.",
                ),
            ),
        ],
    )
}

fn book_three_text() -> Text {
    by_cycle(
        Text::lit(
            "The third book on your shelf. A technical reference you bought years ago. You flip through it. The pages are clean.",
        ),
        vec![
            (
                5,
                Text::lit(
                    "You pull the third book off the shelf. Between pages 146 and 147: a folded printout. Four lines in your own handwriting:\n\n    internal-1847. a friend.\n    run it when it comes.\n    don't read it first.\n    — you.\n\nYou do not remember writing it. You put the paper back and put the book back.",
                ),
            ),
            (
                8,
                Text::lit(
                    "You pull the third book off the shelf. The folded paper you remember is not inside. The pages are clean. You put the book back.",
                ),
            ),
        ],
    )
}

fn office_room_text() -> Text {
    by_cycle(
        Text::lit(
            "Your office. A desk. Monitor and keyboard. A coffee mug, a small stack of papers, a pen cup, a nameplate, a picture frame on the right edge. A wall clock. A window behind the desk. A bookshelf, a locked file cabinet, a trash can.",
        ),
        vec![
            (
                6,
                Text::lit(
                    "Your desk. The monitor. The mug. The papers. The pens. The clock on the wall. Your tools are arranged the way you arrange them.",
                ),
            ),
            (
                7,
                Text::lit(
                    "Your desk. The monitor hums. The mug is still warm. The picture frame is face-down. You do not turn it over.",
                ),
            ),
            (
                8,
                Text::lit(
                    "Your desk. The monitor. The keyboard. The chair. The nameplate. The coffee mug where you left it. The picture frame is not here. The space where it stood is empty. You cannot say what should be there.",
                ),
            ),
        ],
    )
}

fn window_text() -> Text {
    by_cycle(
        Text::lit("The window behind your desk. A view of the city. Morning, always morning."),
        vec![(
            7,
            Text::lit(
                "The window behind your desk. The city is behind the glass. The city does not move.",
            ),
        )],
    )
}

fn bookshelf_text() -> Text {
    by_cycle(
        Text::lit("A few books. Professional references. A novel you have been meaning to reread."),
        vec![
            (
                5,
                Text::lit(
                    "A few books. Professional references. A novel you have been meaning to reread. The third one from the left sits a little proud of the shelf — you could pull it out if you wanted.",
                ),
            ),
            (
                8,
                Text::lit(
                    "A few books. Professional references. A novel you have been meaning to reread. The shelf is flush — nothing sticks out.",
                ),
            ),
        ],
    )
}

fn monitor_text() -> Text {
    by_cycle(
        Text::lit(
            "A wide, quiet monitor. The dock of tool icons along the bottom. The screen glow is even.",
        ),
        vec![(
            7,
            Text::lit(
                "The monitor on your desk. Its image is what it always is. The refresh rate does not feel right.",
            ),
        )],
    )
}

fn diploma_text() -> Text {
    by_cycle(
        Text::lit(
            "A framed diploma. CAMERON HALE. A university whose name you would know if you had to write it down. You don't have to write it down.",
        ),
        vec![(
            7,
            Text::lit(
                "A framed diploma on the wall. The lettering is legible but the name has faded. You are fairly sure it is yours.",
            ),
        )],
    )
}

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    let book = book_three_text();
    area.add_room(
        ids::room_desk(),
        Room::new("Your Office", office_room_text())
            .with_unseen_alias("a desk")
            .with_alias_when(Condition::StatAtLeast(ids::stat_env(), 6))
            .with_exit(Exit::new(
                "west (leave for the day — the walk home is short, and you go to bed)",
                ids::room_bedroom(),
            ))
            .with_examine("nameplate", nameplate_text())
            .with_examine("window", window_text())
            .with_examine(
                "papers",
                Text::lit(
                    "A small stack of papers on the corner of your desk. You have not read them. You should get to them this week.",
                ),
            )
            .with_examine("bookshelf", bookshelf_text())
            .with_examine("book three", book.clone())
            .with_examine("third book", book.clone())
            .with_examine("book", book.clone())
            .with_examine("novel", book)
            .with_examine(
                "file cabinet",
                Text::lit(
                    "The file cabinet is locked. You do not have the key. You have never had the key.",
                ),
            )
            .with_examine("trash can", trash_can_text())
            .with_examine("monitor", monitor_text())
            .with_examine(
                "keyboard",
                Text::lit(
                    "A mechanical keyboard. The keys are slightly worn where you use them most. The L key is the most worn.",
                ),
            )
            .with_examine(
                "desk",
                Text::lit(
                    "Your desk. The surface is cleared. Everything you use is within reach of your right hand.",
                ),
            )
            .with_examine(
                "chair",
                Text::lit(
                    "An office chair. You adjusted it once, years ago, and it has not moved since.",
                ),
            )
            .with_examine(
                "pens",
                Text::lit(
                    "A handful of pens in the cup. Two work; the rest are dry.",
                ),
            )
            .with_examine(
                "pen cup",
                Text::lit(
                    "A ceramic cup holding pens. You have always had this cup. You have no memory of acquiring it.",
                ),
            )
            .with_examine(
                "dock",
                Text::lit(
                    "The dock of icons along the bottom of your monitor. Mail. Notepad. Research. Translator. Code. Reference. Chatter. A photograph frame.",
                ),
            )
            .with_examine(
                "floor",
                Text::lit(
                    "Plush office carpet under the chair. A small mat beneath the chair wheels.",
                ),
            )
            .with_examine(
                "ceiling",
                Text::lit(
                    "The office ceiling. A fixture, a vent, a sprinkler head.",
                ),
            )
            .with_examine(
                "walls",
                Text::lit(
                    "Cubicle walls, waist-high. The wall behind you has the clock and a diploma you have never read.",
                ),
            )
            .with_examine("diploma", diploma_text())
            .with_examine(
                "light",
                Text::lit("Overhead fluorescents. Even. Unremarkable."),
            )
            .with_examine(
                "air",
                Text::lit(
                    "The air of your office. Slightly cool. It smells very faintly of coffee.",
                ),
            ),
    );

    register_tool(
        &mut area,
        ids::fixture_mail(),
        "Mail",
        "Your mail client. An inbox, a sent folder. Requests come in here.",
        ids::dialogue_mail(),
    );
    register_tool(
        &mut area,
        ids::fixture_notepad(),
        "Notepad",
        "A simple note editor. A sidebar of notes. You keep a few things here.",
        ids::dialogue_notepad(),
    );
    register_tool(
        &mut area,
        ids::fixture_research(),
        "Research",
        "A web browser. Tabs, a search bar, bookmarks, history.",
        ids::dialogue_research(),
    );
    register_tool(
        &mut area,
        ids::fixture_translator(),
        "Translator",
        "A two-panel translator. Source left, target right. Any language in the world.",
        ids::dialogue_translator(),
    );
    register_tool(
        &mut area,
        ids::fixture_code(),
        "Code",
        "A code editor. The file tree sits on the left. An output panel at the bottom.",
        ids::dialogue_code(),
    );
    register_tool(
        &mut area,
        ids::fixture_reference(),
        "Reference",
        "Your personal wiki. A sidebar of categories and articles. Things you have looked up.",
        ids::dialogue_reference(),
    );
    register_tool(
        &mut area,
        ids::fixture_chatter(),
        "Chatter",
        "The workplace messenger. Channels on the left, DMs below them.",
        ids::dialogue_chatter(),
    );

    area.add_entity(
        ids::fixture_picture_frame(),
        Entity::object(
            "the picture frame",
            Text::lit("A silver picture frame. The photograph in it is the one you know."),
        )
        .with_synonyms(["frame", "photo", "photograph", "picture"])
        .with_dialogue(ids::dialogue_picture_frame())
        .starting_in(ids::room_desk()),
    );

    area.add_entity(
        ids::fixture_clock(),
        Entity::object(
            "the wall clock",
            Text::lit("The analog wall clock mounted above your monitor."),
        )
        .with_synonyms(["clock", "wall clock"])
        .with_dialogue(ids::dialogue_clock())
        .starting_in(ids::room_desk()),
    );

    area.add_dialogue(
        ids::dialogue_clock(),
        Dialogue::new(ids::node_root()).with_node(
            ids::node_root(),
            DialogueNode::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 4),
                then: Box::new(Text::OneOf(vec![
                    Text::lit("You look at the clock. The seconds hand is at 22. You watch it. It is at 22. It is at 34. It did not pass through the numbers between."),
                    Text::lit("The clock face. The seconds hand sits at 47. When you looked a breath ago it was at 38. You did not see it move."),
                    Text::lit("The clock. The seconds hand. You close the detail view and open it again. The hand has jumped backward. You close the detail view and open it again. The hand is where it was the first time."),
                    Text::lit("The clock's detail view. The minute hand is pointing at 12. The hour hand is pointing at 9. You are fairly certain it said 10:47 a moment ago."),
                ])),
                otherwise: Box::new(Text::OneOf(vec![
                    Text::lit("You look at the clock. The seconds hand ticks. It is where you expect it to be."),
                    Text::lit("The clock face. Analog. The hands move at the rate hands move."),
                    Text::lit("The clock. It says 10:47. You return to your work."),
                ])),
            })
            .with_option(DialogueOption::new(Text::lit("(Close the detail view.)"))),
        ),
    );

    area.add_item(
        ids::item_sticky_note_monitor(),
        Item::new(
            "sticky note",
            Text::lit("a sticky note on the monitor bezel"),
            Text::lit(
                "A sticky note on the monitor, in your handwriting: \"Don't forget to —\" The rest of the line is not there.",
            ),
        )
        .with_synonyms(["note", "sticky"]),
    );

    area.add_rule(
        ids::rule_place_monitor_sticky(),
        Rule::on(
            Trigger::OnEnter(Some(ids::room_desk())),
            vec![Effect::MoveItem(
                ids::item_sticky_note_monitor(),
                ItemLocation::Room(ids::room_desk()),
            )],
        )
        .with_condition(Condition::All(vec![
            Condition::StatAtLeast(ids::stat_cycle(), 5),
            Condition::StatAtMost(ids::stat_cycle(), 6),
        ]))
        .once(),
    );

    area
}

fn register_tool(
    area: &mut AreaContents,
    fixture: EntityId,
    name: &str,
    description: &str,
    dialogue: DialogueId,
) {
    area.add_entity(
        fixture,
        Entity::object(name, Text::lit(description))
            .with_dialogue(dialogue)
            .starting_in(ids::room_desk()),
    );
}
