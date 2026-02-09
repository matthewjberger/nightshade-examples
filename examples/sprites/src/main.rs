use image::GenericImageView;
use nightshade::ecs::sprite_animator::SpriteFrame;
use nightshade::ecs::transform::commands::mark_local_transform_dirty;
use nightshade::prelude::*;

const SLOT_BG: u32 = 0;
const SLOT_SHIP: u32 = 1;
const SLOT_UFO_BLUE: u32 = 2;
const SLOT_UFO_RED: u32 = 3;
const SLOT_METEOR: u32 = 4;
const SLOT_STAR: u32 = 5;
const SLOT_ALIEN_STAND: u32 = 6;
const SLOT_ALIEN_WALK1: u32 = 7;
const SLOT_ALIEN_WALK2: u32 = 8;
const SLOT_ALIEN_JUMP: u32 = 9;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SpriteDemo::default())?;
    Ok(())
}

freecs::ecs! {
    SpriteDemo {
        bounce_velocity: BounceVelocity => BOUNCE_VELOCITY,
        engine_entity: EngineEntity => ENGINE_ENTITY,
    }
    DemoResources {
        camera_entity: Option<Entity>,
        fps_hud: Option<Entity>,
        camera_hud: Option<Entity>,
        sprite_count_hud: Option<Entity>,
        auto_pan: bool,
        auto_pan_time: f32,
        texture_uv_max: Vec<Vec2>,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BounceVelocity {
    pub velocity: Vec2,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EngineEntity(pub Entity);

struct TextureEntry {
    slot: u32,
    bytes: &'static [u8],
}

fn load_textures(world: &mut World) -> Vec<Vec2> {
    let entries = [
        TextureEntry {
            slot: SLOT_BG,
            bytes: include_bytes!("../assets/bg_blue.png"),
        },
        TextureEntry {
            slot: SLOT_SHIP,
            bytes: include_bytes!("../assets/ship_blue.png"),
        },
        TextureEntry {
            slot: SLOT_UFO_BLUE,
            bytes: include_bytes!("../assets/ufo_blue.png"),
        },
        TextureEntry {
            slot: SLOT_UFO_RED,
            bytes: include_bytes!("../assets/ufo_red.png"),
        },
        TextureEntry {
            slot: SLOT_METEOR,
            bytes: include_bytes!("../assets/meteor_brown1.png"),
        },
        TextureEntry {
            slot: SLOT_STAR,
            bytes: include_bytes!("../assets/star_large.png"),
        },
        TextureEntry {
            slot: SLOT_ALIEN_STAND,
            bytes: include_bytes!("../assets/alien_stand.png"),
        },
        TextureEntry {
            slot: SLOT_ALIEN_WALK1,
            bytes: include_bytes!("../assets/alien_walk1.png"),
        },
        TextureEntry {
            slot: SLOT_ALIEN_WALK2,
            bytes: include_bytes!("../assets/alien_walk2.png"),
        },
        TextureEntry {
            slot: SLOT_ALIEN_JUMP,
            bytes: include_bytes!("../assets/alien_jump.png"),
        },
    ];

    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let mut uv_max_table = vec![Vec2::new(1.0, 1.0); 128];

    for entry in &entries {
        let img = image::load_from_memory(entry.bytes).expect("Failed to decode image");
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8().into_raw();

        world
            .resources
            .command_queue
            .push(WorldCommand::UploadSpriteTexture {
                slot: entry.slot,
                rgba_data: rgba,
                width,
                height,
            });

        let half_texel_x = 0.5 / atlas_slot_size.0 as f32;
        let half_texel_y = 0.5 / atlas_slot_size.1 as f32;
        uv_max_table[entry.slot as usize] = Vec2::new(
            width as f32 / atlas_slot_size.0 as f32 - half_texel_x,
            height as f32 / atlas_slot_size.1 as f32 - half_texel_y,
        );
    }

    uv_max_table
}

fn uv_for_slot(uv_max_table: &[Vec2], slot: u32) -> (Vec2, Vec2) {
    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let half_texel = Vec2::new(
        0.5 / atlas_slot_size.0 as f32,
        0.5 / atlas_slot_size.1 as f32,
    );
    (half_texel, uv_max_table[slot as usize])
}

fn spawn_textured_sprite(
    world: &mut World,
    position: Vec3,
    size: Vec2,
    texture_slot: u32,
    uv_max_table: &[Vec2],
) -> Entity {
    let entity = spawn_sprite(world, position, size);
    let (uv_min, uv_max) = uv_for_slot(uv_max_table, texture_slot);
    if let Some(sprite) = world.get_sprite_mut(entity) {
        sprite.texture_index = texture_slot;
        sprite.texture_index2 = texture_slot;
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
    }
    entity
}

impl State for SpriteDemo {
    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.02, 0.02, 0.08, 1.0];
        world.resources.user_interface.enabled = true;

        self.resources.auto_pan = false;
        self.resources.auto_pan_time = 0.0;

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        self.resources.camera_entity = Some(camera);

        self.resources.texture_uv_max = load_textures(world);

        spawn_background_layer(world, &self.resources.texture_uv_max);
        spawn_midground_layer(world, &self.resources.texture_uv_max);
        spawn_foreground_ships(world, &self.resources.texture_uv_max);
        spawn_animated_aliens(world, &self.resources.texture_uv_max);
        spawn_bouncing_ufos(world, self);

        let fps_hud = spawn_hud_text_with_properties(
            world,
            "FPS: 0",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 10.0),
            TextProperties {
                font_size: 36.0,
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        self.resources.fps_hud = Some(fps_hud);

        let camera_hud = spawn_hud_text_with_properties(
            world,
            "Camera: (0, 0) Zoom: 1.0x",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 55.0),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(0.8, 0.8, 0.8, 1.0),
                ..Default::default()
            },
        );
        self.resources.camera_hud = Some(camera_hud);

        let sprite_count_hud = spawn_hud_text_with_properties(
            world,
            "Sprites: 0",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 90.0),
            TextProperties {
                font_size: 24.0,
                color: Vec4::new(0.8, 0.8, 0.8, 1.0),
                ..Default::default()
            },
        );
        self.resources.sprite_count_hud = Some(sprite_count_hud);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        ortho_camera_system(world);
        sprite_animation_system(world);
        update_bouncing_ufos(world, self);
        update_auto_pan(world, self);
        update_hud(world, self);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("2D Sprite Demo")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.heading("Controls");
                ui.separator();
                ui.label("WASD / Arrows - Pan camera");
                ui.label("Mouse Wheel - Zoom");
                ui.label("Q / E - Rotate camera");
                ui.separator();

                if let Some(camera_entity) = self.resources.camera_entity {
                    if let Some(camera) = world.get_camera(camera_entity) {
                        if let Projection::Orthographic(ortho) = &camera.projection {
                            let viewport_width = world
                                .resources
                                .window
                                .handle
                                .as_ref()
                                .map(|handle| handle.inner_size().width as f32)
                                .unwrap_or(1920.0);
                            let zoom = viewport_width / (2.0 * ortho.x_mag);
                            ui.label(format!("Zoom: {:.2}x", zoom));
                        }
                    }
                    if let Some(transform) = world.get_local_transform(camera_entity) {
                        let right = transform.right_vector();
                        let rotation = right.y.atan2(right.x);
                        ui.label(format!("Rotation: {:.1} deg", rotation.to_degrees()));
                        ui.label(format!(
                            "Position: ({:.0}, {:.0})",
                            transform.translation.x, transform.translation.y
                        ));
                    }
                }

                ui.separator();
                ui.checkbox(&mut self.resources.auto_pan, "Auto-pan camera");

                ui.separator();
                if ui.button("Reset Camera").clicked()
                    && let Some(camera_entity) = self.resources.camera_entity
                {
                    if let Some(transform) = world.get_local_transform_mut(camera_entity) {
                        transform.translation.x = 0.0;
                        transform.translation.y = 0.0;
                    }
                    if let Some(camera) = world.get_camera_mut(camera_entity) {
                        if let Projection::Orthographic(ref mut ortho) = camera.projection {
                            ortho.x_mag = 960.0;
                            ortho.y_mag = 540.0;
                        }
                    }
                    mark_local_transform_dirty(world, camera_entity);
                }

                if ui.button("Spawn 50 more UFOs").clicked() {
                    spawn_bouncing_ufos(world, self);
                }

                ui.separator();
                ui.heading("Layers (Z depth)");
                ui.label("Background (Z=0) - far back");
                ui.label("Stars/Meteors (Z=100-200) - midground");
                ui.label("Ships/UFOs (Z=500) - foreground");
                ui.label("Aliens (Z=600) - closest");
            });
    }
}

