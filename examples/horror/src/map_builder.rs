use crate::constants::{
    ATMOSPHERE_AUDIO, CEILING_TEXTURE, DOOR_CREAK_AUDIO, DOOR_TEXTURE, FLOOR_TEXTURE,
    FOOTSTEPS_AUDIO, GENERATOR_AUDIO, LEVER_TEXTURE, MONSTER_AUDIO, NOTE_TEXTURE, ROOM_HEIGHT,
    RUBBLE_AUDIO, WALL_TEXTURE, WALL_THICKNESS,
};
use nightshade::ecs::bounding_volume::components::BoundingVolume;
use nightshade::ecs::graphics::resources::Atmosphere;
use nightshade::ecs::map::{
    Map, MapAudioSource, MapLight, MapMaterial, MapNode, MapPhysics, NodeIndex,
};
use nightshade::ecs::transform::LocalTransform;
use nightshade::prelude::{Vec3, nalgebra_glm};

fn floor_material() -> MapMaterial {
    MapMaterial {
        base_color: [0.3, 0.3, 0.3, 1.0],
        base_texture: Some("horror_floor".to_string()),
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    }
}

fn wall_material() -> MapMaterial {
    MapMaterial {
        base_color: [0.4, 0.4, 0.4, 1.0],
        base_texture: Some("horror_wall".to_string()),
        roughness: 0.9,
        metallic: 0.0,
        ..Default::default()
    }
}

fn ceiling_material() -> MapMaterial {
    MapMaterial {
        base_color: [0.2, 0.2, 0.2, 1.0],
        base_texture: Some("horror_ceiling".to_string()),
        roughness: 0.95,
        metallic: 0.0,
        ..Default::default()
    }
}

fn create_material(color: [f32; 3], roughness: f32, metallic: f32) -> MapMaterial {
    MapMaterial {
        base_color: [color[0], color[1], color[2], 1.0],
        roughness,
        metallic,
        ..Default::default()
    }
}

fn add_static_cube(
    map: &mut Map,
    parent: Option<NodeIndex>,
    name: &str,
    position: Vec3,
    size: Vec3,
    material: MapMaterial,
) -> NodeIndex {
    let entity = MapNode::entity_full(
        Some(name.to_string()),
        LocalTransform {
            translation: position,
            scale: size,
            ..Default::default()
        },
    );
    let entity_index = if let Some(parent_idx) = parent {
        map.add_child_node(parent_idx, entity)
    } else {
        map.add_root_node(entity)
    };

    map.add_child_node(
        entity_index,
        MapNode::mesh_with_physics("Cube", material, MapPhysics::static_from_mesh()),
    );

    entity_index
}

fn add_wall(
    map: &mut Map,
    parent: Option<NodeIndex>,
    name: &str,
    position: Vec3,
    size: Vec3,
    material: MapMaterial,
) -> NodeIndex {
    let mut mat = material;
    if size.x > size.z {
        mat.uv_scale = [size.x, size.y];
    } else {
        mat.uv_scale = [size.z, size.y];
    }
    add_static_cube(map, parent, name, position, size, mat)
}

fn add_floor_ceiling(
    map: &mut Map,
    parent: Option<NodeIndex>,
    name_prefix: &str,
    center: Vec3,
    width: f32,
    depth: f32,
) {
    let mut floor_mat = floor_material();
    floor_mat.uv_scale = [width, depth];

    let floor_pos = nalgebra_glm::vec3(center.x, -WALL_THICKNESS / 2.0, center.z);
    let floor_size = nalgebra_glm::vec3(width, WALL_THICKNESS, depth);
    add_static_cube(
        map,
        parent,
        &format!("{}_Floor", name_prefix),
        floor_pos,
        floor_size,
        floor_mat,
    );

    let mut ceiling_mat = ceiling_material();
    ceiling_mat.uv_scale = [width, depth];

    let ceiling_pos = nalgebra_glm::vec3(center.x, ROOM_HEIGHT + WALL_THICKNESS / 2.0, center.z);
    let ceiling_size = nalgebra_glm::vec3(width, WALL_THICKNESS, depth);
    add_static_cube(
        map,
        parent,
        &format!("{}_Ceiling", name_prefix),
        ceiling_pos,
        ceiling_size,
        ceiling_mat,
    );
}

