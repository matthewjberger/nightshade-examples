use nalgebra_glm::{Vec3, vec3};
use nightshade::ecs::material::components::Material;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::ecs::mesh::components::create_plane_mesh;
use nightshade::ecs::prefab::resources::mesh_cache_insert;
use nightshade::ecs::water::{VolumeFlowType, VolumeShape, Water};
use nightshade::ecs::world::WATER;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(WaterDemo::default())?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScenePreset {
    InfiniteOcean,
    TropicalIsland,
    ForestPond,
    RiverValley,
    MountainWaterfall,
    CanyonWaterfall,
}

impl ScenePreset {
    const ALL: &'static [ScenePreset] = &[
        ScenePreset::InfiniteOcean,
        ScenePreset::TropicalIsland,
        ScenePreset::ForestPond,
        ScenePreset::RiverValley,
        ScenePreset::MountainWaterfall,
        ScenePreset::CanyonWaterfall,
    ];

    fn name(&self) -> &'static str {
        match self {
            ScenePreset::InfiniteOcean => "Infinite Ocean",
            ScenePreset::TropicalIsland => "Tropical Island",
            ScenePreset::ForestPond => "Forest Pond",
            ScenePreset::RiverValley => "River Valley",
            ScenePreset::MountainWaterfall => "Mountain Waterfall",
            ScenePreset::CanyonWaterfall => "Canyon Waterfall",
        }
    }
}

struct WaterDemo {
    current_preset: ScenePreset,
    scene_entities: Vec<Entity>,
    camera_entity: Option<Entity>,
}

impl Default for WaterDemo {
    fn default() -> Self {
        Self {
            current_preset: ScenePreset::InfiniteOcean,
            scene_entities: Vec::new(),
            camera_entity: None,
        }
    }
}

impl State for WaterDemo {
    fn title(&self) -> &str {
        "Water Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::CloudySky;
        world.resources.graphics.bloom_enabled = false;

        self.camera_entity = Some(spawn_camera(world));
        world.resources.active_camera = self.camera_entity;

        spawn_sun(world);
        register_meshes(world);
        register_materials(world);

        self.load_preset(world, ScenePreset::InfiniteOcean);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if self.current_preset == ScenePreset::InfiniteOcean {
            update_orbiting_camera(world, self.camera_entity);
        } else {
            nightshade::ecs::camera::systems::pan_orbit_camera_system(world);
        }
    }

    fn ui(&mut self, world: &mut World, ui_context: &egui::Context) {
        let mut new_preset = None;

        egui::Window::new("Water Demo")
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
            .resizable(false)
            .collapsible(true)
            .show(ui_context, |ui| {
                let fps = world.resources.window.timing.frames_per_second;
                let fps_color = if fps >= 55.0 {
                    egui::Color32::GREEN
                } else if fps >= 30.0 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::from_rgb(255, 80, 0)
                };
                ui.colored_label(fps_color, format!("FPS: {:.0}", fps));

                ui.separator();

                for preset in ScenePreset::ALL {
                    let is_selected = self.current_preset == *preset;
                    if ui.selectable_label(is_selected, preset.name()).clicked() && !is_selected {
                        new_preset = Some(*preset);
                    }
                }
            });

        if let Some(preset) = new_preset {
            self.load_preset(world, preset);
        }
    }
}

impl WaterDemo {
    fn load_preset(&mut self, world: &mut World, preset: ScenePreset) {
        for entity in self.scene_entities.drain(..) {
            despawn_recursive_immediate(world, entity);
        }

        self.current_preset = preset;

        match preset {
            ScenePreset::InfiniteOcean => self.build_infinite_ocean(world),
            ScenePreset::TropicalIsland => self.build_tropical_island(world),
            ScenePreset::ForestPond => self.build_forest_pond(world),
            ScenePreset::RiverValley => self.build_river_valley(world),
            ScenePreset::MountainWaterfall => self.build_mountain_waterfall(world),
            ScenePreset::CanyonWaterfall => self.build_canyon_waterfall(world),
        }
    }

    fn build_infinite_ocean(&mut self, world: &mut World) {
        self.scene_entities.push(spawn_infinite_ocean(world));

        set_camera_orbit(world, self.camera_entity, vec3(0.0, 0.0, 0.0), 50.0);
    }

