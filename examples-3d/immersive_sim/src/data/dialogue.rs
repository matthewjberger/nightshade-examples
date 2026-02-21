use crate::state::{DialogueChoice, DialogueLine, DialogueNode};

pub fn get_dialogue_tree(dialogue_id: usize) -> Vec<DialogueNode> {
    match dialogue_id {
        0 => guard_dialogue(),
        1 => merchant_dialogue(),
        2 => scholar_dialogue(),
        _ => default_dialogue(),
    }
}

fn guard_dialogue() -> Vec<DialogueNode> {
    vec![
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Guard".to_string(),
                    text: "Halt! State your business here.".to_string(),
                },
                DialogueLine {
                    speaker: "Guard".to_string(),
                    text: "We don't get many visitors in these parts.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "I'm just passing through.".to_string(),
                    next_node: Some(1),
                },
                DialogueChoice {
                    text: "Tell me about this area.".to_string(),
                    next_node: Some(2),
                },
                DialogueChoice {
                    text: "Goodbye.".to_string(),
                    next_node: None,
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Guard".to_string(),
                    text: "Just passing through, eh? Well, keep your nose clean.".to_string(),
                },
                DialogueLine {
                    speaker: "Guard".to_string(),
                    text: "The roads ahead can be dangerous. Watch yourself.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "Thanks for the warning.".to_string(),
                    next_node: None,
                },
                DialogueChoice {
                    text: "What kind of dangers?".to_string(),
                    next_node: Some(3),
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Guard".to_string(),
                    text: "This area? It's been peaceful lately, thankfully.".to_string(),
                },
                DialogueLine {
                    speaker: "Guard".to_string(),
                    text: "There's a merchant nearby if you need supplies. And a scholar who knows much about the old ruins.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "I'll check them out. Thanks.".to_string(),
                    next_node: None,
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Guard".to_string(),
                    text: "Bandits, mostly. Sometimes worse things come down from the mountains.".to_string(),
                },
                DialogueLine {
                    speaker: "Guard".to_string(),
                    text: "But as long as you stay on the main paths, you should be fine.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "I'll be careful. Farewell.".to_string(),
                    next_node: None,
                },
            ],
        },
    ]
}

fn merchant_dialogue() -> Vec<DialogueNode> {
    vec![
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Merchant".to_string(),
                    text: "Welcome, welcome! Looking to trade?".to_string(),
                },
                DialogueLine {
                    speaker: "Merchant".to_string(),
                    text: "I've got the finest wares this side of the mountains!".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "What do you have for sale?".to_string(),
                    next_node: Some(1),
                },
                DialogueChoice {
                    text: "Tell me about yourself.".to_string(),
                    next_node: Some(2),
                },
                DialogueChoice {
                    text: "Not interested. Goodbye.".to_string(),
                    next_node: None,
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Merchant".to_string(),
                    text: "I deal in all manner of goods - potions, provisions, equipment...".to_string(),
                },
                DialogueLine {
                    speaker: "Merchant".to_string(),
                    text: "Though I must admit, business has been slow lately. Not many travelers these days.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "Why is business slow?".to_string(),
                    next_node: Some(3),
                },
                DialogueChoice {
                    text: "I'll take a look around.".to_string(),
                    next_node: None,
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Merchant".to_string(),
                    text: "Me? I've been trading these roads for twenty years now.".to_string(),
                },
                DialogueLine {
                    speaker: "Merchant".to_string(),
                    text: "Started with just a cart and a dream. Now look at me - still just a cart and a dream!".to_string(),
                },
                DialogueLine {
                    speaker: "Merchant".to_string(),
                    text: "Hah! But seriously, it's an honest living.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "You have a good sense of humor.".to_string(),
                    next_node: None,
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Merchant".to_string(),
                    text: "Rumors, mostly. People talking about strange happenings in the old ruins.".to_string(),
                },
                DialogueLine {
                    speaker: "Merchant".to_string(),
                    text: "The scholar might know more. He's been studying those ruins for years.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "I'll ask the scholar about it.".to_string(),
                    next_node: None,
                },
            ],
        },
    ]
}

fn scholar_dialogue() -> Vec<DialogueNode> {
    vec![
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "Ah, a visitor! Please, don't mind the mess.".to_string(),
                },
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "I've been immersed in my research. The ancient texts speak of fascinating things...".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "What are you researching?".to_string(),
                    next_node: Some(1),
                },
                DialogueChoice {
                    text: "What can you tell me about the ruins?".to_string(),
                    next_node: Some(2),
                },
                DialogueChoice {
                    text: "I should go.".to_string(),
                    next_node: None,
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "The civilization that once thrived here! They were remarkably advanced.".to_string(),
                },
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "Their architecture, their knowledge... all lost to time. Or so we thought.".to_string(),
                },
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "I believe there are still secrets waiting to be uncovered in the ruins.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "What kind of secrets?".to_string(),
                    next_node: Some(3),
                },
                DialogueChoice {
                    text: "Fascinating. I'll leave you to your work.".to_string(),
                    next_node: None,
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "The ruins to the north? They date back thousands of years.".to_string(),
                },
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "Once a great temple, I believe. The inscriptions are difficult to decipher.".to_string(),
                },
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "Some say the temple was built to house something powerful. Something ancient.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "Something powerful? Like what?".to_string(),
                    next_node: Some(3),
                },
                DialogueChoice {
                    text: "Sounds like superstition to me.".to_string(),
                    next_node: Some(4),
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "The texts mention an artifact of great power. A relic of the old gods, they called it.".to_string(),
                },
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "Whether it truly exists... well, that's what I'm trying to determine.".to_string(),
                },
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "If you ever venture into the ruins, do let me know what you find!".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "I'll keep that in mind.".to_string(),
                    next_node: None,
                },
            ],
        },
        DialogueNode {
            lines: vec![
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "Perhaps. But many legends have a kernel of truth.".to_string(),
                },
                DialogueLine {
                    speaker: "Scholar".to_string(),
                    text: "The ancients knew things we have forgotten. I intend to rediscover that knowledge.".to_string(),
                },
            ],
            choices: vec![
                DialogueChoice {
                    text: "Good luck with your research.".to_string(),
                    next_node: None,
                },
            ],
        },
    ]
}

fn default_dialogue() -> Vec<DialogueNode> {
    vec![DialogueNode {
        lines: vec![DialogueLine {
            speaker: "Stranger".to_string(),
            text: "...".to_string(),
        }],
        choices: vec![DialogueChoice {
            text: "Goodbye.".to_string(),
            next_node: None,
        }],
    }]
}
