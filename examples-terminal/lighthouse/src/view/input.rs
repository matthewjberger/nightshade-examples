//! Free-form input parsing.
//!
//! Maps typed text like "take key" or "n" or "look at the drip" onto the
//! index of the matching entry in the engine's current [`Choice`] menu.
//! Falls through to a digit shortcut when the player just wants the numbered
//! form. Matching is choice-list-driven: we only consider verbs/nouns the
//! engine is currently offering, so disambiguation stays tight.

use crate::data::{Choice, ChoiceAction, RuntimeState};
use crate::engine::Engine;

/// The result of parsing one line of input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// Pick the choice at this menu index.
    Choose(usize),
    /// Meta command: quit the game.
    Quit,
    /// Meta command: undo the last turn.
    Undo,
    /// Meta command: show the list of currently available actions.
    Help,
    /// Input was empty (whitespace-only).
    Empty,
    /// Nothing in the current menu matched.
    NoMatch,
    /// More than one choice matched; player must be more specific.
    Ambiguous,
}

pub fn parse(engine: &Engine, state: &RuntimeState, choices: &[Choice], raw: &str) -> Parsed {
    let normalized = raw.trim().to_lowercase();
    if normalized.is_empty() {
        return Parsed::Empty;
    }

    // Meta commands. These aren't choices in the menu.
    match normalized.as_str() {
        "q" | "quit" | "exit" => return Parsed::Quit,
        "u" | "undo" => return Parsed::Undo,
        "help" | "h" | "?" => return Parsed::Help,
        _ => {}
    }

    // Pure-number shortcut.
    if let Ok(number) = normalized.parse::<usize>()
        && number >= 1
        && number <= choices.len()
    {
        return Parsed::Choose(number - 1);
    }

    // Direction-only shortcut: "n" / "north" / "up" / "d" / etc.
    if let Some(matched) = direction_only(engine, state, choices, &normalized) {
        return matched;
    }

    let (verb, rest) = split_verb_noun(&normalized);
    if let Some(parsed) = dispatch_verb(engine, state, choices, verb, rest) {
        return parsed;
    }

    // Unrecognized verb: try the input as a label substring against dialogue
    // options or any custom OfferChoices entries.
    label_match(engine, state, choices, &normalized)
}

fn dispatch_verb(
    engine: &Engine,
    state: &RuntimeState,
    choices: &[Choice],
    verb: &str,
    rest: &str,
) -> Option<Parsed> {
    let result = match verb {
        "look" | "l" if rest.is_empty() => {
            find_simple(choices, |action| matches!(action, ChoiceAction::Look))
        }
        "look" => {
            let noun = rest.trim_start_matches("at ").trim();
            find_examine(engine, state, choices, noun)
        }
        "inv" | "i" | "inventory" if rest.is_empty() => {
            find_simple(choices, |action| matches!(action, ChoiceAction::Inventory))
        }
        "wait" | "z" if rest.is_empty() => {
            find_simple(choices, |action| matches!(action, ChoiceAction::Wait))
        }
        "leave" | "bye" | "goodbye" if rest.is_empty() => find_simple(choices, |action| {
            matches!(action, ChoiceAction::LeaveDialogue)
        }),
        "go" => find_go(engine, state, choices, rest),
        "take" | "get" | "grab" => item_verb(engine, choices, rest, |action| {
            matches!(action, ChoiceAction::Take(_))
        }),
        "drop" => item_verb(engine, choices, rest, |action| {
            matches!(action, ChoiceAction::Drop(_))
        }),
        "use" => item_verb(engine, choices, rest, |action| {
            matches!(action, ChoiceAction::Use(_))
        }),
        "read" => item_verb(engine, choices, rest, |action| {
            matches!(action, ChoiceAction::Read(_))
        }),
        "examine" | "x" | "inspect" => find_examine(engine, state, choices, rest),
        "talk" | "speak" => {
            let noun = rest.trim_start_matches("to ").trim();
            npc_verb(engine, choices, noun, |action| {
                matches!(action, ChoiceAction::TalkTo(_))
            })
        }
        _ => return None,
    };
    Some(result)
}

fn item_verb(
    engine: &Engine,
    choices: &[Choice],
    rest: &str,
    action_matches: impl Fn(&ChoiceAction) -> bool,
) -> Parsed {
    if rest.is_empty() {
        return Parsed::NoMatch;
    }
    let matches = indexes_of_item_action(engine, choices, rest, action_matches);
    match matches.len() {
        0 => Parsed::NoMatch,
        1 => Parsed::Choose(matches[0]),
        _ => Parsed::Ambiguous,
    }
}

