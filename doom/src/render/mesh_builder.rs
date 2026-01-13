use super::vertex::{DoomVertex, DoomVertexParams, SkyVertex, SpriteVertex};
use super::visitor::{Decor, LevelVisitor, PlayerStart, SkyPoly, SkyQuad, StaticPoly, StaticQuad};
use crate::wad::name::WadName;
use crate::wad::texture::BoundsLookup;
use indexmap::{IndexMap, IndexSet};
use nightshade::prelude::Vec2;

pub struct SpriteInfo {
    pub prefix: [u8; 4],
    pub sequence: &'static [u8],
}

pub struct MeshBuilder {
    pub wall_vertices: Vec<DoomVertex>,
    pub wall_indices: Vec<u32>,
    pub flat_vertices: Vec<DoomVertex>,
    pub flat_indices: Vec<u32>,
    pub sky_vertices: Vec<SkyVertex>,
    pub sky_indices: Vec<u32>,
    pub sprite_vertices: Vec<SpriteVertex>,
    pub sprite_indices: Vec<u32>,

    pub wall_texture_names: IndexSet<WadName>,
    pub flat_texture_names: IndexSet<WadName>,
    pub sprite_texture_names: IndexSet<WadName>,
    pub sprite_info: IndexMap<WadName, SpriteInfo>,

    pub player_start: Option<PlayerStart>,

