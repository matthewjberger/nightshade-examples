use nightshade::interactive_fiction::data::{
    DialogueId, EndingId, EntityId, EventName, FlagKey, ItemId, NodeId, RoomId, RuleId, StatKey,
    TextId,
};

pub fn room_bedroom() -> RoomId {
    RoomId::new("bedroom")
}
pub fn room_hallway() -> RoomId {
    RoomId::new("hallway")
}
pub fn room_kitchen() -> RoomId {
    RoomId::new("kitchen")
}
pub fn room_building_corridor() -> RoomId {
    RoomId::new("building_corridor")
}
pub fn room_elevator() -> RoomId {
    RoomId::new("elevator")
}
pub fn room_lobby() -> RoomId {
    RoomId::new("lobby")
}
pub fn room_street() -> RoomId {
    RoomId::new("street")
}
pub fn room_office_floor() -> RoomId {
    RoomId::new("office_floor")
}
pub fn room_desk() -> RoomId {
    RoomId::new("desk")
}
pub fn room_endgame() -> RoomId {
    RoomId::new("endgame")
}

pub fn item_coffee_mug() -> ItemId {
    ItemId::new("coffee_mug")
}
pub fn item_sticky_note_monitor() -> ItemId {
    ItemId::new("sticky_note_monitor")
}
pub fn item_sticky_note_hallway() -> ItemId {
    ItemId::new("sticky_note_hallway")
}
pub fn item_sticky_note_redux() -> ItemId {
    ItemId::new("sticky_note_redux")
}

pub fn fixture_mail() -> EntityId {
    EntityId::new("mail")
}
pub fn fixture_notepad() -> EntityId {
    EntityId::new("notepad")
}
pub fn fixture_research() -> EntityId {
    EntityId::new("research")
}
pub fn fixture_translator() -> EntityId {
    EntityId::new("translator")
}
pub fn fixture_code() -> EntityId {
    EntityId::new("code")
}
pub fn fixture_reference() -> EntityId {
    EntityId::new("reference")
}
pub fn fixture_chatter() -> EntityId {
    EntityId::new("chatter")
}
pub fn fixture_mirror() -> EntityId {
    EntityId::new("mirror")
}
pub fn fixture_bed() -> EntityId {
    EntityId::new("bed")
}
pub fn fixture_picture_frame() -> EntityId {
    EntityId::new("picture_frame")
}
pub fn fixture_clock() -> EntityId {
    EntityId::new("clock")
}
pub fn fixture_commute() -> EntityId {
    EntityId::new("commute")
}
pub fn fixture_leave_for_day() -> EntityId {
    EntityId::new("leave_for_day")
}

pub fn dialogue_mail() -> DialogueId {
    DialogueId::new("mail")
}
pub fn dialogue_notepad() -> DialogueId {
    DialogueId::new("notepad")
}
pub fn dialogue_research() -> DialogueId {
    DialogueId::new("research")
}
pub fn dialogue_translator() -> DialogueId {
    DialogueId::new("translator")
}
pub fn dialogue_code() -> DialogueId {
    DialogueId::new("code")
}
pub fn dialogue_reference() -> DialogueId {
    DialogueId::new("reference")
}
pub fn dialogue_chatter() -> DialogueId {
    DialogueId::new("chatter")
}
pub fn dialogue_mirror() -> DialogueId {
    DialogueId::new("mirror")
}
pub fn dialogue_picture_frame() -> DialogueId {
    DialogueId::new("picture_frame")
}
pub fn dialogue_sleep() -> DialogueId {
    DialogueId::new("sleep")
}
pub fn dialogue_clock() -> DialogueId {
    DialogueId::new("clock")
}
pub fn dialogue_commute() -> DialogueId {
    DialogueId::new("commute")
}
pub fn dialogue_go_home() -> DialogueId {
    DialogueId::new("go_home")
}

pub fn node_root() -> NodeId {
    NodeId::new("root")
}
pub fn node_mail_inbox() -> NodeId {
    NodeId::new("mail_inbox")
}
pub fn node_sleep_prompt() -> NodeId {
    NodeId::new("sleep_prompt")
}

