use nightshade::prelude::*;
use rand::Rng;

const FACE_PNG: &[u8] = include_bytes!("../../../assets/textures/awesomeface.png");

const GRAVITY: f32 = -981.0;
const BUNNY_SIZE: f32 = 16.0;
const TARGET_FPS: f32 = 60.0;
const MEASUREMENT_FRAMES: usize = 60;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(BunnyWorld::default())?;
    Ok(())
}

freecs::ecs! {
    BunnyWorld {
        bunny_tag: BunnyTag => BUNNY_TAG,
    }
    BunnyResources {
        engine_entities: Vec<Entity>,
        velocities: Vec<Vec2>,
        max_x: f32,
        max_y: f32,
        fps_text: Option<Entity>,
        count_text: Option<Entity>,
        status_text: Option<Entity>,
        frame_times: Vec<f32>,
        frame_time_index: usize,
        cooldown: usize,
        done: bool,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BunnyTag;

impl State for BunnyWorld {
    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;

        if let Some(window) = &world.resources.window.handle {
            let size = window.inner_size();
            self.resources.max_x = size.width as f32;
            self.resources.max_y = size.height as f32;
        }

        let half_x = self.resources.max_x / 2.0;
        let half_y = self.resources.max_y / 2.0;
        let camera = spawn_ortho_camera(world, Vec2::new(half_x, half_y));
        if let Some(camera_data) = world.core.get_camera_mut(camera)
            && let Projection::Orthographic(ref mut ortho) = camera_data.projection
        {
            ortho.x_mag = half_x;
            ortho.y_mag = half_y;
        }

        self.resources.frame_times = vec![0.0; MEASUREMENT_FRAMES];
        self.resources.engine_entities = Vec::with_capacity(500_000);
        self.resources.velocities = Vec::with_capacity(500_000);
        self.resources.cooldown = MEASUREMENT_FRAMES;

        load_sprite_texture(world);
        setup_hud(world, self);
        spawn_bunnies(world, self, 1000);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if let Some(window) = &world.resources.window.handle {
            let size = window.inner_size();
            self.resources.max_x = size.width as f32;
            self.resources.max_y = size.height as f32;
        }

        record_frame_time(world, self);

        if !self.resources.done {
            auto_spawn(world, self);
        }

        update_physics(world, self);
        update_hud(world, self);
    }
}

fn load_sprite_texture(world: &mut World) {
    let img = image::load_from_memory(FACE_PNG).expect("failed to decode awesomeface.png");
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    world
        .resources
        .command_queue
        .push(WorldCommand::UploadSpriteTexture {
            slot: 0,
            rgba_data: rgba.into_raw(),
            width,
            height,
        });
}

fn setup_hud(world: &mut World, bunny_world: &mut BunnyWorld) {
    bunny_world.resources.fps_text = Some(spawn_hud_text_with_properties(
        world,
        "FPS: 0",
        HudAnchor::TopRight,
        Vec2::new(-10.0, 10.0),
        TextProperties {
            font_size: 48.0,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            ..Default::default()
        },
    ));

    bunny_world.resources.count_text = Some(spawn_hud_text_with_properties(
        world,
        "Bunnies: 0",
        HudAnchor::TopRight,
        Vec2::new(-10.0, 70.0),
        TextProperties {
            font_size: 36.0,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            ..Default::default()
        },
    ));

    bunny_world.resources.status_text = Some(spawn_hud_text_with_properties(
        world,
        "Spawning...",
        HudAnchor::TopRight,
        Vec2::new(-10.0, 115.0),
        TextProperties {
            font_size: 28.0,
            color: Vec4::new(0.5, 1.0, 0.5, 1.0),
            ..Default::default()
        },
    ));
}

fn spawn_bunnies(world: &mut World, bunny_world: &mut BunnyWorld, count: usize) {
    let mut rng = rand::rng();
    let spawn_x = bunny_world.resources.max_x * 0.5;
    let spawn_y = bunny_world.resources.max_y * 0.9;

    let new_entities = world.spawn_entities(VISIBILITY, count);
    for &entity in &new_entities {
        world.sprite2d.add_components(entity, SPRITE);
    }

    for &entity in &new_entities {
        if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
            sprite.position = Vec2::new(spawn_x, spawn_y);
            sprite.texture_index = 0;
            sprite.uv_min = Vec2::new(0.0, 0.0);
            sprite.uv_max = Vec2::new(1.0, 1.0);
            sprite.color = [1.0, 1.0, 1.0, 1.0];
            sprite.size = Vec2::new(BUNNY_SIZE, BUNNY_SIZE);
        }

        if let Some(visibility) = world.core.get_visibility_mut(entity) {
            visibility.visible = true;
        }

        bunny_world.resources.engine_entities.push(entity);
        bunny_world.resources.velocities.push(Vec2::new(
            rng.random_range(-250.0..250.0),
            rng.random_range(-50.0..200.0),
        ));
    }
}

