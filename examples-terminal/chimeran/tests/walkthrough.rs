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
    let choices = engine.available_choices(state);
    let parsed = parser::parse(engine, state, &choices, command);
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

/// The travel block from the walkthrough: bedroom → desk.
const WALK_TO_DESK: &[&str] = &["s", "w", "d", "d", "s", "e", "e"];

/// The sleep sequence from the walkthrough (requires having been at
/// the desk this cycle).
const SLEEP_AT_BED: &[&str] = &["open bed", "get into bed"];

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
