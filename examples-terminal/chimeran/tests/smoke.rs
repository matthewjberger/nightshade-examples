use chimeran::game;
use nightshade::interactive_fiction::engine::Engine;

#[test]
fn world_validates() {
    let world = game::build_world();
    let _engine = Engine::new(world).expect("chimeran world should validate");
}

#[test]
fn start_places_player_in_bedroom() {
    let world = game::build_world();
    let engine = Engine::new(world).expect("validate");
    let mut state = engine.start_state();
    engine.start(&mut state);
    assert_eq!(state.current_room, chimeran::game::ids::room_bedroom());
    assert_eq!(
        state.stats.get(&chimeran::game::ids::stat_cycle()).copied(),
        Some(1)
    );
}
