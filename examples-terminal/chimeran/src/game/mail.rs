use crate::game::{ids, plan};
use nightshade::interactive_fiction::data::{RuntimeState, Value};

pub fn unread_count(state: &RuntimeState) -> usize {
    let cycle = state.stats.get(&ids::stat_cycle()).copied().unwrap_or(0);
    let is_redux = matches!(
        state.flags.get(&ids::flag_is_redux()),
        Some(Value::Bool(true))
    );
    let flag_set = |key: &nightshade::interactive_fiction::data::FlagKey| -> bool {
        matches!(state.flags.get(key), Some(Value::Bool(true)))
    };

    let rachel_tag: Option<&str> = if is_redux {
        Some("redux")
    } else {
        match cycle {
            1 => Some("c1"),
            2 => Some("c2"),
            3 => Some("c3"),
            4 => Some("c4"),
            5 => Some("c5"),
            6 => Some("c6"),
            7 => Some("c7"),
            _ => None,
        }
    };

    let mut unread = 0;
    if let Some(tag) = rachel_tag
        && !flag_set(&ids::flag_rachel_archived(tag))
    {
        unread += 1;
    }

    if !is_redux {
        for tag in plan::cycle_requests(cycle) {
            if !flag_set(&ids::flag_req_submitted(tag)) {
                unread += 1;
            }
        }
    }

    unread
}
