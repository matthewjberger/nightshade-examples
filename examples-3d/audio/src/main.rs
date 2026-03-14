use nightshade::ecs::audio::systems::load_sound_from_bytes;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(AudioDemo::default())?;
    Ok(())
}

#[derive(Default)]
struct AudioDemo {
    sound_entities: Vec<Entity>,
    music_entity: Option<Entity>,
    instructions_text: Option<Entity>,
}

impl State for AudioDemo {
    fn title(&self) -> &str {
        "Audio Demo - Nightshade"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;

        self.load_sounds(world);

        let camera_position = Vec3::new(0.0, 4.0, 10.0);
        let main_camera = nightshade::ecs::camera::spawn_camera(
            world,
            camera_position,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(main_camera);

        world.core.add_components(main_camera, AUDIO_LISTENER);
        world.core.set_audio_listener(main_camera, AudioListener);

        spawn_sun(world);

        let ground_plane = spawn_plane_at(world, Vec3::new(0.0, 0.1, 0.0));
        if let Some(transform) = world.core.get_local_transform_mut(ground_plane) {
            transform.scale = Vec3::new(50.0, 1.0, 50.0);
        }

        self.create_sound_spheres(world);

        self.create_instructions_ui(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        nightshade::ecs::transform::systems::run_systems(world);
        fly_camera_system(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if key_state != KeyState::Pressed {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            if !world.resources.audio.is_initialized() {
                use nightshade::ecs::audio::systems::lazy_initialize_audio_system;
                lazy_initialize_audio_system(world);
            }
        }

        match key_code {
            KeyCode::Digit1 => {
                if let Some(entity) = self.sound_entities.first() {
                    self.toggle_sound(world, *entity);
                }
            }
            KeyCode::Digit2 => {
                if let Some(entity) = self.sound_entities.get(1) {
                    self.toggle_sound(world, *entity);
                }
            }
            KeyCode::Digit3 => {
                if let Some(entity) = self.sound_entities.get(2) {
                    self.toggle_sound(world, *entity);
                }
            }
            KeyCode::Digit4 => {
                if let Some(entity) = self.sound_entities.get(3) {
                    self.toggle_sound(world, *entity);
                }
            }
            KeyCode::KeyM => {
                if let Some(music_entity) = self.music_entity {
                    self.toggle_sound(world, music_entity);
                }
            }
            _ => {}
        }
    }
}

impl AudioDemo {
    fn load_sounds(&self, world: &mut World) {
        let sounds = [
            (
                "loop1",
                include_bytes!("../../../assets/audio/loop1.ogg").as_slice(),
            ),
            (
                "loop2",
                include_bytes!("../../../assets/audio/loop2.ogg").as_slice(),
            ),
            (
                "loop3",
                include_bytes!("../../../assets/audio/loop3.ogg").as_slice(),
            ),
            (
                "loop4",
                include_bytes!("../../../assets/audio/loop4.ogg").as_slice(),
            ),
            (
                "music",
                include_bytes!("../../../assets/audio/background_music.ogg").as_slice(),
            ),
        ];

        for (name, bytes) in sounds {
            match load_sound_from_bytes(bytes) {
                Ok(data) => {
                    world.resources.audio.load_sound(name, data);
                }
                Err(error) => {
                    tracing::error!("Failed to load sound '{}': {}", name, error);
                }
            }
        }
    }

    fn create_sound_spheres(&mut self, world: &mut World) {
        let positions = [
            Vec3::new(-30.0, 1.5, -15.0),
            Vec3::new(-15.0, 1.5, 20.0),
            Vec3::new(20.0, 1.5, -15.0),
            Vec3::new(30.0, 1.5, 20.0),
        ];

        let colors = [
            [1.0, 0.2, 0.2, 1.0],
            [0.2, 1.0, 0.2, 1.0],
            [0.2, 0.2, 1.0, 1.0],
            [1.0, 1.0, 0.2, 1.0],
        ];

        let sound_names = ["loop1", "loop2", "loop3", "loop4"];

        for index in 0..4 {
            let entity = spawn_sphere_at(world, positions[index]);

            let material_name = format!("SoundSphere_{}_{}", index, entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: colors[index],
                    ..Default::default()
                },
            );
            if let Some(&idx) = world
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
                    .add_reference(idx);
            }
            world.core.set_material_ref(entity, MaterialRef::new(material_name));

            world.core.add_components(entity, AUDIO_SOURCE);
            world.core.set_audio_source(
                entity,
                AudioSource::new(sound_names[index])
                    .with_looping(true)
                    .with_spatial(true)
                    .with_reverb(index == 0)
                    .playing(),
            );

            self.sound_entities.push(entity);
        }

        self.create_cave_walls(world, positions[0]);

        let music_entity = world.spawn_entities(AUDIO_SOURCE, 1)[0];
        world.core.set_audio_source(
            music_entity,
            AudioSource::new("music")
                .with_volume(0.7)
                .with_looping(true),
        );
        self.music_entity = Some(music_entity);
    }

    fn toggle_sound(&self, world: &mut World, entity: Entity) {
        if let Some(source) = world.core.get_audio_source_mut(entity) {
            source.playing = !source.playing;
            tracing::info!(
                "Toggled sound for entity {:?}: playing = {}",
                entity,
                source.playing
            );
        }
    }

    fn create_instructions_ui(&mut self, world: &mut World) {
        let instructions = "Spatial Audio Demo:\n\
            1, 2, 3, 4 - Toggle sphere sounds\n\
            M - Toggle music\n\
            WASD - Move camera\n\
            Mouse - Look around\n\
            (Red sphere has reverb like a cave)";

        let text_entity = spawn_ui_text_with_properties(
            world,
            instructions,
            Vec2::zeros(),
            TextProperties {
                font_size: 20.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                ..Default::default()
            },
        );

        self.instructions_text = Some(text_entity);
    }

    fn create_cave_walls(&self, world: &mut World, center: Vec3) {
        let wall_height = 5.0;
        let wall_thickness = 0.5;
        let cave_size = 8.0;

        let north_wall = spawn_cube_at(
            world,
            Vec3::new(center.x, wall_height / 2.0, center.z - cave_size),
        );
        if let Some(transform) = world.core.get_local_transform_mut(north_wall) {
            transform.scale = Vec3::new(cave_size * 2.0, wall_height, wall_thickness);
        }

        let south_wall = spawn_cube_at(
            world,
            Vec3::new(center.x, wall_height / 2.0, center.z + cave_size),
        );
        if let Some(transform) = world.core.get_local_transform_mut(south_wall) {
            transform.scale = Vec3::new(cave_size * 2.0, wall_height, wall_thickness);
        }

        let east_wall = spawn_cube_at(
            world,
            Vec3::new(center.x + cave_size, wall_height / 2.0, center.z),
        );
        if let Some(transform) = world.core.get_local_transform_mut(east_wall) {
            transform.scale = Vec3::new(wall_thickness, wall_height, cave_size * 2.0);
        }

        let west_wall = spawn_cube_at(
            world,
            Vec3::new(center.x - cave_size, wall_height / 2.0, center.z),
        );
        if let Some(transform) = world.core.get_local_transform_mut(west_wall) {
            transform.scale = Vec3::new(wall_thickness, wall_height, cave_size * 2.0);
        }

        for wall in [north_wall, south_wall, east_wall, west_wall] {
            let material_name = format!("CaveWall_{}", wall.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.3, 0.3, 0.3, 1.0],
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
            world.core.set_material_ref(wall, MaterialRef::new(material_name));
        }
    }
}
