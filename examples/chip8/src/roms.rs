pub struct BundledRom {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub const BUNDLED_ROMS: &[BundledRom] = &[
    BundledRom {
        name: "Demo",
        data: DEMO_ROM,
    },
    BundledRom {
        name: "Tetris",
        data: include_bytes!("../assets/Tetris.ch8"),
    },
    BundledRom {
        name: "Space Invaders",
        data: include_bytes!("../assets/SpaceInvaders.ch8"),
    },
];

pub const DEMO_ROM: &[u8] = &[
    // --- Draw all 16 hex digits (0-F) on the display ---
    0x00, 0xE0, // CLS
    0x60, 0x00, // LD V0, 0 (x position)
    0x61, 0x00, // LD V1, 0 (y position)
    0x62, 0x00, // LD V2, 0 (digit counter)
    0xF2, 0x29, // digit_loop: LD F, V2
    0xD0, 0x15, // DRW V0, V1, 5
    0x70, 0x06, // ADD V0, 6
    0x72, 0x01, // ADD V2, 1
    0x30, 0x3C, // SE V0, 60 (wrap if x reached 60)
    0x12, 0x18, // JP 0x218 (skip wrap)
    0x60, 0x00, // LD V0, 0 (reset x)
    0x71, 0x07, // ADD V1, 7 (next row)
    0x32, 0x10, // SE V2, 16 (done if drew all 16)
    0x12, 0x08, // JP 0x208 (digit_loop)
    // --- Draw horizontal separator at y=15 ---
    0xA2, 0x48, // LD I, 0x248 (line sprite data)
    0x60, 0x00, // LD V0, 0
    0x61, 0x0F, // LD V1, 15
    0xD0, 0x11, // line_loop: DRW V0, V1, 1
    0x70, 0x08, // ADD V0, 8
    0x30, 0x40, // SE V0, 64
    0x12, 0x22, // JP 0x222 (line_loop)
    // --- Animated hex counter in lower area ---
    0x62, 0x00, // LD V2, 0 (counter value)
    0x60, 0x1C, // LD V0, 28 (x, roughly centered)
    0x61, 0x14, // LD V1, 20 (y, below separator)
    0xF2, 0x29, // count_loop: LD F, V2
    0xD0, 0x15, // DRW V0, V1, 5 (draw digit)
    0x64, 0x0A, // LD V4, 10 (delay ~167ms at 60Hz)
    0xF4, 0x15, // LD DT, V4
    0xF4, 0x07, // wait_loop: LD V4, DT
    0x34, 0x00, // SE V4, 0
    0x12, 0x38, // JP 0x238 (wait_loop)
    0xD0, 0x15, // DRW V0, V1, 5 (erase digit via XOR)
    0x72, 0x01, // ADD V2, 1
    0x63, 0x0F, // LD V3, 0x0F
    0x82, 0x32, // AND V2, V3 (wrap counter to 0-F)
    0x12, 0x30, // JP 0x230 (count_loop)
    // --- Sprite data at 0x248 ---
    0xFF, // Line sprite: 8 pixels all on
];