    fn build_tropical_island(&mut self, world: &mut World) {
        self.scene_entities.push(spawn_infinite_ocean(world));

        self.scene_entities.push(spawn_terrain(
            world,
            "Island",
            vec3(0.0, 1.5, 0.0),
            vec3(40.0, 4.0, 30.0),
            "grass",
        ));
        self.scene_entities.push(spawn_terrain(
            world,
            "Hill",
            vec3(-8.0, 4.0, -5.0),
            vec3(15.0, 6.0, 12.0),
            "grass",
        ));
        self.scene_entities.push(spawn_terrain(
            world,
            "Peak",
            vec3(-10.0, 7.0, -6.0),
            vec3(6.0, 4.0, 5.0),
            "rock",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Beach N",
            vec3(0.0, 0.0, -17.0),
            vec3(35.0, 1.0, 5.0),
            "sand",
        ));
        self.scene_entities.push(spawn_terrain(
            world,
            "Beach S",
            vec3(0.0, 0.0, 17.0),
            vec3(35.0, 1.0, 5.0),
            "sand",
        ));
        self.scene_entities.push(spawn_terrain(
            world,
            "Beach E",
            vec3(22.0, 0.0, 0.0),
            vec3(5.0, 1.0, 25.0),
            "sand",
        ));
        self.scene_entities.push(spawn_terrain(
            world,
            "Beach W",
            vec3(-22.0, 0.0, 0.0),
            vec3(5.0, 1.0, 25.0),
            "sand",
        ));

        for index in 0..5 {
            let angle = index as f32 * std::f32::consts::TAU / 5.0 + 0.3;
            let radius = 60.0 + (index as f32 * 7.0) % 15.0;
            self.scene_entities.push(spawn_terrain(
                world,
                &format!("Rock {}", index),
                vec3(angle.cos() * radius, 0.5, angle.sin() * radius),
                vec3(4.0 + index as f32, 3.0, 3.0 + index as f32 * 0.5),
                "rock",
            ));
        }

        set_camera_orbit(world, self.camera_entity, vec3(0.0, 5.0, 0.0), 80.0);
    }

    fn build_forest_pond(&mut self, world: &mut World) {
        self.scene_entities.push(spawn_water(
            world,
            "Pond",
            "water_plane_medium",
            vec3(0.0, 0.0, 0.0),
            vec3(1.0, 1.0, 1.0),
            Water {
                base_color: [0.02, 0.08, 0.1, 1.0],
                water_color: [0.15, 0.3, 0.25, 1.0],
                wave_height: 0.03,
                choppy: 1.0,
                speed: 0.1,
                frequency: 0.5,
                ..Default::default()
            },
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Clearing",
            vec3(0.0, -0.3, 0.0),
            vec3(50.0, 0.3, 50.0),
            "grass",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Shore Edge",
            vec3(0.0, -0.15, 0.0),
            vec3(22.0, 0.2, 22.0),
            "dirt",
        ));

        for index in 0..12 {
            let angle = index as f32 * std::f32::consts::TAU / 12.0;
            let radius = 18.0 + (index as f32 * 2.0) % 5.0;
            let height = 8.0 + (index as f32 * 3.0) % 6.0;
            self.scene_entities.push(spawn_terrain(
                world,
                &format!("Tree {}", index),
                vec3(angle.cos() * radius, height / 2.0, angle.sin() * radius),
                vec3(1.5, height, 1.5),
                "grass",
            ));
        }

        for index in 0..6 {
            let angle = index as f32 * std::f32::consts::TAU / 6.0 + 0.5;
            let radius = 8.0 + (index as f32 * 1.5) % 3.0;
            self.scene_entities.push(spawn_terrain(
                world,
                &format!("Stone {}", index),
                vec3(angle.cos() * radius, -0.1, angle.sin() * radius),
                vec3(0.8 + index as f32 * 0.1, 0.4, 0.7 + index as f32 * 0.1),
                "rock",
            ));
        }

        set_camera_orbit(world, self.camera_entity, vec3(0.0, 2.0, 0.0), 40.0);
    }

