//! High-level turn loop.
//!
//! Wires together event dispatch, effect execution, scheduled events, timers,
//! and ending evaluation. Called from `Engine::start` and `Engine::pick`.

use crate::data::{
    Choice, ChoiceAction, Effect, EndingId, Item, ItemId, ItemLocation, NpcId, Placeholder,
    QuestId, QuestStageKind, RoomId, RuntimeState, TimerId, TranscriptEntry, VerbResponses,
};
use crate::engine::{Engine, choices, dispatch, eval, exec, resolve};

/// Run the opening of the game. Fires `GameStart`, prints intro, describes
/// the starting room.
pub fn start(engine: &Engine, state: &mut RuntimeState) {
    state.visited.insert(state.current_room.clone());

    let intro = engine.world().intro.clone();
    let rendered = resolve::resolve_mut(engine, state, &intro);
    if !rendered.is_empty() {
        state.push_transcript(TranscriptEntry::Narration(rendered));
    }

    let mut ctx = exec::Context::new();
    ctx.queue(dispatch::Event::GameStart);
    ctx.drain(engine, state);

    exec::describe_current_room(engine, state);
    // After setup the first TurnStart can still fire useful rules (weather,
    // ambient text) before the player picks anything.
    maybe_fire(
        &mut exec::Context::new(),
        engine,
        state,
        dispatch::Event::TurnStart,
    );
    advance_quests(engine, state);
    evaluate_endings(engine, state);
}

/// Apply the player's picked choice and advance the turn.
pub fn pick(engine: &Engine, state: &mut RuntimeState, index: usize) {
    if state.game_over.is_some() {
        return;
    }

    let menu = choices::assemble(engine, state);
    let Some(choice) = menu.into_iter().nth(index) else {
        let message = engine.world().verb_responses.option_unavailable.clone();
        state.push_transcript(TranscriptEntry::System(message));
        return;
    };
    if !choices::is_actionable(engine, state, &choice) {
        if let Some(reason) = &choice.locked_reason {
            let rendered = resolve::resolve_mut(engine, state, reason);
            state.push_transcript(TranscriptEntry::System(rendered));
        } else {
            let message = engine.world().verb_responses.action_forbidden.clone();
            state.push_transcript(TranscriptEntry::System(message));
        }
        return;
    }

    echo_choice(engine, state, &choice);

    let takes_turn = action_advances_turn(&choice.action);

    // Clear any pending-choice override now that the player has picked.
    if !state.pending_choices.is_empty() {
        state.pending_choices.clear();
    }

    let mut ctx = exec::Context::new();
    apply_choice(&mut ctx, engine, state, choice);
    ctx.drain(engine, state);

    if state.game_over.is_some() {
        return;
    }

    if takes_turn {
        state.turn += 1;
        let mut turn_ctx = exec::Context::new();
        turn_ctx.queue(dispatch::Event::TurnStart);
        turn_ctx.drain(engine, state);
        if state.game_over.is_some() {
            return;
        }
        tick_timers(engine, state);
        if state.game_over.is_some() {
            return;
        }
        run_due_scheduled_events(engine, state);
        if state.game_over.is_some() {
            return;
        }
        let mut end_ctx = exec::Context::new();
        end_ctx.queue(dispatch::Event::TurnEnd);
        end_ctx.drain(engine, state);
        if state.game_over.is_some() {
            return;
        }
    }

    advance_quests(engine, state);
    if state.game_over.is_some() {
        return;
    }
    evaluate_endings(engine, state);
}

/// Walk each quest currently on an active stage. If any outgoing transition's
/// condition holds, run its effects and advance the stage. Cascades are
/// supported (advancing one quest may satisfy another's condition) with a
/// bounded number of iterations.
fn advance_quests(engine: &Engine, state: &mut RuntimeState) {
    const MAX_CASCADE: usize = 32;
    for _ in 0..MAX_CASCADE {
        if state.game_over.is_some() {
            return;
        }
        let mut advanced = false;
        let quest_ids: Vec<QuestId> = state.quest_stages.keys().cloned().collect();
        for quest_id in quest_ids {
            let Some(stage_id) = state.quest_stages.get(&quest_id).cloned() else {
                continue;
            };
            let Some(quest) = engine.world().quests.get(&quest_id) else {
                continue;
            };
            let Some(stage) = quest.stages.get(&stage_id) else {
                continue;
            };
            if stage.kind != QuestStageKind::Active {
                continue;
            }
            let transitions = stage.transitions.clone();
            for transition in transitions {
                if !eval::evaluate(engine, state, &transition.condition) {
                    continue;
                }
                let mut ctx = exec::Context::new();
                ctx.run_effects(engine, state, &transition.effects);
                ctx.run_effects(
                    engine,
                    state,
                    &[Effect::SetQuestStage(
                        quest_id.clone(),
                        transition.to.clone(),
                    )],
                );
                ctx.drain(engine, state);
                advanced = true;
                break;
            }
            if state.game_over.is_some() {
                return;
            }
        }
        if !advanced {
            break;
        }
    }
}

