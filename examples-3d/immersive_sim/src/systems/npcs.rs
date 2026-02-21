use crate::data::npcs::NPC_DEFINITIONS;
use crate::state::ImmersiveSim;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

const NPC_HEIGHT: f32 = 1.8;
const NPC_RADIUS: f32 = 0.4;
const NPC_INTERACT_RANGE: f32 = 3.0;

fn spawn_mesh(world: &mut World, mesh_name: &str, position: Vec3, scale: Vec3) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | VISIBILITY,
        1,
    )[0];

    if let Some(name) = world.get_name_mut(entity) {
        name.0 = format!("NPC_{}", entity.id);
    }

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    if let Some(mesh) = world.get_render_mesh_mut(entity) {
        mesh.name = mesh_name.to_string();
    }

    if let Some(bounding_volume) = world.get_bounding_volume_mut(entity) {
        *bounding_volume =
            nightshade::ecs::world::components::BoundingVolume::from_mesh_type(mesh_name);
    }

    entity
}

fn mark_local_transform_dirty(world: &mut World, entity: Entity) {
    world.set_local_transform_dirty(entity, LocalTransformDirty);
}

pub fn sample_lowest_navmesh_height(world: &World, x: f32, z: f32) -> Option<f32> {
    let navmesh = &world.resources.navmesh;
    let mut lowest_y: Option<f32> = None;

    for triangle in &navmesh.triangles {
        let v0 = navmesh.vertices[triangle.vertex_indices[0]];
        let v1 = navmesh.vertices[triangle.vertex_indices[1]];
        let v2 = navmesh.vertices[triangle.vertex_indices[2]];

        let p0 = Vec2::new(v0.x, v0.z);
        let p1 = Vec2::new(v1.x, v1.z);
        let p2 = Vec2::new(v2.x, v2.z);
        let point = Vec2::new(x, z);

        if point_in_triangle_2d(point, p0, p1, p2) {
            let y = interpolate_height(x, z, v0, v1, v2);
            match lowest_y {
                None => lowest_y = Some(y),
                Some(current) if y < current => lowest_y = Some(y),
                _ => {}
            }
        }
    }

    lowest_y
}

fn point_in_triangle_2d(p: Vec2, v0: Vec2, v1: Vec2, v2: Vec2) -> bool {
    let d00 = v1 - v0;
    let d01 = v2 - v0;
    let d02 = p - v0;

    let dot00 = nalgebra_glm::dot(&d00, &d00);
    let dot01 = nalgebra_glm::dot(&d00, &d01);
    let dot02 = nalgebra_glm::dot(&d00, &d02);
    let dot11 = nalgebra_glm::dot(&d01, &d01);
    let dot12 = nalgebra_glm::dot(&d01, &d02);

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    u >= 0.0 && v >= 0.0 && (u + v) <= 1.0
}

fn interpolate_height(x: f32, z: f32, v0: Vec3, v1: Vec3, v2: Vec3) -> f32 {
    let d00 = Vec2::new(v1.x - v0.x, v1.z - v0.z);
    let d01 = Vec2::new(v2.x - v0.x, v2.z - v0.z);
    let d02 = Vec2::new(x - v0.x, z - v0.z);

    let dot00 = nalgebra_glm::dot(&d00, &d00);
    let dot01 = nalgebra_glm::dot(&d00, &d01);
    let dot02 = nalgebra_glm::dot(&d00, &d02);
    let dot11 = nalgebra_glm::dot(&d01, &d01);
    let dot12 = nalgebra_glm::dot(&d01, &d02);

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    v0.y * (1.0 - u - v) + v1.y * u + v2.y * v
}

pub fn spawn_npcs(game: &mut ImmersiveSim, world: &mut World) {
    for (index, npc_def) in NPC_DEFINITIONS.iter().enumerate() {
        let floor_y = sample_lowest_navmesh_height(world, npc_def.position.x, npc_def.position.z)
            .unwrap_or(npc_def.position.y);
        let visual_y = floor_y + NPC_HEIGHT / 2.0;

        let visual = spawn_mesh(
            world,
            "Cylinder",
            Vec3::new(npc_def.position.x, visual_y, npc_def.position.z),
            Vec3::new(NPC_RADIUS * 2.0, NPC_HEIGHT, NPC_RADIUS * 2.0),
        );
        world.set_casts_shadow(visual, CastsShadow);
        mark_local_transform_dirty(world, visual);

        let mat_name = format!("Npc_{}", index);
        material_registry_insert(
            &mut world.resources.material_registry,
            mat_name.clone(),
            Material {
                base_color: npc_def.color,
                roughness: 0.6,
                metallic: 0.1,
                ..Default::default()
            },
        );
        if let Some(&mat_index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&mat_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(mat_index);
        }
        world.set_material_ref(visual, MaterialRef::new(mat_name));

        game.npc_entities.push(visual);
    }
}

pub fn get_looked_at_npc(game: &ImmersiveSim, world: &World) -> Option<usize> {
    let camera_entity = game.camera_entity?;

    let camera_pos = world
        .get_global_transform(camera_entity)
        .map(|t| t.translation())?;

    let camera_forward = world
        .get_global_transform(camera_entity)
        .map(|t| t.forward_vector())?;

    let mut looked_at_npc: Option<usize> = None;
    let mut closest_dot = -1.0_f32;

    for (index, &npc_entity) in game.npc_entities.iter().enumerate() {
        let npc_pos = world
            .get_local_transform(npc_entity)
            .map(|t| t.translation)
            .unwrap_or(Vec3::zeros());

        let to_npc = npc_pos - camera_pos;
        let distance = nalgebra_glm::length(&to_npc);

        if distance > NPC_INTERACT_RANGE {
            continue;
        }

        let to_npc_normalized = nalgebra_glm::normalize(&to_npc);
        let dot = nalgebra_glm::dot(&camera_forward, &to_npc_normalized);

        if dot > 0.9 && dot > closest_dot {
            closest_dot = dot;
            looked_at_npc = Some(index);
        }
    }

    looked_at_npc
}
