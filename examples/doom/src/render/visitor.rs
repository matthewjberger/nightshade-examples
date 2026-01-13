use crate::wad::level::{
    Level, from_wad_coords, from_wad_height, is_sky_flat, is_untextured, parse_child_id,
};
use crate::wad::name::WadName;
use crate::wad::texture::TextureDirectory;
use crate::wad::types::{WadCoord, WadNode, WadSector, WadSeg, WadThing};
use nightshade::prelude::Vec2;
use std::cmp;
use std::cmp::Ordering;

const BSP_TOLERANCE: f32 = 1e-3;
const SEG_TOLERANCE: f32 = 0.1;
const POLY_BIAS: f32 = 0.64 * 3e-4;

const LIGHT_CONTRAST: f32 = 2.0 / 31.0;

pub struct StaticQuad {
    pub vertices: (Vec2, Vec2),
    pub tex_start: (f32, f32),
    pub tex_end: (f32, f32),
    pub height_range: (f32, f32),
    pub light_level: f32,
    pub tex_name: Option<WadName>,
    pub scroll_rate: f32,
}

pub struct StaticPoly {
    pub vertices: Vec<Vec2>,
    pub height: f32,
    pub light_level: f32,
    pub tex_name: WadName,
}

pub struct SkyQuad {
    pub vertices: (Vec2, Vec2),
    pub height_range: (f32, f32),
}

pub struct SkyPoly {
    pub vertices: Vec<Vec2>,
    pub height: f32,
}

#[derive(Clone, Copy)]
pub struct PlayerStart {
    pub position: nightshade::prelude::Vec3,
    pub _angle: f32,
}

pub struct Decor {
    pub position: nightshade::prelude::Vec3,
    pub half_width: f32,
    pub height: f32,
    pub light_level: f32,
    pub sprite_prefix: [u8; 4],
    pub sequence: &'static [u8],
    pub _hanging: bool,
}

struct WallQuadParams<'a> {
    seg: &'a WadSeg,
    vertices: (Vec2, Vec2),
    height_range: (WadCoord, WadCoord),
    texture_name: WadName,
    peg_top: bool,
    light_level: f32,
    scroll_rate: f32,
}

pub trait LevelVisitor {
    fn visit_wall_quad(&mut self, _quad: &StaticQuad) {}
    fn visit_floor_poly(&mut self, _poly: &StaticPoly) {}
    fn visit_ceil_poly(&mut self, _poly: &StaticPoly) {}
    fn visit_floor_sky_poly(&mut self, _poly: &SkyPoly) {}
    fn visit_ceil_sky_poly(&mut self, _poly: &SkyPoly) {}
    fn visit_sky_quad(&mut self, _quad: &SkyQuad) {}
    fn visit_player_start(&mut self, _start: PlayerStart) {}
    fn visit_decor(&mut self, _decor: &Decor) {}
}

#[derive(Clone, Copy)]
struct Line2 {
    origin: Vec2,
    normal: Vec2,
}

impl Line2 {
    fn from_two_points(p1: Vec2, p2: Vec2) -> Self {
        let dir = p2 - p1;
        let normal = Vec2::new(dir.y, -dir.x).normalize();
        Self { origin: p1, normal }
    }

    fn signed_distance(&self, point: Vec2) -> f32 {
        (point - self.origin).dot(&self.normal)
    }

    fn inverted_halfspaces(&self) -> Self {
        Self {
            origin: self.origin,
            normal: -self.normal,
        }
    }

    fn intersect_point(&self, other: &Line2) -> Option<Vec2> {
        let denom = self.normal.x * other.normal.y - self.normal.y * other.normal.x;
        if denom.abs() < 1e-10 {
            return None;
        }
        let d1 = self.origin.dot(&self.normal);
        let d2 = other.origin.dot(&other.normal);
        let x = (d1 * other.normal.y - d2 * self.normal.y) / denom;
        let y = (self.normal.x * d2 - other.normal.x * d1) / denom;
        Some(Vec2::new(x, y))
    }
}

