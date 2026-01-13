use crate::ecs::{
    Direction, ENGINE_ENTITY, EdgeKey, EdgeProfile, EdgeType, EngineEntity, GROUP_MEMBERS,
    GroupMembers, IN_GROUP, IVec2, InGroup, LocalOffset, PIECE_GROUP, PUZZLE_PIECE, PieceGroup,
    PuzzlePiece, PuzzleWorld, ZOrder,
};
use crate::mesh::{TabParams, generate_piece_mesh, generate_piece_outline};
use nightshade::ecs::bounding_volume::components::OrientedBoundingBox;
use nightshade::ecs::lines::components::{Line, Lines};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::texture_cache_add_reference;
use rand::Rng;

pub fn initialize_edge_types(puzzle_world: &mut PuzzleWorld) {
    let cols = puzzle_world.resources.grid_cols;
    let rows = puzzle_world.resources.grid_rows;
    let mut rng = rand::rng();

    for y in 0..rows {
        for x in 0..=cols {
            let key = EdgeKey::new(x as i32, y as i32, Direction::Vertical);
            let edge_type = if x == 0 || x == cols {
                EdgeType::Flat
            } else if rng.random_bool(0.5) {
                EdgeType::Tab
            } else {
                EdgeType::Blank
            };
            puzzle_world.resources.edge_types.insert(key, edge_type);
        }
    }

    for y in 0..=rows {
        for x in 0..cols {
            let key = EdgeKey::new(x as i32, y as i32, Direction::Horizontal);
            let edge_type = if y == 0 || y == rows {
                EdgeType::Flat
            } else if rng.random_bool(0.5) {
                EdgeType::Tab
            } else {
                EdgeType::Blank
            };
            puzzle_world.resources.edge_types.insert(key, edge_type);
        }
    }
}

fn get_edge_profile(puzzle_world: &PuzzleWorld, grid_x: i32, grid_y: i32) -> EdgeProfile {
    let top_key = EdgeKey::new(grid_x, grid_y, Direction::Horizontal);
    let bottom_key = EdgeKey::new(grid_x, grid_y + 1, Direction::Horizontal);
    let left_key = EdgeKey::new(grid_x, grid_y, Direction::Vertical);
    let right_key = EdgeKey::new(grid_x + 1, grid_y, Direction::Vertical);

    let top = puzzle_world
        .resources
        .edge_types
        .get(&top_key)
        .copied()
        .unwrap_or(EdgeType::Flat)
        .inverse();

    let bottom = puzzle_world
        .resources
        .edge_types
        .get(&bottom_key)
        .copied()
        .unwrap_or(EdgeType::Flat);

    let left = puzzle_world
        .resources
        .edge_types
        .get(&left_key)
        .copied()
        .unwrap_or(EdgeType::Flat)
        .inverse();

    let right = puzzle_world
        .resources
        .edge_types
        .get(&right_key)
        .copied()
        .unwrap_or(EdgeType::Flat);

    EdgeProfile {
        top,
        right,
        bottom,
        left,
    }
}