    fn build_river_valley(&mut self, world: &mut World) {
        self.scene_entities.push(spawn_water(
            world,
            "River",
            "water_plane_river",
            vec3(0.0, -0.3, 0.0),
            vec3(0.7, 1.0, 1.0),
            Water {
                base_color: [0.02, 0.1, 0.12, 1.0],
                water_color: [0.25, 0.45, 0.35, 1.0],
                wave_height: 0.15,
                choppy: 2.0,
                speed: 1.2,
                frequency: 0.4,
                flow_direction: [0.0, 1.0],
                flow_strength: 1.0,
                ..Default::default()
            },
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Riverbed",
            vec3(0.0, -1.5, 0.0),
            vec3(10.0, 2.0, 200.0),
            "dirt",
        ));

        for index in 0..20 {
            let z = (index as f32 - 10.0) * 10.0;
            let wobble = ((index as f32) * 0.7).sin() * 0.5;
            self.scene_entities.push(spawn_terrain(
                world,
                &format!("Bank L {}", index),
                vec3(-5.0 + wobble, 0.3, z),
                vec3(3.0, 1.2, 12.0),
                "dirt",
            ));
            self.scene_entities.push(spawn_terrain(
                world,
                &format!("Bank R {}", index),
                vec3(5.0 - wobble, 0.3, z),
                vec3(3.0, 1.2, 12.0),
                "dirt",
            ));
        }

        self.scene_entities.push(spawn_terrain(
            world,
            "Valley Floor L",
            vec3(-18.0, -0.1, 0.0),
            vec3(25.0, 0.4, 200.0),
            "grass",
        ));
        self.scene_entities.push(spawn_terrain(
            world,
            "Valley Floor R",
            vec3(18.0, -0.1, 0.0),
            vec3(25.0, 0.4, 200.0),
            "grass",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Hill L1",
            vec3(-38.0, 4.0, -30.0),
            vec3(20.0, 12.0, 50.0),
            "grass",
        ));
        self.scene_entities.push(spawn_terrain(
            world,
            "Hill L2",
            vec3(-42.0, 6.0, 40.0),
            vec3(25.0, 16.0, 60.0),
            "rock",
        ));
        self.scene_entities.push(spawn_terrain(
            world,
            "Hill R1",
            vec3(38.0, 5.0, 10.0),
            vec3(20.0, 14.0, 70.0),
            "grass",
        ));
        self.scene_entities.push(spawn_terrain(
            world,
            "Hill R2",
            vec3(45.0, 7.0, -40.0),
            vec3(25.0, 18.0, 50.0),
            "rock",
        ));

        let rock_positions: [(f32, f32, f32); 8] = [
            (-3.5, -0.4, -25.0),
            (2.8, -0.35, -10.0),
            (-2.0, -0.45, 15.0),
            (3.2, -0.4, 35.0),
            (-3.0, -0.35, 55.0),
            (2.5, -0.4, -50.0),
            (-2.5, -0.38, 75.0),
            (3.0, -0.42, -70.0),
        ];
        for (index, (x, y, z)) in rock_positions.iter().enumerate() {
            let size = 0.8 + (index as f32 * 0.3) % 0.6;
            self.scene_entities.push(spawn_terrain(
                world,
                &format!("River Rock {}", index),
                vec3(*x, *y, *z),
                vec3(size, size * 0.6, size * 1.2),
                "rock",
            ));
        }

        set_camera_orbit(world, self.camera_entity, vec3(0.0, 4.0, 0.0), 45.0);
    }