fn npc_verb(
    engine: &Engine,
    choices: &[Choice],
    noun: &str,
    action_matches: impl Fn(&ChoiceAction) -> bool,
) -> Parsed {
    let matches = indexes_of_npc_action(engine, choices, noun, action_matches);
    match matches.len() {
        0 => Parsed::NoMatch,
        1 => Parsed::Choose(matches[0]),
        _ => Parsed::Ambiguous,
    }
}

fn split_verb_noun(input: &str) -> (&str, &str) {
    match input.find(char::is_whitespace) {
        Some(idx) => (&input[..idx], input[idx..].trim_start()),
        None => (input, ""),
    }
}

fn find_simple(choices: &[Choice], predicate: impl Fn(&ChoiceAction) -> bool) -> Parsed {
    let matches: Vec<usize> = choices
        .iter()
        .enumerate()
        .filter(|(_, choice)| predicate(&choice.action))
        .map(|(i, _)| i)
        .collect();
    match matches.len() {
        0 => Parsed::NoMatch,
        1 => Parsed::Choose(matches[0]),
        _ => Parsed::Ambiguous,
    }
}

fn direction_only(
    engine: &Engine,
    state: &RuntimeState,
    choices: &[Choice],
    input: &str,
) -> Option<Parsed> {
    // Only intercept if the input is a pure direction word with no verb.
    let expanded = expand_direction(input)?;
    let matches = go_indexes_for_direction(engine, state, choices, expanded);
    Some(match matches.len() {
        0 => return None,
        1 => Parsed::Choose(matches[0]),
        _ => Parsed::Ambiguous,
    })
}

fn find_go(engine: &Engine, state: &RuntimeState, choices: &[Choice], rest: &str) -> Parsed {
    if rest.is_empty() {
        return Parsed::NoMatch;
    }
    let wanted = expand_direction(rest).unwrap_or(rest);
    let matches = go_indexes_for_direction(engine, state, choices, wanted);
    match matches.len() {
        0 => Parsed::NoMatch,
        1 => Parsed::Choose(matches[0]),
        _ => Parsed::Ambiguous,
    }
}