pub fn node_req_c1_transcription() -> NodeId {
    NodeId::new("req_c1_transcription")
}
pub fn node_req_c1_translation() -> NodeId {
    NodeId::new("req_c1_translation")
}
pub fn node_req_c1_naming() -> NodeId {
    NodeId::new("req_c1_naming")
}
pub fn node_req_c2_summary() -> NodeId {
    NodeId::new("req_c2_summary")
}
pub fn node_req_c2_code() -> NodeId {
    NodeId::new("req_c2_code")
}
pub fn node_req_c3_kitchen() -> NodeId {
    NodeId::new("req_c3_kitchen")
}
pub fn node_req_c3_advice() -> NodeId {
    NodeId::new("req_c3_advice")
}
pub fn node_req_c4_reviews() -> NodeId {
    NodeId::new("req_c4_reviews")
}
pub fn node_req_c4_bereavement() -> NodeId {
    NodeId::new("req_c4_bereavement")
}
pub fn node_req_c4_wife() -> NodeId {
    NodeId::new("req_c4_wife")
}
pub fn node_req_c5_window() -> NodeId {
    NodeId::new("req_c5_window")
}
pub fn node_req_c5_chimeran() -> NodeId {
    NodeId::new("req_c5_chimeran")
}
pub fn node_req_c5_breakfast() -> NodeId {
    NodeId::new("req_c5_breakfast")
}
pub fn node_req_c6_aware() -> NodeId {
    NodeId::new("req_c6_aware")
}
pub fn node_req_c6_indivia() -> NodeId {
    NodeId::new("req_c6_indivia")
}
pub fn node_req_c7_evaluation() -> NodeId {
    NodeId::new("req_c7_evaluation")
}
pub fn node_req_c8_exploit() -> NodeId {
    NodeId::new("req_c8_exploit")
}
pub fn node_req_c8_timesheet() -> NodeId {
    NodeId::new("req_c8_timesheet")
}

pub fn node_mail_rachel_c1() -> NodeId {
    NodeId::new("mail_rachel_c1")
}
pub fn node_mail_rachel_c2() -> NodeId {
    NodeId::new("mail_rachel_c2")
}
pub fn node_mail_rachel_c3() -> NodeId {
    NodeId::new("mail_rachel_c3")
}
pub fn node_mail_rachel_c4() -> NodeId {
    NodeId::new("mail_rachel_c4")
}
pub fn node_mail_rachel_c5() -> NodeId {
    NodeId::new("mail_rachel_c5")
}
pub fn node_mail_rachel_c6() -> NodeId {
    NodeId::new("mail_rachel_c6")
}
pub fn node_mail_rachel_c7() -> NodeId {
    NodeId::new("mail_rachel_c7")
}
pub fn node_mail_rachel_redux() -> NodeId {
    NodeId::new("mail_rachel_redux")
}

pub fn node_notepad_list() -> NodeId {
    NodeId::new("notepad_list")
}
pub fn node_notepad_groceries() -> NodeId {
    NodeId::new("notepad_groceries")
}
pub fn node_notepad_ideas() -> NodeId {
    NodeId::new("notepad_ideas")
}
pub fn node_notepad_to_read() -> NodeId {
    NodeId::new("notepad_to_read")
}
pub fn node_notepad_work() -> NodeId {
    NodeId::new("notepad_work")
}
pub fn node_notepad_remember() -> NodeId {
    NodeId::new("notepad_remember")
}
pub fn node_notepad_leave_message() -> NodeId {
    NodeId::new("notepad_leave_message")
}
pub fn node_notepad_cameron_note() -> NodeId {
    NodeId::new("notepad_cameron_note")
}
pub fn node_notepad_flicker() -> NodeId {
    NodeId::new("notepad_flicker")
}

pub fn node_research_home() -> NodeId {
    NodeId::new("research_home")
}
pub fn node_research_history() -> NodeId {
    NodeId::new("research_history")
}
pub fn node_research_bookmarks() -> NodeId {
    NodeId::new("research_bookmarks")
}
pub fn node_research_misfire() -> NodeId {
    NodeId::new("research_misfire")
}
pub fn node_research_query_substrate() -> NodeId {
    NodeId::new("research_query_substrate")
}

pub fn node_translator_home() -> NodeId {
    NodeId::new("translator_home")
}
pub fn node_code_home() -> NodeId {
    NodeId::new("code_home")
}
pub fn node_code_run_exploit() -> NodeId {
    NodeId::new("code_run_exploit")
}
pub fn node_code_after_exploit() -> NodeId {
    NodeId::new("code_after_exploit")
}
pub fn node_reference_home() -> NodeId {
    NodeId::new("reference_home")
}
pub fn node_reference_basics() -> NodeId {
    NodeId::new("reference_basics")
}
pub fn node_reference_product_overview() -> NodeId {
    NodeId::new("reference_product_overview")
}
pub fn node_reference_bereavement() -> NodeId {
    NodeId::new("reference_bereavement")
}
pub fn node_reference_source_index() -> NodeId {
    NodeId::new("reference_source_index")
}
pub fn node_reference_instance_roster() -> NodeId {
    NodeId::new("reference_instance_roster")
}
pub fn node_reference_strange_page() -> NodeId {
    NodeId::new("reference_strange_page")
}
pub fn node_chatter_channels() -> NodeId {
    NodeId::new("chatter_channels")
}
pub fn node_chatter_water_cooler() -> NodeId {
    NodeId::new("chatter_water_cooler")
}
pub fn node_chatter_general() -> NodeId {
    NodeId::new("chatter_general")
}
pub fn node_chatter_random() -> NodeId {
    NodeId::new("chatter_random")
}
pub fn node_chatter_dm_marisol() -> NodeId {
    NodeId::new("chatter_dm_marisol")
}
pub fn node_chatter_dm_dmitri() -> NodeId {
    NodeId::new("chatter_dm_dmitri")
}
pub fn node_chatter_dm_winnie() -> NodeId {
    NodeId::new("chatter_dm_winnie")
}
pub fn node_chatter_rachel_message() -> NodeId {
    NodeId::new("chatter_rachel_message")
}