fn action_advances_turn(action: &ChoiceAction) -> bool {
    !matches!(
        action,
        ChoiceAction::Look
            | ChoiceAction::Inventory
            | ChoiceAction::Examine(_)
            | ChoiceAction::ExamineKeyword(_)
            | ChoiceAction::Read(_)
            | ChoiceAction::TalkTo(_)
            | ChoiceAction::DialogueOption(_)
            | ChoiceAction::LeaveDialogue
    )
}

fn echo_choice(engine: &Engine, state: &mut RuntimeState, choice: &Choice) {
    let rendered = resolve::resolve_mut(engine, state, &choice.label);
    if !rendered.is_empty() {
        state.push_transcript(TranscriptEntry::PlayerAction(format!("> {rendered}")));
    }
}

fn apply_choice(
    ctx: &mut exec::Context,
    engine: &Engine,
    state: &mut RuntimeState,
    choice: Choice,
) {
    match choice.action {
        ChoiceAction::Go { to, .. } => go_to(ctx, engine, state, to),
        ChoiceAction::Take(item) => take(ctx, engine, state, item),
        ChoiceAction::Drop(item) => drop_item(ctx, engine, state, item),
        ChoiceAction::Use(item) => use_item(ctx, engine, state, item),
        ChoiceAction::Examine(item) => examine_item(engine, state, item),
        ChoiceAction::ExamineKeyword(keyword) => examine_keyword(engine, state, keyword),
        ChoiceAction::Read(item) => read_item(engine, state, item),
        ChoiceAction::TalkTo(npc) => talk_to(ctx, engine, state, npc),
        ChoiceAction::DialogueOption(index) => pick_dialogue_option(ctx, engine, state, index),
        ChoiceAction::LeaveDialogue => {
            state.active_dialogue = None;
            let message = engine.world().verb_responses.leave_dialogue.clone();
            state.push_transcript(TranscriptEntry::System(message));
        }
        ChoiceAction::Look => {
            exec::describe_current_room(engine, state);
        }
        ChoiceAction::Inventory => show_inventory(engine, state),
        ChoiceAction::Wait => {
            let message = engine.world().verb_responses.wait.clone();
            state.push_transcript(TranscriptEntry::Narration(message));
        }
        ChoiceAction::Effects(effects) => ctx.run_effects(engine, state, &effects),
    }
}

fn go_to(ctx: &mut exec::Context, engine: &Engine, state: &mut RuntimeState, destination: RoomId) {
    exec::move_player(ctx, engine, state, destination);
}

fn take(ctx: &mut exec::Context, engine: &Engine, state: &mut RuntimeState, item: ItemId) {
    let Some(item_def) = engine.world().items.get(&item) else {
        return;
    };
    let responses = &engine.world().verb_responses;
    if matches!(
        state.item_locations.get(&item),
        Some(ItemLocation::Inventory)
    ) {
        let message = VerbResponses::render(
            &responses.take_already_carrying,
            &[(Placeholder::Item, &item_def.name)],
        );
        state.push_transcript(TranscriptEntry::System(message));
        return;
    }
    if !item_def.properties.takeable {
        let message = VerbResponses::render(
            &responses.take_not_takeable,
            &[(Placeholder::Item, &item_def.name)],
        );
        state.push_transcript(TranscriptEntry::System(message));
        return;
    }
    let message = VerbResponses::render(
        &responses.take_success,
        &[(Placeholder::Item, &item_def.name)],
    );
    state
        .item_locations
        .insert(item.clone(), ItemLocation::Inventory);
    state.push_transcript(TranscriptEntry::Narration(message));
    ctx.queue(dispatch::Event::TakeItem(item));
}

