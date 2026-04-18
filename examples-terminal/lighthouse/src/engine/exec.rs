//! Effect execution and the event bus.
//!
//! The executor processes one event at a time from a FIFO queue. Firing an
//! event finds matching rules (via [`dispatch`]) and runs their effects,
//! which may in turn enqueue further events. The loop continues until the
//! queue is empty.
//!
//! All state mutation goes through this module, so transcript entries, flag
//! sets, item moves, and ending triggers are kept consistent.

use crate::data::{
    DialogueId, Effect, EndingId, FlagKey, ItemId, ItemLocation, NodeId, Placeholder, QuestId,
    RoomId, RuleId, ScheduledEvent, StatKey, TimerId, TranscriptEntry, Value, VerbResponses,
};
use crate::engine::{Engine, dispatch, eval, resolve};
use std::collections::VecDeque;

/// Driver for a sequence of effects and the events they raise.
pub struct Context {
    queue: VecDeque<dispatch::Event>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn queue(&mut self, event: dispatch::Event) {
        self.queue.push_back(event);
    }

    /// Drain the event queue, firing each event and running matching rules.
    pub fn drain(&mut self, engine: &Engine, state: &mut crate::data::RuntimeState) {
        while let Some(event) = self.queue.pop_front() {
            self.dispatch_event(engine, state, event);
            if state.game_over.is_some() {
                return;
            }
        }
    }

    fn dispatch_event(
        &mut self,
        engine: &Engine,
        state: &mut crate::data::RuntimeState,
        event: dispatch::Event,
    ) {
        let rules = dispatch::candidates(engine, state, &event);
        for rule_id in rules {
            self.fire_rule(engine, state, rule_id);
            if state.game_over.is_some() {
                return;
            }
        }
    }

    fn fire_rule(
        &mut self,
        engine: &Engine,
        state: &mut crate::data::RuntimeState,
        rule_id: RuleId,
    ) {
        if engine.rule_tracing_enabled() {
            let line = VerbResponses::render(
                &engine.world().verb_responses.trace_prefix,
                &[(Placeholder::Rule, rule_id.as_str())],
            );
            state.push_transcript(TranscriptEntry::System(line));
        }
        // Record fire metadata before effects so the rule can reference itself
        // via Condition::RuleFired inside nested effects.
        let effects = engine
            .world()
            .rules
            .get(&rule_id)
            .map(|rule| rule.effects.clone())
            .unwrap_or_default();
        state.rules_fired.insert(rule_id.clone());
        state.rule_last_fired.insert(rule_id, state.turn);
        self.run_effects(engine, state, &effects);
    }

    /// Run a sequence of effects against state, pushing events as appropriate.
    pub fn run_effects(
        &mut self,
        engine: &Engine,
        state: &mut crate::data::RuntimeState,
        effects: &[Effect],
    ) {
        for effect in effects {
            if state.game_over.is_some() {
                return;
            }
            self.run_effect(engine, state, effect);
        }
    }

    fn run_effect(
        &mut self,
        engine: &Engine,
        state: &mut crate::data::RuntimeState,
        effect: &Effect,
    ) {
        match effect {
            Effect::Say(text) => {
                let rendered = resolve::resolve_mut(engine, state, text);
                if !rendered.is_empty() {
                    state.push_transcript(TranscriptEntry::Narration(rendered));
                }
            }
            Effect::DescribeRoom => describe_current_room(engine, state),
            Effect::ClearTranscript => state.transcript.clear(),

            Effect::SetFlag(key, value) => set_flag(self, state, key.clone(), value.clone()),
            Effect::UnsetFlag(key) => unset_flag(self, state, key.clone()),
            Effect::AddStat(key, delta) => add_stat(state, key.clone(), *delta),
            Effect::SetStat(key, value) => {
                state.stats.insert(key.clone(), *value);
            }

            Effect::MoveItem(item, location) => {
                move_item(state, item.clone(), location.clone());
            }
            Effect::MovePlayer(room) => move_player(self, engine, state, room.clone()),
            Effect::MoveNpc(npc, room) => {
                state.npc_locations.insert(npc.clone(), room.clone());
            }
            Effect::AdjustDisposition(npc, delta) => {
                let current = state.dispositions.get(npc).copied().unwrap_or(0);
                state.dispositions.insert(npc.clone(), current + *delta);
            }

            Effect::SetQuestStage(quest, stage) => {
                set_quest_stage(self, engine, state, quest.clone(), stage.clone());
            }

            Effect::BeginDialogue(dialogue) => {
                begin_dialogue(self, engine, state, dialogue.clone());
            }
            Effect::EndDialogue => {
                state.active_dialogue = None;
            }
            Effect::GotoDialogue(node) => {
                goto_dialogue(self, engine, state, node.clone());
            }

            Effect::If {
                when,
                then,
                otherwise,
            } => {
                let chosen = if eval::evaluate_mut(engine, state, when) {
                    then
                } else {
                    otherwise
                };
                self.run_effects(engine, state, chosen);
            }
            Effect::Sequence(inner) => self.run_effects(engine, state, inner),
            Effect::OneOf(branches) => {
                if !branches.is_empty() {
                    let pick = state.random_index(branches.len());
                    let owned = branches[pick].clone();
                    self.run_effects(engine, state, &owned);
                }
            }

            Effect::OfferChoices(choices) => {
                state.pending_choices = choices.clone();
            }

            Effect::TriggerEvent(name) => {
                self.queue(dispatch::Event::Named(name.clone()));
            }
            Effect::FireRule(rule) => self.fire_rule(engine, state, rule.clone()),

            Effect::StartTimer(timer) => start_timer(state, engine, timer.clone()),
            Effect::CancelTimer(timer) => cancel_timer(state, timer.clone()),
            Effect::ScheduleEvent { event, in_turns } => {
                state.scheduled_events.push(ScheduledEvent {
                    event: event.clone(),
                    fires_on_turn: state.turn + *in_turns,
                });
            }

            Effect::TriggerEnding(ending) => {
                trigger_ending(self, engine, state, ending.clone());
            }
        }
    }
}