pub struct LevelWalker<'a, V: LevelVisitor> {
    level: &'a Level,
    tex: &'a TextureDirectory,
    visitor: &'a mut V,
    height_range: (WadCoord, WadCoord),
    bsp_lines: Vec<Line2>,
    subsector_points: Vec<Vec2>,
    subsector_seg_lines: Vec<Line2>,
}

impl<'a, V: LevelVisitor> LevelWalker<'a, V> {
    pub fn new(level: &'a Level, tex: &'a TextureDirectory, visitor: &'a mut V) -> Self {
        Self {
            level,
            tex,
            visitor,
            height_range: min_max_height(level),
            bsp_lines: Vec::with_capacity(32),
            subsector_points: Vec::with_capacity(32),
            subsector_seg_lines: Vec::with_capacity(32),
        }
    }

    pub fn walk(&mut self) {
        let root = match self.level.nodes.last() {
            Some(node) => node,
            None => return,
        };
        let partition = partition_line(root);
        self.children(root, partition);
        self.things();
    }

    fn node(&mut self, id: u16, _branch: bool) {
        let (id, is_leaf) = parse_child_id(id);
        if is_leaf {
            self.subsector(id);
            return;
        }

        let node = match self.level.nodes.get(id) {
            Some(node) => node,
            None => return,
        };
        let partition = partition_line(node);
        self.children(node, partition);
    }

    fn children(&mut self, node: &WadNode, partition: Line2) {
        self.bsp_lines.push(partition);
        self.node(node.left, true);
        self.bsp_lines.pop();

        self.bsp_lines.push(partition.inverted_halfspaces());
        self.node(node.right, false);
        self.bsp_lines.pop();
    }

    fn subsector(&mut self, id: usize) {
        let subsector = match self.level.ssector(id) {
            Some(s) => s,
            None => return,
        };
        let segs = match self.level.ssector_segs(subsector) {
            Some(s) if !s.is_empty() => s,
            _ => return,
        };
        let sector = match self.level.seg_sector(&segs[0]) {
            Some(s) => s,
            None => return,
        };

        self.subsector_seg_lines.clear();
        self.subsector_points.clear();

        for seg in segs {
            let (v1, v2) = match self.level.seg_vertices(seg) {
                Some(v) => v,
                None => return,
            };
            self.subsector_points.push(v1);
            self.subsector_points.push(v2);
            self.subsector_seg_lines
                .push(Line2::from_two_points(v1, v2));
            self.seg(sector, seg, (v1, v2));
        }

        for index in 0..(self.bsp_lines.len().saturating_sub(1)) {
            for jndex in (index + 1)..self.bsp_lines.len() {
                let (line1, line2) = (&self.bsp_lines[index], &self.bsp_lines[jndex]);
                let point = match line1.intersect_point(line2) {
                    Some(p) => p,
                    None => continue,
                };

                let within_bsp = self
                    .bsp_lines
                    .iter()
                    .all(|l| l.signed_distance(point) >= -BSP_TOLERANCE);
                let within_seg = self
                    .subsector_seg_lines
                    .iter()
                    .all(|l| l.signed_distance(point) <= SEG_TOLERANCE);
                if within_bsp && within_seg {
                    self.subsector_points.push(point);
                }
            }
        }

        if self.subsector_points.len() >= 3 {
            points_to_polygon(&mut self.subsector_points);
            if self.subsector_points.len() >= 3 {
                self.flat_poly(sector);
            }
        }
    }

