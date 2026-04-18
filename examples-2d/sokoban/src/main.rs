use image::GenericImageView;
use nightshade::prelude::*;

const TILE_SIZE: f32 = 64.0;

const SLOT_TILE_SHEET: u32 = 0;
const SLOT_CRATE: u32 = 1;
const SLOT_CRATE_DARK: u32 = 2;
const SLOT_PLAYER_FRONT: u32 = 3;
const SLOT_PLAYER_BACK: u32 = 4;
const SLOT_PLAYER_LEFT: u32 = 5;
const SLOT_PLAYER_RIGHT: u32 = 6;

const TILE_FLOOR: u32 = 0;
const TILE_WALL: u32 = 1;
const TILE_TARGET: u32 = 2;
const SHEET_COLUMNS: u32 = 4;
const SHEET_ROWS: u32 = 1;

const LAYER_FLOOR: f32 = 0.0;
const LAYER_TARGET: f32 = 1.0;
const LAYER_OBJECTS: f32 = 2.0;
const LAYER_PLAYER: f32 = 3.0;

fn build_tile_sheet(world: &mut World) -> Vec2 {
    let tile_images: Vec<(&[u8], u32)> = vec![
        (include_bytes!("../assets/floor.png"), TILE_FLOOR),
        (include_bytes!("../assets/wall.png"), TILE_WALL),
        (include_bytes!("../assets/target.png"), TILE_TARGET),
    ];

    let mut decoded: Vec<(image::DynamicImage, u32)> = Vec::new();
    let mut cell_width: u32 = 0;
    let mut cell_height: u32 = 0;

    for (bytes, tile_id) in &tile_images {
        let image = image::load_from_memory(bytes).expect("Failed to decode tile image");
        let (width, height) = image.dimensions();
        cell_width = cell_width.max(width);
        cell_height = cell_height.max(height);
        decoded.push((image, *tile_id));
    }

    let sheet_pixel_width = SHEET_COLUMNS * cell_width;
    let sheet_pixel_height = SHEET_ROWS * cell_height;
    let mut sheet = image::RgbaImage::new(sheet_pixel_width, sheet_pixel_height);

    for (image, tile_id) in &decoded {
        let (width, height) = image.dimensions();
        let column = tile_id % SHEET_COLUMNS;
        let row = tile_id / SHEET_COLUMNS;
        let dest_x = column * cell_width + (cell_width - width) / 2;
        let dest_y = row * cell_height + (cell_height - height) / 2;
        image::imageops::overlay(&mut sheet, &image.to_rgba8(), dest_x as i64, dest_y as i64);
    }

    world
        .resources
        .command_queue
        .push(WorldCommand::UploadSpriteTexture {
            slot: SLOT_TILE_SHEET,
            rgba_data: sheet.into_raw(),
            width: sheet_pixel_width,
            height: sheet_pixel_height,
        });

    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let half_texel_x = 0.5 / atlas_slot_size.0 as f32;
    let half_texel_y = 0.5 / atlas_slot_size.1 as f32;
    Vec2::new(
        sheet_pixel_width as f32 / atlas_slot_size.0 as f32 - half_texel_x,
        sheet_pixel_height as f32 / atlas_slot_size.1 as f32 - half_texel_y,
    )
}

struct SpriteTextureEntry {
    slot: u32,
    bytes: &'static [u8],
}

