use crate::constants::ROOM_HEIGHT;
use nightshade::ecs::graphics::resources::Atmosphere;
use nightshade::ecs::scene::{
    AssetUuid, Scene, SceneBodyType, SceneCollider, SceneEntity, SceneLight, SceneMaterial,
    SceneMesh, ScenePhysics,
};
use nightshade::ecs::transform::LocalTransform;
use nightshade::prelude::{Vec3, nalgebra_glm};

const WALL_THICKNESS: f32 = 0.2;

fn floor_material() -> SceneMaterial {
    SceneMaterial {
        base_color: [0.3, 0.3, 0.3, 1.0],
        base_texture_name: Some("horror_floor".to_string()),
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    }
}

fn wall_material() -> SceneMaterial {
    SceneMaterial {
        base_color: [0.4, 0.4, 0.4, 1.0],
        base_texture_name: Some("horror_wall".to_string()),
        roughness: 0.9,
        metallic: 0.0,
        ..Default::default()
    }
}

fn ceiling_material() -> SceneMaterial {
    SceneMaterial {
        base_color: [0.2, 0.2, 0.2, 1.0],
        base_texture_name: Some("horror_ceiling".to_string()),
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    }
}

fn create_material(color: [f32; 3], roughness: f32, metallic: f32) -> SceneMaterial {
    SceneMaterial {
        base_color: [color[0], color[1], color[2], 1.0],
        roughness,
        metallic,
        ..Default::default()
    }
}

fn create_static_cube_entity(
    name: &str,
    position: Vec3,
    size: Vec3,
    material: SceneMaterial,
    parent: Option<AssetUuid>,
) -> SceneEntity {
    let mut entity = SceneEntity::new()
        .with_name(name)
        .with_transform(LocalTransform {
            translation: position,
            scale: size,
            ..Default::default()
        })
        .with_mesh(SceneMesh::from_name("Cube").with_material(material))
        .with_casts_shadow(true)
        .with_visible(true);
    entity.components.physics = Some(ScenePhysics {
        collider: SceneCollider::Cuboid {
            half_extents: [size.x / 2.0, size.y / 2.0, size.z / 2.0],
        },
        ..Default::default()
    });
    if let Some(p) = parent {
        entity = entity.with_parent(p);
    }
    entity
}

fn create_wall_entity(
    name: &str,
    position: Vec3,
    size: Vec3,
    material: SceneMaterial,
    parent: Option<AssetUuid>,
) -> SceneEntity {
    let mut mat = material;
    if size.x > size.z {
        mat.uv_scale = [size.x, size.y];
    } else {
        mat.uv_scale = [size.z, size.y];
    }
    create_static_cube_entity(name, position, size, mat, parent)
}

fn create_empty_entity(name: &str, parent: Option<AssetUuid>) -> SceneEntity {
    let mut entity = SceneEntity::new().with_name(name).with_visible(true);
    if let Some(p) = parent {
        entity = entity.with_parent(p);
    }
    entity
}

fn create_light_entity(
    name: &str,
    position: Vec3,
    light: SceneLight,
    parent: Option<AssetUuid>,
) -> SceneEntity {
    let mut entity = SceneEntity::new()
        .with_name(name)
        .with_transform(LocalTransform {
            translation: position,
            ..Default::default()
        })
        .with_light(light)
        .with_visible(true);
    if let Some(p) = parent {
        entity = entity.with_parent(p);
    }
    entity
}

fn add_floor_ceiling(
    scene: &mut Scene,
    parent: Option<AssetUuid>,
    name_prefix: &str,
    center: Vec3,
    width: f32,
    depth: f32,
) {
    let mut floor_mat = floor_material();
    floor_mat.uv_scale = [width, depth];
    let floor_pos = nalgebra_glm::vec3(center.x, -WALL_THICKNESS / 2.0, center.z);
    let floor_size = nalgebra_glm::vec3(width, WALL_THICKNESS, depth);
    scene.add_entity(create_static_cube_entity(
        &format!("{}_Floor", name_prefix),
        floor_pos,
        floor_size,
        floor_mat,
        parent,
    ));

    let mut ceiling_mat = ceiling_material();
    ceiling_mat.uv_scale = [width, depth];
    let ceiling_pos = nalgebra_glm::vec3(center.x, ROOM_HEIGHT + WALL_THICKNESS / 2.0, center.z);
    let ceiling_size = nalgebra_glm::vec3(width, WALL_THICKNESS, depth);
    scene.add_entity(create_static_cube_entity(
        &format!("{}_Ceiling", name_prefix),
        ceiling_pos,
        ceiling_size,
        ceiling_mat,
        parent,
    ));
}