    fn build_mountain_waterfall(&mut self, world: &mut World) {
        self.scene_entities.push(spawn_terrain(
            world,
            "Cliff Face",
            vec3(0.0, 3.0, -4.0),
            vec3(30.0, 30.0, 4.0),
            "rock",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Cliff Top",
            vec3(0.0, 18.0, -9.0),
            vec3(40.0, 4.0, 12.0),
            "grass",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Cliff Left",
            vec3(-12.0, 3.0, -2.0),
            vec3(8.0, 30.0, 8.0),
            "rock",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Cliff Right",
            vec3(12.0, 3.0, -2.0),
            vec3(8.0, 30.0, 8.0),
            "rock",
        ));

        let volumetric_waterfall = Water {
            base_color: [0.5, 0.6, 0.7, 1.0],
            water_color: [0.85, 0.9, 0.95, 1.0],
            wave_height: 0.3,
            choppy: 4.0,
            speed: 3.0,
            frequency: 0.8,
            specular_strength: 2.0,
            fresnel_power: 2.0,
            is_volumetric: true,
            volume_size: [6.0, 24.0, 2.0],
            ..Default::default()
        };

        self.scene_entities.push(spawn_volumetric_waterfall(
            world,
            "Volumetric Waterfall",
            vec3(0.0, 6.0, -1.0),
            volumetric_waterfall,
        ));

        self.scene_entities.push(spawn_volumetric_waterfall(
            world,
            "Splash Mist",
            vec3(0.0, -5.0, -1.0),
            Water {
                base_color: [0.75, 0.82, 0.9, 1.0],
                water_color: [0.9, 0.94, 1.0, 1.0],
                wave_height: 0.1,
                choppy: 1.0,
                speed: 1.2,
                frequency: 0.8,
                specular_strength: 0.2,
                fresnel_power: 1.0,
                is_volumetric: true,
                volume_shape: VolumeShape::Box,
                volume_flow_type: VolumeFlowType::Mist,
                volume_size: [16.0, 5.0, 12.0],
                ..Default::default()
            },
        ));

        self.scene_entities.push(spawn_water(
            world,
            "Pool",
            "water_plane_medium",
            vec3(0.0, -6.0, 6.0),
            vec3(1.8, 1.0, 1.2),
            Water {
                base_color: [0.02, 0.1, 0.15, 1.0],
                water_color: [0.2, 0.35, 0.4, 1.0],
                wave_height: 0.2,
                choppy: 4.0,
                speed: 0.8,
                frequency: 0.4,
                ..Default::default()
            },
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Pool Basin",
            vec3(0.0, -8.0, 6.0),
            vec3(40.0, 4.0, 30.0),
            "rock",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Ground Left",
            vec3(-25.0, -6.6, 5.0),
            vec3(20.0, 1.0, 40.0),
            "grass",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Ground Right",
            vec3(25.0, -6.6, 5.0),
            vec3(20.0, 1.0, 40.0),
            "grass",
        ));

        for index in 0..6 {
            let x = (index as f32 - 2.5) * 5.0;
            let z = 16.0 + (index as f32 * 2.0) % 4.0;
            self.scene_entities.push(spawn_terrain(
                world,
                &format!("Boulder {}", index),
                vec3(x, -5.5, z),
                vec3(2.0 + index as f32 * 0.3, 1.5, 2.5),
                "rock",
            ));
        }

        set_camera_orbit(world, self.camera_entity, vec3(0.0, 2.0, 8.0), 40.0);
    }

    fn build_canyon_waterfall(&mut self, world: &mut World) {
        self.scene_entities.push(spawn_terrain(
            world,
            "Canyon Wall Left",
            vec3(-6.0, 5.0, 0.0),
            vec3(4.0, 35.0, 25.0),
            "rock",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Canyon Wall Right",
            vec3(6.0, 5.0, 0.0),
            vec3(4.0, 35.0, 25.0),
            "rock",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Canyon Back Wall",
            vec3(0.0, 5.0, -12.0),
            vec3(16.0, 35.0, 4.0),
            "rock",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Canyon Ledge",
            vec3(0.0, 10.0, -8.0),
            vec3(8.0, 2.0, 6.0),
            "rock",
        ));

        self.scene_entities.push(spawn_volumetric_waterfall(
            world,
            "Canyon Waterfall",
            vec3(0.0, 2.0, -4.0),
            Water {
                base_color: [0.15, 0.3, 0.45, 1.0],
                water_color: [0.4, 0.6, 0.75, 1.0],
                wave_height: 0.15,
                choppy: 2.0,
                speed: 1.5,
                frequency: 1.2,
                specular_strength: 1.5,
                fresnel_power: 3.0,
                is_volumetric: true,
                volume_shape: VolumeShape::Cylinder,
                volume_flow_type: VolumeFlowType::Cascade,
                volume_size: [7.0, 20.0, 7.0],
                ..Default::default()
            },
        ));

        self.scene_entities.push(spawn_volumetric_waterfall(
            world,
            "Canyon Mist",
            vec3(0.0, -7.0, -4.0),
            Water {
                base_color: [0.7, 0.78, 0.86, 1.0],
                water_color: [0.88, 0.92, 0.97, 1.0],
                wave_height: 0.1,
                choppy: 1.0,
                speed: 1.0,
                frequency: 0.7,
                specular_strength: 0.2,
                fresnel_power: 1.0,
                is_volumetric: true,
                volume_shape: VolumeShape::Box,
                volume_flow_type: VolumeFlowType::Mist,
                volume_size: [10.0, 4.0, 10.0],
                ..Default::default()
            },
        ));

        self.scene_entities.push(spawn_water(
            world,
            "Canyon Pool",
            "water_plane_medium",
            vec3(0.0, -8.0, 2.0),
            vec3(0.8, 1.0, 1.2),
            Water {
                base_color: [0.02, 0.08, 0.12, 1.0],
                water_color: [0.15, 0.28, 0.35, 1.0],
                wave_height: 0.15,
                choppy: 3.0,
                speed: 0.6,
                frequency: 0.5,
                ..Default::default()
            },
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Canyon Floor",
            vec3(0.0, -10.0, 2.0),
            vec3(30.0, 4.0, 35.0),
            "rock",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Canyon Rim Left",
            vec3(-15.0, -8.1, 2.0),
            vec3(15.0, 1.0, 35.0),
            "grass",
        ));

        self.scene_entities.push(spawn_terrain(
            world,
            "Canyon Rim Right",
            vec3(15.0, -8.1, 2.0),
            vec3(15.0, 1.0, 35.0),
            "grass",
        ));

        set_camera_orbit(world, self.camera_entity, vec3(0.0, -2.0, 5.0), 35.0);
    }
}

