use super::archive::{Archive, Result};
use super::types::{
    WadCoord, WadLinedef, WadNode, WadSector, WadSeg, WadSidedef, WadSubsector, WadThing, WadVertex,
};
use nightshade::prelude::Vec2;

const THINGS_OFFSET: usize = 1;
const LINEDEFS_OFFSET: usize = 2;
const SIDEDEFS_OFFSET: usize = 3;
const VERTICES_OFFSET: usize = 4;
const SEGS_OFFSET: usize = 5;
const SSECTORS_OFFSET: usize = 6;
const NODES_OFFSET: usize = 7;
const SECTORS_OFFSET: usize = 8;

pub struct Level {
    pub things: Vec<WadThing>,
    pub linedefs: Vec<WadLinedef>,
    pub sidedefs: Vec<WadSidedef>,
    pub vertices: Vec<WadVertex>,
    pub segs: Vec<WadSeg>,
    pub subsectors: Vec<WadSubsector>,
    pub nodes: Vec<WadNode>,
    pub sectors: Vec<WadSector>,
}

impl Level {
    pub fn from_archive(wad: &Archive, index: usize) -> Result<Level> {
        let lump = wad.level_lump(index)?;
        let start_index = lump.index();

        let things = wad
            .lump_by_index(start_index + THINGS_OFFSET)?
            .decode_vec()?;
        let linedefs = wad
            .lump_by_index(start_index + LINEDEFS_OFFSET)?
            .decode_vec()?;
        let vertices = wad
            .lump_by_index(start_index + VERTICES_OFFSET)?
            .decode_vec()?;
        let segs = wad.lump_by_index(start_index + SEGS_OFFSET)?.decode_vec()?;
        let subsectors = wad
            .lump_by_index(start_index + SSECTORS_OFFSET)?
            .decode_vec()?;
        let nodes = wad
            .lump_by_index(start_index + NODES_OFFSET)?
            .decode_vec()?;
        let sidedefs = wad
            .lump_by_index(start_index + SIDEDEFS_OFFSET)?
            .decode_vec()?;
        let sectors = wad
            .lump_by_index(start_index + SECTORS_OFFSET)?
            .decode_vec()?;

        Ok(Level {
            things,
            linedefs,
            sidedefs,
            vertices,
            segs,
            subsectors,
            nodes,
            sectors,
        })
    }

    pub fn vertex(&self, id: u16) -> Option<Vec2> {
        self.vertices
            .get(id as usize)
            .map(|v| from_wad_coords(v.x, v.y))
    }

    pub fn seg_linedef(&self, seg: &WadSeg) -> Option<&WadLinedef> {
        self.linedefs.get(seg.linedef as usize)
    }

    pub fn seg_vertices(&self, seg: &WadSeg) -> Option<(Vec2, Vec2)> {
        if let (Some(v1), Some(v2)) = (self.vertex(seg.start_vertex), self.vertex(seg.end_vertex)) {
            Some((v1, v2))
        } else {
            None
        }
    }

    pub fn seg_sidedef(&self, seg: &WadSeg) -> Option<&WadSidedef> {
        self.seg_linedef(seg).and_then(|line| {
            if seg.direction == 0 {
                self.right_sidedef(line)
            } else {
                self.left_sidedef(line)
            }
        })
    }

    pub fn seg_back_sidedef(&self, seg: &WadSeg) -> Option<&WadSidedef> {
        self.seg_linedef(seg).and_then(|line| {
            if seg.direction == 1 {
                self.right_sidedef(line)
            } else {
                self.left_sidedef(line)
            }
        })
    }

    pub fn seg_sector(&self, seg: &WadSeg) -> Option<&WadSector> {
        self.seg_sidedef(seg)
            .and_then(|side| self.sidedef_sector(side))
    }

    pub fn seg_back_sector(&self, seg: &WadSeg) -> Option<&WadSector> {
        self.seg_back_sidedef(seg)
            .and_then(|side| self.sidedef_sector(side))
    }

    pub fn left_sidedef(&self, linedef: &WadLinedef) -> Option<&WadSidedef> {
        match linedef.left_side {
            -1 => None,
            index => self.sidedefs.get(index as usize),
        }
    }

    pub fn right_sidedef(&self, linedef: &WadLinedef) -> Option<&WadSidedef> {
        match linedef.right_side {
            -1 => None,
            index => self.sidedefs.get(index as usize),
        }
    }

    pub fn sidedef_sector(&self, sidedef: &WadSidedef) -> Option<&WadSector> {
        self.sectors.get(sidedef.sector as usize)
    }

    pub fn ssector(&self, index: usize) -> Option<WadSubsector> {
        self.subsectors.get(index).copied()
    }

    pub fn ssector_segs(&self, ssector: WadSubsector) -> Option<&[WadSeg]> {
        let start = ssector.first_seg as usize;
        let end = start + ssector.num_segs as usize;
        if end <= self.segs.len() {
            Some(&self.segs[start..end])
        } else {
            None
        }
    }
}

pub fn from_wad_height(x: WadCoord) -> f32 {
    f32::from(x) / 100.0
}

pub fn from_wad_coords(x: WadCoord, y: WadCoord) -> Vec2 {
    Vec2::new(-from_wad_height(y), -from_wad_height(x))
}

pub fn parse_child_id(id: u16) -> (usize, bool) {
    ((id & 0x7fff) as usize, id & 0x8000 != 0)
}

pub fn is_untextured(name: super::name::WadName) -> bool {
    name[0] == b'-' && name[1] == b'\0'
}

pub fn is_sky_flat(name: super::name::WadName) -> bool {
    &name == b"F_SKY1\0\0"
}
