use std::collections::{HashMap, HashSet};

use nightshade::ecs::gpu_picking::GpuPickResult;
use nightshade::ecs::picking::PickingRay;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::sdf::{
    CollisionChunkResult, CsgOperation, SdfCollisionMesh, SdfEdit, SdfPrimitive,
};
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SdfDemo::default())
}

#[derive(Clone, Copy, PartialEq)]
enum BrushPrimitive {
    Sphere,
    Box,
    Cylinder,
    Torus,
    Capsule,
}

#[derive(Clone, Copy, PartialEq)]
enum BrushOperation {
    Add,
    Subtract,
    SmoothAdd,
    SmoothSubtract,
    PhysicsSphere,
    PhysicsBox,
    PhysicsCapsule,
    PhysicsSnowman,
}

struct PhysicsSdfObject {
    entity: Entity,
    edit_indices: Vec<usize>,
    local_offsets: Vec<nalgebra_glm::Mat4>,
    last_translation: Vec3,
    last_rotation: Quat,
}

struct CollisionChunkInfo {
    entity: Entity,
    wireframe_lines: Vec<Line>,
}

struct SdfDemo {
    brush_primitive: Option<BrushPrimitive>,
    brush_operation: Option<BrushOperation>,
    brush_size: f32,
    brush_smoothness: f32,
    current_material: u32,
    sdf_pass_configured: bool,
    show_debug_info: bool,
    brush_preview_entity: Option<Entity>,
    brush_position: Vec3,
    brush_valid: bool,
    mouse_down: bool,
    last_apply_time: f32,
    apply_interval: f32,
    snap_to_grid: bool,
    snap_level: usize,
    show_brick_grid: bool,
    brick_grid_entity: Option<Entity>,
    brick_grid_level: usize,
    brick_grid_radius: i32,
    debug_brick_coloring: bool,
    terrain_enabled: bool,
    terrain_base_height: f32,
    terrain_seed: u32,
    terrain_frequency: f32,
    terrain_amplitude: f32,
    terrain_octaves: u32,
    terrain_gain: f32,
    helmet_entity: Option<Entity>,
    physics_objects: Vec<PhysicsSdfObject>,
    ground_entity: Option<Entity>,
    physics_spawn_size: f32,
    physics_spawn_material: u32,
    physics_spawn_smoothness: f32,
    collision_mesh: SdfCollisionMesh,
    collision_chunks: HashMap<nalgebra_glm::IVec3, CollisionChunkInfo>,
    collision_mesh_enabled: bool,
    collision_cell_size: f32,
    collision_radius: f32,
    show_collision_wireframe: bool,
    collision_wireframe_entity: Option<Entity>,
    last_collision_camera_pos: Vec3,
    wireframe_dirty: bool,
    last_pick_result: Option<GpuPickResult>,
    last_pick_mouse_pos: (u32, u32),
    fps_hud_text: Option<Entity>,
}

impl Default for SdfDemo {
    fn default() -> Self {
        let chunk_size = 16.0;
        let cell_size = 2.0;
        Self {
            brush_primitive: None,
            brush_operation: None,
            brush_size: 0.0,
            brush_smoothness: 0.0,
            current_material: 0,
            sdf_pass_configured: false,
            show_debug_info: false,
            brush_preview_entity: None,
            brush_position: Vec3::zeros(),
            brush_valid: false,
            mouse_down: false,
            last_apply_time: 0.0,
            apply_interval: 0.0,
            snap_to_grid: false,
            snap_level: 0,
            show_brick_grid: false,
            brick_grid_entity: None,
            brick_grid_level: 0,
            brick_grid_radius: 0,
            debug_brick_coloring: false,
            terrain_enabled: false,
            terrain_base_height: 0.0,
            terrain_seed: 0,
            terrain_frequency: 0.0,
            terrain_amplitude: 0.0,
            terrain_octaves: 0,
            terrain_gain: 0.0,
            helmet_entity: None,
            physics_objects: Vec::new(),
            ground_entity: None,
            physics_spawn_size: 0.0,
            physics_spawn_material: 0,
            physics_spawn_smoothness: 0.0,
            collision_mesh: SdfCollisionMesh::new(chunk_size, cell_size),
            collision_chunks: HashMap::new(),
            collision_mesh_enabled: true,
            collision_cell_size: cell_size,
            collision_radius: 40.0,
            show_collision_wireframe: true,
            collision_wireframe_entity: None,
            last_collision_camera_pos: Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            wireframe_dirty: false,
            last_pick_result: None,
            last_pick_mouse_pos: (u32::MAX, u32::MAX),
            fps_hud_text: None,
        }
    }
}

impl SdfDemo {
    fn brick_pointer_to_color(brick_pointer: i32) -> Vec4 {
        let p = brick_pointer as f32;
        let r = (p * 0.1031 + 0.1).fract();
        let g = (p * 0.1047 + 0.3).fract();
        let b = (p * 0.1087 + 0.7).fract();
        let brightness = 0.4 + 0.6 * (p * 0.0731).fract();
        Vec4::new(r * brightness, g * brightness, b * brightness, 1.0)
    }

    fn spawn_initial_scene(&self, world: &mut World) {
        world.resources.sdf_materials.set_material(
            0,
            nightshade::ecs::sdf::SdfMaterial::new(Vec3::new(0.35, 0.55, 0.25))
                .with_roughness(0.8)
                .with_metallic(0.0),
        );

        let red_mat = world.resources.sdf_materials.add_material(
            nightshade::ecs::sdf::SdfMaterial::new(Vec3::new(0.8, 0.2, 0.15))
                .with_roughness(0.3)
                .with_metallic(0.1),
        );

        let blue_mat = world.resources.sdf_materials.add_material(
            nightshade::ecs::sdf::SdfMaterial::new(Vec3::new(0.15, 0.3, 0.8))
                .with_roughness(0.2)
                .with_metallic(0.5),
        );

        let gold_mat = world.resources.sdf_materials.add_material(
            nightshade::ecs::sdf::SdfMaterial::new(Vec3::new(0.9, 0.7, 0.2))
                .with_roughness(0.1)
                .with_metallic(0.9),
        );

        let _brown_mat = world.resources.sdf_materials.add_material(
            nightshade::ecs::sdf::SdfMaterial::new(Vec3::new(0.4, 0.3, 0.2))
                .with_roughness(0.9)
                .with_metallic(0.0),
        );

        let _stone_mat = world.resources.sdf_materials.add_material(
            nightshade::ecs::sdf::SdfMaterial::new(Vec3::new(0.5, 0.5, 0.5))
                .with_roughness(0.7)
                .with_metallic(0.1),
        );

        let _terrain_material = world.resources.sdf_materials.add_material(
            nightshade::ecs::sdf::SdfMaterial::new(Vec3::new(0.35, 0.55, 0.25))
                .with_roughness(0.85)
                .with_metallic(0.05),
        );

        world
            .resources
            .sdf_world
            .add_sphere(Vec3::new(0.0, 0.0, 0.0), 2.0, red_mat);

        world
            .resources
            .sdf_world
            .add_sphere(Vec3::new(-5.0, 1.0, 0.0), 1.5, blue_mat);

        world
            .resources
            .sdf_world
            .add_sphere(Vec3::new(5.0, 1.0, 0.0), 1.5, gold_mat);
    }

    fn load_preset_csg_demo(world: &mut World) {
        world.resources.sdf_world.clear();

        let transform_a = nalgebra_glm::translation(&Vec3::new(0.0, 0.0, 0.0));
        let transform_b = nalgebra_glm::translation(&Vec3::new(1.5, 0.0, 0.0));
        let transform_c = nalgebra_glm::translation(&Vec3::new(0.0, 1.5, 0.0));

        world.resources.sdf_world.add_edit(SdfEdit::union(
            SdfPrimitive::Sphere { radius: 2.0 },
            transform_a,
            1,
        ));

        world
            .resources
            .sdf_world
            .add_edit(SdfEdit::smooth_subtraction(
                SdfPrimitive::Sphere { radius: 1.5 },
                transform_b,
                0,
                0.3,
            ));

        world.resources.sdf_world.add_edit(SdfEdit::smooth_union(
            SdfPrimitive::Box {
                half_extents: Vec3::new(0.8, 0.8, 0.8),
            },
            transform_c,
            2,
            0.2,
        ));
    }

    fn load_preset_swiss_cheese(world: &mut World) {
        world.resources.sdf_world.clear();

        let base_transform = nalgebra_glm::translation(&Vec3::new(0.0, 0.0, 0.0));
        world.resources.sdf_world.add_edit(SdfEdit::union(
            SdfPrimitive::Box {
                half_extents: Vec3::new(3.0, 2.0, 2.0),
            },
            base_transform,
            3,
        ));

        let hole_positions = [
            Vec3::new(-1.5, 0.5, 0.0),
            Vec3::new(0.0, -0.5, 0.5),
            Vec3::new(1.0, 0.8, -0.3),
            Vec3::new(-0.5, -0.3, -0.8),
            Vec3::new(1.5, 0.0, 0.8),
            Vec3::new(-1.0, 0.2, 1.0),
        ];

        for (index, pos) in hole_positions.iter().enumerate() {
            let radius = 0.4 + (index as f32 * 0.15);
            let transform = nalgebra_glm::translation(pos);
            world
                .resources
                .sdf_world
                .add_edit(SdfEdit::smooth_subtraction(
                    SdfPrimitive::Sphere { radius },
                    transform,
                    0,
                    0.1,
                ));
        }
    }