    fn seg(&mut self, sector: &WadSector, seg: &WadSeg, vertices: (Vec2, Vec2)) {
        let line = match self.level.seg_linedef(seg) {
            Some(l) => l,
            None => return,
        };
        let sidedef = match self.level.seg_sidedef(seg) {
            Some(s) => s,
            None => return,
        };

        let (min, max) = (self.height_range.0, self.height_range.1);
        let (floor, ceiling) = (sector.floor_height, sector.ceiling_height);
        let unpeg_lower = line.lower_unpegged();
        let base_light = sector.light as f32 / 255.0;

        let (v1, v2) = vertices;
        let light_level = apply_wall_contrast(v1, v2, base_light);

        let scroll_rate = if line.special_type == 0x30 { 35.0 } else { 0.0 };

        let back_sector = match self.level.seg_back_sector(seg) {
            None => {
                self.wall_quad(WallQuadParams {
                    seg,
                    vertices,
                    height_range: (floor, ceiling),
                    texture_name: sidedef.middle_texture,
                    peg_top: !unpeg_lower,
                    light_level,
                    scroll_rate,
                });
                if is_sky_flat(sector.ceiling_texture) {
                    self.sky_quad(vertices, (ceiling, max));
                }
                if is_sky_flat(sector.floor_texture) {
                    self.sky_quad(vertices, (min, floor));
                }
                return;
            }
            Some(s) => s,
        };

        let (back_floor, back_ceiling) = (back_sector.floor_height, back_sector.ceiling_height);

        if is_sky_flat(sector.ceiling_texture) && !is_sky_flat(back_sector.ceiling_texture) {
            self.sky_quad(vertices, (ceiling, max));
        }
        if is_sky_flat(sector.floor_texture) && !is_sky_flat(back_sector.floor_texture) {
            self.sky_quad(vertices, (min, floor));
        }

        let unpeg_upper = line.upper_unpegged();

        if back_floor > floor {
            self.wall_quad(WallQuadParams {
                seg,
                vertices,
                height_range: (floor, back_floor),
                texture_name: sidedef.lower_texture,
                peg_top: unpeg_lower,
                light_level,
                scroll_rate,
            });
        }

        if back_ceiling < ceiling && !is_sky_flat(back_sector.ceiling_texture) {
            self.wall_quad(WallQuadParams {
                seg,
                vertices,
                height_range: (back_ceiling, ceiling),
                texture_name: sidedef.upper_texture,
                peg_top: unpeg_upper,
                light_level,
                scroll_rate,
            });
        }

        if !is_untextured(sidedef.middle_texture) {
            let mid_floor = floor.max(back_floor);
            let mid_ceiling = ceiling.min(back_ceiling);
            if mid_ceiling > mid_floor {
                self.wall_quad(WallQuadParams {
                    seg,
                    vertices,
                    height_range: (mid_floor, mid_ceiling),
                    texture_name: sidedef.middle_texture,
                    peg_top: !unpeg_lower,
                    light_level,
                    scroll_rate,
                });
            }
        }
    }

    fn wall_quad(&mut self, params: WallQuadParams<'_>) {
        let (low, high) = params.height_range;
        if low >= high {
            return;
        }

        let tex_size = if is_untextured(params.texture_name) {
            None
        } else if let Some(image) = self.tex.texture(params.texture_name) {
            Some((image.width() as f32, image.height() as f32))
        } else {
            return;
        };

        let sidedef = match self.level.seg_sidedef(params.seg) {
            Some(s) => s,
            None => return,
        };

        let (v1, v2) = params.vertices;
        let (low, high) = (from_wad_height(low), from_wad_height(high));
        let height = (high - low) * 100.0;

        let s1 = f32::from(params.seg.offset) + f32::from(sidedef.x_offset);
        let s2 = s1 + (v2 - v1).magnitude() * 100.0;

        let (t1, t2) = match (tex_size, params.peg_top) {
            (Some(_), true) => (height, 0.0),
            (Some((_, tex_height)), false) => (tex_height, tex_height - height),
            (None, _) => (height, 0.0),
        };

        let (t1, t2) = (
            t1 + f32::from(sidedef.y_offset),
            t2 + f32::from(sidedef.y_offset),
        );

        self.visitor.visit_wall_quad(&StaticQuad {
            vertices: (v1, v2),
            tex_start: (s1, t1),
            tex_end: (s2, t2),
            height_range: (low - POLY_BIAS, high + POLY_BIAS),
            light_level: params.light_level,
            tex_name: tex_size.map(|_| params.texture_name),
            scroll_rate: params.scroll_rate,
        });
    }

