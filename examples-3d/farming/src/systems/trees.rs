use nightshade::prelude::*;

use crate::data::ITEM_WOOD;
use crate::ecs::{GameEntity, TreeState, World as GameWorld};
use crate::events::TreeChoppedEvent;
use crate::systems::player::get_player_position;
use crate::types::{CHOP_RANGE, TOOL_STAMINA_COST};

pub fn update_target(game: &mut GameWorld) {
    let player_pos = get_player_position(game);

    let mut nearest: Option<(f32, GameEntity)> = None;

    for trees in game.resources.trees.by_chunk.values() {
        for &tree_entity in trees {
            let Some(tree) = game.get_tree(tree_entity) else {
                continue;
            };
            if tree.state != TreeState::Standing {
                continue;
            }

            let Some(position) = game.get_position(tree_entity) else {
                continue;
            };

            let dx = player_pos.x - position.0.x;
            let dz = player_pos.z - position.0.z;
            let distance = (dx * dx + dz * dz).sqrt();

            if distance < CHOP_RANGE && (nearest.is_none() || distance < nearest.unwrap().0) {
                nearest = Some((distance, tree_entity));
            }
        }
    }

    game.resources.targeted_tree = nearest.map(|(_, entity)| entity);
}

pub fn try_chop(game: &mut GameWorld) -> bool {
    let Some(tree_entity) = game.resources.targeted_tree else {
        return false;
    };

    let is_standing = game
        .get_tree(tree_entity)
        .map(|t| t.state == TreeState::Standing)
        .unwrap_or(false);

    if !is_standing {
        return false;
    }

    let Some(player_entity) = game.resources.player_entity else {
        return false;
    };

    let has_stamina = game
        .get_player(player_entity)
        .map(|p| p.stamina >= TOOL_STAMINA_COST)
        .unwrap_or(false);

    if !has_stamina {
        return false;
    }

    game.modify_player(player_entity, |p| p.stamina -= TOOL_STAMINA_COST);

    let tree_pos = game
        .get_position(tree_entity)
        .map(|p| p.0)
        .unwrap_or(Vec3::zeros());
    let player_pos = get_player_position(game);
    let to_tree = tree_pos - player_pos;
    let fall_dir = nalgebra_glm::normalize(&Vec3::new(to_tree.x, 0.0, to_tree.z));

    game.modify_tree(tree_entity, |tree| {
        tree.health -= 1.0;
        if tree.health <= 0.0 {
            tree.state = TreeState::Falling;
            tree.fall_direction = fall_dir;
            tree.fall_progress = 0.0;
        }
    });

    true
}

struct TreeAnimData {
    entity: GameEntity,
    position: Vec3,
    trunk_visual: Option<Entity>,
    foliage_visuals: [Option<Entity>; 3],
    trunk_y_offset: f32,
    foliage_y_offsets: [f32; 3],
    original_trunk_scale: Vec3,
    original_foliage_scales: [Vec3; 3],
    fall_direction: Vec3,
    fall_progress: f32,
    shrink_progress: f32,
    state: TreeState,
    chunk: (i32, i32),
    trunk_height: f32,
}

