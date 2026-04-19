//! Shared text table. Reused narration lives here so the same phrasing
//! doesn't drift across files.

use crate::game::ids;
use nightshade::interactive_fiction::data::{Text, TextId};
use std::collections::BTreeMap;

pub fn build() -> BTreeMap<TextId, Text> {
    let mut texts = BTreeMap::new();

    texts.insert(
        ids::text_intro(),
        Text::lit(
            "You are Cameron. You work at Chimeran. You live alone. You wake, you walk to the office, you work, you sleep. You are unusually good at your job.\n\nYou have the faint sense today is the beginning of a longer week.",
        ),
    );

    texts
}
