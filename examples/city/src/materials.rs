use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::world::components::BoundingVolume;
use nightshade::prelude::*;

pub fn spawn_city_mesh(world: &mut World, mesh_name: &str, position: Vec3, scale: Vec3) -> Entity {
    let entity = world.spawn_entities(
        LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | VISIBILITY,
        1,
    )[0];

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = scale;
    }

    world.set_render_mesh(entity, RenderMesh::new(mesh_name));
    mark_local_transform_dirty(world, entity);
    world.resources.mesh_render_state.mark_entity_added(entity);

    if let Some(bounding_volume) = world.get_bounding_volume_mut(entity) {
        *bounding_volume = BoundingVolume::from_mesh_type(mesh_name);
    }

    entity
}

pub fn spawn_point_light(
    world: &mut World,
    position: Vec3,
    color: Vec3,
    intensity: f32,
    range: f32,
) -> Entity {
    let entity = world.spawn_entities(
        LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | LIGHT,
        1,
    )[0];

    if let Some(transform) = world.get_local_transform_mut(entity) {
        transform.translation = position;
    }
    mark_local_transform_dirty(world, entity);

    if let Some(light) = world.get_light_mut(entity) {
        *light = Light {
            light_type: LightType::Point,
            color,
            intensity,
            range,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: false,
            shadow_bias: 0.0,
        };
    }

    entity
}

