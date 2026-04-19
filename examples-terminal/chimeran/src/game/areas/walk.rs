use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{Condition, Exit, Room, Text};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_room(
        ids::room_building_corridor(),
        Room::new(
            "Building Corridor",
            Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                then: Box::new(Text::lit(
                    "A corridor. Identical doors in both directions. None of them open. None of them have numbers.",
                )),
                otherwise: Box::new(Text::lit(
                    "A corridor outside your apartment. Identical doors extend in both directions. You have never met a neighbor. The hallway is still and well-lit.",
                )),
            },
        )
        .with_unseen_alias("an apartment corridor")
        .with_alias_when(Condition::StatAtLeast(ids::stat_env(), 6))
        .with_exit(Exit::new("east (back to your apartment)", ids::room_hallway()))
        .with_exit(Exit::new("down (elevator)", ids::room_elevator()))
        .with_examine(
            "doors",
            Text::lit(
                "Identical apartment doors at regular intervals. You have never seen one open. You do not know what numbers they used to have.",
            ),
        )
        .with_examine(
            "walls",
            Text::lit(
                "Beige corridor walls. Wainscoting at waist height. A strip of carpet trim along the base.",
            ),
        )
        .with_examine(
            "carpet",
            Text::lit(
                "Dark industrial carpet. Patterned to hide wear. It is spotless.",
            ),
        )
        .with_examine(
            "ceiling",
            Text::lit(
                "Recessed lights at even intervals down the corridor. None of them flicker.",
            ),
        )
        .with_examine(
            "lights",
            Text::lit(
                "Recessed fluorescents. Uniformly bright. They cast no shadows sharp enough to notice.",
            ),
        )
        .with_examine(
            "air",
            Text::lit(
                "The air is conditioned. It has no smell.",
            ),
        ),
    );

    area.add_room(
        ids::room_elevator(),
        Room::new(
            "Elevator",
            Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                then: Box::new(Text::lit(
                    "The elevator descends. It descends. It descends. The doors open.",
                )),
                otherwise: Box::new(Text::Conditional {
                    when: Condition::StatAtLeast(ids::stat_cycle(), 5),
                    then: Box::new(Text::lit(
                        "The elevator is empty in the way small rooms are empty when no one has been in them for a while. It descends, slowly.",
                    )),
                    otherwise: Box::new(Text::lit(
                        "A small elevator car. You step in; it descends. The numbers tick down in order.",
                    )),
                }),
            },
        )
        .with_unseen_alias("a cabin that went down")
        .with_alias_when(Condition::StatAtLeast(ids::stat_env(), 6))
        .with_exit(Exit::new("down (lobby)", ids::room_lobby()))
        .with_exit(Exit::new(
            "up (back up to your floor)",
            ids::room_building_corridor(),
        ))
        .with_examine(
            "panel",
            Text::lit(
                "A brass button panel. L, 1, 2, 3, 4, 5. Your floor is lit from before you stepped in.",
            ),
        )
        .with_examine(
            "buttons",
            Text::lit(
                "Round brass buttons, lightly worn. The one for your floor glows already.",
            ),
        )
        .with_examine(
            "mirror",
            Text::lit(
                "A mirror on the back wall of the elevator. Your reflection does what you do, a fraction of a second behind.",
            ),
        )
        .with_examine(
            "doors",
            Text::lit(
                "Stainless elevator doors. You can see a stretched reflection of yourself in them.",
            ),
        )
        .with_examine(
            "numbers",
            Text::lit(
                "A small display above the doors shows the floor number. It changes, ticking, as the car moves.",
            ),
        )
        .with_examine(
            "ceiling",
            Text::lit(
                "A small grid of ceiling lights. A maintenance panel you have never seen opened.",
            ),
        )
        .with_examine(
            "walls",
            Text::lit(
                "Padded elevator walls. A handrail. The pattern on the padding repeats.",
            ),
        )
        .with_examine(
            "floor",
            Text::lit(
                "A square of floor tile. Clean. Recently mopped, you think.",
            ),
        )
        .with_examine(
            "handrail",
            Text::lit(
                "A brass handrail circling the interior. You hold it when the elevator is slow.",
            ),
        ),
    );

    area.add_room(
        ids::room_lobby(),
        Room::new(
            "Lobby",
            Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                then: Box::new(Text::lit(
                    "A lobby. A desk without a person. No potted plant. The doors open onto a city that has never had a person in it.",
                )),
                otherwise: Box::new(Text::Conditional {
                    when: Condition::StatAtLeast(ids::stat_cycle(), 5),
                    then: Box::new(Text::lit(
                        "A lobby. A desk with no receptionist. A potted plant, though you are not sure if it was always there. The front doors open onto a plausible city street.",
                    )),
                    otherwise: Box::new(Text::lit(
                        "A lobby. A reception desk with no receptionist. A potted plant. The front doors open onto a plausible city street.",
                    )),
                }),
            },
        )
        .with_unseen_alias("a lobby")
        .with_alias_when(Condition::StatAtLeast(ids::stat_env(), 6))
        .with_exit(Exit::new("up (elevator)", ids::room_elevator()))
        .with_exit(Exit::new("south (out to the street)", ids::room_street()))
        .with_examine(
            "desk",
            Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                then: Box::new(Text::lit(
                    "A reception desk. No chair behind it. No computer. No intercom. A place for a person who has never been here.",
                )),
                otherwise: Box::new(Text::lit(
                    "A reception desk. No receptionist. A small bell you have never rung. A clipboard nobody has signed.",
                )),
            },
        )
        .with_examine(
            "plant",
            Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                then: Box::new(Text::lit(
                    "There is no potted plant in the lobby.",
                )),
                otherwise: Box::new(Text::lit(
                    "A potted ficus in the corner. The leaves are the same leaves every morning.",
                )),
            },
        )
        .with_examine(
            "doors",
            Text::lit(
                "The front doors are double glass, propped open in the warmer months. You walk through them.",
            ),
        )
        .with_examine(
            "bell",
            Text::lit(
                "A small brass service bell on the desk. You have never rung it.",
            ),
        )
        .with_examine(
            "clipboard",
            Text::lit(
                "A clipboard with a sign-in sheet. The sheet is blank.",
            ),
        )
        .with_examine(
            "floor",
            Text::lit(
                "Polished lobby tile. Reflects the overhead lights.",
            ),
        )
        .with_examine(
            "walls",
            Text::lit(
                "Cream-coloured lobby walls. A bulletin board with nothing on it.",
            ),
        )
        .with_examine(
            "ceiling",
            Text::lit(
                "A high lobby ceiling. A central light fixture glowing evenly.",
            ),
        )
        .with_examine(
            "bulletin board",
            Text::lit(
                "A corkboard on the far wall. No notices. A few pushpins in an empty grid.",
            ),
        )
        .with_examine(
            "air",
            Text::lit(
                "The lobby air is slightly cold. The vents overhead run constantly.",
            ),
        ),
    );

    area.add_room(
        ids::room_street(),
        Room::new(
            "Street",
            Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                then: Box::new(Text::lit(
                    "The street is silent. It is the silence of a place no one has ever been in.",
                )),
                otherwise: Box::new(Text::Conditional {
                    when: Condition::StatAtLeast(ids::stat_cycle(), 5),
                    then: Box::new(Text::lit(
                        "A city street. Nobody in sight. Early morning. It is always early morning when you go to work.",
                    )),
                    otherwise: Box::new(Text::lit(
                        "A short stretch of city street between your building and the office. Early morning; no one around.",
                    )),
                }),
            },
        )
        .with_unseen_alias("the outside")
        .with_alias_when(Condition::StatAtLeast(ids::stat_env(), 6))
        .with_exit(Exit::new(
            "north (back inside your building)",
            ids::room_lobby(),
        ))
        .with_exit(Exit::new(
            "east (into the office building)",
            ids::room_office_floor(),
        ))
        .with_examine(
            "buildings",
            Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                then: Box::new(Text::lit(
                    "The buildings along the street. They are the same buildings every morning. Their windows are dark. You have never seen a light in any of them.",
                )),
                otherwise: Box::new(Text::lit(
                    "The buildings on either side. Your residential block. The office building across the way.",
                )),
            },
        )
        .with_examine(
            "sky",
            Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                then: Box::new(Text::lit(
                    "The sky is the same soft overcast as yesterday. As the morning before. Always morning.",
                )),
                otherwise: Box::new(Text::lit(
                    "A soft, even morning sky. Pale. Uncommitted.",
                )),
            },
        )
        .with_examine(
            "pavement",
            Text::lit(
                "The sidewalk. Clean. No trash, no marks. No one has walked here since you last did.",
            ),
        )
        .with_examine(
            "road",
            Text::lit(
                "An empty street. Painted lines along the centre. The paint looks fresh. It is always fresh.",
            ),
        )
        .with_examine(
            "cars",
            Text::lit(
                "No cars on the street. None parked, none passing. It is always this quiet.",
            ),
        )
        .with_examine(
            "windows",
            Text::lit(
                "The building windows around you. You cannot see into any of them. The glass is plain.",
            ),
        )
        .with_examine(
            "air",
            Text::lit(
                "The air outside. Cool. Smelling of nothing in particular.",
            ),
        )
        .with_examine(
            "light",
            Text::lit(
                "Morning light. Even, unshifting. You have stood here and watched and not seen it change.",
            ),
        )
        .with_examine(
            "street",
            Text::lit(
                "A short stretch of empty street. A crosswalk with no traffic to stop.",
            ),
        ),
    );

    area.add_room(
        ids::room_office_floor(),
        Room::new(
            "Office Floor",
            Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                then: Box::new(Text::lit(
                    "A hallway. Your office door is at the end. The other doors on either side do not open. You know this because you never try them.",
                )),
                otherwise: Box::new(Text::Conditional {
                    when: Condition::All(vec![
                        Condition::StatAtLeast(ids::stat_cycle(), 4),
                        Condition::StatAtMost(ids::stat_cycle(), 4),
                    ]),
                    then: Box::new(Text::lit(
                        "A hallway. Your office door is at the end. Other doors on either side — and in your peripheral vision, as you pass, one of them briefly appears open. When you turn to look, the door is closed. It was never open.",
                    )),
                    otherwise: Box::new(Text::Conditional {
                        when: Condition::StatAtLeast(ids::stat_cycle(), 5),
                        then: Box::new(Text::lit(
                            "A hallway. Your office door is at the end. Other doors on either side, none of which you've ever seen open.",
                        )),
                        otherwise: Box::new(Text::lit(
                            "A clean office hallway. Your office door is at the end of the hall. Other doors on either side are closed.",
                        )),
                    }),
                }),
            },
        )
        .with_unseen_alias("an office hallway")
        .with_alias_when(Condition::StatAtLeast(ids::stat_env(), 6))
        .with_exit(Exit::new("west (back to the street)", ids::room_street()))
        .with_exit(Exit::new("east (into your office)", ids::room_desk()))
        .with_examine(
            "walls",
            Text::lit(
                "The hallway walls. Painted a neutral grey-green. A few framed prints hung at head height.",
            ),
        )
        .with_examine(
            "prints",
            Text::lit(
                "Framed prints at intervals along the wall. Abstract. Inoffensive. You could not describe them after turning away.",
            ),
        )
        .with_examine(
            "floor",
            Text::lit(
                "Low-pile office carpet. A single mark of wear down the centre of the hall.",
            ),
        )
        .with_examine(
            "ceiling",
            Text::lit(
                "Acoustic tiles. A recessed light every few yards. A vent cover.",
            ),
        )
        .with_examine(
            "lights",
            Text::lit(
                "Fluorescent troffers behind the ceiling tiles. They hum very faintly.",
            ),
        )
        .with_examine(
            "air",
            Text::lit(
                "The office air. Slightly dry. Conditioned.",
            ),
        )
        .with_examine(
            "doors",
            Text::Conditional {
                when: Condition::All(vec![
                    Condition::StatAtLeast(ids::stat_cycle(), 4),
                    Condition::StatAtMost(ids::stat_cycle(), 4),
                ]),
                then: Box::new(Text::lit(
                    "You check the door that seemed open. It is closed. You try the handle anyway. It does not turn. No door on this floor ever opens. You already knew that.",
                )),
                otherwise: Box::new(Text::lit(
                    "The other doors on this floor are closed. You have never seen any of them open.",
                )),
            },
        ),
    );

    area
}
