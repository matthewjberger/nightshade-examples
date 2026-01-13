use nightshade::prelude::*;

const HDR_BYTES: &[u8] = include_bytes!("../../../assets/sky/moonrise.hdr");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SkyboxDemo::default())?;
    Ok(())
}

#[derive(Default)]
struct SkyboxDemo {
    camera_entity: Option<Entity>,
}

impl State for SkyboxDemo {
    fn title(&self) -> &str {
        "HDR Skybox Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.atmosphere = Atmosphere::Hdr;

        load_hdr_skybox(world, HDR_BYTES.to_vec());

        let camera_pos = nalgebra_glm::vec3(0.0, 0.0, 0.0);
        let camera_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | CAMERA,
            1,
        )[0];

        world.set_name(camera_entity, Name("Main Camera".to_string()));
        world.set_local_transform(
            camera_entity,
            LocalTransform {
                translation: camera_pos,
                rotation: nalgebra_glm::Quat::identity(),
                ..Default::default()
            },
        );
        world.set_global_transform(camera_entity, GlobalTransform::default());
        world.set_local_transform_dirty(camera_entity, LocalTransformDirty);
        world.set_camera(
            camera_entity,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 45.0_f32.to_radians(),
                    z_far: None,
                    z_near: 0.01,
                }),
                smoothing: Some(Smoothing::default()),
            },
        );

        self.camera_entity = Some(camera_entity);
        world.resources.active_camera = Some(camera_entity);
    }

    fn run_systems(&mut self, world: &mut World) {
        fly_camera_system(world);
    }

    fn ui(&mut self, _world: &mut World, _ui_context: &egui::Context) {}
}