pub fn spawn_puzzle_pieces(puzzle_world: &mut PuzzleWorld, world: &mut World, texture_name: &str) {
    let cols = puzzle_world.resources.grid_cols;
    let rows = puzzle_world.resources.grid_rows;
    let piece_width = puzzle_world.resources.piece_width;
    let piece_height = puzzle_world.resources.piece_height;
    let tab_params = TabParams {
        depth: puzzle_world.resources.tab_depth,
        width: puzzle_world.resources.tab_width,
        neck_width: puzzle_world.resources.neck_width,
    };

    for grid_y in 0..rows {
        for grid_x in 0..cols {
            let grid_pos = IVec2::new(grid_x as i32, grid_y as i32);
            let profile = get_edge_profile(puzzle_world, grid_x as i32, grid_y as i32);

            let (mesh, outline) = generate_piece_mesh(
                &profile,
                grid_pos,
                cols,
                rows,
                piece_width,
                piece_height,
                tab_params,
            );

            puzzle_world
                .resources
                .piece_outlines
                .insert(grid_pos, outline);

            let mesh_name = format!("PuzzlePiece_{}_{}", grid_x, grid_y);
            mesh_cache_insert(&mut world.resources.mesh_cache, mesh_name.clone(), mesh);

            let entity = world.spawn_entities(
                RENDER_MESH
                    | MATERIAL_REF
                    | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | VISIBILITY
                    | BOUNDING_VOLUME
                    | NAME
                    | CASTS_SHADOW,
                1,
            )[0];
            world.set_casts_shadow(entity, CastsShadow);

            if let Some(&index) = world
                .resources
                .mesh_cache
                .registry
                .name_to_index
                .get(&mesh_name)
            {
                world.resources.mesh_cache.registry.add_reference(index);
            }

            world.set_render_mesh(entity, RenderMesh::new(&mesh_name));
            world.set_name(entity, Name(format!("Piece_{}_{}", grid_x, grid_y)));

            let solved_x = (grid_x as f32 - (cols as f32 - 1.0) / 2.0) * piece_width;
            let solved_z = (grid_y as f32 - (rows as f32 - 1.0) / 2.0) * piece_height;

            world.set_local_transform(
                entity,
                LocalTransform {
                    translation: nalgebra_glm::vec3(solved_x, 0.003, solved_z),
                    scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
                    rotation: Quat::identity(),
                },
            );
            world.set_local_transform_dirty(entity, LocalTransformDirty);
            world.set_global_transform(entity, GlobalTransform::default());
            world.set_visibility(entity, Visibility { visible: true });
            let obb = OrientedBoundingBox::new(
                nalgebra_glm::vec3(0.0, 0.0, 0.0),
                nalgebra_glm::vec3(piece_width / 2.0, 0.05, piece_height / 2.0),
                Quat::identity(),
            );
            let sphere_radius =
                (piece_width * piece_width / 4.0 + piece_height * piece_height / 4.0).sqrt();
            world.set_bounding_volume(entity, BoundingVolume::new(obb, sphere_radius));

            let material_name = format!("PuzzleMaterial_{}_{}", grid_x, grid_y);
            texture_cache_add_reference(&mut world.resources.texture_cache, texture_name);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [1.0, 1.0, 1.0, 1.0],
                    base_texture: Some(texture_name.to_string()),
                    unlit: true,
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(entity, MaterialRef::new(material_name));

            let group_entity = puzzle_world.spawn_entities(PIECE_GROUP | GROUP_MEMBERS, 1)[0];
            puzzle_world.set_piece_group(group_entity, PieceGroup);
            puzzle_world.set_group_members(
                group_entity,
                GroupMembers {
                    members: vec![],
                    world_x: solved_x,
                    world_z: solved_z,
                },
            );

            let piece_entity = puzzle_world.spawn_entities(
                ENGINE_ENTITY
                    | PUZZLE_PIECE
                    | IN_GROUP
                    | crate::ecs::LOCAL_OFFSET
                    | crate::ecs::Z_ORDER,
                1,
            )[0];

            puzzle_world.resources.z_counter += 1;
            let z_order = puzzle_world.resources.z_counter;

            puzzle_world.set_engine_entity(piece_entity, EngineEntity(entity));
            puzzle_world.set_puzzle_piece(
                piece_entity,
                PuzzlePiece {
                    grid_pos,
                    rotation: 0,
                    correct_rotation: 0,
                },
            );
            puzzle_world.set_in_group(piece_entity, InGroup(group_entity));
            puzzle_world.set_local_offset(piece_entity, LocalOffset { x: 0.0, y: 0.0 });
            puzzle_world.set_z_order(piece_entity, ZOrder(z_order));

            if let Some(group_members) = puzzle_world.get_group_members_mut(group_entity) {
                group_members.members.push(piece_entity);
            }
        }
    }
}

