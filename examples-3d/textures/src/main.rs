use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::texture_cache_add_reference;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(TexturesDemo)?;
    Ok(())
}

struct TexturesDemo;

impl State for TexturesDemo {
    fn title(&self) -> &str {
        "Textures Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;

        load_procedural_textures(world);

        spawn_sun_without_shadows(world);

        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 2.5, 0.0),
            10.0,
            0.0,
            0.3,
            "Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);

        spawn_textured_cube(world, Vec3::new(-3.0, 2.5, 0.0), "checkerboard");
        spawn_textured_sphere(world, Vec3::new(0.0, 2.5, 0.0), "gradient");
        spawn_textured_plane(world, Vec3::new(3.0, 2.5, 0.0), "uv_test");
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("Textures Demo")
            .default_pos([10.0, 10.0])
            .show(ui_context, |ui| {
                ui.label("Texture System Demo");
                ui.separator();
                ui.label("Procedural textures:");
                ui.label("  Left: Checkerboard texture");
                ui.label("  Center: Gradient texture");
                ui.label("  Right: UV test texture (plane)");
            });
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);
        rotation_system(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if matches!((key_code, key_state), (KeyCode::KeyQ, KeyState::Pressed)) {
            world.resources.window.should_exit = true;
        }
    }
}

fn spawn_textured_cube(world: &mut World, position: Vec3, texture_name: &str) {
    let entity = world.spawn_entities(
        RENDER_MESH
            | MATERIAL_REF
            | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | NAME,
        1,
    )[0];

    world.core.set_render_mesh(entity, RenderMesh::new("Cube"));

    let material_name = format!("TexturedCube_{}_{}", texture_name, entity.id);
    texture_cache_add_reference(&mut world.resources.texture_cache, texture_name);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(texture_name.to_string()),
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
    world.core.set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    if let Some(name) = world.core.get_name_mut(entity) {
        *name = Name(format!("Textured Cube ({})", texture_name));
    }
}

fn spawn_textured_sphere(world: &mut World, position: Vec3, texture_name: &str) {
    let entity = world.spawn_entities(
        RENDER_MESH
            | MATERIAL_REF
            | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | NAME,
        1,
    )[0];

    world.core.set_render_mesh(entity, RenderMesh::new("Sphere"));

    let material_name = format!("TexturedSphere_{}_{}", texture_name, entity.id);
    texture_cache_add_reference(&mut world.resources.texture_cache, texture_name);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(texture_name.to_string()),
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
    world.core.set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    if let Some(name) = world.core.get_name_mut(entity) {
        *name = Name(format!("Textured Sphere ({})", texture_name));
    }
}

fn spawn_textured_plane(world: &mut World, position: Vec3, texture_name: &str) {
    let entity = world.spawn_entities(
        RENDER_MESH
            | MATERIAL_REF
            | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | NAME
            | ROTATION,
        1,
    )[0];

    world.core.set_render_mesh(entity, RenderMesh::new("Plane"));

    let material_name = format!("TexturedPlane_{}_{}", texture_name, entity.id);
    texture_cache_add_reference(&mut world.resources.texture_cache, texture_name);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(texture_name.to_string()),
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
    world.core.set_material_ref(entity, MaterialRef::new(material_name));

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
    }

    if let Some(rotation) = world.core.get_rotation_mut(entity) {
        rotation.axis = Vec3::y();
        rotation.speed = 0.5;
    }

    if let Some(name) = world.core.get_name_mut(entity) {
        *name = Name(format!("Textured Plane ({})", texture_name));
    }
}

fn rotation_system(world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time;

    let entities: Vec<_> = world.core.query_entities(ROTATION).collect();

    for entity in entities {
        if let Some(rotation) = world.core.get_rotation(entity) {
            let axis = rotation.axis;
            let speed = rotation.speed;

            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                let rotation_delta = nalgebra_glm::quat_angle_axis(speed * delta_time, &axis);
                transform.rotation = rotation_delta * transform.rotation;
            }
        }
    }
}