pub(crate) fn describe_current_room(engine: &Engine, state: &mut crate::data::RuntimeState) {
    let Some(room) = engine.world().rooms.get(&state.current_room) else {
        return;
    };
    let dark = room.dark && !crate::engine::helpers::player_has_light(engine, state);
    let header = VerbResponses::render(
        &engine.world().verb_responses.room_header,
        &[(Placeholder::Name, &room.name)],
    );
    state.push_transcript(TranscriptEntry::Narration(header));

    let body = if dark {
        room.dark_description.clone().unwrap_or_default()
    } else {
        room.description.clone()
    };
    let rendered = resolve::resolve_mut(engine, state, &body);
    if !rendered.is_empty() {
        state.push_transcript(TranscriptEntry::Narration(rendered));
    }

    if !dark {
        describe_visible_here(engine, state);
    }
}

fn describe_visible_here(engine: &Engine, state: &mut crate::data::RuntimeState) {
    let here = state.current_room.clone();
    let mut visible: Vec<String> = Vec::new();

    for (npc_id, room) in &state.npc_locations {
        if room == &here
            && let Some(npc) = engine.world().npcs.get(npc_id)
        {
            visible.push(npc.name.clone());
        }
    }

    for (item_id, item) in &engine.world().items {
        if matches!(
            state.item_locations.get(item_id),
            Some(ItemLocation::Room(room)) if room == &here
        ) {
            visible.push(item.name.clone());
        }
    }

    if visible.is_empty() {
        return;
    }
    let prefix = &engine.world().verb_responses.visible_listing_prefix;
    let line = format!("{prefix}{}.", visible.join(", "));
    state.push_transcript(TranscriptEntry::Narration(line));
}

fn set_flag(ctx: &mut Context, state: &mut crate::data::RuntimeState, key: FlagKey, value: Value) {
    let was_unset = matches!(state.flags.get(&key), Some(Value::Bool(false)) | None);
    state.flags.insert(key.clone(), value);
    if was_unset {
        ctx.queue(dispatch::Event::FlagSet(key));
    }
}

fn unset_flag(ctx: &mut Context, state: &mut crate::data::RuntimeState, key: FlagKey) {
    let was_set = matches!(state.flags.get(&key), Some(v) if !matches!(v, Value::Bool(false)));
    state.flags.remove(&key);
    if was_set {
        ctx.queue(dispatch::Event::FlagUnset(key));
    }
}

fn add_stat(state: &mut crate::data::RuntimeState, key: StatKey, delta: i64) {
    let current = state.stats.get(&key).copied().unwrap_or(0);
    state.stats.insert(key, current + delta);
}

fn move_item(state: &mut crate::data::RuntimeState, item: ItemId, location: ItemLocation) {
    state.item_locations.insert(item, location);
}

pub(crate) fn move_player(
    ctx: &mut Context,
    engine: &Engine,
    state: &mut crate::data::RuntimeState,
    room: RoomId,
) {
    let previous = state.current_room.clone();
    if previous != room {
        ctx.queue(dispatch::Event::PlayerExited(previous.clone()));
    }
    state.previous_room = Some(previous.clone());
    state.current_room = room.clone();
    state.visited.insert(room.clone());
    if previous != room {
        ctx.queue(dispatch::Event::PlayerEntered(room.clone()));
    }
    state.push_transcript(TranscriptEntry::Separator);
    describe_current_room(engine, state);
}

