mod constants;
mod ecs;
mod systems;

use constants::SKY_HDR;
use ecs::{GameWorld, default_terrain_config};
use nightshade::ecs::camera::systems::pan_orbit_camera_system;
use nightshade::ecs::physics::debug::physics_debug_draw_system;
use nightshade::prelude::*;
use nightshade::render::wgpu::passes;
use nightshade::render::wgpu::rendergraph::RenderGraph;
use nightshade::run::RenderResources;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(ThirdPersonGame::default())
}

struct ThirdPersonGame {
    game_world: GameWorld,
    loaded: bool,
    debug_key_was_pressed: bool,
}

impl Default for ThirdPersonGame {
    fn default() -> Self {
        let mut game_world = GameWorld::default();
        game_world.resources.terrain_config = default_terrain_config();

        Self {
            game_world,
            loaded: false,
            debug_key_was_pressed: false,
        }
    }
}

impl State for ThirdPersonGame {
    fn title(&self) -> &str {
        "Third Person Demo"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::Hdr;
        world.resources.graphics.use_fullscreen = true;
        world.resources.graphics.ui_scale = Some(1.0);
        world.resources.graphics.bloom_enabled = true;

        load_hdr_skybox(world, SKY_HDR.to_vec());

        let sun = spawn_sun(world);
        if let Some(light) = world.get_light_mut(sun) {
            light.cast_shadows = true;
            light.intensity = 1.5;
        }

        systems::spawn_environment(&mut self.game_world, world);
        systems::spawn_character_controller(&mut self.game_world, world);
        systems::spawn_snow_blizzard(world);
        systems::spawn_footprint_emitter(&mut self.game_world, world);
        systems::spawn_camera(&mut self.game_world, world);

        systems::load_fox_model(&mut self.game_world, world);
        self.loaded = true;
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        let debug_key_pressed = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Digit4);
        if debug_key_pressed && !self.debug_key_was_pressed {
            world.resources.physics.debug_draw = !world.resources.physics.debug_draw;
        }
        self.debug_key_was_pressed = debug_key_pressed;

        pan_orbit_camera_system(world);
        physics_debug_draw_system(world);

        let delta_time = world.resources.window.timing.delta_time;
        update_particle_emitters(world, delta_time);

        systems::update_campfire_light(&self.game_world, world);

        if self.loaded {
            systems::sync_fox_to_controller(&mut self.game_world, world);
            systems::animation_system(&mut self.game_world, world);
            systems::camera_follow_system(&self.game_world, world);
            systems::update_footprint_emitter(&self.game_world, world);
        }
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let particle_pass = passes::ParticlePass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(particle_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.3);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .write("output", resources.swapchain);
    }
}
