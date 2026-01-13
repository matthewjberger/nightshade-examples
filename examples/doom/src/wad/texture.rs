use super::archive::{Archive, ArchiveError, Result};
use super::image::Image;
use super::name::WadName;
use super::types::{Colormap, Palette};
use indexmap::IndexMap;

pub type Flat = Vec<u8>;
pub type BoundsLookup = IndexMap<WadName, Bounds>;

#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub num_frames: usize,
    pub row_height: usize,
    pub x_offset: f32,
}

pub struct TextureDirectory {
    textures: IndexMap<WadName, Image>,
    _patches: Vec<(WadName, Option<Image>)>,
    palettes: Vec<Palette>,
    colormaps: Vec<Colormap>,
    flats: IndexMap<WadName, Flat>,
}

pub struct TransparentImage {
    pub pixels: Vec<u16>,
    pub width: usize,
    pub height: usize,
}

pub struct OpaqueImage {
    pub pixels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

pub struct MappedPalette {
    pub pixels: Vec<u8>,
    pub colormaps: usize,
}

impl TextureDirectory {
    pub fn from_archive(wad: &Archive) -> Result<TextureDirectory> {
        let palettes: Vec<Palette> = wad.required_named_lump(b"PLAYPAL\0")?.read_blobs()?;
        let colormaps: Vec<Colormap> = wad.required_named_lump(b"COLORMAP")?.read_blobs()?;

        let patches = read_patches(wad)?;

        let mut textures = IndexMap::new();
        let mut textures_buffer = Vec::new();

        for &lump_name in &[b"TEXTURE1", b"TEXTURE2"] {
            let mut padded = [0u8; 8];
            padded[..lump_name.len()].copy_from_slice(lump_name);

            if let Ok(name) = WadName::from_bytes(&padded)
                && let Some(lump) = wad.named_lump(&name)?
            {
                textures_buffer.clear();
                textures_buffer = lump.read_bytes()?;
                read_textures(&textures_buffer, &patches, &mut textures)?;
            }
        }

        let flats = read_flats(wad)?;
        let _num_sprites = read_sprites(wad, &mut textures)?;

        Ok(TextureDirectory {
            _patches: patches,
            textures,
            palettes,
            colormaps,
            flats,
        })
    }

    pub fn texture(&self, name: WadName) -> Option<&Image> {
        self.textures.get(&name)
    }

    pub fn flat(&self, name: WadName) -> Option<&Flat> {
        self.flats.get(&name)
    }

    pub fn build_palette_texture(
        &self,
        palette_index: usize,
        colormap_start: usize,
        colormap_end: usize,
    ) -> MappedPalette {
        let num_colormaps = colormap_end - colormap_start;
        let mut mapped = vec![0u8; 256 * num_colormaps * 4];
        let palette = &self.palettes[palette_index];

        for (colormap_offset, colormap) in self
            .colormaps
            .iter()
            .enumerate()
            .take(colormap_end)
            .skip(colormap_start)
        {
            let row_offset = (colormap_offset - colormap_start) * 256 * 4;
            for (color_index, &remapped_color) in colormap.0.iter().enumerate() {
                let palette_offset = usize::from(remapped_color) * 3;
                let pixel_offset = row_offset + color_index * 4;
                mapped[pixel_offset] = palette.0[palette_offset];
                mapped[pixel_offset + 1] = palette.0[palette_offset + 1];
                mapped[pixel_offset + 2] = palette.0[palette_offset + 2];
                mapped[pixel_offset + 3] = 255;
            }
        }

        MappedPalette {
            pixels: mapped,
            colormaps: num_colormaps,
        }
    }