fn add_entry_room(scene: &mut Scene, parent: AssetUuid) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let door_width = 1.2;
    let door_height = 2.2;
    let corridor_width = 4.0;
    let header_height = ROOM_HEIGHT - door_height;
    let header_center_y = door_height + header_height / 2.0;
    let door_frame_height_center = door_height / 2.0;
    let room_half = 4.0;

    scene.add_entity(create_wall_entity(
        "Entry_Wall_Left",
        nalgebra_glm::vec3(-room_half - t / 2.0, h, 4.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 8.0),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Entry_Wall_Right",
        nalgebra_glm::vec3(room_half + t / 2.0, h, 4.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 8.0),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Entry_Wall_Back",
        nalgebra_glm::vec3(0.0, h, 8.0 + t / 2.0),
        nalgebra_glm::vec3(8.0, ROOM_HEIGHT, t),
        wall_mat.clone(),
        Some(parent),
    ));

    let side_wall_width = room_half - corridor_width / 2.0;
    let left_center_x = -room_half + side_wall_width / 2.0;
    let right_center_x = room_half - side_wall_width / 2.0;

    scene.add_entity(create_wall_entity(
        "Entry_Door_Left_Side",
        nalgebra_glm::vec3(left_center_x, h, t / 2.0),
        nalgebra_glm::vec3(side_wall_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Entry_Door_Right_Side",
        nalgebra_glm::vec3(right_center_x, h, t / 2.0),
        nalgebra_glm::vec3(side_wall_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Entry_Door_Header",
        nalgebra_glm::vec3(0.0, header_center_y, t / 2.0),
        nalgebra_glm::vec3(corridor_width, header_height, t),
        wall_mat.clone(),
        Some(parent),
    ));

    let door_frame_width = (corridor_width - door_width) / 2.0;
    let left_frame_x = -door_width / 2.0 - door_frame_width / 2.0;
    let right_frame_x = door_width / 2.0 + door_frame_width / 2.0;

    scene.add_entity(create_wall_entity(
        "Entry_Door_Frame_Left",
        nalgebra_glm::vec3(left_frame_x, door_frame_height_center, t / 2.0),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Entry_Door_Frame_Right",
        nalgebra_glm::vec3(right_frame_x, door_frame_height_center, t / 2.0),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat,
        Some(parent),
    ));

    let desk_material = create_material([0.28, 0.2, 0.12], 0.85, 0.1);
    let chair_material = create_material([0.25, 0.18, 0.1], 0.9, 0.05);

    scene.add_entity(create_static_cube_entity(
        "Entry_Desk",
        nalgebra_glm::vec3(2.5, 0.4, 5.5),
        nalgebra_glm::vec3(1.6, 0.8, 0.8),
        desk_material,
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Entry_Chair",
        nalgebra_glm::vec3(2.5, 0.3, 4.5),
        nalgebra_glm::vec3(0.45, 0.6, 0.45),
        chair_material,
        Some(parent),
    ));
}

fn add_corridor(scene: &mut Scene, parent: AssetUuid) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let corridor_width = 4.0;
    let corridor_half_width = corridor_width / 2.0;

    scene.add_entity(create_wall_entity(
        "Corridor_Wall_Left",
        nalgebra_glm::vec3(-corridor_half_width - t / 2.0, h, -5.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 10.0),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Corridor_Wall_Right",
        nalgebra_glm::vec3(corridor_half_width + t / 2.0, h, -5.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 10.0),
        wall_mat,
        Some(parent),
    ));
}

