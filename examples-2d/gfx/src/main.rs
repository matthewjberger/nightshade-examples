use nightshade::ecs::input::resources::mouse::MouseState;
use nightshade::ecs::world::commands::WorldCommand;
use nightshade::prelude::*;
use nightshade::render::{
    boolean_intersect, boolean_subtract, boolean_union, generate_circle_texture_with_aa,
    generate_filled_polygon_texture, generate_linear_gradient_texture,
    generate_multi_stop_gradient_texture, generate_outlined_rect_texture_with_border,
    generate_radial_gradient_texture, generate_ring_texture_with_thickness,
    generate_rounded_rect_texture,
};

type EllipseConfig = (f32, f32, f32, f32, f32, [f32; 4]);
type RoundedRectConfig = (f32, f32, f32, f32, u32, [f32; 4]);
type DashedLineConfig = (f32, f32, f32, f32, f32, f32, f32, [f32; 4]);
type FillStrokeEllipseConfig = (f32, f32, f32, f32, [f32; 4], [f32; 4]);

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

const ROUNDED_RECT_SLOT: u32 = 4;
const ROUNDED_RECT_SLOT_B: u32 = 5;
const ROUNDED_RECT_SLOT_C: u32 = 6;
const POLYGON_STAR_SLOT: u32 = 7;
const POLYGON_HEX_SLOT: u32 = 8;
const RING_THIN_SLOT: u32 = 9;
const RING_MEDIUM_SLOT: u32 = 10;
const RING_THICK_SLOT: u32 = 11;
const OUTLINED_RECT_THIN_SLOT: u32 = 12;
const OUTLINED_RECT_MEDIUM_SLOT: u32 = 13;
const OUTLINED_RECT_THICK_SLOT: u32 = 14;
const ELLIPSE_RING_SLOT: u32 = 15;
const BOOLEAN_UNION_SLOT: u32 = 16;
const BOOLEAN_SUBTRACT_SLOT: u32 = 17;
const BOOLEAN_INTERSECT_SLOT: u32 = 18;
const AA_HARD_SLOT: u32 = 19;
const AA_DEFAULT_SLOT: u32 = 20;
const AA_SOFT_SLOT: u32 = 21;
const SHADOW_SOURCE_SLOT: u32 = 22;

#[derive(Default)]
struct GfxDemo {
    camera_entity: Option<Entity>,
    current_shape_index: usize,
    spawned_entities: Vec<Entity>,
    demo_entities: Vec<Entity>,
    preview_entity: Option<Entity>,
    next_spawn_depth: f32,
    zoom_line_entities: Vec<Entity>,
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

    if let Some(sprite) = world.core.get_sprite_mut(entity) {
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

const ROW_1_Y: f32 = 160.0;
const ROW_2_Y: f32 = 0.0;
const ROW_3_Y: f32 = -160.0;
const ROW_4_Y: f32 = -320.0;
const ROW_5_Y: f32 = -480.0;

const COL_1_X: f32 = -380.0;
const COL_2_X: f32 = -230.0;
const COL_3_X: f32 = -80.0;
const COL_4_X: f32 = 70.0;
const COL_5_X: f32 = 220.0;
const COL_6_X: f32 = 370.0;

impl GfxDemo {
    fn build_scene(&mut self, world: &mut World) {
        self.build_rect_grid(world);
        self.build_circles(world);
        self.build_ellipses(world);
        self.build_rings(world);
        self.build_triangles(world);
        self.build_capsules(world);

        self.build_rounded_rects(world);
        self.build_outlined_rects(world);
        self.build_fill_and_stroke(world);
        self.build_soft_circles(world);
        self.build_screen_blend(world);
        self.build_gradients(world);

        self.build_lines(world);
        self.build_dashed_lines(world);
        self.build_variable_width(world);
        self.build_bezier_curves(world);
        self.build_paths(world);
        self.build_polygons(world);

        self.build_param_rings(world);
        self.build_param_outlines(world);
        self.build_fill_stroke_ellipse(world);
        self.build_clip_rects(world);
        self.build_zoom_lines(world);

        world.resources.sprite_slot_allocator.next_slot = 23;
        self.build_boolean_ops(world);
        self.build_aa_control(world);
        self.build_shadow_demo(world);
        self.build_glow_demo(world);
        self.build_stencil_demo(world);
        self.build_path_fill_demo(world);

        self.build_labels(world);
        self.build_preview(world);
    }

    fn build_preview(&mut self, world: &mut World) {
        let entity = spawn_circle(world, Vec2::new(0.0, 0.0), 20.0, [1.0, 1.0, 1.0, 0.3]);
        if let Some(sprite) = world.core.get_sprite_mut(entity) {
            sprite.depth = 20.0;
        }
        configure_preview_for_shape(world, entity, current_shape(self));
        self.preview_entity = Some(entity);
    }

    fn build_rect_grid(&mut self, world: &mut World) {
        let center_x = COL_1_X;
        let center_y = ROW_1_Y;
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
                if let Some(sprite) = world.core.get_sprite_mut(entity) {
                    sprite.depth = 5.0;
                }

                world.core.add_components(entity, TWEEN);
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
                world.core.set_tween(entity, tween);
                self.demo_entities.push(entity);
            }
        }
    }

    fn build_circles(&mut self, world: &mut World) {
        let center_x = COL_2_X;
        let center_y = ROW_1_Y;

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
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0 - index as f32 * 0.1;
            }