    pub fn build_texture_atlas<I>(&self, names_iter: I) -> (TransparentImage, BoundsLookup)
    where
        I: IntoIterator<Item = WadName>,
    {
        let names: Vec<WadName> = names_iter.into_iter().collect();

        let entries: Vec<_> = names
            .iter()
            .filter_map(|&name| self.texture(name).map(|image| (name, image)))
            .collect();

        if entries.is_empty() {
            return (
                TransparentImage {
                    pixels: Vec::new(),
                    width: 0,
                    height: 0,
                },
                BoundsLookup::new(),
            );
        }

        let max_width = entries
            .iter()
            .map(|(_, img)| img.width())
            .max()
            .unwrap_or(128);
        let total_pixels: usize = entries.iter().map(|(_, img)| img.num_pixels()).sum();

        let mut atlas_width = next_pow2(max_width.max(128));
        let mut atlas_height = 128;

        while atlas_width * atlas_height < total_pixels {
            if atlas_width <= atlas_height {
                atlas_width *= 2;
            } else {
                atlas_height *= 2;
            }
        }

        let mut positions = Vec::with_capacity(entries.len());
        loop {
            positions.clear();
            let mut offset_x = 0;
            let mut offset_y = 0;
            let mut row_height = 0;
            let mut failed = false;

            for (_, image) in &entries {
                let width = image.width();
                let height = image.height();

                if offset_x + width > atlas_width {
                    offset_x = 0;
                    offset_y += row_height;
                    row_height = 0;
                }

                if height > row_height {
                    row_height = height;
                }

                if offset_y + height > atlas_height {
                    failed = true;
                    break;
                }

                positions.push((offset_x, offset_y, row_height));
                offset_x += width;
            }

            if failed {
                if atlas_width <= atlas_height {
                    atlas_width *= 2;
                } else {
                    atlas_height *= 2;
                }
                if atlas_width > 4096 || atlas_height > 4096 {
                    break;
                }
            } else {
                break;
            }
        }

        let mut atlas = Image::new(atlas_width, atlas_height).expect("atlas too large");
        let mut bound_map = IndexMap::new();

        for (index, (name, image)) in entries.iter().enumerate() {
            let (pos_x, pos_y, row_height) = positions[index];
            atlas.blit(image, (pos_x as isize, pos_y as isize), true);
            bound_map.insert(
                *name,
                Bounds {
                    x: pos_x as f32,
                    y: pos_y as f32,
                    width: image.width() as f32,
                    height: image.height() as f32,
                    num_frames: 1,
                    row_height,
                    x_offset: 0.0,
                },
            );
        }

        (
            TransparentImage {
                pixels: atlas.into_pixels(),
                width: atlas_width,
                height: atlas_height,
            },
            bound_map,
        )
    }