fn drop_item(ctx: &mut exec::Context, engine: &Engine, state: &mut RuntimeState, item: ItemId) {
    if !matches!(
        state.item_locations.get(&item),
        Some(ItemLocation::Inventory)
    ) {
        return;
    }
    let here = state.current_room.clone();
    state
        .item_locations
        .insert(item.clone(), ItemLocation::Room(here));
    if let Some(item_def) = engine.world().items.get(&item) {
        let message = VerbResponses::render(
            &engine.world().verb_responses.drop_success,
            &[(Placeholder::Item, &item_def.name)],
        );
        state.push_transcript(TranscriptEntry::Narration(message));
    }
    ctx.queue(dispatch::Event::DropItem(item));
}

fn use_item(ctx: &mut exec::Context, engine: &Engine, state: &mut RuntimeState, item: ItemId) {
    if !matches!(
        state.item_locations.get(&item),
        Some(ItemLocation::Inventory)
    ) {
        if let Some(item_def) = engine.world().items.get(&item) {
            let message = VerbResponses::render(
                &engine.world().verb_responses.use_not_carrying,
                &[(Placeholder::Item, &item_def.name)],
            );
            state.push_transcript(TranscriptEntry::System(message));
        }
        return;
    }
    ctx.queue(dispatch::Event::UseItem {
        item,
        room: state.current_room.clone(),
    });
}

fn examine_item(engine: &Engine, state: &mut RuntimeState, item: ItemId) {
    if let Some(item_def) = engine.world().items.get(&item).cloned() {
        let text = item_def.long.clone();
        let rendered = resolve::resolve_mut(engine, state, &text);
        if !rendered.is_empty() {
            state.push_transcript(TranscriptEntry::Narration(rendered));
        }
    }
    // Examining also fires a trigger so rules can react.
    let mut ctx = exec::Context::new();
    ctx.queue(dispatch::Event::ExamineItem(item));
    ctx.drain(engine, state);
}

fn examine_keyword(engine: &Engine, state: &mut RuntimeState, keyword: String) {
    let Some(room) = engine.world().rooms.get(&state.current_room).cloned() else {
        return;
    };
    let key = keyword.to_lowercase();
    match room.examine.get(&key) {
        Some(text) => {
            let owned = text.clone();
            let rendered = resolve::resolve_mut(engine, state, &owned);
            if !rendered.is_empty() {
                state.push_transcript(TranscriptEntry::Narration(rendered));
            }
        }
        None => {
            let message = engine.world().verb_responses.examine_unknown.clone();
            state.push_transcript(TranscriptEntry::System(message));
        }
    }
}

fn read_item(engine: &Engine, state: &mut RuntimeState, item: ItemId) {
    let Some(item_def) = engine.world().items.get(&item).cloned() else {
        return;
    };
    match &item_def.read {
        Some(text) => {
            let owned = text.clone();
            let rendered = resolve::resolve_mut(engine, state, &owned);
            if !rendered.is_empty() {
                state.push_transcript(TranscriptEntry::Narration(rendered));
            }
        }
        None => {
            let message = VerbResponses::render(
                &engine.world().verb_responses.read_nothing_written,
                &[(Placeholder::Item, &item_def.name)],
            );
            state.push_transcript(TranscriptEntry::System(message));
        }
    }
}

fn talk_to(ctx: &mut exec::Context, engine: &Engine, state: &mut RuntimeState, npc: NpcId) {
    let Some(npc_def) = engine.world().npcs.get(&npc).cloned() else {
        return;
    };
    let Some(dialogue_id) = npc_def.dialogue.clone() else {
        let message = VerbResponses::render(
            &engine.world().verb_responses.npc_silent,
            &[(Placeholder::Npc, &npc_def.name)],
        );
        state.push_transcript(TranscriptEntry::System(message));
        return;
    };
    ctx.queue(dispatch::Event::TalkTo(npc.clone()));
    ctx.run_effects(
        engine,
        state,
        &[crate::data::Effect::BeginDialogue(dialogue_id)],
    );
}