pub fn reset_puzzle(puzzle_world: &mut PuzzleWorld, world: &mut World) {
    let pieces: Vec<_> = puzzle_world
        .query_entities(ENGINE_ENTITY | PUZZLE_PIECE | IN_GROUP)
        .collect();

    let groups_to_remove: Vec<_> = puzzle_world
        .query_entities(PIECE_GROUP | GROUP_MEMBERS)
        .collect();

    for group_entity in &groups_to_remove {
        puzzle_world.despawn_entities(&[*group_entity]);
    }

    for piece_entity in &pieces {
        let group_entity = puzzle_world.spawn_entities(PIECE_GROUP | GROUP_MEMBERS, 1)[0];
        puzzle_world.set_piece_group(group_entity, PieceGroup);
        puzzle_world.set_group_members(
            group_entity,
            GroupMembers {
                members: vec![*piece_entity],
                world_x: 0.0,
                world_z: 0.0,
            },
        );
        puzzle_world.set_in_group(*piece_entity, InGroup(group_entity));
        puzzle_world.set_local_offset(*piece_entity, LocalOffset { x: 0.0, y: 0.0 });
    }

    puzzle_world.resources.puzzle_complete = false;

    shuffle_pieces(puzzle_world, world);
}

pub fn reslice_puzzle(puzzle_world: &mut PuzzleWorld, world: &mut World, texture_name: &str) {
    let pieces: Vec<_> = puzzle_world
        .query_entities(ENGINE_ENTITY | PUZZLE_PIECE)
        .collect();

    for piece_entity in &pieces {
        if let Some(engine_entity) = puzzle_world.get_engine_entity(*piece_entity) {
            world.despawn_entities(&[engine_entity.0]);
        }
    }

    let all_puzzle_entities: Vec<_> = puzzle_world
        .query_entities(PUZZLE_PIECE | ENGINE_ENTITY)
        .collect();
    puzzle_world.despawn_entities(&all_puzzle_entities);

    let groups: Vec<_> = puzzle_world
        .query_entities(PIECE_GROUP | GROUP_MEMBERS)
        .collect();
    puzzle_world.despawn_entities(&groups);

    if let Some(board_entity) = puzzle_world.resources.board_outline_entity {
        world.despawn_entities(&[board_entity]);
        puzzle_world.resources.board_outline_entity = None;
    }

    puzzle_world.resources.edge_types.clear();
    puzzle_world.resources.piece_outlines.clear();
    puzzle_world.resources.z_counter = 0;
    puzzle_world.resources.puzzle_complete = false;
    puzzle_world.resources.is_solving = false;
    puzzle_world.resources.solve_queue.clear();
    puzzle_world.resources.solve_progress = 0.0;

    initialize_edge_types(puzzle_world);
    spawn_board_outline(puzzle_world, world);
    spawn_puzzle_pieces(puzzle_world, world, texture_name);
    shuffle_pieces(puzzle_world, world);
}

