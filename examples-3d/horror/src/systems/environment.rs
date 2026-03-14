use crate::constants::{
    CEILING_TEXTURE, DOOR_TEXTURE, FLOOR_TEXTURE, LEVER_TEXTURE, NOTE_TEXTURE, WALL_TEXTURE,
};
use nightshade::prelude::*;

pub fn load_textures(world: &mut World) {
    let textures = [
        ("horror_floor", FLOOR_TEXTURE),
        ("horror_wall", WALL_TEXTURE),
        ("horror_ceiling", CEILING_TEXTURE),
        ("horror_door", DOOR_TEXTURE),
        ("horror_note", NOTE_TEXTURE),
        ("horror_lever", LEVER_TEXTURE),
    ];

    for (name, data) in textures {
        if let Ok(img) = image::load_from_memory(data) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            world.queue_command(WorldCommand::LoadTexture {
                name: name.to_string(),
                rgba_data: rgba.into_raw(),
                width,
                height,
            });
        }
    }
}
