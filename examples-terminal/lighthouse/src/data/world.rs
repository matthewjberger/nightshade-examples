//! The immutable authored content.
//!
//! Everything the engine interprets lives here. The engine holds a shared
//! reference to a `World` for the lifetime of a run and only ever mutates a
//! `RuntimeState`.

use crate::data::condition::Condition;
use crate::data::dialogue::Dialogue;
use crate::data::ending::Ending;
use crate::data::ids::{
    ConditionId, DialogueId, EndingId, ItemId, NpcId, QuestId, RoomId, RuleId, TextId, TimerId,
};
use crate::data::item::Item;
use crate::data::npc::Npc;
use crate::data::quest::Quest;
use crate::data::room::Room;
use crate::data::rule::Rule;
use crate::data::text::Text;
use crate::data::timer::Timer;
use crate::data::verb_responses::VerbResponses;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The authored world.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct World {
    /// Human-readable title, shown in the window title and opening banner.
    pub title: String,
    /// Opening text shown before the first room.
    pub intro: Text,
    /// Where the player starts.
    pub start_room: RoomId,

    pub rooms: BTreeMap<RoomId, Room>,
    pub items: BTreeMap<ItemId, Item>,
    pub npcs: BTreeMap<NpcId, Npc>,
    pub dialogues: BTreeMap<DialogueId, Dialogue>,
    pub quests: BTreeMap<QuestId, Quest>,
    pub endings: BTreeMap<EndingId, Ending>,
    pub rules: BTreeMap<RuleId, Rule>,
    pub timers: BTreeMap<TimerId, Timer>,

    /// Shared text table; `Text::Ref(id)` resolves here.
    pub texts: BTreeMap<TextId, Text>,
    /// Shared condition table; `Condition::Ref(id)` resolves here.
    pub conditions: BTreeMap<ConditionId, Condition>,

    /// Every baked string the engine emits on its own (take/drop/wait
    /// responses, room headers, choice-menu labels, the rule-tracing
    /// prefix). Defaults are English; override any field to change or
    /// translate what the engine prints.
    #[serde(default)]
    pub verb_responses: VerbResponses,
}