pub fn shuffle_pieces(puzzle_world: &mut PuzzleWorld, world: &mut World) {
    let cols = puzzle_world.resources.grid_cols;
    let rows = puzzle_world.resources.grid_rows;
    let piece_width = puzzle_world.resources.piece_width;
    let piece_height = puzzle_world.resources.piece_height;

    let board_half_width = cols as f32 * piece_width / 2.0;
    let board_half_height = rows as f32 * piece_height / 2.0;

    let spacing = 1.15;
    let cell_width = piece_width * spacing;
    let cell_height = piece_height * spacing;

    let mut rng = rand::rng();

    let groups: Vec<_> = puzzle_world
        .query_entities(PIECE_GROUP | GROUP_MEMBERS)
        .collect();
    let total_pieces = groups.len();

    let margin = piece_width * 0.2;

    let mut positions: Vec<(f32, f32)> = Vec::with_capacity(total_pieces);

    let right_start_x = board_half_width + margin + cell_width / 2.0;
    let left_start_x = -board_half_width - margin - cell_width / 2.0;
    let top_start_z = board_half_height + margin + cell_height / 2.0;
    let bottom_start_z = -board_half_height - margin - cell_height / 2.0;

    for row in 0..rows {
        let x = right_start_x;
        let z = -board_half_height + row as f32 * cell_height + cell_height / 2.0;
        positions.push((x, z));
    }

    for row in 0..rows {
        let x = left_start_x;
        let z = -board_half_height + row as f32 * cell_height + cell_height / 2.0;
        positions.push((x, z));
    }

    for col in 0..cols {
        let x = -board_half_width + col as f32 * cell_width + cell_width / 2.0;
        let z = top_start_z;
        positions.push((x, z));
    }

    for col in 0..cols {
        let x = -board_half_width + col as f32 * cell_width + cell_width / 2.0;
        let z = bottom_start_z;
        positions.push((x, z));
    }

    while positions.len() < total_pieces {
        let x = right_start_x + cell_width;
        let z = positions.len() as f32 * cell_height - board_half_height;
        positions.push((x, z));
    }

    positions.truncate(total_pieces);

    use rand::seq::SliceRandom;
    positions.shuffle(&mut rng);

    for (i, group_entity) in groups.iter().enumerate() {
        let (pos_x, pos_z) = positions.get(i).copied().unwrap_or((right_start_x, 0.0));
        let random_rotation: u8 = rng.random_range(0..4);

        puzzle_world.resources.z_counter += 1;
        let z_order = puzzle_world.resources.z_counter;

        if let Some(group_members) = puzzle_world.get_group_members_mut(*group_entity) {
            group_members.world_x = pos_x;
            group_members.world_z = pos_z;

            for &piece_entity in &group_members.members.clone() {
                if let Some(puzzle_piece) = puzzle_world.get_puzzle_piece_mut(piece_entity) {
                    puzzle_piece.rotation = random_rotation;
                }

                puzzle_world.set_z_order(piece_entity, ZOrder(z_order));

                if let Some(engine_entity) = puzzle_world.get_engine_entity(piece_entity) {
                    let engine_ent = engine_entity.0;
                    if let Some(transform) = world.get_local_transform_mut(engine_ent) {
                        transform.translation.x = pos_x;
                        transform.translation.y = 0.003 + z_order as f32 * 0.0005;
                        transform.translation.z = pos_z;

                        let angle = random_rotation as f32 * std::f32::consts::FRAC_PI_2;
                        transform.rotation = nalgebra_glm::quat_angle_axis(
                            angle,
                            &nalgebra_glm::vec3(0.0, 1.0, 0.0),
                        );
                    }
                    world.set_local_transform_dirty(engine_ent, LocalTransformDirty);
                }
            }
        }
    }
}

pub fn load_puzzle_texture(world: &mut World, image_data: &[u8]) -> Option<(u32, u32)> {
    let image = image::load_from_memory(image_data).ok()?;
    let rgba_image = image.to_rgba8();
    let width = rgba_image.width();
    let height = rgba_image.height();
    let pixels = rgba_image.into_raw();

    world.queue_command(WorldCommand::LoadTexture {
        name: "puzzle_texture".to_string(),
        rgba_data: pixels,
        width,
        height,
    });

    Some((width, height))
}

