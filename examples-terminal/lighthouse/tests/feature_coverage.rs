//! Focused tests for every engine feature that is not already exercised by
//! the lighthouse narrative.
//!
//! Each test builds a minimal `World` that isolates a single variant of
//! `Condition`, `Effect`, `Trigger`, `Text`, `Value`, or `ChoiceAction` and
//! asserts that the engine interprets it correctly.

use nightshade::interactive_fiction::data::{
    Choice, ChoiceAction, Condition, DialogueId, Effect, Ending, EndingId, EventName, FlagKey,
    ItemId, ItemLocation, ItemProperties, NodeId, Npc, NpcId, Quest, QuestId, QuestStage,
    QuestTransition, Room, RoomId, Rule, RuleId, StatKey, Text, TextId, Timer, TimerId, Trigger,
    Value, World,
};
use nightshade::interactive_fiction::engine::Engine;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// world builder helpers
// ---------------------------------------------------------------------------

fn room(description: &str) -> Room {
    Room::new("Room", Text::lit(description))
}

fn start_world() -> World {
    let mut world = World::default();
    let start = RoomId::new("start");
    world.title = "Coverage".to_string();
    world.start_room = start.clone();
    world.rooms.insert(start, room("a small test room"));
    world
}

fn run(world: World) -> (Engine, nightshade::interactive_fiction::data::RuntimeState) {
    let engine = Engine::new(world).expect("validate");
    let state = engine.start_state();
    (engine, state)
}

fn started(world: World) -> (Engine, nightshade::interactive_fiction::data::RuntimeState) {
    let (engine, mut state) = run(world);
    engine.start(&mut state);
    (engine, state)
}

// ---------------------------------------------------------------------------
// Effect::ClearTranscript
// ---------------------------------------------------------------------------

#[test]
fn effect_clear_transcript_empties_the_log() {
    let mut world = start_world();
    world.rules.insert(
        RuleId::new("clear_then_speak"),
        Rule::on(
            Trigger::GameStart,
            vec![
                Effect::Say(Text::lit("first line that should vanish")),
                Effect::ClearTranscript,
                Effect::Say(Text::lit("only this remains")),
            ],
        )
        .once(),
    );
    let (_engine, state) = started(world);

    let narrations: Vec<&str> = state
        .transcript
        .iter()
        .filter_map(|entry| match entry {
            nightshade::interactive_fiction::data::TranscriptEntry::Narration(text) => {
                Some(text.as_str())
            }
            _ => None,
        })
        .collect();
    assert!(
        narrations
            .iter()
            .any(|line| line.contains("only this remains"))
    );
    assert!(!narrations.iter().any(|line| line.contains("first line")));
}

// ---------------------------------------------------------------------------
// Effect::FireRule
// ---------------------------------------------------------------------------

#[test]
fn effect_fire_rule_triggers_target() {
    let mut world = start_world();
    let target = RuleId::new("target_rule");
    world.rules.insert(
        RuleId::new("kick"),
        Rule::on(Trigger::GameStart, vec![Effect::FireRule(target.clone())]).once(),
    );
    world.rules.insert(
        target,
        // Trigger is a never-matching named event; the rule only fires via FireRule.
        Rule::on(
            Trigger::Named(EventName::new("__unreachable__")),
            vec![Effect::SetFlag(FlagKey::new("target_fired"), Value::TRUE)],
        ),
    );
    let (_engine, state) = started(world);
    assert_eq!(
        state.flags.get(&FlagKey::new("target_fired")),
        Some(&Value::TRUE)
    );
}

// ---------------------------------------------------------------------------
// Effect::ScheduleEvent + Trigger::Named
// ---------------------------------------------------------------------------