fn spawn_camera(world: &mut World) -> Entity {
    let camera = nightshade::ecs::camera::commands::spawn_pan_orbit_camera(
        world,
        Vec3::new(0.0, 5.0, 0.0),
        50.0,
        0.0,
        0.3,
        "Main Camera".to_string(),
    );

    if let Some(camera_component) = world.core.get_camera_mut(camera) {
        camera_component.projection = Projection::Perspective(PerspectiveCamera {
            aspect_ratio: None,
            y_fov_rad: 60.0_f32.to_radians(),
            z_far: Some(2000.0),
            z_near: 0.1,
        });
    }

    camera
}

fn set_camera_orbit(world: &mut World, camera_entity: Option<Entity>, focus: Vec3, radius: f32) {
    let Some(entity) = camera_entity else { return };
    if let Some(orbit) = world.core.get_pan_orbit_camera_mut(entity) {
        orbit.focus = focus;
        orbit.radius = radius;
        orbit.pitch = 0.4;
        orbit.yaw = 0.0;
    }
}

fn update_orbiting_camera(world: &mut World, camera_entity: Option<Entity>) {
    let time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
    let Some(entity) = camera_entity else { return };

    let orbit_speed = 0.08;
    let orbit_radius = 60.0;
    let height = 8.0;

    let angle = time * orbit_speed;
    let position = vec3(
        angle.cos() * orbit_radius,
        height,
        angle.sin() * orbit_radius,
    );

    let look_at = vec3(0.0, 0.0, 0.0);
    let forward = (look_at - position).normalize();
    let up = vec3(0.0, 1.0, 0.0);
    let right = nalgebra_glm::cross(&forward, &up).normalize();
    let corrected_up = nalgebra_glm::cross(&right, &forward);

    let rotation_matrix = nalgebra_glm::mat3(
        right.x,
        corrected_up.x,
        -forward.x,
        right.y,
        corrected_up.y,
        -forward.y,
        right.z,
        corrected_up.z,
        -forward.z,
    );
    let rotation = nalgebra_glm::mat3_to_quat(&rotation_matrix);

    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.rotation = rotation;
    }
    mark_local_transform_dirty(world, entity);
}

fn register_meshes(world: &mut World) {
    register_mesh(world, "water_plane_medium", create_plane_mesh(20.0));
    register_mesh(
        world,
        "water_plane_river",
        create_river_plane_mesh(12.0, 180.0),
    );
}

fn create_river_plane_mesh(width: f32, length: f32) -> nightshade::ecs::mesh::components::Mesh {
    use nightshade::ecs::mesh::components::{Mesh, Vertex};

    let half_width = width / 2.0;
    let half_length = length / 2.0;

    let vertices = vec![
        Vertex {
            position: [-half_width, 0.0, -half_length],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            tex_coords_1: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: [half_width, 0.0, -half_length],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            tex_coords_1: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: [half_width, 0.0, half_length],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 1.0],
            tex_coords_1: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: [-half_width, 0.0, half_length],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            tex_coords_1: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
    ];

    let indices = vec![0, 2, 1, 0, 3, 2];

    Mesh::new(vertices, indices)
}

fn register_mesh(world: &mut World, name: &str, mesh: nightshade::ecs::mesh::components::Mesh) {
    mesh_cache_insert(&mut world.resources.mesh_cache, name.to_string(), mesh);
}

