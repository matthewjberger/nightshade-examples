//! Entry point. The library crate holds the data + engine + view layers; this
//! binary just hands a freshly-built `LighthouseState` to Nightshade.

use lighthouse::view::LighthouseState;
use nightshade::tui::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(LighthouseState::new()))
}
