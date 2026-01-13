use crate::state::LevelDemo;
use nightshade::prelude::*;

pub fn audio_system(demo: &mut LevelDemo, world: &mut World) {
    update_footstep_audio(demo, world);
}

fn update_footstep_audio(demo: &mut LevelDemo, world: &mut World) {
    let Some(player_entity) = demo.player_entity else {
        return;
    };

    let is_moving = world
        .get_character_controller(player_entity)
        .map(|cc| {
            let vel = cc.velocity;
            let horizontal_speed = (vel.x * vel.x + vel.z * vel.z).sqrt();
            horizontal_speed > 0.1 && cc.grounded
        })
        .unwrap_or(false);

    if is_moving != demo.audio.was_moving {
        demo.audio.was_moving = is_moving;

        if let Some(footstep_entity) = demo.audio.footstep_entity
            && let Some(source) = world.get_audio_source_mut(footstep_entity)
        {
            source.playing = is_moving;
        }
    }
}
