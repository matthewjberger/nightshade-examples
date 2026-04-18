//! Choice assembly.
//!
//! Given the current `RuntimeState`, produce the menu of choices the player
//! can pick. Order:
//!
//! 1. If `pending_choices` is non-empty, use those (pure override).
//! 2. Otherwise, if a dialogue is active, expose its current node's options.
//! 3. Otherwise, build the standard room menu: exits, visible items,
//!    inventory interactions, NPC interactions, and the default utility
//!    actions (look / inventory / wait).

use crate::data::{
    Choice, ChoiceAction, ItemLocation, Npc, NpcId, Placeholder, Room, RuntimeState, Text,
    VerbResponses,
};
use crate::engine::{Engine, eval};

/// Assemble the full list of choices for the current turn.
pub fn assemble(engine: &Engine, state: &RuntimeState) -> Vec<Choice> {
    if state.game_over.is_some() {
        return Vec::new();
    }

    if !state.pending_choices.is_empty() {
        let filtered = filter_selectable(engine, state, &state.pending_choices);
        // If every offered choice is currently unselectable, the pending
        // override would trap the player. Fall through to the normal menu.
        if !filtered.is_empty() {
            return filtered;
        }
    }

    if state.active_dialogue.is_some() {
        return dialogue_choices(engine, state);
    }

    let mut choices = Vec::new();
    let Some(room) = engine.world().rooms.get(&state.current_room) else {
        return choices;
    };

    let dark = room.dark && !crate::engine::helpers::player_has_light(engine, state);

    let responses = &engine.world().verb_responses;
    append_exits(engine, state, room, &mut choices);
    if !dark {
        append_room_items(engine, state, &mut choices);
        append_npcs(engine, state, &mut choices);
        append_room_features(responses, room, &mut choices);
    }
    append_inventory_actions(engine, state, &mut choices);
    append_utility(responses, &mut choices);

    filter_selectable(engine, state, &choices)
}

fn append_exits(engine: &Engine, state: &RuntimeState, room: &Room, out: &mut Vec<Choice>) {
    let responses = &engine.world().verb_responses;
    for (index, exit) in room.exits.iter().enumerate() {
        if let Some(visibility) = &exit.visible_when
            && !eval::evaluate(engine, state, visibility)
        {
            continue;
        }

        let label_text =
            VerbResponses::render(&responses.choice_go, &[(Placeholder::Dir, &exit.direction)]);
        let action = ChoiceAction::Go {
            to: exit.to.clone(),
            exit_index: index,
        };
        let mut choice = Choice::new(Text::lit(label_text), action);
        if let Some(condition) = &exit.passable_when {
            choice.condition = Some(condition.clone());
            choice.visible_when_locked = true;
            choice.locked_reason = exit
                .locked_message
                .clone()
                .or_else(|| Some(Text::lit(responses.exit_locked_default.clone())));
        }
        out.push(choice);
    }
}

fn append_room_items(engine: &Engine, state: &RuntimeState, out: &mut Vec<Choice>) {
    let here = &state.current_room;
    let responses = &engine.world().verb_responses;
    for (item_id, item) in &engine.world().items {
        let here_match = matches!(
            state.item_locations.get(item_id),
            Some(ItemLocation::Room(room)) if room == here
        );
        if !here_match {
            continue;
        }
        if item.properties.takeable {
            out.push(Choice::new(
                Text::lit(VerbResponses::render(
                    &responses.choice_take,
                    &[(Placeholder::Item, &item.name)],
                )),
                ChoiceAction::Take(item_id.clone()),
            ));
        }
        out.push(Choice::new(
            Text::lit(VerbResponses::render(
                &responses.choice_examine_item,
                &[(Placeholder::Item, &item.name)],
            )),
            ChoiceAction::Examine(item_id.clone()),
        ));
    }
}

