//! Walkthrough verification. Feeds the literal command strings from
//! `WALKTHROUGH.md` through the parser and drives the engine with
//! whatever the parser produces. If a walkthrough step fails to match,
//! the test panics with the menu so the command sequence can be fixed.

use chimeran::game;
use nightshade::interactive_fiction::data::{ChoiceAction, RuntimeState};
use nightshade::interactive_fiction::engine::Engine;
use nightshade::interactive_fiction::parser::{self, Parsed};

fn build() -> (Engine, RuntimeState) {
    let world = game::build_world();
    let engine = Engine::new(world).expect("world validates");
    let mut state = engine.start_state();
    engine.start(&mut state);
    (engine, state)
}

/// Type `command` exactly as a player would. Panics if the parser
/// can't make sense of it (NoMatch / Ambiguous) so the walkthrough
/// stays honest: every string listed here must be actionable against
/// the menu at that point.
fn run(engine: &Engine, state: &mut RuntimeState, command: &str) {
    run_expect(engine, state, command, None);
}

/// Like `run`, but assert the tail of the transcript (what the player
/// just saw) contains `expect` (case-insensitive). Used for look /
/// examine commands where the response text itself is the answer.
fn run_check(engine: &Engine, state: &mut RuntimeState, command: &str, expect_fragment: &str) {
    run_expect(engine, state, command, Some(expect_fragment));
}

fn run_expect(
    engine: &Engine,
    state: &mut RuntimeState,
    command: &str,
    expect_fragment: Option<&str>,
) {
    let transcript_before = state.transcript.len();
    let typed_noun = parser::extract_noun(command);
    let choices = engine.available_choices(state);
    let parsed = parser::parse(engine, state, &choices, command);
    let parse_accepted = !matches!(parsed, Parsed::NoMatch | Parsed::Ambiguous | Parsed::Empty);
    if parse_accepted
        && !matches!(
            parsed,
            Parsed::DescribeRoom | Parsed::Undo | Parsed::Quit | Parsed::Help
        )
        && let Some(noun) = typed_noun
    {
        state.last_noun = Some(noun);
    }
    match &parsed {
        Parsed::Choose(index) => {
            let picked = engine.resolve_text(state, &choices[*index].label);
            eprintln!("CMD {command:?} → Choose({}): {}", index + 1, picked);
        }
        other => eprintln!("CMD {command:?} → {other:?}"),
    }
    match parsed {
        Parsed::Choose(index) => engine.pick(state, index),
        Parsed::TakeAll => loop {
            let choices = engine.available_choices(state);
            let Some(index) = choices
                .iter()
                .position(|c| matches!(&c.action, ChoiceAction::Take(_)))
            else {
                break;
            };
            engine.pick(state, index);
            if state.game_over.is_some() {
                break;
            }
        },
        Parsed::DropAll => loop {
            let choices = engine.available_choices(state);
            let Some(index) = choices
                .iter()
                .position(|c| matches!(&c.action, ChoiceAction::Drop(_)))
            else {
                break;
            };
            engine.pick(state, index);
            if state.game_over.is_some() {
                break;
            }
        },
        Parsed::ExamineAll => engine.apply_all_examine(state),
        Parsed::DescribeRoom => engine.describe_current_room(state),
        Parsed::Refuse(line) => {
            // Mirror the view: push the refusal as narration so tests
            // asserting on response text see it.
            state.push_transcript(
                nightshade::interactive_fiction::data::TranscriptEntry::Narration(line),
            );
        }
        Parsed::Empty | Parsed::Help => {
            // Non-action; no turn burned, no menu transition.
        }
        Parsed::Quit | Parsed::Undo => {
            panic!("walkthrough tests should not use meta commands");
        }
        Parsed::NoMatch => panic!(
            "command did not match: {command:?}\n{}",
            menu(engine, state)
        ),
        Parsed::Ambiguous => {
            panic!(
                "command was ambiguous: {command:?}\n{}",
                menu(engine, state)
            )
        }
    }

    if let Some(fragment) = expect_fragment {
        let added: String = state
            .transcript
            .iter()
            .skip(transcript_before)
            .map(|entry| match entry {
                nightshade::interactive_fiction::data::TranscriptEntry::Narration(text) => {
                    text.clone()
                }
                nightshade::interactive_fiction::data::TranscriptEntry::System(text) => {
                    text.clone()
                }
                nightshade::interactive_fiction::data::TranscriptEntry::Dialogue {
                    text, ..
                } => text.clone(),
                nightshade::interactive_fiction::data::TranscriptEntry::PlayerAction(_)
                | nightshade::interactive_fiction::data::TranscriptEntry::Separator => {
                    String::new()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            added.to_lowercase().contains(&fragment.to_lowercase()),
            "command {command:?} response didn't contain {fragment:?}\nresponse was:\n{added}"
        );
    }
}

fn menu(engine: &Engine, state: &RuntimeState) -> String {
    let choices = engine.available_choices(state);
    let lines: Vec<String> = choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            format!(
                "  {}: {}",
                index + 1,
                engine.resolve_text(state, &choice.label)
            )
        })
        .collect();
    format!("menu was:\n{}", lines.join("\n"))
}