    fn flat_poly(&mut self, sector: &WadSector) {
        let light_level = sector.light as f32 / 255.0;
        let (floor_tex, ceil_tex) = (sector.floor_texture, sector.ceiling_texture);
        let (floor_sky, ceil_sky) = (is_sky_flat(floor_tex), is_sky_flat(ceil_tex));

        let floor_y = from_wad_height(if floor_sky {
            self.height_range.0
        } else {
            sector.floor_height
        });
        let ceil_y = from_wad_height(if ceil_sky {
            self.height_range.1
        } else {
            sector.ceiling_height
        });

        if floor_sky {
            self.visitor.visit_floor_sky_poly(&SkyPoly {
                vertices: self.subsector_points.clone(),
                height: floor_y,
            });
        } else {
            self.visitor.visit_floor_poly(&StaticPoly {
                vertices: self.subsector_points.clone(),
                height: floor_y,
                light_level,
                tex_name: floor_tex,
            });
        }

        if ceil_sky {
            self.visitor.visit_ceil_sky_poly(&SkyPoly {
                vertices: self.subsector_points.clone(),
                height: ceil_y,
            });
        } else {
            self.visitor.visit_ceil_poly(&StaticPoly {
                vertices: self.subsector_points.clone(),
                height: ceil_y,
                light_level,
                tex_name: ceil_tex,
            });
        }
    }

    fn sky_quad(&mut self, (v1, v2): (Vec2, Vec2), (low, high): (WadCoord, WadCoord)) {
        if low >= high {
            return;
        }
        self.visitor.visit_sky_quad(&SkyQuad {
            vertices: (v1, v2),
            height_range: (from_wad_height(low), from_wad_height(high)),
        });
    }

    fn things(&mut self) {
        for thing in &self.level.things {
            let pos = from_wad_coords(thing.x, thing.y);
            let sector = self.sector_at(pos);

            if thing.thing_type == 1 {
                let floor_height = sector
                    .map(|s| from_wad_height(s.floor_height))
                    .unwrap_or(0.0);
                self.visitor.visit_player_start(PlayerStart {
                    position: nightshade::prelude::Vec3::new(pos.x, floor_height + 0.56, pos.y),
                    _angle: (thing.angle as f32).to_radians(),
                });
            } else if let Some(sector) = sector {
                self.decor(thing, pos, sector);
            }
        }
    }

    fn decor(&mut self, thing: &WadThing, pos: Vec2, sector: &WadSector) {
        let (sprite_prefix, sequence, hanging) = match thing_to_sprite(thing.thing_type) {
            Some(info) => info,
            None => return,
        };

        let first_frame = sequence[0];
        let mut sprite_name_bytes = [0u8; 8];
        sprite_name_bytes[0..4].copy_from_slice(&sprite_prefix);
        sprite_name_bytes[4] = first_frame;
        sprite_name_bytes[5] = b'0';

        let mut sprite_name = match WadName::from_bytes(&sprite_name_bytes) {
            Ok(name) => name,
            Err(_) => return,
        };

        let mut image = self.tex.texture(sprite_name);
        if image.is_none() {
            sprite_name_bytes[5] = b'1';
            sprite_name = match WadName::from_bytes(&sprite_name_bytes) {
                Ok(name) => name,
                Err(_) => return,
            };
            image = self.tex.texture(sprite_name);
        }

        let image = match image {
            Some(img) => img,
            None => return,
        };

        let width = from_wad_height(image.width() as i16);
        let height = from_wad_height(image.height() as i16);
        let half_width = width * 0.5;
        let light_level = sector.light as f32 / 255.0;

        let base_y = if hanging {
            from_wad_height(sector.ceiling_height) - height
        } else {
            from_wad_height(sector.floor_height)
        };

        self.visitor.visit_decor(&Decor {
            position: nightshade::prelude::Vec3::new(pos.x, base_y, pos.y),
            half_width,
            height,
            light_level,
            sprite_prefix,
            sequence,
            _hanging: hanging,
        });
    }

