use nightshade::prelude::*;
use std::collections::HashMap;

pub use freecs::Entity;

freecs::ecs! {
    PuzzleWorld {
        engine_entity: EngineEntity => ENGINE_ENTITY,
        puzzle_piece: PuzzlePiece => PUZZLE_PIECE,
        edge_profile: EdgeProfile => EDGE_PROFILE,
        in_group: InGroup => IN_GROUP,
        local_offset: LocalOffset => LOCAL_OFFSET,
        z_order: ZOrder => Z_ORDER,
        dragging: Dragging => DRAGGING,
        piece_group: PieceGroup => PIECE_GROUP,
        group_members: GroupMembers => GROUP_MEMBERS,
    }
    PuzzleResources {
        grid_cols: u32,
        grid_rows: u32,
        piece_width: f32,
        piece_height: f32,
        snap_threshold: f32,
        z_counter: u32,
        puzzle_complete: bool,
        image_width: u32,
        image_height: u32,
        edge_types: HashMap<EdgeKey, EdgeType>,
        piece_outlines: HashMap<IVec2, Vec<Vec2>>,
        hovered_piece: Option<freecs::Entity>,
        show_board_outline: bool,
        board_outline_entity: Option<Entity>,
        tab_depth: f32,
        tab_width: f32,
        neck_width: f32,
        is_solving: bool,
        solve_queue: Vec<freecs::Entity>,
        solve_progress: f32,
        pending_cols: u32,
        pending_rows: u32,
        pending_tab_depth: f32,
        pending_tab_width: f32,
        pending_neck_width: f32,
        has_pending_changes: bool,
        victory_active: bool,
        victory_flash_index: usize,
        victory_flash_timer: f32,
        victory_text_entity: Option<Entity>,
        victory_text_lifetime: f32,
        all_piece_entities: Vec<freecs::Entity>,
        victory_lines_entity: Option<Entity>,
        victory_time: f32,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeKey {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
}

impl EdgeKey {
    pub fn new(x: i32, y: i32, direction: Direction) -> Self {
        Self { x, y, direction }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EdgeType {
    #[default]
    Flat,
    Tab,
    Blank,
}

impl EdgeType {
    pub fn inverse(&self) -> Self {
        match self {
            EdgeType::Flat => EdgeType::Flat,
            EdgeType::Tab => EdgeType::Blank,
            EdgeType::Blank => EdgeType::Tab,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EngineEntity(pub Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct PuzzlePiece {
    pub grid_pos: IVec2,
    pub rotation: u8,
    pub correct_rotation: u8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeProfile {
    pub top: EdgeType,
    pub right: EdgeType,
    pub bottom: EdgeType,
    pub left: EdgeType,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InGroup(pub freecs::Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct LocalOffset {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ZOrder(pub u32);

#[derive(Debug, Clone, Copy, Default)]
pub struct Dragging {
    pub cursor_offset_x: f32,
    pub cursor_offset_z: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PieceGroup;

#[derive(Debug, Clone, Default)]
pub struct GroupMembers {
    pub members: Vec<freecs::Entity>,
    pub world_x: f32,
    pub world_z: f32,
}