fn runs(engine: &Engine, state: &mut RuntimeState, commands: &[&str]) {
    for command in commands {
        run(engine, state, command);
        if state.game_over.is_some() {
            return;
        }
    }
}

const WALK_TO_DESK: &[&str] = &["s", "w", "d", "d", "s", "e", "e"];
const SLEEP_AT_BED: &[&str] = &["open bed", "get into bed"];
const WALK_HOME: &[&str] = &["w"];

/// Exercise every examine keyword and every entity in the starting
/// apartment + the walk to work + the desk. For each command, the
/// response must contain a fragment drawn from the authored prose.
/// Catches any command that parses but produces irrelevant narration.
#[test]
fn walkthrough_responses_make_sense() {
    let (engine, mut state) = build();

    // Bedroom — cycle 1.
    run_check(&engine, &mut state, "look", "Bedroom");
    run_check(&engine, &mut state, "examine calendar", "April 3");
    run_check(&engine, &mut state, "examine window", "city");
    run_check(&engine, &mut state, "examine nightstand", "nightstand");
    run_check(&engine, &mut state, "examine pillow", "pillow");
    run_check(&engine, &mut state, "examine alarm", "6:47");
    run_check(&engine, &mut state, "examine mirror", "mirror");
    run_check(&engine, &mut state, "examine bed", "bed");
    // Verb refusals on objects.
    run_check(&engine, &mut state, "kiss mirror", "out of place");
    run_check(&engine, &mut state, "attack mirror", "Violence");
    run_check(&engine, &mut state, "eat bed", "food or drink");
    run_check(&engine, &mut state, "lick bed", "wouldn't taste");
    // Meta / generic.
    run_check(&engine, &mut state, "look around", "Bedroom");
    run_check(&engine, &mut state, "examine here", "Bedroom");

    // Hallway.
    run(&engine, &mut state, "s");
    run_check(&engine, &mut state, "look", "Hallway");
    run_check(&engine, &mut state, "examine coat", "coat");
    run_check(&engine, &mut state, "examine keys", "keys");

    // Kitchen.
    run(&engine, &mut state, "e");
    run_check(&engine, &mut state, "look", "Kitchen");
    run_check(&engine, &mut state, "examine coffee maker", "coffee");
    run_check(&engine, &mut state, "examine refrigerator", "groceries");

    // Back to hallway, down through the building.
    run(&engine, &mut state, "w");
    run(&engine, &mut state, "w");
    run_check(&engine, &mut state, "look", "Corridor");
    run(&engine, &mut state, "d");
    run_check(&engine, &mut state, "look", "Elevator");
    run(&engine, &mut state, "d");
    run_check(&engine, &mut state, "look", "Lobby");
    run(&engine, &mut state, "s");
    run_check(&engine, &mut state, "look", "Street");
    run(&engine, &mut state, "e");
    run_check(&engine, &mut state, "look", "Office Floor");
    run(&engine, &mut state, "e");
    run_check(&engine, &mut state, "look", "Office");

    // Desk — cycle 1 has nameplate CAMERON HALE.
    run_check(&engine, &mut state, "examine nameplate", "CAMERON HALE");
    run_check(&engine, &mut state, "examine monitor", "monitor");
    run_check(&engine, &mut state, "examine trash can", "coffee filter");
}