fn spawn_background_layer(world: &mut World, uv_max_table: &[Vec2]) {
    let tile_size = 256.0;

    for row in -8..=8 {
        for col in -12..=12 {
            let entity = spawn_textured_sprite(
                world,
                Vec3::new(col as f32 * tile_size, row as f32 * tile_size, 0.0),
                Vec2::new(tile_size, tile_size),
                SLOT_BG,
                uv_max_table,
            );
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.color = [1.0, 1.0, 1.0, 1.0];
            }
        }
    }
}

fn spawn_midground_layer(world: &mut World, uv_max_table: &[Vec2]) {
    for index in 0..30 {
        let angle = index as f32 * 0.7;
        let radius = 200.0 + index as f32 * 60.0;
        let scale_factor = 0.8 + (index as f32 * 0.3).sin().abs() * 0.8;

        let entity = spawn_textured_sprite(
            world,
            Vec3::new(angle.cos() * radius, angle.sin() * radius, 200.0),
            Vec2::new(101.0, 84.0),
            SLOT_METEOR,
            uv_max_table,
        );
        if let Some(sprite) = world.get_sprite_mut(entity) {
            sprite.color = [1.0, 1.0, 1.0, 0.9];
        }
        if let Some(local_transform) = world.get_local_transform_mut(entity) {
            local_transform.rotation = nalgebra_glm::quat_angle_axis(angle * 2.0, &Vec3::z());
            local_transform.scale = Vec3::new(scale_factor, scale_factor, 1.0);
        }
        mark_local_transform_dirty(world, entity);
    }

    for index in 0..20 {
        let angle = index as f32 * 1.1 + 0.5;
        let radius = 300.0 + index as f32 * 50.0;
        let scale_factor = 0.5 + (index as f32 * 0.5).sin().abs();

        let entity = spawn_textured_sprite(
            world,
            Vec3::new(angle.cos() * radius, angle.sin() * radius, 100.0),
            Vec2::new(64.0, 64.0),
            SLOT_STAR,
            uv_max_table,
        );
        if let Some(sprite) = world.get_sprite_mut(entity) {
            sprite.color = [1.0, 1.0, 1.0, 0.7];
        }
        if let Some(local_transform) = world.get_local_transform_mut(entity) {
            local_transform.scale = Vec3::new(scale_factor, scale_factor, 1.0);
        }
        mark_local_transform_dirty(world, entity);
    }
}

