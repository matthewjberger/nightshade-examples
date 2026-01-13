use nightshade::ecs::physics::*;
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Platformer::default())
}

#[derive(Default)]
struct Platformer {
    player_entity: Option<Entity>,
    camera_entity: Option<Entity>,
    physics_object_pool: Vec<Entity>,
    gamepad_interact_pressed: bool,
}

impl State for Platformer {
    fn title(&self) -> &str {
        "Platformer Physics Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = false;
        world.resources.physics.debug_draw = true;

        #[cfg(feature = "openxr")]
        {
            world.resources.xr.locomotion_enabled = false;
        }

        spawn_sun(world);

        let player_position = nalgebra_glm::vec3(-15.0, 2.0, -15.0);
        let (player_entity, camera_entity) = spawn_first_person_player(world, player_position);

        self.player_entity = Some(player_entity);
        self.camera_entity = Some(camera_entity);

        self.spawn_ground(world);
        self.spawn_platformer_level(world);
        self.spawn_physics_objects(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        #[cfg(feature = "openxr")]
        self.xr_input_system(world);

        #[cfg(not(feature = "openxr"))]
        look_camera_system(world);

        self.interaction_system(world);
        self.cleanup_fallen_objects(world);
    }

    fn ui(&mut self, world: &mut World, ui: &egui::Context) {
        egui::Window::new("Platformer Demo")
            .default_pos([10.0, 10.0])
            .resizable(false)
            .show(ui, |ui| {
                ui.label("Controls:");
                ui.label("- WASD / Left Stick: Move");
                ui.label("- Space / A Button: Jump");
                ui.label("- Right-click drag / Right Stick: Look");
                ui.label("- F / X Button: Kick objects");
                ui.label("- ESC: Exit");
                ui.separator();
                ui.label(format!(
                    "FPS: {:.1}",
                    world.resources.window.timing.frames_per_second
                ));
                ui.label(format!(
                    "Physics Objects: {}",
                    self.physics_object_pool.len()
                ));
            });
    }

    fn on_gamepad_event(&mut self, _world: &mut World, event: gilrs::Event) {
        match event.event {
            gilrs::EventType::ButtonPressed(gilrs::Button::West, _) => {
                self.gamepad_interact_pressed = true;
            }
            gilrs::EventType::ButtonReleased(gilrs::Button::West, _) => {
                self.gamepad_interact_pressed = false;
            }
            _ => {}
        }
    }
}

impl Platformer {
    fn spawn_ground(&self, world: &mut World) {
        let ground_material = create_textured_material(nalgebra_glm::vec3(0.3, 0.5, 0.2), 0.9, 0.0);
        spawn_static_physics_cube_with_material(
            world,
            nalgebra_glm::vec3(0.0, -1.0, 0.0),
            nalgebra_glm::vec3(100.0, 0.5, 100.0),
            ground_material,
        );
    }