fn go_indexes_for_direction(
    engine: &Engine,
    state: &RuntimeState,
    choices: &[Choice],
    wanted: &str,
) -> Vec<usize> {
    let Some(room) = engine.world().rooms.get(&state.current_room) else {
        return Vec::new();
    };
    choices
        .iter()
        .enumerate()
        .filter_map(|(i, choice)| {
            let ChoiceAction::Go { exit_index, .. } = choice.action else {
                return None;
            };
            let exit = room.exits.get(exit_index)?;
            let direction = exit.direction.to_lowercase();
            if matches_direction(&direction, wanted) {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

fn matches_direction(direction: &str, wanted: &str) -> bool {
    if direction == wanted {
        return true;
    }
    if direction.starts_with(wanted) {
        return true;
    }
    // The exit direction is often "north (to the cottage)"; match if the
    // wanted word appears as a whole word inside the direction phrase, or as
    // the first word's prefix.
    let first_word = direction.split_whitespace().next().unwrap_or("");
    if first_word == wanted || first_word.starts_with(wanted) {
        return true;
    }
    direction
        .split_whitespace()
        .any(|word| word.trim_matches(|character: char| !character.is_alphanumeric()) == wanted)
}

fn expand_direction(input: &str) -> Option<&'static str> {
    let trimmed = input.trim();
    Some(match trimmed {
        "n" | "north" => "north",
        "s" | "south" => "south",
        "e" | "east" => "east",
        "w" | "west" => "west",
        "u" | "up" => "up",
        "d" | "down" => "down",
        "ne" | "northeast" => "northeast",
        "nw" | "northwest" => "northwest",
        "se" | "southeast" => "southeast",
        "sw" | "southwest" => "southwest",
        _ => return None,
    })
}

fn find_examine(engine: &Engine, _state: &RuntimeState, choices: &[Choice], noun: &str) -> Parsed {
    if noun.is_empty() {
        return Parsed::NoMatch;
    }
    let noun = noun
        .trim_start_matches("the ")
        .trim_start_matches("a ")
        .trim();
    let mut matches: Vec<usize> = Vec::new();
    for (index, choice) in choices.iter().enumerate() {
        match &choice.action {
            ChoiceAction::Examine(item) if item_matches_noun(engine, item, noun) => {
                matches.push(index);
            }
            ChoiceAction::ExamineKeyword(keyword)
                if keyword.eq_ignore_ascii_case(noun) || noun_matches_phrase(keyword, noun) =>
            {
                matches.push(index);
            }
            _ => {}
        }
    }
    match matches.len() {
        0 => Parsed::NoMatch,
        1 => Parsed::Choose(matches[0]),
        _ => Parsed::Ambiguous,
    }
}

fn indexes_of_item_action(
    engine: &Engine,
    choices: &[Choice],
    rest: &str,
    action_matches: impl Fn(&ChoiceAction) -> bool,
) -> Vec<usize> {
    let noun = rest
        .trim_start_matches("the ")
        .trim_start_matches("a ")
        .trim();
    if noun.is_empty() {
        return Vec::new();
    }
    choices
        .iter()
        .enumerate()
        .filter_map(|(i, choice)| {
            if !action_matches(&choice.action) {
                return None;
            }
            let item_id = match &choice.action {
                ChoiceAction::Take(id)
                | ChoiceAction::Drop(id)
                | ChoiceAction::Use(id)
                | ChoiceAction::Read(id)
                | ChoiceAction::Examine(id) => id,
                _ => return None,
            };
            if item_matches_noun(engine, item_id, noun) {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

fn indexes_of_npc_action(
    engine: &Engine,
    choices: &[Choice],
    noun: &str,
    action_matches: impl Fn(&ChoiceAction) -> bool,
) -> Vec<usize> {
    let noun = noun
        .trim_start_matches("the ")
        .trim_start_matches("a ")
        .trim();
    choices
        .iter()
        .enumerate()
        .filter_map(|(i, choice)| {
            if !action_matches(&choice.action) {
                return None;
            }
            let npc_id = match &choice.action {
                ChoiceAction::TalkTo(id) => id,
                _ => return None,
            };
            let npc = engine.world().npcs.get(npc_id)?;
            if noun.is_empty() {
                // Bare "talk" matches iff there's only one NPC to talk to.
                return Some(i);
            }
            if noun_matches_phrase(&npc.name, noun)
                || npc
                    .synonyms
                    .iter()
                    .any(|synonym| noun_matches_phrase(synonym, noun))
            {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

fn item_matches_noun(engine: &Engine, item_id: &crate::data::ItemId, noun: &str) -> bool {
    let Some(item) = engine.world().items.get(item_id) else {
        return false;
    };
    if noun_matches_phrase(&item.name, noun) {
        return true;
    }
    item.synonyms
        .iter()
        .any(|synonym| noun_matches_phrase(synonym, noun))
}

/// Case-insensitive check: does the player's `noun` identify the given phrase?
/// True when the phrase equals the noun, ends with the noun as a whole word,
/// or contains the noun as a whole word (so "key" matches "iron key").
fn noun_matches_phrase(phrase: &str, noun: &str) -> bool {
    let phrase = phrase.to_lowercase();
    let noun = noun.to_lowercase();
    if phrase == noun {
        return true;
    }
    phrase.split_whitespace().any(|word| {
        let word = word.trim_matches(|character: char| !character.is_alphanumeric());
        word == noun
    })
}

fn label_match(engine: &Engine, state: &RuntimeState, choices: &[Choice], input: &str) -> Parsed {
    let mut matches: Vec<usize> = Vec::new();
    for (index, choice) in choices.iter().enumerate() {
        let label = engine.resolve_text(state, &choice.label).to_lowercase();
        if label.contains(input) {
            matches.push(index);
        }
    }
    match matches.len() {
        0 => Parsed::NoMatch,
        1 => Parsed::Choose(matches[0]),
        _ => Parsed::Ambiguous,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Choice, ChoiceAction, ItemId, ItemLocation};
    use crate::game;

    fn fresh() -> (Engine, crate::data::RuntimeState) {
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
                ChoiceAction::Go { ref to, .. } if *to == game::ids::room_cliff_path() => {
                    Some(index)
                }
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
            .position(
                |choice| matches!(&choice.action, ChoiceAction::ExamineKeyword(k) if k == "sea"),
            )
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
}