pub fn spawn_board_outline(puzzle_world: &mut PuzzleWorld, world: &mut World) {
    let cols = puzzle_world.resources.grid_cols;
    let rows = puzzle_world.resources.grid_rows;
    let piece_width = puzzle_world.resources.piece_width;
    let piece_height = puzzle_world.resources.piece_height;
    let tab_params = TabParams {
        depth: puzzle_world.resources.tab_depth,
        width: puzzle_world.resources.tab_width,
        neck_width: puzzle_world.resources.neck_width,
    };

    let total_width = cols as f32 * piece_width;
    let total_height = rows as f32 * piece_height;
    let half_width = total_width / 2.0;
    let half_height = total_height / 2.0;

    let mut lines_data = Vec::new();
    let line_color = nalgebra_glm::vec4(0.6, 0.6, 0.6, 1.0);
    let board_y = 0.003;

    lines_data.push(Line {
        start: nalgebra_glm::vec3(-half_width, board_y, -half_height),
        end: nalgebra_glm::vec3(half_width, board_y, -half_height),
        color: line_color,
    });
    lines_data.push(Line {
        start: nalgebra_glm::vec3(half_width, board_y, -half_height),
        end: nalgebra_glm::vec3(half_width, board_y, half_height),
        color: line_color,
    });
    lines_data.push(Line {
        start: nalgebra_glm::vec3(half_width, board_y, half_height),
        end: nalgebra_glm::vec3(-half_width, board_y, half_height),
        color: line_color,
    });
    lines_data.push(Line {
        start: nalgebra_glm::vec3(-half_width, board_y, half_height),
        end: nalgebra_glm::vec3(-half_width, board_y, -half_height),
        color: line_color,
    });

    for grid_y in 0..rows {
        for grid_x in 1..cols {
            let edge_key = EdgeKey::new(grid_x as i32, grid_y as i32, Direction::Vertical);
            let edge_type = puzzle_world
                .resources
                .edge_types
                .get(&edge_key)
                .copied()
                .unwrap_or(EdgeType::Flat);

            let edge_points =
                crate::mesh::generate_edge_points_for_board(edge_type, 16, tab_params);

            let base_x = -half_width + grid_x as f32 * piece_width;
            let base_z = -half_height + grid_y as f32 * piece_height;

            for index in 0..edge_points.len().saturating_sub(1) {
                let start = &edge_points[index];
                let end = &edge_points[index + 1];

                lines_data.push(Line {
                    start: nalgebra_glm::vec3(
                        base_x + start.y * piece_width,
                        board_y,
                        base_z + start.x * piece_height,
                    ),
                    end: nalgebra_glm::vec3(
                        base_x + end.y * piece_width,
                        board_y,
                        base_z + end.x * piece_height,
                    ),
                    color: line_color,
                });
            }
        }
    }

    for grid_y in 1..rows {
        for grid_x in 0..cols {
            let edge_key = EdgeKey::new(grid_x as i32, grid_y as i32, Direction::Horizontal);
            let edge_type = puzzle_world
                .resources
                .edge_types
                .get(&edge_key)
                .copied()
                .unwrap_or(EdgeType::Flat);

            let edge_points =
                crate::mesh::generate_edge_points_for_board(edge_type, 16, tab_params);

            let base_x = -half_width + grid_x as f32 * piece_width;
            let base_z = -half_height + grid_y as f32 * piece_height;

            for index in 0..edge_points.len().saturating_sub(1) {
                let start = &edge_points[index];
                let end = &edge_points[index + 1];

                lines_data.push(Line {
                    start: nalgebra_glm::vec3(
                        base_x + start.x * piece_width,
                        board_y,
                        base_z + start.y * piece_height,
                    ),
                    end: nalgebra_glm::vec3(
                        base_x + end.x * piece_width,
                        board_y,
                        base_z + end.y * piece_height,
                    ),
                    color: line_color,
                });
            }
        }
    }

    for grid_y in 0..rows {
        for grid_x in 0..cols {
            let grid_pos = IVec2::new(grid_x as i32, grid_y as i32);
            let profile = get_edge_profile(puzzle_world, grid_x as i32, grid_y as i32);
            let outline =
                generate_piece_outline(&profile, piece_width, piece_height, 16, tab_params);
            puzzle_world
                .resources
                .piece_outlines
                .insert(grid_pos, outline);
        }
    }

    let entity = world.spawn_entities(
        LINES | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | VISIBILITY,
        1,
    )[0];

    let mut lines = Lines::new(lines_data);
    lines.mark_dirty();
    world.set_lines(entity, lines);

    world.set_local_transform(
        entity,
        LocalTransform {
            translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
            scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
            rotation: Quat::identity(),
        },
    );
    world.set_local_transform_dirty(entity, LocalTransformDirty);
    world.set_global_transform(entity, GlobalTransform::default());
    world.set_visibility(
        entity,
        Visibility {
            visible: puzzle_world.resources.show_board_outline,
        },
    );

    puzzle_world.resources.board_outline_entity = Some(entity);
}

