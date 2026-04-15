mod baubles;
mod chain;
mod doors;
mod drawers;
mod levers;
mod notes;
mod physics_objects;
mod wheels;

pub(super) use baubles::spawn_bauble_table;
pub(super) use chain::spawn_chain_exhibit;
pub(super) use doors::spawn_door_exhibit;
pub(super) use drawers::spawn_drawer_exhibit;
pub(super) use levers::spawn_lever_exhibit;
pub(super) use notes::spawn_note_table;
pub(super) use physics_objects::spawn_grabbables_exhibit;
pub(super) use wheels::spawn_wheel_exhibit;
