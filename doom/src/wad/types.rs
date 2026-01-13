use super::name::WadName;
use serde::Deserialize;

pub type LightLevel = i16;
pub type LinedefFlags = u16;
pub type SectorId = u16;
pub type SectorTag = u16;
pub type SectorType = u16;
pub type SidedefId = i16;
pub type SpecialType = u16;
pub type ThingFlags = u16;
pub type ThingType = u16;
pub type VertexId = u16;
pub type WadCoord = i16;
pub type SegId = u16;
pub type LinedefId = u16;
pub type ChildId = u16;

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadInfo {
    pub identifier: [u8; 4],
    pub num_lumps: i32,
    pub info_table_offset: i32,
}

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadLump {
    pub file_pos: i32,
    pub size: i32,
    pub name: WadName,
}

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadThing {
    pub x: WadCoord,
    pub y: WadCoord,
    pub angle: WadCoord,
    pub thing_type: ThingType,
    pub flags: ThingFlags,
}

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadVertex {
    pub x: WadCoord,
    pub y: WadCoord,
}

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadLinedef {
    pub start_vertex: VertexId,
    pub end_vertex: VertexId,
    pub flags: LinedefFlags,
    pub special_type: SpecialType,
    pub sector_tag: SectorTag,
    pub right_side: SidedefId,
    pub left_side: SidedefId,
}

impl WadLinedef {
    pub fn upper_unpegged(&self) -> bool {
        self.flags & 0x0008 != 0
    }

    pub fn lower_unpegged(&self) -> bool {
        self.flags & 0x0010 != 0
    }
}

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadSidedef {
    pub x_offset: WadCoord,
    pub y_offset: WadCoord,
    pub upper_texture: WadName,
    pub lower_texture: WadName,
    pub middle_texture: WadName,
    pub sector: SectorId,
}

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadSector {
    pub floor_height: WadCoord,
    pub ceiling_height: WadCoord,
    pub floor_texture: WadName,
    pub ceiling_texture: WadName,
    pub light: LightLevel,
    pub sector_type: SectorType,
    pub tag: SectorTag,
}

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadSubsector {
    pub num_segs: u16,
    pub first_seg: SegId,
}

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadSeg {
    pub start_vertex: VertexId,
    pub end_vertex: VertexId,
    pub angle: u16,
    pub linedef: LinedefId,
    pub direction: u16,
    pub offset: u16,
}

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct WadNode {
    pub line_x: WadCoord,
    pub line_y: WadCoord,
    pub step_x: WadCoord,
    pub step_y: WadCoord,
    pub right_y_max: WadCoord,
    pub right_y_min: WadCoord,
    pub right_x_max: WadCoord,
    pub right_x_min: WadCoord,
    pub left_y_max: WadCoord,
    pub left_y_min: WadCoord,
    pub left_x_max: WadCoord,
    pub left_x_min: WadCoord,
    pub right: ChildId,
    pub left: ChildId,
}

pub const PALETTE_SIZE: usize = 256 * 3;
pub const COLORMAP_SIZE: usize = 256;

pub struct Palette(pub [u8; PALETTE_SIZE]);

impl Default for Palette {
    fn default() -> Self {
        Palette([0u8; PALETTE_SIZE])
    }
}

impl AsMut<[u8]> for Palette {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

pub struct Colormap(pub [u8; COLORMAP_SIZE]);

impl Default for Colormap {
    fn default() -> Self {
        Colormap([0u8; COLORMAP_SIZE])
    }
}

impl AsMut<[u8]> for Colormap {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
