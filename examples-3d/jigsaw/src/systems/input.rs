use crate::ecs::{DRAGGING, Dragging, ENGINE_ENTITY, IN_GROUP, PUZZLE_PIECE, PuzzleWorld, ZOrder};
use crate::mesh::point_in_polygon;
use nightshade::prelude::*;

pub fn input_system(puzzle_world: &mut PuzzleWorld, world: &mut World) {
    let mouse = &world.resources.input.mouse;
    let mouse_pos = nalgebra_glm::vec2(mouse.position.x, mouse.position.y);

    let world_pos = get_ground_position_from_screen(world, mouse_pos, 0.0);

    let left_just_pressed = mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let left_released = mouse.state.contains(MouseState::LEFT_JUST_RELEASED);
    let left_held = mouse.state.contains(MouseState::LEFT_CLICKED);
    let right_just_pressed = mouse.state.contains(MouseState::RIGHT_JUST_PRESSED);

    if right_just_pressed && let Some(hovered) = puzzle_world.resources.hovered_piece {
        rotate_piece(puzzle_world, world, hovered);
    }

    let dragging_entities: Vec<_> = puzzle_world.query_entities(DRAGGING).collect();

    if left_released && !dragging_entities.is_empty() {
        for entity in dragging_entities {
            puzzle_world.remove_dragging(entity);
        }
        return;
    }

    if left_held && !dragging_entities.is_empty() {
        if let Some(world_pos) = world_pos {
            for drag_entity in &dragging_entities {
                if let Some(dragging) = puzzle_world.get_dragging(*drag_entity) {
                    let new_x = world_pos.x - dragging.cursor_offset_x;
                    let new_z = world_pos.z - dragging.cursor_offset_z;

                    if let Some(in_group) = puzzle_world.get_in_group(*drag_entity) {
                        let group_entity = in_group.0;
                        update_group_position(puzzle_world, world, group_entity, new_x, new_z);
                    }
                }
            }
        }
        return;
    }

    if left_just_pressed
        && let Some(world_pos) = world_pos
        && let Some(hit_piece) = pick_piece_at(puzzle_world, world, world_pos.x, world_pos.z)
    {
        puzzle_world.resources.z_counter += 1;
        let new_z_order = puzzle_world.resources.z_counter;

        if let Some(in_group) = puzzle_world.get_in_group(hit_piece) {
            let group_entity = in_group.0;

            if let Some(group_members) = puzzle_world.get_group_members(group_entity) {
                let offset_x = world_pos.x - group_members.world_x;
                let offset_z = world_pos.z - group_members.world_z;

                for &member in &group_members.members.clone() {
                    puzzle_world.set_z_order(member, ZOrder(new_z_order));

                    if !puzzle_world.entity_has_dragging(member) {
                        puzzle_world.add_dragging(member);
                    }
                    puzzle_world.set_dragging(
                        member,
                        Dragging {
                            cursor_offset_x: offset_x,
                            cursor_offset_z: offset_z,
                        },
                    );

                    if let Some(engine_entity) = puzzle_world.get_engine_entity(member) {
                        if let Some(transform) = world.core.get_local_transform_mut(engine_entity.0) {
                            transform.translation.y = 0.2;
                        }
                        world.core.set_local_transform_dirty(engine_entity.0, LocalTransformDirty);
                    }
                }
            }
        }
    }

    puzzle_world.resources.hovered_piece = None;
    if let Some(world_pos) = world_pos
        && let Some(hit_piece) = pick_piece_at(puzzle_world, world, world_pos.x, world_pos.z)
    {
        puzzle_world.resources.hovered_piece = Some(hit_piece);
    }
}

