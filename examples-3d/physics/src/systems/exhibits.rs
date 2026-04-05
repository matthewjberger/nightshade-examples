mod environment;
mod grabbables;
mod joints;
mod rooms;

use crate::ecs::GameWorld;
use nightshade::prelude::*;

pub use environment::{spawn_environment, spawn_sun_overhead};
pub use joints::{
    setup_velocity_friction_joints, update_coulomb_friction_joints, update_joint_visuals,
    update_prismatic_sliders,
};

pub fn spawn_exhibits(game_world: &mut GameWorld, world: &mut World) {
    grabbables::spawn_grabbables_exhibit(game_world, world, nalgebra_glm::vec3(-10.0, 0.0, -10.0));
    grabbables::spawn_door_exhibit(game_world, world, nalgebra_glm::vec3(-4.0, 0.0, -12.0));
    grabbables::spawn_drawer_exhibit(game_world, world, nalgebra_glm::vec3(4.0, 0.0, -12.0));
    grabbables::spawn_lever_exhibit(game_world, world, nalgebra_glm::vec3(10.0, 0.0, -10.0));
    grabbables::spawn_wheel_exhibit(game_world, world, nalgebra_glm::vec3(-10.0, 0.0, -4.0));
    grabbables::spawn_chain_exhibit(game_world, world, nalgebra_glm::vec3(0.0, 0.0, -12.0));
    grabbables::spawn_bauble_table(game_world, world, nalgebra_glm::vec3(10.0, 0.0, -4.0));
    grabbables::spawn_note_table(game_world, world, nalgebra_glm::vec3(0.0, 0.0, 4.0));

    joints::spawn_fixed_joint_exhibit(game_world, world, nalgebra_glm::vec3(-10.0, 0.0, 4.0));
    joints::spawn_spherical_joint_exhibit(game_world, world, nalgebra_glm::vec3(-4.0, 0.0, 4.0));
    joints::spawn_rope_joint_exhibit(game_world, world, nalgebra_glm::vec3(4.0, 0.0, 4.0));
    joints::spawn_spring_joint_exhibit(game_world, world, nalgebra_glm::vec3(10.0, 0.0, 4.0));
    joints::spawn_prismatic_joint_exhibit(game_world, world, nalgebra_glm::vec3(-6.0, 0.0, 8.0));
    joints::spawn_revolute_joint_exhibit(game_world, world, nalgebra_glm::vec3(6.0, 0.0, 8.0));
    joints::spawn_velocity_friction_joint_exhibit(
        game_world,
        world,
        nalgebra_glm::vec3(0.0, 0.0, 10.0),
    );
    joints::spawn_coulomb_friction_joint_exhibit(
        game_world,
        world,
        nalgebra_glm::vec3(-6.0, 0.0, 12.0),
    );

    rooms::spawn_curiosity_room(game_world, world, nalgebra_glm::vec3(12.0, 0.0, 11.0));
    rooms::spawn_workshop_room(game_world, world, nalgebra_glm::vec3(-12.0, 0.0, 11.0));
}
