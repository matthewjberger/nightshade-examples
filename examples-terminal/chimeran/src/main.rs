use chimeran::view::ChimeranState;
use nightshade::tui::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(ChimeranState::new()))
}