pub fn toggle_board_outline(puzzle_world: &mut PuzzleWorld, world: &mut World) {
    puzzle_world.resources.show_board_outline = !puzzle_world.resources.show_board_outline;

    if let Some(entity) = puzzle_world.resources.board_outline_entity
        && let Some(visibility) = world.get_visibility_mut(entity)
    {
        visibility.visible = puzzle_world.resources.show_board_outline;
    }
}

pub fn start_solving(puzzle_world: &mut PuzzleWorld) {
    use rand::seq::SliceRandom;

    let cols = puzzle_world.resources.grid_cols;
    let rows = puzzle_world.resources.grid_rows;
    let piece_width = puzzle_world.resources.piece_width;
    let piece_height = puzzle_world.resources.piece_height;

    let mut unsolved_pieces = Vec::new();

    for piece_entity in puzzle_world.query_entities(ENGINE_ENTITY | PUZZLE_PIECE | IN_GROUP) {
        let puzzle_piece = match puzzle_world.get_puzzle_piece(piece_entity) {
            Some(p) => p,
            None => continue,
        };

        let group_entity = match puzzle_world.get_in_group(piece_entity) {
            Some(g) => g.0,
            None => continue,
        };

        let group_members = match puzzle_world.get_group_members(group_entity) {
            Some(m) => m,
            None => continue,
        };

        let solved_x = (puzzle_piece.grid_pos.x as f32 - (cols as f32 - 1.0) / 2.0) * piece_width;
        let solved_z = (puzzle_piece.grid_pos.y as f32 - (rows as f32 - 1.0) / 2.0) * piece_height;

        let current_x = group_members.world_x;
        let current_z = group_members.world_z;

        let error = (current_x - solved_x).abs() + (current_z - solved_z).abs();
        let rotation_wrong = puzzle_piece.rotation != puzzle_piece.correct_rotation;

        if error > 0.01 || rotation_wrong {
            unsolved_pieces.push(piece_entity);
        }
    }

    let mut rng = rand::rng();
    unsolved_pieces.shuffle(&mut rng);

    puzzle_world.resources.solve_queue = unsolved_pieces;
    puzzle_world.resources.solve_progress = 0.0;
    puzzle_world.resources.is_solving = !puzzle_world.resources.solve_queue.is_empty();
}

