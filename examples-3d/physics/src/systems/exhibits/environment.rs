use nightshade::ecs::light::components::{Light, LightType};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::physics::*;
use nightshade::ecs::world::{
    BOUNDING_VOLUME, CASTS_SHADOW, GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM,
    LOCAL_TRANSFORM_DIRTY, MATERIAL_REF, NAME, RENDER_MESH, VISIBILITY,
};
use nightshade::prelude::*;

pub(super) struct RoomConfig {
    pub center: Vec3,
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub wall_thickness: f32,
    pub doorway_width: f32,
    pub doorway_height: f32,
    pub wall_material: nightshade::ecs::material::components::Material,
    pub ceiling_material: nightshade::ecs::material::components::Material,
}

pub fn spawn_sun_overhead(world: &mut World) -> Entity {
    use nightshade::ecs::world::components;
    use nightshade::ecs::world::{
        GLOBAL_TRANSFORM, LIGHT, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, NAME,
    };

    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];

    world
        .core
        .set_name(entity, components::Name("Sun".to_string()));
    world.core.set_local_transform(
        entity,
        components::LocalTransform {
            translation: nalgebra_glm::Vec3::new(5.0, 10.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                std::f32::consts::FRAC_PI_4,
                &nalgebra_glm::Vec3::new(0.0, 1.0, 0.0),
            ) * nalgebra_glm::quat_angle_axis(
                -std::f32::consts::FRAC_PI_4,
                &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0),
            ),
            scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, components::LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, components::GlobalTransform::default());
    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Directional,
            color: nalgebra_glm::vec3(1.0, 0.95, 0.8),
            intensity: 5.0,
            range: 100.0,
            inner_cone_angle: std::f32::consts::PI / 6.0,
            outer_cone_angle: std::f32::consts::PI / 4.0,
            cast_shadows: true,
            shadow_bias: 0.0005,
        },
    );

    entity
}

pub fn spawn_environment(world: &mut World) {
    let floor_material =
        create_textured_material(nalgebra_glm::vec3(0.15, 0.15, 0.18), 0.9, 0.0);
    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(0.0, -0.25, 0.0),
        nalgebra_glm::vec3(30.0, 0.5, 30.0),
        floor_material,
    );

    let wall_material =
        create_textured_material(nalgebra_glm::vec3(0.2, 0.18, 0.16), 0.95, 0.0);

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(0.0, 2.0, -15.0),
        nalgebra_glm::vec3(30.0, 4.0, 0.5),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(-15.0, 2.0, 0.0),
        nalgebra_glm::vec3(0.5, 4.0, 30.0),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(15.0, 2.0, 0.0),
        nalgebra_glm::vec3(0.5, 4.0, 30.0),
        wall_material,
    );
}

pub(super) fn spawn_visual_cube(
    world: &mut World,
    position: Vec3,
    scale: Vec3,
    material: nightshade::ecs::material::components::Material,
    name: String,
) {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW
            | VISIBILITY,
        1,
    )[0];

    if let Some(n) = world.core.get_name_mut(entity) {
        n.0 = name;
    }

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.core.get_render_mesh_mut(entity) {
        mesh.name = "Cube".to_string();
    }

    let material_name = format!("VisualCube_{}", entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        material,
    );
    if let Some(&mat_index) = world
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
            .add_reference(mat_index);
    }
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(bv) = world.core.get_bounding_volume_mut(entity) {
        *bv = nightshade::ecs::world::components::BoundingVolume::from_mesh_type("Cube");
    }
}

pub(super) fn spawn_room_walls(world: &mut World, config: &RoomConfig) {
    let center = config.center;
    let room_width = config.width;
    let room_depth = config.depth;
    let room_height = config.height;
    let wall_thickness = config.wall_thickness;
    let doorway_width = config.doorway_width;
    let doorway_height = config.doorway_height;
    let wall_material = config.wall_material.clone();
    let ceiling_material = config.ceiling_material.clone();

    let half_width = room_width / 2.0;
    let half_depth = room_depth / 2.0;
    let wall_center_y = room_height / 2.0;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, wall_center_y, center.z + half_depth - wall_thickness / 2.0),
        nalgebra_glm::vec3(room_width, room_height, wall_thickness),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x - half_width + wall_thickness / 2.0, wall_center_y, center.z),
        nalgebra_glm::vec3(wall_thickness, room_height, room_depth),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x + half_width - wall_thickness / 2.0, wall_center_y, center.z),
        nalgebra_glm::vec3(wall_thickness, room_height, room_depth),
        wall_material.clone(),
    );

    let front_z = center.z - half_depth + wall_thickness / 2.0;
    let segment_width = (room_width - doorway_width) / 2.0;

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x - half_width + segment_width / 2.0, wall_center_y, front_z),
        nalgebra_glm::vec3(segment_width, room_height, wall_thickness),
        wall_material.clone(),
    );

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x + half_width - segment_width / 2.0, wall_center_y, front_z),
        nalgebra_glm::vec3(segment_width, room_height, wall_thickness),
        wall_material.clone(),
    );

    let header_height = room_height - doorway_height;
    if header_height > 0.01 {
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(center.x, doorway_height + header_height / 2.0, front_z),
            nalgebra_glm::vec3(doorway_width, header_height, wall_thickness),
            wall_material,
        );
    }

    spawn_static_physics_cube_with_material(
        world,
        nalgebra_glm::vec3(center.x, room_height + wall_thickness / 2.0, center.z),
        nalgebra_glm::vec3(room_width, wall_thickness, room_depth),
        ceiling_material,
    );
}

pub(super) fn spawn_room_light(world: &mut World, position: Vec3, color: Vec3, intensity: f32) {
    let light_entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | LIGHT,
        1,
    )[0];

    if let Some(name) = world.core.get_name_mut(light_entity) {
        name.0 = "Room Light".to_string();
    }

    if let Some(transform) = world.core.get_local_transform_mut(light_entity) {
        transform.translation = position;
    }

    if let Some(light) = world.core.get_light_mut(light_entity) {
        *light = Light {
            light_type: LightType::Point,
            color,
            intensity,
            range: 8.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: true,
            shadow_bias: 0.005,
        };
    }
}
