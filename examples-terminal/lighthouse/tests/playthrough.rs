use lighthouse::game;
use nightshade::interactive_fiction::data::{Choice, ChoiceAction, ItemLocation, RuntimeState};
use nightshade::interactive_fiction::engine::Engine;

fn build_engine() -> Engine {
    let world = game::build_world();
    Engine::new(world).expect("world should validate")
}

fn fresh_state(engine: &Engine) -> RuntimeState {
    let mut state = engine.start_state();
    engine.start(&mut state);
    state
}

/// Return the first choice whose action matches the predicate, along with its index.
fn pick_by<F>(engine: &Engine, state: &RuntimeState, mut predicate: F) -> usize
where
    F: FnMut(&Choice) -> bool,
{
    engine
        .available_choices(state)
        .into_iter()
        .enumerate()
        .find_map(|(index, choice)| {
            if predicate(&choice) {
                Some(index)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            let mut labels = Vec::new();
            for (index, choice) in engine.available_choices(state).into_iter().enumerate() {
                labels.push(format!(
                    "{index}: {}",
                    engine.resolve_text(state, &choice.label)
                ));
            }
            panic!("no matching choice; menu was:\n{}", labels.join("\n"))
        })
}

fn pick_labeled(engine: &Engine, state: &RuntimeState, substring: &str) -> usize {
    pick_by(engine, state, |choice| {
        let rendered = engine.resolve_text(state, &choice.label);
        rendered.to_lowercase().contains(&substring.to_lowercase())
    })
}

#[test]
fn validates_world() {
    // Smoke test: building the world and the engine runs validation.
    let _engine = build_engine();
}

#[test]
fn good_ending_via_lighting_lantern() {
    let engine = build_engine();
    let mut state = fresh_state(&engine);

    // Pick up the cottage key from the shore.
    let take_key = pick_labeled(&engine, &state, "Take the iron key");
    engine.pick(&mut state, take_key);

    // The cottage door is locked; use the key at the shore.
    let use_key = pick_labeled(&engine, &state, "Use the iron key");
    engine.pick(&mut state, use_key);

    // Now the north exit is passable.
    let go_north = pick_labeled(&engine, &state, "Go north");
    engine.pick(&mut state, go_north);

    assert_eq!(state.current_room, lighthouse::game::ids::room_cottage());
    // The quest should have auto-advanced to stage_unlocked_cottage.
    assert_eq!(
        state
            .quest_stages
            .get(&lighthouse::game::ids::quest_lantern()),
        Some(&lighthouse::game::ids::stage_unlocked_cottage())
    );

    // Pick up the lantern and tinderbox.
    let take_lantern = pick_labeled(&engine, &state, "Take the storm lantern");
    engine.pick(&mut state, take_lantern);
    let take_tinder = pick_labeled(&engine, &state, "Take the tinderbox");
    engine.pick(&mut state, take_tinder);

    // Light the lantern.
    let use_tinder = pick_labeled(&engine, &state, "Use the tinderbox");
    engine.pick(&mut state, use_tinder);
    assert!(matches!(
        state
            .flags
            .get(&nightshade::interactive_fiction::data::FlagKey::new(
                "lantern_is_lit"
            )),
        Some(nightshade::interactive_fiction::data::Value::Bool(true))
    ));

    // Pick up the oil can.
    let take_oil = pick_labeled(&engine, &state, "Take the oil can");
    engine.pick(&mut state, take_oil);

    // Descend into the cellar; with the lit lantern, the keeper is found.
    let go_down = pick_labeled(&engine, &state, "Go down");
    engine.pick(&mut state, go_down);
    assert_eq!(state.current_room, lighthouse::game::ids::room_cellar());
    assert!(
        state
            .flags
            .contains_key(&lighthouse::game::ids::flag_found_keeper())
    );

    // Back up.
    let go_up = pick_labeled(&engine, &state, "Go up");
    engine.pick(&mut state, go_up);

    // Into the tower.
    let enter_tower = pick_by(
        &engine,
        &state,
        |choice| matches!(choice.action, ChoiceAction::Go { ref to, .. } if *to == lighthouse::game::ids::room_tower_base()),
    );
    engine.pick(&mut state, enter_tower);

    // Pick up the tower key.
    let take_tower_key = pick_labeled(&engine, &state, "Take the tower key");
    engine.pick(&mut state, take_tower_key);

    // Unlock the tower.
    let use_tower_key = pick_labeled(&engine, &state, "Use the tower key");
    engine.pick(&mut state, use_tower_key);

    // Climb up to the stairwell.
    let go_up_stairs = pick_by(
        &engine,
        &state,
        |choice| matches!(choice.action, ChoiceAction::Go { ref to, .. } if *to == lighthouse::game::ids::room_tower_stairs()),
    );
    engine.pick(&mut state, go_up_stairs);

    // Up to the lantern room.
    let go_up_lens = pick_by(
        &engine,
        &state,
        |choice| matches!(choice.action, ChoiceAction::Go { ref to, .. } if *to == lighthouse::game::ids::room_lantern_room()),
    );
    engine.pick(&mut state, go_up_lens);

    // Oil the mechanism.
    let use_oil = pick_labeled(&engine, &state, "Use the oil can");
    engine.pick(&mut state, use_oil);

    // Light the great lantern.
    let use_tinder_up_top = pick_labeled(&engine, &state, "Use the tinderbox");
    engine.pick(&mut state, use_tinder_up_top);

    assert_eq!(
        state.game_over.as_ref(),
        Some(&lighthouse::game::ids::ending_the_lantern_burns()),
        "should have reached the good ending"
    );
    assert!(
        state
            .unlocked_endings
            .contains(&lighthouse::game::ids::ending_the_lantern_burns())
    );
    // The quest should have auto-advanced through all active stages to stage_restored.
    assert_eq!(
        state
            .quest_stages
            .get(&lighthouse::game::ids::quest_lantern()),
        Some(&lighthouse::game::ids::stage_restored())
    );
}

#[test]
fn safe_ashore_ending_by_fleeing() {
    let engine = build_engine();
    let mut state = fresh_state(&engine);

    // From the shore, go east up the cliff path...
    let go_east = pick_labeled(&engine, &state, "Go east");
    engine.pick(&mut state, go_east);

    // ...then west, away from the headland.
    let go_west = pick_labeled(&engine, &state, "Go west");
    engine.pick(&mut state, go_west);

    assert_eq!(
        state.game_over.as_ref(),
        Some(&lighthouse::game::ids::ending_safe_ashore())
    );
}

#[test]
fn storm_timer_expires_without_action() {
    let engine = build_engine();
    let mut state = fresh_state(&engine);

    // Burn turns by repeatedly picking "Wait" until the storm timer expires.
    // The storm timer starts at 18 turns; 24 waits is a comfortable upper bound.
    for _ in 0..24 {
        if state.game_over.is_some() {
            break;
        }
        let wait = pick_labeled(&engine, &state, "Wait");
        engine.pick(&mut state, wait);
    }

    assert_eq!(
        state.game_over.as_ref(),
        Some(&lighthouse::game::ids::ending_lost_to_the_storm()),
        "storm timer expiration should fire the failure ending"
    );
}

#[test]
fn dialogue_confront_stranger_with_note() {
    let engine = build_engine();
    let mut state = fresh_state(&engine);

    // Unlock the cottage so we can descend and find the wrecker's note.
    let take_key = pick_labeled(&engine, &state, "Take the iron key");
    engine.pick(&mut state, take_key);
    let use_key = pick_labeled(&engine, &state, "Use the iron key");
    engine.pick(&mut state, use_key);
    let go_north = pick_labeled(&engine, &state, "Go north");
    engine.pick(&mut state, go_north);

    // Light the lantern so the cellar is not dark.
    let take_lantern = pick_labeled(&engine, &state, "Take the storm lantern");
    engine.pick(&mut state, take_lantern);
    let take_tinder = pick_labeled(&engine, &state, "Take the tinderbox");
    engine.pick(&mut state, take_tinder);
    let light = pick_labeled(&engine, &state, "Use the tinderbox");
    engine.pick(&mut state, light);

    // Descend and pick up the wrecker's note.
    let go_down = pick_labeled(&engine, &state, "Go down");
    engine.pick(&mut state, go_down);
    let take_note = pick_labeled(&engine, &state, "Take the folded note");
    engine.pick(&mut state, take_note);

    // Climb back out and wait until the stranger arrives at the shore.
    let go_up = pick_labeled(&engine, &state, "Go up");
    engine.pick(&mut state, go_up);
    let go_south = pick_labeled(&engine, &state, "Go south");
    engine.pick(&mut state, go_south);
    for _ in 0..6 {
        if state.game_over.is_some() {
            break;
        }
        if engine.available_choices(&state).iter().any(|choice| {
            engine
                .resolve_text(&state, &choice.label)
                .to_lowercase()
                .contains("talk to the stranger")
        }) {
            break;
        }
        let wait = pick_labeled(&engine, &state, "Wait");
        engine.pick(&mut state, wait);
    }

    let talk = pick_labeled(&engine, &state, "Talk to the stranger");
    engine.pick(&mut state, talk);

    // The "I know what you are" branch is visible because we hold the note.
    let confront = pick_labeled(&engine, &state, "I know what you are");
    engine.pick(&mut state, confront);

    // Disposition should have dropped, the "refused" flag should be set, and
    // the dialogue should have ended.
    assert!(
        matches!(
            state
                .flags
                .get(&lighthouse::game::ids::flag_offer_refused()),
            Some(nightshade::interactive_fiction::data::Value::Bool(true))
        ),
        "refusal flag should be set after confronting the stranger"
    );
    assert!(
        state
            .dispositions
            .get(&lighthouse::game::ids::npc_stranger())
            .copied()
            .unwrap_or(0)
            <= -5,
        "stranger's disposition should plunge after being confronted"
    );
}

#[test]
fn item_location_single_sourced() {
    let engine = build_engine();
    let mut state = fresh_state(&engine);
    let take_key = pick_labeled(&engine, &state, "Take the iron key");
    engine.pick(&mut state, take_key);

    assert_eq!(
        state
            .item_locations
            .get(&lighthouse::game::ids::item_cottage_key()),
        Some(&ItemLocation::Inventory)
    );
}
