use crate::ecs::GameWorld;
use nightshade::ecs::scene::SceneLoadState;

#[derive(Default)]
pub struct HorrorGame {
    pub game_world: GameWorld,
    pub scene_loader: SceneLoadState,
    pub scene_loaded: bool,
}