fn add_entry_room(map: &mut Map, parent: NodeIndex) {
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

    add_wall(
        map,
        Some(parent),
        "Entry_Wall_Left",
        nalgebra_glm::vec3(-room_half - t / 2.0, h, 4.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 8.0),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Entry_Wall_Right",
        nalgebra_glm::vec3(room_half + t / 2.0, h, 4.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 8.0),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Entry_Wall_Back",
        nalgebra_glm::vec3(0.0, h, 8.0 + t / 2.0),
        nalgebra_glm::vec3(8.0, ROOM_HEIGHT, t),
        wall_mat.clone(),
    );

    let side_wall_width = room_half - corridor_width / 2.0;
    let left_center_x = -room_half + side_wall_width / 2.0;
    let right_center_x = room_half - side_wall_width / 2.0;

    add_wall(
        map,
        Some(parent),
        "Entry_Door_Left_Side",
        nalgebra_glm::vec3(left_center_x, h, t / 2.0),
        nalgebra_glm::vec3(side_wall_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Entry_Door_Right_Side",
        nalgebra_glm::vec3(right_center_x, h, t / 2.0),
        nalgebra_glm::vec3(side_wall_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Entry_Door_Header",
        nalgebra_glm::vec3(0.0, header_center_y, t / 2.0),
        nalgebra_glm::vec3(corridor_width, header_height, t),
        wall_mat.clone(),
    );

    let door_frame_width = (corridor_width - door_width) / 2.0;
    let left_frame_x = -door_width / 2.0 - door_frame_width / 2.0;
    let right_frame_x = door_width / 2.0 + door_frame_width / 2.0;

    add_wall(
        map,
        Some(parent),
        "Entry_Door_Frame_Left",
        nalgebra_glm::vec3(left_frame_x, door_frame_height_center, t / 2.0),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Entry_Door_Frame_Right",
        nalgebra_glm::vec3(right_frame_x, door_frame_height_center, t / 2.0),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat,
    );

    let desk_material = create_material([0.28, 0.2, 0.12], 0.85, 0.1);
    let chair_material = create_material([0.25, 0.18, 0.1], 0.9, 0.05);

    add_static_cube(
        map,
        Some(parent),
        "Entry_Desk",
        nalgebra_glm::vec3(2.5, 0.4, 5.5),
        nalgebra_glm::vec3(1.6, 0.8, 0.8),
        desk_material,
    );
    add_static_cube(
        map,
        Some(parent),
        "Entry_Chair",
        nalgebra_glm::vec3(2.5, 0.3, 4.5),
        nalgebra_glm::vec3(0.45, 0.6, 0.45),
        chair_material,
    );
}

fn add_corridor(map: &mut Map, parent: NodeIndex) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let corridor_width = 4.0;
    let corridor_half_width = corridor_width / 2.0;

    add_wall(
        map,
        Some(parent),
        "Corridor_Wall_Left",
        nalgebra_glm::vec3(-corridor_half_width - t / 2.0, h, -5.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 10.0),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Corridor_Wall_Right",
        nalgebra_glm::vec3(corridor_half_width + t / 2.0, h, -5.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 10.0),
        wall_mat,
    );
}

fn add_main_hall(map: &mut Map, parent: NodeIndex) {
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

    add_wall(
        map,
        Some(parent),
        "MainHall_North_Left",
        nalgebra_glm::vec3(-room_half + side_section_width / 2.0, h, north_z),
        nalgebra_glm::vec3(side_section_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_North_Right",
        nalgebra_glm::vec3(room_half - side_section_width / 2.0, h, north_z),
        nalgebra_glm::vec3(side_section_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_North_Header",
        nalgebra_glm::vec3(0.0, header_center_y, north_z),
        nalgebra_glm::vec3(corridor_width, header_height, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_North_Frame_Left",
        nalgebra_glm::vec3(
            -door_width / 2.0 - door_frame_width / 2.0,
            door_frame_height_center,
            north_z,
        ),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_North_Frame_Right",
        nalgebra_glm::vec3(
            door_width / 2.0 + door_frame_width / 2.0,
            door_frame_height_center,
            north_z,
        ),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
    );

    add_wall(
        map,
        Some(parent),
        "MainHall_South_Left",
        nalgebra_glm::vec3(-room_half + side_section_width / 2.0, h, south_z),
        nalgebra_glm::vec3(side_section_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_South_Right",
        nalgebra_glm::vec3(room_half - side_section_width / 2.0, h, south_z),
        nalgebra_glm::vec3(side_section_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_South_Header",
        nalgebra_glm::vec3(0.0, header_center_y, south_z),
        nalgebra_glm::vec3(corridor_width, header_height, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_South_Frame_Left",
        nalgebra_glm::vec3(
            -door_width / 2.0 - door_frame_width / 2.0,
            door_frame_height_center,
            south_z,
        ),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_South_Frame_Right",
        nalgebra_glm::vec3(
            door_width / 2.0 + door_frame_width / 2.0,
            door_frame_height_center,
            south_z,
        ),
        nalgebra_glm::vec3(door_frame_width, door_height, t),
        wall_mat.clone(),
    );

    let side_room_depth = 6.0;
    let side_room_top_z = room_center_z + side_room_depth / 2.0;
    let side_room_bottom_z = room_center_z - side_room_depth / 2.0;
    let top_wall_depth = (room_center_z + room_half) - side_room_top_z;
    let bottom_wall_depth = side_room_bottom_z - (room_center_z - room_half);
    let east_x = room_half + t / 2.0;
    let west_x = -room_half - t / 2.0;

    add_wall(
        map,
        Some(parent),
        "MainHall_East_Top",
        nalgebra_glm::vec3(east_x, h, room_center_z + room_half - top_wall_depth / 2.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, top_wall_depth),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_East_Bottom",
        nalgebra_glm::vec3(
            east_x,
            h,
            room_center_z - room_half + bottom_wall_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, bottom_wall_depth),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_East_Header",
        nalgebra_glm::vec3(east_x, header_center_y, room_center_z),
        nalgebra_glm::vec3(t, header_height, side_room_depth),
        wall_mat.clone(),
    );

    let side_frame_depth = (side_room_depth - door_width) / 2.0;
    add_wall(
        map,
        Some(parent),
        "MainHall_East_Frame_Top",
        nalgebra_glm::vec3(
            east_x,
            door_frame_height_center,
            side_room_top_z - side_frame_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, door_height, side_frame_depth),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_East_Frame_Bottom",
        nalgebra_glm::vec3(
            east_x,
            door_frame_height_center,
            side_room_bottom_z + side_frame_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, door_height, side_frame_depth),
        wall_mat.clone(),
    );

    add_wall(
        map,
        Some(parent),
        "MainHall_West_Top",
        nalgebra_glm::vec3(west_x, h, room_center_z + room_half - top_wall_depth / 2.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, top_wall_depth),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_West_Bottom",
        nalgebra_glm::vec3(
            west_x,
            h,
            room_center_z - room_half + bottom_wall_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, bottom_wall_depth),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_West_Header",
        nalgebra_glm::vec3(west_x, header_center_y, room_center_z),
        nalgebra_glm::vec3(t, header_height, side_room_depth),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_West_Frame_Top",
        nalgebra_glm::vec3(
            west_x,
            door_frame_height_center,
            side_room_top_z - side_frame_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, door_height, side_frame_depth),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "MainHall_West_Frame_Bottom",
        nalgebra_glm::vec3(
            west_x,
            door_frame_height_center,
            side_room_bottom_z + side_frame_depth / 2.0,
        ),
        nalgebra_glm::vec3(t, door_height, side_frame_depth),
        wall_mat,
    );

    let pillar_material = create_material([0.35, 0.35, 0.38], 0.8, 0.2);
    let table_material = create_material([0.3, 0.22, 0.15], 0.85, 0.1);

    add_static_cube(
        map,
        Some(parent),
        "MainHall_Pillar_NW",
        nalgebra_glm::vec3(-4.0, h, room_center_z + 4.0),
        nalgebra_glm::vec3(0.5, ROOM_HEIGHT, 0.5),
        pillar_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "MainHall_Pillar_NE",
        nalgebra_glm::vec3(4.0, h, room_center_z + 4.0),
        nalgebra_glm::vec3(0.5, ROOM_HEIGHT, 0.5),
        pillar_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "MainHall_Pillar_SW",
        nalgebra_glm::vec3(-4.0, h, room_center_z - 4.0),
        nalgebra_glm::vec3(0.5, ROOM_HEIGHT, 0.5),
        pillar_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "MainHall_Pillar_SE",
        nalgebra_glm::vec3(4.0, h, room_center_z - 4.0),
        nalgebra_glm::vec3(0.5, ROOM_HEIGHT, 0.5),
        pillar_material,
    );

    add_static_cube(
        map,
        Some(parent),
        "MainHall_Table_Center",
        nalgebra_glm::vec3(0.0, 0.4, room_center_z),
        nalgebra_glm::vec3(1.8, 0.8, 1.0),
        table_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "MainHall_Table_Side",
        nalgebra_glm::vec3(-3.0, 0.35, room_center_z - 3.0),
        nalgebra_glm::vec3(1.2, 0.7, 0.8),
        table_material,
    );
}

fn add_storage_room(map: &mut Map, parent: NodeIndex) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let room_width = 6.0;
    let room_depth = 6.0;
    let room_center_x = 9.0;
    let room_center_z = -16.0;

    add_wall(
        map,
        Some(parent),
        "Storage_Wall_East",
        nalgebra_glm::vec3(room_center_x + room_width / 2.0 + t / 2.0, h, room_center_z),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, room_depth),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Storage_Wall_North",
        nalgebra_glm::vec3(room_center_x, h, room_center_z + room_depth / 2.0 + t / 2.0),
        nalgebra_glm::vec3(room_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Storage_Wall_South",
        nalgebra_glm::vec3(room_center_x, h, room_center_z - room_depth / 2.0 - t / 2.0),
        nalgebra_glm::vec3(room_width, ROOM_HEIGHT, t),
        wall_mat,
    );

    let shelf_material = create_material([0.35, 0.25, 0.15], 0.85, 0.1);
    let crate_material = create_material([0.4, 0.3, 0.2], 0.9, 0.0);

    add_static_cube(
        map,
        Some(parent),
        "Storage_Shelf_1",
        nalgebra_glm::vec3(room_center_x + 2.2, 1.0, room_center_z + 2.0),
        nalgebra_glm::vec3(0.8, 2.0, 0.4),
        shelf_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "Storage_Shelf_2",
        nalgebra_glm::vec3(room_center_x + 2.2, 1.0, room_center_z),
        nalgebra_glm::vec3(0.8, 2.0, 0.4),
        shelf_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "Storage_Shelf_3",
        nalgebra_glm::vec3(room_center_x + 2.2, 1.0, room_center_z - 2.0),
        nalgebra_glm::vec3(0.8, 2.0, 0.4),
        shelf_material,
    );

    add_static_cube(
        map,
        Some(parent),
        "Storage_Crate_1",
        nalgebra_glm::vec3(room_center_x, 0.25, room_center_z + 1.5),
        nalgebra_glm::vec3(0.5, 0.5, 0.5),
        crate_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "Storage_Crate_2",
        nalgebra_glm::vec3(room_center_x + 0.3, 0.25, room_center_z + 0.8),
        nalgebra_glm::vec3(0.4, 0.5, 0.4),
        crate_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "Storage_Crate_3",
        nalgebra_glm::vec3(room_center_x - 0.2, 0.75, room_center_z + 1.3),
        nalgebra_glm::vec3(0.35, 0.35, 0.35),
        crate_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "Storage_Crate_4",
        nalgebra_glm::vec3(room_center_x - 1.5, 0.3, room_center_z - 1.0),
        nalgebra_glm::vec3(0.6, 0.6, 0.6),
        crate_material,
    );
}

fn add_generator_room(map: &mut Map, parent: NodeIndex) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let room_width = 6.0;
    let room_depth = 6.0;
    let room_center_x = -9.0;
    let room_center_z = -16.0;

    add_wall(
        map,
        Some(parent),
        "Generator_Wall_West",
        nalgebra_glm::vec3(room_center_x - room_width / 2.0 - t / 2.0, h, room_center_z),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, room_depth),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Generator_Wall_North",
        nalgebra_glm::vec3(room_center_x, h, room_center_z + room_depth / 2.0 + t / 2.0),
        nalgebra_glm::vec3(room_width, ROOM_HEIGHT, t),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Generator_Wall_South",
        nalgebra_glm::vec3(room_center_x, h, room_center_z - room_depth / 2.0 - t / 2.0),
        nalgebra_glm::vec3(room_width, ROOM_HEIGHT, t),
        wall_mat,
    );

    let generator_material = create_material([0.2, 0.22, 0.25], 0.6, 0.4);
    let pipe_material = create_material([0.35, 0.3, 0.25], 0.5, 0.6);
    let panel_material = create_material([0.15, 0.15, 0.18], 0.4, 0.7);

    add_static_cube(
        map,
        Some(parent),
        "Generator_Main",
        nalgebra_glm::vec3(room_center_x - 1.5, 0.6, room_center_z),
        nalgebra_glm::vec3(1.8, 1.2, 1.2),
        generator_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "Generator_Top",
        nalgebra_glm::vec3(room_center_x - 1.5, 1.4, room_center_z),
        nalgebra_glm::vec3(1.4, 0.4, 0.8),
        generator_material,
    );

    add_static_cube(
        map,
        Some(parent),
        "Generator_Pipe_1",
        nalgebra_glm::vec3(room_center_x - 2.5, 1.5, room_center_z + 1.5),
        nalgebra_glm::vec3(0.15, 1.5, 0.15),
        pipe_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "Generator_Pipe_2",
        nalgebra_glm::vec3(room_center_x - 2.5, 2.2, room_center_z + 0.5),
        nalgebra_glm::vec3(0.15, 0.15, 2.0),
        pipe_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "Generator_Pipe_3",
        nalgebra_glm::vec3(room_center_x - 2.5, 1.5, room_center_z - 1.5),
        nalgebra_glm::vec3(0.15, 1.5, 0.15),
        pipe_material,
    );

    add_static_cube(
        map,
        Some(parent),
        "Generator_Panel_1",
        nalgebra_glm::vec3(room_center_x + 1.8, 1.0, room_center_z + 2.0),
        nalgebra_glm::vec3(0.6, 1.4, 0.3),
        panel_material.clone(),
    );
    add_static_cube(
        map,
        Some(parent),
        "Generator_Panel_2",
        nalgebra_glm::vec3(room_center_x + 1.8, 1.0, room_center_z - 2.0),
        nalgebra_glm::vec3(0.6, 1.4, 0.3),
        panel_material,
    );
}

fn add_exit_corridor(map: &mut Map, parent: NodeIndex) {
    let wall_mat = wall_material();
    let h = ROOM_HEIGHT / 2.0;
    let t = WALL_THICKNESS;
    let corridor_width = 4.0;
    let corridor_half_width = corridor_width / 2.0;

    add_wall(
        map,
        Some(parent),
        "Exit_Wall_Left",
        nalgebra_glm::vec3(-corridor_half_width - t / 2.0, h, -26.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 8.0),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Exit_Wall_Right",
        nalgebra_glm::vec3(corridor_half_width + t / 2.0, h, -26.0),
        nalgebra_glm::vec3(t, ROOM_HEIGHT, 8.0),
        wall_mat.clone(),
    );
    add_wall(
        map,
        Some(parent),
        "Exit_Wall_End",
        nalgebra_glm::vec3(0.0, h, -30.0 - t / 2.0),
        nalgebra_glm::vec3(corridor_width, ROOM_HEIGHT, t),
        wall_mat,
    );
}

fn door_material() -> MapMaterial {
    MapMaterial {
        base_color: [0.45, 0.32, 0.2, 1.0],
        base_texture: Some("horror_door".to_string()),
        roughness: 0.8,
        metallic: 0.0,
        ..Default::default()
    }
}

fn add_door(
    map: &mut Map,
    parent: Option<NodeIndex>,
    name: &str,
    position: Vec3,
    rotation: f32,
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

    let entity = MapNode::entity_full(
        Some(name.to_string()),
        LocalTransform {
            translation: nalgebra_glm::vec3(position.x, door_height / 2.0, position.z),
            rotation: nalgebra_glm::quat_angle_axis(rotation, &nalgebra_glm::Vec3::y_axis()),
            scale,
        },
    );
    let entity_index = if let Some(parent_idx) = parent {
        map.add_child_node(parent_idx, entity)
    } else {
        map.add_root_node(entity)
    };

    map.add_child_node(
        entity_index,
        MapNode::mesh_full(
            "Cube",
            Some(mat),
            true,
            Some(BoundingVolume::from_mesh_type("Cube")),
            Some(MapPhysics::kinematic_from_mesh()),
        ),
    );
}

fn add_doors(map: &mut Map, parent: NodeIndex) {
    let t = WALL_THICKNESS;

    add_door(
        map,
        Some(parent),
        "Door_Entry",
        nalgebra_glm::vec3(0.0, 0.0, t / 2.0),
        0.0,
        false,
    );
    add_door(
        map,
        Some(parent),
        "Door_Storage",
        nalgebra_glm::vec3(6.0 + t / 2.0, 0.0, -16.0),
        0.0,
        true,
    );
    add_door(
        map,
        Some(parent),
        "Door_Generator",
        nalgebra_glm::vec3(-6.0 - t / 2.0, 0.0, -16.0),
        0.0,
        true,
    );
    add_door(
        map,
        Some(parent),
        "Door_Exit",
        nalgebra_glm::vec3(0.0, 0.0, -22.0 - t / 2.0),
        0.0,
        false,
    );
}

fn lever_base_material() -> MapMaterial {
    MapMaterial {
        base_color: [0.25, 0.25, 0.28, 1.0],
        roughness: 0.85,
        metallic: 0.2,
        ..Default::default()
    }
}

fn lever_arm_material() -> MapMaterial {
    MapMaterial {
        base_color: [0.4, 0.3, 0.2, 1.0],
        roughness: 0.7,
        metallic: 0.3,
        ..Default::default()
    }
}

fn lever_handle_material() -> MapMaterial {
    MapMaterial {
        base_color: [0.6, 0.1, 0.1, 1.0],
        roughness: 0.4,
        metallic: 0.6,
        ..Default::default()
    }
}

fn lever_light_fixture_material() -> MapMaterial {
    MapMaterial {
        base_color: [0.2, 0.1, 0.1, 1.0],
        roughness: 0.3,
        metallic: 0.8,
        ..Default::default()
    }
}

fn add_lever(map: &mut Map, parent: Option<NodeIndex>, name: &str, position: Vec3) {
    let arm_half_length = 0.2;
    let arm_half_thickness = 0.025;
    let handle_radius = 0.04;
    let light_fixture_size = 0.1;
    let initial_angle = -std::f32::consts::FRAC_PI_4;

    add_static_cube(
        map,
        parent,
        &format!("{}_Base", name),
        nalgebra_glm::vec3(position.x, position.y - 0.3, position.z),
        nalgebra_glm::vec3(0.3, 0.6, 0.15),
        lever_base_material(),
    );

    let pivot_entity = MapNode::entity_full(
        Some(format!("{}_Pivot", name)),
        LocalTransform {
            translation: position,
            rotation: nalgebra_glm::quat_angle_axis(initial_angle, &nalgebra_glm::Vec3::x_axis()),
            ..Default::default()
        },
    );
    let pivot_index = if let Some(parent_idx) = parent {
        map.add_child_node(parent_idx, pivot_entity)
    } else {
        map.add_root_node(pivot_entity)
    };

    let arm_entity = MapNode::entity_full(
        Some(format!("{}_Arm", name)),
        LocalTransform {
            translation: nalgebra_glm::vec3(0.0, 0.0, arm_half_length),
            scale: nalgebra_glm::vec3(
                arm_half_thickness * 2.0,
                arm_half_thickness * 2.0,
                arm_half_length * 2.0,
            ),
            ..Default::default()
        },
    );
    let arm_index = map.add_child_node(pivot_index, arm_entity);
    map.add_child_node(
        arm_index,
        MapNode::mesh_with_material("Cube", lever_arm_material()),
    );

    let handle_offset = arm_half_length * 2.0 + handle_radius;
    let handle_entity = MapNode::entity_full(
        Some(format!("{}_Handle", name)),
        LocalTransform {
            translation: nalgebra_glm::vec3(0.0, 0.0, handle_offset),
            scale: nalgebra_glm::vec3(
                handle_radius * 2.0,
                handle_radius * 2.0,
                handle_radius * 2.0,
            ),
            ..Default::default()
        },
    );
    let handle_index = map.add_child_node(pivot_index, handle_entity);
    map.add_child_node(
        handle_index,
        MapNode::mesh_with_material("Sphere", lever_handle_material()),
    );

    let light_entity = MapNode::entity_full(
        Some(format!("{}_Light", name)),
        LocalTransform {
            translation: nalgebra_glm::vec3(position.x, position.y + 0.4, position.z - 0.15),
            scale: nalgebra_glm::vec3(light_fixture_size, light_fixture_size, light_fixture_size),
            ..Default::default()
        },
    );
    let light_index = if let Some(parent_idx) = parent {
        map.add_child_node(parent_idx, light_entity)
    } else {
        map.add_root_node(light_entity)
    };
    map.add_child_node(
        light_index,
        MapNode::mesh_with_material("Cube", lever_light_fixture_material()),
    );
}

fn add_levers(map: &mut Map, parent: NodeIndex) {
    add_lever(
        map,
        Some(parent),
        "Lever_RestorePower",
        nalgebra_glm::vec3(-8.0, 0.6, -14.5),
    );
    add_lever(
        map,
        Some(parent),
        "Lever_UnlockExit",
        nalgebra_glm::vec3(3.0, 0.6, -18.0),
    );
}

fn add_overhead_light(
    map: &mut Map,
    parent: NodeIndex,
    name: &str,
    position: Vec3,
    color: [f32; 3],
    intensity: f32,
    range: f32,
) {
    let light_entity = MapNode::entity_full(
        Some(name.to_string()),
        LocalTransform {
            translation: position,
            ..Default::default()
        },
    );
    let light_idx = map.add_child_node(parent, light_entity);
    map.add_child_node(
        light_idx,
        MapNode::light(MapLight::point(color, intensity, range)),
    );
}

fn add_overhead_lights(map: &mut Map, parent: NodeIndex) {
    let warm_white = [1.0, 0.9, 0.8];
    let dim_white = [0.8, 0.8, 0.9];

    add_overhead_light(
        map,
        parent,
        "Light_Entry",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, 4.0),
        warm_white,
        2.0,
        10.0,
    );

    add_overhead_light(
        map,
        parent,
        "Light_Corridor",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -5.0),
        dim_white,
        1.5,
        8.0,
    );

    add_overhead_light(
        map,
        parent,
        "Light_MainHall_Center",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -16.0),
        warm_white,
        3.0,
        12.0,
    );

    add_overhead_light(
        map,
        parent,
        "Light_MainHall_North",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -12.0),
        dim_white,
        1.5,
        8.0,
    );

    add_overhead_light(
        map,
        parent,
        "Light_MainHall_South",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -20.0),
        dim_white,
        1.5,
        8.0,
    );

    add_overhead_light(
        map,
        parent,
        "Light_Storage",
        nalgebra_glm::vec3(9.0, ROOM_HEIGHT - 0.5, -16.0),
        dim_white,
        2.0,
        8.0,
    );

    add_overhead_light(
        map,
        parent,
        "Light_Generator",
        nalgebra_glm::vec3(-9.0, ROOM_HEIGHT - 0.5, -16.0),
        [0.6, 0.7, 0.8],
        1.5,
        8.0,
    );

    add_overhead_light(
        map,
        parent,
        "Light_Exit",
        nalgebra_glm::vec3(0.0, ROOM_HEIGHT - 0.5, -26.0),
        dim_white,
        1.0,
        8.0,
    );
}

