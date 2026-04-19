use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueNode, DialogueOption, Text,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_dialogue(
        ids::dialogue_translator(),
        Dialogue::new(ids::node_translator_home()).with_node(
            ids::node_translator_home(),
            DialogueNode::new(Text::Conditional {
                when: Condition::StatAtLeast(ids::stat_cycle(), 8),
                then: Box::new(Text::lit(
                    "The translator. You type a short phrase into the source panel. The target panel returns it in the language you typed it in. You did not set the target to English. You try again. The source panel clears before you finish typing; the history at the bottom shows your entry in a language you do not read, attributed to an account you do not have.",
                )),
                otherwise: Box::new(Text::Conditional {
                    when: Condition::StatAtLeast(ids::stat_cycle(), 7),
                    then: Box::new(Text::lit(
                        "The translator. Source panel. Target panel. A history of previous translations at the bottom. You try a short phrase. The target panel stalls. It resolves. The text is subtly different from what you meant. The phrase you typed is no longer in the source panel; you did not delete it.",
                    )),
                    otherwise: Box::new(Text::lit(
                        "The translator. Source panel on the left. Target panel on the right. A history of previous translations. Source language auto-detects. Target language is a dropdown.",
                    )),
                }),
            })
            .with_option(DialogueOption::new(Text::lit("(Close the translator.)"))),
        ),
    );

    area
}
