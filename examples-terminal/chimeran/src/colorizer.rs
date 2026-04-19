use nightshade::interactive_fiction::engine::Engine;
use nightshade::tui::prelude::TermColor;

pub const ITEM_COLOR: TermColor = TermColor::Yellow;
pub const NPC_COLOR: TermColor = TermColor::Green;
pub const ROOM_COLOR: TermColor = TermColor::Cyan;
pub const DIRECTION_COLOR: TermColor = TermColor::Blue;
pub const VERB_COLOR: TermColor = TermColor::Magenta;

const HINTABLE_VERBS: &[&str] = &[
    "take",
    "drop",
    "drink",
    "eat",
    "examine",
    "look",
    "open",
    "close",
    "read",
    "use",
    "wear",
    "give",
    "push",
    "pull",
    "turn",
    "smell",
    "taste",
    "touch",
    "listen",
    "wait",
    "sleep",
    "wake",
    "inventory",
];

const VERB_NOUN_DISQUALIFIERS: &[&str] = &[
    "a", "an", "the", "or", "and", "of", "on", "in", "as", "to", "by", "for", "some", "any", "no",
    "my", "your", "his", "her", "their", "this", "that", "these", "those", "with",
];

pub struct Keyword {
    pub lower: Vec<char>,
    pub color: TermColor,
}

impl Keyword {
    pub fn new(word: &str, color: TermColor) -> Self {
        Self {
            lower: word.to_lowercase().chars().collect(),
            color,
        }
    }
}

pub type ColoredLine = Vec<(char, TermColor)>;

pub fn build_keywords(engine: &Engine) -> Vec<Keyword> {
    let mut keywords: Vec<Keyword> = Vec::new();

    for item in engine.world().items.values() {
        keywords.push(Keyword::new(&item.name, ITEM_COLOR));
        for synonym in &item.synonyms {
            keywords.push(Keyword::new(synonym, ITEM_COLOR));
        }
    }
    for entity in engine.world().entities.values() {
        keywords.push(Keyword::new(&entity.name, NPC_COLOR));
        for synonym in &entity.synonyms {
            keywords.push(Keyword::new(synonym, NPC_COLOR));
        }
    }
    for room in engine.world().rooms.values() {
        keywords.push(Keyword::new(&room.name, ROOM_COLOR));
    }
    for direction in [
        "north",
        "south",
        "east",
        "west",
        "up",
        "down",
        "northeast",
        "northwest",
        "southeast",
        "southwest",
    ] {
        keywords.push(Keyword::new(direction, DIRECTION_COLOR));
    }
    for verb in HINTABLE_VERBS {
        keywords.push(Keyword::new(verb, VERB_COLOR));
    }

    keywords.sort_by_key(|keyword| std::cmp::Reverse(keyword.lower.len()));
    keywords.retain(|keyword| keyword.lower.len() >= 2);
    keywords
}

pub fn colorize(line: &str, default: TermColor, keywords: &[Keyword]) -> ColoredLine {
    let chars: Vec<char> = line.chars().collect();
    let lower: Vec<char> = chars.iter().map(|c| c.to_ascii_lowercase()).collect();
    let mut colors: Vec<TermColor> = vec![default; chars.len()];

    let mut position = 0;
    while position < chars.len() {
        let mut advanced = false;
        for keyword in keywords {
            let length = keyword.lower.len();
            if position + length > chars.len() {
                continue;
            }
            if lower[position..position + length] != keyword.lower[..] {
                continue;
            }
            let before_ok = position == 0 || !chars[position - 1].is_alphanumeric();
            let after_ok =
                position + length == chars.len() || !chars[position + length].is_alphanumeric();
            if !(before_ok && after_ok) {
                continue;
            }
            if keyword.color == VERB_COLOR && preceded_by_disqualifier(&lower, position) {
                continue;
            }
            for color in colors.iter_mut().skip(position).take(length) {
                *color = keyword.color;
            }
            position += length;
            advanced = true;
            break;
        }
        if !advanced {
            position += 1;
        }
    }

    chars.into_iter().zip(colors).collect()
}

fn preceded_by_disqualifier(lower: &[char], position: usize) -> bool {
    let mut idx = position;
    while idx > 0 && !lower[idx - 1].is_alphanumeric() {
        idx -= 1;
    }
    let end = idx;
    while idx > 0 && lower[idx - 1].is_alphanumeric() {
        idx -= 1;
    }
    if idx == end {
        return false;
    }
    let prev: String = lower[idx..end].iter().collect();
    VERB_NOUN_DISQUALIFIERS.iter().any(|w| *w == prev)
}
