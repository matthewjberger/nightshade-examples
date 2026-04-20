use crate::game::ids;
use nightshade::interactive_fiction::data::{RoomId, RuntimeState, Value};

pub fn render(state: &RuntimeState) -> String {
    let current = &state.current_room;
    let cycle = state.stats.get(&ids::stat_cycle()).copied().unwrap_or(0);
    let is_redux = matches!(
        state.flags.get(&ids::flag_is_redux()),
        Some(Value::Bool(true))
    );
    let forever_at_desk = !is_redux && cycle >= 7 && current == &ids::room_desk();

    let label = |room: RoomId, name: &str| -> String {
        if &room == current {
            format!("[{:<8}]", name.to_uppercase())
        } else {
            format!("[{name:<8}]")
        }
    };

    let bedroom = label(ids::room_bedroom(), "bedroom");
    let hallway = label(ids::room_hallway(), "hallway");
    let kitchen = label(ids::room_kitchen(), "kitchen");
    let corridor = label(ids::room_building_corridor(), "corridor");
    let elevator = label(ids::room_elevator(), "elevator");
    let lobby = label(ids::room_lobby(), "lobby");
    let street = label(ids::room_street(), "street");
    let office = label(ids::room_office_floor(), "office");
    let desk = label(ids::room_desk(), "desk");

    let mut lines = vec![
        format!("  {bedroom} --s-- {hallway} --e-- {kitchen}"),
        "                       |".to_string(),
        "                       w".to_string(),
        "                       |".to_string(),
        format!("                  {corridor}"),
        "                       |".to_string(),
        "                       d".to_string(),
        "                       |".to_string(),
        format!("                  {elevator}"),
        "                       |".to_string(),
        "                       d".to_string(),
        "                       |".to_string(),
        format!("                  {lobby}"),
        "                       |".to_string(),
        "                       s".to_string(),
        "                       |".to_string(),
        format!("                  {street} --e-- {office} --e-- {desk}"),
        String::new(),
        "(your location is in CAPS)".to_string(),
    ];

    if forever_at_desk {
        lines.push("you-are-here-forever".to_string());
    }

    lines.push(String::new());
    lines.push(
        "(bedroom → 'commute' reaches the office in one step; desk → 'leaving for the day' walks you home and puts you to bed)"
            .to_string(),
    );

    lines.join("\n")
}
