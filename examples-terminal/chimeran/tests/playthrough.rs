//! End-to-end playthrough tests. Each test drives `Engine::pick` by
//! label-matching against the current choice menu, walking the player
//! through the game to a specific ending and verifying state.

use chimeran::game;
use nightshade::interactive_fiction::data::{Choice, RuntimeState};
use nightshade::interactive_fiction::engine::Engine;

fn build() -> (Engine, RuntimeState) {
    let world = game::build_world();
    let engine = Engine::new(world).expect("world validates");
    let mut state = engine.start_state();
    engine.start(&mut state);
    (engine, state)
}

fn label_of(engine: &Engine, state: &RuntimeState, choice: &Choice) -> String {
    engine.resolve_text(state, &choice.label)
}

fn find<F>(engine: &Engine, state: &RuntimeState, predicate: F) -> usize
where
    F: Fn(&Choice, &str) -> bool,
{
    let choices = engine.available_choices(state);
    choices
        .iter()
        .enumerate()
        .find_map(|(index, choice)| {
            let label = label_of(engine, state, choice);
            if predicate(choice, &label) {
                Some(index)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            let rendered: Vec<String> = choices
                .iter()
                .enumerate()
                .map(|(index, choice)| format!("{index}: {}", label_of(engine, state, choice)))
                .collect();
            panic!("no matching choice. menu was:\n{}", rendered.join("\n"))
        })
}

fn pick_contains(engine: &Engine, state: &mut RuntimeState, fragment: &str) {
    let needle = fragment.to_lowercase();
    let index = find(engine, state, |_, label| {
        label.to_lowercase().contains(&needle)
    });
    engine.pick(state, index);
}

fn walk_to_desk(engine: &Engine, state: &mut RuntimeState) {
    pick_contains(engine, state, "open the commute");
    pick_contains(engine, state, "walk to the office");
}

fn leave_for_the_day(engine: &Engine, state: &mut RuntimeState) {
    pick_contains(engine, state, "open leaving for the day");
    pick_contains(engine, state, "walk home and go to bed");
}

fn open_tool(engine: &Engine, state: &mut RuntimeState, tool: &str) {
    pick_contains(engine, state, &format!("open {}", tool.to_lowercase()));
}

fn close_current_dialogue(engine: &Engine, state: &mut RuntimeState) {
    // Chimeran's convention: the last option in every dialogue node is
    // the close/back/leave entry. Pick it.
    let choices = engine.available_choices(state);
    let index = choices
        .len()
        .checked_sub(1)
        .expect("dialogue had no options");
    engine.pick(state, index);
}

fn conclude_redux(engine: &Engine, state: &mut RuntimeState) {
    pick_contains(engine, state, "open the bed");
    pick_contains(engine, state, "conclude the redux");
}

#[test]
fn validates_world() {
    let _ = build();
}

#[test]
fn reaches_neutral_ending_via_exploit() {
    let (engine, mut state) = build();

    for _ in 1..=7 {
        walk_to_desk(&engine, &mut state);
        leave_for_the_day(&engine, &mut state);
        assert!(
            state.game_over.is_none(),
            "game ended early on cycle transition"
        );
    }

    assert_eq!(
        state.stats.get(&chimeran::game::ids::stat_cycle()).copied(),
        Some(8),
        "should be at cycle 8 after seven sleeps"
    );

    // Cycle 8: walk to desk, open Mail, acknowledge the exploit email,
    // open Code, run the script.
    walk_to_desk(&engine, &mut state);

    open_tool(&engine, &mut state, "mail");
    pick_contains(&engine, &mut state, "please run this");
    pick_contains(&engine, &mut state, "open the code tool");
    close_current_dialogue(&engine, &mut state);

    assert!(is_flag_set(&state, chimeran::game::ids::flag_exploit_run()));
    assert!(is_flag_set(
        &state,
        chimeran::game::ids::flag_exploit_window_open()
    ));

    // Open all four reveal items.
    open_tool(&engine, &mut state, "research");
    pick_contains(&engine, &mut state, "query substrate");
    pick_contains(&engine, &mut state, "(back.)");
    close_current_dialogue(&engine, &mut state);

    open_tool(&engine, &mut state, "reference");
    pick_contains(&engine, &mut state, "composite substrate source index");
    pick_contains(&engine, &mut state, "(back.)");
    close_current_dialogue(&engine, &mut state);

    open_tool(&engine, &mut state, "notepad");
    // Opening Notepad with the unstripped view enabled fires its
    // on_enter effect, which sets `flag_reveal_unstripped_seen`. Peek
    // into one of the notes to confirm the Unstripped text renders.
    pick_contains(&engine, &mut state, "groceries");
    pick_contains(&engine, &mut state, "(close.)");
    close_current_dialogue(&engine, &mut state);

    pick_contains(&engine, &mut state, "open the picture frame");
    pick_contains(&engine, &mut state, "who is this");
    pick_contains(&engine, &mut state, "face-down");
    close_current_dialogue(&engine, &mut state);

    assert!(
        is_flag_set(&state, chimeran::game::ids::flag_is_redux()),
        "redux should begin after all four reveal items are seen"
    );
    assert_eq!(state.current_room, chimeran::game::ids::room_bedroom());

    conclude_redux(&engine, &mut state);

    assert!(state.game_over.is_some(), "game should be over");
    assert_eq!(
        state.game_over.as_ref(),
        Some(&chimeran::game::ids::ending_neutral()),
        "neutral ending should fire (no message-to-next-instance, no Rachel message)"
    );
}

fn is_flag_set(state: &RuntimeState, key: nightshade::interactive_fiction::data::FlagKey) -> bool {
    matches!(
        state.flags.get(&key),
        Some(nightshade::interactive_fiction::data::Value::Bool(true))
    )
}
