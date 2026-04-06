use crate::ecs::GameWorld;
use crate::systems::ui::spawn_wall_label;
use super::environment::{spawn_room_light, spawn_room_walls, RoomConfig};
use nightshade::ecs::physics::*;
use nightshade::ecs::text::components::{TextAlignment, TextProperties, VerticalAlignment};
use nightshade::prelude::*;

pub(super) fn spawn_curiosity_room(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let room_height = 3.0;

    let config = RoomConfig {
        center,
        width: 4.0,
        depth: 4.0,
        height: room_height,
        wall_thickness: 0.15,
        doorway_width: 1.2,
        doorway_height: 2.3,
        wall_material: create_textured_material(nalgebra_glm::vec3(0.28, 0.22, 0.18), 0.92, 0.0),
        ceiling_material: create_textured_material(nalgebra_glm::vec3(0.3, 0.28, 0.25), 0.95, 0.0),
    };

    spawn_room_walls(world, &config);

    let front_z = center.z - config.depth / 2.0;
    spawn_wall_label(
        world,
        "Curiosity Cabinet",
        nalgebra_glm::vec3(center.x, config.doorway_height + 0.25, front_z - 0.3),
        TextProperties {
            font_size: 20.0,
            color: Vec4::new(1.0, 0.9, 0.7, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.04,
            outline_color: Vec4::new(0.15, 0.1, 0.05, 1.0),
            ..Default::default()
        },
    );

    let back_wall_z = center.z + config.depth / 2.0 - config.wall_thickness - 0.05;
    spawn_wall_label(
        world,
        "Take only what you need",
        nalgebra_glm::vec3(center.x, 2.0, back_wall_z - 0.05),
        TextProperties {
            font_size: 12.0,
            color: Vec4::new(0.8, 0.75, 0.6, 0.9),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.03,
            outline_color: Vec4::new(0.1, 0.08, 0.05, 0.8),
            ..Default::default()
        },
    );

    spawn_room_light(
        world,
        nalgebra_glm::vec3(center.x, room_height - 0.3, center.z),
        nalgebra_glm::vec3(1.0, 0.9, 0.7),
        8.0,
    );

    let shelf_material = create_textured_material(nalgebra_glm::vec3(0.4, 0.3, 0.2), 0.75, 0.1);
    let shelf_y = 0.9;
    let shelf_z = center.z + 1.2;
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, shelf_y, shelf_z),
        nalgebra_glm::vec3(2.0, 0.06, 0.5),
        shelf_material,
    );

    let gold_material = create_textured_material(nalgebra_glm::vec3(0.85, 0.7, 0.2), 0.3, 0.9);
    let gem_positions = [
        nalgebra_glm::vec3(center.x - 0.4, shelf_y + 0.03 + 0.1, shelf_z),
        nalgebra_glm::vec3(center.x + 0.5, shelf_y + 0.03 + 0.1, shelf_z - 0.1),
    ];
    for (index, position) in gem_positions.iter().enumerate() {
        let entity = spawn_dynamic_physics_sphere_with_material(
            world,
            *position,
            0.1,
            0.5,
            gold_material.clone(),
        );
        world
            .core
            .set_name(entity, Name(format!("Gold Gem {}", index + 1)));
        game_world.add_grabbable(entity);
    }

    let crystal_material =
        create_textured_material(nalgebra_glm::vec3(0.2, 0.4, 0.9), 0.15, 0.7);
    let crystal_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(center.x + 0.1, shelf_y + 0.03 + 0.12, shelf_z + 0.1),
        0.12,
        0.8,
        crystal_material,
    );
    world
        .core
        .set_name(crystal_entity, Name("Crystal Orb".to_string()));
    game_world.add_grabbable(crystal_entity);

    let trinket_material =
        create_textured_material(nalgebra_glm::vec3(0.6, 0.5, 0.35), 0.7, 0.0);
    let trinket_size = 0.12;
    let trinket_positions = [
        nalgebra_glm::vec3(center.x - 0.7, trinket_size / 2.0, center.z - 0.5),
        nalgebra_glm::vec3(center.x + 0.8, trinket_size / 2.0, center.z + 0.3),
    ];
    for (index, position) in trinket_positions.iter().enumerate() {
        let entity = spawn_dynamic_physics_cube_with_material(
            world,
            *position,
            nalgebra_glm::vec3(trinket_size, trinket_size, trinket_size),
            0.8,
            trinket_material.clone(),
        );
        world
            .core
            .set_name(entity, Name(format!("Trinket Box {}", index + 1)));
        game_world.add_grabbable(entity);
    }

    let vase_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.3, 0.25), 0.5, 0.2);
    let vase_entity = spawn_dynamic_physics_cylinder_with_material(
        world,
        nalgebra_glm::vec3(center.x - 0.6, 0.2, center.z + 0.8),
        0.2,
        0.08,
        1.0,
        vase_material,
    );
    world
        .core
        .set_name(vase_entity, Name("Ceramic Vase".to_string()));
    game_world.add_grabbable(vase_entity);

    let emerald_material =
        create_textured_material(nalgebra_glm::vec3(0.1, 0.7, 0.3), 0.2, 0.6);
    let emerald_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(center.x + 0.3, 0.08, center.z - 0.8),
        0.08,
        0.3,
        emerald_material,
    );
    world
        .core
        .set_name(emerald_entity, Name("Emerald".to_string()));
    game_world.add_grabbable(emerald_entity);
}

