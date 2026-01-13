use crate::ecs::{ChessWorld, PieceColor, PieceType, SquarePosition, calculate_valid_moves};
use crate::systems::hover::get_ground_intersection;
use nightshade::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

fn extract_circle_index(name: &str) -> Option<usize> {
    if name == "Circle" {
        return Some(0);
    }
    if let Some(rest) = name.strip_prefix("Circle.") {
        let numeric_part: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !numeric_part.is_empty() {
            return numeric_part.parse().ok();
        }
    }
    if let Some(rest) = name.strip_prefix("Circle") {
        let numeric_part: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !numeric_part.is_empty() {
            return numeric_part.parse().ok();
        }
    }
    None
}

fn get_piece_info_at_entity(
    world: &World,
    entity: Entity,
) -> (Option<PieceType>, Option<PieceColor>) {
    if let Some(name) = world.get_name(entity) {
        let name_str = &name.0;

        if let Some(index) = extract_circle_index(name_str) {
            let (piece_type, color) = match index {
                0..=5 => {
                    let piece_type = match index {
                        0 => PieceType::Pawn,
                        1 => PieceType::Rook,
                        2 => PieceType::Knight,
                        3 => PieceType::Bishop,
                        4 => PieceType::Queen,
                        5 => PieceType::King,
                        _ => PieceType::Pawn,
                    };
                    (piece_type, PieceColor::White)
                }
                6..=7 => return (None, None),
                8..=13 => {
                    let piece_type = match index {
                        8 => PieceType::Pawn,
                        9 => PieceType::Rook,
                        10 => PieceType::Knight,
                        11 => PieceType::Bishop,
                        12 => PieceType::Queen,
                        13 => PieceType::King,
                        _ => PieceType::Pawn,
                    };
                    (piece_type, PieceColor::Black)
                }
                _ => return (None, None),
            };
            return (Some(piece_type), Some(color));
        }
    }
    (None, None)
}

fn is_piece_entity(name: &str) -> bool {
    name.contains("Circle") && name != "Plane" && name != "Board"
}

fn get_occupied_squares_with_colors(
    world: &World,
    square_size: f32,
) -> HashMap<SquarePosition, PieceColor> {
    let mut occupied = HashMap::new();
    for entity in world.query_entities(GLOBAL_TRANSFORM | NAME) {
        if let Some(name) = world.get_name(entity)
            && is_piece_entity(&name.0)
            && let Some(transform) = world.get_global_transform(entity)
        {
            let pos = transform.0.column(3).xyz();
            let square = SquarePosition::from_world_position(pos, square_size);
            if square.is_valid() {
                let (_, piece_color) = get_piece_info_at_entity(world, entity);
                if let Some(color) = piece_color {
                    occupied.insert(square, color);
                }
            }
        }
    }
    occupied
}

fn find_piece_entity_at_square(
    world: &World,
    target_square: SquarePosition,
    square_size: f32,
    exclude_entity: Entity,
) -> Option<Entity> {
    for entity in world.query_entities(GLOBAL_TRANSFORM | NAME) {
        if entity == exclude_entity {
            continue;
        }
        if let Some(name) = world.get_name(entity)
            && is_piece_entity(&name.0)
            && let Some(transform) = world.get_global_transform(entity)
        {
            let pos = transform.0.column(3).xyz();
            let square = SquarePosition::from_world_position(pos, square_size);
            if square == target_square {
                return Some(entity);
            }
        }
    }
    None
}

fn find_closest_valid_square(
    piece_pos: Vec3,
    valid_moves: &HashSet<SquarePosition>,
    square_size: f32,
) -> Option<SquarePosition> {
    let mut closest: Option<(SquarePosition, f32)> = None;

    for square in valid_moves {
        let square_world = square.to_world_position(square_size);
        let distance = nalgebra_glm::distance(
            &nalgebra_glm::vec2(piece_pos.x, piece_pos.z),
            &nalgebra_glm::vec2(square_world.x, square_world.z),
        );

        if distance < square_size * 0.8 {
            match closest {
                None => closest = Some((*square, distance)),
                Some((_, best_dist)) if distance < best_dist => closest = Some((*square, distance)),
                _ => {}
            }
        }
    }

    closest.map(|(square, _)| square)
}

