use nightshade::ecs::input::resources::mouse::MouseState;
use nightshade::ecs::world::commands::WorldCommand;
use nightshade::prelude::*;
use nightshade::render::{
    generate_linear_gradient_texture, generate_multi_stop_gradient_texture,
    generate_radial_gradient_texture,
};

const TAG_POSITION: u32 = 0;
const TAG_SCALE: u32 = 1;
const TAG_ROTATION: u32 = 3;

const ALL_SHAPES: &[SpriteShape] = &[
    SpriteShape::Rect,
    SpriteShape::Circle,
    SpriteShape::Ring,
    SpriteShape::Triangle,
    SpriteShape::Capsule,
    SpriteShape::OutlinedRect,
    SpriteShape::SoftCircle,
];

#[derive(Default)]
struct GfxDemo {
    camera_entity: Option<Entity>,
    current_shape_index: usize,
    spawned_entities: Vec<Entity>,
    demo_entities: Vec<Entity>,
    preview_entity: Option<Entity>,
    next_spawn_depth: f32,
}

fn hue_to_rgb(hue: f32) -> (f32, f32, f32) {
    let hue = hue.fract();
    let segment = hue * 6.0;
    let fraction = segment.fract();
    match segment as u32 {
        0 => (1.0, fraction, 0.0),
        1 => (1.0 - fraction, 1.0, 0.0),
        2 => (0.0, 1.0, fraction),
        3 => (0.0, 1.0 - fraction, 1.0),
        4 => (fraction, 0.0, 1.0),
        _ => (1.0, 0.0, 1.0 - fraction),
    }
}

fn current_shape(demo: &GfxDemo) -> SpriteShape {
    ALL_SHAPES[demo.current_shape_index]
}

fn configure_preview_for_shape(world: &mut World, entity: Entity, shape: SpriteShape) {
    let (texture_index, uv_min, uv_max) = shape_texture_info(shape);

    let size = match shape {
        SpriteShape::Rect | SpriteShape::OutlinedRect => Vec2::new(40.0, 40.0),
        SpriteShape::Circle | SpriteShape::Ring => Vec2::new(40.0, 40.0),
        SpriteShape::Triangle => Vec2::new(40.0, 40.0),
        SpriteShape::Capsule => Vec2::new(60.0, 28.0),
        SpriteShape::SoftCircle => Vec2::new(60.0, 60.0),
    };

    if let Some(sprite) = world.get_sprite_mut(entity) {
        sprite.texture_index = texture_index;
        sprite.texture_index2 = texture_index;
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
        sprite.color = [1.0, 1.0, 1.0, 0.3];
        sprite.size = size;
        sprite.depth = 20.0;
        sprite.blend_mode = if shape == SpriteShape::SoftCircle {
            SpriteBlendMode::Additive
        } else {
            SpriteBlendMode::Alpha
        };
    }
}

impl GfxDemo {
    fn build_scene(&mut self, world: &mut World) {
        self.build_rect_grid(world);
        self.build_circles(world);
        self.build_rings(world);
        self.build_triangles(world);
        self.build_capsules(world);
        self.build_outlined_rects(world);
        self.build_lines(world);
        self.build_bezier_curves(world);
        self.build_paths(world);
        self.build_gradients(world);
        self.build_soft_circles(world);
        self.build_labels(world);
        self.build_preview(world);
    }

    fn build_preview(&mut self, world: &mut World) {
        let entity = spawn_circle(world, Vec2::new(0.0, 0.0), 20.0, [1.0, 1.0, 1.0, 0.3]);
        if let Some(sprite) = world.get_sprite_mut(entity) {
            sprite.depth = 20.0;
        }
        configure_preview_for_shape(world, entity, current_shape(self));
        self.preview_entity = Some(entity);
    }