    fn load_preset_tower(world: &mut World) {
        world.resources.sdf_world.clear();

        let base_transform = nalgebra_glm::translation(&Vec3::new(0.0, -1.0, 0.0));
        world.resources.sdf_world.add_edit(SdfEdit::union(
            SdfPrimitive::Cylinder {
                radius: 2.0,
                half_height: 0.5,
            },
            base_transform,
            4,
        ));

        for level in 0..5 {
            let y = level as f32 * 1.2;
            let radius = 1.5 - level as f32 * 0.2;
            let transform = nalgebra_glm::translation(&Vec3::new(0.0, y, 0.0));
            world.resources.sdf_world.add_edit(SdfEdit::smooth_union(
                SdfPrimitive::Cylinder {
                    radius,
                    half_height: 0.5,
                },
                transform,
                4,
                0.15,
            ));
        }

        let top_transform = nalgebra_glm::translation(&Vec3::new(0.0, 5.5, 0.0));
        world.resources.sdf_world.add_edit(SdfEdit::smooth_union(
            SdfPrimitive::Sphere { radius: 1.0 },
            top_transform,
            2,
            0.2,
        ));
    }

    fn load_preset_molecule(world: &mut World) {
        world.resources.sdf_world.clear();

        let center = nalgebra_glm::translation(&Vec3::new(0.0, 0.0, 0.0));
        world.resources.sdf_world.add_edit(SdfEdit::union(
            SdfPrimitive::Sphere { radius: 1.0 },
            center,
            1,
        ));

        let offsets = [
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(-2.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, -2.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 0.0, -2.0),
        ];

        let materials = [2, 2, 3, 3, 4, 4];

        for (index, offset) in offsets.iter().enumerate() {
            let bond_center = *offset * 0.5;
            let bond_transform = nalgebra_glm::translation(&bond_center);

            let length = nalgebra_glm::length(offset);
            world.resources.sdf_world.add_edit(SdfEdit::smooth_union(
                SdfPrimitive::Capsule {
                    radius: 0.15,
                    half_height: length * 0.4,
                },
                bond_transform,
                0,
                0.1,
            ));

            let atom_transform = nalgebra_glm::translation(offset);
            world.resources.sdf_world.add_edit(SdfEdit::smooth_union(
                SdfPrimitive::Sphere { radius: 0.6 },
                atom_transform,
                materials[index],
                0.15,
            ));
        }
    }

    fn load_preset_donut(world: &mut World) {
        world.resources.sdf_world.clear();

        let torus_transform = nalgebra_glm::translation(&Vec3::new(0.0, 0.0, 0.0));
        world.resources.sdf_world.add_edit(SdfEdit::union(
            SdfPrimitive::Torus {
                major_radius: 2.0,
                minor_radius: 0.8,
            },
            torus_transform,
            3,
        ));

        let bite_transform = nalgebra_glm::translation(&Vec3::new(2.5, 0.3, 0.0));
        world
            .resources
            .sdf_world
            .add_edit(SdfEdit::smooth_subtraction(
                SdfPrimitive::Sphere { radius: 1.2 },
                bite_transform,
                0,
                0.15,
            ));

        for sprinkle_index in 0..12 {
            let angle = sprinkle_index as f32 * std::f32::consts::TAU / 12.0;
            let radius = 2.0;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let y = 0.8 + (sprinkle_index as f32 * 0.1).sin() * 0.2;

            if x < 1.5 {
                let transform = nalgebra_glm::translation(&Vec3::new(x, y, z));
                world.resources.sdf_world.add_edit(SdfEdit::union(
                    SdfPrimitive::Capsule {
                        radius: 0.08,
                        half_height: 0.15,
                    },
                    transform,
                    (sprinkle_index % 3) as u32 + 1,
                ));
            }
        }
    }

    fn generate_brush_lines(&self, color: Vec4) -> Vec<Line> {
        let mut lines = Vec::new();
        let size = self.brush_size;

        match self.brush_primitive {
            Some(BrushPrimitive::Sphere) => {
                Self::add_circle_lines(&mut lines, Vec3::zeros(), size, 0, color);
                Self::add_circle_lines(&mut lines, Vec3::zeros(), size, 1, color);
                Self::add_circle_lines(&mut lines, Vec3::zeros(), size, 2, color);
            }
            Some(BrushPrimitive::Box) => {
                let half = size;
                let corners = [
                    Vec3::new(-half, -half, -half),
                    Vec3::new(half, -half, -half),
                    Vec3::new(half, half, -half),
                    Vec3::new(-half, half, -half),
                    Vec3::new(-half, -half, half),
                    Vec3::new(half, -half, half),
                    Vec3::new(half, half, half),
                    Vec3::new(-half, half, half),
                ];
                let edges = [
                    (0, 1),
                    (1, 2),
                    (2, 3),
                    (3, 0),
                    (4, 5),
                    (5, 6),
                    (6, 7),
                    (7, 4),
                    (0, 4),
                    (1, 5),
                    (2, 6),
                    (3, 7),
                ];
                for (start_index, end_index) in edges {
                    lines.push(Line {
                        start: corners[start_index],
                        end: corners[end_index],
                        color,
                    });
                }
            }
            Some(BrushPrimitive::Cylinder) => {
                let half_height = size;
                let radius = size * 0.5;
                Self::add_circle_lines_at(
                    &mut lines,
                    Vec3::new(0.0, -half_height, 0.0),
                    radius,
                    1,
                    color,
                );
                Self::add_circle_lines_at(
                    &mut lines,
                    Vec3::new(0.0, half_height, 0.0),
                    radius,
                    1,
                    color,
                );
                for angle_index in 0..4 {
                    let angle = (angle_index as f32) * std::f32::consts::PI * 0.5;
                    let x = angle.cos() * radius;
                    let z = angle.sin() * radius;
                    lines.push(Line {
                        start: Vec3::new(x, -half_height, z),
                        end: Vec3::new(x, half_height, z),
                        color,
                    });
                }
            }
            Some(BrushPrimitive::Torus) => {
                let major_radius = size;
                let minor_radius = size * 0.3;
                Self::add_circle_lines(&mut lines, Vec3::zeros(), major_radius, 1, color);
                Self::add_circle_lines(
                    &mut lines,
                    Vec3::zeros(),
                    major_radius + minor_radius,
                    1,
                    color,
                );
                Self::add_circle_lines(
                    &mut lines,
                    Vec3::zeros(),
                    major_radius - minor_radius,
                    1,
                    color,
                );
                for angle_index in 0..8 {
                    let angle = (angle_index as f32) * std::f32::consts::PI * 0.25;
                    let center_x = angle.cos() * major_radius;
                    let center_z = angle.sin() * major_radius;
                    Self::add_circle_lines_at_rotated(
                        &mut lines,
                        Vec3::new(center_x, 0.0, center_z),
                        minor_radius,
                        angle,
                        color,
                    );
                }
            }
            Some(BrushPrimitive::Capsule) => {
                let half_height = size * 0.5;
                let radius = size * 0.5;
                Self::add_circle_lines_at(
                    &mut lines,
                    Vec3::new(0.0, -half_height, 0.0),
                    radius,
                    1,
                    color,
                );
                Self::add_circle_lines_at(
                    &mut lines,
                    Vec3::new(0.0, half_height, 0.0),
                    radius,
                    1,
                    color,
                );
                Self::add_hemisphere_lines(
                    &mut lines,
                    Vec3::new(0.0, half_height, 0.0),
                    radius,
                    true,
                    color,
                );
                Self::add_hemisphere_lines(
                    &mut lines,
                    Vec3::new(0.0, -half_height, 0.0),
                    radius,
                    false,
                    color,
                );
                for angle_index in 0..4 {
                    let angle = (angle_index as f32) * std::f32::consts::PI * 0.5;
                    let x = angle.cos() * radius;
                    let z = angle.sin() * radius;
                    lines.push(Line {
                        start: Vec3::new(x, -half_height, z),
                        end: Vec3::new(x, half_height, z),
                        color,
                    });
                }
            }
            None => {}
        }

        lines
    }

    fn add_circle_lines(
        lines: &mut Vec<Line>,
        center: Vec3,
        radius: f32,
        axis: usize,
        color: Vec4,
    ) {
        let segments = 32;
        for segment_index in 0..segments {
            let angle1 = (segment_index as f32) * std::f32::consts::TAU / (segments as f32);
            let angle2 = ((segment_index + 1) as f32) * std::f32::consts::TAU / (segments as f32);

            let (start, end) = match axis {
                0 => (
                    center + Vec3::new(0.0, angle1.cos() * radius, angle1.sin() * radius),
                    center + Vec3::new(0.0, angle2.cos() * radius, angle2.sin() * radius),
                ),
                1 => (
                    center + Vec3::new(angle1.cos() * radius, 0.0, angle1.sin() * radius),
                    center + Vec3::new(angle2.cos() * radius, 0.0, angle2.sin() * radius),
                ),
                _ => (
                    center + Vec3::new(angle1.cos() * radius, angle1.sin() * radius, 0.0),
                    center + Vec3::new(angle2.cos() * radius, angle2.sin() * radius, 0.0),
                ),
            };

            lines.push(Line { start, end, color });
        }
    }

