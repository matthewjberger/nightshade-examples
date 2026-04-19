//! Centralized ID constructors for the adventure's content.
//!
//! Every room, item, NPC, etc. referenced anywhere in `crate::game` should go
//! through a function here. This is the single place to rename a room ID; the
//! validator catches typos from string drift.

use nightshade::interactive_fiction::data::{
    ConditionId, DialogueId, EndingId, EntityId, EventName, FlagKey, ItemId, NodeId, QuestId,
    RoomId, RuleId, TextId, TimerId,
};

// Rooms -----------------------------------------------------------------
pub fn room_shore() -> RoomId {
    RoomId::new("shore")
}
pub fn room_cottage() -> RoomId {
    RoomId::new("cottage")
}
pub fn room_tower_base() -> RoomId {
    RoomId::new("tower_base")
}
pub fn room_tower_stairs() -> RoomId {
    RoomId::new("tower_stairs")
}
pub fn room_lantern_room() -> RoomId {
    RoomId::new("lantern_room")
}
pub fn room_cellar() -> RoomId {
    RoomId::new("cellar")
}
pub fn room_cliff_path() -> RoomId {
    RoomId::new("cliff_path")
}
pub fn room_gone() -> RoomId {
    RoomId::new("gone")
}

// Items -----------------------------------------------------------------
pub fn item_driftwood() -> ItemId {
    ItemId::new("driftwood")
}
pub fn item_cottage_key() -> ItemId {
    ItemId::new("cottage_key")
}
pub fn item_lantern() -> ItemId {
    ItemId::new("lantern")
}
pub fn item_tinderbox() -> ItemId {
    ItemId::new("tinderbox")
}
pub fn item_keeper_log() -> ItemId {
    ItemId::new("keeper_log")
}
pub fn item_oil_can() -> ItemId {
    ItemId::new("oil_can")
}
pub fn item_tower_key() -> ItemId {
    ItemId::new("tower_key")
}
pub fn item_ledger() -> ItemId {
    ItemId::new("ledger")
}
pub fn item_wreckers_note() -> ItemId {
    ItemId::new("wreckers_note")
}
pub fn item_rope() -> ItemId {
    ItemId::new("rope")
}
pub fn item_keeper_remains() -> ItemId {
    ItemId::new("keeper_remains")
}

// NPCs ------------------------------------------------------------------
pub fn npc_stranger() -> EntityId {
    EntityId::new("stranger")
}

// Dialogues -------------------------------------------------------------
pub fn dialogue_stranger() -> DialogueId {
    DialogueId::new("stranger")
}
pub fn node_intro() -> NodeId {
    NodeId::new("intro")
}
pub fn node_offer() -> NodeId {
    NodeId::new("offer")
}
pub fn node_accepted() -> NodeId {
    NodeId::new("accepted")
}
pub fn node_refused() -> NodeId {
    NodeId::new("refused")
}
pub fn node_confront() -> NodeId {
    NodeId::new("confront")
}

// Quests ----------------------------------------------------------------
pub fn quest_lantern() -> QuestId {
    QuestId::new("lantern")
}
pub fn stage_begin() -> NodeId {
    NodeId::new("begin")
}
pub fn stage_unlocked_cottage() -> NodeId {
    NodeId::new("unlocked_cottage")
}
pub fn stage_found_keeper() -> NodeId {
    NodeId::new("found_keeper")
}
pub fn stage_restored() -> NodeId {
    NodeId::new("restored")
}
pub fn stage_sabotaged() -> NodeId {
    NodeId::new("sabotaged")
}
pub fn stage_abandoned() -> NodeId {
    NodeId::new("abandoned")
}

// Endings ---------------------------------------------------------------
pub fn ending_the_lantern_burns() -> EndingId {
    EndingId::new("the_lantern_burns")
}
pub fn ending_the_wreckers_gold() -> EndingId {
    EndingId::new("the_wreckers_gold")
}
pub fn ending_safe_ashore() -> EndingId {
    EndingId::new("safe_ashore")
}
pub fn ending_lost_to_the_storm() -> EndingId {
    EndingId::new("lost_to_the_storm")
}