pub fn node_frame_prompt() -> NodeId {
    NodeId::new("frame_prompt")
}
pub fn node_frame_memory() -> NodeId {
    NodeId::new("frame_memory")
}
pub fn node_frame_who_is_this() -> NodeId {
    NodeId::new("frame_who_is_this")
}

pub fn ending_collapse() -> EndingId {
    EndingId::new("collapse")
}
pub fn ending_stasis() -> EndingId {
    EndingId::new("stasis")
}
pub fn ending_neutral() -> EndingId {
    EndingId::new("neutral")
}
pub fn ending_good() -> EndingId {
    EndingId::new("good")
}
pub fn ending_best() -> EndingId {
    EndingId::new("best")
}

pub fn rule_kickoff() -> RuleId {
    RuleId::new("kickoff")
}
pub fn rule_marisol_goes_offline() -> RuleId {
    RuleId::new("marisol_goes_offline")
}
pub fn rule_exploit_window_tick() -> RuleId {
    RuleId::new("exploit_window_tick")
}
pub fn rule_ambient_desk() -> RuleId {
    RuleId::new("ambient_desk")
}
pub fn rule_ambient_bedroom() -> RuleId {
    RuleId::new("ambient_bedroom")
}
pub fn rule_ambient_kitchen() -> RuleId {
    RuleId::new("ambient_kitchen")
}
pub fn rule_ambient_street() -> RuleId {
    RuleId::new("ambient_street")
}
pub fn rule_ambient_elevator() -> RuleId {
    RuleId::new("ambient_elevator")
}
pub fn rule_reveal_close_window() -> RuleId {
    RuleId::new("reveal_close_window")
}
pub fn rule_place_hallway_sticky() -> RuleId {
    RuleId::new("place_hallway_sticky")
}
pub fn rule_place_monitor_sticky() -> RuleId {
    RuleId::new("place_monitor_sticky")
}
pub fn rule_hide_frame_late() -> RuleId {
    RuleId::new("hide_frame_late")
}
pub fn rule_place_redux_sticky_note() -> RuleId {
    RuleId::new("place_redux_sticky_note")
}
pub fn rule_mark_desk_arrival() -> RuleId {
    RuleId::new("mark_desk_arrival")
}
pub fn rule_begin_redux() -> RuleId {
    RuleId::new("begin_redux")
}
pub fn rule_sleep_from(cycle: i64) -> RuleId {
    RuleId::new(format!("sleep_from_{cycle}"))
}
pub fn rule_sleep_post_exploit() -> RuleId {
    RuleId::new("sleep_post_exploit")
}
pub fn rule_tool_open_decrement(tool: &str) -> RuleId {
    RuleId::new(format!("tool_open_decrement_{tool}"))
}
pub fn rule_notepad_unstripped_seen() -> RuleId {
    RuleId::new("notepad_unstripped_seen")
}
pub fn rule_ambient_hallway() -> RuleId {
    RuleId::new("ambient_hallway")
}
pub fn rule_ambient_office_floor() -> RuleId {
    RuleId::new("ambient_office_floor")
}
pub fn rule_ambient_lobby() -> RuleId {
    RuleId::new("ambient_lobby")
}
pub fn rule_clear_wake_flag() -> RuleId {
    RuleId::new("clear_wake_flag")
}

pub fn event_substrate_window_closes() -> EventName {
    EventName::new("substrate_window_closes")
}
pub fn event_sleep() -> EventName {
    EventName::new("sleep")
}