fn set_quest_stage(
    ctx: &mut Context,
    engine: &Engine,
    state: &mut crate::data::RuntimeState,
    quest: QuestId,
    stage: NodeId,
) {
    state.quest_stages.insert(quest.clone(), stage.clone());
    state
        .quest_history
        .entry(quest.clone())
        .or_default()
        .insert(stage.clone());

    if let Some(quest_def) = engine.world().quests.get(&quest)
        && let Some(stage_def) = quest_def.stages.get(&stage)
    {
        let on_enter = stage_def.on_enter.clone();
        ctx.run_effects(engine, state, &on_enter);
    }
}

fn begin_dialogue(
    ctx: &mut Context,
    engine: &Engine,
    state: &mut crate::data::RuntimeState,
    dialogue: DialogueId,
) {
    let Some(def) = engine.world().dialogues.get(&dialogue) else {
        return;
    };
    let start = def.start.clone();
    state.active_dialogue = Some((dialogue.clone(), start.clone()));
    run_dialogue_on_enter(ctx, engine, state, &dialogue, &start);
}

fn goto_dialogue(
    ctx: &mut Context,
    engine: &Engine,
    state: &mut crate::data::RuntimeState,
    node: NodeId,
) {
    let Some((dialogue_id, _)) = state.active_dialogue.clone() else {
        return;
    };
    state.active_dialogue = Some((dialogue_id.clone(), node.clone()));
    run_dialogue_on_enter(ctx, engine, state, &dialogue_id, &node);
}

fn run_dialogue_on_enter(
    ctx: &mut Context,
    engine: &Engine,
    state: &mut crate::data::RuntimeState,
    dialogue: &DialogueId,
    node: &NodeId,
) {
    let (text, on_enter) = match engine
        .world()
        .dialogues
        .get(dialogue)
        .and_then(|d| d.nodes.get(node))
    {
        Some(def) => (def.text.clone(), def.on_enter.clone()),
        None => return,
    };
    let speaker = engine
        .world()
        .dialogues
        .get(dialogue)
        .and_then(|_| {
            engine.world().npcs.values().find_map(|npc| {
                if npc.dialogue.as_ref() == Some(dialogue) {
                    Some(npc.name.clone())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| {
            engine
                .world()
                .verb_responses
                .dialogue_default_speaker
                .clone()
        });
    let rendered = resolve::resolve_mut(engine, state, &text);
    if !rendered.is_empty() {
        state.push_transcript(TranscriptEntry::Dialogue {
            speaker,
            text: rendered,
        });
    }
    ctx.run_effects(engine, state, &on_enter);
}

fn start_timer(state: &mut crate::data::RuntimeState, engine: &Engine, timer: TimerId) {
    if let Some(def) = engine.world().timers.get(&timer) {
        state
            .timers_remaining
            .insert(timer.clone(), def.initial_turns);
        state.timers_expired.remove(&timer);
        state.timers_cancelled.remove(&timer);
    }
}

fn cancel_timer(state: &mut crate::data::RuntimeState, timer: TimerId) {
    if state.timers_remaining.remove(&timer).is_some() {
        state.timers_cancelled.insert(timer);
    }
}

fn trigger_ending(
    ctx: &mut Context,
    engine: &Engine,
    state: &mut crate::data::RuntimeState,
    ending: EndingId,
) {
    state.unlocked_endings.insert(ending.clone());
    state.game_over = Some(ending.clone());

    if let Some(def) = engine.world().endings.get(&ending) {
        let header = format!("*** {} ***", def.title);
        state.push_transcript(TranscriptEntry::Separator);
        state.push_transcript(TranscriptEntry::Narration(header));
        let description = def.description.clone();
        let epilogue = def.epilogue.clone();
        let rendered_desc = resolve::resolve_mut(engine, state, &description);
        if !rendered_desc.is_empty() {
            state.push_transcript(TranscriptEntry::Narration(rendered_desc));
        }
        let rendered_epilogue = resolve::resolve_mut(engine, state, &epilogue);
        if !rendered_epilogue.is_empty() {
            state.push_transcript(TranscriptEntry::Narration(rendered_epilogue));
        }
    }
    // Clear any pending UI so the game-over screen isn't hidden behind a menu.
    state.pending_choices.clear();
    // Drain any remaining events to keep behavior deterministic on ending.
    let _ = ctx;
}

/// Re-export for checks from outside the module.
pub use super::eval::evaluate;