#[test]
fn scheduled_event_fires_after_delay() {
    let mut world = start_world();
    world.rules.insert(
        RuleId::new("kickoff"),
        Rule::on(
            Trigger::GameStart,
            vec![Effect::ScheduleEvent {
                event: EventName::new("bell"),
                in_turns: 2,
            }],
        )
        .once(),
    );
    world.rules.insert(
        RuleId::new("bell_listener"),
        Rule::on(
            Trigger::Named(EventName::new("bell")),
            vec![Effect::SetFlag(FlagKey::new("bell_rang"), Value::TRUE)],
        ),
    );
    let (engine, mut state) = started(world);

    // Advance turns by picking Wait until the event has fired.
    for _ in 0..5 {
        if state.flags.contains_key(&FlagKey::new("bell_rang")) {
            break;
        }
        let wait = engine
            .available_choices(&state)
            .iter()
            .position(|choice| {
                engine
                    .resolve_text(&state, &choice.label)
                    .eq_ignore_ascii_case("Wait")
            })
            .expect("Wait choice");
        engine.pick(&mut state, wait);
    }
    assert_eq!(
        state.flags.get(&FlagKey::new("bell_rang")),
        Some(&Value::TRUE)
    );
}

// ---------------------------------------------------------------------------
// Condition::DispositionAtLeast + Condition::RuleFired
// ---------------------------------------------------------------------------

#[test]
fn condition_disposition_and_rule_fired() {
    // `bump` and `gate` must fire in separate events: candidate sets are
    // collected before any effects run, so a gate whose condition depends on
    // the bump having already fired must listen for a downstream event.
    let mut world = start_world();
    let npc_id = NpcId::new("witness");
    world.npcs.insert(
        npc_id.clone(),
        Npc::new("Witness", Text::lit("a quiet figure"))
            .starting_in(RoomId::new("start"))
            .with_disposition(2),
    );
    let bump_id = RuleId::new("bump");
    world.rules.insert(
        bump_id.clone(),
        Rule::on(
            Trigger::GameStart,
            vec![
                Effect::AdjustDisposition(npc_id.clone(), 1),
                Effect::TriggerEvent(EventName::new("after_bump")),
            ],
        )
        .once(),
    );
    world.rules.insert(
        RuleId::new("gate"),
        Rule::on(
            Trigger::Named(EventName::new("after_bump")),
            vec![Effect::SetFlag(FlagKey::new("gate_ok"), Value::TRUE)],
        )
        .with_condition(Condition::All(vec![
            Condition::DispositionAtLeast(npc_id, 3),
            Condition::RuleFired(bump_id),
        ])),
    );
    let (_engine, state) = started(world);
    assert_eq!(
        state.flags.get(&FlagKey::new("gate_ok")),
        Some(&Value::TRUE)
    );
}

// ---------------------------------------------------------------------------
// Condition::ItemIsSomewhere
// ---------------------------------------------------------------------------

#[test]
fn condition_item_is_somewhere() {
    let mut world = start_world();
    let placed = ItemId::new("placed");
    let absent = ItemId::new("absent");
    world.items.insert(
        placed.clone(),
        nightshade::interactive_fiction::data::Item::new(
            "placed",
            Text::lit("p"),
            Text::lit("placed"),
        )
        .with_properties(ItemProperties {
            takeable: true,
            ..Default::default()
        }),
    );
    world.items.insert(
        absent.clone(),
        nightshade::interactive_fiction::data::Item::new(
            "absent",
            Text::lit("a"),
            Text::lit("absent"),
        )
        .with_properties(ItemProperties::default()),
    );
    world.rules.insert(
        RuleId::new("check"),
        Rule::on(
            Trigger::GameStart,
            vec![
                Effect::If {
                    when: Condition::ItemIsSomewhere(placed.clone()),
                    then: vec![Effect::SetFlag(FlagKey::new("placed_seen"), Value::TRUE)],
                    otherwise: vec![],
                },
                Effect::If {
                    when: Condition::ItemIsSomewhere(absent.clone()),
                    then: vec![Effect::SetFlag(FlagKey::new("absent_seen"), Value::TRUE)],
                    otherwise: vec![],
                },
            ],
        )
        .with_priority(-10)
        .once(),
    );

    let engine = Engine::new(world).expect("validate");
    let mut state = engine.start_state();
    state
        .item_locations
        .insert(placed, ItemLocation::Room(RoomId::new("start")));
    // `absent` stays at ItemLocation::Nowhere (default).
    engine.start(&mut state);

    assert_eq!(
        state.flags.get(&FlagKey::new("placed_seen")),
        Some(&Value::TRUE)
    );
    assert!(!state.flags.contains_key(&FlagKey::new("absent_seen")));
}

