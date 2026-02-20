use nightshade::prelude::*;

mod chip8;
mod roms;

use chip8::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};

const PIXEL_SIZE: f32 = 5.0;
const BASE_Y_MAG: f32 = 225.0;
const COLOR_ON: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const COLOR_OFF: [f32; 4] = [0.05, 0.05, 0.05, 1.0];
const WHITE_TEXTURE_SLOT: u32 = 0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Chip8App::default())
}

#[derive(Default)]
struct Chip8App {
    emulator: Option<Chip8>,
    pixel_entities: Vec<Entity>,
    camera_entity: Option<Entity>,
    cycles_per_frame: u32,
    paused: bool,
    rom_name: String,
    error_message: Option<String>,
}

impl State for Chip8App {
    fn title(&self) -> &str {
        "Super CHIP-8"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.clear_color = [0.02, 0.02, 0.02, 1.0];

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        if let Some(camera_component) = world.get_camera_mut(camera)
            && let Projection::Orthographic(ref mut ortho) = camera_component.projection
        {
            ortho.y_mag = BASE_Y_MAG;
        }
        self.camera_entity = Some(camera);

        let white_pixel: Vec<u8> = vec![255, 255, 255, 255];
        world
            .resources
            .command_queue
            .push(WorldCommand::UploadSpriteTexture {
                slot: WHITE_TEXTURE_SLOT,
                rgba_data: white_pixel,
                width: 1,
                height: 1,
            });

        let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
        let uv_max = Vec2::new(
            1.0 / atlas_slot_size.0 as f32 - 0.5 / atlas_slot_size.0 as f32,
            1.0 / atlas_slot_size.1 as f32 - 0.5 / atlas_slot_size.1 as f32,
        );
        let uv_min = Vec2::new(
            0.5 / atlas_slot_size.0 as f32,
            0.5 / atlas_slot_size.1 as f32,
        );

        let half_display_width = (DISPLAY_WIDTH as f32 * PIXEL_SIZE) / 2.0;
        let half_display_height = (DISPLAY_HEIGHT as f32 * PIXEL_SIZE) / 2.0;

        self.pixel_entities.reserve(DISPLAY_WIDTH * DISPLAY_HEIGHT);
        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                let world_x = col as f32 * PIXEL_SIZE - half_display_width + PIXEL_SIZE / 2.0;
                let world_y = (DISPLAY_HEIGHT as f32 - 1.0 - row as f32) * PIXEL_SIZE
                    - half_display_height
                    + PIXEL_SIZE / 2.0;

                let entity = spawn_sprite(
                    world,
                    Vec2::new(world_x, world_y),
                    Vec2::new(PIXEL_SIZE - 0.5, PIXEL_SIZE - 0.5),
                );

                if let Some(sprite) = world.get_sprite_mut(entity) {
                    sprite.texture_index = WHITE_TEXTURE_SLOT;
                    sprite.texture_index2 = WHITE_TEXTURE_SLOT;
                    sprite.uv_min = uv_min;
                    sprite.uv_max = uv_max;
                    sprite.color = COLOR_OFF;
                }

                self.pixel_entities.push(entity);
            }
        }

        let mut emulator = Chip8::new();
        emulator.load_rom(roms::DEMO_ROM);
        self.emulator = Some(emulator);
        self.cycles_per_frame = 10;
        self.paused = false;
        self.rom_name = "Demo".to_string();
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);

        if let Some(camera_entity) = self.camera_entity
            && let Some(window_handle) = &world.resources.window.handle
        {
            let size = window_handle.inner_size();
            if size.height > 0 {
                let aspect = size.width as f32 / size.height as f32;
                if let Some(camera_component) = world.get_camera_mut(camera_entity)
                    && let Projection::Orthographic(ref mut ortho) = camera_component.projection
                {
                    ortho.x_mag = BASE_Y_MAG * aspect;
                }
            }
        }

        let Some(emulator) = self.emulator.as_mut() else {
            return;
        };

        update_keypad(world, emulator);

        if !self.paused {
            for _ in 0..self.cycles_per_frame {
                emulator.tick();
            }
            emulator.tick_timers();
        }

        if emulator.draw_flag {
            for (pixel_index, &pixel_on) in emulator.display.iter().enumerate() {
                let entity = self.pixel_entities[pixel_index];
                let color = if pixel_on { COLOR_ON } else { COLOR_OFF };
                if let Some(sprite) = world.get_sprite_mut(entity) {
                    sprite.color = color;
                }
            }
            emulator.draw_flag = false;
        }
    }

    fn on_dropped_file(&mut self, _world: &mut World, path: &std::path::Path) {
        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("ch8"))
            && let Ok(data) = std::fs::read(path)
        {
            self.load_rom_data(
                &data,
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "dropped.ch8".to_string()),
            );
        }
    }

    fn on_dropped_file_data(&mut self, _world: &mut World, name: &str, data: &[u8]) {
        if name.to_lowercase().ends_with(".ch8") {
            self.load_rom_data(data, name.to_string());
        }
    }

    fn on_keyboard_input(&mut self, _world: &mut World, key: KeyCode, state: KeyState) {
        if state == KeyState::Pressed && key == KeyCode::Space {
            self.paused = !self.paused;
        }

        if state == KeyState::Pressed && key == KeyCode::Backspace {
            let rom_data = roms::BUNDLED_ROMS
                .iter()
                .find(|rom| rom.name == self.rom_name)
                .map(|rom| rom.data);
            if let Some(data) = rom_data {
                self.load_rom_data(data, self.rom_name.clone());
            }
        }
    }

    fn ui(&mut self, _world: &mut World, context: &egui::Context) {
        egui::Window::new("CHIP-8 Emulator")
            .default_pos([10.0, 10.0])
            .resizable(false)
            .show(context, |ui| {
                ui.heading("Controls");
                ui.separator();
                ui.label("CHIP-8 Keypad:");
                ui.label("1 2 3 4  ->  1 2 3 C");
                ui.label("Q W E R  ->  4 5 6 D");
                ui.label("A S D F  ->  7 8 9 E");
                ui.label("Z X C V  ->  A 0 B F");
                ui.separator();
                ui.label("SPACE - Pause/Resume");
                ui.label("BACKSPACE - Reset");
                ui.label("ESC - Exit");
                ui.separator();

                ui.heading("ROM");
                let mut selected_rom = None;
                egui::ComboBox::from_label("Select ROM")
                    .selected_text(&self.rom_name)
                    .show_ui(ui, |ui| {
                        for (index, rom) in roms::BUNDLED_ROMS.iter().enumerate() {
                            if ui
                                .selectable_label(self.rom_name == rom.name, rom.name)
                                .clicked()
                            {
                                selected_rom = Some(index);
                            }
                        }
                    });
                if let Some(index) = selected_rom {
                    let rom = &roms::BUNDLED_ROMS[index];
                    self.load_rom_data(rom.data, rom.name.to_string());
                }
                ui.label("Or drop a .ch8 file to load");
                if let Some(error) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, error);
                }
                ui.separator();

                let mut speed = self.cycles_per_frame as i32;
                ui.add(egui::Slider::new(&mut speed, 1..=30).text("Cycles/frame"));
                self.cycles_per_frame = speed as u32;

                ui.separator();

                if let Some(emulator) = &self.emulator {
                    ui.heading("State");
                    ui.separator();

                    if self.paused {
                        ui.colored_label(egui::Color32::YELLOW, "PAUSED");
                    }
                    if emulator.halted {
                        ui.colored_label(egui::Color32::RED, "HALTED");
                    }
                    ui.label(if emulator.hi_res {
                        "Mode: Hi-Res (128x64)"
                    } else {
                        "Mode: Lo-Res (64x32)"
                    });
                    ui.separator();

                    ui.label(format!("PC: 0x{:04X}", emulator.pc));
                    ui.label(format!("I:  0x{:04X}", emulator.index));
                    ui.label(format!("SP: {}", emulator.sp));
                    ui.separator();

                    ui.label(format!(
                        "DT: {}  ST: {}",
                        emulator.delay_timer, emulator.sound_timer
                    ));
                    ui.separator();

                    egui::Grid::new("registers").num_columns(4).show(ui, |ui| {
                        for register in 0..16 {
                            ui.label(format!("V{:X}: 0x{:02X}", register, emulator.v[register]));
                            if register % 4 == 3 {
                                ui.end_row();
                            }
                        }
                    });
                }
            });
    }
}

