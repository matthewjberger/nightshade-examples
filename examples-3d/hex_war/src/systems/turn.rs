use crate::constants::ACTIONS_PER_TURN;
use crate::ecs::{ActionRecord, FACTION_COUNT, Faction, GameEvents, GameWorld, MOVEMENT, UNIT};
use crate::selection::clear_selection;
use crate::systems::{PendingSpawn, build_turn_order, reinforcement_system};

pub struct TurnTransition {
    pub new_faction: Faction,
    pub turn_number: u32,
    pub pending_spawns: Vec<PendingSpawn>,
}

pub fn end_turn(game_world: &mut GameWorld, events: &mut GameEvents) -> TurnTransition {
    clear_selection(game_world);

    for entity in game_world.query_entities(UNIT).collect::<Vec<_>>() {
        if let Some(unit) = game_world.get_unit(entity) {
            let mut unit = *unit;
            unit.has_moved = false;
            game_world.set_unit(entity, unit);
        }
    }

    let mut next = game_world.resources.current_faction.next();
    let mut attempts = 0;

    while attempts < FACTION_COUNT {
        if !game_world.resources.faction_eliminated[next.index()] {
            break;
        }

        next = next.next();
        attempts += 1;
    }

    if next == Faction::Redosia {
        game_world.resources.turn_number += 1;
    }

    game_world.resources.current_faction = next;
    game_world.resources.actions_remaining = ACTIONS_PER_TURN;
    game_world.resources.speech_used = false;

    build_turn_order(game_world);

    events.action_history.push(ActionRecord {
        faction: next,
        turn: game_world.resources.turn_number,
        description: format!("Turn {} begins", game_world.resources.turn_number),
    });

    let pending_spawns = reinforcement_system(game_world, events);

    TurnTransition {
        new_faction: next,
        turn_number: game_world.resources.turn_number,
        pending_spawns,
    }
}

pub fn can_end_turn(game_world: &GameWorld) -> bool {
    game_world.query_entities(MOVEMENT).next().is_none()
}
