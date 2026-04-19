use crate::game::ids;
use nightshade::interactive_fiction::data::{Text, TextId};
use std::collections::BTreeMap;

pub fn build() -> BTreeMap<TextId, Text> {
    let mut texts = BTreeMap::new();

    texts.insert(
        ids::text_intro(),
        Text::lit(
            "The alarm buzzes at 6:47. The coffee in the kitchen is already the temperature it was yesterday. Morning light sits on the window the way it always sits — even, unshifting.\n\nYou are Cameron. You work at Chimeran. You live alone. You wake, you walk to the office, you work, you sleep. You are unusually good at your job.\n\nYou have the sense of a longer week. You have the sense of having had this sense before.",
        ),
    );

    texts
}