    pub fn build_sprite_atlas_with_animations<I>(
        &self,
        sprite_info_iter: I,
    ) -> (TransparentImage, BoundsLookup)
    where
        I: IntoIterator<Item = (WadName, [u8; 4], &'static [u8])>,
    {
        let sprite_infos: Vec<_> = sprite_info_iter.into_iter().collect();

        if sprite_infos.is_empty() {
            return (
                TransparentImage {
                    pixels: Vec::new(),
                    width: 0,
                    height: 0,
                },
                BoundsLookup::new(),
            );
        }

        struct SpriteGroup<'a> {
            first_frame_name: WadName,
            frames: Vec<(WadName, &'a Image)>,
            total_width: usize,
            height: usize,
            x_offset: isize,
        }

        let mut groups: Vec<SpriteGroup> = Vec::new();
        let mut seen_first_frames: std::collections::HashSet<WadName> =
            std::collections::HashSet::new();

        for (first_frame_name, prefix, sequence) in &sprite_infos {
            if seen_first_frames.contains(first_frame_name) {
                continue;
            }
            seen_first_frames.insert(*first_frame_name);

            let mut frames = Vec::new();
            let mut total_width = 0;
            let mut max_height = 0;
            let mut first_x_offset = 0isize;

            for &frame_char in *sequence {
                let mut frame_name_bytes = [0u8; 8];
                frame_name_bytes[0..4].copy_from_slice(prefix);
                frame_name_bytes[4] = frame_char;
                frame_name_bytes[5] = b'0';

                let mut frame_name = WadName::from_bytes(&frame_name_bytes).ok();
                let mut image = frame_name.and_then(|n| self.texture(n));

                if image.is_none() {
                    frame_name_bytes[5] = b'1';
                    frame_name = WadName::from_bytes(&frame_name_bytes).ok();
                    image = frame_name.and_then(|n| self.texture(n));
                }

                if let (Some(name), Some(img)) = (frame_name, image) {
                    if frames.is_empty() {
                        first_x_offset = img.x_offset();
                    }
                    total_width += img.width();
                    max_height = max_height.max(img.height());
                    frames.push((name, img));
                }
            }

            if !frames.is_empty() {
                groups.push(SpriteGroup {
                    first_frame_name: *first_frame_name,
                    frames,
                    total_width,
                    height: max_height,
                    x_offset: first_x_offset,
                });
            }
        }

        if groups.is_empty() {
            return (
                TransparentImage {
                    pixels: Vec::new(),
                    width: 0,
                    height: 0,
                },
                BoundsLookup::new(),
            );
        }

        let max_group_width = groups.iter().map(|g| g.total_width).max().unwrap_or(128);
        let total_pixels: usize = groups.iter().map(|g| g.total_width * g.height).sum();

        let mut atlas_width = next_pow2(max_group_width.max(128));
        let mut atlas_height = 128;

        while atlas_width * atlas_height < total_pixels {
            if atlas_width <= atlas_height {
                atlas_width *= 2;
            } else {
                atlas_height *= 2;
            }
        }

        struct GroupPosition {
            x: usize,
            y: usize,
            row_height: usize,
        }

        let mut positions: Vec<GroupPosition> = Vec::with_capacity(groups.len());
        loop {
            positions.clear();
            let mut offset_x = 0;
            let mut offset_y = 0;
            let mut row_height = 0;
            let mut failed = false;

            for group in &groups {
                if offset_x + group.total_width > atlas_width {
                    offset_x = 0;
                    offset_y += row_height;
                    row_height = 0;
                }

                if group.height > row_height {
                    row_height = group.height;
                }

                if offset_y + group.height > atlas_height {
                    failed = true;
                    break;
                }

                positions.push(GroupPosition {
                    x: offset_x,
                    y: offset_y,
                    row_height,
                });
                offset_x += group.total_width;
            }

            if failed {
                if atlas_width <= atlas_height {
                    atlas_width *= 2;
                } else {
                    atlas_height *= 2;
                }
                if atlas_width > 8192 || atlas_height > 8192 {
                    break;
                }
            } else {
                break;
            }
        }

        let mut atlas = Image::new(atlas_width, atlas_height).expect("sprite atlas too large");
        let mut bound_map = IndexMap::new();

        for (group_index, group) in groups.iter().enumerate() {
            let group_pos = &positions[group_index];
            let mut frame_x = group_pos.x;
            let first_frame_width = group
                .frames
                .first()
                .map(|(_, img)| img.width())
                .unwrap_or(0);

            for (_, image) in &group.frames {
                atlas.blit(image, (frame_x as isize, group_pos.y as isize), true);
                frame_x += image.width();
            }

            bound_map.insert(
                group.first_frame_name,
                Bounds {
                    x: group_pos.x as f32,
                    y: group_pos.y as f32,
                    width: first_frame_width as f32,
                    height: group.height as f32,
                    num_frames: group.frames.len(),
                    row_height: group_pos.row_height,
                    x_offset: group.x_offset as f32,
                },
            );
        }

        (
            TransparentImage {
                pixels: atlas.into_pixels(),
                width: atlas_width,
                height: atlas_height,
            },
            bound_map,
        )
    }

    pub fn build_flat_atlas<I>(&self, names_iter: I) -> (OpaqueImage, BoundsLookup)
    where
        I: IntoIterator<Item = WadName>,
    {
        let names: Vec<WadName> = names_iter.into_iter().collect();

        let entries: Vec<_> = names
            .iter()
            .filter_map(|&name| self.flat(name).map(|flat| (name, flat)))
            .collect();

        let num_entries = entries.len();
        let flats_per_row = ((num_entries as f64).sqrt().ceil() as usize).max(1);
        let num_rows = num_entries.div_ceil(flats_per_row);

        let width = next_pow2(flats_per_row * 64);
        let height = next_pow2(num_rows * 64);

        let mut data = vec![255u8; width * height];
        let mut offsets = IndexMap::new();

        for (index, (name, flat)) in entries.iter().enumerate() {
            let row = index / flats_per_row;
            let column = index % flats_per_row;
            let offset_x = column * 64;
            let offset_y = row * 64;

            for y in 0..64 {
                for x in 0..64 {
                    let flat_index = x + y * 64;
                    if flat_index < flat.len() {
                        data[offset_x + x + (y + offset_y) * width] = flat[flat_index];
                    }
                }
            }

            offsets.insert(
                *name,
                Bounds {
                    x: offset_x as f32,
                    y: offset_y as f32,
                    width: 64.0,
                    height: 64.0,
                    num_frames: 1,
                    row_height: 64,
                    x_offset: 0.0,
                },
            );
        }

        (
            OpaqueImage {
                pixels: data,
                width,
                height,
            },
            offsets,
        )
    }

    pub fn build_sky_texture(&self, level_index: usize) -> Option<OpaqueImage> {
        let sky_name = match level_index / 9 {
            0 => WadName::from_bytes(b"SKY1\0\0\0\0").ok()?,
            1 => WadName::from_bytes(b"SKY2\0\0\0\0").ok()?,
            2 => WadName::from_bytes(b"SKY3\0\0\0\0").ok()?,
            _ => WadName::from_bytes(b"SKY1\0\0\0\0").ok()?,
        };

        let image = self.texture(sky_name)?;
        let width = image.width();
        let height = image.height();

        let pixels_u16 = image.pixels();
        let mut pixels = vec![0u8; width * height];
        for (index, &value) in pixels_u16.iter().enumerate() {
            pixels[index] = (value & 0xff) as u8;
        }

        Some(OpaqueImage {
            pixels,
            width,
            height,
        })
    }
}

fn next_pow2(x: usize) -> usize {
    let mut pow2 = 1;
    while pow2 < x {
        pow2 *= 2;
    }
    pow2
}

fn read_patches(wad: &Archive) -> Result<Vec<(WadName, Option<Image>)>> {
    let pnames_buffer = wad.required_named_lump(b"PNAMES\0\0")?.read_bytes()?;

    if pnames_buffer.len() < 4 {
        return Err(ArchiveError::BadLump("PNAMES too small".into()));
    }

    let num_patches = u32::from_le_bytes([
        pnames_buffer[0],
        pnames_buffer[1],
        pnames_buffer[2],
        pnames_buffer[3],
    ]) as usize;

    let mut patches = Vec::with_capacity(num_patches);
    let mut offset = 4;

    for _ in 0..num_patches {
        if offset + 8 > pnames_buffer.len() {
            break;
        }

        let name_bytes: [u8; 8] = pnames_buffer[offset..offset + 8].try_into().unwrap();
        offset += 8;

        let name = match WadName::from_bytes(&name_bytes) {
            Ok(n) => n,
            Err(_) => continue,
        };

        let image = if let Some(lump) = wad.named_lump(&name)? {
            let bytes = lump.read_bytes()?;
            Image::from_buffer(&bytes).ok()
        } else {
            None
        };

        patches.push((name, image));
    }

    Ok(patches)
}

fn read_textures(
    lump_buffer: &[u8],
    patches: &[(WadName, Option<Image>)],
    textures: &mut IndexMap<WadName, Image>,
) -> Result<usize> {
    if lump_buffer.len() < 4 {
        return Ok(0);
    }

    let num_textures = u32::from_le_bytes([
        lump_buffer[0],
        lump_buffer[1],
        lump_buffer[2],
        lump_buffer[3],
    ]) as usize;

    let offsets_end = 4 + num_textures * 4;
    if offsets_end > lump_buffer.len() {
        return Err(ArchiveError::BadLump(
            "Texture lump too small for offsets".into(),
        ));
    }

    for texture_index in 0..num_textures {
        let offset_pos = 4 + texture_index * 4;
        let offset = u32::from_le_bytes([
            lump_buffer[offset_pos],
            lump_buffer[offset_pos + 1],
            lump_buffer[offset_pos + 2],
            lump_buffer[offset_pos + 3],
        ]) as usize;

        if offset + 22 > lump_buffer.len() {
            continue;
        }

        let header_bytes = &lump_buffer[offset..];
        let name_bytes: [u8; 8] = header_bytes[0..8].try_into().unwrap();
        let name = match WadName::from_bytes(&name_bytes) {
            Ok(n) => n,
            Err(_) => continue,
        };

        let width = u16::from_le_bytes([header_bytes[12], header_bytes[13]]) as usize;
        let height = u16::from_le_bytes([header_bytes[14], header_bytes[15]]) as usize;
        let num_patches = u16::from_le_bytes([header_bytes[20], header_bytes[21]]) as usize;

        let mut image = match Image::new(width, height) {
            Ok(i) => i,
            Err(_) => continue,
        };

        let mut patch_offset = offset + 22;
        for patch_index in 0..num_patches {
            if patch_offset + 10 > lump_buffer.len() {
                break;
            }

            let origin_x =
                i16::from_le_bytes([lump_buffer[patch_offset], lump_buffer[patch_offset + 1]])
                    as isize;
            let origin_y =
                i16::from_le_bytes([lump_buffer[patch_offset + 2], lump_buffer[patch_offset + 3]])
                    as isize;
            let patch_id =
                u16::from_le_bytes([lump_buffer[patch_offset + 4], lump_buffer[patch_offset + 5]])
                    as usize;

            patch_offset += 10;

            if let Some((_, Some(patch))) = patches.get(patch_id) {
                let offset_y = if origin_y <= 0 { 0 } else { origin_y };
                image.blit(patch, (origin_x, offset_y), patch_index == 0);
            }
        }

        textures.insert(name, image);
    }

    Ok(num_textures)
}

fn read_flats(wad: &Archive) -> Result<IndexMap<WadName, Flat>> {
    let start = wad.required_named_lump(b"F_START\0")?.index();
    let end = wad.required_named_lump(b"F_END\0\0\0")?.index();

    let mut flats = IndexMap::new();
    for lump_index in start..end {
        let lump = wad.lump_by_index(lump_index)?;
        if lump.is_virtual() {
            continue;
        }
        let bytes = lump.read_bytes()?;
        if bytes.len() == 64 * 64 {
            flats.insert(lump.name(), bytes);
        }
    }
    Ok(flats)
}

fn read_sprites(wad: &Archive, textures: &mut IndexMap<WadName, Image>) -> Result<usize> {
    let start = wad.required_named_lump(b"S_START\0")?.index() + 1;
    let end = wad.required_named_lump(b"S_END\0\0\0")?.index();

    let mut count = 0;
    for index in start..end {
        let lump = wad.lump_by_index(index)?;
        let bytes = lump.read_bytes()?;
        if let Ok(texture) = Image::from_buffer(&bytes) {
            textures.insert(lump.name(), texture);
            count += 1;
        }
    }
    Ok(count)
}