    wall_bounds: Option<BoundsLookup>,
    flat_bounds: Option<BoundsLookup>,
    sprite_bounds: Option<BoundsLookup>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            wall_vertices: Vec::new(),
            wall_indices: Vec::new(),
            flat_vertices: Vec::new(),
            flat_indices: Vec::new(),
            sky_vertices: Vec::new(),
            sky_indices: Vec::new(),
            sprite_vertices: Vec::new(),
            sprite_indices: Vec::new(),
            wall_texture_names: IndexSet::new(),
            flat_texture_names: IndexSet::new(),
            sprite_texture_names: IndexSet::new(),
            sprite_info: IndexMap::new(),
            player_start: None,
            wall_bounds: None,
            flat_bounds: None,
            sprite_bounds: None,
        }
    }

    pub fn set_wall_bounds(&mut self, bounds: BoundsLookup) {
        self.wall_bounds = Some(bounds);
    }

    pub fn set_flat_bounds(&mut self, bounds: BoundsLookup) {
        self.flat_bounds = Some(bounds);
    }

    pub fn set_sprite_bounds(&mut self, bounds: BoundsLookup) {
        self.sprite_bounds = Some(bounds);
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelVisitor for MeshBuilder {
    fn visit_wall_quad(&mut self, quad: &StaticQuad) {
        let tex_name = match quad.tex_name {
            Some(name) => name,
            None => return,
        };

        self.wall_texture_names.insert(tex_name);

        let bounds = self.wall_bounds.as_ref().and_then(|b| b.get(&tex_name));
        let (atlas_u, atlas_v, tex_w, tex_h, num_frames, row_height) = match bounds {
            Some(b) => (b.x, b.y, b.width, b.height, b.num_frames, b.row_height),
            None => return,
        };

        let (v1, v2) = quad.vertices;
        let (low, high) = quad.height_range;
        let (s1, t1) = quad.tex_start;
        let (s2, t2) = quad.tex_end;
        let light = quad.light_level;
        let scroll_rate = quad.scroll_rate;

        let base_index = self.wall_vertices.len() as u32;

        self.wall_vertices.push(DoomVertex::new(DoomVertexParams {
            position: [v1.x, low, v1.y],
            atlas_uv: [atlas_u, atlas_v],
            tile_uv: [s1, t1],
            tile_size: [tex_w, tex_h],
            light,
            num_frames: num_frames as u32,
            scroll_rate,
            row_height: row_height as f32,
        }));
        self.wall_vertices.push(DoomVertex::new(DoomVertexParams {
            position: [v2.x, low, v2.y],
            atlas_uv: [atlas_u, atlas_v],
            tile_uv: [s2, t1],
            tile_size: [tex_w, tex_h],
            light,
            num_frames: num_frames as u32,
            scroll_rate,
            row_height: row_height as f32,
        }));
        self.wall_vertices.push(DoomVertex::new(DoomVertexParams {
            position: [v2.x, high, v2.y],
            atlas_uv: [atlas_u, atlas_v],
            tile_uv: [s2, t2],
            tile_size: [tex_w, tex_h],
            light,
            num_frames: num_frames as u32,
            scroll_rate,
            row_height: row_height as f32,
        }));
        self.wall_vertices.push(DoomVertex::new(DoomVertexParams {
            position: [v1.x, high, v1.y],
            atlas_uv: [atlas_u, atlas_v],
            tile_uv: [s1, t2],
            tile_size: [tex_w, tex_h],
            light,
            num_frames: num_frames as u32,
            scroll_rate,
            row_height: row_height as f32,
        }));

        self.wall_indices.push(base_index);
        self.wall_indices.push(base_index + 1);
        self.wall_indices.push(base_index + 2);
        self.wall_indices.push(base_index);
        self.wall_indices.push(base_index + 2);
        self.wall_indices.push(base_index + 3);
    }

    fn visit_floor_poly(&mut self, poly: &StaticPoly) {
        self.flat_texture_names.insert(poly.tex_name);
        self.add_flat_poly(
            &poly.vertices,
            poly.height,
            poly.light_level,
            poly.tex_name,
            false,
        );
    }

    fn visit_ceil_poly(&mut self, poly: &StaticPoly) {
        self.flat_texture_names.insert(poly.tex_name);
        self.add_flat_poly(
            &poly.vertices,
            poly.height,
            poly.light_level,
            poly.tex_name,
            true,
        );
    }

    fn visit_floor_sky_poly(&mut self, poly: &SkyPoly) {
        self.add_sky_poly(&poly.vertices, poly.height, false);
    }

    fn visit_ceil_sky_poly(&mut self, poly: &SkyPoly) {
        self.add_sky_poly(&poly.vertices, poly.height, true);
    }

    fn visit_sky_quad(&mut self, quad: &SkyQuad) {
        let (v1, v2) = quad.vertices;
        let (low, high) = quad.height_range;

        let base_index = self.sky_vertices.len() as u32;

        self.sky_vertices.push(SkyVertex::new([v1.x, low, v1.y]));
        self.sky_vertices.push(SkyVertex::new([v2.x, low, v2.y]));
        self.sky_vertices.push(SkyVertex::new([v2.x, high, v2.y]));
        self.sky_vertices.push(SkyVertex::new([v1.x, high, v1.y]));

        self.sky_indices.push(base_index);
        self.sky_indices.push(base_index + 1);
        self.sky_indices.push(base_index + 2);
        self.sky_indices.push(base_index);
        self.sky_indices.push(base_index + 2);
        self.sky_indices.push(base_index + 3);
    }

    fn visit_player_start(&mut self, start: PlayerStart) {
        if self.player_start.is_none() {
            self.player_start = Some(start);
        }
    }

    fn visit_decor(&mut self, decor: &Decor) {
        let mut first_frame_name_bytes = [0u8; 8];
        first_frame_name_bytes[0..4].copy_from_slice(&decor.sprite_prefix);
        first_frame_name_bytes[4] = decor.sequence[0];
        first_frame_name_bytes[5] = b'0';

        let first_frame_name = match WadName::from_bytes(&first_frame_name_bytes) {
            Ok(name) => name,
            Err(_) => return,
        };

        for &frame_char in decor.sequence {
            let mut frame_name_bytes = [0u8; 8];
            frame_name_bytes[0..4].copy_from_slice(&decor.sprite_prefix);
            frame_name_bytes[4] = frame_char;
            frame_name_bytes[5] = b'0';

            if let Ok(frame_name) = WadName::from_bytes(&frame_name_bytes) {
                self.sprite_texture_names.insert(frame_name);
            }
        }

        self.sprite_info.insert(
            first_frame_name,
            SpriteInfo {
                prefix: decor.sprite_prefix,
                sequence: decor.sequence,
            },
        );

        let bounds = self
            .sprite_bounds
            .as_ref()
            .and_then(|b| b.get(&first_frame_name));
        let (atlas_u, atlas_v, tex_w, tex_h, num_frames, x_off) = match bounds {
            Some(b) => (b.x, b.y, b.width, b.height, b.num_frames, b.x_offset),
            None => return,
        };

        let pos = decor.position;
        let half_width = decor.half_width;
        let height = decor.height;
        let light = decor.light_level;

        let x_offset_world = x_off / 100.0;
        let width_world = half_width * 2.0;
        let bottom_y = pos.y;
        let top_y = bottom_y + height;

        let left_local_x = -x_offset_world;
        let right_local_x = width_world - x_offset_world;

        let base_index = self.sprite_vertices.len() as u32;

        self.sprite_vertices.push(SpriteVertex::new(
            [pos.x, bottom_y, pos.z],
            [atlas_u, atlas_v],
            [0.0, tex_h],
            [tex_w, tex_h],
            left_local_x,
            light,
            num_frames as u32,
        ));
        self.sprite_vertices.push(SpriteVertex::new(
            [pos.x, bottom_y, pos.z],
            [atlas_u, atlas_v],
            [tex_w, tex_h],
            [tex_w, tex_h],
            right_local_x,
            light,
            num_frames as u32,
        ));
        self.sprite_vertices.push(SpriteVertex::new(
            [pos.x, top_y, pos.z],
            [atlas_u, atlas_v],
            [tex_w, 0.0],
            [tex_w, tex_h],
            right_local_x,
            light,
            num_frames as u32,
        ));
        self.sprite_vertices.push(SpriteVertex::new(
            [pos.x, top_y, pos.z],
            [atlas_u, atlas_v],
            [0.0, 0.0],
            [tex_w, tex_h],
            left_local_x,
            light,
            num_frames as u32,
        ));

        self.sprite_indices.push(base_index);
        self.sprite_indices.push(base_index + 1);
        self.sprite_indices.push(base_index + 2);
        self.sprite_indices.push(base_index);
        self.sprite_indices.push(base_index + 2);
        self.sprite_indices.push(base_index + 3);
    }
}

impl MeshBuilder {
    fn add_flat_poly(
        &mut self,
        vertices: &[Vec2],
        height: f32,
        light: f32,
        tex_name: WadName,
        is_ceiling: bool,
    ) {
        if vertices.len() < 3 {
            return;
        }

        let bounds = self.flat_bounds.as_ref().and_then(|b| b.get(&tex_name));
        let (atlas_u, atlas_v, tex_w, tex_h, num_frames, row_height) = match bounds {
            Some(b) => (b.x, b.y, b.width, b.height, b.num_frames, b.row_height),
            None => return,
        };

        let base_index = self.flat_vertices.len() as u32;

        for vertex in vertices {
            let tile_u = vertex.x * 100.0;
            let tile_v = vertex.y * 100.0;

            self.flat_vertices.push(DoomVertex::new(DoomVertexParams {
                position: [vertex.x, height, vertex.y],
                atlas_uv: [atlas_u, atlas_v],
                tile_uv: [tile_u, tile_v],
                tile_size: [tex_w, tex_h],
                light,
                num_frames: num_frames as u32,
                scroll_rate: 0.0,
                row_height: row_height as f32,
            }));
        }

        for index in 1..(vertices.len() - 1) {
            if is_ceiling {
                self.flat_indices.push(base_index);
                self.flat_indices.push(base_index + index as u32 + 1);
                self.flat_indices.push(base_index + index as u32);
            } else {
                self.flat_indices.push(base_index);
                self.flat_indices.push(base_index + index as u32);
                self.flat_indices.push(base_index + index as u32 + 1);
            }
        }
    }

    fn add_sky_poly(&mut self, vertices: &[Vec2], height: f32, is_ceiling: bool) {
        if vertices.len() < 3 {
            return;
        }

        let base_index = self.sky_vertices.len() as u32;

        for vertex in vertices {
            self.sky_vertices
                .push(SkyVertex::new([vertex.x, height, vertex.y]));
        }

        for index in 1..(vertices.len() - 1) {
            if is_ceiling {
                self.sky_indices.push(base_index);
                self.sky_indices.push(base_index + index as u32 + 1);
                self.sky_indices.push(base_index + index as u32);
            } else {
                self.sky_indices.push(base_index);
                self.sky_indices.push(base_index + index as u32);
                self.sky_indices.push(base_index + index as u32 + 1);
            }
        }
    }
}