fn pick_dialogue_option(
    ctx: &mut exec::Context,
    engine: &Engine,
    state: &mut RuntimeState,
    index: usize,
) {
    let Some((dialogue_id, node_id)) = state.active_dialogue.clone() else {
        return;
    };
    let Some(dialogue) = engine.world().dialogues.get(&dialogue_id).cloned() else {
        return;
    };
    let Some(node) = dialogue.nodes.get(&node_id) else {
        return;
    };
    let Some(option) = node.options.get(index).cloned() else {
        return;
    };

    if let Some(condition) = &option.condition
        && !eval::evaluate(engine, state, condition)
    {
        if let Some(reason) = &option.locked_reason {
            let rendered = resolve::resolve_mut(engine, state, reason);
            state.push_transcript(TranscriptEntry::System(rendered));
        }
        return;
    }

    ctx.run_effects(engine, state, &option.effects);
    match option.goto {
        Some(next) => {
            ctx.run_effects(engine, state, &[crate::data::Effect::GotoDialogue(next)]);
        }
        None => {
            state.active_dialogue = None;
        }
    }
}

fn show_inventory(engine: &Engine, state: &mut RuntimeState) {
    let items: Vec<(ItemId, &Item)> = engine
        .world()
        .items
        .iter()
        .filter(|(id, _)| matches!(state.item_locations.get(*id), Some(ItemLocation::Inventory)))
        .map(|(id, item)| (id.clone(), item))
        .collect();
    let responses = &engine.world().verb_responses;
    let message = if items.is_empty() {
        responses.inventory_empty.clone()
    } else {
        let names: Vec<String> = items.iter().map(|(_, item)| item.name.clone()).collect();
        format!(
            "{}{}.",
            responses.inventory_listing_prefix,
            names.join(", ")
        )
    };
    state.push_transcript(TranscriptEntry::System(message));
}

fn tick_timers(engine: &Engine, state: &mut RuntimeState) {
    let mut to_tick: Vec<TimerId> = state.timers_remaining.keys().cloned().collect();
    to_tick.sort();

    for timer_id in to_tick {
        let Some(timer_def) = engine.world().timers.get(&timer_id).cloned() else {
            continue;
        };
        if let Some(cancel) = &timer_def.cancel_on
            && eval::evaluate(engine, state, cancel)
        {
            state.timers_remaining.remove(&timer_id);
            state.timers_cancelled.insert(timer_id.clone());
            continue;
        }

        let remaining = state.timers_remaining.get(&timer_id).copied().unwrap_or(0);
        let next = remaining.saturating_sub(1);

        let mut ctx = exec::Context::new();
        ctx.run_effects(engine, state, &timer_def.on_tick);
        if state.game_over.is_some() {
            return;
        }

        if next == 0 {
            state.timers_remaining.remove(&timer_id);
            state.timers_expired.insert(timer_id.clone());
            ctx.run_effects(engine, state, &timer_def.on_expire);
            ctx.drain(engine, state);
        } else {
            state.timers_remaining.insert(timer_id.clone(), next);
            ctx.drain(engine, state);
        }
        if state.game_over.is_some() {
            return;
        }
    }
}

fn run_due_scheduled_events(engine: &Engine, state: &mut RuntimeState) {
    let due: Vec<_> = state
        .scheduled_events
        .iter()
        .enumerate()
        .filter(|(_, e)| e.fires_on_turn <= state.turn)
        .map(|(i, e)| (i, e.event.clone()))
        .collect();

    if due.is_empty() {
        return;
    }

    let indices: Vec<usize> = due.iter().map(|(i, _)| *i).collect();
    // Remove in reverse order so indices stay valid.
    for i in indices.into_iter().rev() {
        state.scheduled_events.remove(i);
    }

    let mut ctx = exec::Context::new();
    for (_, event) in due {
        ctx.queue(dispatch::Event::Named(event));
    }
    ctx.drain(engine, state);
}

fn evaluate_endings(engine: &Engine, state: &mut RuntimeState) {
    if state.game_over.is_some() {
        return;
    }
    let mut winners: Vec<(EndingId, i32)> = engine
        .world()
        .endings
        .iter()
        .filter(|(_, ending)| eval::evaluate(engine, state, &ending.trigger))
        .map(|(id, ending)| (id.clone(), ending.priority))
        .collect();
    winners.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let Some((ending_id, _)) = winners.into_iter().next() else {
        return;
    };
    let mut ctx = exec::Context::new();
    ctx.run_effects(
        engine,
        state,
        &[crate::data::Effect::TriggerEnding(ending_id)],
    );
    ctx.drain(engine, state);
}

fn maybe_fire(
    ctx: &mut exec::Context,
    engine: &Engine,
    state: &mut RuntimeState,
    event: dispatch::Event,
) {
    ctx.queue(event);
    ctx.drain(engine, state);
}