    fn add_circle_lines_at(
        lines: &mut Vec<Line>,
        center: Vec3,
        radius: f32,
        axis: usize,
        color: Vec4,
    ) {
        Self::add_circle_lines(lines, center, radius, axis, color);
    }

    fn add_circle_lines_at_rotated(
        lines: &mut Vec<Line>,
        center: Vec3,
        radius: f32,
        rotation_y: f32,
        color: Vec4,
    ) {
        let segments = 16;
        let cos_r = rotation_y.cos();
        let sin_r = rotation_y.sin();

        for segment_index in 0..segments {
            let angle1 = (segment_index as f32) * std::f32::consts::TAU / (segments as f32);
            let angle2 = ((segment_index + 1) as f32) * std::f32::consts::TAU / (segments as f32);

            let local_start = Vec3::new(angle1.cos() * radius, angle1.sin() * radius, 0.0);
            let local_end = Vec3::new(angle2.cos() * radius, angle2.sin() * radius, 0.0);

            let start =
                center + Vec3::new(local_start.x * cos_r, local_start.y, local_start.x * sin_r);
            let end = center + Vec3::new(local_end.x * cos_r, local_end.y, local_end.x * sin_r);

            lines.push(Line { start, end, color });
        }
    }

    fn add_hemisphere_lines(
        lines: &mut Vec<Line>,
        center: Vec3,
        radius: f32,
        top: bool,
        color: Vec4,
    ) {
        let segments = 16;
        let sign = if top { 1.0 } else { -1.0 };

        for arc_index in 0..4 {
            let arc_angle = (arc_index as f32) * std::f32::consts::PI * 0.5;
            let cos_arc = arc_angle.cos();
            let sin_arc = arc_angle.sin();

            for segment_index in 0..(segments / 2) {
                let phi1 = (segment_index as f32) * std::f32::consts::PI / (segments as f32);
                let phi2 = ((segment_index + 1) as f32) * std::f32::consts::PI / (segments as f32);

                let start = center
                    + Vec3::new(
                        phi1.sin() * cos_arc * radius,
                        phi1.cos() * radius * sign,
                        phi1.sin() * sin_arc * radius,
                    );
                let end = center
                    + Vec3::new(
                        phi2.sin() * cos_arc * radius,
                        phi2.cos() * radius * sign,
                        phi2.sin() * sin_arc * radius,
                    );

                lines.push(Line { start, end, color });
            }
        }
    }

    fn update_brush_preview(&self, world: &mut World) {
        if let Some(entity) = self.brush_preview_entity {
            if let Some(visibility) = world.get_visibility_mut(entity) {
                visibility.visible = self.brush_valid;
            }

            if !self.brush_valid {
                return;
            }

            let color = match self.brush_operation {
                Some(BrushOperation::Add) | Some(BrushOperation::SmoothAdd) => {
                    Vec4::new(0.2, 1.0, 0.2, 1.0)
                }
                Some(BrushOperation::Subtract) | Some(BrushOperation::SmoothSubtract) => {
                    Vec4::new(1.0, 0.2, 0.2, 1.0)
                }
                Some(BrushOperation::PhysicsSphere)
                | Some(BrushOperation::PhysicsBox)
                | Some(BrushOperation::PhysicsCapsule)
                | Some(BrushOperation::PhysicsSnowman) => Vec4::new(0.2, 0.6, 1.0, 1.0),
                None => Vec4::new(1.0, 1.0, 1.0, 1.0),
            };

            let new_lines = self.generate_brush_lines(color);

            if let Some(lines) = world.get_lines_mut(entity) {
                lines.lines = new_lines;
                lines.mark_dirty();
            }

            if let Some(transform) = world.get_local_transform(entity) {
                let mut new_transform = *transform;
                new_transform.translation = self.brush_position;
                world.assign_local_transform(entity, new_transform);
            }
        }
    }

    fn apply_brush_edit(&mut self, world: &mut World) {
        match self.brush_operation {
            Some(BrushOperation::PhysicsSphere) => {
                let spawn_pos =
                    self.brush_position + Vec3::new(0.0, self.physics_spawn_size + 2.0, 0.0);
                self.spawn_physics_sphere(world, spawn_pos);
                return;
            }
            Some(BrushOperation::PhysicsBox) => {
                let spawn_pos =
                    self.brush_position + Vec3::new(0.0, self.physics_spawn_size + 2.0, 0.0);
                self.spawn_physics_box(world, spawn_pos);
                return;
            }
            Some(BrushOperation::PhysicsCapsule) => {
                let spawn_pos =
                    self.brush_position + Vec3::new(0.0, self.physics_spawn_size + 2.0, 0.0);
                self.spawn_physics_capsule(world, spawn_pos);
                return;
            }
            Some(BrushOperation::PhysicsSnowman) => {
                let spawn_pos =
                    self.brush_position + Vec3::new(0.0, self.physics_spawn_size + 2.0, 0.0);
                self.spawn_physics_snowman(world, spawn_pos);
                return;
            }
            _ => {}
        }

        let primitive = match self.brush_primitive {
            Some(BrushPrimitive::Sphere) => SdfPrimitive::Sphere {
                radius: self.brush_size,
            },
            Some(BrushPrimitive::Box) => SdfPrimitive::Box {
                half_extents: Vec3::new(self.brush_size, self.brush_size, self.brush_size),
            },
            Some(BrushPrimitive::Cylinder) => SdfPrimitive::Cylinder {
                radius: self.brush_size * 0.5,
                half_height: self.brush_size,
            },
            Some(BrushPrimitive::Torus) => SdfPrimitive::Torus {
                major_radius: self.brush_size,
                minor_radius: self.brush_size * 0.3,
            },
            Some(BrushPrimitive::Capsule) => SdfPrimitive::Capsule {
                radius: self.brush_size * 0.5,
                half_height: self.brush_size * 0.5,
            },
            None => return,
        };

        let operation = match self.brush_operation {
            Some(BrushOperation::Add) => CsgOperation::Union,
            Some(BrushOperation::Subtract) => CsgOperation::Subtraction,
            Some(BrushOperation::SmoothAdd) => CsgOperation::SmoothUnion {
                smoothness: self.brush_smoothness,
            },
            Some(BrushOperation::SmoothSubtract) => CsgOperation::SmoothSubtraction {
                smoothness: self.brush_smoothness,
            },
            _ => return,
        };

        let transform = nalgebra_glm::translation(&self.brush_position);
        let edit = SdfEdit::from_operation(primitive, operation, transform, self.current_material);
        world.resources.sdf_world.add_edit(edit);
        self.mark_collision_dirty_for_edit_bounds(self.brush_position, self.brush_size * 2.0);
    }

    fn generate_brick_grid_lines(&self, world: &World, camera_pos: Vec3) -> Vec<Line> {
        let mut lines = Vec::new();

        let voxel_sizes = world.resources.sdf_world.voxel_sizes();
        if self.brick_grid_level >= voxel_sizes.len() {
            return lines;
        }

        let voxel_size = voxel_sizes[self.brick_grid_level];
        let brick_size = 8.0 * voxel_size;

        let default_color = Vec4::new(1.0, 0.8, 0.2, 0.8);

        let allocated_bricks = world
            .resources
            .sdf_world
            .get_allocated_bricks_in_range_with_pointers(
                self.brick_grid_level,
                camera_pos,
                self.brick_grid_radius,
            );

        for (_brick_coord, world_origin, brick_pointer) in allocated_bricks {
            let color = if self.debug_brick_coloring {
                Self::brick_pointer_to_color(brick_pointer)
            } else {
                default_color
            };

            let offset = brick_size * 0.002;
            let min_corner = world_origin - Vec3::new(offset, offset, offset);
            let max_corner = world_origin
                + Vec3::new(
                    brick_size + offset,
                    brick_size + offset,
                    brick_size + offset,
                );

            let corners = [
                min_corner,
                Vec3::new(max_corner.x, min_corner.y, min_corner.z),
                Vec3::new(max_corner.x, max_corner.y, min_corner.z),
                Vec3::new(min_corner.x, max_corner.y, min_corner.z),
                Vec3::new(min_corner.x, min_corner.y, max_corner.z),
                Vec3::new(max_corner.x, min_corner.y, max_corner.z),
                max_corner,
                Vec3::new(min_corner.x, max_corner.y, max_corner.z),
            ];

            let edges = [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4),
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7),
            ];