// ---------------------------------------------------------------------------
// Condition::QuestReached
// ---------------------------------------------------------------------------

#[test]
fn condition_quest_reached_tracks_history() {
    let quest_id = QuestId::new("q");
    let begin = NodeId::new("begin");
    let mid = NodeId::new("mid");
    let done = NodeId::new("done");

    let mut stages: BTreeMap<NodeId, QuestStage> = BTreeMap::new();
    stages.insert(
        begin.clone(),
        QuestStage::active(Text::lit("begin"))
            .with_transition(QuestTransition::new(mid.clone(), Condition::Always)),
    );
    stages.insert(
        mid.clone(),
        QuestStage::active(Text::lit("mid"))
            .with_transition(QuestTransition::new(done.clone(), Condition::Always)),
    );
    stages.insert(done.clone(), QuestStage::success(Text::lit("done")));

    let mut world = start_world();
    world.quests.insert(
        quest_id.clone(),
        Quest {
            name: "Q".to_string(),
            start: begin,
            stages,
        },
    );
    // Auto-quest-eval happens after GameStart/TurnStart event drain during
    // `engine.start()`. By checking on a later TurnEnd, we run after the
    // cascade has completed begin → mid → done.
    world.rules.insert(
        RuleId::new("after_quest_done"),
        Rule::on(
            Trigger::TurnEnd,
            vec![
                Effect::If {
                    when: Condition::QuestReached(quest_id.clone(), mid),
                    then: vec![Effect::SetFlag(FlagKey::new("reached_mid"), Value::TRUE)],
                    otherwise: vec![],
                },
                Effect::If {
                    when: Condition::QuestReached(quest_id, done),
                    then: vec![Effect::SetFlag(FlagKey::new("reached_done"), Value::TRUE)],
                    otherwise: vec![],
                },
            ],
        )
        .once(),
    );

    let (engine, mut state) = started(world);
    let wait = engine
        .available_choices(&state)
        .iter()
        .position(|choice| {
            engine
                .resolve_text(&state, &choice.label)
                .eq_ignore_ascii_case("Wait")
        })
        .expect("Wait");
    engine.pick(&mut state, wait);
    assert_eq!(
        state.flags.get(&FlagKey::new("reached_mid")),
        Some(&Value::TRUE)
    );
    assert_eq!(
        state.flags.get(&FlagKey::new("reached_done")),
        Some(&Value::TRUE)
    );
}

// ---------------------------------------------------------------------------
// Trigger::OnTalk + Trigger::OnExit
// ---------------------------------------------------------------------------

#[test]
fn trigger_on_talk_and_on_exit_fire() {
    let mut world = start_world();
    let other = RoomId::new("other");
    world.rooms.insert(other.clone(), room("the other room"));
    if let Some(start_room) = world.rooms.get_mut(&RoomId::new("start")) {
        start_room
            .exits
            .push(nightshade::interactive_fiction::data::Exit::new(
                "go",
                other.clone(),
            ));
    }

    let npc_id = NpcId::new("guard");
    let dialogue_id = DialogueId::new("guard_talk");
    let start_node = NodeId::new("s");
    let mut dialogue_nodes = BTreeMap::new();
    dialogue_nodes.insert(
        start_node.clone(),
        nightshade::interactive_fiction::data::DialogueNode::new(Text::lit("hello.")).with_option(
            nightshade::interactive_fiction::data::DialogueOption::new(Text::lit("(end)")),
        ),
    );
    world.dialogues.insert(
        dialogue_id.clone(),
        nightshade::interactive_fiction::data::Dialogue {
            start: start_node,
            nodes: dialogue_nodes,
        },
    );
    world.npcs.insert(
        npc_id.clone(),
        Npc::new("Guard", Text::lit("a stern figure"))
            .with_dialogue(dialogue_id)
            .starting_in(RoomId::new("start")),
    );

    world.rules.insert(
        RuleId::new("saw_talk"),
        Rule::on(
            Trigger::OnTalk(Some(npc_id)),
            vec![Effect::SetFlag(FlagKey::new("talked"), Value::TRUE)],
        ),
    );
    world.rules.insert(
        RuleId::new("saw_exit"),
        Rule::on(
            Trigger::OnExit(Some(RoomId::new("start"))),
            vec![Effect::SetFlag(FlagKey::new("exited_start"), Value::TRUE)],
        ),
    );

    let (engine, mut state) = started(world);
    let talk = engine
        .available_choices(&state)
        .iter()
        .position(|choice| {
            engine
                .resolve_text(&state, &choice.label)
                .to_lowercase()
                .contains("talk to")
        })
        .expect("talk");
    engine.pick(&mut state, talk);
    assert_eq!(state.flags.get(&FlagKey::new("talked")), Some(&Value::TRUE));

    // Pick the "(end)" dialogue option to close the conversation (it has no goto).
    let end_option = engine
        .available_choices(&state)
        .iter()
        .position(|choice| matches!(choice.action, ChoiceAction::DialogueOption(_)))
        .expect("dialogue end option");
    engine.pick(&mut state, end_option);

    let go = engine
        .available_choices(&state)
        .iter()
        .position(|choice| matches!(choice.action, ChoiceAction::Go { .. }))
        .expect("go");
    engine.pick(&mut state, go);
    assert_eq!(
        state.flags.get(&FlagKey::new("exited_start")),
        Some(&Value::TRUE)
    );
}