    fn sector_at(&self, pos: Vec2) -> Option<&'a WadSector> {
        let mut child_id = (self.level.nodes.len() - 1) as u16;
        loop {
            let (id, is_leaf) = parse_child_id(child_id);
            if is_leaf {
                let segs = self
                    .level
                    .ssector(id)
                    .and_then(|s| self.level.ssector_segs(s))?;
                if segs.is_empty() {
                    return None;
                }
                return self.level.seg_sector(&segs[0]);
            } else {
                let node = self.level.nodes.get(id)?;
                let partition = partition_line(node);
                if partition.signed_distance(pos) > 0.0 {
                    child_id = node.left;
                } else {
                    child_id = node.right;
                }
            }
        }
    }
}

fn apply_wall_contrast(v1: Vec2, v2: Vec2, base_light: f32) -> f32 {
    let dx = (v1.x - v2.x).abs();
    let dy = (v1.y - v2.y).abs();

    if dx < 0.001 {
        (base_light + LIGHT_CONTRAST).min(1.0)
    } else if dy < 0.001 {
        (base_light - LIGHT_CONTRAST).max(0.0)
    } else {
        base_light
    }
}

fn thing_to_sprite(thing_type: u16) -> Option<([u8; 4], &'static [u8], bool)> {
    let (prefix, sequence, hanging): ([u8; 4], &'static [u8], bool) = match thing_type {
        7 => (*b"SPID", b"AB".as_slice(), false),
        9 => (*b"SPOS", b"AB".as_slice(), false),
        16 => (*b"CYBR", b"AB".as_slice(), false),
        58 => (*b"SARG", b"AB".as_slice(), false),
        64 => (*b"VILE", b"AB".as_slice(), false),
        65 => (*b"CPOS", b"AB".as_slice(), false),
        66 => (*b"SKEL", b"AB".as_slice(), false),
        67 => (*b"FATT", b"AB".as_slice(), false),
        68 => (*b"BSPI", b"AB".as_slice(), false),
        69 => (*b"BOS2", b"AB".as_slice(), false),
        71 => (*b"PAIN", b"AB".as_slice(), false),
        72 => (*b"KEEN", b"AB".as_slice(), false),
        84 => (*b"SSWV", b"AB".as_slice(), false),
        3001 => (*b"TROO", b"AB".as_slice(), false),
        3002 => (*b"SARG", b"AB".as_slice(), false),
        3003 => (*b"BOSS", b"AB".as_slice(), false),
        3004 => (*b"POSS", b"AB".as_slice(), false),
        3005 => (*b"HEAD", b"AB".as_slice(), false),
        3006 => (*b"SKUL", b"AB".as_slice(), false),

        2001 => (*b"SHOT", b"A".as_slice(), false),
        2002 => (*b"MGUN", b"A".as_slice(), false),
        2003 => (*b"LAUN", b"A".as_slice(), false),
        2004 => (*b"PLAS", b"A".as_slice(), false),
        2005 => (*b"CSAW", b"A".as_slice(), false),
        2006 => (*b"BFUG", b"A".as_slice(), false),
        82 => (*b"SGN2", b"A".as_slice(), false),

        2007 => (*b"CLIP", b"A".as_slice(), false),
        2008 => (*b"SHEL", b"A".as_slice(), false),
        2010 => (*b"ROCK", b"A".as_slice(), false),
        2046 => (*b"BROK", b"A".as_slice(), false),
        2047 => (*b"CELL", b"A".as_slice(), false),
        2048 => (*b"AMMO", b"A".as_slice(), false),
        2049 => (*b"SBOX", b"A".as_slice(), false),
        17 => (*b"CELP", b"A".as_slice(), false),

        2011 => (*b"STIM", b"A".as_slice(), false),
        2012 => (*b"MEDI", b"A".as_slice(), false),
        2013 => (*b"SOUL", b"ABCDCB".as_slice(), false),
        2014 => (*b"BON1", b"ABCDCB".as_slice(), false),
        2015 => (*b"BON2", b"ABCDCB".as_slice(), false),
        2018 => (*b"ARM1", b"AB".as_slice(), false),
        2019 => (*b"ARM2", b"AB".as_slice(), false),
        2022 => (*b"PINV", b"ABCD".as_slice(), false),
        2023 => (*b"PSTR", b"A".as_slice(), false),
        2024 => (*b"PINS", b"ABCD".as_slice(), false),
        2025 => (*b"SUIT", b"A".as_slice(), false),
        2026 => (*b"PMAP", b"ABCDCB".as_slice(), false),
        2045 => (*b"PVIS", b"AB".as_slice(), false),
        8 => (*b"BPAK", b"A".as_slice(), false),
        83 => (*b"MEGA", b"ABCD".as_slice(), false),

        5 => (*b"BKEY", b"AB".as_slice(), false),
        40 => (*b"BSKU", b"AB".as_slice(), false),
        13 => (*b"RKEY", b"AB".as_slice(), false),
        38 => (*b"RSKU", b"AB".as_slice(), false),
        6 => (*b"YKEY", b"AB".as_slice(), false),
        39 => (*b"YSKU", b"AB".as_slice(), false),

        10 => (*b"PLAY", b"W".as_slice(), false),
        12 => (*b"PLAY", b"W".as_slice(), false),
        15 => (*b"PLAY", b"N".as_slice(), false),
        18 => (*b"POSS", b"L".as_slice(), false),
        19 => (*b"SPOS", b"L".as_slice(), false),
        20 => (*b"TROO", b"M".as_slice(), false),
        21 => (*b"SARG", b"N".as_slice(), false),
        22 => (*b"HEAD", b"L".as_slice(), false),
        23 => (*b"SKUL", b"K".as_slice(), false),
        24 => (*b"POL5", b"A".as_slice(), false),
        79 => (*b"POB1", b"A".as_slice(), false),
        80 => (*b"POB2", b"A".as_slice(), false),
        81 => (*b"BRS1", b"A".as_slice(), false),

        2035 => (*b"BAR1", b"AB".as_slice(), false),
        70 => (*b"FCAN", b"ABC".as_slice(), false),

        25 => (*b"POL1", b"A".as_slice(), false),
        26 => (*b"POL6", b"AB".as_slice(), false),
        27 => (*b"POL4", b"A".as_slice(), false),
        28 => (*b"POL2", b"A".as_slice(), false),
        29 => (*b"POL3", b"AB".as_slice(), false),

        30 => (*b"COL1", b"A".as_slice(), false),
        31 => (*b"COL2", b"A".as_slice(), false),
        32 => (*b"COL3", b"A".as_slice(), false),
        33 => (*b"COL4", b"A".as_slice(), false),
        36 => (*b"COL5", b"AB".as_slice(), false),
        37 => (*b"COL6", b"A".as_slice(), false),

        34 => (*b"CAND", b"A".as_slice(), false),
        35 => (*b"CBRA", b"A".as_slice(), false),

        41 => (*b"CEYE", b"ABCB".as_slice(), false),
        42 => (*b"FSKU", b"ABC".as_slice(), false),

        43 => (*b"TRE1", b"A".as_slice(), false),
        54 => (*b"TRE2", b"A".as_slice(), false),

        44 => (*b"TBLU", b"ABCD".as_slice(), false),
        45 => (*b"TGRN", b"ABCD".as_slice(), false),
        46 => (*b"TRED", b"ABCD".as_slice(), false),
        55 => (*b"SMBT", b"ABCD".as_slice(), false),
        56 => (*b"SMGT", b"ABCD".as_slice(), false),
        57 => (*b"SMRT", b"ABCD".as_slice(), false),

        47 => (*b"SMIT", b"A".as_slice(), false),
        48 => (*b"ELEC", b"A".as_slice(), false),
        2028 => (*b"COLU", b"A".as_slice(), false),
        85 => (*b"TLMP", b"ABCD".as_slice(), false),
        86 => (*b"TLP2", b"ABCD".as_slice(), false),

        49 => (*b"GOR1", b"ABCB".as_slice(), true),
        50 => (*b"GOR2", b"A".as_slice(), true),
        51 => (*b"GOR3", b"A".as_slice(), true),
        52 => (*b"GOR4", b"A".as_slice(), true),
        53 => (*b"GOR5", b"A".as_slice(), true),

        59 => (*b"GOR2", b"A".as_slice(), false),
        60 => (*b"GOR4", b"A".as_slice(), false),
        61 => (*b"GOR3", b"A".as_slice(), false),
        62 => (*b"GOR5", b"A".as_slice(), false),
        63 => (*b"GOR1", b"ABCB".as_slice(), false),

        73 => (*b"HDB1", b"A".as_slice(), true),
        74 => (*b"HDB2", b"A".as_slice(), true),
        75 => (*b"HDB3", b"A".as_slice(), true),
        76 => (*b"HDB4", b"A".as_slice(), true),
        77 => (*b"HDB5", b"A".as_slice(), true),
        78 => (*b"HDB6", b"A".as_slice(), true),

        88 => (*b"BBRN", b"A".as_slice(), false),

        _ => return None,
    };
    Some((prefix, sequence, hanging))
}