    fn build_rect_grid(&mut self, world: &mut World) {
        let center_x = -380.0;
        let center_y = 100.0;
        let columns = 5;
        let rows = 4;
        let cell_size = 18.0;
        let gap = 3.0;
        let stride = cell_size + gap;
        let grid_width = columns as f32 * stride - gap;
        let grid_height = rows as f32 * stride - gap;
        let start_x = center_x - grid_width / 2.0 + cell_size / 2.0;
        let start_y = center_y + grid_height / 2.0 - cell_size / 2.0;

        for row in 0..rows {
            for column in 0..columns {
                let hue = (row * columns + column) as f32 / (rows * columns) as f32;
                let (red, green, blue) = hue_to_rgb(hue);
                let position_x = start_x + column as f32 * stride;
                let position_y = start_y - row as f32 * stride;

                let entity = spawn_rect(
                    world,
                    Vec2::new(position_x, position_y),
                    Vec2::new(cell_size, cell_size),
                    [red, green, blue, 1.0],
                );
                if let Some(sprite) = world.get_sprite_mut(entity) {
                    sprite.depth = 5.0;
                }

                world.add_components(entity, TWEEN);
                let mut tween = Tween::new();
                let phase = (row * columns + column) as f32 / (rows * columns) as f32;
                let duration = 3.0 + phase * 2.0;
                tween.add_track(
                    TweenTrack::new(TweenValue::F32(0.85), TweenValue::F32(1.1), duration)
                        .with_easing(EasingFunction::SineInOut)
                        .with_loop_mode(TweenLoopMode::PingPong)
                        .with_tag(TAG_SCALE)
                        .with_delay(phase * duration),
                );
                world.set_tween(entity, tween);
                self.demo_entities.push(entity);
            }
        }
    }