fn pick_piece_at(
    puzzle_world: &PuzzleWorld,
    world: &World,
    world_x: f32,
    world_z: f32,
) -> Option<freecs::Entity> {
    let mut candidates: Vec<(freecs::Entity, u32)> = Vec::new();

    for piece_entity in puzzle_world.query_entities(ENGINE_ENTITY | PUZZLE_PIECE | IN_GROUP) {
        let engine_entity = puzzle_world.get_engine_entity(piece_entity)?;
        let puzzle_piece = puzzle_world.get_puzzle_piece(piece_entity)?;
        let z_order = puzzle_world
            .get_z_order(piece_entity)
            .map(|z| z.0)
            .unwrap_or(0);

        let transform = world.core.get_local_transform(engine_entity.0)?;
        let piece_x = transform.translation.x;
        let piece_z = transform.translation.z;

        let local_x = world_x - piece_x;
        let local_z = world_z - piece_z;

        if let Some(outline) = puzzle_world
            .resources
            .piece_outlines
            .get(&puzzle_piece.grid_pos)
        {
            let local_point = nalgebra_glm::vec2(local_x, -local_z);

            if point_in_polygon(local_point, outline, puzzle_piece.rotation) {
                candidates.push((piece_entity, z_order));
            }
        }
    }

    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates.first().map(|(entity, _)| *entity)
}

fn update_group_position(
    puzzle_world: &mut PuzzleWorld,
    world: &mut World,
    group_entity: freecs::Entity,
    new_x: f32,
    new_z: f32,
) {
    if let Some(group_members) = puzzle_world.get_group_members_mut(group_entity) {
        group_members.world_x = new_x;
        group_members.world_z = new_z;

        for &member in &group_members.members.clone() {
            let local_offset = puzzle_world
                .get_local_offset(member)
                .map(|o| (o.x, o.y))
                .unwrap_or((0.0, 0.0));

            if let Some(engine_entity) = puzzle_world.get_engine_entity(member) {
                if let Some(transform) = world.core.get_local_transform_mut(engine_entity.0) {
                    transform.translation.x = new_x + local_offset.0;
                    transform.translation.z = new_z + local_offset.1;
                }
                world.core.set_local_transform_dirty(engine_entity.0, LocalTransformDirty);
            }
        }
    }
}

fn rotate_piece(puzzle_world: &mut PuzzleWorld, world: &mut World, piece_entity: freecs::Entity) {
    if let Some(in_group) = puzzle_world.get_in_group(piece_entity) {
        let group_entity = in_group.0;

        let group_pos = puzzle_world
            .get_group_members(group_entity)
            .map(|g| (g.world_x, g.world_z))
            .unwrap_or((0.0, 0.0));

        let members: Vec<_> = puzzle_world
            .get_group_members(group_entity)
            .map(|g| g.members.clone())
            .unwrap_or_default();

        for &member in &members {
            if let Some(local_offset) = puzzle_world.get_local_offset_mut(member) {
                let old_x = local_offset.x;
                let old_y = local_offset.y;
                local_offset.x = -old_y;
                local_offset.y = old_x;
            }

            if let Some(puzzle_piece) = puzzle_world.get_puzzle_piece_mut(member) {
                puzzle_piece.rotation = (puzzle_piece.rotation + 1) % 4;
            }
        }

        for &member in &members {
            let local_offset = puzzle_world
                .get_local_offset(member)
                .map(|o| (o.x, o.y))
                .unwrap_or((0.0, 0.0));

            let new_rotation = puzzle_world
                .get_puzzle_piece(member)
                .map(|p| p.rotation)
                .unwrap_or(0);

            if let Some(engine_entity) = puzzle_world.get_engine_entity(member) {
                if let Some(transform) = world.core.get_local_transform_mut(engine_entity.0) {
                    transform.translation.x = group_pos.0 + local_offset.0;
                    transform.translation.z = group_pos.1 + local_offset.1;

                    let angle = new_rotation as f32 * std::f32::consts::FRAC_PI_2;
                    transform.rotation =
                        nalgebra_glm::quat_angle_axis(angle, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
                }
                world.core.set_local_transform_dirty(engine_entity.0, LocalTransformDirty);
            }
        }
    }
}