pub fn create_materials(world: &mut World) {
    let materials: &[(&str, [f32; 4], f32, f32)] = &[
        ("Ground", [0.25, 0.25, 0.22, 1.0], 0.95, 0.0),
        ("Road", [0.15, 0.15, 0.15, 1.0], 0.85, 0.0),
        ("Sidewalk", [0.55, 0.53, 0.50, 1.0], 0.90, 0.0),
        ("ConcreteLight", [0.72, 0.70, 0.67, 1.0], 0.80, 0.0),
        ("ConcreteMedium", [0.55, 0.53, 0.50, 1.0], 0.82, 0.0),
        ("ConcreteDark", [0.38, 0.36, 0.34, 1.0], 0.85, 0.0),
        ("GlassBlue", [0.35, 0.55, 0.75, 1.0], 0.15, 0.6),
        ("GlassTeal", [0.30, 0.60, 0.65, 1.0], 0.12, 0.65),
        ("GlassDark", [0.20, 0.25, 0.35, 1.0], 0.10, 0.7),
        ("BrickRed", [0.55, 0.22, 0.18, 1.0], 0.88, 0.0),
        ("BrickBrown", [0.50, 0.35, 0.22, 1.0], 0.85, 0.0),
        ("BrickTan", [0.65, 0.55, 0.40, 1.0], 0.82, 0.0),
        ("ModernWhite", [0.88, 0.87, 0.85, 1.0], 0.70, 0.05),
        ("ModernCream", [0.85, 0.80, 0.70, 1.0], 0.72, 0.03),
        ("RooftopGrey", [0.40, 0.40, 0.42, 1.0], 0.75, 0.2),
        ("RooftopMetal", [0.50, 0.52, 0.55, 1.0], 0.45, 0.6),
        ("RooftopRed", [0.60, 0.20, 0.15, 1.0], 0.80, 0.1),
        ("WindowDark", [0.15, 0.18, 0.25, 1.0], 0.20, 0.5),
        ("ParkGreen", [0.20, 0.50, 0.15, 1.0], 0.90, 0.0),
        ("ParkDarkGreen", [0.12, 0.35, 0.10, 1.0], 0.88, 0.0),
        ("TreeTrunk", [0.40, 0.28, 0.18, 1.0], 0.85, 0.0),
        ("Silhouette", [0.35, 0.38, 0.42, 1.0], 0.90, 0.0),
        ("Antenna", [0.60, 0.60, 0.65, 1.0], 0.30, 0.7),
        ("DockConcrete", [0.65, 0.63, 0.60, 1.0], 0.85, 0.0),
        ("DockWood", [0.45, 0.32, 0.20, 1.0], 0.90, 0.0),
        ("DockMetal", [0.35, 0.35, 0.38, 1.0], 0.40, 0.7),
        ("LampPole", [0.25, 0.25, 0.28, 1.0], 0.60, 0.7),
        ("BoatHull", [0.30, 0.30, 0.35, 1.0], 0.70, 0.3),
        ("BoatCabin", [0.80, 0.75, 0.65, 1.0], 0.85, 0.0),
        ("BridgeMetal", [0.45, 0.45, 0.50, 1.0], 0.50, 0.6),
        ("BridgeConcrete", [0.60, 0.58, 0.55, 1.0], 0.85, 0.0),
        ("CarRed", [0.70, 0.12, 0.10, 1.0], 0.35, 0.4),
        ("CarBlue", [0.10, 0.20, 0.65, 1.0], 0.35, 0.4),
        ("CarWhite", [0.90, 0.90, 0.88, 1.0], 0.40, 0.3),
        ("CarBlack", [0.08, 0.08, 0.10, 1.0], 0.30, 0.5),
        ("CarSilver", [0.70, 0.70, 0.72, 1.0], 0.25, 0.7),
        ("CraneMetal", [0.85, 0.65, 0.10, 1.0], 0.60, 0.4),
        ("RoadMarking", [0.95, 0.95, 0.90, 1.0], 0.80, 0.0),
        ("FlowerRed", [0.75, 0.15, 0.20, 1.0], 0.90, 0.0),
        ("FlowerYellow", [0.90, 0.80, 0.15, 1.0], 0.90, 0.0),
    ];

    for &(name, color, roughness, metallic) in materials {
        material_registry_insert(
            &mut world.resources.material_registry,
            name.to_string(),
            Material {
                base_color: color,
                roughness,
                metallic,
                ..Default::default()
            },
        );
    }

    struct EmissiveDef {
        name: &'static str,
        base_color: [f32; 4],
        emissive_factor: [f32; 3],
        emissive_strength: f32,
        roughness: f32,
    }

    let emissive_materials = [
        EmissiveDef {
            name: "LampGlow",
            base_color: [1.0, 0.9, 0.6, 1.0],
            emissive_factor: [2.0, 1.5, 0.8],
            emissive_strength: 3.0,
            roughness: 0.2,
        },
        EmissiveDef {
            name: "NeonRed",
            base_color: [1.0, 0.15, 0.1, 1.0],
            emissive_factor: [3.0, 0.3, 0.2],
            emissive_strength: 2.0,
            roughness: 0.2,
        },
        EmissiveDef {
            name: "NeonBlue",
            base_color: [0.1, 0.3, 1.0, 1.0],
            emissive_factor: [0.2, 0.6, 3.0],
            emissive_strength: 2.0,
            roughness: 0.2,
        },
        EmissiveDef {
            name: "NeonPink",
            base_color: [1.0, 0.2, 0.6, 1.0],
            emissive_factor: [3.0, 0.4, 1.2],
            emissive_strength: 2.0,
            roughness: 0.2,
        },
        EmissiveDef {
            name: "WindowLit",
            base_color: [0.90, 0.85, 0.55, 1.0],
            emissive_factor: [1.5, 1.3, 0.6],
            emissive_strength: 1.0,
            roughness: 0.3,
        },
        EmissiveDef {
            name: "ShopfrontLit",
            base_color: [0.85, 0.80, 0.60, 1.0],
            emissive_factor: [1.2, 1.0, 0.5],
            emissive_strength: 0.8,
            roughness: 0.3,
        },
        EmissiveDef {
            name: "BillboardWhite",
            base_color: [1.0, 1.0, 0.95, 1.0],
            emissive_factor: [2.0, 2.0, 1.8],
            emissive_strength: 1.5,
            roughness: 0.3,
        },
        EmissiveDef {
            name: "BillboardYellow",
            base_color: [1.0, 0.9, 0.3, 1.0],
            emissive_factor: [2.5, 2.0, 0.5],
            emissive_strength: 1.5,
            roughness: 0.3,
        },
    ];

    for emissive in &emissive_materials {
        let EmissiveDef {
            name,
            base_color,
            emissive_factor,
            emissive_strength,
            roughness,
        } = *emissive;
        material_registry_insert(
            &mut world.resources.material_registry,
            name.to_string(),
            Material {
                base_color,
                emissive_factor,
                emissive_strength,
                roughness,
                ..Default::default()
            },
        );
    }
}

pub fn apply_material(world: &mut World, entity: Entity, name: &str) {
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(entity, MaterialRef::new(name.to_string()));
}
