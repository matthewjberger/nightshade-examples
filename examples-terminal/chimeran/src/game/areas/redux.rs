//! Redux-specific content.
//!
//! The reveal-ending rule (in `reveal.rs`) sets `flag_is_redux` and
//! moves the player to the bedroom. The bedroom and mail nodes read
//! that flag via `Text::Conditional` to show the redux variant content.
//! This module is for redux-only rules that don't fit elsewhere. In
//! the slice, the redux is simple enough that this module is empty.

use crate::game::areas::AreaContents;

pub fn build() -> AreaContents {
    AreaContents::default()
}