fn load_sprite_textures(world: &mut World) -> Vec<Vec2> {
    let entries = [
        SpriteTextureEntry {
            slot: SLOT_CRATE,
            bytes: include_bytes!("../assets/crate.png"),
        },
        SpriteTextureEntry {
            slot: SLOT_CRATE_DARK,
            bytes: include_bytes!("../assets/crate_dark.png"),
        },
        SpriteTextureEntry {
            slot: SLOT_PLAYER_FRONT,
            bytes: include_bytes!("../assets/player_front.png"),
        },
        SpriteTextureEntry {
            slot: SLOT_PLAYER_BACK,
            bytes: include_bytes!("../assets/player_back.png"),
        },
        SpriteTextureEntry {
            slot: SLOT_PLAYER_LEFT,
            bytes: include_bytes!("../assets/player_left.png"),
        },
        SpriteTextureEntry {
            slot: SLOT_PLAYER_RIGHT,
            bytes: include_bytes!("../assets/player_right.png"),
        },
    ];

    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let mut uv_max_table = vec![Vec2::new(1.0, 1.0); 128];

    for entry in &entries {
        let image = image::load_from_memory(entry.bytes).expect("Failed to decode image");
        let (width, height) = image.dimensions();
        let rgba = image.to_rgba8().into_raw();

        world
            .resources
            .command_queue
            .push(WorldCommand::UploadSpriteTexture {
                slot: entry.slot,
                rgba_data: rgba,
                width,
                height,
            });

        let half_texel_x = 0.5 / atlas_slot_size.0 as f32;
        let half_texel_y = 0.5 / atlas_slot_size.1 as f32;
        uv_max_table[entry.slot as usize] = Vec2::new(
            width as f32 / atlas_slot_size.0 as f32 - half_texel_x,
            height as f32 / atlas_slot_size.1 as f32 - half_texel_y,
        );
    }

    uv_max_table
}

fn uv_for_slot(uv_max_table: &[Vec2], slot: u32) -> (Vec2, Vec2) {
    let atlas_slot_size = nightshade::render::SPRITE_ATLAS_SLOT_SIZE;
    let half_texel = Vec2::new(
        0.5 / atlas_slot_size.0 as f32,
        0.5 / atlas_slot_size.1 as f32,
    );
    (half_texel, uv_max_table[slot as usize])
}

fn spawn_textured_sprite(
    world: &mut World,
    position: Vec2,
    depth: f32,
    size: Vec2,
    texture_slot: u32,
    uv_max_table: &[Vec2],
) -> Entity {
    let entity = spawn_sprite(world, position, size);
    let (uv_min, uv_max) = uv_for_slot(uv_max_table, texture_slot);
    if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
        sprite.depth = depth;
        sprite.texture_index = texture_slot;
        sprite.texture_index2 = texture_slot;
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
    }
    entity
}

