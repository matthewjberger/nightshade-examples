//! Integration tests for `nightshade::interactive_fiction::parser` driven by
//! the lighthouse content. Lives here (rather than in nightshade) because
//! these tests assert against lighthouse-specific choices, items, and
//! room features.

use lighthouse::game;
use nightshade::interactive_fiction::data::{Choice, ChoiceAction, ItemId, ItemLocation};
use nightshade::interactive_fiction::engine::Engine;
use nightshade::interactive_fiction::parser::{Parsed, parse};

fn fresh() -> (Engine, nightshade::interactive_fiction::data::RuntimeState) {
    let world = game::build_world();
    let engine = Engine::new(world).expect("validate");
    let mut state = engine.start_state();
    engine.start(&mut state);
    (engine, state)
}

fn find_index(choices: &[Choice], action_matches: impl Fn(&ChoiceAction) -> bool) -> usize {
    choices
        .iter()
        .position(|choice| action_matches(&choice.action))
        .expect("expected matching choice")
}

#[test]
fn digit_shortcut() {
    let (engine, state) = fresh();
    let choices = engine.available_choices(&state);
    assert_eq!(parse(&engine, &state, &choices, "1"), Parsed::Choose(0));
    assert_eq!(parse(&engine, &state, &choices, "3"), Parsed::Choose(2));
}

#[test]
fn direction_alone_selects_go() {
    let (engine, state) = fresh();
    let choices = engine.available_choices(&state);
    let east_index = choices
        .iter()
        .enumerate()
        .find_map(|(index, choice)| match choice.action {
            ChoiceAction::Go { ref to, .. } if *to == game::ids::room_cliff_path() => Some(index),
            _ => None,
        })
        .expect("go east choice");
    assert_eq!(
        parse(&engine, &state, &choices, "e"),
        Parsed::Choose(east_index)
    );
    assert_eq!(
        parse(&engine, &state, &choices, "east"),
        Parsed::Choose(east_index)
    );
    assert_eq!(
        parse(&engine, &state, &choices, "go east"),
        Parsed::Choose(east_index)
    );
}

#[test]
fn take_verb_matches_item_by_synonym() {
    let (engine, state) = fresh();
    let choices = engine.available_choices(&state);
    let take_key = find_index(
        &choices,
        |action| matches!(action, ChoiceAction::Take(id) if *id == game::ids::item_cottage_key()),
    );
    assert_eq!(
        parse(&engine, &state, &choices, "take key"),
        Parsed::Choose(take_key)
    );
    assert_eq!(
        parse(&engine, &state, &choices, "get the key"),
        Parsed::Choose(take_key)
    );
}

#[test]
fn quit_and_undo_are_meta() {
    let (engine, state) = fresh();
    let choices = engine.available_choices(&state);
    assert_eq!(parse(&engine, &state, &choices, "q"), Parsed::Quit);
    assert_eq!(parse(&engine, &state, &choices, "quit"), Parsed::Quit);
    assert_eq!(parse(&engine, &state, &choices, "undo"), Parsed::Undo);
}

#[test]
fn empty_and_no_match() {
    let (engine, state) = fresh();
    let choices = engine.available_choices(&state);
    assert_eq!(parse(&engine, &state, &choices, ""), Parsed::Empty);
    assert_eq!(parse(&engine, &state, &choices, "xyzzy"), Parsed::NoMatch);
}

#[test]
fn bare_take_without_noun_is_not_a_match() {
    let (engine, state) = fresh();
    let choices = engine.available_choices(&state);
    assert_eq!(parse(&engine, &state, &choices, "take"), Parsed::NoMatch);
}

#[test]
fn ambiguous_take_by_partial_noun() {
    let (engine, mut state) = fresh();
    // Place two items in the starting room whose names share the word "key".
    state.item_locations.insert(
        ItemId::new("tower_key"),
        ItemLocation::Room(game::ids::room_shore()),
    );
    let choices = engine.available_choices(&state);
    assert_eq!(
        parse(&engine, &state, &choices, "take key"),
        Parsed::Ambiguous
    );
}

#[test]
fn examine_keyword_room_feature() {
    let (engine, state) = fresh();
    let choices = engine.available_choices(&state);
    let index = choices
        .iter()
        .position(|choice| matches!(&choice.action, ChoiceAction::ExamineKeyword(k) if k == "sea"))
        .expect("sea keyword");
    assert_eq!(
        parse(&engine, &state, &choices, "look at the sea"),
        Parsed::Choose(index)
    );
    assert_eq!(
        parse(&engine, &state, &choices, "examine sea"),
        Parsed::Choose(index)
    );
}