fn add_audio_sources(map: &mut Map, parent: NodeIndex) {
    let ambient_entity = MapNode::entity_named("Audio_Ambient");
    let ambient_idx = map.add_child_node(parent, ambient_entity);
    map.add_child_node(
        ambient_idx,
        MapNode::audio_source(
            MapAudioSource::new("atmosphere")
                .with_volume(0.4)
                .with_looping(true),
        ),
    );

    let generator_entity = MapNode::entity_full(
        Some("Audio_Generator".to_string()),
        LocalTransform {
            translation: nalgebra_glm::vec3(-8.0, 1.0, -14.5),
            ..Default::default()
        },
    );
    let generator_idx = map.add_child_node(parent, generator_entity);
    map.add_child_node(
        generator_idx,
        MapNode::audio_source(
            MapAudioSource::new("generator")
                .with_volume(1.0)
                .with_looping(false)
                .with_spatial(true),
        ),
    );

    let rubble_entity = MapNode::entity_full(
        Some("Audio_Rubble".to_string()),
        LocalTransform {
            translation: nalgebra_glm::vec3(-4.5, 1.5, -16.0),
            ..Default::default()
        },
    );
    let rubble_idx = map.add_child_node(parent, rubble_entity);
    map.add_child_node(
        rubble_idx,
        MapNode::audio_source(
            MapAudioSource::new("rubble")
                .with_volume(1.0)
                .with_looping(false)
                .with_spatial(true)
                .with_reverb(true),
        ),
    );

    let monster_entity = MapNode::entity_named("Audio_Monster");
    let monster_idx = map.add_child_node(parent, monster_entity);
    map.add_child_node(
        monster_idx,
        MapNode::audio_source(
            MapAudioSource::new("monster")
                .with_volume(0.6)
                .with_looping(true),
        ),
    );

    let footsteps_entity = MapNode::entity_named("Audio_Footsteps");
    let footsteps_idx = map.add_child_node(parent, footsteps_entity);
    map.add_child_node(
        footsteps_idx,
        MapNode::audio_source(
            MapAudioSource::new("footsteps")
                .with_volume(0.4)
                .with_looping(true),
        ),
    );

    let door_creak_entity = MapNode::entity_named("Audio_DoorCreak");
    let door_creak_idx = map.add_child_node(parent, door_creak_entity);
    map.add_child_node(
        door_creak_idx,
        MapNode::audio_source(
            MapAudioSource::new("door_creak")
                .with_volume(0.6)
                .with_looping(false),
        ),
    );
}