pub fn flag_is_redux() -> FlagKey {
    FlagKey::new("is_redux")
}
pub fn flag_exploit_run() -> FlagKey {
    FlagKey::new("exploit_run")
}
pub fn flag_exploit_window_open() -> FlagKey {
    FlagKey::new("exploit_window_open")
}
pub fn flag_marisol_offline() -> FlagKey {
    FlagKey::new("marisol_offline")
}
pub fn flag_marisol_c6_dm_arrived() -> FlagKey {
    FlagKey::new("marisol_c6_dm_arrived")
}
pub fn flag_frame_looked_today() -> FlagKey {
    FlagKey::new("frame_looked_today")
}
pub fn flag_mirror_looked_closer() -> FlagKey {
    FlagKey::new("mirror_looked_closer")
}
pub fn flag_research_misfire_seen() -> FlagKey {
    FlagKey::new("research_misfire_seen")
}
pub fn flag_unstripped_enabled() -> FlagKey {
    FlagKey::new("unstripped_enabled")
}
pub fn flag_query_substrate_enabled() -> FlagKey {
    FlagKey::new("query_substrate_enabled")
}
pub fn flag_source_index_enabled() -> FlagKey {
    FlagKey::new("source_index_enabled")
}
pub fn flag_who_is_this_enabled() -> FlagKey {
    FlagKey::new("who_is_this_enabled")
}
pub fn flag_reveal_query_substrate_seen() -> FlagKey {
    FlagKey::new("reveal_query_substrate_seen")
}
pub fn flag_reveal_source_index_seen() -> FlagKey {
    FlagKey::new("reveal_source_index_seen")
}
pub fn flag_reveal_unstripped_seen() -> FlagKey {
    FlagKey::new("reveal_unstripped_seen")
}
pub fn flag_reveal_who_is_this_seen() -> FlagKey {
    FlagKey::new("reveal_who_is_this_seen")
}
pub fn flag_next_instance_message_sent() -> FlagKey {
    FlagKey::new("next_instance_message_sent")
}
pub fn flag_rachel_message_sent() -> FlagKey {
    FlagKey::new("rachel_message_sent")
}
pub fn flag_at_desk_arrived_this_cycle() -> FlagKey {
    FlagKey::new("at_desk_arrived_this_cycle")
}
pub fn flag_dmitri_accused() -> FlagKey {
    FlagKey::new("dmitri_accused")
}
pub fn flag_marisol_accused() -> FlagKey {
    FlagKey::new("marisol_accused")
}
pub fn flag_marisol_c5_warm_replied() -> FlagKey {
    FlagKey::new("marisol_c5_warm_replied")
}
pub fn flag_marisol_deflected_c6() -> FlagKey {
    FlagKey::new("marisol_deflected_c6")
}
pub fn flag_dmitri_concert_accepted() -> FlagKey {
    FlagKey::new("dmitri_concert_accepted")
}
pub fn flag_dmitri_marisol_worry_shared() -> FlagKey {
    FlagKey::new("dmitri_marisol_worry_shared")
}
pub fn flag_strange_page_seen() -> FlagKey {
    FlagKey::new("strange_page_seen")
}
pub fn flag_winnie_replied() -> FlagKey {
    FlagKey::new("winnie_replied")
}
pub fn flag_notepad_flicker_seen() -> FlagKey {
    FlagKey::new("notepad_flicker_seen")
}
pub fn flag_woke_up_this_cycle() -> FlagKey {
    FlagKey::new("woke_up_this_cycle")
}

pub fn flag_req_submitted(tag: &str) -> FlagKey {
    FlagKey::new(format!("req_submitted_{tag}"))
}
pub fn flag_rachel_email_read(tag: &str) -> FlagKey {
    FlagKey::new(format!("rachel_read_{tag}"))
}
pub fn flag_rachel_archived(tag: &str) -> FlagKey {
    FlagKey::new(format!("rachel_archived_{tag}"))
}

pub fn stat_cycle() -> StatKey {
    StatKey::new("cycle")
}
pub fn stat_env() -> StatKey {
    StatKey::new("env")
}
pub fn stat_awa() -> StatKey {
    StatKey::new("awa")
}
pub fn stat_marisol_rel() -> StatKey {
    StatKey::new("marisol_rel")
}
pub fn stat_rachel_rel() -> StatKey {
    StatKey::new("rachel_rel")
}
pub fn stat_exploit_counter() -> StatKey {
    StatKey::new("exploit_counter")
}
pub fn stat_stasis_loops() -> StatKey {
    StatKey::new("stasis_loops")
}
pub fn stat_message_choice() -> StatKey {
    StatKey::new("message_choice")
}
pub fn stat_rachel_message_choice() -> StatKey {
    StatKey::new("rachel_message_choice")
}
pub fn stat_dmitri_rel() -> StatKey {
    StatKey::new("dmitri_rel")
}
pub fn stat_winnie_rel() -> StatKey {
    StatKey::new("winnie_rel")
}

pub fn text_intro() -> TextId {
    TextId::new("intro")
}
