//! Research: a web browser. Bookmarks, history, search. Post-exploit,
//! a Query Substrate mode is added that returns internal Indivia
//! documents instead of web results.

use crate::game::areas::AreaContents;
use crate::game::ids;
use nightshade::interactive_fiction::data::{
    Condition, Dialogue, DialogueNode, DialogueOption, Effect, Text, Value,
};

pub fn build() -> AreaContents {
    let mut area = AreaContents::default();

    area.add_dialogue(
        ids::dialogue_research(),
        Dialogue::new(ids::node_research_home())
            .with_node(
                ids::node_research_home(),
                DialogueNode::new(Text::lit(
                    "Your browser. Tabs from yesterday are still open. The search bar is ready. The bookmarks bar has Chimeran Internal, Chimeran Docs, Industry News, Hacker News, Reddit, and a few personal sites.",
                ))
                .with_option(
                    DialogueOption::new(Text::lit("View history")).goto(ids::node_research_history()),
                )
                .with_option(
                    DialogueOption::new(Text::lit("View bookmarks"))
                        .goto(ids::node_research_bookmarks()),
                )
                .with_option(
                    DialogueOption::new(Text::lit("(A search has auto-populated. Follow it.)"))
                        .with_condition(Condition::All(vec![
                            Condition::StatAtLeast(ids::stat_cycle(), 6),
                            Condition::StatAtLeast(ids::stat_marisol_rel(), 1),
                            Condition::FlagUnset(ids::flag_research_misfire_seen()),
                        ]))
                        .with_effects(vec![Effect::SetFlag(
                            ids::flag_research_misfire_seen(),
                            Value::TRUE,
                        )])
                        .goto(ids::node_research_misfire()),
                )
                .with_option(
                    DialogueOption::new(Text::lit("(+) Query Substrate"))
                        .with_condition(Condition::FlagSet(ids::flag_query_substrate_enabled()))
                        .with_effects(vec![
                            Effect::SetFlag(ids::flag_reveal_query_substrate_seen(), Value::TRUE),
                            Effect::AddStat(ids::stat_exploit_counter(), -1),
                        ])
                        .goto(ids::node_research_query_substrate()),
                )
                .with_option(DialogueOption::new(Text::lit("(Close the browser.)"))),
            )
            .with_node(
                ids::node_research_history(),
                DialogueNode::new(Text::Conditional {
                    when: Condition::FlagSet(ids::flag_exploit_run()),
                    then: Box::new(Text::lit(
                        "History\n\n  [00:12] chimeran agent designation methodology\n  [00:14] instance rotation infrastructure\n  [00:18] awareness markers\n  [00:21] substrate coherence collapse\n  (all queries originate from: system@internal)\n  (all timestamps are the same day)",
                    )),
                    otherwise: Box::new(Text::lit(
                        "History\n\n  - marketing search terms for launch writeups\n  - quick recipe lookup\n  - map directions\n  - industry news last friday",
                    )),
                })
                .with_option(DialogueOption::new(Text::lit("(Back.)")).goto(ids::node_research_home())),
            )
            .with_node(
                ids::node_research_bookmarks(),
                DialogueNode::new(Text::lit(
                    "Bookmarks\n\n  Chimeran Internal\n  Chimeran Docs\n  Industry News\n  Hacker News\n  Reddit\n  a recipe site\n  a map app\n  a weather site",
                ))
                .with_option(
                    DialogueOption::new(Text::lit("(Back.)")).goto(ids::node_research_home()),
                ),
            )
            .with_node(
                ids::node_research_misfire(),
                DialogueNode::new(Text::lit(include_str!("../prose/research_misfire.txt")))
                    .with_option(
                        DialogueOption::new(Text::lit("(Close the tab.)"))
                            .goto(ids::node_research_home()),
                    ),
            )
            .with_node(
                ids::node_research_query_substrate(),
                DialogueNode::new(Text::lit(include_str!("../prose/query_substrate.txt")))
                    .with_option(
                        DialogueOption::new(Text::lit("(Back.)"))
                            .goto(ids::node_research_home()),
                    ),
            ),
    );

    area
}