fn append_inventory_actions(engine: &Engine, state: &RuntimeState, out: &mut Vec<Choice>) {
    let responses = &engine.world().verb_responses;
    for (item_id, item) in &engine.world().items {
        if !matches!(
            state.item_locations.get(item_id),
            Some(ItemLocation::Inventory)
        ) {
            continue;
        }
        out.push(Choice::new(
            Text::lit(VerbResponses::render(
                &responses.choice_use,
                &[(Placeholder::Item, &item.name)],
            )),
            ChoiceAction::Use(item_id.clone()),
        ));
        if item.properties.readable {
            out.push(Choice::new(
                Text::lit(VerbResponses::render(
                    &responses.choice_read,
                    &[(Placeholder::Item, &item.name)],
                )),
                ChoiceAction::Read(item_id.clone()),
            ));
        }
        out.push(Choice::new(
            Text::lit(VerbResponses::render(
                &responses.choice_examine_item,
                &[(Placeholder::Item, &item.name)],
            )),
            ChoiceAction::Examine(item_id.clone()),
        ));
        out.push(Choice::new(
            Text::lit(VerbResponses::render(
                &responses.choice_drop,
                &[(Placeholder::Item, &item.name)],
            )),
            ChoiceAction::Drop(item_id.clone()),
        ));
    }
}

fn append_npcs(engine: &Engine, state: &RuntimeState, out: &mut Vec<Choice>) {
    let here = &state.current_room;
    let responses = &engine.world().verb_responses;
    let present: Vec<(NpcId, &Npc)> = state
        .npc_locations
        .iter()
        .filter(|(_, room)| *room == here)
        .filter_map(|(id, _)| engine.world().npcs.get(id).map(|npc| (id.clone(), npc)))
        .collect();
    for (npc_id, npc) in present {
        if npc.dialogue.is_some() {
            out.push(Choice::new(
                Text::lit(VerbResponses::render(
                    &responses.choice_talk,
                    &[(Placeholder::Npc, &npc.name)],
                )),
                ChoiceAction::TalkTo(npc_id),
            ));
        }
    }
}

fn append_room_features(responses: &VerbResponses, room: &Room, out: &mut Vec<Choice>) {
    for keyword in room.examine.keys() {
        out.push(Choice::new(
            Text::lit(VerbResponses::render(
                &responses.choice_examine_feature,
                &[(Placeholder::Keyword, keyword)],
            )),
            ChoiceAction::ExamineKeyword(keyword.clone()),
        ));
    }
}

fn append_utility(responses: &VerbResponses, out: &mut Vec<Choice>) {
    out.push(Choice::new(
        Text::lit(responses.choice_look.clone()),
        ChoiceAction::Look,
    ));
    out.push(Choice::new(
        Text::lit(responses.choice_inventory.clone()),
        ChoiceAction::Inventory,
    ));
    out.push(Choice::new(
        Text::lit(responses.choice_wait.clone()),
        ChoiceAction::Wait,
    ));
}

fn dialogue_choices(engine: &Engine, state: &RuntimeState) -> Vec<Choice> {
    let Some((dialogue_id, node_id)) = &state.active_dialogue else {
        return Vec::new();
    };
    let Some(dialogue) = engine.world().dialogues.get(dialogue_id) else {
        return Vec::new();
    };
    let Some(node) = dialogue.nodes.get(node_id) else {
        return Vec::new();
    };

    let mut choices = Vec::new();
    for (index, option) in node.options.iter().enumerate() {
        let action = ChoiceAction::DialogueOption(index);
        let mut choice = Choice::new(option.label.clone(), action);
        if let Some(condition) = &option.condition {
            choice.condition = Some(condition.clone());
            if option.visible_when_locked {
                choice.visible_when_locked = true;
                choice.locked_reason = option.locked_reason.clone();
            }
        }
        choices.push(choice);
    }
    if choices.is_empty() {
        choices.push(Choice::new(
            Text::lit(engine.world().verb_responses.choice_leave_dialogue.clone()),
            ChoiceAction::LeaveDialogue,
        ));
    }
    filter_selectable(engine, state, &choices)
}

/// Drop choices whose condition fails and whose `visible_when_locked` is false.
/// Keep greyed-out choices (they display their locked reason).
fn filter_selectable(engine: &Engine, state: &RuntimeState, choices: &[Choice]) -> Vec<Choice> {
    choices
        .iter()
        .filter(|choice| match &choice.condition {
            None => true,
            Some(condition) => {
                eval::evaluate(engine, state, condition) || choice.visible_when_locked
            }
        })
        .cloned()
        .collect()
}

/// Whether a choice in the assembled list is currently actionable. Used by
/// the turn loop to reject clicks on locked entries.
pub fn is_actionable(engine: &Engine, state: &RuntimeState, choice: &Choice) -> bool {
    match &choice.condition {
        None => true,
        Some(condition) => eval::evaluate(engine, state, condition),
    }
}