    fn build_circles(&mut self, world: &mut World) {
        let center_x = -230.0;
        let center_y = 100.0;

        let configs: &[(f32, f32, f32, [f32; 4])] = &[
            (-30.0, 0.0, 20.0, [1.0, 0.3, 0.3, 1.0]),
            (0.0, 25.0, 16.0, [0.3, 1.0, 0.3, 1.0]),
            (30.0, 0.0, 18.0, [0.3, 0.5, 1.0, 1.0]),
            (0.0, -25.0, 14.0, [1.0, 0.8, 0.2, 1.0]),
            (0.0, 0.0, 12.0, [1.0, 1.0, 1.0, 0.9]),
        ];

        for (index, &(offset_x, offset_y, radius, color)) in configs.iter().enumerate() {
            let entity = spawn_circle(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                radius,
                color,
            );
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 4.0 - index as f32 * 0.1;
            }

            world.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 4.0 + index as f32 * 0.5;
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.85), TweenValue::F32(1.2), duration)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_rings(&mut self, world: &mut World) {
        let center_x = -90.0;
        let center_y = 100.0;

        let ring_configs: &[(f32, [f32; 4])] = &[
            (35.0, [1.0, 0.4, 0.1, 0.9]),
            (27.0, [0.1, 0.8, 0.4, 0.9]),
            (19.0, [0.3, 0.4, 1.0, 0.9]),
            (12.0, [1.0, 0.9, 0.2, 0.9]),
        ];

        for (index, &(radius, color)) in ring_configs.iter().enumerate() {
            let entity = spawn_ring(world, Vec2::new(center_x, center_y), radius, color);
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 3.0;
            }

            world.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 3.0 + index as f32;
            let direction = if index % 2 == 0 { 1.0 } else { -1.0 };
            tween.add_track(
                TweenTrack::new(
                    TweenValue::F32(0.0),
                    TweenValue::F32(direction * std::f32::consts::TAU),
                    duration,
                )
                .with_easing(EasingFunction::Linear)
                .with_loop_mode(TweenLoopMode::Loop)
                .with_tag(TAG_ROTATION),
            );
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.9), TweenValue::F32(1.15), duration * 0.7)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_triangles(&mut self, world: &mut World) {
        let center_x = 50.0;
        let center_y = 100.0;

        let triangle_configs: &[(f32, f32, f32, f32, [f32; 4])] = &[
            (-25.0, 15.0, 30.0, 35.0, [1.0, 0.4, 0.6, 1.0]),
            (20.0, -10.0, 25.0, 28.0, [0.4, 0.8, 1.0, 1.0]),
            (0.0, 0.0, 20.0, 22.0, [0.5, 1.0, 0.5, 0.9]),
        ];

        for (index, &(offset_x, offset_y, width, height, color)) in
            triangle_configs.iter().enumerate()
        {
            let entity = spawn_triangle(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                Vec2::new(width, height),
                color,
            );
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 4.0 - index as f32 * 0.1;
            }

            world.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 3.0 + index as f32 * 0.7;
            let direction = if index % 2 == 0 { 1.0 } else { -1.0 };
            tween.add_track(
                TweenTrack::new(
                    TweenValue::F32(0.0),
                    TweenValue::F32(direction * std::f32::consts::TAU),
                    duration * 2.0,
                )
                .with_easing(EasingFunction::Linear)
                .with_loop_mode(TweenLoopMode::Loop)
                .with_tag(TAG_ROTATION),
            );
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.9), TweenValue::F32(1.15), duration)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_capsules(&mut self, world: &mut World) {
        let center_x = 190.0;
        let center_y = 100.0;

        let offsets = [(0.0_f32, 20.0_f32), (0.0, -5.0), (0.0, -25.0)];
        let sizes = [(60.0_f32, 22.0_f32), (50.0, 18.0), (40.0, 16.0)];
        let rotations = [0.0_f32, 0.4, -0.3];
        let colors: [[f32; 4]; 3] = [
            [0.9, 0.3, 0.8, 1.0],
            [0.3, 0.9, 0.6, 1.0],
            [0.4, 0.5, 1.0, 1.0],
        ];

        for index in 0..3
        {
            let entity = spawn_capsule(
                world,
                Vec2::new(center_x + offsets[index].0, center_y + offsets[index].1),
                Vec2::new(sizes[index].0, sizes[index].1),
                colors[index],
            );
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 4.0;
                sprite.rotation = rotations[index];
            }

            world.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 2.5 + index as f32 * 0.6;
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.9), TweenValue::F32(1.15), duration)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_outlined_rects(&mut self, world: &mut World) {
        let center_x = 330.0;
        let center_y = 100.0;

        let rect_configs: &[(f32, f32, f32, f32, [f32; 4])] = &[
            (-20.0, 15.0, 35.0, 35.0, [1.0, 0.6, 0.2, 1.0]),
            (20.0, -10.0, 28.0, 40.0, [0.2, 0.8, 1.0, 1.0]),
            (0.0, 0.0, 22.0, 22.0, [1.0, 1.0, 0.4, 0.9]),
        ];

        for (index, &(offset_x, offset_y, width, height, color)) in
            rect_configs.iter().enumerate()
        {
            let entity = spawn_outlined_rect(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                Vec2::new(width, height),
                color,
            );
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 4.0 - index as f32 * 0.1;
            }

            world.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 3.0 + index as f32 * 0.5;
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.85), TweenValue::F32(1.15), duration)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_soft_circles(&mut self, world: &mut World) {
        let center_x = 400.0;
        let center_y = -120.0;

        let glow_configs: &[(f32, f32, [f32; 4])] = &[
            (-20.0, 12.0, [1.0, 0.2, 0.4, 0.6]),
            (20.0, 12.0, [0.2, 0.5, 1.0, 0.6]),
            (0.0, -15.0, [0.3, 1.0, 0.4, 0.6]),
        ];

        for (index, &(offset_x, offset_y, color)) in glow_configs.iter().enumerate() {
            let entity = spawn_shape(
                world,
                SpriteShape::SoftCircle,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                Vec2::new(70.0, 70.0),
                color,
            );
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 2.0;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }

            world.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 2.5 + index as f32 * 0.8;
            let position = Vec2::new(center_x + offset_x, center_y + offset_y);
            let offset = 10.0;
            let angle = index as f32 * std::f32::consts::TAU / 3.0;
            let target = Vec2::new(
                position.x + offset * angle.cos(),
                position.y + offset * angle.sin(),
            );
            tween.add_track(
                TweenTrack::new(TweenValue::Vec2(position), TweenValue::Vec2(target), duration)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_POSITION),
            );
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.8), TweenValue::F32(1.3), duration * 0.6)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_lines(&mut self, world: &mut World) {
        let center_x = -400.0;
        let center_y = -120.0;

        let line_starts = [(-30.0_f32, -25.0_f32), (-25.0, 18.0), (-15.0, -20.0), (0.0, -25.0)];
        let line_ends = [(30.0_f32, 25.0_f32), (25.0, -18.0), (20.0, 20.0), (0.0, 25.0)];
        let line_thicknesses = [2.5_f32, 2.0, 1.5, 3.0];
        let line_colors: [[f32; 4]; 4] = [
            [1.0, 0.5, 0.2, 1.0],
            [0.3, 0.9, 0.5, 1.0],
            [0.5, 0.4, 1.0, 1.0],
            [1.0, 0.8, 0.3, 1.0],
        ];

        for index in 0..4 {
            let (start_x, start_y) = line_starts[index];
            let (end_x, end_y) = line_ends[index];
            let entity = spawn_line(
                world,
                Vec2::new(center_x + start_x, center_y + start_y),
                Vec2::new(center_x + end_x, center_y + end_y),
                line_thicknesses[index],
                line_colors[index],
            );
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 4.0;
            }
            self.demo_entities.push(entity);
        }

        let (line_entity, head_entity) = spawn_arrow(
            world,
            Vec2::new(center_x - 25.0, center_y - 15.0),
            Vec2::new(center_x + 25.0, center_y + 15.0),
            2.0,
            10.0,
            [0.8, 0.3, 0.8, 1.0],
        );
        if let Some(sprite) = world.get_sprite_mut(line_entity) {
            sprite.depth = 4.0;
        }
        if let Some(sprite) = world.get_sprite_mut(head_entity) {
            sprite.depth = 4.0;
        }
        self.demo_entities.push(line_entity);
        self.demo_entities.push(head_entity);
    }

    fn build_bezier_curves(&mut self, world: &mut World) {
        let center_x = -210.0;
        let center_y = -120.0;

        let quadratic_entities = spawn_quadratic_bezier(
            world,
            Vec2::new(center_x - 35.0, center_y - 25.0),
            Vec2::new(center_x, center_y + 35.0),
            Vec2::new(center_x + 35.0, center_y - 25.0),
            24,
            2.0,
            [1.0, 0.4, 0.3, 1.0],
        );
        for &entity in &quadratic_entities {
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 4.0;
            }
        }
        self.demo_entities.extend(quadratic_entities);

        let cubic_entities = spawn_cubic_bezier(
            world,
            &CubicBezier {
                start: Vec2::new(center_x - 35.0, center_y + 10.0),
                control_a: Vec2::new(center_x - 10.0, center_y - 30.0),
                control_b: Vec2::new(center_x + 10.0, center_y + 40.0),
                end: Vec2::new(center_x + 35.0, center_y + 10.0),
            },
            32,
            2.0,
            [0.3, 0.6, 1.0, 1.0],
        );
        for &entity in &cubic_entities {
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 4.0;
            }
        }
        self.demo_entities.extend(cubic_entities);

        let wave_entities = spawn_cubic_bezier(
            world,
            &CubicBezier {
                start: Vec2::new(center_x - 35.0, center_y - 5.0),
                control_a: Vec2::new(center_x - 15.0, center_y + 25.0),
                control_b: Vec2::new(center_x + 15.0, center_y - 25.0),
                end: Vec2::new(center_x + 35.0, center_y - 5.0),
            },
            28,
            1.5,
            [0.5, 1.0, 0.4, 0.8],
        );
        for &entity in &wave_entities {
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 4.0;
            }
        }
        self.demo_entities.extend(wave_entities);
    }

    fn build_paths(&mut self, world: &mut World) {
        let center_x = -10.0;
        let center_y = -120.0;

        let star_points: Vec<Vec2> = (0..10)
            .map(|index| {
                let angle = index as f32 * std::f32::consts::TAU / 10.0 - std::f32::consts::FRAC_PI_2;
                let radius = if index % 2 == 0 { 30.0 } else { 14.0 };
                Vec2::new(center_x + radius * angle.cos(), center_y + radius * angle.sin())
            })
            .collect();
        let star_entities = spawn_path(world, &star_points, 2.0, [1.0, 0.8, 0.2, 1.0], true);
        for &entity in &star_entities {
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 4.0;
            }
        }
        self.demo_entities.extend(star_entities);

        let hex_points: Vec<Vec2> = (0..6)
            .map(|index| {
                let angle = index as f32 * std::f32::consts::TAU / 6.0;
                Vec2::new(
                    center_x + 20.0 * angle.cos(),
                    center_y - 25.0 + 12.0 * angle.sin(),
                )
            })
            .collect();
        let hex_entities = spawn_path(world, &hex_points, 1.5, [0.4, 0.8, 1.0, 0.8], true);
        for &entity in &hex_entities {
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 3.5;
            }
        }
        self.demo_entities.extend(hex_entities);
    }

    fn build_gradients(&mut self, world: &mut World) {
        let center_x = 200.0;
        let center_y = -120.0;
        let gradient_size = 128;

        let linear_data = generate_linear_gradient_texture(
            gradient_size,
            gradient_size,
            [1.0, 0.2, 0.3, 1.0],
            [0.2, 0.4, 1.0, 1.0],
            true,
        );
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: 1,
            rgba_data: linear_data,
            width: gradient_size,
            height: gradient_size,
        });

        let radial_data = generate_radial_gradient_texture(
            gradient_size,
            [1.0, 0.9, 0.3, 1.0],
            [0.1, 0.0, 0.3, 0.0],
        );
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: 2,
            rgba_data: radial_data,
            width: gradient_size,
            height: gradient_size,
        });

        let rainbow_data = generate_multi_stop_gradient_texture(
            gradient_size,
            gradient_size,
            &[
                (0.0, [1.0, 0.0, 0.0, 1.0]),
                (0.2, [1.0, 0.5, 0.0, 1.0]),
                (0.4, [1.0, 1.0, 0.0, 1.0]),
                (0.6, [0.0, 1.0, 0.0, 1.0]),
                (0.8, [0.0, 0.5, 1.0, 1.0]),
                (1.0, [0.5, 0.0, 1.0, 1.0]),
            ],
            true,
        );
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: 3,
            rgba_data: rainbow_data,
            width: gradient_size,
            height: gradient_size,
        });

        let gradient_uv_max = gradient_size as f32 / 512.0;
        let half_texel = 0.5 / 512.0;
        let uv_min = Vec2::new(half_texel, half_texel);
        let uv_max = Vec2::new(gradient_uv_max - half_texel, gradient_uv_max - half_texel);

        let linear_entity = spawn_rect(
            world,
            Vec2::new(center_x - 30.0, center_y + 15.0),
            Vec2::new(45.0, 30.0),
            [1.0, 1.0, 1.0, 1.0],
        );
        if let Some(sprite) = world.get_sprite_mut(linear_entity) {
            sprite.texture_index = 1;
            sprite.texture_index2 = 1;
            sprite.uv_min = uv_min;
            sprite.uv_max = uv_max;
            sprite.depth = 4.0;
        }

        let radial_entity = spawn_rect(
            world,
            Vec2::new(center_x + 25.0, center_y + 15.0),
            Vec2::new(40.0, 40.0),
            [1.0, 1.0, 1.0, 1.0],
        );
        if let Some(sprite) = world.get_sprite_mut(radial_entity) {
            sprite.texture_index = 2;
            sprite.texture_index2 = 2;
            sprite.uv_min = uv_min;
            sprite.uv_max = uv_max;
            sprite.depth = 4.0;
        }

        let rainbow_entity = spawn_rect(
            world,
            Vec2::new(center_x, center_y - 22.0),
            Vec2::new(80.0, 18.0),
            [1.0, 1.0, 1.0, 1.0],
        );
        if let Some(sprite) = world.get_sprite_mut(rainbow_entity) {
            sprite.texture_index = 3;
            sprite.texture_index2 = 3;
            sprite.uv_min = uv_min;
            sprite.uv_max = uv_max;
            sprite.depth = 4.0;
        }
    }

    fn build_labels(&mut self, world: &mut World) {
        let title = spawn_sprite_text(
            world,
            "2D Graphics Primitives",
            Vec2::new(-130.0, 230.0),
            22.0,
        );
        if let Some(text) = world.get_sprite_text_mut(title) {
            text.color = [1.0, 1.0, 1.0, 1.0];
            text.depth = 1.0;
        }

        let top_label_y = 155.0;
        let top_labels: &[(&str, f32)] = &[
            ("Rects", -380.0),
            ("Circles", -230.0),
            ("Rings", -90.0),
            ("Triangles", 50.0),
            ("Capsules", 190.0),
            ("Outlined", 330.0),
        ];

        for &(label, center_x) in top_labels {
            let offset_x = label.len() as f32 * -3.5;
            let entity =
                spawn_sprite_text(world, label, Vec2::new(center_x + offset_x, top_label_y), 11.0);
            if let Some(text) = world.get_sprite_text_mut(entity) {
                text.color = [0.6, 0.6, 0.6, 1.0];
                text.depth = 1.0;
            }
        }

        let bottom_label_y = -70.0;
        let bottom_labels: &[(&str, f32)] = &[
            ("Lines", -400.0),
            ("Bezier", -210.0),
            ("Paths", -10.0),
            ("Gradients", 200.0),
            ("Soft Circles", 400.0),
        ];

        for &(label, center_x) in bottom_labels {
            let offset_x = label.len() as f32 * -3.5;
            let entity = spawn_sprite_text(
                world,
                label,
                Vec2::new(center_x + offset_x, bottom_label_y),
                11.0,
            );
            if let Some(text) = world.get_sprite_text_mut(entity) {
                text.color = [0.6, 0.6, 0.6, 1.0];
                text.depth = 1.0;
            }
        }

        let instructions = spawn_sprite_text(
            world,
            "Click to spawn | Scroll to switch shape | C clear",
            Vec2::new(-220.0, -250.0),
            11.0,
        );
        if let Some(text) = world.get_sprite_text_mut(instructions) {
            text.color = [0.5, 0.5, 0.5, 1.0];
            text.depth = 1.0;
        }
    }

    fn apply_tweens(&self, world: &mut World) {
        for &entity in &self.demo_entities {
            let tween_data = world.get_tween(entity).cloned();
            let Some(tween) = tween_data else {
                continue;
            };

            if let Some(track) = tween.track_by_tag(TAG_POSITION) {
                let position = track.value_vec2();
                if let Some(sprite) = world.get_sprite_mut(entity) {
                    sprite.position = position;
                }
            }

            if let Some(track) = tween.track_by_tag(TAG_SCALE) {
                let scale = track.value_f32();
                if let Some(sprite) = world.get_sprite_mut(entity) {
                    sprite.scale = Vec2::new(scale, scale);
                }
            }

            if let Some(track) = tween.track_by_tag(TAG_ROTATION) {
                let rotation = track.value_f32();
                if let Some(sprite) = world.get_sprite_mut(entity) {
                    sprite.rotation = rotation;
                }
            }
        }
    }

    fn screen_to_world(&self, world: &World, screen_position: Vec2) -> Vec2 {
        let window_size = world
            .resources
            .window
            .handle
            .as_ref()
            .map(|handle| {
                let size = handle.inner_size();
                Vec2::new(size.width as f32, size.height as f32)
            })
            .unwrap_or(Vec2::new(1920.0, 1080.0));

        let camera_position = self
            .camera_entity
            .and_then(|entity| world.get_local_transform(entity))
            .map(|transform| Vec2::new(transform.translation.x, transform.translation.y))
            .unwrap_or(Vec2::zeros());

        let half_view = self
            .camera_entity
            .and_then(|entity| world.get_camera(entity))
            .map(|camera| {
                if let Projection::Orthographic(ortho) = &camera.projection {
                    Vec2::new(ortho.x_mag, ortho.y_mag)
                } else {
                    Vec2::new(480.0, 270.0)
                }
            })
            .unwrap_or(Vec2::new(480.0, 270.0));

        let normalized_x = screen_position.x / window_size.x - 0.5;
        let normalized_y = -(screen_position.y / window_size.y - 0.5);

        Vec2::new(
            camera_position.x + normalized_x * 2.0 * half_view.x,
            camera_position.y + normalized_y * 2.0 * half_view.y,
        )
    }

    fn update_preview(&mut self, world: &mut World) {
        let screen_position = Vec2::new(
            world.resources.input.mouse.position.x,
            world.resources.input.mouse.position.y,
        );
        let world_position = self.screen_to_world(world, screen_position);

        if let Some(entity) = self.preview_entity
            && let Some(sprite) = world.get_sprite_mut(entity)
        {
            sprite.position = world_position;
        }
    }

    fn handle_input(&mut self, world: &mut World) {
        let keyboard = &world.resources.input.keyboard;

        let mut shape_changed = false;
        if keyboard.just_pressed(KeyCode::Digit1) {
            self.current_shape_index = 0;
            shape_changed = true;
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            self.current_shape_index = 1;
            shape_changed = true;
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            self.current_shape_index = 2;
            shape_changed = true;
        } else if keyboard.just_pressed(KeyCode::Digit4) {
            self.current_shape_index = 3;
            shape_changed = true;
        } else if keyboard.just_pressed(KeyCode::Digit5) {
            self.current_shape_index = 4;
            shape_changed = true;
        } else if keyboard.just_pressed(KeyCode::Digit6) {
            self.current_shape_index = 5;
            shape_changed = true;
        } else if keyboard.just_pressed(KeyCode::Digit7) {
            self.current_shape_index = 6;
            shape_changed = true;
        }

        let mouse_state = world.resources.input.mouse.state;
        if mouse_state.contains(MouseState::SCROLLED) {
            let scroll_y = world.resources.input.mouse.wheel_delta.y;
            if scroll_y > 0.0 {
                self.current_shape_index =
                    (self.current_shape_index + 1) % ALL_SHAPES.len();
                shape_changed = true;
            } else if scroll_y < 0.0 {
                self.current_shape_index = if self.current_shape_index == 0 {
                    ALL_SHAPES.len() - 1
                } else {
                    self.current_shape_index - 1
                };
                shape_changed = true;
            }
        }

        if shape_changed
            && let Some(entity) = self.preview_entity
        {
            configure_preview_for_shape(world, entity, current_shape(self));
        }

        if world.resources.input.keyboard.just_pressed(KeyCode::KeyC)
            && !self.spawned_entities.is_empty()
        {
            let entities: Vec<Entity> = self.spawned_entities.drain(..).collect();
            world.despawn_entities(&entities);
            self.next_spawn_depth = 0.0;
        }

        if mouse_state.contains(MouseState::LEFT_JUST_PRESSED) {
            let screen_position = Vec2::new(
                world.resources.input.mouse.position.x,
                world.resources.input.mouse.position.y,
            );
            let world_position = self.screen_to_world(world, screen_position);

            let uptime = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
            let hue = (uptime * 0.15).fract();
            let (red, green, blue) = hue_to_rgb(hue);
            let shape = current_shape(self);

            let entity = match shape {
                SpriteShape::Rect => spawn_rect(
                    world,
                    world_position,
                    Vec2::new(40.0, 40.0),
                    [red, green, blue, 0.9],
                ),
                SpriteShape::Circle => {
                    spawn_circle(world, world_position, 20.0, [red, green, blue, 0.9])
                }
                SpriteShape::Ring => {
                    spawn_ring(world, world_position, 22.0, [red, green, blue, 0.9])
                }
                SpriteShape::Triangle => spawn_triangle(
                    world,
                    world_position,
                    Vec2::new(40.0, 40.0),
                    [red, green, blue, 0.9],
                ),
                SpriteShape::Capsule => spawn_capsule(
                    world,
                    world_position,
                    Vec2::new(60.0, 24.0),
                    [red, green, blue, 0.9],
                ),
                SpriteShape::OutlinedRect => spawn_outlined_rect(
                    world,
                    world_position,
                    Vec2::new(40.0, 40.0),
                    [red, green, blue, 0.9],
                ),
                SpriteShape::SoftCircle => {
                    let entity = spawn_shape(
                        world,
                        SpriteShape::SoftCircle,
                        world_position,
                        Vec2::new(60.0, 60.0),
                        [red, green, blue, 0.7],
                    );
                    if let Some(sprite) = world.get_sprite_mut(entity) {
                        sprite.blend_mode = SpriteBlendMode::Additive;
                    }
                    entity
                }
            };

            self.next_spawn_depth += 0.01;
            if let Some(sprite) = world.get_sprite_mut(entity) {
                sprite.depth = 10.0 + self.next_spawn_depth;
            }
            self.spawned_entities.push(entity);
        }
    }
}

impl State for GfxDemo {
    fn title(&self) -> &str {
        "2D Graphics Primitives"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.06, 0.06, 0.1, 1.0];

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        self.camera_entity = Some(camera);

        if let Some(camera_data) = world.get_camera_mut(camera)
            && let Projection::Orthographic(ref mut ortho) = camera_data.projection
        {
            ortho.x_mag = 480.0;
            ortho.y_mag = 270.0;
        }

        self.build_scene(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        self.handle_input(world);
        self.update_preview(world);
        self.apply_tweens(world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("2D Graphics Primitives")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                let fps = world.resources.window.timing.frames_per_second;
                ui.label(format!("FPS: {fps:.0}"));
                ui.separator();

                let shape = current_shape(self);
                ui.label(format!("Shape: {shape:?} [{}]", self.current_shape_index + 1));
                ui.label(format!("Spawned: {}", self.spawned_entities.len()));
                ui.separator();

                ui.label("1-7 or Scroll to switch | Click to spawn | C to clear");
            });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(GfxDemo::default())
}
