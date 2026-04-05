mod constants;
mod ecs;
mod state;
mod systems;

use state::PhysicsGame;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nightshade::prelude::launch(PhysicsGame::default())
}