fn spawn_foreground_ships(world: &mut World, uv_max_table: &[Vec2]) {
    let positions = [
        Vec2::new(200.0, 100.0),
        Vec2::new(-300.0, -200.0),
        Vec2::new(400.0, -100.0),
        Vec2::new(-200.0, 300.0),
        Vec2::new(0.0, -400.0),
        Vec2::new(500.0, 200.0),
    ];

    for (index, position) in positions.iter().enumerate() {
        let entity = spawn_textured_sprite(
            world,
            Vec3::new(position.x, position.y, 500.0),
            Vec2::new(99.0, 75.0),
            SLOT_SHIP,
            uv_max_table,
        );
        if let Some(local_transform) = world.get_local_transform_mut(entity) {
            local_transform.rotation =
                nalgebra_glm::quat_angle_axis((index as f32 * 0.8).sin() * 0.5, &Vec3::z());
            local_transform.scale = Vec3::new(1.5, 1.5, 1.0);
        }
        mark_local_transform_dirty(world, entity);
    }
}

fn spawn_animated_aliens(world: &mut World, uv_max_table: &[Vec2]) {
    let frame_slots = [
        SLOT_ALIEN_STAND,
        SLOT_ALIEN_WALK1,
        SLOT_ALIEN_WALK2,
        SLOT_ALIEN_JUMP,
    ];

    let positions = [
        Vec2::new(-100.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(0.0, 200.0),
        Vec2::new(0.0, -200.0),
        Vec2::new(-250.0, 150.0),
        Vec2::new(250.0, -150.0),
    ];

    for (index, position) in positions.iter().enumerate() {
        let mut frames = Vec::new();
        for &slot in &frame_slots {
            let (uv_min, uv_max) = uv_for_slot(uv_max_table, slot);
            frames.push(SpriteFrame {
                uv_min,
                uv_max,
                duration: 0.2 + index as f32 * 0.03,
                texture_index: Some(slot),
            });
        }

        let entity = world.spawn_entities(
            nightshade::ecs::SPRITE
                | nightshade::ecs::VISIBILITY
                | nightshade::ecs::SPRITE_ANIMATOR
                | nightshade::ecs::LOCAL_TRANSFORM
                | nightshade::ecs::LOCAL_TRANSFORM_DIRTY
                | nightshade::ecs::GLOBAL_TRANSFORM,
            1,
        )[0];

        let first_slot = frame_slots[0];
        let (first_uv_min, first_uv_max) = uv_for_slot(uv_max_table, first_slot);

        if let Some(local_transform) = world.get_local_transform_mut(entity) {
            local_transform.translation = Vec3::new(position.x, position.y, 600.0);
            local_transform.scale = Vec3::new(0.4, 0.4, 1.0);
        }

        if let Some(sprite) = world.get_sprite_mut(entity) {
            sprite.size = Vec2::new(128.0, 256.0);
            sprite.texture_index = first_slot;
            sprite.texture_index2 = first_slot;
            sprite.color = [1.0, 1.0, 1.0, 1.0];
            sprite.uv_min = first_uv_min;
            sprite.uv_max = first_uv_max;
        }

        if let Some(visibility) = world.get_visibility_mut(entity) {
            visibility.visible = true;
        }

        if let Some(animator) = world.get_sprite_animator_mut(entity) {
            animator.frames = frames;
            animator.playing = true;
            animator.speed = 0.8 + (index as f32 * 0.3).sin().abs() * 0.8;
            animator.loop_mode = if index % 2 == 0 {
                nightshade::ecs::sprite_animator::LoopMode::Loop
            } else {
                nightshade::ecs::sprite_animator::LoopMode::PingPong
            };
        }
    }
}

fn spawn_bouncing_ufos(world: &mut World, demo: &mut SpriteDemo) {
    let uv_max_table = demo.resources.texture_uv_max.clone();

    for index in 0..50 {
        let angle = index as f32 * 0.4;
        let radius = 80.0 + index as f32 * 12.0;
        let use_red = index % 3 == 0;
        let slot = if use_red { SLOT_UFO_RED } else { SLOT_UFO_BLUE };
        let scale_factor = 0.6 + (index as f32 * 0.2).sin().abs() * 0.4;

        let engine_entity = spawn_textured_sprite(
            world,
            Vec3::new(angle.cos() * radius, angle.sin() * radius, 500.0),
            Vec2::new(91.0, 91.0),
            slot,
            &uv_max_table,
        );

        if let Some(local_transform) = world.get_local_transform_mut(engine_entity) {
            local_transform.scale = Vec3::new(scale_factor, scale_factor, 1.0);
        }
        mark_local_transform_dirty(world, engine_entity);

        let game_entity = demo.spawn_entities(BOUNCE_VELOCITY | ENGINE_ENTITY, 1)[0];
        demo.set_bounce_velocity(
            game_entity,
            BounceVelocity {
                velocity: Vec2::new((angle + 1.0).cos() * 120.0, (angle + 1.0).sin() * 120.0),
            },
        );
        demo.set_engine_entity(game_entity, EngineEntity(engine_entity));
    }
}

fn update_bouncing_ufos(world: &mut World, demo: &mut SpriteDemo) {
    let delta_time = world.resources.window.timing.delta_time;
    let elapsed = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;

    let entities: Vec<_> = demo
        .query_entities(BOUNCE_VELOCITY | ENGINE_ENTITY)
        .collect();

    for entity in entities {
        let handle = demo.get_engine_entity(entity).copied();

        if let (Some(physics), Some(handle)) = (demo.get_bounce_velocity_mut(entity), handle) {
            if let Some(local_transform) = world.get_local_transform_mut(handle.0) {
                local_transform.translation.x += physics.velocity.x * delta_time;
                local_transform.translation.y += physics.velocity.y * delta_time;

                let bound = 800.0;
                if local_transform.translation.x.abs() > bound {
                    physics.velocity.x *= -1.0;
                    local_transform.translation.x =
                        local_transform.translation.x.clamp(-bound, bound);
                }
                if local_transform.translation.y.abs() > bound {
                    physics.velocity.y *= -1.0;
                    local_transform.translation.y =
                        local_transform.translation.y.clamp(-bound, bound);
                }

                local_transform.rotation = nalgebra_glm::quat_angle_axis(
                    elapsed * physics.velocity.x.signum() * 1.5,
                    &Vec3::z(),
                );
            }
            mark_local_transform_dirty(world, handle.0);
        }
    }
}

fn update_auto_pan(world: &mut World, demo: &mut SpriteDemo) {
    if !demo.resources.auto_pan {
        return;
    }

    let delta_time = world.resources.window.timing.delta_time;
    demo.resources.auto_pan_time += delta_time;

    if let Some(camera_entity) = demo.resources.camera_entity {
        let time = demo.resources.auto_pan_time;
        let target_x = (time * 0.3).sin() * 600.0;
        let target_y = (time * 0.2).cos() * 400.0;

        if let Some(transform) = world.get_local_transform_mut(camera_entity) {
            let lerp_factor = 1.0 - 0.95_f32.powf(delta_time * 60.0);
            transform.translation.x += (target_x - transform.translation.x) * lerp_factor;
            transform.translation.y += (target_y - transform.translation.y) * lerp_factor;
        }
        mark_local_transform_dirty(world, camera_entity);
    }
}

fn update_hud(world: &mut World, demo: &mut SpriteDemo) {
    if let Some(fps_entity) = demo.resources.fps_hud {
        let fps = world.resources.window.timing.frames_per_second;
        let text_index = world.get_hud_text(fps_entity).map(|text| text.text_index);
        if let Some(text_index) = text_index {
            world
                .resources
                .text_cache
                .set_text(text_index, format!("FPS: {:.0}", fps));
            if let Some(hud_text) = world.get_hud_text_mut(fps_entity) {
                hud_text.dirty = true;
            }
        }
    }

    if let Some(camera_hud_entity) = demo.resources.camera_hud {
        let camera_info = demo.resources.camera_entity.and_then(|camera_entity| {
            let transform = world.get_local_transform(camera_entity)?;
            let camera = world.get_camera(camera_entity)?;
            let Projection::Orthographic(ortho) = &camera.projection else {
                return None;
            };
            let viewport_width = world
                .resources
                .window
                .handle
                .as_ref()
                .map(|handle| handle.inner_size().width as f32)
                .unwrap_or(1920.0);
            let zoom = viewport_width / (2.0 * ortho.x_mag);
            let right = transform.right_vector();
            let rotation = right.y.atan2(right.x);
            Some(format!(
                "Camera: ({:.0}, {:.0}) Zoom: {:.2}x Rot: {:.0} deg",
                transform.translation.x,
                transform.translation.y,
                zoom,
                rotation.to_degrees()
            ))
        });

        if let Some(info) = camera_info {
            let text_index = world
                .get_hud_text(camera_hud_entity)
                .map(|text| text.text_index);
            if let Some(text_index) = text_index {
                world.resources.text_cache.set_text(text_index, info);
                if let Some(hud_text) = world.get_hud_text_mut(camera_hud_entity) {
                    hud_text.dirty = true;
                }
            }
        }
    }

    if let Some(count_entity) = demo.resources.sprite_count_hud {
        let sprite_count = world.query_entities(nightshade::ecs::SPRITE).count();
        let text_index = world.get_hud_text(count_entity).map(|text| text.text_index);
        if let Some(text_index) = text_index {
            world
                .resources
                .text_cache
                .set_text(text_index, format!("Sprites: {}", sprite_count));
            if let Some(hud_text) = world.get_hud_text_mut(count_entity) {
                hud_text.dirty = true;
            }
        }
    }
}