// Rules -----------------------------------------------------------------
pub fn rule_kickoff() -> RuleId {
    RuleId::new("kickoff")
}
pub fn rule_cottage_unlocked() -> RuleId {
    RuleId::new("cottage_unlocked")
}
pub fn rule_light_lantern() -> RuleId {
    RuleId::new("light_lantern")
}
pub fn rule_drop_cottage_key() -> RuleId {
    RuleId::new("drop_cottage_key")
}
pub fn rule_tower_unlocked() -> RuleId {
    RuleId::new("tower_unlocked")
}
pub fn rule_oil_applied() -> RuleId {
    RuleId::new("oil_applied")
}
pub fn rule_relight_lantern() -> RuleId {
    RuleId::new("relight_lantern")
}
pub fn rule_sabotage_lantern() -> RuleId {
    RuleId::new("sabotage_lantern")
}
pub fn rule_hidden_passage() -> RuleId {
    RuleId::new("hidden_passage")
}
pub fn rule_first_stairs() -> RuleId {
    RuleId::new("first_stairs")
}
pub fn rule_found_keeper() -> RuleId {
    RuleId::new("found_keeper")
}
pub fn rule_log_examined() -> RuleId {
    RuleId::new("log_examined")
}
pub fn rule_stranger_arrives_event() -> RuleId {
    RuleId::new("stranger_arrives_event")
}
pub fn rule_stranger_moves() -> RuleId {
    RuleId::new("stranger_moves")
}
pub fn rule_storm_whisper() -> RuleId {
    RuleId::new("storm_whisper")
}
pub fn rule_storm_whisper_slow() -> RuleId {
    RuleId::new("storm_whisper_slow")
}
pub fn rule_read_log_hint() -> RuleId {
    RuleId::new("read_log_hint")
}
pub fn rule_take_note_flavor() -> RuleId {
    RuleId::new("take_note_flavor")
}

// Timers ----------------------------------------------------------------
pub fn timer_storm() -> TimerId {
    TimerId::new("storm")
}
pub fn timer_stranger_arrival() -> TimerId {
    TimerId::new("stranger_arrival")
}

// Flags -----------------------------------------------------------------
pub fn flag_cottage_unlocked() -> FlagKey {
    FlagKey::new("cottage_unlocked")
}
pub fn flag_tower_unlocked() -> FlagKey {
    FlagKey::new("tower_unlocked")
}
pub fn flag_lens_oiled() -> FlagKey {
    FlagKey::new("lens_oiled")
}
pub fn flag_found_keeper() -> FlagKey {
    FlagKey::new("found_keeper")
}
pub fn flag_lantern_restored() -> FlagKey {
    FlagKey::new("lantern_restored")
}
pub fn flag_lantern_sabotaged() -> FlagKey {
    FlagKey::new("lantern_sabotaged")
}
pub fn flag_offer_accepted() -> FlagKey {
    FlagKey::new("wrecker_offer_accepted")
}
pub fn flag_offer_refused() -> FlagKey {
    FlagKey::new("wrecker_offer_refused")
}
pub fn flag_hidden_passage() -> FlagKey {
    FlagKey::new("hidden_passage_open")
}
pub fn flag_intro_shown() -> FlagKey {
    FlagKey::new("intro_shown")
}
pub fn flag_lantern_is_lit() -> FlagKey {
    FlagKey::new("lantern_is_lit")
}
pub fn flag_stranger_has_arrived() -> FlagKey {
    FlagKey::new("stranger_has_arrived")
}
pub fn flag_read_second_ledger() -> FlagKey {
    FlagKey::new("read_second_ledger")
}

// Event names -----------------------------------------------------------
pub fn event_stranger_arrives() -> EventName {
    EventName::new("stranger_arrives")
}
pub fn event_lantern_restored() -> EventName {
    EventName::new("lantern_restored")
}

// Shared texts ----------------------------------------------------------
pub fn text_intro() -> TextId {
    TextId::new("intro")
}
pub fn text_storm_close() -> TextId {
    TextId::new("storm_close")
}
pub fn text_storm_far() -> TextId {
    TextId::new("storm_far")
}

// Shared conditions -----------------------------------------------------
pub fn cond_has_lit_lantern() -> ConditionId {
    ConditionId::new("has_lit_lantern")
}
