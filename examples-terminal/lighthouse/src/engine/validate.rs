//! Load-time validation.
//!
//! Runs once at `Engine::new` and catches content bugs that would otherwise
//! only surface during play: dangling references, missing start rooms,
//! unreachable rooms, quest stages without exits, and similar.

use crate::data::{
    Condition, DialogueId, Effect, EndingId, ItemId, ItemLocation, NodeId, NpcId, QuestId, RoomId,
    RuleId, Text, TextId, TimerId, World,
};
use std::collections::{BTreeSet, VecDeque};
use std::fmt;

/// One thing wrong with the content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    MissingStartRoom(RoomId),
    UnreachableRoom(RoomId),
    ExitTargetMissing {
        from: RoomId,
        to: RoomId,
    },
    ItemRefMissing {
        context: String,
        item: ItemId,
    },
    NpcRefMissing {
        context: String,
        npc: NpcId,
    },
    RoomRefMissing {
        context: String,
        room: RoomId,
    },
    QuestRefMissing {
        context: String,
        quest: QuestId,
    },
    QuestStageRefMissing {
        quest: QuestId,
        stage: NodeId,
    },
    DialogueRefMissing {
        context: String,
        dialogue: DialogueId,
    },
    DialogueNodeMissing {
        dialogue: DialogueId,
        node: NodeId,
    },
    EndingRefMissing(EndingId),
    RuleRefMissing(RuleId),
    TimerRefMissing {
        context: String,
        timer: TimerId,
    },
    TextRefMissing(TextId),
    ActiveStageWithoutTransitions {
        quest: QuestId,
        stage: NodeId,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingStartRoom(room) => {
                write!(f, "World.start_room '{room}' not found in rooms")
            }
            ValidationError::UnreachableRoom(room) => {
                write!(f, "Room '{room}' is not reachable from start_room")
            }
            ValidationError::ExitTargetMissing { from, to } => {
                write!(f, "Exit from '{from}' targets missing room '{to}'")
            }
            ValidationError::ItemRefMissing { context, item } => {
                write!(f, "{context} references missing item '{item}'")
            }
            ValidationError::NpcRefMissing { context, npc } => {
                write!(f, "{context} references missing NPC '{npc}'")
            }
            ValidationError::RoomRefMissing { context, room } => {
                write!(f, "{context} references missing room '{room}'")
            }
            ValidationError::QuestRefMissing { context, quest } => {
                write!(f, "{context} references missing quest '{quest}'")
            }
            ValidationError::QuestStageRefMissing { quest, stage } => {
                write!(f, "Quest '{quest}' references missing stage '{stage}'")
            }
            ValidationError::DialogueRefMissing { context, dialogue } => {
                write!(f, "{context} references missing dialogue '{dialogue}'")
            }
            ValidationError::DialogueNodeMissing { dialogue, node } => {
                write!(f, "Dialogue '{dialogue}' references missing node '{node}'")
            }
            ValidationError::EndingRefMissing(ending) => {
                write!(f, "Reference to missing ending '{ending}'")
            }
            ValidationError::RuleRefMissing(rule) => {
                write!(f, "Reference to missing rule '{rule}'")
            }
            ValidationError::TimerRefMissing { context, timer } => {
                write!(f, "{context} references missing timer '{timer}'")
            }
            ValidationError::TextRefMissing(text) => {
                write!(f, "Reference to missing text '{text}'")
            }
            ValidationError::ActiveStageWithoutTransitions { quest, stage } => {
                write!(
                    f,
                    "Quest '{quest}' active stage '{stage}' has no outgoing transitions"
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Run all validation checks. Returns `Ok(())` if everything is well-formed,
/// otherwise a list of every error found.
pub fn validate(world: &World) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    validate_start(world, &mut errors);
    validate_room_exits(world, &mut errors);
    validate_reachability(world, &mut errors);
    validate_rule_effects(world, &mut errors);
    validate_quests(world, &mut errors);
    validate_dialogues(world, &mut errors);
    validate_endings(world, &mut errors);
    validate_items_and_npcs(world, &mut errors);
    validate_timers(world, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_start(world: &World, errors: &mut Vec<ValidationError>) {
    if !world.rooms.contains_key(&world.start_room) {
        errors.push(ValidationError::MissingStartRoom(world.start_room.clone()));
    }
}

fn validate_room_exits(world: &World, errors: &mut Vec<ValidationError>) {
    for (room_id, room) in &world.rooms {
        let context = format!("Room '{room_id}' exit");
        for exit in &room.exits {
            if !world.rooms.contains_key(&exit.to) {
                errors.push(ValidationError::ExitTargetMissing {
                    from: room_id.clone(),
                    to: exit.to.clone(),
                });
            }
            if let Some(condition) = &exit.passable_when {
                validate_condition(world, &context, condition, errors);
            }
            if let Some(condition) = &exit.visible_when {
                validate_condition(world, &context, condition, errors);
            }
            if let Some(text) = &exit.locked_message {
                validate_text(world, text, errors);
            }
        }
        for text in room.examine.values() {
            validate_text(world, text, errors);
        }
        if let Some(text) = &room.dark_description {
            validate_text(world, text, errors);
        }
        validate_text(world, &room.description, errors);
    }
}

fn validate_reachability(world: &World, errors: &mut Vec<ValidationError>) {
    if !world.rooms.contains_key(&world.start_room) {
        return;
    }
    let mut seen: BTreeSet<RoomId> = BTreeSet::new();
    let mut queue: VecDeque<RoomId> = VecDeque::new();
    queue.push_back(world.start_room.clone());
    seen.insert(world.start_room.clone());

    while let Some(room_id) = queue.pop_front() {
        let Some(room) = world.rooms.get(&room_id) else {
            continue;
        };
        for exit in &room.exits {
            if seen.insert(exit.to.clone()) {
                queue.push_back(exit.to.clone());
            }
        }
    }

    // Also include rooms reachable via `Effect::MovePlayer` in any rule,
    // quest, dialogue, ending, or timer. This captures teleports.
    let mut teleport_targets = BTreeSet::new();
    for rule in world.rules.values() {
        collect_teleport_targets(&rule.effects, &mut teleport_targets);
    }
    for quest in world.quests.values() {
        for stage in quest.stages.values() {
            collect_teleport_targets(&stage.on_enter, &mut teleport_targets);
            for transition in &stage.transitions {
                collect_teleport_targets(&transition.effects, &mut teleport_targets);
            }
        }
    }
    for dialogue in world.dialogues.values() {
        for node in dialogue.nodes.values() {
            collect_teleport_targets(&node.on_enter, &mut teleport_targets);
            for option in &node.options {
                collect_teleport_targets(&option.effects, &mut teleport_targets);
            }
        }
    }
    for timer in world.timers.values() {
        collect_teleport_targets(&timer.on_tick, &mut teleport_targets);
        collect_teleport_targets(&timer.on_expire, &mut teleport_targets);
    }
    for target in teleport_targets {
        seen.insert(target);
    }

    for room_id in world.rooms.keys() {
        if !seen.contains(room_id) {
            errors.push(ValidationError::UnreachableRoom(room_id.clone()));
        }
    }
}

fn collect_teleport_targets(effects: &[Effect], out: &mut BTreeSet<RoomId>) {
    for effect in effects {
        match effect {
            Effect::MovePlayer(room) => {
                out.insert(room.clone());
            }
            Effect::If {
                then, otherwise, ..
            } => {
                collect_teleport_targets(then, out);
                collect_teleport_targets(otherwise, out);
            }
            Effect::Sequence(inner) => collect_teleport_targets(inner, out),
            Effect::OneOf(branches) => {
                for branch in branches {
                    collect_teleport_targets(branch, out);
                }
            }

            // Effects that cannot teleport the player. Listed exhaustively so
            // adding a new movement variant forces an update here instead of
            // silently falling through the wildcard.
            Effect::Say(_)
            | Effect::DescribeRoom
            | Effect::ClearTranscript
            | Effect::SetFlag(_, _)
            | Effect::UnsetFlag(_)
            | Effect::AddStat(_, _)
            | Effect::SetStat(_, _)
            | Effect::MoveItem(_, _)
            | Effect::MoveNpc(_, _)
            | Effect::AdjustDisposition(_, _)
            | Effect::SetQuestStage(_, _)
            | Effect::BeginDialogue(_)
            | Effect::EndDialogue
            | Effect::GotoDialogue(_)
            | Effect::OfferChoices(_)
            | Effect::TriggerEvent(_)
            | Effect::FireRule(_)
            | Effect::StartTimer(_)
            | Effect::CancelTimer(_)
            | Effect::ScheduleEvent { .. }
            | Effect::TriggerEnding(_) => {}
        }
    }
}

fn validate_rule_effects(world: &World, errors: &mut Vec<ValidationError>) {
    for (rule_id, rule) in &world.rules {
        let ctx = format!("Rule '{rule_id}'");
        validate_condition(world, &ctx, &rule.condition, errors);
        validate_effects(world, &ctx, &rule.effects, errors);
    }
}

fn validate_effects(
    world: &World,
    context: &str,
    effects: &[Effect],
    errors: &mut Vec<ValidationError>,
) {
    for effect in effects {
        match effect {
            Effect::Say(text) => validate_text(world, text, errors),
            Effect::DescribeRoom => {}
            Effect::ClearTranscript => {}
            Effect::SetFlag(_, _) | Effect::UnsetFlag(_) => {}
            Effect::AddStat(_, _) | Effect::SetStat(_, _) => {}
            Effect::MoveItem(item, location) => {
                check_item(world, context, item, errors);
                check_location(world, context, location, errors);
            }
            Effect::MovePlayer(room) => check_room(world, context, room, errors),
            Effect::MoveNpc(npc, room) => {
                check_npc(world, context, npc, errors);
                check_room(world, context, room, errors);
            }
            Effect::AdjustDisposition(npc, _) => check_npc(world, context, npc, errors),
            Effect::SetQuestStage(quest, stage) => check_quest_stage(world, quest, stage, errors),
            Effect::BeginDialogue(dialogue) => {
                check_dialogue(world, context, dialogue, errors);
            }
            Effect::EndDialogue => {}
            Effect::GotoDialogue(_node) => {
                // Validated per-dialogue when dialogues are checked.
            }
            Effect::If {
                when,
                then,
                otherwise,
            } => {
                validate_condition(world, context, when, errors);
                validate_effects(world, context, then, errors);
                validate_effects(world, context, otherwise, errors);
            }
            Effect::Sequence(inner) => validate_effects(world, context, inner, errors),
            Effect::OneOf(branches) => {
                for branch in branches {
                    validate_effects(world, context, branch, errors);
                }
            }
            Effect::OfferChoices(choices) => {
                for choice in choices {
                    if let Some(condition) = &choice.condition {
                        validate_condition(world, context, condition, errors);
                    }
                }
            }
            Effect::TriggerEvent(_) => {}
            Effect::FireRule(rule) => {
                if !world.rules.contains_key(rule) {
                    errors.push(ValidationError::RuleRefMissing(rule.clone()));
                }
            }
            Effect::StartTimer(timer) | Effect::CancelTimer(timer) => {
                check_timer(world, context, timer, errors);
            }
            Effect::ScheduleEvent { .. } => {}
            Effect::TriggerEnding(ending) => {
                if !world.endings.contains_key(ending) {
                    errors.push(ValidationError::EndingRefMissing(ending.clone()));
                }
            }
        }
    }
}

fn validate_condition(
    world: &World,
    context: &str,
    condition: &Condition,
    errors: &mut Vec<ValidationError>,
) {
    match condition {
        Condition::All(inner) | Condition::Any(inner) => {
            for inner_condition in inner {
                validate_condition(world, context, inner_condition, errors);
            }
        }
        Condition::Not(inner) => validate_condition(world, context, inner, errors),
        Condition::PlayerIn(room) | Condition::Visited(room) => {
            check_room(world, context, room, errors)
        }
        Condition::HasItem(item) | Condition::ItemIsSomewhere(item) => {
            check_item(world, context, item, errors)
        }
        Condition::ItemInRoom(item, room) => {
            check_item(world, context, item, errors);
            check_room(world, context, room, errors);
        }
        Condition::NpcIn(npc, room) => {
            check_npc(world, context, npc, errors);
            check_room(world, context, room, errors);
        }
        Condition::DispositionAtLeast(npc, _) => check_npc(world, context, npc, errors),
        Condition::QuestAt(quest, stage) | Condition::QuestReached(quest, stage) => {
            check_quest_stage(world, quest, stage, errors)
        }
        Condition::TimerRunning(timer) | Condition::TimerExpired(timer) => {
            check_timer(world, context, timer, errors)
        }
        Condition::RuleFired(rule) => {
            if !world.rules.contains_key(rule) {
                errors.push(ValidationError::RuleRefMissing(rule.clone()));
            }
        }
        Condition::Ref(id) => {
            if !world.conditions.contains_key(id) {
                errors.push(ValidationError::TextRefMissing(crate::data::TextId::new(
                    id.as_str(),
                )));
            }
        }

        // Variants that do not embed any referential data. Listed
        // exhaustively so adding a new variant forces an update here
        // rather than silently falling through.
        Condition::Always
        | Condition::Never
        | Condition::FlagEquals(_, _)
        | Condition::FlagSet(_)
        | Condition::FlagUnset(_)
        | Condition::StatAtLeast(_, _)
        | Condition::StatAtMost(_, _)
        | Condition::TurnAtLeast(_)
        | Condition::TurnAtMost(_)
        | Condition::Chance(_) => {}
    }
}

fn validate_text(world: &World, text: &Text, errors: &mut Vec<ValidationError>) {
    match text {
        Text::Ref(id) if !world.texts.contains_key(id) => {
            errors.push(ValidationError::TextRefMissing(id.clone()));
        }
        Text::Conditional {
            when,
            then,
            otherwise,
        } => {
            validate_condition(world, "Text::Conditional", when, errors);
            validate_text(world, then, errors);
            validate_text(world, otherwise, errors);
        }
        Text::OneOf(variants) => {
            for variant in variants {
                validate_text(world, variant, errors);
            }
        }
        Text::Sequence(parts) => {
            for part in parts {
                validate_text(world, part, errors);
            }
        }
        _ => {}
    }
}

fn validate_quests(world: &World, errors: &mut Vec<ValidationError>) {
    for (quest_id, quest) in &world.quests {
        if !quest.stages.contains_key(&quest.start) {
            errors.push(ValidationError::QuestStageRefMissing {
                quest: quest_id.clone(),
                stage: quest.start.clone(),
            });
        }
        for (stage_id, stage) in &quest.stages {
            validate_effects(
                world,
                &format!("Quest '{quest_id}' stage '{stage_id}'"),
                &stage.on_enter,
                errors,
            );
            if stage.kind == crate::data::QuestStageKind::Active && stage.transitions.is_empty() {
                errors.push(ValidationError::ActiveStageWithoutTransitions {
                    quest: quest_id.clone(),
                    stage: stage_id.clone(),
                });
            }
            for transition in &stage.transitions {
                if !quest.stages.contains_key(&transition.to) {
                    errors.push(ValidationError::QuestStageRefMissing {
                        quest: quest_id.clone(),
                        stage: transition.to.clone(),
                    });
                }
                validate_condition(
                    world,
                    &format!("Quest '{quest_id}' transition to '{}'", transition.to),
                    &transition.condition,
                    errors,
                );
                validate_effects(
                    world,
                    &format!("Quest '{quest_id}' transition effects"),
                    &transition.effects,
                    errors,
                );
            }
        }
    }
}

fn validate_dialogues(world: &World, errors: &mut Vec<ValidationError>) {
    for (dialogue_id, dialogue) in &world.dialogues {
        if !dialogue.nodes.contains_key(&dialogue.start) {
            errors.push(ValidationError::DialogueNodeMissing {
                dialogue: dialogue_id.clone(),
                node: dialogue.start.clone(),
            });
        }
        for (node_id, node) in &dialogue.nodes {
            validate_text(world, &node.text, errors);
            validate_effects_in_dialogue(
                world,
                dialogue_id,
                dialogue,
                &format!("Dialogue '{dialogue_id}' node '{node_id}'"),
                &node.on_enter,
                errors,
            );
            for option in &node.options {
                if let Some(goto) = &option.goto
                    && !dialogue.nodes.contains_key(goto)
                {
                    errors.push(ValidationError::DialogueNodeMissing {
                        dialogue: dialogue_id.clone(),
                        node: goto.clone(),
                    });
                }
                if let Some(condition) = &option.condition {
                    validate_condition(
                        world,
                        &format!("Dialogue '{dialogue_id}' option"),
                        condition,
                        errors,
                    );
                }
                if let Some(reason) = &option.locked_reason {
                    validate_text(world, reason, errors);
                }
                validate_text(world, &option.label, errors);
                validate_effects_in_dialogue(
                    world,
                    dialogue_id,
                    dialogue,
                    &format!("Dialogue '{dialogue_id}' option"),
                    &option.effects,
                    errors,
                );
            }
        }
    }
}

/// Walk effects inside a dialogue. Shares behaviour with `validate_effects`
/// but additionally checks `Effect::GotoDialogue` node references against the
/// currently-nested dialogue.
fn validate_effects_in_dialogue(
    world: &World,
    dialogue_id: &crate::data::DialogueId,
    dialogue: &crate::data::Dialogue,
    context: &str,
    effects: &[crate::data::Effect],
    errors: &mut Vec<ValidationError>,
) {
    for effect in effects {
        match effect {
            crate::data::Effect::GotoDialogue(node) if !dialogue.nodes.contains_key(node) => {
                errors.push(ValidationError::DialogueNodeMissing {
                    dialogue: dialogue_id.clone(),
                    node: node.clone(),
                });
            }
            crate::data::Effect::If {
                when,
                then,
                otherwise,
            } => {
                validate_condition(world, context, when, errors);
                validate_effects_in_dialogue(world, dialogue_id, dialogue, context, then, errors);
                validate_effects_in_dialogue(
                    world,
                    dialogue_id,
                    dialogue,
                    context,
                    otherwise,
                    errors,
                );
            }
            crate::data::Effect::Sequence(inner) => {
                validate_effects_in_dialogue(world, dialogue_id, dialogue, context, inner, errors);
            }
            crate::data::Effect::OneOf(branches) => {
                for branch in branches {
                    validate_effects_in_dialogue(
                        world,
                        dialogue_id,
                        dialogue,
                        context,
                        branch,
                        errors,
                    );
                }
            }
            _ => {
                validate_effects(world, context, std::slice::from_ref(effect), errors);
            }
        }
    }
}

fn validate_endings(world: &World, errors: &mut Vec<ValidationError>) {
    for (ending_id, ending) in &world.endings {
        let ctx = format!("Ending '{ending_id}'");
        validate_condition(world, &ctx, &ending.trigger, errors);
        validate_text(world, &ending.description, errors);
        validate_text(world, &ending.epilogue, errors);
    }
}

fn validate_items_and_npcs(world: &World, errors: &mut Vec<ValidationError>) {
    for item in world.items.values() {
        validate_text(world, &item.short, errors);
        validate_text(world, &item.long, errors);
        if let Some(text) = &item.read {
            validate_text(world, text, errors);
        }
    }
    for (npc_id, npc) in &world.npcs {
        let ctx = format!("NPC '{npc_id}'");
        validate_text(world, &npc.description, errors);
        if let Some(dialogue) = &npc.dialogue
            && !world.dialogues.contains_key(dialogue)
        {
            errors.push(ValidationError::DialogueRefMissing {
                context: ctx.clone(),
                dialogue: dialogue.clone(),
            });
        }
        if let Some(room) = &npc.initial_room
            && !world.rooms.contains_key(room)
        {
            errors.push(ValidationError::RoomRefMissing {
                context: ctx,
                room: room.clone(),
            });
        }
    }
}

fn validate_timers(world: &World, errors: &mut Vec<ValidationError>) {
    for (timer_id, timer) in &world.timers {
        let ctx = format!("Timer '{timer_id}'");
        validate_effects(world, &ctx, &timer.on_tick, errors);
        validate_effects(world, &ctx, &timer.on_expire, errors);
        if let Some(cancel) = &timer.cancel_on {
            validate_condition(world, &ctx, cancel, errors);
        }
    }
}

fn check_room(world: &World, context: &str, room: &RoomId, errors: &mut Vec<ValidationError>) {
    if !world.rooms.contains_key(room) {
        errors.push(ValidationError::RoomRefMissing {
            context: context.to_string(),
            room: room.clone(),
        });
    }
}

fn check_item(world: &World, context: &str, item: &ItemId, errors: &mut Vec<ValidationError>) {
    if !world.items.contains_key(item) {
        errors.push(ValidationError::ItemRefMissing {
            context: context.to_string(),
            item: item.clone(),
        });
    }
}

fn check_npc(world: &World, context: &str, npc: &NpcId, errors: &mut Vec<ValidationError>) {
    if !world.npcs.contains_key(npc) {
        errors.push(ValidationError::NpcRefMissing {
            context: context.to_string(),
            npc: npc.clone(),
        });
    }
}

fn check_quest_stage(
    world: &World,
    quest: &QuestId,
    stage: &NodeId,
    errors: &mut Vec<ValidationError>,
) {
    match world.quests.get(quest) {
        Some(quest_def) => {
            if !quest_def.stages.contains_key(stage) {
                errors.push(ValidationError::QuestStageRefMissing {
                    quest: quest.clone(),
                    stage: stage.clone(),
                });
            }
        }
        None => {
            errors.push(ValidationError::QuestRefMissing {
                context: "Condition/Effect".to_string(),
                quest: quest.clone(),
            });
        }
    }
}

fn check_dialogue(
    world: &World,
    context: &str,
    dialogue: &DialogueId,
    errors: &mut Vec<ValidationError>,
) {
    if !world.dialogues.contains_key(dialogue) {
        errors.push(ValidationError::DialogueRefMissing {
            context: context.to_string(),
            dialogue: dialogue.clone(),
        });
    }
}

fn check_timer(world: &World, context: &str, timer: &TimerId, errors: &mut Vec<ValidationError>) {
    if !world.timers.contains_key(timer) {
        errors.push(ValidationError::TimerRefMissing {
            context: context.to_string(),
            timer: timer.clone(),
        });
    }
}

fn check_location(
    world: &World,
    context: &str,
    location: &ItemLocation,
    errors: &mut Vec<ValidationError>,
) {
    match location {
        ItemLocation::Room(room) => check_room(world, context, room, errors),
        ItemLocation::HeldBy(npc) => check_npc(world, context, npc, errors),
        ItemLocation::Inventory | ItemLocation::Nowhere => {}
    }
}
