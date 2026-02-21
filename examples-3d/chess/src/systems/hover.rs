use crate::ecs::{ChessWorld, SquarePosition};
use nightshade::ecs::bounding_volume::components::{BoundingVolume, OrientedBoundingBox};
use nightshade::ecs::picking::queries::{PickingOptions, PickingRay, pick_entities_trimesh};
use nightshade::prelude::*;

fn collect_descendants(world: &World, entity: Entity, descendants: &mut Vec<Entity>) {
    descendants.push(entity);
    if let Some(children) = world.resources.children_cache.get(&entity) {
        for child in children {
            collect_descendants(world, *child, descendants);
        }
    }
}

fn compute_combined_bounding_volume(world: &World, root_entity: Entity) -> Option<BoundingVolume> {
    let mut descendants = Vec::new();
    collect_descendants(world, root_entity, &mut descendants);

    let root_global = world.get_global_transform(root_entity)?;
    let root_inverse = root_global.0.try_inverse()?;

    let mut min_corner = nalgebra_glm::vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max_corner = nalgebra_glm::vec3(f32::MIN, f32::MIN, f32::MIN);
    let mut found_any = false;

    for entity in descendants {
        if let Some(bounding_volume) = world.get_bounding_volume(entity)
            && let Some(global_transform) = world.get_global_transform(entity)
        {
            let world_obb = bounding_volume.obb.transform(&global_transform.0);
            let corners = world_obb.get_corners();
            for corner in &corners {
                let corner_point = nalgebra_glm::Vec4::new(corner.x, corner.y, corner.z, 1.0);
                let local_corner = root_inverse * corner_point;
                let local_vec = nalgebra_glm::vec3(local_corner.x, local_corner.y, local_corner.z);
                min_corner = nalgebra_glm::min2(&min_corner, &local_vec);
                max_corner = nalgebra_glm::max2(&max_corner, &local_vec);
            }
            found_any = true;
        }
    }

    if found_any {
        let combined_obb = OrientedBoundingBox::from_aabb(min_corner, max_corner);
        let sphere_radius = nalgebra_glm::length(&combined_obb.half_extents);
        Some(BoundingVolume::new(combined_obb, sphere_radius))
    } else {
        None
    }
}

fn find_piece_root(world: &World, entity: Entity) -> Option<Entity> {
    let mut current = entity;
    let mut best_candidate: Option<Entity> = None;

    loop {
        if let Some(name) = world.get_name(current) {
            let name_str = &name.0;
            if name_str == "Plane" || name_str == "Board" {
                return None;
            }
            if name_str.contains("Circle") {
                best_candidate = Some(current);
            }
        }

        if let Some(parent) = world.get_parent(current) {
            if let Some(parent_entity) = parent.0 {
                current = parent_entity;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    best_candidate
}

pub fn get_ground_intersection(world: &World, mouse_pos: Vec2) -> Option<Vec3> {
    let ray = PickingRay::from_screen_position(world, mouse_pos)?;
    ray.intersect_ground_plane(0.0)
}

pub fn hover_system(chess_world: &mut ChessWorld, world: &mut World) {
    let mouse = &world.resources.input.mouse;
    let mouse_pos = mouse.position;
    let square_size = chess_world.resources.square_size;

    if let Some(world_pos) = get_ground_intersection(world, mouse_pos) {
        let square = SquarePosition::from_world_position(world_pos, square_size);
        if square.is_valid() {
            chess_world.resources.hovered_square = Some(square);
        } else {
            chess_world.resources.hovered_square = None;
        }
    } else {
        chess_world.resources.hovered_square = None;
    }

    let picked = pick_entities_trimesh(world, mouse_pos, PickingOptions::default());

    let camera_entity = world.resources.active_camera;
    let hovered_entity = picked.iter().find_map(|result| {
        if Some(result.entity) == camera_entity {
            return None;
        }
        find_piece_root(world, result.entity)
    });

    chess_world.resources.hovered_engine_entity = hovered_entity;

    let display_entity = if chess_world.resources.is_dragging {
        chess_world.resources.dragged_engine_entity
    } else {
        hovered_entity
    };

    if let Some(piece_root) = display_entity {
        if let Some(combined_bv) = compute_combined_bounding_volume(world, piece_root) {
            world.set_bounding_volume(piece_root, combined_bv);
        }
        world.resources.graphics.show_selected_bounding_volume = false;
        world.resources.graphics.bounding_volume_selected_entity = Some(piece_root);
    } else {
        world.resources.graphics.show_selected_bounding_volume = false;
        world.resources.graphics.bounding_volume_selected_entity = None;
    }
}