fn set_sprite_texture(world: &mut World, entity: Entity, slot: u32, uv_max_table: &[Vec2]) {
    let (uv_min, uv_max) = uv_for_slot(uv_max_table, slot);
    if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
        sprite.texture_index = slot;
        sprite.texture_index2 = slot;
        sprite.uv_min = uv_min;
        sprite.uv_max = uv_max;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct GridPos {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    fn texture_slot(self) -> u32 {
        match self {
            Direction::Down => SLOT_PLAYER_FRONT,
            Direction::Up => SLOT_PLAYER_BACK,
            Direction::Left => SLOT_PLAYER_LEFT,
            Direction::Right => SLOT_PLAYER_RIGHT,
        }
    }
}

#[derive(Clone)]
struct UndoState {
    player_pos: GridPos,
    crate_positions: Vec<GridPos>,
}

struct LevelData {
    width: usize,
    height: usize,
    walls: Vec<bool>,
    targets: Vec<GridPos>,
    initial_player: GridPos,
    initial_crates: Vec<GridPos>,
}

fn parse_level(data: &str) -> LevelData {
    let lines: Vec<&str> = data.lines().collect();
    let height = lines.len();
    let width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let mut walls = vec![false; width * height];
    let mut targets = Vec::new();
    let mut initial_player = GridPos { x: 0, y: 0 };
    let mut initial_crates = Vec::new();

    for (row, line) in lines.iter().enumerate() {
        for (col, character) in line.chars().enumerate() {
            let pos = GridPos {
                x: col as i32,
                y: row as i32,
            };
            match character {
                '#' => walls[row * width + col] = true,
                '@' => initial_player = pos,
                '+' => {
                    initial_player = pos;
                    targets.push(pos);
                }
                '$' => initial_crates.push(pos),
                '*' => {
                    initial_crates.push(pos);
                    targets.push(pos);
                }
                '.' => targets.push(pos),
                _ => {}
            }
        }
    }

    LevelData {
        width,
        height,
        walls,
        targets,
        initial_player,
        initial_crates,
    }
}

const LEVELS: &[&str] = &[
    "    #####
    #   #
    #$  #
  ###  $##
  #  $ $ #
### # ## #   ######
#   # ## #####  ..#
# $  $          ..#
##### ### #@##  ..#
    #     #########
    #######",
    "############
#..  #     ###
#..  # $  $  #
#..  #$####  #
#..    @ ##  #
#..  # #  $ ##
###### ##$ $ #
  # $  $ $ $ #
  #    #     #
  ############",
    "        ########
        #     @#
        # $#$ ##
        # $  $#
        ##$ $ #
######### $ # ###
#....  ## $  $  #
##...    $  $   #
#....  ##########
########",
    "           ########
           #  ....#
############  ....#
#    #  $ $   ....#
# $$$#$  $ #  ....#
#  $     $  #  ...#
# $$ #$ $ $ ########
#  $ #     ##
## #########
#    #    ##
#     $   ##
#  $$#$$  @#
#    #    ##
###########",
    "        #####
        #   #####
        # #$##  #
        #     $ #
######### ###   #
#....  ## $  $###
#....    $ $$ ##
#....  ##$  $ @#
#########  $  ##
        # $ $  #
        ### ## #
          #    #
          ######",
];

struct Sokoban {
    camera_entity: Option<Entity>,
    uv_max_table: Vec<Vec2>,
    tile_sheet_uv_max: Vec2,
    current_level: usize,
    level_data: Option<LevelData>,
    player_pos: GridPos,
    player_direction: Direction,
    crate_positions: Vec<GridPos>,
    undo_stack: Vec<UndoState>,
    level_complete: bool,
    floor_tilemap_entity: Option<Entity>,
    target_tilemap_entity: Option<Entity>,
    crate_entities: Vec<Entity>,
    player_entity: Option<Entity>,
    score_hud: Option<Entity>,
    level_hud: Option<Entity>,
    message_hud: Option<Entity>,
    moves: u32,
    pushes: u32,
    initialized: bool,
}

impl Default for Sokoban {
    fn default() -> Self {
        Self {
            camera_entity: None,
            uv_max_table: Vec::new(),
            tile_sheet_uv_max: Vec2::new(1.0, 1.0),
            current_level: 0,
            level_data: None,
            player_pos: GridPos { x: 0, y: 0 },
            player_direction: Direction::Down,
            crate_positions: Vec::new(),
            undo_stack: Vec::new(),
            level_complete: false,
            floor_tilemap_entity: None,
            target_tilemap_entity: None,
            crate_entities: Vec::new(),
            player_entity: None,
            score_hud: None,
            level_hud: None,
            message_hud: None,
            moves: 0,
            pushes: 0,
            initialized: false,
        }
    }
}

impl Sokoban {
    fn is_wall(&self, pos: GridPos) -> bool {
        let level = self.level_data.as_ref().unwrap();
        if pos.x < 0 || pos.y < 0 || pos.x >= level.width as i32 || pos.y >= level.height as i32 {
            return true;
        }
        level.walls[pos.y as usize * level.width + pos.x as usize]
    }

    fn crate_at(&self, pos: GridPos) -> Option<usize> {
        self.crate_positions
            .iter()
            .position(|crate_pos| *crate_pos == pos)
    }

    fn try_move(&mut self, direction: Direction) {
        if self.level_complete {
            return;
        }

        self.player_direction = direction;
        let (delta_x, delta_y) = direction.delta();
        let target = GridPos {
            x: self.player_pos.x + delta_x,
            y: self.player_pos.y + delta_y,
        };

        if self.is_wall(target) {
            return;
        }

        if let Some(crate_index) = self.crate_at(target) {
            let beyond = GridPos {
                x: target.x + delta_x,
                y: target.y + delta_y,
            };
            if self.is_wall(beyond) || self.crate_at(beyond).is_some() {
                return;
            }

            self.undo_stack.push(UndoState {
                player_pos: self.player_pos,
                crate_positions: self.crate_positions.clone(),
            });

            self.crate_positions[crate_index] = beyond;
            self.player_pos = target;
            self.moves += 1;
            self.pushes += 1;
        } else {
            self.undo_stack.push(UndoState {
                player_pos: self.player_pos,
                crate_positions: self.crate_positions.clone(),
            });

            self.player_pos = target;
            self.moves += 1;
        }

        self.check_win();
    }

    fn undo(&mut self) {
        if self.level_complete {
            return;
        }
        if let Some(state) = self.undo_stack.pop() {
            self.player_pos = state.player_pos;
            self.crate_positions = state.crate_positions;
            if self.moves > 0 {
                self.moves -= 1;
            }
        }
    }

    fn check_win(&mut self) {
        let level = self.level_data.as_ref().unwrap();
        let all_on_target = level
            .targets
            .iter()
            .all(|target| self.crate_positions.contains(target));
        if all_on_target && !level.targets.is_empty() {
            self.level_complete = true;
        }
    }

    fn is_floor_neighbor(&self, level: &LevelData, pos: GridPos) -> bool {
        for delta_y in -1..=1_i32 {
            for delta_x in -1..=1_i32 {
                let neighbor_x = pos.x + delta_x;
                let neighbor_y = pos.y + delta_y;
                if neighbor_x >= 0
                    && neighbor_y >= 0
                    && neighbor_x < level.width as i32
                    && neighbor_y < level.height as i32
                {
                    let index = neighbor_y as usize * level.width + neighbor_x as usize;
                    if level.walls[index] {
                        return true;
                    }
                    let neighbor_pos = GridPos {
                        x: neighbor_x,
                        y: neighbor_y,
                    };
                    if level.targets.contains(&neighbor_pos)
                        || level.initial_crates.contains(&neighbor_pos)
                        || level.initial_player == neighbor_pos
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn load_level(&mut self, world: &mut World) {
        self.despawn_level(world);

        let level_index = self.current_level % LEVELS.len();
        let level = parse_level(LEVELS[level_index]);

        self.player_pos = level.initial_player;
        self.crate_positions = level.initial_crates.clone();
        self.player_direction = Direction::Down;
        self.undo_stack.clear();
        self.level_complete = false;
        self.moves = 0;
        self.pushes = 0;

        let grid_width = level.width as u32;
        let grid_height = level.height as u32;
        let tilemap_position = Vec2::new(
            -(level.width as f32 * TILE_SIZE / 2.0),
            -(level.height as f32 * TILE_SIZE / 2.0),
        );
        let tile_size = Vec2::new(TILE_SIZE, TILE_SIZE);

        let floor_entity =
            spawn_tilemap(world, tilemap_position, tile_size, grid_width, grid_height);
        if let Some(tilemap) = world.sprite2d.get_tilemap_mut(floor_entity) {
            tilemap.texture_index = SLOT_TILE_SHEET;
            tilemap.sheet_columns = SHEET_COLUMNS;
            tilemap.sheet_rows = SHEET_ROWS;
            tilemap.depth = LAYER_FLOOR;
            tilemap.uv_max = self.tile_sheet_uv_max;

            for row in 0..level.height {
                for col in 0..level.width {
                    let pos = GridPos {
                        x: col as i32,
                        y: row as i32,
                    };
                    let tilemap_row = (grid_height - 1) - row as u32;

                    let is_wall = level.walls[row * level.width + col];
                    if is_wall {
                        tilemap.set_tile(col as u32, tilemap_row, Some(TileData::new(TILE_WALL)));
                    } else {
                        let has_content = level.targets.contains(&pos)
                            || level.initial_crates.contains(&pos)
                            || level.initial_player == pos
                            || self.is_floor_neighbor(&level, pos);

                        if has_content {
                            tilemap.set_tile(
                                col as u32,
                                tilemap_row,
                                Some(TileData::new(TILE_FLOOR)),
                            );
                        }
                    }
                }
            }
        }
        self.floor_tilemap_entity = Some(floor_entity);

        let target_entity =
            spawn_tilemap(world, tilemap_position, tile_size, grid_width, grid_height);
        if let Some(tilemap) = world.sprite2d.get_tilemap_mut(target_entity) {
            tilemap.texture_index = SLOT_TILE_SHEET;
            tilemap.sheet_columns = SHEET_COLUMNS;
            tilemap.sheet_rows = SHEET_ROWS;
            tilemap.depth = LAYER_TARGET;
            tilemap.uv_max = self.tile_sheet_uv_max;

            for target in &level.targets {
                let tilemap_row = (grid_height - 1) - target.y as u32;
                tilemap.set_tile(
                    target.x as u32,
                    tilemap_row,
                    Some(TileData::new(TILE_TARGET)),
                );
            }
        }
        self.target_tilemap_entity = Some(target_entity);

        for _ in &self.crate_positions {
            let entity = spawn_textured_sprite(
                world,
                Vec2::new(0.0, 0.0),
                LAYER_OBJECTS,
                Vec2::new(TILE_SIZE, TILE_SIZE),
                SLOT_CRATE,
                &self.uv_max_table,
            );
            self.crate_entities.push(entity);
        }

        let player_entity = spawn_textured_sprite(
            world,
            Vec2::new(0.0, 0.0),
            LAYER_PLAYER,
            Vec2::new(TILE_SIZE, TILE_SIZE),
            SLOT_PLAYER_FRONT,
            &self.uv_max_table,
        );
        self.player_entity = Some(player_entity);

        self.level_data = Some(level);
    }

    fn grid_to_world_with_level(&self, level: &LevelData, pos: GridPos) -> Vec2 {
        let offset_x = level.width as f32 * TILE_SIZE / 2.0;
        let offset_y = level.height as f32 * TILE_SIZE / 2.0;
        Vec2::new(
            pos.x as f32 * TILE_SIZE - offset_x + TILE_SIZE / 2.0,
            offset_y - pos.y as f32 * TILE_SIZE - TILE_SIZE / 2.0,
        )
    }

    fn despawn_level(&mut self, world: &mut World) {
        let mut to_despawn = Vec::new();
        if let Some(entity) = self.floor_tilemap_entity {
            to_despawn.push(entity);
        }
        if let Some(entity) = self.target_tilemap_entity {
            to_despawn.push(entity);
        }
        to_despawn.extend(&self.crate_entities);
        if let Some(player) = self.player_entity {
            to_despawn.push(player);
        }
        if !to_despawn.is_empty() {
            world.despawn_entities(&to_despawn);
        }
        self.floor_tilemap_entity = None;
        self.target_tilemap_entity = None;
        self.crate_entities.clear();
        self.player_entity = None;
    }

    fn render_sync(&mut self, world: &mut World) {
        let uv_max_table = self.uv_max_table.clone();
        let level = match &self.level_data {
            Some(level) => level,
            None => return,
        };

        if let Some(player_entity) = self.player_entity {
            let world_pos = self.grid_to_world_with_level(level, self.player_pos);
            if let Some(sprite) = world.sprite2d.get_sprite_mut(player_entity) {
                sprite.position = world_pos;
            }
            set_sprite_texture(
                world,
                player_entity,
                self.player_direction.texture_slot(),
                &uv_max_table,
            );
        }

        for (index, crate_pos) in self.crate_positions.iter().enumerate() {
            if index < self.crate_entities.len() {
                let entity = self.crate_entities[index];
                let world_pos = self.grid_to_world_with_level(level, *crate_pos);
                if let Some(sprite) = world.sprite2d.get_sprite_mut(entity) {
                    sprite.position = world_pos;
                }

                let on_target = level.targets.contains(crate_pos);
                let slot = if on_target {
                    SLOT_CRATE_DARK
                } else {
                    SLOT_CRATE
                };
                set_sprite_texture(world, entity, slot, &uv_max_table);
            }
        }
    }

    fn update_hud(&self, world: &mut World) {
        if let Some(score_entity) = self.score_hud {
            let text_index = world
                .core
                .get_text(score_entity)
                .map(|text| text.text_index);
            if let Some(text_index) = text_index {
                world.resources.text_cache.set_text(
                    text_index,
                    format!("Moves: {}  Pushes: {}", self.moves, self.pushes),
                );
                if let Some(hud_text) = world.core.get_text_mut(score_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(level_entity) = self.level_hud {
            let text_index = world
                .core
                .get_text(level_entity)
                .map(|text| text.text_index);
            if let Some(text_index) = text_index {
                world.resources.text_cache.set_text(
                    text_index,
                    format!("Level {}/{}", self.current_level + 1, LEVELS.len()),
                );
                if let Some(hud_text) = world.core.get_text_mut(level_entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(message_entity) = self.message_hud {
            let text_index = world
                .core
                .get_text(message_entity)
                .map(|text| text.text_index);
            if let Some(text_index) = text_index {
                let message = if self.level_complete {
                    if self.current_level + 1 < LEVELS.len() {
                        "Level Complete! Press N for next level".to_string()
                    } else {
                        "All levels complete! Press R to restart".to_string()
                    }
                } else {
                    "Arrows: Move  Z: Undo  R: Restart  N: Next".to_string()
                };
                world.resources.text_cache.set_text(text_index, message);
                if let Some(hud_text) = world.core.get_text_mut(message_entity) {
                    hud_text.dirty = true;
                }
            }
        }
    }
}

impl State for Sokoban {
    fn title(&self) -> &str {
        "Sokoban"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.15, 0.15, 0.2, 1.0];

        let camera = spawn_ortho_camera(world, Vec2::new(0.0, 0.0));
        self.camera_entity = Some(camera);

        self.tile_sheet_uv_max = build_tile_sheet(world);
        self.uv_max_table = load_sprite_textures(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        if !self.initialized {
            self.initialized = true;
            self.load_level(world);

            self.score_hud = Some(spawn_ui_text_with_properties(
                world,
                "Moves: 0  Pushes: 0",
                Vec2::zeros(),
                TextProperties {
                    font_size: 28.0,
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    ..Default::default()
                },
            ));

            self.level_hud = Some(spawn_ui_text_with_properties(
                world,
                "Level 1/5",
                Vec2::zeros(),
                TextProperties {
                    font_size: 28.0,
                    color: Vec4::new(1.0, 1.0, 0.5, 1.0),
                    ..Default::default()
                },
            ));

            self.message_hud = Some(spawn_ui_text_with_properties(
                world,
                "Arrows: Move  Z: Undo  R: Restart  N: Next",
                Vec2::zeros(),
                TextProperties {
                    font_size: 22.0,
                    color: Vec4::new(0.8, 0.8, 0.8, 1.0),
                    ..Default::default()
                },
            ));
        }

        escape_key_exit_system(world);
        self.render_sync(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if !matches!(key_state, KeyState::Pressed) {
            return;
        }

        match key_code {
            KeyCode::ArrowUp | KeyCode::KeyW => self.try_move(Direction::Up),
            KeyCode::ArrowDown | KeyCode::KeyS => self.try_move(Direction::Down),
            KeyCode::ArrowLeft | KeyCode::KeyA => self.try_move(Direction::Left),
            KeyCode::ArrowRight | KeyCode::KeyD => self.try_move(Direction::Right),
            KeyCode::KeyZ => self.undo(),
            KeyCode::KeyR => {
                self.load_level(world);
            }
            KeyCode::KeyN if self.level_complete => {
                self.current_level = (self.current_level + 1) % LEVELS.len();
                self.load_level(world);
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Sokoban::default())
}