fn set_entity_position(world: &mut World, entity: Entity, position: Vec3) {
    if let Some(parent) = world.get_parent(entity)
        && let Some(parent_entity) = parent.0
        && let Some(parent_global) = world.get_global_transform(parent_entity)
        && let Some(parent_inverse) = parent_global.0.try_inverse()
    {
        let local_pos = parent_inverse.transform_point(&nalgebra_glm::Vec3::from(position).into());
        if let Some(transform) = world.get_local_transform_mut(entity) {
            transform.translation = local_pos.coords;
        }
        world.mark_local_transform_dirty(entity);
        return;
    }

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.translation = position;
    }
    world.mark_local_transform_dirty(entity);
}

pub fn input_system(chess_world: &mut ChessWorld, world: &mut World) {
    let mouse = &world.resources.input.mouse;
    let left_pressed = mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let left_held = mouse.state.contains(MouseState::LEFT_CLICKED);
    let left_released = mouse.state.contains(MouseState::LEFT_JUST_RELEASED);
    let mouse_pos = mouse.position;

    if left_pressed
        && !chess_world.resources.is_dragging
        && let Some(hovered_entity) = chess_world.resources.hovered_engine_entity
    {
        chess_world.resources.dragged_engine_entity = Some(hovered_entity);
        chess_world.resources.is_dragging = true;
        world.resources.graphics.bounding_volume_selected_entity = Some(hovered_entity);

        if let Some(click_pos) = get_ground_intersection(world, mouse_pos)
            && let Some(transform) = world.get_global_transform(hovered_entity)
        {
            let entity_pos = transform.0.column(3).xyz();
            chess_world.resources.drag_offset = entity_pos - click_pos;
            chess_world.resources.drag_start_pos = entity_pos;

            let square_size = chess_world.resources.square_size;
            let start_square = SquarePosition::from_world_position(entity_pos, square_size);
            chess_world.resources.drag_start_square = start_square;

            let (piece_type, piece_color) = get_piece_info_at_entity(world, hovered_entity);
            chess_world.resources.dragged_piece_type = piece_type;
            chess_world.resources.dragged_piece_color = piece_color;

            if let (Some(piece_type), Some(color)) = (piece_type, piece_color) {
                let occupied = get_occupied_squares_with_colors(world, square_size);
                let valid_moves = calculate_valid_moves(piece_type, color, start_square, &occupied);
                chess_world.resources.valid_moves = valid_moves.clone();
                super::highlight::spawn_move_indicators(
                    chess_world,
                    world,
                    &valid_moves,
                    square_size,
                );
            }
        }
    }

    if left_held
        && chess_world.resources.is_dragging
        && let Some(dragged_entity) = chess_world.resources.dragged_engine_entity
        && let Some(click_pos) = get_ground_intersection(world, mouse_pos)
    {
        let new_pos = click_pos + chess_world.resources.drag_offset;
        let lifted_pos = nalgebra_glm::vec3(new_pos.x, 0.0006, new_pos.z);

        set_entity_position(world, dragged_entity, lifted_pos);

        let square_size = chess_world.resources.square_size;
        let closest =
            find_closest_valid_square(lifted_pos, &chess_world.resources.valid_moves, square_size);
        chess_world.resources.closest_valid_square = closest;
    }

    if left_released && chess_world.resources.is_dragging {
        super::highlight::despawn_move_indicators(chess_world, world);

        if let Some(dragged_entity) = chess_world.resources.dragged_engine_entity {
            let square_size = chess_world.resources.square_size;

            let final_pos = if let Some(target_square) = chess_world.resources.closest_valid_square
            {
                if let Some(captured_entity) =
                    find_piece_entity_at_square(world, target_square, square_size, dragged_entity)
                {
                    world.queue_command(WorldCommand::DespawnRecursive {
                        entity: captured_entity,
                    });
                }
                target_square.to_world_position(square_size)
            } else {
                chess_world.resources.drag_start_pos
            };

            set_entity_position(world, dragged_entity, final_pos);
        }

        chess_world.resources.dragged_engine_entity = None;
        chess_world.resources.is_dragging = false;
        chess_world.resources.valid_moves.clear();
        chess_world.resources.closest_valid_square = None;
        chess_world.resources.dragged_piece_type = None;
        chess_world.resources.dragged_piece_color = None;
        world.resources.graphics.bounding_volume_selected_entity = None;
    }
}
