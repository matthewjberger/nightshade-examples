use nightshade::prelude::*;

pub use freecs::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameState {
    #[default]
    Playing,
    PlayerDead,
    Victory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemType {
    #[default]
    HealthPotion,
    Sword,
    Shield,
}

impl ItemType {
    pub fn name(&self) -> &'static str {
        match self {
            ItemType::HealthPotion => "Health Potion",
            ItemType::Sword => "Sword",
            ItemType::Shield => "Shield",
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            ItemType::HealthPotion => '!',
            ItemType::Sword => '/',
            ItemType::Shield => ']',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyType {
    #[default]
    Goblin,
    Orc,
    Troll,
}

impl EnemyType {
    pub fn name(&self) -> &'static str {
        match self {
            EnemyType::Goblin => "Goblin",
            EnemyType::Orc => "Orc",
            EnemyType::Troll => "Troll",
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            EnemyType::Goblin => 'g',
            EnemyType::Orc => 'o',
            EnemyType::Troll => 'T',
        }
    }

    pub fn base_stats(&self) -> (i32, i32, i32, i32) {
        match self {
            EnemyType::Goblin => (10, 10, 2, 0),
            EnemyType::Orc => (20, 20, 4, 1),
            EnemyType::Troll => (40, 40, 6, 2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileType {
    #[default]
    Wall,
    Floor,
    StairsDown,
}

#[derive(Clone, Default)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TileType>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            tiles: vec![TileType::Wall; (width * height) as usize],
        }
    }

    pub fn index(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    pub fn get_tile(&self, x: i32, y: i32) -> TileType {
        if self.in_bounds(x, y) {
            self.tiles[self.index(x, y)]
        } else {
            TileType::Wall
        }
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: TileType) {
        if self.in_bounds(x, y) {
            let index = self.index(x, y);
            self.tiles[index] = tile;
        }
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        matches!(self.get_tile(x, y), TileType::Floor | TileType::StairsDown)
    }
}

#[derive(Clone, Default)]
pub struct FovMap {
    pub width: i32,
    pub height: i32,
    pub visible: Vec<bool>,
    pub explored: Vec<bool>,
}

impl FovMap {
    pub fn new(width: i32, height: i32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            visible: vec![false; size],
            explored: vec![false; size],
        }
    }

    pub fn index(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        if self.in_bounds(x, y) {
            self.visible[self.index(x, y)]
        } else {
            false
        }
    }

    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        if self.in_bounds(x, y) {
            self.explored[self.index(x, y)]
        } else {
            false
        }
    }

    pub fn set_visible(&mut self, x: i32, y: i32, value: bool) {
        if self.in_bounds(x, y) {
            let index = self.index(x, y);
            self.visible[index] = value;
            if value {
                self.explored[index] = true;
            }
        }
    }

    pub fn clear_visible(&mut self) {
        self.visible.fill(false);
    }
}

#[derive(Clone, Default)]
pub struct Inventory {
    pub items: Vec<ItemType>,
    pub equipped_weapon: Option<ItemType>,
    pub equipped_armor: Option<ItemType>,
}

impl Inventory {
    pub fn attack_bonus(&self) -> i32 {
        match self.equipped_weapon {
            Some(ItemType::Sword) => 2,
            _ => 0,
        }
    }

    pub fn defense_bonus(&self) -> i32 {
        match self.equipped_armor {
            Some(ItemType::Shield) => 2,
            _ => 0,
        }
    }
}

freecs::ecs! {
    GameWorld {
        position: Position => POSITION,
        renderable: Renderable => RENDERABLE,
        player: Player => PLAYER,
        enemy: Enemy => ENEMY,
        item: Item => ITEM,
        combat_stats: CombatStats => COMBAT_STATS,
        ai: Ai => AI,
        blocker: Blocker => BLOCKER,
    }
    GameResources {
        map: Map,
        fov_map: FovMap,
        player_entity: Option<freecs::Entity>,
        game_state: GameState,
        message_log: Vec<String>,
        current_depth: u32,
        rng_seed: u64,
        inventory: Inventory,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Renderable {
    pub glyph: char,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Player;

#[derive(Debug, Clone, Copy, Default)]
pub struct Enemy {
    pub enemy_type: EnemyType,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Item {
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CombatStats {
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Ai;

#[derive(Debug, Clone, Copy, Default)]
pub struct Blocker;
