pub mod camera;
pub mod character;
pub mod environment;
pub mod particles;

pub use camera::{camera_follow_system, spawn_camera};
pub use character::{
    animation_system, load_fox_model, spawn_character_controller, sync_fox_to_controller,
};
pub use environment::{spawn_environment, update_campfire_light};
pub use particles::{spawn_footprint_emitter, spawn_snow_blizzard, update_footprint_emitter};
