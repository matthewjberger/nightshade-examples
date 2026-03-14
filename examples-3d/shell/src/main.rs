use nightshade::ecs::camera::commands::spawn_pan_orbit_camera;
use nightshade::ecs::camera::components::Smoothing;
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::picking::queries::pick_closest_entity;
use nightshade::ecs::world::World;
use nightshade::prelude::*;
use nightshade::shell::{format_entity, shell_retained_ui};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(ShellDemo::default())
}

#[derive(Default)]
struct ShellContext {
    noclip: bool,
}

struct ShellDemo {
    shell: ShellState<ShellContext>,
    spawned_entities: Vec<Entity>,
    hover_prompt_entity: Option<Entity>,
    hover_prompt_text_index: Option<usize>,
}

impl Default for ShellDemo {
    fn default() -> Self {
        let mut shell = ShellState::new(ShellContext::default());
        shell.register_builtin_commands();
        Self {
            shell,
            spawned_entities: Vec::new(),
            hover_prompt_entity: None,
            hover_prompt_text_index: None,
        }
    }
}

impl State for ShellDemo {
    fn title(&self) -> &str {
        "Shell Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.retained_ui.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::Sky;
        world.resources.graphics.show_grid = true;

        let camera_entity = spawn_pan_orbit_camera(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            15.0,
            0.0,
            0.3,
            "Main Camera".to_string(),
        );
        world.resources.active_camera = Some(camera_entity);

        spawn_sun(world);

        self.shell.register_command(Box::new(NoclipCommand));

        let cube = spawn_cube_at(world, Vec3::new(-4.0, 0.0, 0.0));
        let sphere = spawn_sphere_at(world, Vec3::new(0.0, 0.0, 0.0));
        let cylinder = spawn_cylinder_at(world, Vec3::new(4.0, 0.0, 0.0));

        self.spawned_entities.push(cube);
        self.spawned_entities.push(sphere);
        self.spawned_entities.push(cylinder);

        spawn_ui_text_with_properties(
            world,
            "Alt+C: Console | Alt+Click: Select object",
            Vec2::zeros(),
            TextProperties {
                font_size: 20.0,
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                alignment: TextAlignment::Center,
                outline_width: 0.01,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );

        let hover_prompt_entity =
            spawn_ui_text(world, "", Vec2::zeros());
        if let Some(hud_text) = world.core.get_text(hover_prompt_entity) {
            self.hover_prompt_text_index = Some(hud_text.text_index);
        }
        self.hover_prompt_entity = Some(hover_prompt_entity);
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.shell.visible && !self.shell.dragging_resize {
            escape_key_exit_system(world);
            if self.shell.context.noclip {
                fly_camera_system(world);
            } else {
                pan_orbit_camera_system(world);
            }
        }

        self.update_hover_prompt(world);
        self.handle_picking(world);

        if self.shell.visible {
            for character in world.resources.input.keyboard.frame_chars.clone() {
                if !character.is_control() {
                    self.shell.input_buffer.push(character);
                }
            }
        }

        let delta_time = world.resources.window.timing.delta_time;
        self.shell.update_animation(delta_time);

        shell_retained_ui(&mut self.shell, world);

        sync_text_meshes_system(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: ElementState) {
        let pressed = key_state == ElementState::Pressed;

        if pressed {
            let alt_pressed = world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::AltLeft)
                || world
                    .resources
                    .input
                    .keyboard
                    .is_key_pressed(KeyCode::AltRight);

            if key_code == KeyCode::KeyC && alt_pressed {
                self.shell.toggle();
                return;
            }
        }

        if self.shell.visible {
            self.shell.handle_key(key_code, pressed);
        }
    }
}

impl ShellDemo {
    fn update_hover_prompt(&mut self, world: &mut World) {
        let Some(text_index) = self.hover_prompt_text_index else {
            return;
        };
        let Some(prompt_entity) = self.hover_prompt_entity else {
            return;
        };

        let alt_pressed = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::AltLeft)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::AltRight);

        if !alt_pressed {
            world.resources.text_cache.set_text(text_index, "");
            if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
                hud_text.dirty = true;
            }
            return;
        }

        let mouse_pos = world.resources.input.mouse.position;

        if self.shell.visible {
            let shell_height = self.shell.height * self.shell.animation_progress;
            if mouse_pos.y < shell_height {
                world.resources.text_cache.set_text(text_index, "");
                if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
                    hud_text.dirty = true;
                }
                return;
            }
        }

        let prompt_text = if let Some(result) = pick_closest_entity(world, mouse_pos) {
            format_entity(result.entity)
        } else {
            String::new()
        };

        world
            .resources
            .text_cache
            .set_text(text_index, &prompt_text);
        if let Some(hud_text) = world.core.get_text_mut(prompt_entity) {
            hud_text.dirty = true;
        }
    }

    fn handle_picking(&mut self, world: &mut World) {
        let left_just_pressed = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_JUST_PRESSED);

        if !left_just_pressed {
            return;
        }

        let alt_pressed = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::AltLeft)
            || world
                .resources
                .input
                .keyboard
                .is_key_pressed(KeyCode::AltRight);

        if !alt_pressed {
            return;
        }

        if self.shell.dragging_resize {
            return;
        }

        let mouse_pos = world.resources.input.mouse.position;

        if self.shell.visible {
            let shell_height = self.shell.height * self.shell.animation_progress;
            if mouse_pos.y < shell_height {
                return;
            }
        }

        if let Some(result) = pick_closest_entity(world, mouse_pos) {
            self.shell.insert_entity(result.entity);
            if !self.shell.visible {
                self.shell.toggle();
            }
        }
    }
}

struct NoclipCommand;

impl Command<ShellContext> for NoclipCommand {
    fn name(&self) -> &str {
        "noclip"
    }

    fn description(&self) -> &str {
        "Toggle noclip fly camera mode"
    }

    fn usage(&self) -> &str {
        "noclip"
    }

    fn execute(&self, _args: &[&str], world: &mut World, context: &mut ShellContext) -> String {
        context.noclip = !context.noclip;
        if context.noclip {
            if let Some(camera_entity) = world.resources.active_camera
                && let Some(camera) = world.core.get_camera_mut(camera_entity)
            {
                camera.smoothing = Some(Smoothing::default());
            }
            "Noclip enabled - WASD to move, right-click to look".to_string()
        } else {
            if let Some(camera_entity) = world.resources.active_camera
                && let Some(camera) = world.core.get_camera_mut(camera_entity)
            {
                camera.smoothing = None;
            }
            "Noclip disabled - orbit camera restored".to_string()
        }
    }
}