pub(super) fn spawn_workshop_room(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let room_height = 3.0;

    let config = RoomConfig {
        center,
        width: 4.0,
        depth: 4.0,
        height: room_height,
        wall_thickness: 0.15,
        doorway_width: 1.2,
        doorway_height: 2.3,
        wall_material: create_textured_material(nalgebra_glm::vec3(0.22, 0.22, 0.24), 0.9, 0.05),
        ceiling_material: create_textured_material(nalgebra_glm::vec3(0.25, 0.25, 0.27), 0.95, 0.0),
    };

    spawn_room_walls(world, &config);

    let front_z = center.z - config.depth / 2.0;
    spawn_wall_label(
        world,
        "Workshop",
        nalgebra_glm::vec3(center.x, config.doorway_height + 0.25, front_z - 0.3),
        TextProperties {
            font_size: 20.0,
            color: Vec4::new(0.9, 0.95, 1.0, 1.0),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.04,
            outline_color: Vec4::new(0.08, 0.08, 0.12, 1.0),
            ..Default::default()
        },
    );

    let back_wall_z = center.z + config.depth / 2.0 - config.wall_thickness - 0.05;
    spawn_wall_label(
        world,
        "Mind the sharp edges",
        nalgebra_glm::vec3(center.x, 2.0, back_wall_z - 0.05),
        TextProperties {
            font_size: 12.0,
            color: Vec4::new(0.85, 0.85, 0.9, 0.9),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 0.03,
            outline_color: Vec4::new(0.08, 0.08, 0.1, 0.8),
            ..Default::default()
        },
    );

    spawn_room_light(
        world,
        nalgebra_glm::vec3(center.x, room_height - 0.3, center.z),
        nalgebra_glm::vec3(0.9, 0.95, 1.0),
        10.0,
    );

    let bench_material =
        create_textured_material(nalgebra_glm::vec3(0.35, 0.28, 0.18), 0.8, 0.05);
    let bench_y = 0.8;
    let bench_z = center.z + 1.0;
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, bench_y / 2.0, bench_z),
        nalgebra_glm::vec3(2.4, bench_y, 0.7),
        bench_material,
    );

    let metal_material =
        create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.85);
    let tool_configs = [
        (
            nalgebra_glm::vec3(center.x - 0.5, bench_y + 0.1, bench_z),
            0.1,
            0.05,
        ),
        (
            nalgebra_glm::vec3(center.x + 0.3, bench_y + 0.08, bench_z + 0.1),
            0.08,
            0.04,
        ),
        (
            nalgebra_glm::vec3(center.x + 0.7, bench_y + 0.12, bench_z - 0.1),
            0.12,
            0.035,
        ),
    ];
    for (index, (position, half_height, radius)) in tool_configs.iter().enumerate() {
        let entity = spawn_dynamic_physics_cylinder_with_material(
            world,
            *position,
            *half_height,
            *radius,
            2.0,
            metal_material.clone(),
        );
        world
            .core
            .set_name(entity, Name(format!("Metal Part {}", index + 1)));
        game_world.add_grabbable(entity);
    }

    let brick_material =
        create_textured_material(nalgebra_glm::vec3(0.65, 0.25, 0.2), 0.85, 0.0);
    let brick_size = 0.15;
    let brick_positions = [
        nalgebra_glm::vec3(center.x - 0.7, brick_size / 2.0, center.z - 0.5),
        nalgebra_glm::vec3(center.x - 0.5, brick_size / 2.0, center.z - 0.7),
    ];
    for (index, position) in brick_positions.iter().enumerate() {
        let entity = spawn_dynamic_physics_cube_with_material(
            world,
            *position,
            nalgebra_glm::vec3(brick_size, brick_size, brick_size),
            3.0,
            brick_material.clone(),
        );
        world
            .core
            .set_name(entity, Name(format!("Brick {}", index + 1)));
        game_world.add_grabbable(entity);
    }

    let orb_material = create_textured_material(nalgebra_glm::vec3(0.2, 0.7, 0.3), 0.4, 0.5);
    let orb_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(center.x + 0.6, 0.12, center.z - 0.6),
        0.12,
        1.0,
        orb_material,
    );
    world
        .core
        .set_name(orb_entity, Name("Green Orb".to_string()));
    game_world.add_grabbable(orb_entity);

    let gear_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.55, 0.5), 0.25, 0.9);
    let gear_entity = spawn_dynamic_physics_cylinder_with_material(
        world,
        nalgebra_glm::vec3(center.x + 0.2, bench_y + 0.06, bench_z - 0.2),
        0.03,
        0.1,
        1.5,
        gear_material,
    );
    world
        .core
        .set_name(gear_entity, Name("Brass Gear".to_string()));
    game_world.add_grabbable(gear_entity);

    let bolt_material = create_textured_material(nalgebra_glm::vec3(0.4, 0.4, 0.45), 0.2, 0.9);
    let bolt_positions = [
        nalgebra_glm::vec3(center.x - 0.2, bench_y + 0.05, bench_z + 0.2),
        nalgebra_glm::vec3(center.x + 0.6, bench_y + 0.05, bench_z + 0.15),
    ];
    for (index, position) in bolt_positions.iter().enumerate() {
        let entity = spawn_dynamic_physics_cube_with_material(
            world,
            *position,
            nalgebra_glm::vec3(0.06, 0.06, 0.06),
            0.5,
            bolt_material.clone(),
        );
        world
            .core
            .set_name(entity, Name(format!("Bolt {}", index + 1)));
        game_world.add_grabbable(entity);
    }
}