fn add_main_hall(scene: &mut Scene, parent: AssetUuid) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let door_width = 1.2;
    let door_height = 2.2;
    let corridor_width = 4.0;
    let header_height = ROOM_HEIGHT - door_height;
    let header_center_y = door_height + header_height / 2.0;
    let door_frame_height_center = door_height / 2.0;
    let door_frame_width = (corridor_width - door_width) / 2.0;
    let room_half = 6.0;
    let room_center_z = -16.0;

    let north_z = room_center_z + room_half + t / 2.0;
    let south_z = room_center_z - room_half - t / 2.0;
    let side_section_width = (room_half * 2.0 - corridor_width) / 2.0;

    scene.add_entity(create_wall_entity(
        "MainHall_North_Left",
        nalgebra_glm::vec3(-room_half + side_section_width / 2.0, h, north_z),
        nalgebra_glm::vec3(side_section_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_North_Right",
        nalgebra_glm::vec3(room_half - side_section_width / 2.0, h, north_z),
        nalgebra_glm::vec3(side_section_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_North_Header",
        nalgebra_glm::vec3(0.0, header_center_y, north_z),
        nalgebra_glm::vec3(corridor_width, header_height, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_North_Frame_Left",
        nalgebra_glm::vec3(
            -door_width / 2.0 - door_frame_width / 2.0,
            door_frame_height_center,
            north_z,
        ),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_North_Frame_Right",
        nalgebra_glm::vec3(
            door_width / 2.0 + door_frame_width / 2.0,
            door_frame_height_center,
            north_z,
        ),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
        Some(parent),
    ));

    scene.add_entity(create_wall_entity(
        "MainHall_South_Left",
        nalgebra_glm::vec3(-room_half + side_section_width / 2.0, h, south_z),
        nalgebra_glm::vec3(side_section_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_South_Right",
        nalgebra_glm::vec3(room_half - side_section_width / 2.0, h, south_z),
        nalgebra_glm::vec3(side_section_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_South_Header",
        nalgebra_glm::vec3(0.0, header_center_y, south_z),
        nalgebra_glm::vec3(corridor_width, header_height, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_South_Frame_Left",
        nalgebra_glm::vec3(
            -door_width / 2.0 - door_frame_width / 2.0,
            door_frame_height_center,
            south_z,
        ),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_South_Frame_Right",
        nalgebra_glm::vec3(
            door_width / 2.0 + door_frame_width / 2.0,
            door_frame_height_center,
            south_z,
        ),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
        Some(parent),
    ));

    let side_room_depth = 6.0;
    let side_room_top_z = room_center_z + side_room_depth / 2.0;
    let side_room_bottom_z = room_center_z - side_room_depth / 2.0;
    let top_wall_depth = (room_center_z + room_half) - side_room_top_z;
    let bottom_wall_depth = side_room_bottom_z - (room_center_z - room_half);
    let east_x = room_half + t / 2.0;
    let west_x = -room_half - t / 2.0;

    scene.add_entity(create_wall_entity(
        "MainHall_East_Top",
        nalgebra_glm::vec3(east_x, h, room_center_z + room_half - top_wall_depth / 2.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, top_wall_depth),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_East_Bottom",
        nalgebra_glm::vec3(
            east_x,
            h,
            room_center_z - room_half + bottom_wall_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, bottom_wall_depth),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_East_Header",
        nalgebra_glm::vec3(east_x, header_center_y, room_center_z),
        nalgebra_glm::vec3(t, header_height, side_room_depth),
        wall_mat.clone(),
        Some(parent),
    ));

    let side_frame_depth = (side_room_depth - door_width) / 2.0;
    scene.add_entity(create_wall_entity(
        "MainHall_East_Frame_Top",
        nalgebra_glm::vec3(
            east_x,
            door_frame_height_center,
            side_room_top_z - side_frame_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, door_height, side_frame_depth),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_East_Frame_Bottom",
        nalgebra_glm::vec3(
            east_x,
            door_frame_height_center,
            side_room_bottom_z + side_frame_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, door_height, side_frame_depth),
        wall_mat.clone(),
        Some(parent),
    ));

    scene.add_entity(create_wall_entity(
        "MainHall_West_Top",
        nalgebra_glm::vec3(west_x, h, room_center_z + room_half - top_wall_depth / 2.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, top_wall_depth),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_West_Bottom",
        nalgebra_glm::vec3(
            west_x,
            h,
            room_center_z - room_half + bottom_wall_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, bottom_wall_depth),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_West_Header",
        nalgebra_glm::vec3(west_x, header_center_y, room_center_z),
        nalgebra_glm::vec3(t, header_height, side_room_depth),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_West_Frame_Top",
        nalgebra_glm::vec3(
            west_x,
            door_frame_height_center,
            side_room_top_z - side_frame_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, door_height, side_frame_depth),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "MainHall_West_Frame_Bottom",
        nalgebra_glm::vec3(
            west_x,
            door_frame_height_center,
            side_room_bottom_z + side_frame_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, door_height, side_frame_depth),
        wall_mat,
        Some(parent),
    ));

    let pillar_material = create_material([0.35, 0.35, 0.38], 0.8, 0.2);
    let table_material = create_material([0.3, 0.22, 0.15], 0.85, 0.1);

    scene.add_entity(create_static_cube_entity(
        "MainHall_Pillar_NW",
        nalgebra_glm::vec3(-4.0, h, room_center_z + 4.0),
        nalgebra_glm::vec3(0.5, ROOM_HEIGHT, 0.5),
        pillar_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "MainHall_Pillar_NE",
        nalgebra_glm::vec3(4.0, h, room_center_z + 4.0),
        nalgebra_glm::vec3(0.5, ROOM_HEIGHT, 0.5),
        pillar_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "MainHall_Pillar_SW",
        nalgebra_glm::vec3(-4.0, h, room_center_z - 4.0),
        nalgebra_glm::vec3(0.5, ROOM_HEIGHT, 0.5),
        pillar_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "MainHall_Pillar_SE",
        nalgebra_glm::vec3(4.0, h, room_center_z - 4.0),
        nalgebra_glm::vec3(0.5, ROOM_HEIGHT, 0.5),
        pillar_material,
        Some(parent),
    ));

    scene.add_entity(create_static_cube_entity(
        "MainHall_Table_Center",
        nalgebra_glm::vec3(0.0, 0.4, room_center_z),
        nalgebra_glm::vec3(1.8, 0.8, 1.0),
        table_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "MainHall_Table_Side",
        nalgebra_glm::vec3(-3.0, 0.35, room_center_z - 3.0),
        nalgebra_glm::vec3(1.2, 0.7, 0.8),
        table_material,
        Some(parent),
    ));
}

fn add_storage_room(scene: &mut Scene, parent: AssetUuid) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let room_width = 6.0;
    let room_depth = 6.0;
    let room_center_x = 9.0;
    let room_center_z = -16.0;

    scene.add_entity(create_wall_entity(
        "Storage_Wall_East",
        nalgebra_glm::vec3(room_center_x + room_width / 2.0 + t / 2.0, h, room_center_z),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, room_depth),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Storage_Wall_North",
        nalgebra_glm::vec3(room_center_x, h, room_center_z + room_depth / 2.0 + t / 2.0),
        nalgebra_glm::vec3(room_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Storage_Wall_South",
        nalgebra_glm::vec3(room_center_x, h, room_center_z - room_depth / 2.0 - t / 2.0),
        nalgebra_glm::vec3(room_width, ROOM_HEIGHT, t),
        wall_mat,
        Some(parent),
    ));

    let shelf_material = create_material([0.35, 0.25, 0.15], 0.85, 0.1);
    let crate_material = create_material([0.4, 0.3, 0.2], 0.9, 0.0);

    scene.add_entity(create_static_cube_entity(
        "Storage_Shelf_1",
        nalgebra_glm::vec3(room_center_x + 2.2, 1.0, room_center_z + 2.0),
        nalgebra_glm::vec3(0.8, 2.0, 0.4),
        shelf_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Storage_Shelf_2",
        nalgebra_glm::vec3(room_center_x + 2.2, 1.0, room_center_z),
        nalgebra_glm::vec3(0.8, 2.0, 0.4),
        shelf_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Storage_Shelf_3",
        nalgebra_glm::vec3(room_center_x + 2.2, 1.0, room_center_z - 2.0),
        nalgebra_glm::vec3(0.8, 2.0, 0.4),
        shelf_material,
        Some(parent),
    ));

    scene.add_entity(create_static_cube_entity(
        "Storage_Crate_1",
        nalgebra_glm::vec3(room_center_x, 0.25, room_center_z + 1.5),
        nalgebra_glm::vec3(0.5, 0.5, 0.5),
        crate_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Storage_Crate_2",
        nalgebra_glm::vec3(room_center_x + 0.3, 0.25, room_center_z + 0.8),
        nalgebra_glm::vec3(0.4, 0.5, 0.4),
        crate_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Storage_Crate_3",
        nalgebra_glm::vec3(room_center_x - 0.2, 0.75, room_center_z + 1.3),
        nalgebra_glm::vec3(0.35, 0.35, 0.35),
        crate_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Storage_Crate_4",
        nalgebra_glm::vec3(room_center_x - 1.5, 0.3, room_center_z - 1.0),
        nalgebra_glm::vec3(0.6, 0.6, 0.6),
        crate_material,
        Some(parent),
    ));
}

fn add_generator_room(scene: &mut Scene, parent: AssetUuid) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let room_width = 6.0;
    let room_depth = 6.0;
    let room_center_x = -9.0;
    let room_center_z = -16.0;

    scene.add_entity(create_wall_entity(
        "Generator_Wall_West",
        nalgebra_glm::vec3(room_center_x - room_width / 2.0 - t / 2.0, h, room_center_z),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, room_depth),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Generator_Wall_North",
        nalgebra_glm::vec3(room_center_x, h, room_center_z + room_depth / 2.0 + t / 2.0),
        nalgebra_glm::vec3(room_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Generator_Wall_South",
        nalgebra_glm::vec3(room_center_x, h, room_center_z - room_depth / 2.0 - t / 2.0),
        nalgebra_glm::vec3(room_width, ROOM_HEIGHT, t),
        wall_mat,
        Some(parent),
    ));

    let generator_material = create_material([0.2, 0.22, 0.25], 0.6, 0.4);
    let pipe_material = create_material([0.35, 0.3, 0.25], 0.5, 0.6);
    let panel_material = create_material([0.15, 0.15, 0.18], 0.4, 0.7);

    scene.add_entity(create_static_cube_entity(
        "Generator_Main",
        nalgebra_glm::vec3(room_center_x - 1.5, 0.6, room_center_z),
        nalgebra_glm::vec3(1.8, 1.2, 1.2),
        generator_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Generator_Top",
        nalgebra_glm::vec3(room_center_x - 1.5, 1.4, room_center_z),
        nalgebra_glm::vec3(1.4, 0.4, 0.8),
        generator_material,
        Some(parent),
    ));

    scene.add_entity(create_static_cube_entity(
        "Generator_Pipe_1",
        nalgebra_glm::vec3(room_center_x - 2.5, 1.5, room_center_z + 1.5),
        nalgebra_glm::vec3(0.15, 1.5, 0.15),
        pipe_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Generator_Pipe_2",
        nalgebra_glm::vec3(room_center_x - 2.5, 2.2, room_center_z + 0.5),
        nalgebra_glm::vec3(0.15, 0.15, 2.0),
        pipe_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Generator_Pipe_3",
        nalgebra_glm::vec3(room_center_x - 2.5, 1.5, room_center_z - 1.5),
        nalgebra_glm::vec3(0.15, 1.5, 0.15),
        pipe_material,
        Some(parent),
    ));

    scene.add_entity(create_static_cube_entity(
        "Generator_Panel_1",
        nalgebra_glm::vec3(room_center_x + 1.8, 1.0, room_center_z + 2.0),
        nalgebra_glm::vec3(0.6, 1.4, 0.3),
        panel_material.clone(),
        Some(parent),
    ));
    scene.add_entity(create_static_cube_entity(
        "Generator_Panel_2",
        nalgebra_glm::vec3(room_center_x + 1.8, 1.0, room_center_z - 2.0),
        nalgebra_glm::vec3(0.6, 1.4, 0.3),
        panel_material,
        Some(parent),
    ));
}

fn add_exit_corridor(scene: &mut Scene, parent: AssetUuid) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let corridor_width = 4.0;
    let corridor_half_width = corridor_width / 2.0;

    scene.add_entity(create_wall_entity(
        "Exit_Wall_Left",
        nalgebra_glm::vec3(-corridor_half_width - t / 2.0, h, -26.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 8.0),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Exit_Wall_Right",
        nalgebra_glm::vec3(corridor_half_width + t / 2.0, h, -26.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 8.0),
        wall_mat.clone(),
        Some(parent),
    ));
    scene.add_entity(create_wall_entity(
        "Exit_Wall_End",
        nalgebra_glm::vec3(0.0, h, -30.0 - t / 2.0),
        nalgebra_glm::vec3(corridor_width, ROOM_HEIGHT, t),
        wall_mat,
        Some(parent),
    ));
}

fn door_material() -> SceneMaterial {
    SceneMaterial {
        base_color: [0.45, 0.32, 0.2, 1.0],
        base_texture_name: Some("horror_door".to_string()),
        roughness: 0.8,
        metallic: 0.0,
        ..Default::default()
    }
}

fn add_door(
    scene: &mut Scene,
    parent: Option<AssetUuid>,
    name: &str,
    position: Vec3,
    side_door: bool,
) {
    let door_width = 1.2;
    let door_height = 2.2;
    let door_thickness = 0.08;

    let mut mat = door_material();
    mat.uv_scale = [door_width, door_height];

    let scale = if side_door {
        nalgebra_glm::vec3(door_thickness, door_height, door_width)
    } else {
        nalgebra_glm::vec3(door_width, door_height, door_thickness)
    };

    let mut entity = SceneEntity::new()
        .with_name(name)
        .with_transform(LocalTransform {
            translation: nalgebra_glm::vec3(position.x, door_height / 2.0, position.z),
            scale,
            ..Default::default()
        })
        .with_mesh(SceneMesh::from_name("Cube").with_material(mat))
        .with_casts_shadow(true)
        .with_visible(true);
    entity.components.physics = Some(ScenePhysics {
        body_type: SceneBodyType::KinematicPositionBased,
        collider: SceneCollider::Cuboid {
            half_extents: [scale.x / 2.0, scale.y / 2.0, scale.z / 2.0],
        },
        ..Default::default()
    });
    if let Some(p) = parent {
        entity = entity.with_parent(p);
    }
    scene.add_entity(entity);
}

fn add_doors(scene: &mut Scene, parent: AssetUuid) {
    let t = WALL_THICKNESS;

    add_door(
        scene,
        Some(parent),
        "Door_Entry",
        nalgebra_glm::vec3(0.0, 0.0, t / 2.0),
        false,
    );
    add_door(
        scene,
        Some(parent),
        "Door_Storage",
        nalgebra_glm::vec3(6.0 + t / 2.0, 0.0, -16.0),
        true,
    );
    add_door(
        scene,
        Some(parent),
        "Door_Generator",
        nalgebra_glm::vec3(-6.0 - t / 2.0, 0.0, -16.0),
        true,
    );
    add_door(
        scene,
        Some(parent),
        "Door_Exit",
        nalgebra_glm::vec3(0.0, 0.0, -22.0 - t / 2.0),
        false,
    );
}

fn lever_base_material() -> SceneMaterial {
    SceneMaterial {
        base_color: [0.25, 0.25, 0.28, 1.0],
        roughness: 0.85,
        metallic: 0.2,
        ..Default::default()
    }
}

fn lever_arm_material() -> SceneMaterial {
    SceneMaterial {
        base_color: [0.4, 0.3, 0.2, 1.0],
        roughness: 0.7,
        metallic: 0.3,
        ..Default::default()
    }
}

fn lever_handle_material() -> SceneMaterial {
    SceneMaterial {
        base_color: [0.6, 0.1, 0.1, 1.0],
        roughness: 0.4,
        metallic: 0.6,
        ..Default::default()
    }
}

fn lever_light_fixture_material() -> SceneMaterial {
    SceneMaterial {
        base_color: [0.2, 0.1, 0.1, 1.0],
        roughness: 0.3,
        metallic: 0.8,
        ..Default::default()
    }
}

fn create_mesh_entity(
    name: &str,
    transform: LocalTransform,
    mesh_name: &str,
    material: SceneMaterial,
    parent: Option<AssetUuid>,
) -> SceneEntity {
    let mut entity = SceneEntity::new()
        .with_name(name)
        .with_transform(transform)
        .with_mesh(SceneMesh::from_name(mesh_name).with_material(material))
        .with_casts_shadow(true)
        .with_visible(true);
    if let Some(p) = parent {
        entity = entity.with_parent(p);
    }
    entity
}

fn add_lever(scene: &mut Scene, parent: Option<AssetUuid>, name: &str, position: Vec3) {
    let arm_half_length = 0.2;
    let arm_half_thickness = 0.025;
    let handle_radius = 0.04;
    let light_fixture_size = 0.1;
    let initial_angle = -std::f32::consts::FRAC_PI_4;

    scene.add_entity(create_static_cube_entity(
        &format!("{}_Base", name),
        nalgebra_glm::vec3(position.x, position.y - 0.3, position.z),
        nalgebra_glm::vec3(0.3, 0.6, 0.15),
        lever_base_material(),
        parent,
    ));

    let pivot_entity = {
        let mut entity = SceneEntity::new()
            .with_name(format!("{}_Pivot", name))
            .with_transform(LocalTransform {
                translation: position,
                rotation: nalgebra_glm::quat_angle_axis(
                    initial_angle,
                    &nalgebra_glm::Vec3::x_axis(),
                ),
                ..Default::default()
            })
            .with_visible(true);
        if let Some(p) = parent {
            entity = entity.with_parent(p);
        }
        entity
    };
    let pivot_uuid = pivot_entity.uuid;
    scene.add_entity(pivot_entity);

    scene.add_entity(create_mesh_entity(
        &format!("{}_Arm", name),
        LocalTransform {
            translation: nalgebra_glm::vec3(0.0, 0.0, arm_half_length),
            scale: nalgebra_glm::vec3(
                arm_half_thickness * 2.0,
                arm_half_thickness * 2.0,
                arm_half_length * 2.0,
            ),
            ..Default::default()
        },
        "Cube",
        lever_arm_material(),
        Some(pivot_uuid),
    ));

    let handle_offset = arm_half_length * 2.0 + handle_radius;
    scene.add_entity(create_mesh_entity(
        &format!("{}_Handle", name),
        LocalTransform {
            translation: nalgebra_glm::vec3(0.0, 0.0, handle_offset),
            scale: nalgebra_glm::vec3(
                handle_radius * 2.0,
                handle_radius * 2.0,
                handle_radius * 2.0,
            ),
            ..Default::default()
        },
        "Sphere",
        lever_handle_material(),
        Some(pivot_uuid),
    ));

    scene.add_entity(create_mesh_entity(
        &format!("{}_Light", name),
        LocalTransform {
            translation: nalgebra_glm::vec3(position.x, position.y + 0.4, position.z - 0.15),
            scale: nalgebra_glm::vec3(light_fixture_size, light_fixture_size, light_fixture_size),
            ..Default::default()
        },
        "Cube",
        lever_light_fixture_material(),
        parent,
    ));
}

fn add_levers(scene: &mut Scene, parent: AssetUuid) {
    add_lever(
        scene,
        Some(parent),
        "Lever_RestorePower",
        nalgebra_glm::vec3(-8.0, 0.6, -14.5),
    );
    add_lever(
        scene,
        Some(parent),
        "Lever_UnlockExit",
        nalgebra_glm::vec3(3.0, 0.6, -18.0),
    );
}

fn add_overhead_light(
    scene: &mut Scene,
    parent: AssetUuid,
    name: &str,
    position: Vec3,
    color: [f32; 3],
    intensity: f32,
    range: f32,
) {
    scene.add_entity(create_light_entity(
        name,
        position,
        SceneLight::Point {
            color,
            intensity,
            range,
            cast_shadows: false,
            shadow_bias: 0.0,
        },
        Some(parent),
    ));
}

fn add_overhead_lights(scene: &mut Scene, parent: AssetUuid) {
    let warm_white = [1.0, 0.9, 0.8];
    let dim_white = [0.8, 0.8, 0.9];

    add_overhead_light(
        scene,
        parent,
        "Light_Entry",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, 4.0),
        warm_white,
        2.0,
        10.0,
    );

    add_overhead_light(
        scene,
        parent,
        "Light_Corridor",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -5.0),
        dim_white,
        1.5,
        8.0,
    );

    add_overhead_light(
        scene,
        parent,
        "Light_MainHall_Center",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -16.0),
        warm_white,
        3.0,
        12.0,
    );

    add_overhead_light(
        scene,
        parent,
        "Light_MainHall_North",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -12.0),
        dim_white,
        1.5,
        8.0,
    );

    add_overhead_light(
        scene,
        parent,
        "Light_MainHall_South",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -20.0),
        dim_white,
        1.5,
        8.0,
    );

    add_overhead_light(
        scene,
        parent,
        "Light_Storage",
        nalgebra_glm::vec3(9.0, ROOM_HEIGHT - 0.5, -16.0),
        dim_white,
        2.0,
        8.0,
    );

    add_overhead_light(
        scene,
        parent,
        "Light_Generator",
        nalgebra_glm::vec3(-9.0, ROOM_HEIGHT - 0.5, -16.0),
        [0.6, 0.7, 0.8],
        1.5,
        8.0,
    );

    add_overhead_light(
        scene,
        parent,
        "Light_Exit",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -26.0),
        dim_white,
        1.0,
        8.0,
    );
}