fn record_frame_time(world: &World, bunny_world: &mut BunnyWorld) {
    if bunny_world.resources.frame_times.is_empty() {
        return;
    }
    let frame_time = world.resources.window.timing.raw_delta_time * 1000.0;
    let index = bunny_world.resources.frame_time_index;
    bunny_world.resources.frame_times[index] = frame_time;
    bunny_world.resources.frame_time_index = (index + 1) % MEASUREMENT_FRAMES;
}

fn auto_spawn(world: &mut World, bunny_world: &mut BunnyWorld) {
    if bunny_world.resources.cooldown > 0 {
        bunny_world.resources.cooldown -= 1;
        return;
    }

    let avg_frame_time: f32 =
        bunny_world.resources.frame_times.iter().sum::<f32>() / MEASUREMENT_FRAMES as f32;

    if avg_frame_time < 0.001 {
        return;
    }

    let avg_fps = 1000.0 / avg_frame_time;

    if avg_fps < TARGET_FPS {
        bunny_world.resources.done = true;
        return;
    }

    let headroom = avg_fps - TARGET_FPS;
    let batch_size = if headroom > 500.0 {
        50_000
    } else if headroom > 100.0 {
        10_000
    } else if headroom > 50.0 {
        5_000
    } else if headroom > 20.0 {
        2_000
    } else if headroom > 10.0 {
        500
    } else if headroom > 5.0 {
        100
    } else {
        50
    };

    spawn_bunnies(world, bunny_world, batch_size);
    bunny_world.resources.cooldown = MEASUREMENT_FRAMES;
}

fn update_physics(world: &mut World, bunny_world: &mut BunnyWorld) {
    let delta_time = world.resources.window.timing.delta_time;
    let max_x = bunny_world.resources.max_x;
    let max_y = bunny_world.resources.max_y;

    for index in 0..bunny_world.resources.engine_entities.len() {
        let entity = bunny_world.resources.engine_entities[index];
        let velocity = &mut bunny_world.resources.velocities[index];

        velocity.y += GRAVITY * delta_time;

        if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
            sprite.position.x += velocity.x * delta_time;
            sprite.position.y += velocity.y * delta_time;

            if sprite.position.x > max_x - BUNNY_SIZE {
                sprite.position.x = max_x - BUNNY_SIZE;
                velocity.x = -velocity.x;
            } else if sprite.position.x < 0.0 {
                sprite.position.x = 0.0;
                velocity.x = -velocity.x;
            }

            if sprite.position.y < 0.0 {
                sprite.position.y = 0.0;
                velocity.y = -velocity.y;
            } else if sprite.position.y > max_y - BUNNY_SIZE {
                sprite.position.y = max_y - BUNNY_SIZE;
                velocity.y = -velocity.y;
            }
        }
    }
}

fn update_hud(world: &mut World, bunny_world: &BunnyWorld) {
    let fps = world.resources.window.timing.frames_per_second;
    let count = bunny_world.resources.engine_entities.len();

    if let Some(fps_entity) = bunny_world.resources.fps_text
        && let Some(text_index) = world.core.get_hud_text(fps_entity).map(|text| text.text_index)
    {
        world
            .resources
            .text_cache
            .set_text(text_index, format!("FPS: {:.0}", fps));

        let color = if fps >= TARGET_FPS - 2.0 {
            Vec4::new(0.0, 1.0, 0.0, 1.0)
        } else {
            Vec4::new(1.0, 0.3, 0.3, 1.0)
        };
        if let Some(hud) = world.core.get_hud_text_mut(fps_entity) {
            hud.properties.color = color;
            hud.dirty = true;
        }
    }

    if let Some(count_entity) = bunny_world.resources.count_text
        && let Some(text_index) = world.core.get_hud_text(count_entity).map(|text| text.text_index)
    {
        world.resources.text_cache.set_text(
            text_index,
            format!("Bunnies: {}", format_number_with_commas(count)),
        );
        if let Some(hud) = world.core.get_hud_text_mut(count_entity) {
            hud.dirty = true;
        }
    }

    if let Some(status_entity) = bunny_world.resources.status_text
        && let Some(text_index) = world
            .core.get_hud_text(status_entity)
            .map(|text| text.text_index)
    {
        let (text, color) = if bunny_world.resources.done {
            (
                format!(
                    "Done: {} bunnies at {:.0} FPS",
                    format_number_with_commas(count),
                    TARGET_FPS
                ),
                Vec4::new(1.0, 1.0, 0.0, 1.0),
            )
        } else {
            ("Spawning...".to_string(), Vec4::new(0.5, 1.0, 0.5, 1.0))
        };
        world.resources.text_cache.set_text(text_index, text);
        if let Some(hud) = world.core.get_hud_text_mut(status_entity) {
            hud.properties.color = color;
            hud.dirty = true;
        }
    }
}

fn format_number_with_commas(number: usize) -> String {
    let number_str = number.to_string();
    let mut result = String::new();
    let chars: Vec<char> = number_str.chars().collect();

    for (index, character) in chars.iter().enumerate() {
        if index > 0 && (chars.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*character);
    }

    result
}