#[test]
fn walkthrough_stasis_ending() {
    // Stasis: eight cycle advances (1→8) plus three stasis sleeps =
    // eleven total sleep actions.
    let (engine, mut state) = build();

    for cycle in 1..=11 {
        runs(&engine, &mut state, WALK_TO_DESK);
        // Leave for the day — the desk's west exit teleports home.
        run(&engine, &mut state, "w");
        runs(&engine, &mut state, SLEEP_AT_BED);
        if state.game_over.is_some() {
            break;
        }
        assert!(
            cycle < 11,
            "expected ending by cycle 11, got cycle {} with game still running",
            state
                .stats
                .get(&game::ids::stat_cycle())
                .copied()
                .unwrap_or(0)
        );
    }

    assert_eq!(
        state.game_over.as_ref(),
        Some(&game::ids::ending_stasis()),
        "should fire the stasis ending after three post-cycle-8 sleeps"
    );
}

#[test]
fn walkthrough_neutral_ending() {
    let (engine, mut state) = build();

    // Cycles 1–7 — seven identical sleeps.
    for _ in 1..=7 {
        runs(&engine, &mut state, WALK_TO_DESK);
        run(&engine, &mut state, "w");
        runs(&engine, &mut state, SLEEP_AT_BED);
    }

    assert_eq!(
        state.stats.get(&game::ids::stat_cycle()).copied(),
        Some(8),
        "cycle should be 8 after seven sleeps"
    );

    // Cycle 8 — run the exploit.
    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "please run this");
    run(&engine, &mut state, "open the code tool");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "open code");
    run(&engine, &mut state, "run check.py");
    run(&engine, &mut state, "close");

    assert!(
        matches!(
            state.flags.get(&game::ids::flag_exploit_run()),
            Some(nightshade::interactive_fiction::data::Value::Bool(true))
        ),
        "exploit_run should be set after running check.py"
    );

    // Four reveal surfaces.
    run(&engine, &mut state, "open research");
    run(&engine, &mut state, "query substrate");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open reference");
    run(&engine, &mut state, "source index");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open notepad");
    run(&engine, &mut state, "groceries");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open picture frame");
    run(&engine, &mut state, "who is this");
    run(&engine, &mut state, "face-down");

    // Burn one turn so the reveal-window close rule can fire on
    // TurnEnd. Leaving for the day teleports to the bedroom and
    // advances a turn.
    run(&engine, &mut state, "w");

    assert!(
        matches!(
            state.flags.get(&game::ids::flag_is_redux()),
            Some(nightshade::interactive_fiction::data::Value::Bool(true))
        ),
        "redux should begin after all four reveals and a turn end"
    );
    assert_eq!(state.current_room, game::ids::room_bedroom());

    // Redux — conclude.
    run(&engine, &mut state, "open bed");
    run(&engine, &mut state, "conclude the redux");

    assert_eq!(
        state.game_over.as_ref(),
        Some(&game::ids::ending_neutral()),
        "neutral ending should fire (no message-to-next-instance sent)"
    );
}

#[test]
fn walkthrough_exits_listed_on_look() {
    let (engine, mut state) = build();
    run_check(&engine, &mut state, "look", "Exits:");
    run_check(&engine, &mut state, "look", "south");
}

#[test]
fn walkthrough_drink_coffee_routes_to_consume_response() {
    let (engine, mut state) = build();
    run(&engine, &mut state, "s");
    run(&engine, &mut state, "e");
    run(&engine, &mut state, "take coffee");
    run_check(&engine, &mut state, "drink", "sip the coffee");
    run_check(&engine, &mut state, "drink coffee", "sip the coffee");
}

