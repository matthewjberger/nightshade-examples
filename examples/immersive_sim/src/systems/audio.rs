use crate::state::ImmersiveSim;
use nightshade::prelude::*;

pub fn audio_system(game: &mut ImmersiveSim, world: &mut World) {
    update_footstep_audio(game, world);
}

fn update_footstep_audio(game: &mut ImmersiveSim, world: &mut World) {
    let Some(player_entity) = game.player_entity else {
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

    if is_moving != game.audio.was_moving {
        game.audio.was_moving = is_moving;

        if let Some(footstep_entity) = game.audio.footstep_entity
            && let Some(source) = world.get_audio_source_mut(footstep_entity)
        {
            source.playing = is_moving;
        }
    }
}