fn register_materials(world: &mut World) {
    register_material(
        world,
        "grass",
        Material {
            base_color: [0.2, 0.45, 0.15, 1.0],
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
    );

    register_material(
        world,
        "sand",
        Material {
            base_color: [0.76, 0.70, 0.50, 1.0],
            roughness: 0.95,
            metallic: 0.0,
            ..Default::default()
        },
    );

    register_material(
        world,
        "rock",
        Material {
            base_color: [0.35, 0.33, 0.30, 1.0],
            roughness: 0.85,
            metallic: 0.0,
            ..Default::default()
        },
    );

    register_material(
        world,
        "dirt",
        Material {
            base_color: [0.35, 0.25, 0.18, 1.0],
            roughness: 0.95,
            metallic: 0.0,
            ..Default::default()
        },
    );
}

fn register_material(world: &mut World, name: &str, material: Material) {
    material_registry_insert(
        &mut world.resources.material_registry,
        name.to_string(),
        material,
    );
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
}

fn spawn_infinite_ocean(world: &mut World) -> Entity {
    let entity = world.spawn_entities(WATER | NAME, 1)[0];

    world
        .core
        .set_name(entity, Name("Infinite Ocean".to_string()));
    world.core.set_water(entity, Water::default());

    entity
}

fn spawn_water(
    world: &mut World,
    name: &str,
    mesh: &str,
    position: Vec3,
    scale: Vec3,
    water: Water,
) -> Entity {
    let entity = world.spawn_entities(
        LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | VISIBILITY
            | WATER
            | NAME,
        1,
    )[0];

    world.core.set_name(entity, Name(name.to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale,
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_render_mesh(entity, RenderMesh::new(mesh));
    world.core.set_water(entity, water);

    if let Some(&index) = world.resources.mesh_cache.registry.name_to_index.get(mesh) {
        world.resources.mesh_cache.registry.add_reference(index);
    }

    entity
}

fn spawn_volumetric_waterfall(
    world: &mut World,
    name: &str,
    position: Vec3,
    water: Water,
) -> Entity {
    let entity = world.spawn_entities(
        LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | VISIBILITY | WATER | NAME,
        1,
    )[0];

    world.core.set_name(entity, Name(name.to_string()));

    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: vec3(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_water(entity, water);

    entity
}

fn spawn_terrain(
    world: &mut World,
    name: &str,
    position: Vec3,
    scale: Vec3,
    material: &str,
) -> Entity {
    let entity = world.spawn_entities(
        LOCAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | VISIBILITY
            | BOUNDING_VOLUME
            | NAME,
        1,
    )[0];

    world.core.set_name(entity, Name(name.to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale,
        },
    );
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world.core.set_render_mesh(entity, RenderMesh::new("Cube"));
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material));

    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(material)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    if let Some(&index) = world
        .resources
        .mesh_cache
        .registry
        .name_to_index
        .get("Cube")
    {
        world.resources.mesh_cache.registry.add_reference(index);
    }

    if let Some(bounding_volume) = world.core.get_bounding_volume_mut(entity) {
        *bounding_volume = BoundingVolume::from_mesh_type("Cube");
    }

    entity
}

fn spawn_sun(world: &mut World) {
    let sun_entity = world.spawn_entities(
        nightshade::ecs::world::NAME
            | nightshade::ecs::world::LOCAL_TRANSFORM
            | nightshade::ecs::world::LOCAL_TRANSFORM_DIRTY
            | nightshade::ecs::world::GLOBAL_TRANSFORM
            | nightshade::ecs::world::LIGHT,
        1,
    )[0];

    world.core.set_name(sun_entity, Name("Sun".to_string()));
    world.core.set_local_transform(
        sun_entity,
        LocalTransform {
            translation: Vec3::new(100.0, 100.0, 50.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world
        .core
        .set_local_transform_dirty(sun_entity, LocalTransformDirty);
    world
        .core
        .set_global_transform(sun_entity, GlobalTransform::default());
    world.core.set_light(
        sun_entity,
        Light {
            light_type: LightType::Directional,
            color: Vec3::new(1.0, 0.95, 0.8),
            intensity: 3.0,
            range: 0.0,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            cast_shadows: false,
            shadow_bias: 0.005,
        },
    );

    if let Some(transform) = world.core.get_local_transform_mut(sun_entity) {
        let sun_direction = Vec3::new(-0.5, -0.8, -0.3).normalize();
        let forward = -sun_direction;
        let up = Vec3::new(0.0, 1.0, 0.0);
        let right = nalgebra_glm::cross(&up, &forward).normalize();
        let corrected_up = nalgebra_glm::cross(&forward, &right);
        let rotation_matrix = nalgebra_glm::mat3(
            right.x,
            corrected_up.x,
            forward.x,
            right.y,
            corrected_up.y,
            forward.y,
            right.z,
            corrected_up.z,
            forward.z,
        );
        transform.rotation = nalgebra_glm::mat3_to_quat(&rotation_matrix);
    }
    mark_local_transform_dirty(world, sun_entity);
}