    fn spawn_platformer_level(&self, world: &mut World) {
        let platform_color = nalgebra_glm::vec3(0.6, 0.4, 0.2);
        let platform_material = create_textured_material(platform_color, 0.8, 0.0);

        let platforms = vec![
            (
                nalgebra_glm::vec3(-15.0, 0.5, -15.0),
                nalgebra_glm::vec3(5.0, 0.5, 5.0),
            ),
            (
                nalgebra_glm::vec3(-8.0, 0.5, -15.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(-1.0, 0.5, -15.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(6.0, 0.5, -15.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(12.0, 1.5, -15.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(18.0, 2.5, -15.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(18.0, 2.5, -8.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(18.0, 3.5, -1.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(18.0, 4.5, 6.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(12.0, 4.5, 6.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(5.0, 4.5, 6.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(-2.0, 5.5, 6.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(-2.0, 6.5, 12.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(5.0, 7.5, 12.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(12.0, 8.5, 12.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(18.0, 9.5, 6.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(-5.0, 10.5, -15.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(0.0, 11.0, -10.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(5.0, 11.5, -5.0),
                nalgebra_glm::vec3(3.0, 0.5, 3.0),
            ),
            (
                nalgebra_glm::vec3(10.0, 12.0, 0.0),
                nalgebra_glm::vec3(4.0, 0.5, 4.0),
            ),
            (
                nalgebra_glm::vec3(15.0, 10.0, 3.0),
                nalgebra_glm::vec3(2.5, 0.5, 2.5),
            ),
            (
                nalgebra_glm::vec3(12.0, 11.0, -2.0),
                nalgebra_glm::vec3(2.5, 0.5, 2.5),
            ),
            (
                nalgebra_glm::vec3(0.0, 2.0, -8.0),
                nalgebra_glm::vec3(2.0, 0.5, 2.0),
            ),
            (
                nalgebra_glm::vec3(8.0, 3.0, 0.0),
                nalgebra_glm::vec3(2.0, 0.5, 2.0),
            ),
            (
                nalgebra_glm::vec3(-8.0, 3.5, 0.0),
                nalgebra_glm::vec3(2.0, 0.5, 2.0),
            ),
        ];

        for (position, size) in platforms {
            spawn_static_physics_cube_with_material(
                world,
                position,
                size,
                platform_material.clone(),
            );
        }
    }

    fn spawn_physics_objects(&mut self, world: &mut World) {
        let colors = [
            nalgebra_glm::vec3(0.9, 0.2, 0.2),
            nalgebra_glm::vec3(0.2, 0.8, 0.2),
            nalgebra_glm::vec3(0.2, 0.4, 0.9),
            nalgebra_glm::vec3(1.0, 0.8, 0.0),
            nalgebra_glm::vec3(0.8, 0.2, 0.8),
        ];

        let objects = vec![
            (nalgebra_glm::vec3(-12.0, 2.0, -12.0), 0, 0),
            (nalgebra_glm::vec3(-18.0, 2.0, -18.0), 1, 1),
            (nalgebra_glm::vec3(-6.0, 2.0, -13.0), 2, 2),
            (nalgebra_glm::vec3(2.0, 2.0, -13.0), 0, 3),
            (nalgebra_glm::vec3(16.0, 4.0, -6.0), 1, 4),
            (nalgebra_glm::vec3(16.0, 6.0, 8.0), 2, 0),
            (nalgebra_glm::vec3(3.0, 6.0, 8.0), 0, 1),
            (nalgebra_glm::vec3(-4.0, 7.0, 14.0), 1, 2),
            (nalgebra_glm::vec3(14.0, 9.0, 14.0), 2, 3),
            (nalgebra_glm::vec3(20.0, 10.0, 8.0), 0, 4),
        ];

        for (position, shape_type, color_index) in objects {
            let color = colors[color_index % colors.len()];
            let material = create_textured_material(color, 0.7, 0.0);

            let entity = match shape_type {
                0 => {
                    spawn_dynamic_physics_sphere_with_material(world, position, 0.5, 1.0, material)
                }
                1 => spawn_dynamic_physics_cube_with_material(
                    world,
                    position,
                    nalgebra_glm::vec3(0.8, 0.8, 0.8),
                    1.0,
                    material,
                ),
                _ => spawn_dynamic_physics_cylinder_with_material(
                    world, position, 0.5, 0.5, 0.8, material,
                ),
            };

            self.physics_object_pool.push(entity);
        }
    }

    fn interaction_system(&mut self, world: &mut World) {
        let interact_key_pressed = world.resources.input.keyboard.is_key_pressed(KeyCode::KeyF);
        let interact_gamepad_pressed = self.gamepad_interact_pressed;

        if !interact_key_pressed && !interact_gamepad_pressed {
            return;
        }

        self.kick_nearby_object(world);
    }

    fn cleanup_fallen_objects(&mut self, world: &mut World) {
        self.physics_object_pool.retain(|&entity| {
            if let Some(transform) = world.get_local_transform(entity) {
                if transform.translation.y < -10.0 {
                    world.despawn_entities(&[entity]);
                    false
                } else {
                    true
                }
            } else {
                false
            }
        });
    }

    #[cfg(feature = "openxr")]
    fn xr_input_system(&mut self, world: &mut World) {
        let Some(xr_input) = world.resources.xr.input.clone() else {
            return;
        };

        let kick_pressed = xr_input.b_button_pressed() || xr_input.right_trigger_pressed();

        if kick_pressed {
            self.kick_nearby_object(world);
        }
    }

    fn kick_nearby_object(&self, world: &mut World) {
        let Some(player_entity) = self.player_entity else {
            return;
        };

        let Some(player_transform) = world.get_local_transform(player_entity) else {
            return;
        };

        let player_position = player_transform.translation;
        let interaction_range = 3.5;
        let kick_force = 15.0;

        for &object_entity in &self.physics_object_pool {
            if let Some(object_transform) = world.get_local_transform(object_entity) {
                let distance =
                    nalgebra_glm::distance(&player_position, &object_transform.translation);

                if distance < interaction_range {
                    let kick_direction =
                        nalgebra_glm::normalize(&(object_transform.translation - player_position));
                    let kick_velocity = kick_direction * kick_force;

                    if let Some(rigid_body_component) = world.get_rigid_body(object_entity)
                        && let Some(handle) = rigid_body_component.handle
                        && let Some(rigid_body) = world
                            .resources
                            .physics
                            .rigid_body_set
                            .get_mut(handle.into())
                    {
                        let rapier_velocity = rapier3d::math::Vector::new(
                            kick_velocity.x,
                            kick_velocity.y,
                            kick_velocity.z,
                        );
                        rigid_body.set_linvel(rapier_velocity, true);
                    }
                    break;
                }
            }
        }
    }
}