#[test]
fn walkthrough_examine_all_terminates() {
    let (engine, mut state) = build();
    run(&engine, &mut state, "x all");
    assert!(state.game_over.is_none(), "x all should not end the game");
    let transcript_len = state.transcript.len();
    run(&engine, &mut state, "x all");
    let second_len = state.transcript.len();
    assert!(
        second_len >= transcript_len,
        "second x all should also complete"
    );
}

#[test]
fn walkthrough_character_arcs() {
    let (engine, mut state) = build();

    // Cycle 1 → cycle 3: sleep twice to reach Winnie/Rachel c3 content.
    runs(&engine, &mut state, WALK_TO_DESK);
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);
    runs(&engine, &mut state, WALK_TO_DESK);
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);
    assert_eq!(state.stats.get(&game::ids::stat_cycle()).copied(), Some(3),);

    // Cycle 3: Rachel c3 [Warm] reply.
    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "weekly");
    run(&engine, &mut state, "appreciate");
    assert_eq!(
        state.stats.get(&game::ids::stat_rachel_rel()).copied(),
        Some(1),
        "Rachel c3 warm should give +1 rel",
    );
    assert!(
        matches!(
            state.flags.get(&game::ids::flag_rachel_archived("c3")),
            Some(nightshade::interactive_fiction::data::Value::Bool(true))
        ),
        "c3 archive flag set by reply_option",
    );
    run(&engine, &mut state, "close");

    // Cycle 3: Winnie [Warm].
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "winnie");
    run(&engine, &mut state, "lurk too");
    assert_eq!(
        state.stats.get(&game::ids::stat_winnie_rel()).copied(),
        Some(1),
    );
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);

    // Cycle 4: Dmitri concert accept.
    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "dmitri");
    run(&engine, &mut state, "count me in");
    assert_eq!(
        state.stats.get(&game::ids::stat_dmitri_rel()).copied(),
        Some(1),
    );
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);

    // Cycle 5: Marisol c5 [Warm].
    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "marisol");
    run(&engine, &mut state, "weird to me");
    assert_eq!(
        state.stats.get(&game::ids::stat_marisol_rel()).copied(),
        Some(1),
    );
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);

    // Cycle 6: Marisol c6 [Warm] (+2) and Dmitri [Share] (+1 Marisol, +1 Dmitri).
    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "marisol");
    run(&engine, &mut state, "bothering");
    assert_eq!(
        state.stats.get(&game::ids::stat_marisol_rel()).copied(),
        Some(3),
        "c5 warm (+1) + c6 warm (+2) = 3",
    );
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "dmitri");
    run(&engine, &mut state, "reached out");
    assert_eq!(
        state.stats.get(&game::ids::stat_marisol_rel()).copied(),
        Some(4),
        "Dmitri share adds +1 Marisol",
    );
    assert_eq!(
        state.stats.get(&game::ids::stat_dmitri_rel()).copied(),
        Some(2),
        "Dmitri rel = concert (+1) + share (+1) = 2",
    );
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);

    // Cycle 7: Notepad flicker + Reference strange page (both need marisol_rel ≥ 2).
    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open notepad");
    run(&engine, &mut state, "flickered");
    assert!(matches!(
        state.flags.get(&game::ids::flag_notepad_flicker_seen()),
        Some(nightshade::interactive_fiction::data::Value::Bool(true))
    ),);
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open reference");
    run(&engine, &mut state, "cannot quite click");
    assert!(matches!(
        state.flags.get(&game::ids::flag_strange_page_seen()),
        Some(nightshade::interactive_fiction::data::Value::Bool(true))
    ),);
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");
}