            for (start_idx, end_idx) in edges {
                lines.push(Line {
                    start: corners[start_idx],
                    end: corners[end_idx],
                    color,
                });
            }
        }

        lines
    }

    fn update_brick_grid_vis(&self, world: &mut World, camera_pos: Vec3) {
        if let Some(entity) = self.brick_grid_entity {
            if let Some(visibility) = world.get_visibility_mut(entity) {
                visibility.visible = self.show_brick_grid;
            }

            if !self.show_brick_grid {
                return;
            }

            let new_lines = self.generate_brick_grid_lines(world, camera_pos);

            if let Some(lines) = world.get_lines_mut(entity) {
                lines.lines = new_lines;
                lines.mark_dirty();
            }
        }
    }

    fn update_brush_position_from_mouse(&mut self, world: &World) {
        self.brush_valid = false;

        if let Some(ref pick) = self.last_pick_result
            && pick.depth < 0.9999
        {
            let normal = pick.world_normal;

            let offset = match self.brush_operation {
                Some(BrushOperation::Add) | Some(BrushOperation::SmoothAdd) => {
                    normal * self.brush_size * 0.5
                }
                Some(BrushOperation::Subtract) | Some(BrushOperation::SmoothSubtract) => {
                    -normal * self.brush_size * 0.3
                }
                Some(BrushOperation::PhysicsSphere)
                | Some(BrushOperation::PhysicsBox)
                | Some(BrushOperation::PhysicsCapsule)
                | Some(BrushOperation::PhysicsSnowman) => normal * 0.1,
                None => Vec3::zeros(),
            };

            let mut final_pos = pick.world_position + offset;

            if self.snap_to_grid {
                final_pos = world
                    .resources
                    .sdf_world
                    .snap_to_voxel_grid(final_pos, self.snap_level);
            }

            self.brush_position = final_pos;
            self.brush_valid = true;
            return;
        }

        let mouse_pos = world.resources.input.mouse.position;
        let screen_pos = Vec2::new(mouse_pos.x, mouse_pos.y);
        if let Some(ray) = PickingRay::from_screen_position(world, screen_pos)
            && let Some(ground_pos) = ray.intersect_ground_plane(0.0)
        {
            let mut final_pos = ground_pos;

            if self.snap_to_grid {
                final_pos = world
                    .resources
                    .sdf_world
                    .snap_to_voxel_grid(final_pos, self.snap_level);
            }

            self.brush_position = final_pos;
            self.brush_valid = true;
        }
    }

    fn make_physics_edit(
        &self,
        primitive: SdfPrimitive,
        transform: nalgebra_glm::Mat4,
        material_id: u32,
    ) -> SdfEdit {
        if self.physics_spawn_smoothness > 0.0 {
            SdfEdit::smooth_union(
                primitive,
                transform,
                material_id,
                self.physics_spawn_smoothness,
            )
        } else {
            SdfEdit::union(primitive, transform, material_id)
        }
    }

    fn spawn_ground_body(&mut self, world: &mut World) {
        let ground = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | RIGID_BODY | COLLIDER,
            1,
        )[0];

        let ground_y = self.terrain_base_height - 0.05;
        world.set_local_transform(
            ground,
            LocalTransform {
                translation: Vec3::new(0.0, ground_y, 0.0),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(ground, LocalTransformDirty);
        world.set_global_transform(ground, GlobalTransform::default());

        if let Some(rigid_body) = world.get_rigid_body_mut(ground) {
            *rigid_body = RigidBodyComponent::new_static().with_translation(0.0, ground_y, 0.0);
        }
        if let Some(collider) = world.get_collider_mut(ground) {
            *collider = ColliderComponent::new_cuboid(200.0, 0.05, 200.0).with_friction(0.6);
        }

        self.ground_entity = Some(ground);
    }

    fn spawn_physics_sphere(&mut self, world: &mut World, position: Vec3) {
        let radius = self.physics_spawn_size;
        let material_id = self.physics_spawn_material;

        let entity = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | RIGID_BODY | COLLIDER,
            1,
        )[0];

        world.set_local_transform(
            entity,
            LocalTransform {
                translation: position,
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(entity, LocalTransformDirty);
        world.set_global_transform(entity, GlobalTransform::default());

        if let Some(rigid_body) = world.get_rigid_body_mut(entity) {
            *rigid_body = RigidBodyComponent::new_dynamic()
                .with_translation(position.x, position.y, position.z)
                .with_mass(5.0);
        }
        if let Some(collider) = world.get_collider_mut(entity) {
            *collider = ColliderComponent::new_ball(radius)
                .with_restitution(0.3)
                .with_friction(0.5);
        }

        let transform = nalgebra_glm::translation(&position);
        let primitive = SdfPrimitive::Sphere { radius };
        let edit = self.make_physics_edit(primitive, transform, material_id);
        let edit_index = world.resources.sdf_world.add_edit_no_undo(edit);

        self.physics_objects.push(PhysicsSdfObject {
            entity,
            edit_indices: vec![edit_index],
            local_offsets: vec![nalgebra_glm::identity()],
            last_translation: position,
            last_rotation: Quat::identity(),
        });
    }

    fn spawn_physics_box(&mut self, world: &mut World, position: Vec3) {
        let half_extent = self.physics_spawn_size;
        let material_id = self.physics_spawn_material;

        let entity = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | RIGID_BODY | COLLIDER,
            1,
        )[0];

        world.set_local_transform(
            entity,
            LocalTransform {
                translation: position,
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(entity, LocalTransformDirty);
        world.set_global_transform(entity, GlobalTransform::default());

        if let Some(rigid_body) = world.get_rigid_body_mut(entity) {
            *rigid_body = RigidBodyComponent::new_dynamic()
                .with_translation(position.x, position.y, position.z)
                .with_mass(8.0);
        }
        if let Some(collider) = world.get_collider_mut(entity) {
            *collider = ColliderComponent::new_cuboid(half_extent, half_extent, half_extent)
                .with_restitution(0.2)
                .with_friction(0.5);
        }

        let transform = nalgebra_glm::translation(&position);
        let half_extents = Vec3::new(half_extent, half_extent, half_extent);
        let primitive = SdfPrimitive::Box { half_extents };
        let edit = self.make_physics_edit(primitive, transform, material_id);
        let edit_index = world.resources.sdf_world.add_edit_no_undo(edit);

        self.physics_objects.push(PhysicsSdfObject {
            entity,
            edit_indices: vec![edit_index],
            local_offsets: vec![nalgebra_glm::identity()],
            last_translation: position,
            last_rotation: Quat::identity(),
        });
    }

    fn spawn_physics_capsule(&mut self, world: &mut World, position: Vec3) {
        let radius = self.physics_spawn_size * 0.5;
        let half_height = self.physics_spawn_size;
        let material_id = self.physics_spawn_material;

        let entity = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | RIGID_BODY | COLLIDER,
            1,
        )[0];

        world.set_local_transform(
            entity,
            LocalTransform {
                translation: position,
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(entity, LocalTransformDirty);
        world.set_global_transform(entity, GlobalTransform::default());

        if let Some(rigid_body) = world.get_rigid_body_mut(entity) {
            *rigid_body = RigidBodyComponent::new_dynamic()
                .with_translation(position.x, position.y, position.z)
                .with_mass(4.0);
        }
        if let Some(collider) = world.get_collider_mut(entity) {
            *collider = ColliderComponent::new_capsule(half_height, radius)
                .with_restitution(0.3)
                .with_friction(0.5);
        }

        let transform = nalgebra_glm::translation(&position);
        let primitive = SdfPrimitive::Capsule {
            radius,
            half_height,
        };
        let edit = self.make_physics_edit(primitive, transform, material_id);
        let edit_index = world.resources.sdf_world.add_edit_no_undo(edit);

        self.physics_objects.push(PhysicsSdfObject {
            entity,
            edit_indices: vec![edit_index],
            local_offsets: vec![nalgebra_glm::identity()],
            last_translation: position,
            last_rotation: Quat::identity(),
        });
    }

    fn spawn_physics_snowman(&mut self, world: &mut World, position: Vec3) {
        let base_radius = self.physics_spawn_size;
        let material_id = self.physics_spawn_material;

        let entity = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | RIGID_BODY | COLLIDER,
            1,
        )[0];

        world.set_local_transform(
            entity,
            LocalTransform {
                translation: position,
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(entity, LocalTransformDirty);
        world.set_global_transform(entity, GlobalTransform::default());

        if let Some(rigid_body) = world.get_rigid_body_mut(entity) {
            *rigid_body = RigidBodyComponent::new_dynamic()
                .with_translation(position.x, position.y, position.z)
                .with_mass(10.0);
        }
        if let Some(collider) = world.get_collider_mut(entity) {
            *collider = ColliderComponent::new_ball(base_radius)
                .with_restitution(0.1)
                .with_friction(0.6);
        }

        let body_offset = Vec3::new(0.0, 0.0, 0.0);
        let mid_offset = Vec3::new(0.0, base_radius + base_radius * 0.65, 0.0);
        let head_offset = Vec3::new(
            0.0,
            base_radius + base_radius * 0.65 * 2.0 + base_radius * 0.4,
            0.0,
        );

        let offsets = [body_offset, mid_offset, head_offset];
        let radii = [base_radius, base_radius * 0.65, base_radius * 0.4];

        let mut edit_indices = Vec::new();
        let mut local_offsets = Vec::new();

        for (part_index, (offset, radius)) in offsets.iter().zip(radii.iter()).enumerate() {
            let world_pos = position + offset;
            let transform = nalgebra_glm::translation(&world_pos);

            let primitive = SdfPrimitive::Sphere { radius: *radius };
            let edit = if part_index == 0 {
                self.make_physics_edit(primitive, transform, material_id)
            } else {
                let structural_smoothness = 0.15_f32.max(self.physics_spawn_smoothness);
                SdfEdit::smooth_union(primitive, transform, material_id, structural_smoothness)
            };

            let edit_index = world.resources.sdf_world.add_edit_no_undo(edit);
            edit_indices.push(edit_index);
            local_offsets.push(nalgebra_glm::translation(offset));
        }

        self.physics_objects.push(PhysicsSdfObject {
            entity,
            edit_indices,
            local_offsets,
            last_translation: position,
            last_rotation: Quat::identity(),
        });
    }

    fn sync_physics_objects(&mut self, world: &mut World) {
        for object in &mut self.physics_objects {
            let (entity_translation, entity_rotation) =
                if let Some(transform) = world.get_local_transform(object.entity) {
                    (transform.translation, transform.rotation)
                } else {
                    continue;
                };

            let translation_delta =
                nalgebra_glm::length(&(entity_translation - object.last_translation));
            let rotation_dot =
                nalgebra_glm::quat_dot(&entity_rotation, &object.last_rotation).abs();

            if translation_delta < 0.0005 && rotation_dot > 0.9999 {
                continue;
            }

            let old_translation = object.last_translation;
            let old_rotation = object.last_rotation;
            object.last_translation = entity_translation;
            object.last_rotation = entity_rotation;

            let entity_matrix = nalgebra_glm::translation(&entity_translation)
                * nalgebra_glm::quat_to_mat4(&entity_rotation);
            let old_entity_matrix = nalgebra_glm::translation(&old_translation)
                * nalgebra_glm::quat_to_mat4(&old_rotation);

            for (offset_index, &edit_index) in object.edit_indices.iter().enumerate() {
                if edit_index >= world.resources.sdf_world.edits.len() {
                    continue;
                }

                let local_offset = &object.local_offsets[offset_index];
                let world_transform = entity_matrix * local_offset;

                if self.collision_mesh_enabled {
                    let bounding_radius = world.resources.sdf_world.edits[edit_index]
                        .primitive()
                        .bounding_radius();
                    let dirty_radius = bounding_radius + 1.0;

                    let old_part_transform = old_entity_matrix * local_offset;
                    let old_part_pos = Vec3::new(
                        old_part_transform[(0, 3)],
                        old_part_transform[(1, 3)],
                        old_part_transform[(2, 3)],
                    );
                    let new_part_pos = Vec3::new(
                        world_transform[(0, 3)],
                        world_transform[(1, 3)],
                        world_transform[(2, 3)],
                    );

                    self.collision_mesh.mark_dirty_in_bounds(
                        old_part_pos - Vec3::new(dirty_radius, dirty_radius, dirty_radius),
                        old_part_pos + Vec3::new(dirty_radius, dirty_radius, dirty_radius),
                    );
                    self.collision_mesh.mark_dirty_in_bounds(
                        new_part_pos - Vec3::new(dirty_radius, dirty_radius, dirty_radius),
                        new_part_pos + Vec3::new(dirty_radius, dirty_radius, dirty_radius),
                    );
                }

                world
                    .resources
                    .sdf_world
                    .modify_edit_no_undo(edit_index, |edit| {
                        edit.set_transform(world_transform);
                    });
            }
        }
    }

    fn clear_physics_objects(&mut self) {
        self.physics_objects.clear();
    }

    fn adjust_physics_indices_after_removal(&mut self, removed_index: usize) {
        self.physics_objects.retain_mut(|object| {
            object.edit_indices.retain(|&index| index != removed_index);
            for index in &mut object.edit_indices {
                if *index > removed_index {
                    *index -= 1;
                }
            }
            !object.edit_indices.is_empty()
        });
    }

    fn populate_new_collision_chunks(&mut self, world: &World, camera_pos: Vec3) {
        let terrain = &world.resources.sdf_world.terrain;
        if !terrain.enabled {
            return;
        }

        let extent = terrain.max_surface_extent();
        let chunk_size = self.collision_mesh.chunk_size;

        let bounds_min = Vec3::new(
            camera_pos.x - self.collision_radius,
            terrain.base_height - extent,
            camera_pos.z - self.collision_radius,
        );
        let bounds_max = Vec3::new(
            camera_pos.x + self.collision_radius,
            terrain.base_height + extent,
            camera_pos.z + self.collision_radius,
        );

        let chunk_min = nalgebra_glm::IVec3::new(
            (bounds_min.x / chunk_size).floor() as i32,
            (bounds_min.y / chunk_size).floor() as i32,
            (bounds_min.z / chunk_size).floor() as i32,
        );
        let chunk_max = nalgebra_glm::IVec3::new(
            (bounds_max.x / chunk_size).ceil() as i32,
            (bounds_max.y / chunk_size).ceil() as i32,
            (bounds_max.z / chunk_size).ceil() as i32,
        );

        for cz in chunk_min.z..=chunk_max.z {
            for cy in chunk_min.y..=chunk_max.y {
                for cx in chunk_min.x..=chunk_max.x {
                    let coord = nalgebra_glm::IVec3::new(cx, cy, cz);
                    if !self.collision_chunks.contains_key(&coord) {
                        self.collision_mesh.dirty_chunks.insert(coord);
                    }
                }
            }
        }
    }

    fn evict_out_of_range_chunks(&mut self, world: &mut World, camera_pos: Vec3) {
        let eviction_radius = self.collision_radius + self.collision_mesh.chunk_size * 2.0;
        let eviction_radius_sq = eviction_radius * eviction_radius;
        let chunk_size = self.collision_mesh.chunk_size;

        let chunks_to_remove: Vec<nalgebra_glm::IVec3> = self
            .collision_chunks
            .keys()
            .filter(|coord| {
                let center_x = (coord.x as f32 + 0.5) * chunk_size;
                let center_z = (coord.z as f32 + 0.5) * chunk_size;
                let dx = center_x - camera_pos.x;
                let dz = center_z - camera_pos.z;
                dx * dx + dz * dz > eviction_radius_sq
            })
            .copied()
            .collect();

        if !chunks_to_remove.is_empty() {
            self.wireframe_dirty = true;
            let entities: Vec<Entity> = chunks_to_remove
                .iter()
                .filter_map(|coord| self.collision_chunks.remove(coord).map(|info| info.entity))
                .collect();
            if !entities.is_empty() {
                world.despawn_entities(&entities);
            }
            for coord in &chunks_to_remove {
                self.collision_mesh.dirty_chunks.remove(coord);
            }
        }
    }

    fn generate_chunk_wireframe_lines(vertices: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<Line> {
        let color = Vec4::new(0.0, 1.0, 0.5, 0.6);
        let mut lines = Vec::with_capacity(indices.len() * 3);

        for triangle in indices {
            let v0 = vertices[triangle[0] as usize];
            let v1 = vertices[triangle[1] as usize];
            let v2 = vertices[triangle[2] as usize];

            let p0 = Vec3::new(v0[0], v0[1], v0[2]);
            let p1 = Vec3::new(v1[0], v1[1], v1[2]);
            let p2 = Vec3::new(v2[0], v2[1], v2[2]);

            lines.push(Line {
                start: p0,
                end: p1,
                color,
            });
            lines.push(Line {
                start: p1,
                end: p2,
                color,
            });
            lines.push(Line {
                start: p2,
                end: p0,
                color,
            });
        }

        lines
    }

    fn update_collision_mesh(&mut self, world: &mut World, camera_pos: Vec3) {
        if !self.collision_mesh_enabled {
            return;
        }

        let camera_moved = nalgebra_glm::length(&(camera_pos - self.last_collision_camera_pos))
            > self.collision_mesh.chunk_size * 0.5;
        if camera_moved {
            self.populate_new_collision_chunks(world, camera_pos);
            self.evict_out_of_range_chunks(world, camera_pos);
            self.last_collision_camera_pos = camera_pos;
        }

        let physics_edit_indices: HashSet<usize> = self
            .physics_objects
            .iter()
            .flat_map(|object| object.edit_indices.iter().copied())
            .collect();
        let results = self.collision_mesh.update(
            &world.resources.sdf_world,
            camera_pos,
            &physics_edit_indices,
        );

        if !results.is_empty() {
            self.wireframe_dirty = true;
        }

        for (chunk_coord, result) in results {
            match result {
                CollisionChunkResult::Mesh { vertices, indices } => {
                    let wireframe_lines = Self::generate_chunk_wireframe_lines(&vertices, &indices);

                    if let Some(existing_info) = self.collision_chunks.get(&chunk_coord) {
                        world.despawn_entities(&[existing_info.entity]);
                    }

                    let entity = world.spawn_entities(
                        LOCAL_TRANSFORM
                            | LOCAL_TRANSFORM_DIRTY
                            | GLOBAL_TRANSFORM
                            | RIGID_BODY
                            | COLLIDER,
                        1,
                    )[0];

                    world.set_local_transform(
                        entity,
                        LocalTransform {
                            translation: Vec3::zeros(),
                            rotation: Quat::identity(),
                            scale: Vec3::new(1.0, 1.0, 1.0),
                        },
                    );
                    world.set_local_transform_dirty(entity, LocalTransformDirty);
                    world.set_global_transform(entity, GlobalTransform::default());

                    if let Some(rigid_body) = world.get_rigid_body_mut(entity) {
                        *rigid_body = RigidBodyComponent::new_static();
                    }
                    if let Some(collider) = world.get_collider_mut(entity) {
                        *collider = ColliderComponent {
                            shape: ColliderShape::TriMesh { vertices, indices },
                            friction: 0.6,
                            restitution: 0.0,
                            ..Default::default()
                        };
                    }

                    self.collision_chunks.insert(
                        chunk_coord,
                        CollisionChunkInfo {
                            entity,
                            wireframe_lines,
                        },
                    );
                }
                CollisionChunkResult::Empty => {
                    if let Some(info) = self.collision_chunks.remove(&chunk_coord) {
                        world.despawn_entities(&[info.entity]);
                    }
                }
            }
        }
    }

    fn clear_collision_mesh(&mut self, world: &mut World) {
        let entities: Vec<Entity> = self
            .collision_chunks
            .values()
            .map(|info| info.entity)
            .collect();
        if !entities.is_empty() {
            world.despawn_entities(&entities);
        }
        self.collision_chunks.clear();
        self.collision_mesh.clear();
        self.last_collision_camera_pos = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        self.wireframe_dirty = true;
    }

    fn mark_collision_dirty_for_edit_bounds(&mut self, position: Vec3, radius: f32) {
        if self.collision_mesh_enabled {
            let bounds_min = position - Vec3::new(radius, radius, radius);
            let bounds_max = position + Vec3::new(radius, radius, radius);
            self.collision_mesh
                .mark_dirty_in_bounds(bounds_min, bounds_max);
        }
    }

    fn update_collision_wireframe(&mut self, world: &mut World) {
        if let Some(entity) = self.collision_wireframe_entity {
            if let Some(visibility) = world.get_visibility_mut(entity) {
                visibility.visible = self.show_collision_wireframe;
            }

            if !self.show_collision_wireframe {
                return;
            }

            if !self.wireframe_dirty {
                return;
            }
            self.wireframe_dirty = false;

            let mut all_lines = Vec::new();
            for info in self.collision_chunks.values() {
                all_lines.extend_from_slice(&info.wireframe_lines);
            }

            if let Some(lines) = world.get_lines_mut(entity) {
                lines.lines = all_lines;
                lines.mark_dirty();
            }
        }
    }
}

impl State for SdfDemo {
    fn title(&self) -> &str {
        "SDF Sculpt Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Sunset;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.clear_color = [0.05, 0.07, 0.1, 1.0];
        world.resources.graphics.bloom_enabled = true;
        world.resources.graphics.bloom_intensity = 0.05;

        capture_procedural_atmosphere_ibl(world, Atmosphere::Sunset, 0.0);

        let camera = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | CAMERA,
            1,
        )[0];

        let look_target = Vec3::new(0.0, 0.0, 0.0);
        let initial_position = Vec3::new(0.0, 10.0, 20.0);
        let look_direction = nalgebra_glm::normalize(&(look_target - initial_position));
        let initial_rotation = nalgebra_glm::quat_look_at(&look_direction, &Vec3::y());

        world.set_local_transform(
            camera,
            LocalTransform {
                translation: initial_position,
                rotation: initial_rotation,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(camera, LocalTransformDirty);
        world.set_global_transform(camera, GlobalTransform::default());
        world.set_camera(
            camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 60.0_f32.to_radians(),
                    z_near: 0.1,
                    z_far: Some(2000.0),
                }),
                smoothing: Some(Smoothing::default()),
            },
        );
        world.resources.active_camera = Some(camera);

        self.brush_primitive = Some(BrushPrimitive::Sphere);
        self.brush_operation = Some(BrushOperation::Add);
        self.brush_size = 1.0;
        self.brush_smoothness = 0.2;
        self.show_debug_info = true;
        self.brush_position = Vec3::zeros();
        self.brush_valid = false;
        self.mouse_down = false;
        self.last_apply_time = 0.0;
        self.apply_interval = 0.1;
        self.snap_to_grid = true;
        self.snap_level = 0;
        self.show_brick_grid = false;
        self.brick_grid_level = 0;
        self.brick_grid_radius = 32;
        self.terrain_enabled = true;
        self.terrain_base_height = 7.0;
        self.terrain_seed = 0;
        self.terrain_frequency = 0.01;
        self.terrain_amplitude = 30.0;
        self.terrain_octaves = 11;
        self.terrain_gain = 0.5;
        self.physics_spawn_size = 1.0;
        self.physics_spawn_material = 1;
        self.physics_spawn_smoothness = 0.0;

        if !self.collision_mesh_enabled && !self.terrain_enabled {
            self.spawn_ground_body(world);
        }

        let brush_preview = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | VISIBILITY | LINES,
            1,
        )[0];
        world.set_local_transform(
            brush_preview,
            LocalTransform {
                translation: Vec3::zeros(),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(brush_preview, LocalTransformDirty);
        world.set_global_transform(brush_preview, GlobalTransform::default());
        world.set_visibility(brush_preview, Visibility { visible: true });
        world.set_lines(brush_preview, Lines::default());
        self.brush_preview_entity = Some(brush_preview);

        let brick_grid = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | VISIBILITY | LINES,
            1,
        )[0];
        world.set_local_transform(
            brick_grid,
            LocalTransform {
                translation: Vec3::zeros(),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(brick_grid, LocalTransformDirty);
        world.set_global_transform(brick_grid, GlobalTransform::default());
        world.set_visibility(brick_grid, Visibility { visible: false });
        world.set_lines(brick_grid, Lines::default());
        self.brick_grid_entity = Some(brick_grid);

        let collision_wireframe = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | VISIBILITY | LINES,
            1,
        )[0];
        world.set_local_transform(
            collision_wireframe,
            LocalTransform {
                translation: Vec3::zeros(),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(collision_wireframe, LocalTransformDirty);
        world.set_global_transform(collision_wireframe, GlobalTransform::default());
        world.set_visibility(collision_wireframe, Visibility { visible: false });
        world.set_lines(collision_wireframe, Lines::default());
        self.collision_wireframe_entity = Some(collision_wireframe);

        self.spawn_initial_scene(world);

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 3.0;
        }

        const GLTF_DATA: &[u8] = include_bytes!("../../../assets/gltf/DamagedHelmet.glb");
        if let Ok(result) = nightshade::ecs::prefab::import_gltf_from_bytes(GLTF_DATA) {
            for (name, (rgba_data, width, height)) in result.textures {
                world.queue_command(WorldCommand::LoadTexture {
                    name,
                    rgba_data,
                    width,
                    height,
                });
            }

            for (name, mesh) in result.meshes {
                mesh_cache_insert(&mut world.resources.mesh_cache, name, mesh);
            }

            for prefab in result.prefabs {
                let entity = nightshade::ecs::prefab::spawn_prefab(
                    world,
                    &prefab,
                    nalgebra_glm::vec3(8.0, 2.0, 0.0),
                );
                self.helmet_entity = Some(entity);
            }
        }

        spawn_hud_text_with_properties(
            world,
            "SDF Sculpt Demo\nWASD: Move | Right-Drag: Look | Left-Click: Sculpt | ESC: Exit",
            HudAnchor::TopCenter,
            Vec2::new(0.0, 20.0),
            TextProperties {
                font_size: 18.0,
                color: Vec4::new(1.0, 1.0, 1.0, 0.9),
                alignment: TextAlignment::Center,
                outline_width: 0.01,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let fps_text = spawn_hud_text_with_properties(
            world,
            "FPS: 0",
            HudAnchor::TopRight,
            Vec2::new(-10.0, 10.0),
            TextProperties {
                font_size: 48.0,
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        self.fps_hud_text = Some(fps_text);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        fly_camera_system(world);
        sync_text_meshes_system(world);

        if let Some(fps_text_entity) = self.fps_hud_text {
            let fps = world.resources.window.timing.frames_per_second;
            let text_index = world.get_hud_text(fps_text_entity).map(|t| t.text_index);
            if let Some(text_index) = text_index {
                world
                    .resources
                    .text_cache
                    .set_text(text_index, format!("FPS: {:.0}", fps));
                if let Some(hud_text) = world.get_hud_text_mut(fps_text_entity) {
                    hud_text.properties.color = if fps >= 56.0 {
                        Vec4::new(0.0, 1.0, 0.0, 1.0)
                    } else if fps >= 30.0 {
                        Vec4::new(1.0, 1.0, 0.0, 1.0)
                    } else {
                        Vec4::new(1.0, 0.3, 0.0, 1.0)
                    };
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(result) = world.resources.gpu_picking.take_result() {
            self.last_pick_result = Some(result);
        }

        let mouse_pos = world.resources.input.mouse.position;
        let current_mouse_pos = (mouse_pos.x as u32, mouse_pos.y as u32);
        if !world.resources.user_interface.hud_wants_pointer
            && current_mouse_pos != self.last_pick_mouse_pos
        {
            world
                .resources
                .gpu_picking
                .request_pick(current_mouse_pos.0, current_mouse_pos.1);
            self.last_pick_mouse_pos = current_mouse_pos;
        }

        let camera_position = if let Some(camera_entity) = world.resources.active_camera {
            if let Some(transform) = world.get_global_transform(camera_entity) {
                transform.translation()
            } else {
                Vec3::zeros()
            }
        } else {
            Vec3::zeros()
        };

        self.update_brush_position_from_mouse(world);

        let current_time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
        let ui_wants_input = world.resources.user_interface.hud_wants_pointer
            || world.resources.user_interface.consumed_event;

        let is_physics_mode = matches!(
            self.brush_operation,
            Some(BrushOperation::PhysicsSphere)
                | Some(BrushOperation::PhysicsBox)
                | Some(BrushOperation::PhysicsCapsule)
                | Some(BrushOperation::PhysicsSnowman)
        );
        let effective_interval = if is_physics_mode {
            0.3
        } else {
            self.apply_interval
        };

        if self.mouse_down
            && self.brush_valid
            && !ui_wants_input
            && current_time - self.last_apply_time >= effective_interval
        {
            self.apply_brush_edit(world);
            self.last_apply_time = current_time;
        }

        self.update_brush_preview(world);
        self.update_brick_grid_vis(world, camera_position);

        self.sync_physics_objects(world);

        world.resources.sdf_world.debug_brick_coloring = self.debug_brick_coloring;
        world
            .resources
            .sdf_world
            .set_terrain_config(nightshade::ecs::sdf::TerrainConfig {
                enabled: self.terrain_enabled,
                base_height: self.terrain_base_height,
                material_id: 6,
                seed: self.terrain_seed,
                frequency: self.terrain_frequency,
                amplitude: self.terrain_amplitude,
                octaves: self.terrain_octaves,
                lacunarity: 2.0,
                gain: self.terrain_gain,
            });

        let old_clipmap_center = world.resources.sdf_world.clipmap.center;
        world.resources.sdf_world.update(camera_position);

        let center_delta =
            nalgebra_glm::length(&(world.resources.sdf_world.clipmap.center - old_clipmap_center));
        if center_delta > 0.001 && !self.physics_objects.is_empty() {
            let edit_indices: Vec<usize> = self
                .physics_objects
                .iter()
                .flat_map(|object| object.edit_indices.iter().copied())
                .collect();
            for edit_index in edit_indices {
                world.resources.sdf_world.mark_edit_dirty(edit_index);
            }
        }

        if !self.collision_mesh_enabled && !self.collision_chunks.is_empty() {
            self.clear_collision_mesh(world);
        }

        let need_ground_body = !self.collision_mesh_enabled && !self.terrain_enabled;
        if !need_ground_body && self.ground_entity.is_some() {
            if let Some(entity) = self.ground_entity.take() {
                world.despawn_entities(&[entity]);
            }
        } else if need_ground_body && self.ground_entity.is_none() {
            self.spawn_ground_body(world);
        }

        self.update_collision_mesh(world, camera_position);
        self.update_collision_wireframe(world);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let sdf_pass = passes::SdfPass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(sdf_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

        let lines_pass = passes::LinesPass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(lines_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 1.0);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .read("ssao", resources.ssao)
            .write("output", resources.compute_output);

        let fxaa_output = graph
            .add_color_texture("fxaa_output")
            .format(surface_format)
            .size(resources.surface_width.max(1), resources.surface_height.max(1))
            .transient();

        let fxaa_pass = passes::FxaaPass::new(device, surface_format);
        graph
            .pass(Box::new(fxaa_pass))
            .read("input", resources.compute_output)
            .write("output", fxaa_output);

        let swapchain_blit_pass = passes::BlitPass::new(device, surface_format)
            .with_name("default_swapchain_blit");
        graph
            .pass(Box::new(swapchain_blit_pass))
            .read("input", fxaa_output)
            .write("output", resources.swapchain);

        self.sdf_pass_configured = true;
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        if world.resources.user_interface.hud_wants_pointer
            || world.resources.user_interface.consumed_event
        {
            return;
        }

        match key {
            KeyCode::Digit1 => self.brush_primitive = Some(BrushPrimitive::Sphere),
            KeyCode::Digit2 => self.brush_primitive = Some(BrushPrimitive::Box),
            KeyCode::Digit3 => self.brush_primitive = Some(BrushPrimitive::Cylinder),
            KeyCode::Digit4 => self.brush_primitive = Some(BrushPrimitive::Torus),
            KeyCode::Digit5 => self.brush_primitive = Some(BrushPrimitive::Capsule),
            KeyCode::F1 => self.brush_operation = Some(BrushOperation::Add),
            KeyCode::F2 => self.brush_operation = Some(BrushOperation::Subtract),
            KeyCode::F3 => self.brush_operation = Some(BrushOperation::SmoothAdd),
            KeyCode::F4 => self.brush_operation = Some(BrushOperation::SmoothSubtract),
            KeyCode::F5 => self.brush_operation = Some(BrushOperation::PhysicsSphere),
            KeyCode::F6 => self.brush_operation = Some(BrushOperation::PhysicsBox),
            KeyCode::F7 => self.brush_operation = Some(BrushOperation::PhysicsCapsule),
            KeyCode::F8 => self.brush_operation = Some(BrushOperation::PhysicsSnowman),
            _ => {}
        }
    }

    fn on_mouse_input(&mut self, world: &mut World, state: KeyState, button: MouseButton) {
        if button == MouseButton::Left {
            self.mouse_down = state == KeyState::Pressed;
            if self.mouse_down {
                self.last_apply_time = world.resources.window.timing.uptime_milliseconds as f32
                    / 1000.0
                    - self.apply_interval;
            }
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("SDF Sculpt")
            .default_pos([10.0, 60.0])
            .show(ui_context, |ui| {
                ui.heading("Brush Primitive");
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.brush_primitive,
                        Some(BrushPrimitive::Sphere),
                        "Sphere",
                    );
                    ui.selectable_value(
                        &mut self.brush_primitive,
                        Some(BrushPrimitive::Box),
                        "Box",
                    );
                    ui.selectable_value(
                        &mut self.brush_primitive,
                        Some(BrushPrimitive::Cylinder),
                        "Cylinder",
                    );
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.brush_primitive,
                        Some(BrushPrimitive::Torus),
                        "Torus",
                    );
                    ui.selectable_value(
                        &mut self.brush_primitive,
                        Some(BrushPrimitive::Capsule),
                        "Capsule",
                    );
                });

                ui.separator();

                ui.heading("Operation");
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.brush_operation,
                        Some(BrushOperation::Add),
                        "Add",
                    );
                    ui.selectable_value(
                        &mut self.brush_operation,
                        Some(BrushOperation::Subtract),
                        "Subtract",
                    );
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.brush_operation,
                        Some(BrushOperation::SmoothAdd),
                        "Smooth Add",
                    );
                    ui.selectable_value(
                        &mut self.brush_operation,
                        Some(BrushOperation::SmoothSubtract),
                        "Smooth Sub",
                    );
                });

                ui.separator();

                ui.heading("Brush Settings");
                ui.add(egui::Slider::new(&mut self.brush_size, 0.1..=5.0).text("Size"));
                ui.add(egui::Slider::new(&mut self.brush_smoothness, 0.0..=1.0).text("Smoothness"));
                ui.add(egui::Slider::new(&mut self.current_material, 0..=5).text("Material ID"));

                ui.separator();

                let mut smoothness_scale = world.resources.sdf_world.smoothness_scale;
                if ui
                    .add(
                        egui::Slider::new(&mut smoothness_scale, 0.0..=2.0)
                            .text("Global Smoothness"),
                    )
                    .changed()
                {
                    world
                        .resources
                        .sdf_world
                        .set_smoothness_scale(smoothness_scale);
                }

                ui.separator();

                ui.heading("Grid Snapping");
                ui.checkbox(&mut self.snap_to_grid, "Snap to Grid");
                if self.snap_to_grid {
                    ui.add(egui::Slider::new(&mut self.snap_level, 0..=5).text("Snap Level"));
                    let voxel_sizes = world.resources.sdf_world.voxel_sizes();
                    if let Some(voxel_size) = voxel_sizes.get(self.snap_level) {
                        ui.label(format!("Voxel size: {:.4}", voxel_size));
                    }
                }

                ui.separator();

                if ui.button("Reset Scene").clicked() {
                    self.clear_physics_objects();
                    self.clear_collision_mesh(world);
                    world.resources.sdf_world.clear();
                    self.spawn_initial_scene(world);
                }

                ui.separator();

                ui.checkbox(&mut self.terrain_enabled, "Enable Terrain");
                if self.terrain_enabled {
                    egui::CollapsingHeader::new("Terrain Settings")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add(
                                egui::Slider::new(&mut self.terrain_base_height, -20.0..=20.0)
                                    .text("Base Height"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.terrain_seed, 0..=9999).text("Seed"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.terrain_frequency, 0.01..=1.0)
                                    .logarithmic(true)
                                    .text("Frequency"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.terrain_amplitude, 0.1..=30.0)
                                    .text("Amplitude"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.terrain_octaves, 1..=11)
                                    .text("Octaves"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.terrain_gain, 0.1..=0.9).text("Gain"),
                            );
                        });
                }

                ui.separator();

                ui.checkbox(&mut self.collision_mesh_enabled, "Collision Mesh");
                if self.collision_mesh_enabled {
                    egui::CollapsingHeader::new("Collision Mesh Settings")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add(
                                egui::Slider::new(&mut self.collision_cell_size, 0.5..=4.0)
                                    .text("Cell Size"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.collision_mesh.terrain_octaves, 1..=11)
                                    .text("Octaves"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.collision_radius, 20.0..=80.0)
                                    .text("Radius"),
                            );
                            ui.add(
                                egui::Slider::new(
                                    &mut self.collision_mesh.max_chunks_per_frame,
                                    1..=8,
                                )
                                .text("Chunks/Frame"),
                            );
                            ui.checkbox(&mut self.show_collision_wireframe, "Show Wireframe");
                            ui.label(format!("Active chunks: {}", self.collision_chunks.len()));
                            ui.label(format!(
                                "Dirty chunks: {}",
                                self.collision_mesh.dirty_chunks.len()
                            ));
                            if ui.button("Rebuild All").clicked() {
                                self.clear_collision_mesh(world);
                                self.collision_mesh.cell_size = self.collision_cell_size;
                            }
                        });
                }

                ui.separator();

                ui.heading("Physics Objects");
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.brush_operation,
                        Some(BrushOperation::PhysicsSphere),
                        "Sphere",
                    );
                    ui.selectable_value(
                        &mut self.brush_operation,
                        Some(BrushOperation::PhysicsBox),
                        "Box",
                    );
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.brush_operation,
                        Some(BrushOperation::PhysicsCapsule),
                        "Capsule",
                    );
                    ui.selectable_value(
                        &mut self.brush_operation,
                        Some(BrushOperation::PhysicsSnowman),
                        "Snowman",
                    );
                });

                let is_physics_mode = matches!(
                    self.brush_operation,
                    Some(BrushOperation::PhysicsSphere)
                        | Some(BrushOperation::PhysicsBox)
                        | Some(BrushOperation::PhysicsCapsule)
                        | Some(BrushOperation::PhysicsSnowman)
                );

                if is_physics_mode {
                    ui.add(
                        egui::Slider::new(&mut self.physics_spawn_size, 0.3..=3.0)
                            .text("Spawn Size"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.physics_spawn_material, 0..=5)
                            .text("Spawn Material"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.physics_spawn_smoothness, 0.0..=1.0)
                            .text("Blending"),
                    );
                }

                ui.label(format!("Active: {}", self.physics_objects.len()));
                if ui.button("Clear Physics").clicked() {
                    let physics_edit_indices: Vec<usize> = self
                        .physics_objects
                        .iter()
                        .flat_map(|object| object.edit_indices.iter().copied())
                        .collect();

                    let mut sorted_indices = physics_edit_indices;
                    sorted_indices.sort_unstable();
                    for &index in sorted_indices.iter().rev() {
                        world.resources.sdf_world.remove_edit(index);
                    }
                    self.physics_objects.clear();
                }

                ui.separator();

                ui.checkbox(&mut self.show_debug_info, "Show Debug Info");
                ui.checkbox(&mut self.show_brick_grid, "Show Brick Grid");
                ui.checkbox(&mut self.debug_brick_coloring, "Debug Brick Colors");

                if self.show_brick_grid || self.debug_brick_coloring {
                    ui.add(egui::Slider::new(&mut self.brick_grid_level, 0..=5).text("Grid Level"));
                    ui.add(
                        egui::Slider::new(&mut self.brick_grid_radius, 1..=64).text("Grid Radius"),
                    );

                    let voxel_sizes = world.resources.sdf_world.voxel_sizes();
                    if let Some(voxel_size) = voxel_sizes.get(self.brick_grid_level) {
                        let brick_size = voxel_size * 8.0;
                        ui.label(format!("Brick size: {:.3}", brick_size));
                    }
                }

                if self.show_debug_info {
                    ui.separator();
                    ui.heading("Debug Info");

                    ui.label(format!(
                        "Brush position: ({:.2}, {:.2}, {:.2})",
                        self.brush_position.x, self.brush_position.y, self.brush_position.z
                    ));
                    ui.label(format!("Brush valid: {}", self.brush_valid));

                    ui.separator();

                    let sdf_world = &world.resources.sdf_world;
                    ui.label(format!("Edit count: {}", sdf_world.edit_count()));
                    ui.label(format!(
                        "Allocated bricks: {} / {}",
                        sdf_world.allocated_brick_count(),
                        sdf_world.max_brick_count()
                    ));
                    ui.label(format!("Clipmap levels: {}", sdf_world.level_count()));
                    ui.label(format!(
                        "Pending GPU dispatches: {}",
                        sdf_world.pending_gpu_dispatches.len()
                    ));

                    let voxel_sizes = sdf_world.voxel_sizes();
                    ui.collapsing("Voxel Sizes", |ui| {
                        for (level_index, size) in voxel_sizes.iter().enumerate() {
                            ui.label(format!("Level {}: {:.4}", level_index, size));
                        }
                    });
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Presets:");
                    if ui.small_button("CSG").clicked() {
                        Self::load_preset_csg_demo(world);
                    }
                    if ui.small_button("Cheese").clicked() {
                        Self::load_preset_swiss_cheese(world);
                    }
                    if ui.small_button("Tower").clicked() {
                        Self::load_preset_tower(world);
                    }
                });
                ui.horizontal(|ui| {
                    ui.add_space(50.0);
                    if ui.small_button("Molecule").clicked() {
                        Self::load_preset_molecule(world);
                    }
                    if ui.small_button("Donut").clicked() {
                        Self::load_preset_donut(world);
                    }
                });

                ui.add_space(4.0);

                let mut edit_to_remove: Option<usize> = None;
                let edit_count = world.resources.sdf_world.edits.len();

                let header_text = format!("Scene Edits ({})", edit_count);
                egui::CollapsingHeader::new(header_text)
                    .default_open(false)
                    .show(ui, |ui| {
                        if edit_count == 0 {
                            ui.weak("No edits - scene is empty");
                        } else {
                            let row_height = 20.0;
                            let max_visible_rows = 10;
                            let scroll_height =
                                (edit_count.min(max_visible_rows) as f32) * row_height;

                            egui::ScrollArea::vertical()
                                .max_height(scroll_height)
                                .auto_shrink([false, true])
                                .show_rows(ui, row_height, edit_count, |ui, row_range| {
                                    for edit_index in row_range {
                                        let edit = &world.resources.sdf_world.edits[edit_index];
                                        let pos = edit.position();
                                        let smoothness = edit.smoothness();

                                        ui.horizontal(|ui| {
                                            ui.monospace(format!("{:2}.", edit_index));

                                            let op_color = match edit {
                                                SdfEdit::Union { .. } => {
                                                    egui::Color32::from_rgb(100, 200, 100)
                                                }
                                                SdfEdit::Subtraction { .. } => {
                                                    egui::Color32::from_rgb(200, 100, 100)
                                                }
                                                SdfEdit::Intersection { .. } => {
                                                    egui::Color32::from_rgb(100, 100, 200)
                                                }
                                                SdfEdit::SmoothUnion { .. } => {
                                                    egui::Color32::from_rgb(150, 255, 150)
                                                }
                                                SdfEdit::SmoothSubtraction { .. } => {
                                                    egui::Color32::from_rgb(255, 150, 150)
                                                }
                                                SdfEdit::SmoothIntersection { .. } => {
                                                    egui::Color32::from_rgb(150, 150, 255)
                                                }
                                            };

                                            ui.colored_label(op_color, edit.operation_name());
                                            ui.label(edit.primitive_name());

                                            if smoothness > 0.0 {
                                                ui.weak(format!("k={:.2}", smoothness));
                                            }

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.small_button("X").clicked() {
                                                        edit_to_remove = Some(edit_index);
                                                    }
                                                    ui.weak(format!(
                                                        "({:.1}, {:.1}, {:.1})",
                                                        pos.x, pos.y, pos.z
                                                    ));
                                                },
                                            );
                                        });
                                    }
                                });
                        }

                        ui.add_space(2.0);
                        if edit_count > 0 && ui.small_button("Clear All").clicked() {
                            self.clear_physics_objects();
                            world.resources.sdf_world.clear();
                        }
                    });

                if let Some(index) = edit_to_remove {
                    world.resources.sdf_world.remove_edit(index);
                    self.adjust_physics_indices_after_removal(index);
                }

                ui.separator();

                ui.label("Controls:");
                ui.label("WASD: Move | Space/Shift: Up/Down");
                ui.label("Right-Drag: Look");
                ui.label("Hold Left-Click: Sculpt");
                ui.label("1-5: Select primitive");
                ui.label("F1-F4: Select operation");
                ui.label("F5: Spawn physics sphere");
            });
    }
}
