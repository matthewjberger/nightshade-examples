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
    StimPack,
    Katana,
    CyberArmor,
    EmpGrenade,
    NeuralImplant,
    CredChip,
}

impl ItemType {
    pub fn name(&self) -> &'static str {
        match self {
            ItemType::StimPack => "Stim Pack",
            ItemType::Katana => "Katana",
            ItemType::CyberArmor => "Cyber Armor",
            ItemType::EmpGrenade => "EMP Grenade",
            ItemType::NeuralImplant => "Neural Implant",
            ItemType::CredChip => "Cred Chip",
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            ItemType::StimPack => '!',
            ItemType::Katana => '/',
            ItemType::CyberArmor => ']',
            ItemType::EmpGrenade => '*',
            ItemType::NeuralImplant => '%',
            ItemType::CredChip => '$',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyType {
    #[default]
    StreetPunk,
    CorpoGuard,
    Cyborg,
    Netrunner,
    Drone,
}

impl EnemyType {
    pub fn name(&self) -> &'static str {
        match self {
            EnemyType::StreetPunk => "Street Punk",
            EnemyType::CorpoGuard => "Corpo Guard",
            EnemyType::Cyborg => "Cyborg",
            EnemyType::Netrunner => "Netrunner",
            EnemyType::Drone => "Drone",
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            EnemyType::StreetPunk => 'p',
            EnemyType::CorpoGuard => 'c',
            EnemyType::Cyborg => 'C',
            EnemyType::Netrunner => 'n',
            EnemyType::Drone => 'd',
        }
    }

    pub fn base_stats(&self) -> (i32, i32, i32, i32) {
        match self {
            EnemyType::StreetPunk => (8, 8, 2, 0),
            EnemyType::CorpoGuard => (15, 15, 3, 2),
            EnemyType::Cyborg => (35, 35, 6, 3),
            EnemyType::Netrunner => (12, 12, 5, 0),
            EnemyType::Drone => (6, 6, 4, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileType {
    #[default]
    Wall,
    Floor,
    DataPort,
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
        matches!(self.get_tile(x, y), TileType::Floor | TileType::DataPort)
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
    pub credits: u32,
    pub emp_grenades: u32,
}

impl Inventory {
    pub fn attack_bonus(&self) -> i32 {
        match self.equipped_weapon {
            Some(ItemType::Katana) => 3,
            _ => 0,
        }
    }

    pub fn defense_bonus(&self) -> i32 {
        match self.equipped_armor {
            Some(ItemType::CyberArmor) => 3,
            _ => 0,
        }
    }
}

#[derive(Clone, Default)]
pub struct GameStats {
    pub kills: u32,
    pub items_collected: u32,
    pub max_depth_reached: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
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
        stats: GameStats,
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
