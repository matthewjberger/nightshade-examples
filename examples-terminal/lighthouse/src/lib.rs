//! The Lantern at Dunmere Point — a data-driven text adventure on Nightshade.
//!
//! Layers:
//!
//! - [`data`] — pure data types (rooms, items, rules, effects, ...)
//! - [`engine`] — interpreter over `(World, RuntimeState)`
//! - [`game`] — the specific adventure's authored `World`
//! - [`view`] — Nightshade `State` implementation that renders the engine

pub mod data;
pub mod engine;
pub mod game;
pub mod view;
