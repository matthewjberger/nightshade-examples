use crate::ecs::GameWorld;
use nightshade::ecs::physics::*;
use nightshade::prelude::*;

pub(crate) fn spawn_grabbables_exhibit(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
) {
    let pedestal_material =
        create_textured_material(nalgebra_glm::vec3(0.25, 0.25, 0.28), 0.85, 0.0);

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, 0.4, center.z),
        nalgebra_glm::vec3(2.5, 0.8, 2.5),
        pedestal_material,
    );

    let table_top_y = 0.8;
    let box_size = 0.25;
    let box_material = create_textured_material(nalgebra_glm::vec3(0.6, 0.5, 0.35), 0.7, 0.0);

    let positions = [
        nalgebra_glm::vec3(center.x - 0.5, table_top_y + box_size / 2.0, center.z - 0.5),
        nalgebra_glm::vec3(center.x + 0.5, table_top_y + box_size / 2.0, center.z - 0.5),
        nalgebra_glm::vec3(center.x, table_top_y + box_size / 2.0, center.z + 0.5),
    ];

    for position in positions {
        let entity = spawn_dynamic_physics_cube_with_material(
            world,
            position,
            nalgebra_glm::vec3(box_size, box_size, box_size),
            2.0,
            box_material.clone(),
        );
        crate::systems::spawn::register_grabbable(game_world, entity);
    }

    let sphere_radius = 0.2;
    let sphere_material = create_textured_material(nalgebra_glm::vec3(0.7, 0.2, 0.2), 0.5, 0.3);
    let sphere_entity = spawn_dynamic_physics_sphere_with_material(
        world,
        nalgebra_glm::vec3(center.x, table_top_y + sphere_radius, center.z),
        sphere_radius,
        1.5,
        sphere_material,
    );
    crate::systems::spawn::register_grabbable(game_world, sphere_entity);

    let cylinder_half_height = 0.2;
    let cylinder_radius = 0.12;
    let metal_material = create_textured_material(nalgebra_glm::vec3(0.5, 0.5, 0.55), 0.3, 0.8);
    let cylinder_entity = spawn_dynamic_physics_cylinder_with_material(
        world,
        nalgebra_glm::vec3(center.x - 0.7, table_top_y + cylinder_half_height, center.z),
        cylinder_half_height,
        cylinder_radius,
        3.0,
        metal_material,
    );
    crate::systems::spawn::register_grabbable(game_world, cylinder_entity);
}