pub fn build_horror_scene() -> Scene {
    let mut scene = Scene::new("Horror Demo");
    scene.atmosphere = Atmosphere::None;

    let corridor_width = 4.0;
    let corridor_total_width = corridor_width + WALL_THICKNESS * 2.0;

    let rooms_parent = create_empty_entity("Rooms", None);
    let rooms_uuid = rooms_parent.uuid;
    scene.add_entity(rooms_parent);

    add_floor_ceiling(
        &mut scene,
        Some(rooms_uuid),
        "Entry",
        nalgebra_glm::vec3(0.0, 0.0, 4.0),
        8.0,
        8.0,
    );
    add_floor_ceiling(
        &mut scene,
        Some(rooms_uuid),
        "Corridor",
        nalgebra_glm::vec3(0.0, 0.0, -5.0),
        corridor_total_width,
        10.0,
    );
    add_floor_ceiling(
        &mut scene,
        Some(rooms_uuid),
        "MainHall",
        nalgebra_glm::vec3(0.0, 0.0, -16.0),
        12.0,
        12.0,
    );
    add_floor_ceiling(
        &mut scene,
        Some(rooms_uuid),
        "Storage",
        nalgebra_glm::vec3(9.0, 0.0, -16.0),
        6.0,
        6.0,
    );
    add_floor_ceiling(
        &mut scene,
        Some(rooms_uuid),
        "Generator",
        nalgebra_glm::vec3(-9.0, 0.0, -16.0),
        6.0,
        6.0,
    );
    add_floor_ceiling(
        &mut scene,
        Some(rooms_uuid),
        "Exit",
        nalgebra_glm::vec3(0.0, 0.0, -26.0),
        corridor_total_width,
        8.0,
    );

    let entry_parent = create_empty_entity("EntryRoom", Some(rooms_uuid));
    let entry_uuid = entry_parent.uuid;
    scene.add_entity(entry_parent);
    add_entry_room(&mut scene, entry_uuid);

    let corridor_parent = create_empty_entity("Corridor", Some(rooms_uuid));
    let corridor_uuid = corridor_parent.uuid;
    scene.add_entity(corridor_parent);
    add_corridor(&mut scene, corridor_uuid);

    let main_hall_parent = create_empty_entity("MainHall", Some(rooms_uuid));
    let main_hall_uuid = main_hall_parent.uuid;
    scene.add_entity(main_hall_parent);
    add_main_hall(&mut scene, main_hall_uuid);

    let storage_parent = create_empty_entity("StorageRoom", Some(rooms_uuid));
    let storage_uuid = storage_parent.uuid;
    scene.add_entity(storage_parent);
    add_storage_room(&mut scene, storage_uuid);

    let generator_parent = create_empty_entity("GeneratorRoom", Some(rooms_uuid));
    let generator_uuid = generator_parent.uuid;
    scene.add_entity(generator_parent);
    add_generator_room(&mut scene, generator_uuid);

    let exit_parent = create_empty_entity("ExitCorridor", Some(rooms_uuid));
    let exit_uuid = exit_parent.uuid;
    scene.add_entity(exit_parent);
    add_exit_corridor(&mut scene, exit_uuid);

    let doors_parent = create_empty_entity("Doors", Some(rooms_uuid));
    let doors_uuid = doors_parent.uuid;
    scene.add_entity(doors_parent);
    add_doors(&mut scene, doors_uuid);

    let interactables_parent = create_empty_entity("Interactables", Some(rooms_uuid));
    let interactables_uuid = interactables_parent.uuid;
    scene.add_entity(interactables_parent);
    add_levers(&mut scene, interactables_uuid);

    let lights_parent = create_empty_entity("Lights", Some(rooms_uuid));
    let lights_uuid = lights_parent.uuid;
    scene.add_entity(lights_parent);
    add_overhead_lights(&mut scene, lights_uuid);

    scene
}