            world.core.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 4.0 + index as f32 * 0.5;
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.85), TweenValue::F32(1.2), duration)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.core.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_ellipses(&mut self, world: &mut World) {
        let center_x = COL_3_X;
        let center_y = ROW_1_Y;

        let ellipse_configs: &[EllipseConfig] = &[
            (-15.0, 15.0, 25.0, 14.0, 0.0, [0.9, 0.4, 0.8, 1.0]),
            (15.0, -10.0, 14.0, 22.0, 0.5, [0.4, 0.9, 0.6, 1.0]),
            (0.0, 0.0, 20.0, 10.0, -0.3, [0.5, 0.6, 1.0, 0.9]),
        ];

        for (index, &(offset_x, offset_y, radius_x, radius_y, rotation, color)) in
            ellipse_configs.iter().enumerate()
        {
            let entity = spawn_ellipse(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                radius_x,
                radius_y,
                color,
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0 - index as f32 * 0.1;
                sprite.rotation = rotation;
            }

            world.core.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 3.0 + index as f32 * 0.8;
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.85), TweenValue::F32(1.15), duration)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.core.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_rings(&mut self, world: &mut World) {
        let center_x = COL_4_X;
        let center_y = ROW_1_Y;

        let ring_configs: &[(f32, [f32; 4])] = &[
            (35.0, [1.0, 0.4, 0.1, 0.9]),
            (27.0, [0.1, 0.8, 0.4, 0.9]),
            (19.0, [0.3, 0.4, 1.0, 0.9]),
            (12.0, [1.0, 0.9, 0.2, 0.9]),
        ];

        for (index, &(radius, color)) in ring_configs.iter().enumerate() {
            let entity = spawn_ring(world, Vec2::new(center_x, center_y), radius, color);
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 3.0;
            }

            world.core.add_components(entity, TWEEN);
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
            world.core.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_triangles(&mut self, world: &mut World) {
        let center_x = COL_5_X;
        let center_y = ROW_1_Y;

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
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0 - index as f32 * 0.1;
            }

            world.core.add_components(entity, TWEEN);
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
            world.core.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_capsules(&mut self, world: &mut World) {
        let center_x = COL_6_X;
        let center_y = ROW_1_Y;

        let offsets = [(0.0_f32, 20.0_f32), (0.0, -5.0), (0.0, -25.0)];
        let sizes = [(60.0_f32, 22.0_f32), (50.0, 18.0), (40.0, 16.0)];
        let rotations = [0.0_f32, 0.4, -0.3];
        let colors: [[f32; 4]; 3] = [
            [0.9, 0.3, 0.8, 1.0],
            [0.3, 0.9, 0.6, 1.0],
            [0.4, 0.5, 1.0, 1.0],
        ];

        for index in 0..3 {
            let entity = spawn_capsule(
                world,
                Vec2::new(center_x + offsets[index].0, center_y + offsets[index].1),
                Vec2::new(sizes[index].0, sizes[index].1),
                colors[index],
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0;
                sprite.rotation = rotations[index];
            }

            world.core.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 2.5 + index as f32 * 0.6;
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.9), TweenValue::F32(1.15), duration)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.core.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_rounded_rects(&mut self, world: &mut World) {
        let center_x = COL_1_X;
        let center_y = ROW_2_Y;
        let texture_size = 128;

        let data_a = generate_rounded_rect_texture(texture_size, 20.0);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: ROUNDED_RECT_SLOT,
            rgba_data: data_a,
            width: texture_size,
            height: texture_size,
        });

        let data_b = generate_rounded_rect_texture(texture_size, 40.0);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: ROUNDED_RECT_SLOT_B,
            rgba_data: data_b,
            width: texture_size,
            height: texture_size,
        });

        let data_c = generate_rounded_rect_texture(texture_size, 64.0);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: ROUNDED_RECT_SLOT_C,
            rgba_data: data_c,
            width: texture_size,
            height: texture_size,
        });

        let configs: &[RoundedRectConfig] = &[
            (
                -25.0,
                12.0,
                40.0,
                30.0,
                ROUNDED_RECT_SLOT,
                [0.9, 0.4, 0.3, 1.0],
            ),
            (
                20.0,
                -8.0,
                35.0,
                35.0,
                ROUNDED_RECT_SLOT_B,
                [0.3, 0.8, 0.5, 1.0],
            ),
            (
                0.0,
                -25.0,
                50.0,
                20.0,
                ROUNDED_RECT_SLOT_C,
                [0.4, 0.5, 1.0, 1.0],
            ),
        ];

        for (index, &(offset_x, offset_y, width, height, slot, color)) in configs.iter().enumerate()
        {
            let entity = spawn_rounded_rect(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                Vec2::new(width, height),
                slot,
                color,
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0 - index as f32 * 0.1;
            }
            self.demo_entities.push(entity);
        }
    }

    fn build_outlined_rects(&mut self, world: &mut World) {
        let center_x = COL_2_X;
        let center_y = ROW_2_Y;

        let rect_configs: &[(f32, f32, f32, f32, [f32; 4])] = &[
            (-20.0, 15.0, 35.0, 35.0, [1.0, 0.6, 0.2, 1.0]),
            (20.0, -10.0, 28.0, 40.0, [0.2, 0.8, 1.0, 1.0]),
            (0.0, 0.0, 22.0, 22.0, [1.0, 1.0, 0.4, 0.9]),
        ];

        for (index, &(offset_x, offset_y, width, height, color)) in rect_configs.iter().enumerate()
        {
            let entity = spawn_outlined_rect(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                Vec2::new(width, height),
                color,
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0 - index as f32 * 0.1;
            }

            world.core.add_components(entity, TWEEN);
            let mut tween = Tween::new();
            let duration = 3.0 + index as f32 * 0.5;
            tween.add_track(
                TweenTrack::new(TweenValue::F32(0.85), TweenValue::F32(1.15), duration)
                    .with_easing(EasingFunction::SineInOut)
                    .with_loop_mode(TweenLoopMode::PingPong)
                    .with_tag(TAG_SCALE),
            );
            world.core.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_fill_and_stroke(&mut self, world: &mut World) {
        let center_x = COL_3_X;
        let center_y = ROW_2_Y;

        let (fill_circle, stroke_circle) = spawn_filled_and_stroked_circle(
            world,
            Vec2::new(center_x - 18.0, center_y + 10.0),
            18.0,
            [0.3, 0.6, 1.0, 0.8],
            [1.0, 1.0, 1.0, 1.0],
        );
        if let Some(sprite) = world.core.get_sprite_mut(fill_circle) {
            sprite.depth = 4.1;
        }
        if let Some(sprite) = world.core.get_sprite_mut(stroke_circle) {
            sprite.depth = 4.0;
        }
        self.demo_entities.push(fill_circle);
        self.demo_entities.push(stroke_circle);

        let (fill_rect, stroke_rect) = spawn_filled_and_stroked_rect(
            world,
            Vec2::new(center_x + 18.0, center_y + 10.0),
            Vec2::new(30.0, 30.0),
            [1.0, 0.4, 0.3, 0.8],
            [1.0, 1.0, 1.0, 1.0],
        );
        if let Some(sprite) = world.core.get_sprite_mut(fill_rect) {
            sprite.depth = 4.1;
        }
        if let Some(sprite) = world.core.get_sprite_mut(stroke_rect) {
            sprite.depth = 4.0;
        }
        self.demo_entities.push(fill_rect);
        self.demo_entities.push(stroke_rect);

        let (fill_circle_b, stroke_circle_b) = spawn_filled_and_stroked_circle(
            world,
            Vec2::new(center_x, center_y - 20.0),
            14.0,
            [0.4, 0.9, 0.4, 0.8],
            [0.9, 0.9, 0.3, 1.0],
        );
        if let Some(sprite) = world.core.get_sprite_mut(fill_circle_b) {
            sprite.depth = 4.1;
        }
        if let Some(sprite) = world.core.get_sprite_mut(stroke_circle_b) {
            sprite.depth = 4.0;
        }
        self.demo_entities.push(fill_circle_b);
        self.demo_entities.push(stroke_circle_b);
    }

    fn build_soft_circles(&mut self, world: &mut World) {
        let center_x = COL_4_X;
        let center_y = ROW_2_Y;

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
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 2.0;
                sprite.blend_mode = SpriteBlendMode::Additive;
            }

            world.core.add_components(entity, TWEEN);
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
                TweenTrack::new(
                    TweenValue::Vec2(position),
                    TweenValue::Vec2(target),
                    duration,
                )
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
            world.core.set_tween(entity, tween);
            self.demo_entities.push(entity);
        }
    }

    fn build_screen_blend(&mut self, world: &mut World) {
        let center_x = COL_5_X;
        let center_y = ROW_2_Y;

        let background = spawn_rect(
            world,
            Vec2::new(center_x, center_y),
            Vec2::new(80.0, 60.0),
            [0.15, 0.15, 0.2, 1.0],
        );
        if let Some(sprite) = world.core.get_sprite_mut(background) {
            sprite.depth = 3.0;
        }
        self.demo_entities.push(background);

        let screen_configs: &[(f32, f32, f32, [f32; 4])] = &[
            (-15.0, 8.0, 20.0, [0.8, 0.2, 0.1, 1.0]),
            (10.0, 8.0, 18.0, [0.1, 0.4, 0.8, 1.0]),
            (0.0, -10.0, 16.0, [0.2, 0.7, 0.2, 1.0]),
        ];

        for &(offset_x, offset_y, radius, color) in screen_configs {
            let entity = spawn_circle(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                radius,
                color,
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0;
                sprite.blend_mode = SpriteBlendMode::Screen;
            }
            self.demo_entities.push(entity);
        }
    }

    fn build_gradients(&mut self, world: &mut World) {
        let center_x = COL_6_X;
        let center_y = ROW_2_Y;
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
        if let Some(sprite) = world.core.get_sprite_mut(linear_entity) {
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
        if let Some(sprite) = world.core.get_sprite_mut(radial_entity) {
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
        if let Some(sprite) = world.core.get_sprite_mut(rainbow_entity) {
            sprite.texture_index = 3;
            sprite.texture_index2 = 3;
            sprite.uv_min = uv_min;
            sprite.uv_max = uv_max;
            sprite.depth = 4.0;
        }
    }

    fn build_lines(&mut self, world: &mut World) {
        let center_x = COL_1_X;
        let center_y = ROW_3_Y;

        let line_starts = [
            (-30.0_f32, -25.0_f32),
            (-25.0, 18.0),
            (-15.0, -20.0),
            (0.0, -25.0),
        ];
        let line_ends = [
            (30.0_f32, 25.0_f32),
            (25.0, -18.0),
            (20.0, 20.0),
            (0.0, 25.0),
        ];
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
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
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
        if let Some(sprite) = world.core.get_sprite_mut(line_entity) {
            sprite.depth = 4.0;
        }
        if let Some(sprite) = world.core.get_sprite_mut(head_entity) {
            sprite.depth = 4.0;
        }
        self.demo_entities.push(line_entity);
        self.demo_entities.push(head_entity);
    }

    fn build_dashed_lines(&mut self, world: &mut World) {
        let center_x = COL_2_X;
        let center_y = ROW_3_Y;

        let dashed_configs: &[DashedLineConfig] = &[
            (-30.0, 20.0, 30.0, 20.0, 2.0, 8.0, 4.0, [1.0, 0.5, 0.2, 1.0]),
            (-25.0, 0.0, 25.0, 0.0, 1.5, 6.0, 3.0, [0.3, 0.8, 1.0, 1.0]),
            (
                -30.0,
                -20.0,
                30.0,
                -20.0,
                2.5,
                12.0,
                6.0,
                [0.5, 1.0, 0.4, 1.0],
            ),
        ];

        for &(start_x, start_y, end_x, end_y, thickness, dash, gap, color) in dashed_configs {
            let entities = spawn_dashed_line(
                world,
                Vec2::new(center_x + start_x, center_y + start_y),
                Vec2::new(center_x + end_x, center_y + end_y),
                thickness,
                dash,
                gap,
                color,
            );
            for &entity in &entities {
                if let Some(sprite) = world.core.get_sprite_mut(entity) {
                    sprite.depth = 4.0;
                }
            }
            self.demo_entities.extend(entities);
        }
    }

    fn build_variable_width(&mut self, world: &mut World) {
        let center_x = COL_3_X;
        let center_y = ROW_3_Y;

        let brush_points: Vec<Vec2> = (0..20)
            .map(|index| {
                let parameter = index as f32 / 19.0;
                let x = center_x - 30.0 + parameter * 60.0;
                let y = center_y + (parameter * std::f32::consts::PI * 2.0).sin() * 15.0;
                Vec2::new(x, y)
            })
            .collect();

        let pressures: Vec<f32> = (0..20)
            .map(|index| {
                let parameter = index as f32 / 19.0;
                0.3 + 0.7 * (parameter * std::f32::consts::PI).sin()
            })
            .collect();

        let entities =
            spawn_variable_width_path(world, &brush_points, &pressures, 5.0, [0.9, 0.5, 0.2, 1.0]);
        for &entity in &entities {
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0;
            }
        }
        self.demo_entities.extend(entities);

        let ribbon_points: Vec<Vec2> = (0..15)
            .map(|index| {
                let parameter = index as f32 / 14.0;
                let x = center_x - 25.0 + parameter * 50.0;
                let y = center_y - 20.0 + (parameter * std::f32::consts::PI * 3.0).sin() * 8.0;
                Vec2::new(x, y)
            })
            .collect();

        let ribbon_pressures: Vec<f32> = (0..15)
            .map(|index| {
                let parameter = index as f32 / 14.0;
                0.2 + 0.8 * (1.0 - (parameter * 2.0 - 1.0).abs())
            })
            .collect();

        let ribbon_entities = spawn_variable_width_path(
            world,
            &ribbon_points,
            &ribbon_pressures,
            4.0,
            [0.4, 0.6, 1.0, 1.0],
        );
        for &entity in &ribbon_entities {
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 3.9;
            }
        }
        self.demo_entities.extend(ribbon_entities);
    }

    fn build_bezier_curves(&mut self, world: &mut World) {
        let center_x = COL_4_X;
        let center_y = ROW_3_Y;

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
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
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
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
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
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0;
            }
        }
        self.demo_entities.extend(wave_entities);
    }

    fn build_paths(&mut self, world: &mut World) {
        let center_x = COL_5_X;
        let center_y = ROW_3_Y;

        let star_points: Vec<Vec2> = (0..10)
            .map(|index| {
                let angle =
                    index as f32 * std::f32::consts::TAU / 10.0 - std::f32::consts::FRAC_PI_2;
                let radius = if index % 2 == 0 { 30.0 } else { 14.0 };
                Vec2::new(
                    center_x + radius * angle.cos(),
                    center_y + radius * angle.sin(),
                )
            })
            .collect();
        let star_entities = spawn_path(world, &star_points, 2.0, [1.0, 0.8, 0.2, 1.0], true);
        for &entity in &star_entities {
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
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
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 3.5;
            }
        }
        self.demo_entities.extend(hex_entities);
    }

    fn build_polygons(&mut self, world: &mut World) {
        let center_x = COL_6_X;
        let center_y = ROW_3_Y;
        let texture_size = 128;

        let star_points: Vec<[f32; 2]> = (0..10)
            .map(|index| {
                let angle =
                    index as f32 * std::f32::consts::TAU / 10.0 - std::f32::consts::FRAC_PI_2;
                let radius = if index % 2 == 0 { 0.48 } else { 0.22 };
                [0.5 + radius * angle.cos(), 0.5 + radius * angle.sin()]
            })
            .collect();
        let star_data = generate_filled_polygon_texture(texture_size, texture_size, &star_points);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: POLYGON_STAR_SLOT,
            rgba_data: star_data,
            width: texture_size,
            height: texture_size,
        });

        let hex_points: Vec<[f32; 2]> = (0..6)
            .map(|index| {
                let angle = index as f32 * std::f32::consts::TAU / 6.0;
                [0.5 + 0.45 * angle.cos(), 0.5 + 0.45 * angle.sin()]
            })
            .collect();
        let hex_data = generate_filled_polygon_texture(texture_size, texture_size, &hex_points);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: POLYGON_HEX_SLOT,
            rgba_data: hex_data,
            width: texture_size,
            height: texture_size,
        });

        let star_entity = spawn_filled_polygon(
            world,
            Vec2::new(center_x - 20.0, center_y + 5.0),
            Vec2::new(50.0, 50.0),
            POLYGON_STAR_SLOT,
            [1.0, 0.8, 0.2, 1.0],
        );
        if let Some(sprite) = world.core.get_sprite_mut(star_entity) {
            sprite.depth = 4.0;
        }
        self.demo_entities.push(star_entity);

        let hex_entity = spawn_filled_polygon(
            world,
            Vec2::new(center_x + 20.0, center_y - 10.0),
            Vec2::new(40.0, 40.0),
            POLYGON_HEX_SLOT,
            [0.4, 0.7, 1.0, 1.0],
        );
        if let Some(sprite) = world.core.get_sprite_mut(hex_entity) {
            sprite.depth = 4.0;
        }
        self.demo_entities.push(hex_entity);
    }

    fn build_param_rings(&mut self, world: &mut World) {
        let center_x = COL_1_X;
        let center_y = ROW_4_Y;
        let texture_size = 128;

        let thicknesses = [0.1_f32, 0.3, 0.8];
        let slots = [RING_THIN_SLOT, RING_MEDIUM_SLOT, RING_THICK_SLOT];
        let colors: [[f32; 4]; 3] = [
            [1.0, 0.4, 0.3, 1.0],
            [0.3, 0.8, 1.0, 1.0],
            [0.5, 1.0, 0.4, 1.0],
        ];
        let offsets = [-25.0_f32, 0.0, 25.0];

        let uv_max_coord = texture_size as f32 / 512.0;
        let half_texel = 0.5 / 512.0;
        let uv_min = Vec2::new(half_texel, half_texel);
        let uv_max = Vec2::new(uv_max_coord - half_texel, uv_max_coord - half_texel);

        for index in 0..3 {
            let data = generate_ring_texture_with_thickness(texture_size, thicknesses[index]);
            world.queue_command(WorldCommand::UploadSpriteTexture {
                slot: slots[index],
                rgba_data: data,
                width: texture_size,
                height: texture_size,
            });

            let entity = spawn_sprite(
                world,
                Vec2::new(center_x + offsets[index], center_y),
                Vec2::new(40.0, 40.0),
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.texture_index = slots[index];
                sprite.texture_index2 = slots[index];
                sprite.uv_min = uv_min;
                sprite.uv_max = uv_max;
                sprite.color = colors[index];
                sprite.depth = 4.0;
            }
            self.demo_entities.push(entity);
        }
    }

    fn build_param_outlines(&mut self, world: &mut World) {
        let center_x = COL_2_X;
        let center_y = ROW_4_Y;
        let texture_size = 128;

        let border_widths = [2.0_f32, 6.0, 15.0];
        let slots = [
            OUTLINED_RECT_THIN_SLOT,
            OUTLINED_RECT_MEDIUM_SLOT,
            OUTLINED_RECT_THICK_SLOT,
        ];
        let colors: [[f32; 4]; 3] = [
            [1.0, 0.8, 0.2, 1.0],
            [0.8, 0.3, 1.0, 1.0],
            [0.3, 1.0, 0.7, 1.0],
        ];
        let offsets = [-25.0_f32, 0.0, 25.0];

        let uv_max_coord = texture_size as f32 / 512.0;
        let half_texel = 0.5 / 512.0;
        let uv_min = Vec2::new(half_texel, half_texel);
        let uv_max = Vec2::new(uv_max_coord - half_texel, uv_max_coord - half_texel);

        for index in 0..3 {
            let data =
                generate_outlined_rect_texture_with_border(texture_size, border_widths[index]);
            world.queue_command(WorldCommand::UploadSpriteTexture {
                slot: slots[index],
                rgba_data: data,
                width: texture_size,
                height: texture_size,
            });

            let entity = spawn_sprite(
                world,
                Vec2::new(center_x + offsets[index], center_y),
                Vec2::new(35.0, 35.0),
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.texture_index = slots[index];
                sprite.texture_index2 = slots[index];
                sprite.uv_min = uv_min;
                sprite.uv_max = uv_max;
                sprite.color = colors[index];
                sprite.depth = 4.0;
            }
            self.demo_entities.push(entity);
        }
    }

    fn build_fill_stroke_ellipse(&mut self, world: &mut World) {
        let center_x = COL_3_X;
        let center_y = ROW_4_Y;
        let texture_size = 128;

        let ring_data = generate_ring_texture_with_thickness(texture_size, 0.15);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: ELLIPSE_RING_SLOT,
            rgba_data: ring_data,
            width: texture_size,
            height: texture_size,
        });

        let configs: &[FillStrokeEllipseConfig] = &[
            (
                -20.0,
                10.0,
                22.0,
                14.0,
                [0.3, 0.6, 1.0, 0.8],
                [1.0, 1.0, 1.0, 1.0],
            ),
            (
                15.0,
                -10.0,
                14.0,
                22.0,
                [1.0, 0.4, 0.3, 0.8],
                [1.0, 0.9, 0.3, 1.0],
            ),
            (
                0.0,
                -25.0,
                28.0,
                10.0,
                [0.4, 0.9, 0.4, 0.8],
                [1.0, 1.0, 1.0, 1.0],
            ),
        ];

        for (index, &(offset_x, offset_y, radius_x, radius_y, fill_color, stroke_color)) in
            configs.iter().enumerate()
        {
            let (fill_entity, stroke_entity) = spawn_filled_and_stroked_ellipse(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                radius_x,
                radius_y,
                fill_color,
                stroke_color,
                ELLIPSE_RING_SLOT,
            );
            if let Some(sprite) = world.core.get_sprite_mut(fill_entity) {
                sprite.depth = 4.1 - index as f32 * 0.01;
            }
            if let Some(sprite) = world.core.get_sprite_mut(stroke_entity) {
                sprite.depth = 4.0 - index as f32 * 0.01;
            }
            self.demo_entities.push(fill_entity);
            self.demo_entities.push(stroke_entity);
        }
    }

    fn build_clip_rects(&mut self, world: &mut World) {
        let center_x = COL_4_X;
        let center_y = ROW_4_Y;

        let clip_region = [
            center_x - 30.0,
            center_y - 25.0,
            center_x + 30.0,
            center_y + 25.0,
        ];

        let border_entity = spawn_outlined_rect(
            world,
            Vec2::new(center_x, center_y),
            Vec2::new(60.0, 50.0),
            [0.5, 0.5, 0.5, 0.4],
        );
        if let Some(sprite) = world.core.get_sprite_mut(border_entity) {
            sprite.depth = 3.5;
        }
        self.demo_entities.push(border_entity);

        let circle_configs: &[(f32, f32, f32, [f32; 4])] = &[
            (-25.0, 15.0, 20.0, [1.0, 0.3, 0.3, 1.0]),
            (0.0, 0.0, 22.0, [0.3, 0.6, 1.0, 1.0]),
            (25.0, -15.0, 18.0, [0.4, 1.0, 0.4, 1.0]),
            (-10.0, -20.0, 16.0, [1.0, 0.8, 0.2, 1.0]),
        ];

        for &(offset_x, offset_y, radius, color) in circle_configs {
            let entity = spawn_circle(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                radius,
                color,
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0;
            }
            self.demo_entities.push(entity);
        }

        let window_size = world
            .resources
            .window
            .handle
            .as_ref()
            .map(|handle| {
                let size = handle.inner_size();
                (size.width as f32, size.height as f32)
            })
            .unwrap_or((1920.0, 1080.0));

        let half_view = self
            .camera_entity
            .and_then(|entity| world.core.get_camera(entity))
            .map(|camera| {
                if let Projection::Orthographic(ortho) = &camera.projection {
                    (ortho.x_mag, ortho.y_mag)
                } else {
                    (480.0, 560.0)
                }
            })
            .unwrap_or((480.0, 560.0));

        let world_to_screen_x =
            |world_x: f32| -> f32 { (world_x / (half_view.0 * 2.0) + 0.5) * window_size.0 };
        let world_to_screen_y =
            |world_y: f32| -> f32 { (-world_y / (half_view.1 * 2.0) + 0.5) * window_size.1 };

        let screen_clip = [
            world_to_screen_x(clip_region[0]),
            world_to_screen_y(clip_region[3]),
            world_to_screen_x(clip_region[2]),
            world_to_screen_y(clip_region[1]),
        ];

        let clipped_configs: &[(f32, f32, f32, [f32; 4])] = &[
            (-25.0, 15.0, 20.0, [1.0, 0.3, 0.3, 1.0]),
            (0.0, 0.0, 22.0, [0.3, 0.6, 1.0, 1.0]),
            (25.0, -15.0, 18.0, [0.4, 1.0, 0.4, 1.0]),
            (-10.0, -20.0, 16.0, [1.0, 0.8, 0.2, 1.0]),
        ];

        for &(offset_x, offset_y, radius, color) in clipped_configs {
            let entity = spawn_circle(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                radius,
                [color[0], color[1], color[2], 0.6],
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 5.0;
                sprite.clip_rect = Some(screen_clip);
            }
            self.demo_entities.push(entity);
        }
    }

    fn build_zoom_lines(&mut self, world: &mut World) {
        let center_x = COL_5_X;
        let center_y = ROW_4_Y;

        let line_configs: &[(f32, f32, f32, f32, [f32; 4])] = &[
            (-30.0, -25.0, 30.0, 25.0, [1.0, 0.5, 0.2, 1.0]),
            (-25.0, 20.0, 25.0, -20.0, [0.3, 0.9, 0.5, 1.0]),
            (-30.0, 0.0, 30.0, 0.0, [0.5, 0.4, 1.0, 1.0]),
            (0.0, -25.0, 0.0, 25.0, [1.0, 0.8, 0.3, 1.0]),
        ];

        let pixel_thickness = 2.0;
        let world_thickness = screen_pixels_to_world_size(world, pixel_thickness);

        for &(start_x, start_y, end_x, end_y, color) in line_configs {
            let entity = spawn_line(
                world,
                Vec2::new(center_x + start_x, center_y + start_y),
                Vec2::new(center_x + end_x, center_y + end_y),
                world_thickness,
                color,
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0;
            }
            self.zoom_line_entities.push(entity);
            self.demo_entities.push(entity);
        }
    }

    fn build_boolean_ops(&mut self, world: &mut World) {
        let center_x = COL_1_X;
        let center_y = ROW_5_Y;
        let texture_size = 128;

        let circle_a = generate_circle_texture_with_aa(texture_size, 1.0);
        let circle_b = generate_circle_texture_with_aa(texture_size, 1.0);

        let shift = (texture_size as f32 * 0.25) as usize;
        let mut shifted_b = vec![0u8; (texture_size * texture_size * 4) as usize];
        for row in 0..texture_size as usize {
            for col in shift..texture_size as usize {
                let source_col = col - shift;
                let destination_offset = (row * texture_size as usize + col) * 4;
                let source_offset = (row * texture_size as usize + source_col) * 4;
                shifted_b[destination_offset..destination_offset + 4]
                    .copy_from_slice(&circle_b[source_offset..source_offset + 4]);
            }
        }

        let union_data = boolean_union(&circle_a, &shifted_b, texture_size, texture_size);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: BOOLEAN_UNION_SLOT,
            rgba_data: union_data,
            width: texture_size,
            height: texture_size,
        });

        let subtract_data = boolean_subtract(&circle_a, &shifted_b, texture_size, texture_size);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: BOOLEAN_SUBTRACT_SLOT,
            rgba_data: subtract_data,
            width: texture_size,
            height: texture_size,
        });

        let intersect_data = boolean_intersect(&circle_a, &shifted_b, texture_size, texture_size);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: BOOLEAN_INTERSECT_SLOT,
            rgba_data: intersect_data,
            width: texture_size,
            height: texture_size,
        });

        let uv_max_coord = texture_size as f32 / 512.0;
        let half_texel = 0.5 / 512.0;
        let uv_min = Vec2::new(half_texel, half_texel);
        let uv_max = Vec2::new(uv_max_coord - half_texel, uv_max_coord - half_texel);

        let ops = [
            (BOOLEAN_UNION_SLOT, -25.0, 8.0, [0.3, 0.8, 1.0, 1.0]),
            (BOOLEAN_SUBTRACT_SLOT, 25.0, 8.0, [1.0, 0.4, 0.3, 1.0]),
            (BOOLEAN_INTERSECT_SLOT, 0.0, -18.0, [0.4, 1.0, 0.4, 1.0]),
        ];

        for (slot, offset_x, offset_y, color) in ops {
            let entity = spawn_sprite(
                world,
                Vec2::new(center_x + offset_x, center_y + offset_y),
                Vec2::new(35.0, 35.0),
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.texture_index = slot;
                sprite.texture_index2 = slot;
                sprite.uv_min = uv_min;
                sprite.uv_max = uv_max;
                sprite.color = color;
                sprite.depth = 4.0;
            }
            self.demo_entities.push(entity);
        }
    }

    fn build_aa_control(&mut self, world: &mut World) {
        let center_x = COL_2_X;
        let center_y = ROW_5_Y;
        let texture_size = 128;

        let hard_data = generate_circle_texture_with_aa(texture_size, 0.0);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: AA_HARD_SLOT,
            rgba_data: hard_data,
            width: texture_size,
            height: texture_size,
        });

        let default_data = generate_circle_texture_with_aa(texture_size, 1.0);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: AA_DEFAULT_SLOT,
            rgba_data: default_data,
            width: texture_size,
            height: texture_size,
        });

        let soft_data = generate_circle_texture_with_aa(texture_size, 6.0);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: AA_SOFT_SLOT,
            rgba_data: soft_data,
            width: texture_size,
            height: texture_size,
        });

        let uv_max_coord = texture_size as f32 / 512.0;
        let half_texel = 0.5 / 512.0;
        let uv_min = Vec2::new(half_texel, half_texel);
        let uv_max = Vec2::new(uv_max_coord - half_texel, uv_max_coord - half_texel);

        let aa_configs = [
            (AA_HARD_SLOT, -25.0, [1.0, 0.5, 0.2, 1.0]),
            (AA_DEFAULT_SLOT, 0.0, [0.4, 0.8, 1.0, 1.0]),
            (AA_SOFT_SLOT, 25.0, [0.5, 1.0, 0.4, 1.0]),
        ];

        for (slot, offset_x, color) in aa_configs {
            let entity = spawn_sprite(
                world,
                Vec2::new(center_x + offset_x, center_y),
                Vec2::new(40.0, 40.0),
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.texture_index = slot;
                sprite.texture_index2 = slot;
                sprite.uv_min = uv_min;
                sprite.uv_max = uv_max;
                sprite.color = color;
                sprite.depth = 4.0;
            }
            self.demo_entities.push(entity);
        }
    }

    fn build_shadow_demo(&mut self, world: &mut World) {
        let center_x = COL_3_X;
        let center_y = ROW_5_Y;
        let texture_size = 128;

        let source_data = generate_circle_texture_with_aa(texture_size, 1.0);
        world.queue_command(WorldCommand::UploadSpriteTexture {
            slot: SHADOW_SOURCE_SLOT,
            rgba_data: source_data.clone(),
            width: texture_size,
            height: texture_size,
        });

        let shadow_entity = spawn_shadow(
            world,
            &BlurSource {
                texture: &source_data,
                width: texture_size,
                height: texture_size,
                blur_radius: 8,
            },
            [0.0, 0.0, 0.0, 0.7],
            Vec2::new(4.0, -4.0),
            Vec2::new(center_x, center_y),
            Vec2::new(50.0, 50.0),
            4.5,
        );
        self.demo_entities.push(shadow_entity);

        let uv_max_coord = texture_size as f32 / 512.0;
        let half_texel = 0.5 / 512.0;
        let uv_min = Vec2::new(half_texel, half_texel);
        let uv_max = Vec2::new(uv_max_coord - half_texel, uv_max_coord - half_texel);

        let circle_entity =
            spawn_sprite(world, Vec2::new(center_x, center_y), Vec2::new(50.0, 50.0));
        if let Some(sprite) = world.core.get_sprite_mut(circle_entity) {
            sprite.texture_index = SHADOW_SOURCE_SLOT;
            sprite.texture_index2 = SHADOW_SOURCE_SLOT;
            sprite.uv_min = uv_min;
            sprite.uv_max = uv_max;
            sprite.color = [0.3, 0.6, 1.0, 1.0];
            sprite.depth = 5.0;
        }
        self.demo_entities.push(circle_entity);
    }

    fn build_glow_demo(&mut self, world: &mut World) {
        let center_x = COL_4_X;
        let center_y = ROW_5_Y;
        let texture_size = 128;

        let source_data = generate_circle_texture_with_aa(texture_size, 1.0);

        let glow_entity = spawn_glow(
            world,
            &BlurSource {
                texture: &source_data,
                width: texture_size,
                height: texture_size,
                blur_radius: 12,
            },
            [0.2, 0.8, 1.0, 0.8],
            Vec2::new(center_x, center_y),
            Vec2::new(70.0, 70.0),
            3.0,
        );
        self.demo_entities.push(glow_entity);

        let shape_entity = spawn_circle(
            world,
            Vec2::new(center_x, center_y),
            18.0,
            [0.3, 0.9, 1.0, 1.0],
        );
        if let Some(sprite) = world.core.get_sprite_mut(shape_entity) {
            sprite.depth = 5.0;
        }
        self.demo_entities.push(shape_entity);
    }

    fn build_stencil_demo(&mut self, world: &mut World) {
        let center_x = COL_5_X;
        let center_y = ROW_5_Y;

        let mask_entity = spawn_circle(
            world,
            Vec2::new(center_x, center_y),
            25.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        if let Some(sprite) = world.core.get_sprite_mut(mask_entity) {
            sprite.depth = 3.0;
            sprite.stencil_mode = SpriteStencilMode::Write;
        }
        self.demo_entities.push(mask_entity);

        let stripe_colors: [[f32; 4]; 4] = [
            [1.0, 0.3, 0.3, 1.0],
            [0.3, 1.0, 0.3, 1.0],
            [0.3, 0.3, 1.0, 1.0],
            [1.0, 0.8, 0.2, 1.0],
        ];

        for (index, color) in stripe_colors.iter().enumerate() {
            let stripe_x = center_x - 30.0 + index as f32 * 20.0;
            let entity = spawn_rect(
                world,
                Vec2::new(stripe_x, center_y),
                Vec2::new(18.0, 60.0),
                *color,
            );
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                sprite.depth = 4.0;
                sprite.stencil_mode = SpriteStencilMode::Test;
            }
            self.demo_entities.push(entity);
        }

        let border_entity = spawn_ring(
            world,
            Vec2::new(center_x, center_y),
            25.0,
            [0.8, 0.8, 0.8, 0.5],
        );
        if let Some(sprite) = world.core.get_sprite_mut(border_entity) {
            sprite.depth = 5.0;
        }
        self.demo_entities.push(border_entity);
    }

    fn build_path_fill_demo(&mut self, world: &mut World) {
        let center_x = COL_6_X;
        let center_y = ROW_5_Y;

        let star_points: Vec<Vec2> = (0..10)
            .map(|index| {
                let angle =
                    index as f32 * std::f32::consts::TAU / 10.0 - std::f32::consts::FRAC_PI_2;
                let radius = if index % 2 == 0 { 0.48 } else { 0.22 };
                Vec2::new(0.5 + radius * angle.cos(), 0.5 + radius * angle.sin())
            })
            .collect();

        let entity = spawn_filled_path(
            world,
            &star_points,
            [1.0, 0.8, 0.2, 1.0],
            Vec2::new(center_x, center_y + 8.0),
            Vec2::new(45.0, 45.0),
            4.0,
        );
        self.demo_entities.push(entity);

        let hex_points: Vec<Vec2> = (0..6)
            .map(|index| {
                let angle = index as f32 * std::f32::consts::TAU / 6.0;
                Vec2::new(0.5 + 0.45 * angle.cos(), 0.5 + 0.45 * angle.sin())
            })
            .collect();

        let hex_entity = spawn_filled_path(
            world,
            &hex_points,
            [0.4, 0.7, 1.0, 1.0],
            Vec2::new(center_x, center_y - 18.0),
            Vec2::new(35.0, 35.0),
            4.0,
        );
        self.demo_entities.push(hex_entity);
    }

    fn build_labels(&mut self, world: &mut World) {
        let title = spawn_sprite_text(
            world,
            "2D Graphics Primitives",
            Vec2::new(-130.0, 300.0),
            22.0,
        );
        if let Some(text) = world.core.get_sprite_text_mut(title) {
            text.color = [1.0, 1.0, 1.0, 1.0];
            text.depth = 1.0;
        }

        let row_1_label_y = ROW_1_Y + 55.0;
        let row_1_labels: &[(&str, f32)] = &[
            ("Rects", COL_1_X),
            ("Circles", COL_2_X),
            ("Ellipses", COL_3_X),
            ("Rings", COL_4_X),
            ("Triangles", COL_5_X),
            ("Capsules", COL_6_X),
        ];

        for &(label, center_x) in row_1_labels {
            let offset_x = label.len() as f32 * -3.5;
            let entity = spawn_sprite_text(
                world,
                label,
                Vec2::new(center_x + offset_x, row_1_label_y),
                11.0,
            );
            if let Some(text) = world.core.get_sprite_text_mut(entity) {
                text.color = [0.6, 0.6, 0.6, 1.0];
                text.depth = 1.0;
            }
        }

        let row_2_label_y = ROW_2_Y + 55.0;
        let row_2_labels: &[(&str, f32)] = &[
            ("Rounded", COL_1_X),
            ("Outlined", COL_2_X),
            ("Fill+Stroke", COL_3_X),
            ("Soft Circles", COL_4_X),
            ("Screen Blend", COL_5_X),
            ("Gradients", COL_6_X),
        ];

        for &(label, center_x) in row_2_labels {
            let offset_x = label.len() as f32 * -3.5;
            let entity = spawn_sprite_text(
                world,
                label,
                Vec2::new(center_x + offset_x, row_2_label_y),
                11.0,
            );
            if let Some(text) = world.core.get_sprite_text_mut(entity) {
                text.color = [0.6, 0.6, 0.6, 1.0];
                text.depth = 1.0;
            }
        }

        let row_3_label_y = ROW_3_Y + 55.0;
        let row_3_labels: &[(&str, f32)] = &[
            ("Lines", COL_1_X),
            ("Dashed", COL_2_X),
            ("Var-Width", COL_3_X),
            ("Bezier", COL_4_X),
            ("Paths", COL_5_X),
            ("Polygons", COL_6_X),
        ];

        for &(label, center_x) in row_3_labels {
            let offset_x = label.len() as f32 * -3.5;
            let entity = spawn_sprite_text(
                world,
                label,
                Vec2::new(center_x + offset_x, row_3_label_y),
                11.0,
            );
            if let Some(text) = world.core.get_sprite_text_mut(entity) {
                text.color = [0.6, 0.6, 0.6, 1.0];
                text.depth = 1.0;
            }
        }

        let row_4_label_y = ROW_4_Y + 55.0;
        let row_4_labels: &[(&str, f32)] = &[
            ("Param Rings", COL_1_X),
            ("Param Outlines", COL_2_X),
            ("Fill+Stroke El", COL_3_X),
            ("Clip Rects", COL_4_X),
            ("Zoom Lines", COL_5_X),
        ];

        for &(label, center_x) in row_4_labels {
            let offset_x = label.len() as f32 * -3.5;
            let entity = spawn_sprite_text(
                world,
                label,
                Vec2::new(center_x + offset_x, row_4_label_y),
                11.0,
            );
            if let Some(text) = world.core.get_sprite_text_mut(entity) {
                text.color = [0.6, 0.6, 0.6, 1.0];
                text.depth = 1.0;
            }
        }

        let row_5_label_y = ROW_5_Y + 55.0;
        let row_5_labels: &[(&str, f32)] = &[
            ("Boolean Ops", COL_1_X),
            ("AA Control", COL_2_X),
            ("Shadow", COL_3_X),
            ("Glow", COL_4_X),
            ("Stencil", COL_5_X),
            ("Path Fill", COL_6_X),
        ];

        for &(label, center_x) in row_5_labels {
            let offset_x = label.len() as f32 * -3.5;
            let entity = spawn_sprite_text(
                world,
                label,
                Vec2::new(center_x + offset_x, row_5_label_y),
                11.0,
            );
            if let Some(text) = world.core.get_sprite_text_mut(entity) {
                text.color = [0.6, 0.6, 0.6, 1.0];
                text.depth = 1.0;
            }
        }

        let instructions = spawn_sprite_text(
            world,
            "Click to spawn | Scroll: switch shape | Shift+Scroll: zoom | C: clear",
            Vec2::new(-270.0, -560.0),
            11.0,
        );
        if let Some(text) = world.core.get_sprite_text_mut(instructions) {
            text.color = [0.5, 0.5, 0.5, 1.0];
            text.depth = 1.0;
        }
    }

    fn apply_tweens(&self, world: &mut World) {
        for &entity in &self.demo_entities {
            let tween_data = world.core.get_tween(entity).cloned();
            let Some(tween) = tween_data else {
                continue;
            };

            if let Some(track) = tween.track_by_tag(TAG_POSITION) {
                let position = track.value_vec2();
                if let Some(sprite) = world.core.get_sprite_mut(entity) {
                    sprite.position = position;
                }
            }

            if let Some(track) = tween.track_by_tag(TAG_SCALE) {
                let scale = track.value_f32();
                if let Some(sprite) = world.core.get_sprite_mut(entity) {
                    sprite.scale = Vec2::new(scale, scale);
                }
            }

            if let Some(track) = tween.track_by_tag(TAG_ROTATION) {
                let rotation = track.value_f32();
                if let Some(sprite) = world.core.get_sprite_mut(entity) {
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
            .and_then(|entity| world.core.get_local_transform(entity))
            .map(|transform| Vec2::new(transform.translation.x, transform.translation.y))
            .unwrap_or(Vec2::zeros());

        let half_view = self
            .camera_entity
            .and_then(|entity| world.core.get_camera(entity))
            .map(|camera| {
                if let Projection::Orthographic(ortho) = &camera.projection {
                    Vec2::new(ortho.x_mag, ortho.y_mag)
                } else {
                    Vec2::new(480.0, 400.0)
                }
            })
            .unwrap_or(Vec2::new(480.0, 560.0));

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
            && let Some(sprite) = world.core.get_sprite_mut(entity)
        {
            sprite.position = world_position;
        }
    }

    fn update_zoom_lines(&mut self, world: &mut World) {
        let world_thickness = screen_pixels_to_world_size(world, 2.0);
        let aa_padding = (world_thickness * 0.3).max(screen_pixels_to_world_size(world, 1.0));
        for &entity in &self.zoom_line_entities {
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
                let length = sprite.size.x;
                sprite.size = Vec2::new(length, world_thickness + aa_padding);
            }
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
            let shift_held = keyboard.is_key_pressed(KeyCode::ShiftLeft)
                || keyboard.is_key_pressed(KeyCode::ShiftRight);

            if shift_held {
                if let Some(camera_entity) = self.camera_entity
                    && let Some(camera) = world.core.get_camera_mut(camera_entity)
                    && let Projection::Orthographic(ref mut ortho) = camera.projection
                {
                    let zoom_factor = if scroll_y > 0.0 { 0.9 } else { 1.1 };
                    ortho.x_mag = (ortho.x_mag * zoom_factor).clamp(100.0, 2000.0);
                    ortho.y_mag = (ortho.y_mag * zoom_factor).clamp(70.0, 1400.0);
                }
            } else if scroll_y > 0.0 {
                self.current_shape_index = (self.current_shape_index + 1) % ALL_SHAPES.len();
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

        if shape_changed && let Some(entity) = self.preview_entity {
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
                    if let Some(sprite) = world.core.get_sprite_mut(entity) {
                        sprite.blend_mode = SpriteBlendMode::Additive;
                    }
                    entity
                }
            };

            self.next_spawn_depth += 0.01;
            if let Some(sprite) = world.core.get_sprite_mut(entity) {
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

        if let Some(camera_data) = world.core.get_camera_mut(camera)
            && let Projection::Orthographic(ref mut ortho) = camera_data.projection
        {
            ortho.x_mag = 480.0;
            ortho.y_mag = 560.0;
        }

        self.build_scene(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        self.handle_input(world);
        self.update_preview(world);
        self.apply_tweens(world);
        self.update_zoom_lines(world);
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("2D Graphics Primitives")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                let fps = world.resources.window.timing.frames_per_second;
                ui.label(format!("FPS: {fps:.0}"));
                ui.separator();

                let shape = current_shape(self);
                ui.label(format!(
                    "Shape: {shape:?} [{}]",
                    self.current_shape_index + 1
                ));
                ui.label(format!("Spawned: {}", self.spawned_entities.len()));
                ui.separator();

                ui.label("1-7 or Scroll: switch | Click: spawn | Shift+Scroll: zoom | C: clear");
            });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(GfxDemo::default())
}