pub fn build_horror_map() -> Map {
    let mut map = Map::new("Horror Demo");
    map.atmosphere = Atmosphere::None;

    map.add_texture_from_bytes("horror_floor", FLOOR_TEXTURE);
    map.add_texture_from_bytes("horror_wall", WALL_TEXTURE);
    map.add_texture_from_bytes("horror_ceiling", CEILING_TEXTURE);
    map.add_texture_from_bytes("horror_door", DOOR_TEXTURE);
    map.add_texture_from_bytes("horror_note", NOTE_TEXTURE);
    map.add_texture_from_bytes("horror_lever", LEVER_TEXTURE);

    map.add_audio_from_bytes("atmosphere", ATMOSPHERE_AUDIO);
    map.add_audio_from_bytes("generator", GENERATOR_AUDIO);
    map.add_audio_from_bytes("rubble", RUBBLE_AUDIO);
    map.add_audio_from_bytes("monster", MONSTER_AUDIO);
    map.add_audio_from_bytes("footsteps", FOOTSTEPS_AUDIO);
    map.add_audio_from_bytes("door_creak", DOOR_CREAK_AUDIO);

    let corridor_width = 4.0;
    let corridor_total_width = corridor_width + WALL_THICKNESS * 2.0;

    let rooms_parent = map.add_root_node(MapNode::entity_named("Rooms"));

    add_floor_ceiling(
        &mut map,
        Some(rooms_parent),
        "Entry",
        nalgebra_glm::vec3(0.0, 0.0, 4.0),
        8.0,
        8.0,
    );
    add_floor_ceiling(
        &mut map,
        Some(rooms_parent),
        "Corridor",
        nalgebra_glm::vec3(0.0, 0.0, -5.0),
        corridor_total_width,
        10.0,
    );
    add_floor_ceiling(
        &mut map,
        Some(rooms_parent),
        "MainHall",
        nalgebra_glm::vec3(0.0, 0.0, -16.0),
        12.0,
        12.0,
    );
    add_floor_ceiling(
        &mut map,
        Some(rooms_parent),
        "Storage",
        nalgebra_glm::vec3(9.0, 0.0, -16.0),
        6.0,
        6.0,
    );
    add_floor_ceiling(
        &mut map,
        Some(rooms_parent),
        "Generator",
        nalgebra_glm::vec3(-9.0, 0.0, -16.0),
        6.0,
        6.0,
    );
    add_floor_ceiling(
        &mut map,
        Some(rooms_parent),
        "Exit",
        nalgebra_glm::vec3(0.0, 0.0, -26.0),
        corridor_total_width,
        8.0,
    );

    let entry_parent = map.add_child_node(rooms_parent, MapNode::entity_named("EntryRoom"));
    add_entry_room(&mut map, entry_parent);

    let corridor_parent = map.add_child_node(rooms_parent, MapNode::entity_named("Corridor"));
    add_corridor(&mut map, corridor_parent);

    let main_hall_parent = map.add_child_node(rooms_parent, MapNode::entity_named("MainHall"));
    add_main_hall(&mut map, main_hall_parent);

    let storage_parent = map.add_child_node(rooms_parent, MapNode::entity_named("StorageRoom"));
    add_storage_room(&mut map, storage_parent);

    let generator_parent = map.add_child_node(rooms_parent, MapNode::entity_named("GeneratorRoom"));
    add_generator_room(&mut map, generator_parent);

    let exit_parent = map.add_child_node(rooms_parent, MapNode::entity_named("ExitCorridor"));
    add_exit_corridor(&mut map, exit_parent);

    let doors_parent = map.add_child_node(rooms_parent, MapNode::entity_named("Doors"));
    add_doors(&mut map, doors_parent);

    let interactables_parent =
        map.add_child_node(rooms_parent, MapNode::entity_named("Interactables"));
    add_levers(&mut map, interactables_parent);

    let lights_parent = map.add_child_node(rooms_parent, MapNode::entity_named("Lights"));
    add_overhead_lights(&mut map, lights_parent);

    let audio_parent = map.add_child_node(rooms_parent, MapNode::entity_named("Audio"));
    add_audio_sources(&mut map, audio_parent);

    map
}
