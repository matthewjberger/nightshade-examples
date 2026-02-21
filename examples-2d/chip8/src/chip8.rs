pub const DISPLAY_WIDTH: usize = 128;
pub const DISPLAY_HEIGHT: usize = 64;
const LORES_WIDTH: usize = 64;
const LORES_HEIGHT: usize = 32;
const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const PROGRAM_START: u16 = 0x200;
const FONT_START: u16 = 0x000;
const HIRES_FONT_START: u16 = 0x050;
const NUM_RPL_FLAGS: usize = 8;

const FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const HIRES_FONT_DATA: [u8; 100] = [
    0x3C, 0x7E, 0xE7, 0xC3, 0xC3, 0xC3, 0xC3, 0xE7, 0x7E, 0x3C, // 0
    0x18, 0x38, 0x58, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x3C, // 1
    0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, // 2
    0x3C, 0x7E, 0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, // 3
    0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF, 0x06, 0x06, // 4
    0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE, 0x03, 0xC3, 0x7E, 0x3C, // 5
    0x3E, 0x7C, 0xC0, 0xC0, 0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, // 6
    0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60, // 7
    0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, // 8
    0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F, 0x03, 0x03, 0x3E, 0x7C, // 9
];

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    pub v: [u8; NUM_REGISTERS],
    pub index: u16,
    pub pc: u16,
    stack: [u16; STACK_SIZE],
    pub sp: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pub keypad: [bool; NUM_KEYS],
    pub draw_flag: bool,
    pub waiting_for_key: Option<u8>,
    pub hi_res: bool,
    pub halted: bool,
    rpl_flags: [u8; NUM_RPL_FLAGS],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Self {
            memory: [0; MEMORY_SIZE],
            v: [0; NUM_REGISTERS],
            index: 0,
            pc: PROGRAM_START,
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            keypad: [false; NUM_KEYS],
            draw_flag: false,
            waiting_for_key: None,
            hi_res: false,
            halted: false,
            rpl_flags: [0; NUM_RPL_FLAGS],
        };

        for (offset, &byte) in FONT_DATA.iter().enumerate() {
            chip8.memory[FONT_START as usize + offset] = byte;
        }
        for (offset, &byte) in HIRES_FONT_DATA.iter().enumerate() {
            chip8.memory[HIRES_FONT_START as usize + offset] = byte;
        }

        chip8
    }

    pub fn reset(&mut self) {
        self.v = [0; NUM_REGISTERS];
        self.index = 0;
        self.pc = PROGRAM_START;
        self.stack = [0; STACK_SIZE];
        self.sp = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        self.keypad = [false; NUM_KEYS];
        self.draw_flag = false;
        self.waiting_for_key = None;
        self.hi_res = false;
        self.halted = false;
        self.rpl_flags = [0; NUM_RPL_FLAGS];
    }

    pub fn load_rom(&mut self, data: &[u8]) -> bool {
        let max_rom_size = MEMORY_SIZE - PROGRAM_START as usize;
        if data.len() > max_rom_size {
            return false;
        }
        for (offset, &byte) in data.iter().enumerate() {
            self.memory[PROGRAM_START as usize + offset] = byte;
        }
        true
    }

    fn set_pixel(&mut self, buffer_x: usize, buffer_y: usize) -> bool {
        let wrapped_x = buffer_x % DISPLAY_WIDTH;
        let wrapped_y = buffer_y % DISPLAY_HEIGHT;
        let index = wrapped_y * DISPLAY_WIDTH + wrapped_x;
        let collision = self.display[index];
        self.display[index] ^= true;
        collision
    }

    fn draw_lores(&mut self, vx: usize, vy: usize, height: usize) {
        self.v[0xF] = 0;
        for row in 0..height {
            let sprite_byte = self.memory[self.index as usize + row];
            for col in 0..8 {
                if (sprite_byte & (0x80 >> col)) != 0 {
                    let base_x = (vx + col) * 2;
                    let base_y = (vy + row) * 2;
                    if self.set_pixel(base_x, base_y)
                        | self.set_pixel(base_x + 1, base_y)
                        | self.set_pixel(base_x, base_y + 1)
                        | self.set_pixel(base_x + 1, base_y + 1)
                    {
                        self.v[0xF] = 1;
                    }
                }
            }
        }
        self.draw_flag = true;
    }

    fn draw_hires_8xn(&mut self, vx: usize, vy: usize, height: usize) {
        self.v[0xF] = 0;
        for row in 0..height {
            let sprite_byte = self.memory[self.index as usize + row];
            for col in 0..8 {
                if (sprite_byte & (0x80 >> col)) != 0 {
                    let pixel_x = (vx + col) % DISPLAY_WIDTH;
                    let pixel_y = (vy + row) % DISPLAY_HEIGHT;
                    let pixel_index = pixel_y * DISPLAY_WIDTH + pixel_x;
                    if self.display[pixel_index] {
                        self.v[0xF] = 1;
                    }
                    self.display[pixel_index] ^= true;
                }
            }
        }
        self.draw_flag = true;
    }

    fn draw_hires_16x16(&mut self, vx: usize, vy: usize) {
        self.v[0xF] = 0;
        for row in 0..16 {
            let byte_hi = self.memory[self.index as usize + row * 2];
            let byte_lo = self.memory[self.index as usize + row * 2 + 1];
            let sprite_word = (byte_hi as u16) << 8 | byte_lo as u16;
            for col in 0..16 {
                if (sprite_word & (0x8000 >> col)) != 0 {
                    let pixel_x = (vx + col) % DISPLAY_WIDTH;
                    let pixel_y = (vy + row) % DISPLAY_HEIGHT;
                    let pixel_index = pixel_y * DISPLAY_WIDTH + pixel_x;
                    if self.display[pixel_index] {
                        self.v[0xF] = 1;
                    }
                    self.display[pixel_index] ^= true;
                }
            }
        }
        self.draw_flag = true;
    }

    fn scroll_down(&mut self, lines: usize) {
        let shift = lines * DISPLAY_WIDTH;
        let total = DISPLAY_WIDTH * DISPLAY_HEIGHT;
        self.display.copy_within(0..total - shift, shift);
        for pixel in &mut self.display[0..shift] {
            *pixel = false;
        }
        self.draw_flag = true;
    }

    fn scroll_right(&mut self) {
        for row in (0..DISPLAY_HEIGHT).rev() {
            let base = row * DISPLAY_WIDTH;
            self.display
                .copy_within(base..base + DISPLAY_WIDTH - 4, base + 4);
            for col in 0..4 {
                self.display[base + col] = false;
            }
        }
        self.draw_flag = true;
    }

    fn scroll_left(&mut self) {
        for row in 0..DISPLAY_HEIGHT {
            let base = row * DISPLAY_WIDTH;
            self.display
                .copy_within(base + 4..base + DISPLAY_WIDTH, base);
            for col in (DISPLAY_WIDTH - 4)..DISPLAY_WIDTH {
                self.display[base + col] = false;
            }
        }
        self.draw_flag = true;
    }

    pub fn tick(&mut self) {
        if self.halted {
            return;
        }

        if let Some(register) = self.waiting_for_key {
            for key_index in 0..NUM_KEYS {
                if self.keypad[key_index] {
                    self.v[register as usize] = key_index as u8;
                    self.waiting_for_key = None;
                    break;
                }
            }
            return;
        }

        let high = self.memory[self.pc as usize] as u16;
        let low = self.memory[(self.pc + 1) as usize] as u16;
        let opcode = (high << 8) | low;
        self.pc += 2;

        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0xFFF0 {
                0x00C0 => {
                    self.scroll_down(n as usize);
                }
                _ => match opcode {
                    0x00E0 => {
                        self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
                        self.draw_flag = true;
                    }
                    0x00EE => {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }
                    0x00FB => {
                        self.scroll_right();
                    }
                    0x00FC => {
                        self.scroll_left();
                    }
                    0x00FD => {
                        self.halted = true;
                    }
                    0x00FE => {
                        self.hi_res = false;
                        self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
                        self.draw_flag = true;
                    }
                    0x00FF => {
                        self.hi_res = true;
                        self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
                        self.draw_flag = true;
                    }
                    _ => {}
                },
            },
            0x1000 => {
                self.pc = nnn;
            }
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            0x3000 => {
                if self.v[x] == kk {
                    self.pc += 2;
                }
            }
            0x4000 => {
                if self.v[x] != kk {
                    self.pc += 2;
                }
            }
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                self.v[x] = kk;
            }
            0x7000 => {
                self.v[x] = self.v[x].wrapping_add(kk);
            }
            0x8000 => match n {
                0x0 => {
                    self.v[x] = self.v[y];
                }
                0x1 => {
                    self.v[x] |= self.v[y];
                }
                0x2 => {
                    self.v[x] &= self.v[y];
                }
                0x3 => {
                    self.v[x] ^= self.v[y];
                }
                0x4 => {
                    let sum = self.v[x] as u16 + self.v[y] as u16;
                    self.v[0xF] = if sum > 255 { 1 } else { 0 };
                    self.v[x] = sum as u8;
                }
                0x5 => {
                    let not_borrow = if self.v[x] >= self.v[y] { 1 } else { 0 };
                    self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                    self.v[0xF] = not_borrow;
                }
                0x6 => {
                    self.v[0xF] = self.v[x] & 0x1;
                    self.v[x] >>= 1;
                }
                0x7 => {
                    let not_borrow = if self.v[y] >= self.v[x] { 1 } else { 0 };
                    self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                    self.v[0xF] = not_borrow;
                }
                0xE => {
                    self.v[0xF] = (self.v[x] >> 7) & 0x1;
                    self.v[x] <<= 1;
                }
                _ => {}
            },
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xA000 => {
                self.index = nnn;
            }
            0xB000 => {
                self.pc = nnn + self.v[0] as u16;
            }
            0xC000 => {
                let random: u8 = rand::random();
                self.v[x] = random & kk;
            }
            0xD000 => {
                let vx = self.v[x] as usize;
                let vy = self.v[y] as usize;

                if n == 0 && self.hi_res {
                    self.draw_hires_16x16(vx, vy);
                } else if self.hi_res {
                    self.draw_hires_8xn(vx, vy, n as usize);
                } else {
                    let lores_x = vx % LORES_WIDTH;
                    let lores_y = vy % LORES_HEIGHT;
                    self.draw_lores(lores_x, lores_y, n as usize);
                }
            }
            0xE000 => match kk {
                0x9E => {
                    if self.keypad[self.v[x] as usize & 0xF] {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if !self.keypad[self.v[x] as usize & 0xF] {
                        self.pc += 2;
                    }
                }
                _ => {}
            },
            0xF000 => match kk {
                0x07 => {
                    self.v[x] = self.delay_timer;
                }
                0x0A => {
                    self.waiting_for_key = Some(x as u8);
                }
                0x15 => {
                    self.delay_timer = self.v[x];
                }
                0x18 => {
                    self.sound_timer = self.v[x];
                }
                0x1E => {
                    self.index += self.v[x] as u16;
                }
                0x29 => {
                    self.index = FONT_START + (self.v[x] as u16) * 5;
                }
                0x30 => {
                    self.index = HIRES_FONT_START + (self.v[x] as u16 & 0x0F) * 10;
                }
                0x33 => {
                    let value = self.v[x];
                    self.memory[self.index as usize] = value / 100;
                    self.memory[self.index as usize + 1] = (value / 10) % 10;
                    self.memory[self.index as usize + 2] = value % 10;
                }
                0x55 => {
                    for register in 0..=x {
                        self.memory[self.index as usize + register] = self.v[register];
                    }
                }
                0x65 => {
                    for register in 0..=x {
                        self.v[register] = self.memory[self.index as usize + register];
                    }
                }
                0x75 => {
                    let limit = x.min(NUM_RPL_FLAGS - 1);
                    for register in 0..=limit {
                        self.rpl_flags[register] = self.v[register];
                    }
                }
                0x85 => {
                    let limit = x.min(NUM_RPL_FLAGS - 1);
                    for register in 0..=limit {
                        self.v[register] = self.rpl_flags[register];
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instruction(chip8: &mut Chip8, high: u8, low: u8) {
        chip8.memory[chip8.pc as usize] = high;
        chip8.memory[(chip8.pc + 1) as usize] = low;
        chip8.tick();
    }

    #[test]
    fn test_initial_state() {
        let chip8 = Chip8::new();
        assert_eq!(chip8.pc, PROGRAM_START);
        assert_eq!(chip8.sp, 0);
        assert_eq!(chip8.index, 0);
        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);
        assert!(chip8.display.iter().all(|&pixel| !pixel));
        assert!(chip8.keypad.iter().all(|&key| !key));
        assert!(chip8.waiting_for_key.is_none());
        assert!(!chip8.hi_res);
        assert!(!chip8.halted);
    }

    #[test]
    fn test_font_loaded() {
        let chip8 = Chip8::new();
        assert_eq!(chip8.memory[0], 0xF0);
        assert_eq!(chip8.memory[4], 0xF0);
        assert_eq!(chip8.memory[79], 0x80);
    }

    #[test]
    fn test_hires_font_loaded() {
        let chip8 = Chip8::new();
        assert_eq!(chip8.memory[HIRES_FONT_START as usize], 0x3C);
        assert_eq!(chip8.memory[HIRES_FONT_START as usize + 10], 0x18);
        assert_eq!(chip8.memory[HIRES_FONT_START as usize + 99], 0x7C);
    }

    #[test]
    fn test_load_rom() {
        let mut chip8 = Chip8::new();
        let rom = [0x12, 0x34, 0x56, 0x78];
        chip8.load_rom(&rom);
        assert_eq!(chip8.memory[0x200], 0x12);
        assert_eq!(chip8.memory[0x201], 0x34);
        assert_eq!(chip8.memory[0x202], 0x56);
        assert_eq!(chip8.memory[0x203], 0x78);
    }

    #[test]
    fn test_load_rom_too_large() {
        let mut chip8 = Chip8::new();
        let rom = vec![0u8; 4000];
        assert!(!chip8.load_rom(&rom));
    }

    #[test]
    fn test_00e0_cls() {
        let mut chip8 = Chip8::new();
        chip8.display[0] = true;
        chip8.display[100] = true;
        run_instruction(&mut chip8, 0x00, 0xE0);
        assert!(chip8.display.iter().all(|&pixel| !pixel));
        assert!(chip8.draw_flag);
    }

    #[test]
    fn test_00ee_ret() {
        let mut chip8 = Chip8::new();
        chip8.stack[0] = 0x400;
        chip8.sp = 1;
        run_instruction(&mut chip8, 0x00, 0xEE);
        assert_eq!(chip8.pc, 0x400);
        assert_eq!(chip8.sp, 0);
    }

    #[test]
    fn test_0nnn_sys_ignored() {
        let mut chip8 = Chip8::new();
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x01, 0x23);
        assert_eq!(chip8.pc, pc_before + 2);
    }

    #[test]
    fn test_1nnn_jp() {
        let mut chip8 = Chip8::new();
        run_instruction(&mut chip8, 0x13, 0x00);
        assert_eq!(chip8.pc, 0x300);
    }

    #[test]
    fn test_2nnn_call() {
        let mut chip8 = Chip8::new();
        let return_addr = chip8.pc + 2;
        run_instruction(&mut chip8, 0x24, 0x00);
        assert_eq!(chip8.pc, 0x400);
        assert_eq!(chip8.sp, 1);
        assert_eq!(chip8.stack[0], return_addr);
    }

    #[test]
    fn test_3xkk_se_equal() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x42;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x30, 0x42);
        assert_eq!(chip8.pc, pc_before + 4);
    }

    #[test]
    fn test_3xkk_se_not_equal() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x42;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x30, 0x43);
        assert_eq!(chip8.pc, pc_before + 2);
    }

    #[test]
    fn test_4xkk_sne_not_equal() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x42;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x40, 0x43);
        assert_eq!(chip8.pc, pc_before + 4);
    }

    #[test]
    fn test_4xkk_sne_equal() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x42;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x40, 0x42);
        assert_eq!(chip8.pc, pc_before + 2);
    }

    #[test]
    fn test_5xy0_se_vx_vy_equal() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x42;
        chip8.v[1] = 0x42;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x50, 0x10);
        assert_eq!(chip8.pc, pc_before + 4);
    }

    #[test]
    fn test_5xy0_se_vx_vy_not_equal() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x42;
        chip8.v[1] = 0x43;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x50, 0x10);
        assert_eq!(chip8.pc, pc_before + 2);
    }

    #[test]
    fn test_6xkk_ld() {
        let mut chip8 = Chip8::new();
        run_instruction(&mut chip8, 0x6A, 0xFF);
        assert_eq!(chip8.v[0xA], 0xFF);
    }

    #[test]
    fn test_7xkk_add() {
        let mut chip8 = Chip8::new();
        chip8.v[3] = 0x10;
        run_instruction(&mut chip8, 0x73, 0x05);
        assert_eq!(chip8.v[3], 0x15);
    }

    #[test]
    fn test_7xkk_add_wraps() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0xFF;
        run_instruction(&mut chip8, 0x70, 0x02);
        assert_eq!(chip8.v[0], 0x01);
    }

    #[test]
    fn test_8xy0_ld_vx_vy() {
        let mut chip8 = Chip8::new();
        chip8.v[1] = 0x42;
        run_instruction(&mut chip8, 0x80, 0x10);
        assert_eq!(chip8.v[0], 0x42);
    }

    #[test]
    fn test_8xy1_or() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x0F;
        chip8.v[1] = 0xF0;
        run_instruction(&mut chip8, 0x80, 0x11);
        assert_eq!(chip8.v[0], 0xFF);
    }

    #[test]
    fn test_8xy2_and() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x0F;
        chip8.v[1] = 0xFF;
        run_instruction(&mut chip8, 0x80, 0x12);
        assert_eq!(chip8.v[0], 0x0F);
    }

    #[test]
    fn test_8xy3_xor() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0xFF;
        chip8.v[1] = 0x0F;
        run_instruction(&mut chip8, 0x80, 0x13);
        assert_eq!(chip8.v[0], 0xF0);
    }

    #[test]
    fn test_8xy4_add_no_carry() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x10;
        chip8.v[1] = 0x20;
        run_instruction(&mut chip8, 0x80, 0x14);
        assert_eq!(chip8.v[0], 0x30);
        assert_eq!(chip8.v[0xF], 0);
    }

    #[test]
    fn test_8xy4_add_with_carry() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0xFF;
        chip8.v[1] = 0x02;
        run_instruction(&mut chip8, 0x80, 0x14);
        assert_eq!(chip8.v[0], 0x01);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_8xy5_sub_no_borrow() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x20;
        chip8.v[1] = 0x10;
        run_instruction(&mut chip8, 0x80, 0x15);
        assert_eq!(chip8.v[0], 0x10);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_8xy5_sub_with_borrow() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x10;
        chip8.v[1] = 0x20;
        run_instruction(&mut chip8, 0x80, 0x15);
        assert_eq!(chip8.v[0], 0xF0);
        assert_eq!(chip8.v[0xF], 0);
    }

    #[test]
    fn test_8xy6_shr() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x07;
        run_instruction(&mut chip8, 0x80, 0x06);
        assert_eq!(chip8.v[0], 0x03);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_8xy6_shr_no_lsb() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x06;
        run_instruction(&mut chip8, 0x80, 0x06);
        assert_eq!(chip8.v[0], 0x03);
        assert_eq!(chip8.v[0xF], 0);
    }

    #[test]
    fn test_8xy7_subn_no_borrow() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x10;
        chip8.v[1] = 0x20;
        run_instruction(&mut chip8, 0x80, 0x17);
        assert_eq!(chip8.v[0], 0x10);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_8xy7_subn_with_borrow() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x20;
        chip8.v[1] = 0x10;
        run_instruction(&mut chip8, 0x80, 0x17);
        assert_eq!(chip8.v[0], 0xF0);
        assert_eq!(chip8.v[0xF], 0);
    }

    #[test]
    fn test_8xye_shl() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x81;
        run_instruction(&mut chip8, 0x80, 0x0E);
        assert_eq!(chip8.v[0], 0x02);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_8xye_shl_no_msb() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x01;
        run_instruction(&mut chip8, 0x80, 0x0E);
        assert_eq!(chip8.v[0], 0x02);
        assert_eq!(chip8.v[0xF], 0);
    }

    #[test]
    fn test_9xy0_sne_not_equal() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x42;
        chip8.v[1] = 0x43;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x90, 0x10);
        assert_eq!(chip8.pc, pc_before + 4);
    }

    #[test]
    fn test_9xy0_sne_equal() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x42;
        chip8.v[1] = 0x42;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x90, 0x10);
        assert_eq!(chip8.pc, pc_before + 2);
    }

    #[test]
    fn test_annn_ld_i() {
        let mut chip8 = Chip8::new();
        run_instruction(&mut chip8, 0xA1, 0x23);
        assert_eq!(chip8.index, 0x123);
    }

    #[test]
    fn test_bnnn_jp_v0() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x10;
        run_instruction(&mut chip8, 0xB3, 0x00);
        assert_eq!(chip8.pc, 0x310);
    }

    #[test]
    fn test_cxkk_rnd() {
        let mut chip8 = Chip8::new();
        run_instruction(&mut chip8, 0xC0, 0x0F);
        assert_eq!(chip8.v[0] & 0xF0, 0x00);
    }

    #[test]
    fn test_dxyn_draw_lores_no_collision() {
        let mut chip8 = Chip8::new();
        chip8.index = 0x000;
        chip8.v[0] = 0;
        chip8.v[1] = 0;
        run_instruction(&mut chip8, 0xD0, 0x11);
        assert_eq!(chip8.v[0xF], 0);
        assert!(chip8.draw_flag);
        assert!(chip8.display[0]);
        assert!(chip8.display[1]);
        assert!(chip8.display[DISPLAY_WIDTH]);
        assert!(chip8.display[DISPLAY_WIDTH + 1]);
    }

    #[test]
    fn test_dxyn_draw_lores_collision() {
        let mut chip8 = Chip8::new();
        chip8.display[0] = true;
        chip8.index = 0x000;
        chip8.v[0] = 0;
        chip8.v[1] = 0;
        run_instruction(&mut chip8, 0xD0, 0x11);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_dxyn_draw_lores_2x_scaling() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x300] = 0x80;
        chip8.index = 0x300;
        chip8.v[0] = 0;
        chip8.v[1] = 0;
        run_instruction(&mut chip8, 0xD0, 0x11);
        assert!(chip8.display[0]);
        assert!(chip8.display[1]);
        assert!(chip8.display[DISPLAY_WIDTH]);
        assert!(chip8.display[DISPLAY_WIDTH + 1]);
        assert!(!chip8.display[2]);
    }

    #[test]
    fn test_dxyn_draw_hires_no_collision() {
        let mut chip8 = Chip8::new();
        chip8.hi_res = true;
        chip8.memory[0x300] = 0x80;
        chip8.index = 0x300;
        chip8.v[0] = 0;
        chip8.v[1] = 0;
        run_instruction(&mut chip8, 0xD0, 0x11);
        assert_eq!(chip8.v[0xF], 0);
        assert!(chip8.display[0]);
        assert!(!chip8.display[1]);
    }

    #[test]
    fn test_dxyn_draw_hires_collision() {
        let mut chip8 = Chip8::new();
        chip8.hi_res = true;
        chip8.display[0] = true;
        chip8.memory[0x300] = 0x80;
        chip8.index = 0x300;
        chip8.v[0] = 0;
        chip8.v[1] = 0;
        run_instruction(&mut chip8, 0xD0, 0x11);
        assert_eq!(chip8.v[0xF], 1);
        assert!(!chip8.display[0]);
    }

    #[test]
    fn test_dxy0_draw_16x16() {
        let mut chip8 = Chip8::new();
        chip8.hi_res = true;
        chip8.memory[0x300] = 0x80;
        chip8.memory[0x301] = 0x00;
        for byte_index in 2..32 {
            chip8.memory[0x300 + byte_index] = 0x00;
        }
        chip8.index = 0x300;
        chip8.v[0] = 0;
        chip8.v[1] = 0;
        run_instruction(&mut chip8, 0xD0, 0x10);
        assert!(chip8.display[0]);
        assert!(!chip8.display[1]);
        assert_eq!(chip8.v[0xF], 0);
    }

    #[test]
    fn test_dxy0_draw_16x16_collision() {
        let mut chip8 = Chip8::new();
        chip8.hi_res = true;
        chip8.display[0] = true;
        chip8.memory[0x300] = 0x80;
        chip8.memory[0x301] = 0x00;
        for byte_index in 2..32 {
            chip8.memory[0x300 + byte_index] = 0x00;
        }
        chip8.index = 0x300;
        chip8.v[0] = 0;
        chip8.v[1] = 0;
        run_instruction(&mut chip8, 0xD0, 0x10);
        assert!(!chip8.display[0]);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_dxyn_draw_hires_wrapping() {
        let mut chip8 = Chip8::new();
        chip8.hi_res = true;
        chip8.memory[0x300] = 0x80;
        chip8.index = 0x300;
        chip8.v[0] = 127;
        chip8.v[1] = 63;
        run_instruction(&mut chip8, 0xD0, 0x11);
        assert!(chip8.display[63 * DISPLAY_WIDTH + 127]);
    }

    #[test]
    fn test_ex9e_skp_pressed() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 5;
        chip8.keypad[5] = true;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0xE0, 0x9E);
        assert_eq!(chip8.pc, pc_before + 4);
    }

    #[test]
    fn test_ex9e_skp_not_pressed() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 5;
        chip8.keypad[5] = false;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0xE0, 0x9E);
        assert_eq!(chip8.pc, pc_before + 2);
    }

    #[test]
    fn test_exa1_sknp_not_pressed() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 5;
        chip8.keypad[5] = false;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0xE0, 0xA1);
        assert_eq!(chip8.pc, pc_before + 4);
    }

    #[test]
    fn test_exa1_sknp_pressed() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 5;
        chip8.keypad[5] = true;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0xE0, 0xA1);
        assert_eq!(chip8.pc, pc_before + 2);
    }

    #[test]
    fn test_fx07_ld_vx_dt() {
        let mut chip8 = Chip8::new();
        chip8.delay_timer = 42;
        run_instruction(&mut chip8, 0xF0, 0x07);
        assert_eq!(chip8.v[0], 42);
    }

    #[test]
    fn test_fx0a_ld_vx_k_waiting() {
        let mut chip8 = Chip8::new();
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0xF0, 0x0A);
        assert_eq!(chip8.waiting_for_key, Some(0));
        chip8.tick();
        assert_eq!(chip8.pc, pc_before + 2);
    }

    #[test]
    fn test_fx0a_ld_vx_k_key_pressed() {
        let mut chip8 = Chip8::new();
        run_instruction(&mut chip8, 0xF0, 0x0A);
        assert!(chip8.waiting_for_key.is_some());
        chip8.keypad[7] = true;
        chip8.tick();
        assert_eq!(chip8.v[0], 7);
        assert!(chip8.waiting_for_key.is_none());
    }

    #[test]
    fn test_fx15_ld_dt_vx() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 30;
        run_instruction(&mut chip8, 0xF0, 0x15);
        assert_eq!(chip8.delay_timer, 30);
    }

    #[test]
    fn test_fx18_ld_st_vx() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 15;
        run_instruction(&mut chip8, 0xF0, 0x18);
        assert_eq!(chip8.sound_timer, 15);
    }

    #[test]
    fn test_fx1e_add_i_vx() {
        let mut chip8 = Chip8::new();
        chip8.index = 0x100;
        chip8.v[0] = 0x10;
        run_instruction(&mut chip8, 0xF0, 0x1E);
        assert_eq!(chip8.index, 0x110);
    }

    #[test]
    fn test_fx29_ld_f_vx() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0;
        run_instruction(&mut chip8, 0xF0, 0x29);
        assert_eq!(chip8.index, 0);

        chip8.v[0] = 1;
        run_instruction(&mut chip8, 0xF0, 0x29);
        assert_eq!(chip8.index, 5);

        chip8.v[0] = 0xF;
        run_instruction(&mut chip8, 0xF0, 0x29);
        assert_eq!(chip8.index, 75);
    }

    #[test]
    fn test_fx33_bcd() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 123;
        chip8.index = 0x300;
        run_instruction(&mut chip8, 0xF0, 0x33);
        assert_eq!(chip8.memory[0x300], 1);
        assert_eq!(chip8.memory[0x301], 2);
        assert_eq!(chip8.memory[0x302], 3);
    }

    #[test]
    fn test_fx33_bcd_255() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 255;
        chip8.index = 0x300;
        run_instruction(&mut chip8, 0xF0, 0x33);
        assert_eq!(chip8.memory[0x300], 2);
        assert_eq!(chip8.memory[0x301], 5);
        assert_eq!(chip8.memory[0x302], 5);
    }

    #[test]
    fn test_fx55_ld_i_vx() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0x11;
        chip8.v[1] = 0x22;
        chip8.v[2] = 0x33;
        chip8.index = 0x300;
        run_instruction(&mut chip8, 0xF2, 0x55);
        assert_eq!(chip8.memory[0x300], 0x11);
        assert_eq!(chip8.memory[0x301], 0x22);
        assert_eq!(chip8.memory[0x302], 0x33);
    }

    #[test]
    fn test_fx65_ld_vx_i() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x300] = 0xAA;
        chip8.memory[0x301] = 0xBB;
        chip8.memory[0x302] = 0xCC;
        chip8.index = 0x300;
        run_instruction(&mut chip8, 0xF2, 0x65);
        assert_eq!(chip8.v[0], 0xAA);
        assert_eq!(chip8.v[1], 0xBB);
        assert_eq!(chip8.v[2], 0xCC);
    }

    #[test]
    fn test_tick_timers() {
        let mut chip8 = Chip8::new();
        chip8.delay_timer = 5;
        chip8.sound_timer = 3;
        chip8.tick_timers();
        assert_eq!(chip8.delay_timer, 4);
        assert_eq!(chip8.sound_timer, 2);
    }

    #[test]
    fn test_tick_timers_at_zero() {
        let mut chip8 = Chip8::new();
        chip8.delay_timer = 0;
        chip8.sound_timer = 0;
        chip8.tick_timers();
        assert_eq!(chip8.delay_timer, 0);
        assert_eq!(chip8.sound_timer, 0);
    }

    #[test]
    fn test_reset() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0xFF;
        chip8.pc = 0x400;
        chip8.sp = 5;
        chip8.display[0] = true;
        chip8.hi_res = true;
        chip8.halted = true;
        chip8.rpl_flags[0] = 0x42;
        chip8.reset();
        assert_eq!(chip8.pc, PROGRAM_START);
        assert_eq!(chip8.sp, 0);
        assert_eq!(chip8.v[0], 0);
        assert!(!chip8.display[0]);
        assert!(!chip8.hi_res);
        assert!(!chip8.halted);
        assert_eq!(chip8.rpl_flags[0], 0);
    }

    #[test]
    fn test_call_and_ret() {
        let mut chip8 = Chip8::new();
        let original_pc = chip8.pc;
        run_instruction(&mut chip8, 0x24, 0x00);
        assert_eq!(chip8.pc, 0x400);
        assert_eq!(chip8.sp, 1);
        chip8.memory[0x400] = 0x00;
        chip8.memory[0x401] = 0xEE;
        chip8.tick();
        assert_eq!(chip8.pc, original_pc + 2);
        assert_eq!(chip8.sp, 0);
    }

    #[test]
    fn test_dxyn_draw_lores_xor_erase() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x300] = 0xFF;
        chip8.index = 0x300;
        chip8.v[0] = 0;
        chip8.v[1] = 0;
        run_instruction(&mut chip8, 0xD0, 0x11);
        for col in 0..8 {
            assert!(chip8.display[col * 2]);
            assert!(chip8.display[col * 2 + 1]);
            assert!(chip8.display[DISPLAY_WIDTH + col * 2]);
            assert!(chip8.display[DISPLAY_WIDTH + col * 2 + 1]);
        }

        chip8.pc = PROGRAM_START;
        chip8.index = 0x300;
        run_instruction(&mut chip8, 0xD0, 0x11);
        for col in 0..16 {
            assert!(!chip8.display[col]);
            assert!(!chip8.display[DISPLAY_WIDTH + col]);
        }
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_00cn_scroll_down() {
        let mut chip8 = Chip8::new();
        chip8.display[5] = true;
        run_instruction(&mut chip8, 0x00, 0xC2);
        assert!(!chip8.display[5]);
        assert!(chip8.display[2 * DISPLAY_WIDTH + 5]);
        assert!(chip8.draw_flag);
    }

    #[test]
    fn test_00cn_scroll_down_clears_top() {
        let mut chip8 = Chip8::new();
        chip8.display[0] = true;
        chip8.display[DISPLAY_WIDTH + 3] = true;
        run_instruction(&mut chip8, 0x00, 0xC3);
        assert!(!chip8.display[0]);
        assert!(!chip8.display[DISPLAY_WIDTH + 3]);
        assert!(chip8.display[3 * DISPLAY_WIDTH]);
        assert!(chip8.display[4 * DISPLAY_WIDTH + 3]);
    }

    #[test]
    fn test_00fb_scroll_right() {
        let mut chip8 = Chip8::new();
        chip8.display[10] = true;
        run_instruction(&mut chip8, 0x00, 0xFB);
        assert!(!chip8.display[10]);
        assert!(chip8.display[14]);
        assert!(!chip8.display[0]);
        assert!(!chip8.display[1]);
        assert!(!chip8.display[2]);
        assert!(!chip8.display[3]);
        assert!(chip8.draw_flag);
    }

    #[test]
    fn test_00fb_scroll_right_clears_left() {
        let mut chip8 = Chip8::new();
        chip8.display[0] = true;
        chip8.display[1] = true;
        chip8.display[2] = true;
        chip8.display[3] = true;
        run_instruction(&mut chip8, 0x00, 0xFB);
        assert!(chip8.display[4]);
        assert!(chip8.display[5]);
        assert!(chip8.display[6]);
        assert!(chip8.display[7]);
        assert!(!chip8.display[0]);
        assert!(!chip8.display[1]);
        assert!(!chip8.display[2]);
        assert!(!chip8.display[3]);
    }

    #[test]
    fn test_00fc_scroll_left() {
        let mut chip8 = Chip8::new();
        chip8.display[10] = true;
        run_instruction(&mut chip8, 0x00, 0xFC);
        assert!(!chip8.display[10]);
        assert!(chip8.display[6]);
        assert!(chip8.draw_flag);
    }

    #[test]
    fn test_00fc_scroll_left_clears_right() {
        let mut chip8 = Chip8::new();
        chip8.display[DISPLAY_WIDTH - 1] = true;
        chip8.display[DISPLAY_WIDTH - 2] = true;
        run_instruction(&mut chip8, 0x00, 0xFC);
        assert!(chip8.display[DISPLAY_WIDTH - 5]);
        assert!(chip8.display[DISPLAY_WIDTH - 6]);
        assert!(!chip8.display[DISPLAY_WIDTH - 1]);
        assert!(!chip8.display[DISPLAY_WIDTH - 2]);
        assert!(!chip8.display[DISPLAY_WIDTH - 3]);
        assert!(!chip8.display[DISPLAY_WIDTH - 4]);
    }

    #[test]
    fn test_00fd_exit() {
        let mut chip8 = Chip8::new();
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x00, 0xFD);
        assert!(chip8.halted);
        assert_eq!(chip8.pc, pc_before + 2);
        chip8.tick();
        assert_eq!(chip8.pc, pc_before + 2);
    }

    #[test]
    fn test_00fe_low_res() {
        let mut chip8 = Chip8::new();
        chip8.hi_res = true;
        chip8.display[100] = true;
        run_instruction(&mut chip8, 0x00, 0xFE);
        assert!(!chip8.hi_res);
        assert!(chip8.display.iter().all(|&pixel| !pixel));
        assert!(chip8.draw_flag);
    }

    #[test]
    fn test_00ff_high_res() {
        let mut chip8 = Chip8::new();
        chip8.display[100] = true;
        run_instruction(&mut chip8, 0x00, 0xFF);
        assert!(chip8.hi_res);
        assert!(chip8.display.iter().all(|&pixel| !pixel));
        assert!(chip8.draw_flag);
    }

    #[test]
    fn test_fx30_ld_hf_vx() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0;
        run_instruction(&mut chip8, 0xF0, 0x30);
        assert_eq!(chip8.index, HIRES_FONT_START);

        chip8.v[0] = 5;
        run_instruction(&mut chip8, 0xF0, 0x30);
        assert_eq!(chip8.index, HIRES_FONT_START + 50);

        chip8.v[0] = 9;
        run_instruction(&mut chip8, 0xF0, 0x30);
        assert_eq!(chip8.index, HIRES_FONT_START + 90);
    }

    #[test]
    fn test_fx75_store_rpl_flags() {
        let mut chip8 = Chip8::new();
        chip8.v[0] = 0xAA;
        chip8.v[1] = 0xBB;
        chip8.v[2] = 0xCC;
        run_instruction(&mut chip8, 0xF2, 0x75);
        assert_eq!(chip8.rpl_flags[0], 0xAA);
        assert_eq!(chip8.rpl_flags[1], 0xBB);
        assert_eq!(chip8.rpl_flags[2], 0xCC);
    }

    #[test]
    fn test_fx75_store_rpl_flags_clamped() {
        let mut chip8 = Chip8::new();
        for register in 0..16 {
            chip8.v[register] = register as u8 + 1;
        }
        run_instruction(&mut chip8, 0xFF, 0x75);
        for register in 0..NUM_RPL_FLAGS {
            assert_eq!(chip8.rpl_flags[register], register as u8 + 1);
        }
    }

    #[test]
    fn test_fx85_load_rpl_flags() {
        let mut chip8 = Chip8::new();
        chip8.rpl_flags[0] = 0x11;
        chip8.rpl_flags[1] = 0x22;
        chip8.rpl_flags[2] = 0x33;
        run_instruction(&mut chip8, 0xF2, 0x85);
        assert_eq!(chip8.v[0], 0x11);
        assert_eq!(chip8.v[1], 0x22);
        assert_eq!(chip8.v[2], 0x33);
    }

    #[test]
    fn test_fx85_load_rpl_flags_clamped() {
        let mut chip8 = Chip8::new();
        for register in 0..NUM_RPL_FLAGS {
            chip8.rpl_flags[register] = (register as u8 + 1) * 10;
        }
        run_instruction(&mut chip8, 0xFF, 0x85);
        for register in 0..NUM_RPL_FLAGS {
            assert_eq!(chip8.v[register], (register as u8 + 1) * 10);
        }
    }

    #[test]
    fn test_halted_prevents_execution() {
        let mut chip8 = Chip8::new();
        chip8.halted = true;
        let pc_before = chip8.pc;
        run_instruction(&mut chip8, 0x60, 0x42);
        assert_eq!(chip8.pc, pc_before);
        assert_eq!(chip8.v[0], 0);
    }
}
