mod render;
mod wad;

use nightshade::ecs::camera::components::{Camera, PerspectiveCamera, Projection, Smoothing};
use nightshade::prelude::*;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use render::DoomPass;
use wad::{Archive, TextureDirectory};

const WAD_DATA: &[u8] = include_bytes!("../../assets/wads/Doom1.WAD");
const MOVE_SPEED: f32 = 1.5;
const MOUSE_SENSITIVITY: f32 = 0.003;

#[derive(Default)]
struct DoomGame {
    archive: Option<Archive>,
    tex_dir: Option<TextureDirectory>,
    current_level: usize,
    camera_entity: Option<Entity>,
    player_start_position: Option<Vec3>,
}

impl State for DoomGame {
    fn title(&self) -> &str {
        "Doom Renderer"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.atmosphere = Atmosphere::Space;
        world.resources.user_interface.enabled = false;

        let start_pos = self
            .player_start_position
            .unwrap_or(Vec3::new(0.0, 0.5, 0.0));
        let camera = spawn_camera(world, start_pos, "Camera".to_string());

        if let Some(camera_component) = world.get_camera_mut(camera) {
            *camera_component = Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 90.0_f32.to_radians(),
                    z_far: None,
                    z_near: 0.001,
                }),
                smoothing: Some(Smoothing::default()),
            };
        }

        world.resources.active_camera = Some(camera);
        self.camera_entity = Some(camera);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        if self.archive.is_none() {
            match Archive::from_bytes(WAD_DATA) {
                Ok(archive) => match TextureDirectory::from_archive(&archive) {
                    Ok(tex_dir) => {
                        self.tex_dir = Some(tex_dir);
                        self.archive = Some(archive);
                    }
                    Err(error) => {
                        tracing::error!("Failed to load texture directory: {}", error);
                    }
                },
                Err(error) => {
                    tracing::error!("Failed to parse WAD data: {}", error);
                }
            }
        }

        if let (Some(archive), Some(tex_dir)) = (&self.archive, &self.tex_dir) {
            match DoomPass::new(
                device,
                archive,
                tex_dir,
                self.current_level,
                wgpu::TextureFormat::Rgba16Float,
            ) {
                Ok(doom_pass) => {
                    if let Some(start) = doom_pass.player_start {
                        self.player_start_position = Some(start.position);
                    }
                    graph
                        .pass(Box::new(doom_pass))
                        .slot("color", resources.scene_color)
                        .slot("depth", resources.depth);
                }
                Err(error) => {
                    tracing::error!("Failed to create doom pass: {}", error);
                }
            }
        }

        let blit_pass = passes::BlitPass::new(device, surface_format);
        graph
            .pass(Box::new(blit_pass))
            .read("input", resources.scene_color)
            .write("output", resources.swapchain);
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        doom_camera_system(world);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(DoomGame::default())
}

fn doom_camera_system(world: &mut World) {
    doom_look_system(world);
    doom_movement_system(world);
}

fn doom_look_system(world: &mut World) {
    use nightshade::ecs::transform::commands::mark_local_transform_dirty;
    use nightshade::ecs::world::resources::MouseState;
    use std::sync::atomic::{AtomicBool, Ordering};

    static CAMERA_HAS_CURSOR: AtomicBool = AtomicBool::new(false);

    let Some(camera_entity) = world.resources.active_camera else {
        return;
    };

    let right_clicked = world
        .resources
        .input
        .mouse
        .state
        .contains(MouseState::RIGHT_CLICKED);

    if right_clicked {
        let already_grabbed = CAMERA_HAS_CURSOR.load(Ordering::Relaxed);

        if !already_grabbed {
            if let Some(window_handle) = &world.resources.window.handle {
                if window_handle
                    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                    .is_err()
                {
                    let _ = window_handle.set_cursor_grab(winit::window::CursorGrabMode::Confined);
                }
                window_handle.set_cursor_visible(false);
            }
            CAMERA_HAS_CURSOR.store(true, Ordering::Relaxed);
        }

        let raw_delta = world.resources.input.mouse.raw_mouse_delta;

        let Some(local_transform) = world.get_local_transform_mut(camera_entity) else {
            return;
        };

        let delta_x = -raw_delta.x * MOUSE_SENSITIVITY;
        let delta_y = -raw_delta.y * MOUSE_SENSITIVITY;

        let yaw = nalgebra_glm::quat_angle_axis(delta_x, &Vec3::y());
        local_transform.rotation = yaw * local_transform.rotation;

        let forward = local_transform.forward_vector();
        let current_pitch = forward.y.asin();

        let new_pitch = current_pitch + delta_y;
        if new_pitch.abs() <= 89_f32.to_radians() {
            let pitch = nalgebra_glm::quat_angle_axis(delta_y, &Vec3::x());
            local_transform.rotation *= pitch;
        }

        mark_local_transform_dirty(world, camera_entity);
    } else if CAMERA_HAS_CURSOR.load(Ordering::Relaxed) {
        if let Some(window_handle) = &world.resources.window.handle {
            let _ = window_handle.set_cursor_grab(winit::window::CursorGrabMode::None);
            window_handle.set_cursor_visible(true);
        }
        CAMERA_HAS_CURSOR.store(false, Ordering::Relaxed);
    }
}

fn doom_movement_system(world: &mut World) {
    use nightshade::ecs::transform::commands::mark_local_transform_dirty;

    let Some(camera_entity) = world.resources.active_camera else {
        return;
    };

    let delta_time = world.resources.window.timing.delta_time;

    let (
        left_key_pressed,
        right_key_pressed,
        forward_key_pressed,
        backward_key_pressed,
        up_key_pressed,
        down_key_pressed,
        shift_pressed,
    ) = {
        let keyboard = &world.resources.input.keyboard;
        (
            keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyA),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyD),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyW),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::KeyS),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::Space),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::ShiftLeft)
                || keyboard.is_key_pressed(winit::keyboard::KeyCode::ShiftRight),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::ControlLeft)
                || keyboard.is_key_pressed(winit::keyboard::KeyCode::ControlRight),
        )
    };

    let speed = if shift_pressed {
        MOVE_SPEED * 3.0
    } else {
        MOVE_SPEED
    };

    let mut movement = Vec3::zeros();

    if forward_key_pressed {
        movement.z += 1.0;
    }
    if backward_key_pressed {
        movement.z -= 1.0;
    }
    if left_key_pressed {
        movement.x -= 1.0;
    }
    if right_key_pressed {
        movement.x += 1.0;
    }
    if up_key_pressed {
        movement.y += 1.0;
    }
    if down_key_pressed {
        movement.y -= 1.0;
    }

    if movement.magnitude() < 0.001 {
        return;
    }

    movement = movement.normalize();

    let Some(local_transform) = world.get_local_transform_mut(camera_entity) else {
        return;
    };

    let forward = local_transform.forward_vector();
    let right = local_transform.right_vector();
    let up = Vec3::y();

    let translation = forward * movement.z * speed * delta_time
        + right * movement.x * speed * delta_time
        + up * movement.y * speed * delta_time;

    local_transform.translation += translation;

    mark_local_transform_dirty(world, camera_entity);
}