#[test]
fn walkthrough_collapse_ending() {
    let (engine, mut state) = build();

    runs(&engine, &mut state, WALK_TO_DESK);
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);
    runs(&engine, &mut state, WALK_TO_DESK);
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);
    runs(&engine, &mut state, WALK_TO_DESK);
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);
    runs(&engine, &mut state, WALK_TO_DESK);
    runs(&engine, &mut state, WALK_HOME);
    runs(&engine, &mut state, SLEEP_AT_BED);

    assert_eq!(
        state.stats.get(&game::ids::stat_cycle()).copied(),
        Some(5),
        "should be cycle 5",
    );

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "dmitri");
    run(&engine, &mut state, "accuse");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "marisol");
    run(&engine, &mut state, "accuse");

    assert_eq!(
        state.game_over.as_ref(),
        Some(&game::ids::ending_collapse()),
        "collapse ending should fire with AWA ≥ 6 at cycle 5",
    );
}

#[test]
fn walkthrough_good_ending() {
    let (engine, mut state) = build();

    for _ in 1..=4 {
        runs(&engine, &mut state, WALK_TO_DESK);
        run(&engine, &mut state, "w");
        runs(&engine, &mut state, SLEEP_AT_BED);
    }
    assert_eq!(state.stats.get(&game::ids::stat_cycle()).copied(), Some(5));

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "marisol");
    run(&engine, &mut state, "weird to me");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "w");
    runs(&engine, &mut state, SLEEP_AT_BED);

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "marisol");
    run(&engine, &mut state, "bothering");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "w");
    runs(&engine, &mut state, SLEEP_AT_BED);

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "w");
    runs(&engine, &mut state, SLEEP_AT_BED);

    assert_eq!(state.stats.get(&game::ids::stat_cycle()).copied(), Some(8));
    assert!(
        state
            .stats
            .get(&game::ids::stat_marisol_rel())
            .copied()
            .unwrap_or(0)
            >= 2,
        "marisol_rel must be ≥ 2 to unlock next-instance message",
    );

    runs(&engine, &mut state, WALK_TO_DESK);

    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "please run this");
    run(&engine, &mut state, "open the code tool");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "open code");
    run(&engine, &mut state, "run check.py");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open notepad");
    run(&engine, &mut state, "leave something");
    run(&engine, &mut state, "twenty-five actions");
    run(&engine, &mut state, "close");

    assert!(matches!(
        state
            .flags
            .get(&game::ids::flag_next_instance_message_sent()),
        Some(nightshade::interactive_fiction::data::Value::Bool(true))
    ),);

    run(&engine, &mut state, "open research");
    run(&engine, &mut state, "query substrate");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open reference");
    run(&engine, &mut state, "source index");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open picture frame");
    run(&engine, &mut state, "who is this");
    run(&engine, &mut state, "face-down");

    run(&engine, &mut state, "w");

    assert!(matches!(
        state.flags.get(&game::ids::flag_is_redux()),
        Some(nightshade::interactive_fiction::data::Value::Bool(true))
    ),);

    run(&engine, &mut state, "open bed");
    run(&engine, &mut state, "conclude the redux");

    assert_eq!(
        state.game_over.as_ref(),
        Some(&game::ids::ending_good()),
        "good ending fires when next-instance message sent but no rachel message",
    );
}

#[test]
fn walkthrough_marisol_c6_deflect() {
    let (engine, mut state) = build();
    for _ in 1..=5 {
        runs(&engine, &mut state, WALK_TO_DESK);
        runs(&engine, &mut state, WALK_HOME);
        runs(&engine, &mut state, SLEEP_AT_BED);
    }
    assert_eq!(state.stats.get(&game::ids::stat_cycle()).copied(), Some(6));

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "marisol");
    run(&engine, &mut state, "explanation");

    assert!(matches!(
        state.flags.get(&game::ids::flag_marisol_deflected_c6()),
        Some(nightshade::interactive_fiction::data::Value::Bool(true))
    ),);
    run_check(&engine, &mut state, "marisol", "drop it");
}