fn partition_line(node: &WadNode) -> Line2 {
    Line2::from_two_points(
        from_wad_coords(node.line_x, node.line_y),
        from_wad_coords(node.line_x + node.step_x, node.line_y + node.step_y),
    )
}

fn min_max_height(level: &Level) -> (WadCoord, WadCoord) {
    let (min, max) = level
        .sectors
        .iter()
        .map(|s| (s.floor_height, s.ceiling_height))
        .fold((32_767, -32_768), |(min, max), (f, c)| {
            (cmp::min(min, f), cmp::max(max, c))
        });
    (min - 512, max + 512)
}

fn polygon_center(points: &[Vec2]) -> Vec2 {
    let mut center = Vec2::zeros();
    for p in points {
        center += p;
    }
    center / (points.len() as f32)
}

fn points_to_polygon(points: &mut Vec<Vec2>) {
    if points.len() < 3 {
        return;
    }

    let center = polygon_center(points);
    points.sort_by(|a, b| {
        let ac = *a - center;
        let bc = *b - center;
        if ac.x >= 0.0 && bc.x < 0.0 {
            return Ordering::Less;
        }
        if ac.x < 0.0 && bc.x >= 0.0 {
            return Ordering::Greater;
        }
        if ac.x == 0.0 && bc.x == 0.0 {
            if ac.y >= 0.0 || bc.y >= 0.0 {
                return if a.y > b.y {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
            return if b.y > a.y {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        let cross = ac.x * bc.y - ac.y * bc.x;
        if cross < 0.0 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    let mut simplified = Vec::with_capacity(points.len());
    simplified.push(points[0]);

    for point in points.iter().skip(1) {
        if let Some(last) = simplified.last()
            && (*point - *last).magnitude() > 0.0001
        {
            simplified.push(*point);
        }
    }

    if simplified.len() >= 2
        && let (Some(first), Some(last)) = (simplified.first(), simplified.last())
        && (*first - *last).magnitude() < 0.0001
    {
        simplified.pop();
    }

    if simplified.len() < 3 {
        points.clear();
        return;
    }

    let center = polygon_center(&simplified);
    for point in &mut simplified {
        let dir = *point - center;
        let len = dir.magnitude();
        if len > 1e-6 {
            *point += dir / len * POLY_BIAS;
        }
    }

    *points = simplified;
}
