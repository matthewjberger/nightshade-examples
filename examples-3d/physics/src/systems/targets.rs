mod combat;
mod effects;
mod spawn;

pub(crate) use combat::{process_target_killed_events, update_targets};
pub(crate) use spawn::spawn_targets;