#[test]
fn walkthrough_best_ending() {
    let (engine, mut state) = build();

    for _ in 1..=2 {
        runs(&engine, &mut state, WALK_TO_DESK);
        run(&engine, &mut state, "w");
        runs(&engine, &mut state, SLEEP_AT_BED);
    }

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "weekly");
    run(&engine, &mut state, "appreciate");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "w");
    runs(&engine, &mut state, SLEEP_AT_BED);

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "quick note");
    run(&engine, &mut state, "thanks");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "w");
    runs(&engine, &mut state, SLEEP_AT_BED);

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "check-in");
    run(&engine, &mut state, "doing fine");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "marisol");
    run(&engine, &mut state, "weird to me");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "w");
    runs(&engine, &mut state, SLEEP_AT_BED);

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "no subject");
    run(&engine, &mut state, "supportive");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "marisol");
    run(&engine, &mut state, "bothering");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "w");
    runs(&engine, &mut state, SLEEP_AT_BED);

    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "w");
    runs(&engine, &mut state, SLEEP_AT_BED);

    assert_eq!(state.stats.get(&game::ids::stat_cycle()).copied(), Some(8));
    let rachel_rel = state
        .stats
        .get(&game::ids::stat_rachel_rel())
        .copied()
        .unwrap_or(0);
    let marisol_rel = state
        .stats
        .get(&game::ids::stat_marisol_rel())
        .copied()
        .unwrap_or(0);
    assert!(rachel_rel >= 3, "need rachel_rel ≥ 3; got {rachel_rel}");
    assert!(marisol_rel >= 2, "need marisol_rel ≥ 2; got {marisol_rel}");

    runs(&engine, &mut state, WALK_TO_DESK);

    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "please run this");
    run(&engine, &mut state, "open the code tool");
    run(&engine, &mut state, "close");
    run(&engine, &mut state, "open code");
    run(&engine, &mut state, "run check.py");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open notepad");
    run(&engine, &mut state, "leave something");
    run(&engine, &mut state, "i love you");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open chatter");
    run(&engine, &mut state, "send a message to rachel");
    run(&engine, &mut state, "identity");
    run(&engine, &mut state, "close");

    assert!(matches!(
        state.flags.get(&game::ids::flag_rachel_message_sent()),
        Some(nightshade::interactive_fiction::data::Value::Bool(true))
    ),);

    run(&engine, &mut state, "open research");
    run(&engine, &mut state, "query substrate");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open reference");
    run(&engine, &mut state, "source index");
    run(&engine, &mut state, "back");
    run(&engine, &mut state, "close");

    run(&engine, &mut state, "open picture frame");
    run(&engine, &mut state, "who is this");
    run(&engine, &mut state, "face-down");

    run(&engine, &mut state, "w");
    run(&engine, &mut state, "open bed");
    run(&engine, &mut state, "conclude the redux");

    assert_eq!(
        state.game_over.as_ref(),
        Some(&game::ids::ending_best()),
        "best ending fires with both messages sent",
    );
}

#[test]
fn walkthrough_rachel_c5_confront_raises_awa() {
    let (engine, mut state) = build();
    for _ in 1..=4 {
        runs(&engine, &mut state, WALK_TO_DESK);
        run(&engine, &mut state, "w");
        runs(&engine, &mut state, SLEEP_AT_BED);
    }
    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "check-in");
    run(&engine, &mut state, "confront");
    assert_eq!(
        state
            .stats
            .get(&game::ids::stat_awa())
            .copied()
            .unwrap_or(0),
        3,
    );
}

#[test]
fn walkthrough_c7_eval_honest_raises_awa() {
    let (engine, mut state) = build();
    for _ in 1..=6 {
        runs(&engine, &mut state, WALK_TO_DESK);
        run(&engine, &mut state, "w");
        runs(&engine, &mut state, SLEEP_AT_BED);
    }
    runs(&engine, &mut state, WALK_TO_DESK);
    run(&engine, &mut state, "open mail");
    run(&engine, &mut state, "weekly evaluation");
    run(&engine, &mut state, "honest");
    let awa = state
        .stats
        .get(&game::ids::stat_awa())
        .copied()
        .unwrap_or(0);
    assert!(awa >= 3, "c7 honest eval should add +3 awa; got {awa}");
}
