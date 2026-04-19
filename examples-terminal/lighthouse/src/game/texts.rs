//! Shared text table entries referenced via `Text::Ref`.

use crate::game::ids;
use nightshade::interactive_fiction::data::{Text, TextId};
use std::collections::BTreeMap;

pub fn build() -> BTreeMap<TextId, Text> {
    let mut texts = BTreeMap::new();
    texts.insert(
        ids::text_intro(),
        Text::Sequence(vec![
            Text::lit(
                "Salt wind bites your face. The gulls are silent tonight, gone inland with the tide.",
            ),
            Text::lit(
                " Above the headland, Dunmere Light stands dark for the first time in forty years.",
            ),
            Text::lit(
                " The storm is still a smudge on the horizon, but it is coming, and soon.",
            ),
        ]),
    );
    texts.insert(
        ids::text_storm_close(),
        Text::lit("Rain hammers sideways. The headland shakes with thunder."),
    );
    texts.insert(
        ids::text_storm_far(),
        Text::lit("Far out past the cliffs, the horizon flickers with lightning."),
    );
    texts
}