// ---------------------------------------------------------------------------
// Effect::OfferChoices + ChoiceAction::Effects
// ---------------------------------------------------------------------------

#[test]
fn offer_choices_with_effects_action() {
    let mut world = start_world();
    world.rules.insert(
        RuleId::new("offer"),
        Rule::on(
            Trigger::GameStart,
            vec![Effect::OfferChoices(vec![
                Choice::new(
                    Text::lit("pick alpha"),
                    ChoiceAction::Effects(vec![Effect::SetFlag(
                        FlagKey::new("picked"),
                        Value::Text("alpha".to_string()),
                    )]),
                ),
                Choice::new(
                    Text::lit("pick beta"),
                    ChoiceAction::Effects(vec![Effect::SetFlag(
                        FlagKey::new("picked"),
                        Value::Text("beta".to_string()),
                    )]),
                ),
            ])],
        )
        .once(),
    );

    let (engine, mut state) = started(world);
    let menu = engine.available_choices(&state);
    // The pending offer should be the only presented options.
    assert_eq!(menu.len(), 2);
    engine.pick(&mut state, 1);
    assert_eq!(
        state.flags.get(&FlagKey::new("picked")),
        Some(&Value::Text("beta".to_string()))
    );
    assert!(state.pending_choices.is_empty());
}

// ---------------------------------------------------------------------------
// Text::Flag, Text::Stat, Text::Ref, Value::Int / Value::Text
// ---------------------------------------------------------------------------

#[test]
fn text_interpolates_flag_stat_and_ref() {
    let mut world = start_world();
    let stat = StatKey::new("count");
    let flag = FlagKey::new("label");
    let shared_text = TextId::new("shared");
    world
        .texts
        .insert(shared_text.clone(), Text::lit("shared value"));
    world.rules.insert(
        RuleId::new("setters"),
        Rule::on(
            Trigger::GameStart,
            vec![
                Effect::SetStat(stat.clone(), 42),
                Effect::SetFlag(flag.clone(), Value::Text("hello".to_string())),
                Effect::SetFlag(FlagKey::new("n"), Value::Int(7)),
            ],
        )
        .once(),
    );
    let (engine, state) = started(world);

    let text = Text::Sequence(vec![
        Text::lit("count="),
        Text::Stat(stat),
        Text::lit(" label="),
        Text::Flag(flag),
        Text::lit(" n="),
        Text::Flag(FlagKey::new("n")),
        Text::lit(" shared="),
        Text::Ref(shared_text),
    ]);
    let rendered = engine.resolve_text(&state, &text);
    assert_eq!(rendered, "count=42 label=hello n=7 shared=shared value");
}

// ---------------------------------------------------------------------------
// Condition::Chance (mutable path)
// ---------------------------------------------------------------------------