impl Chip8App {
    fn load_rom_data(&mut self, data: &[u8], name: String) {
        if let Some(emulator) = self.emulator.as_mut() {
            emulator.reset();
            if emulator.load_rom(data) {
                self.rom_name = name;
                self.paused = false;
                self.error_message = None;
            } else {
                self.error_message = Some(format!(
                    "ROM too large: {} bytes (max {})",
                    data.len(),
                    4096 - 0x200
                ));
            }
        }
    }
}

fn update_keypad(world: &World, emulator: &mut Chip8) {
    let keyboard = &world.resources.input.keyboard;
    let key_mappings = [
        (KeyCode::Digit1, 0x1),
        (KeyCode::Digit2, 0x2),
        (KeyCode::Digit3, 0x3),
        (KeyCode::Digit4, 0xC),
        (KeyCode::KeyQ, 0x4),
        (KeyCode::KeyW, 0x5),
        (KeyCode::KeyE, 0x6),
        (KeyCode::KeyR, 0xD),
        (KeyCode::KeyA, 0x7),
        (KeyCode::KeyS, 0x8),
        (KeyCode::KeyD, 0x9),
        (KeyCode::KeyF, 0xE),
        (KeyCode::KeyZ, 0xA),
        (KeyCode::KeyX, 0x0),
        (KeyCode::KeyC, 0xB),
        (KeyCode::KeyV, 0xF),
    ];

    for &(keycode, chip8_key) in &key_mappings {
        emulator.keypad[chip8_key] = keyboard.is_key_pressed(keycode);
    }
}