pub fn update(game: &mut GameWorld, world: &mut World) -> Vec<TreeChoppedEvent> {
    let delta = world.resources.window.timing.delta_time;

    let mut anim_data: Vec<TreeAnimData> = Vec::new();

    for trees in game.resources.trees.by_chunk.values() {
        for &tree_entity in trees {
            let Some(tree) = game.get_tree(tree_entity) else {
                continue;
            };
            if tree.state == TreeState::Standing {
                continue;
            }

            let position = game
                .get_position(tree_entity)
                .map(|p| p.0)
                .unwrap_or(Vec3::zeros());

            anim_data.push(TreeAnimData {
                entity: tree_entity,
                position,
                trunk_visual: tree.trunk_visual,
                foliage_visuals: tree.foliage_visuals,
                trunk_y_offset: tree.trunk_height / 2.0,
                foliage_y_offsets: tree.foliage_y_offsets,
                original_trunk_scale: tree.original_trunk_scale,
                original_foliage_scales: tree.original_foliage_scales,
                fall_direction: tree.fall_direction,
                fall_progress: tree.fall_progress,
                shrink_progress: tree.shrink_progress,
                state: tree.state,
                chunk: tree.chunk,
                trunk_height: tree.trunk_height,
            });
        }
    }

    let mut trees_to_finalize: Vec<TreeAnimData> = Vec::new();

    for data in anim_data {
        match data.state {
            TreeState::Falling => {
                let new_progress = data.fall_progress + delta * 1.5;
                let progress = new_progress.min(1.0);
                let ease = progress * progress;
                let fall_angle = ease * std::f32::consts::FRAC_PI_2;

                let fall_axis = nalgebra_glm::cross(&Vec3::y(), &data.fall_direction);
                let fall_axis_norm = if nalgebra_glm::length(&fall_axis) > 0.001 {
                    nalgebra_glm::normalize(&fall_axis)
                } else {
                    Vec3::x()
                };
                let fall_rot = nalgebra_glm::quat_angle_axis(fall_angle, &fall_axis_norm);

                if let Some(trunk) = data.trunk_visual {
                    if let Some(transform) = world.get_local_transform_mut(trunk) {
                        let offset = nalgebra_glm::quat_rotate_vec3(
                            &fall_rot,
                            &Vec3::new(0.0, data.trunk_y_offset, 0.0),
                        );
                        transform.translation =
                            Vec3::new(data.position.x, 0.0, data.position.z) + offset;
                        transform.rotation = fall_rot;
                    }
                    mark_local_transform_dirty(world, trunk);
                }

                for (foliage_index, foliage) in data.foliage_visuals.iter().enumerate() {
                    let Some(foliage_entity) = foliage else {
                        continue;
                    };
                    if let Some(transform) = world.get_local_transform_mut(*foliage_entity) {
                        let offset = nalgebra_glm::quat_rotate_vec3(
                            &fall_rot,
                            &Vec3::new(0.0, data.foliage_y_offsets[foliage_index], 0.0),
                        );
                        transform.translation =
                            Vec3::new(data.position.x, 0.0, data.position.z) + offset;
                        transform.rotation = fall_rot;
                    }
                    mark_local_transform_dirty(world, *foliage_entity);
                }

                game.modify_tree(data.entity, |tree| {
                    tree.fall_progress = new_progress;
                    if new_progress >= 1.0 {
                        tree.state = TreeState::Shrinking;
                        tree.shrink_progress = 0.0;
                    }
                });
            }
            TreeState::Shrinking => {
                let new_progress = data.shrink_progress + delta * 3.0;
                let progress = new_progress.min(1.0);
                let scale_factor = (1.0 - progress).max(0.01);

                if let Some(trunk) = data.trunk_visual {
                    if let Some(transform) = world.get_local_transform_mut(trunk) {
                        transform.scale = data.original_trunk_scale * scale_factor;
                    }
                    mark_local_transform_dirty(world, trunk);
                }

                for (foliage_index, foliage) in data.foliage_visuals.iter().enumerate() {
                    let Some(foliage_entity) = foliage else {
                        continue;
                    };
                    if let Some(transform) = world.get_local_transform_mut(*foliage_entity) {
                        transform.scale =
                            data.original_foliage_scales[foliage_index] * scale_factor;
                    }
                    mark_local_transform_dirty(world, *foliage_entity);
                }

                game.modify_tree(data.entity, |tree| {
                    tree.shrink_progress = new_progress;
                });

                if new_progress >= 1.0 {
                    trees_to_finalize.push(data);
                }
            }
            TreeState::Standing => {}
        }
    }

    let mut chopped_events: Vec<TreeChoppedEvent> = Vec::new();

    for data in trees_to_finalize {
        if let Some(trunk) = data.trunk_visual {
            world.queue_despawn_entity(trunk);
        }
        for foliage in data.foliage_visuals.iter().flatten() {
            world.queue_despawn_entity(*foliage);
        }

        if let Some(chunk_trees) = game.resources.trees.by_chunk.get_mut(&data.chunk) {
            chunk_trees.retain(|&entity| entity != data.entity);
        }

        if game.resources.targeted_tree == Some(data.entity) {
            game.resources.targeted_tree = None;
        }

        game.queue_despawn_entity(data.entity);

        let wood: u32 = 3 + (rand::random::<u32>() % 3);
        game.resources.inventory.add_item(ITEM_WOOD, wood);

        let fall_end = data.position + Vec3::new(data.trunk_height * 0.3, 0.0, 0.0);
        chopped_events.push(TreeChoppedEvent {
            position: Vec3::new(fall_end.x, 1.5, fall_end.z),
            wood,
        });
    }

    game.apply_commands();
    chopped_events
}