#[test]
fn condition_chance_randomizes_in_mutable_context() {
    // Chance(0) never fires; Chance(100) always fires — even with the
    // deterministic default RNG.
    let mut world = start_world();
    world.rules.insert(
        RuleId::new("zero"),
        Rule::on(
            Trigger::GameStart,
            vec![Effect::If {
                when: Condition::Chance(0),
                then: vec![Effect::SetFlag(FlagKey::new("zero_fired"), Value::TRUE)],
                otherwise: vec![],
            }],
        )
        .once(),
    );
    world.rules.insert(
        RuleId::new("hundred"),
        Rule::on(
            Trigger::GameStart,
            vec![Effect::If {
                when: Condition::Chance(100),
                then: vec![Effect::SetFlag(FlagKey::new("hundred_fired"), Value::TRUE)],
                otherwise: vec![],
            }],
        )
        .once(),
    );
    let (_engine, state) = started(world);
    assert!(!state.flags.contains_key(&FlagKey::new("zero_fired")));
    assert_eq!(
        state.flags.get(&FlagKey::new("hundred_fired")),
        Some(&Value::TRUE)
    );
}

// ---------------------------------------------------------------------------
// Timer.cancel_on + Condition::TimerRunning + Condition::TimerExpired
// ---------------------------------------------------------------------------

#[test]
fn timer_cancel_on_prevents_expiration() {
    let mut world = start_world();
    let timer_id = TimerId::new("clock");
    world.timers.insert(
        timer_id.clone(),
        Timer::new(2)
            .with_on_expire(vec![Effect::SetFlag(FlagKey::new("expired"), Value::TRUE)])
            .cancel_on(Condition::FlagSet(FlagKey::new("stop"))),
    );
    world.rules.insert(
        RuleId::new("kickoff"),
        Rule::on(
            Trigger::GameStart,
            vec![Effect::StartTimer(timer_id.clone())],
        )
        .once(),
    );
    world.rules.insert(
        RuleId::new("stop_soon"),
        Rule::on(
            Trigger::TurnStart,
            vec![Effect::SetFlag(FlagKey::new("stop"), Value::TRUE)],
        )
        .once(),
    );

    let (engine, mut state) = started(world);
    // Advance turns; the timer is cancelled instead of expiring.
    for _ in 0..4 {
        let wait = engine
            .available_choices(&state)
            .iter()
            .position(|choice| {
                engine
                    .resolve_text(&state, &choice.label)
                    .eq_ignore_ascii_case("Wait")
            })
            .expect("Wait");
        engine.pick(&mut state, wait);
    }
    assert!(!state.flags.contains_key(&FlagKey::new("expired")));
    assert!(state.timers_cancelled.contains(&timer_id));
}

// ---------------------------------------------------------------------------
// Validator rejects typos that used to slip through
// ---------------------------------------------------------------------------

#[test]
fn validator_catches_bad_exit_condition() {
    let mut world = start_world();
    let start_id = RoomId::new("start");
    let other_id = RoomId::new("other");
    world.rooms.insert(other_id.clone(), room("other"));
    if let Some(start) = world.rooms.get_mut(&start_id) {
        start.exits.push(
            nightshade::interactive_fiction::data::Exit::new("go", other_id).gated(
                // Condition::Ref points at a condition ID that does not exist.
                Condition::Ref(nightshade::interactive_fiction::data::ConditionId::new(
                    "missing_condition",
                )),
                Text::lit("locked"),
            ),
        );
    }
    let errors = Engine::new(world).err().expect("should fail");
    assert!(
        !errors.is_empty(),
        "validator should flag the dangling Condition::Ref"
    );
}

#[test]
fn validator_catches_bad_ending_reference() {
    let mut world = start_world();
    world.endings.insert(
        EndingId::new("real"),
        Ending::new("Real", Text::lit("d"), Text::lit("e"), Condition::Always),
    );
    world.rules.insert(
        RuleId::new("trigger_bogus_ending"),
        Rule::on(
            Trigger::GameStart,
            vec![Effect::TriggerEnding(EndingId::new("bogus_id"))],
        ),
    );
    let errors = Engine::new(world).err().expect("should fail");
    assert!(
        errors.iter().any(|error| matches!(
            error,
            nightshade::interactive_fiction::engine::ValidationError::EndingRefMissing(id) if id.as_str() == "bogus_id"
        )),
        "validator should flag missing ending reference, got: {errors:?}"
    );
}