pub fn solve_system(puzzle_world: &mut PuzzleWorld, world: &mut World, delta_time: f32) {
    if !puzzle_world.resources.is_solving {
        return;
    }

    if puzzle_world.resources.solve_queue.is_empty() {
        puzzle_world.resources.is_solving = false;
        puzzle_world.resources.puzzle_complete = true;
        return;
    }

    let cols = puzzle_world.resources.grid_cols;
    let rows = puzzle_world.resources.grid_rows;
    let piece_width = puzzle_world.resources.piece_width;
    let piece_height = puzzle_world.resources.piece_height;

    let speed = 4.0;
    puzzle_world.resources.solve_progress += delta_time * speed;

    let piece_entity = puzzle_world.resources.solve_queue[0];

    let puzzle_piece = match puzzle_world.get_puzzle_piece(piece_entity) {
        Some(p) => *p,
        None => {
            puzzle_world.resources.solve_queue.remove(0);
            puzzle_world.resources.solve_progress = 0.0;
            return;
        }
    };

    let engine_entity = match puzzle_world.get_engine_entity(piece_entity) {
        Some(e) => e.0,
        None => {
            puzzle_world.resources.solve_queue.remove(0);
            puzzle_world.resources.solve_progress = 0.0;
            return;
        }
    };

    let group_entity = match puzzle_world.get_in_group(piece_entity) {
        Some(g) => g.0,
        None => {
            puzzle_world.resources.solve_queue.remove(0);
            puzzle_world.resources.solve_progress = 0.0;
            return;
        }
    };

    let group_members = match puzzle_world.get_group_members(group_entity) {
        Some(m) => m.clone(),
        None => {
            puzzle_world.resources.solve_queue.remove(0);
            puzzle_world.resources.solve_progress = 0.0;
            return;
        }
    };

    let solved_x = (puzzle_piece.grid_pos.x as f32 - (cols as f32 - 1.0) / 2.0) * piece_width;
    let solved_z = (puzzle_piece.grid_pos.y as f32 - (rows as f32 - 1.0) / 2.0) * piece_height;

    let start_x = group_members.world_x;
    let start_z = group_members.world_z;

    let t = puzzle_world.resources.solve_progress.min(1.0);
    let ease_t = t * t * (3.0 - 2.0 * t);

    let current_x = start_x + (solved_x - start_x) * ease_t;
    let current_z = start_z + (solved_z - start_z) * ease_t;

    let lift_height = 0.15 * (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0);

    if let Some(transform) = world.get_local_transform_mut(engine_entity) {
        transform.translation.x = current_x;
        transform.translation.y = 0.003 + lift_height;
        transform.translation.z = current_z;

        let target_rotation = puzzle_piece.correct_rotation;
        let current_rotation = puzzle_piece.rotation;
        if target_rotation != current_rotation {
            let target_angle = target_rotation as f32 * std::f32::consts::FRAC_PI_2;
            let current_angle = current_rotation as f32 * std::f32::consts::FRAC_PI_2;
            let lerp_angle = current_angle + (target_angle - current_angle) * ease_t;
            transform.rotation =
                nalgebra_glm::quat_angle_axis(lerp_angle, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
        }

        world.set_local_transform_dirty(engine_entity, LocalTransformDirty);
    }

    if t >= 1.0 {
        if let Some(gm) = puzzle_world.get_group_members_mut(group_entity) {
            gm.world_x = solved_x;
            gm.world_z = solved_z;
        }

        if let Some(pp) = puzzle_world.get_puzzle_piece_mut(piece_entity) {
            pp.rotation = pp.correct_rotation;
        }

        if let Some(transform) = world.get_local_transform_mut(engine_entity) {
            transform.translation.x = solved_x;
            transform.translation.y = 0.003;
            transform.translation.z = solved_z;
            let target_angle = puzzle_piece.correct_rotation as f32 * std::f32::consts::FRAC_PI_2;
            transform.rotation =
                nalgebra_glm::quat_angle_axis(target_angle, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
            world.set_local_transform_dirty(engine_entity, LocalTransformDirty);
        }

        puzzle_world.resources.solve_queue.remove(0);
        puzzle_world.resources.solve_progress = 0.0;
    }
}

pub fn start_victory_celebration(puzzle_world: &mut PuzzleWorld, world: &mut World) {
    use rand::seq::SliceRandom;

    let mut piece_entities: Vec<_> = puzzle_world
        .query_entities(ENGINE_ENTITY | PUZZLE_PIECE)
        .collect();

    let mut rng = rand::rng();
    piece_entities.shuffle(&mut rng);

    puzzle_world.resources.all_piece_entities = piece_entities;
    puzzle_world.resources.victory_active = true;
    puzzle_world.resources.victory_flash_index = 0;
    puzzle_world.resources.victory_flash_timer = 0.0;
    puzzle_world.resources.victory_time = 0.0;

    let text_index = world.resources.text_cache.add_text("Solved!");
    let text_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.get_name_mut(text_entity) {
        *name = Name("Victory Text".to_string());
    }

    if let Some(transform) = world.get_local_transform_mut(text_entity) {
        transform.translation = nalgebra_glm::vec3(0.0, 0.5, 0.0);
    }

    if let Some(text_component) = world.get_text_mut(text_entity) {
        text_component.text_index = text_index;
        text_component.properties = TextProperties {
            font_size: 128.0,
            color: nalgebra_glm::vec4(0.2, 1.0, 0.3, 1.0),
            alignment: TextAlignment::Center,
            outline_width: 0.1,
            outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
            smoothing: 0.1,
            ..Default::default()
        };
        text_component.dirty = true;
    }

    puzzle_world.resources.victory_text_entity = Some(text_entity);
    puzzle_world.resources.victory_text_lifetime = 0.0;
}

pub fn victory_system(puzzle_world: &mut PuzzleWorld, world: &mut World, delta_time: f32) {
    if !puzzle_world.resources.victory_active {
        return;
    }

    let total_pieces = puzzle_world.resources.all_piece_entities.len();

    puzzle_world.resources.victory_time += delta_time;
    let ripple_time = puzzle_world.resources.victory_time;

    let flash_duration = 0.06;
    puzzle_world.resources.victory_flash_timer += delta_time;

    if puzzle_world.resources.victory_flash_timer >= flash_duration {
        puzzle_world.resources.victory_flash_timer = 0.0;
        puzzle_world.resources.victory_flash_index += 1;
    }

    let current_index = puzzle_world.resources.victory_flash_index;

    for i in 0..total_pieces {
        let piece_entity = puzzle_world.resources.all_piece_entities[i];
        if let Some(engine_entity) = puzzle_world.get_engine_entity(piece_entity)
            && let Some(transform) = world.get_local_transform_mut(engine_entity.0)
        {
            let phase = i as f32 * 0.15;
            let wave_pos = ripple_time * 2.0 - phase;
            let wave = if wave_pos > 0.0 && wave_pos < std::f32::consts::PI {
                wave_pos.sin() * 0.15
            } else {
                0.0
            };
            transform.translation.y = 0.003 + wave;
            world.set_local_transform_dirty(engine_entity.0, LocalTransformDirty);
        }
    }

    if current_index < total_pieces {
        let piece_entity = puzzle_world.resources.all_piece_entities[current_index];
        if let Some(engine_entity) = puzzle_world.get_engine_entity(piece_entity) {
            world.resources.graphics.selection_outline_enabled = true;
            world.resources.graphics.bounding_volume_selected_entity = Some(engine_entity.0);
            world.resources.graphics.selection_outline_color = [0.0, 1.0, 0.2, 1.0];
        }
    } else {
        world.resources.graphics.selection_outline_enabled = false;
        world.resources.graphics.bounding_volume_selected_entity = None;
        world.resources.graphics.selection_outline_color = [1.0, 0.45, 0.0, 1.0];
    }

    puzzle_world.resources.victory_text_lifetime += delta_time;
    let lifetime = puzzle_world.resources.victory_text_lifetime;

    if let Some(text_entity) = puzzle_world.resources.victory_text_entity {
        if lifetime < 4.0 {
            if let Some(transform) = world.get_local_transform_mut(text_entity) {
                transform.translation.y += delta_time * 0.8;
                world.set_local_transform_dirty(text_entity, LocalTransformDirty);
            }

            if let Some(text_component) = world.get_text_mut(text_entity) {
                let alpha = if lifetime > 3.0 {
                    1.0 - (lifetime - 3.0)
                } else {
                    1.0
                };
                text_component.properties.color.w = alpha;
                text_component.dirty = true;
            }
        } else {
            world.despawn_entities(&[text_entity]);
            puzzle_world.resources.victory_text_entity = None;

            for i in 0..total_pieces {
                let piece_entity = puzzle_world.resources.all_piece_entities[i];
                if let Some(engine_entity) = puzzle_world.get_engine_entity(piece_entity)
                    && let Some(transform) = world.get_local_transform_mut(engine_entity.0)
                {
                    transform.translation.y = 0.003;
                    world.set_local_transform_dirty(engine_entity.0, LocalTransformDirty);
                }
            }

            puzzle_world.resources.victory_active = false;
        }
    }
}
