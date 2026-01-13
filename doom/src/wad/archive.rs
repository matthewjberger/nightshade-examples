use super::name::WadName;
use super::types::{WadInfo, WadLump};
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use std::borrow::Borrow;
use std::hash::Hash;
use std::io::{Cursor, Read};
use std::mem;

#[derive(Debug)]
pub enum ArchiveError {
    Io(std::io::Error),
    BadHeader(String),
    BadLump(String),
    MissingLump(String),
    Bincode(String),
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveError::Io(error) => write!(f, "IO error: {}", error),
            ArchiveError::BadHeader(msg) => write!(f, "Bad WAD header: {}", msg),
            ArchiveError::BadLump(msg) => write!(f, "Bad lump: {}", msg),
            ArchiveError::MissingLump(name) => write!(f, "Missing lump: {}", name),
            ArchiveError::Bincode(msg) => write!(f, "Bincode error: {}", msg),
        }
    }
}

impl std::error::Error for ArchiveError {}

impl From<std::io::Error> for ArchiveError {
    fn from(error: std::io::Error) -> Self {
        ArchiveError::Io(error)
    }
}

pub type Result<T> = std::result::Result<T, ArchiveError>;

const IWAD_HEADER: &[u8] = b"IWAD";

#[derive(Clone)]
pub struct Archive {
    data: Vec<u8>,
    index_map: IndexMap<WadName, usize>,
    lumps: Vec<LumpInfo>,
    levels: Vec<usize>,
}

#[derive(Copy, Clone, Debug)]
struct LumpInfo {
    name: WadName,
    offset: usize,
    size: usize,
}

impl std::fmt::Debug for Archive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Archive")
            .field("data_len", &self.data.len())
            .field("num_lumps", &self.lumps.len())
            .finish()
    }
}

impl Archive {
    pub fn from_bytes(data: &[u8]) -> Result<Archive> {
        let data = data.to_vec();
        let mut cursor = Cursor::new(&data);

        let header: WadInfo = bincode::deserialize_from(&mut cursor)
            .map_err(|e| ArchiveError::BadHeader(e.to_string()))?;

        if header.identifier != IWAD_HEADER {
            return Err(ArchiveError::BadHeader(format!(
                "Expected IWAD, got {:?}",
                header.identifier
            )));
        }

        let mut lumps = Vec::with_capacity(header.num_lumps as usize);
        let mut levels = Vec::with_capacity(64);
        let mut index_map = IndexMap::new();

        cursor.set_position(header.info_table_offset as u64);

        for index in 0..header.num_lumps {
            let fileinfo: WadLump = bincode::deserialize_from(&mut cursor)
                .map_err(|e| ArchiveError::BadLump(format!("Lump {}: {}", index, e)))?;

            index_map.insert(fileinfo.name, lumps.len());
            lumps.push(LumpInfo {
                name: fileinfo.name,
                offset: fileinfo.file_pos as usize,
                size: fileinfo.size as usize,
            });

            if &fileinfo.name == b"THINGS\0\0" && index > 0 {
                levels.push((index - 1) as usize);
            }
        }

        Ok(Archive {
            data,
            index_map,
            lumps,
            levels,
        })
    }

    pub fn level_lump(&self, level_index: usize) -> Result<LumpReader<'_>> {
        self.lump_by_index(self.levels[level_index])
    }

    pub fn required_named_lump(&self, name: &[u8; 8]) -> Result<LumpReader<'_>> {
        let wad_name =
            WadName::from_bytes(name).map_err(|e| ArchiveError::BadLump(e.to_string()))?;
        self.named_lump(&wad_name)?
            .ok_or_else(|| ArchiveError::MissingLump(wad_name.to_string()))
    }

    pub fn named_lump<Q>(&self, name: &Q) -> Result<Option<LumpReader<'_>>>
    where
        WadName: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self.index_map.get(name) {
            Some(&index) => self.lump_by_index(index).map(Some),
            None => Ok(None),
        }
    }

    pub fn lump_by_index(&self, index: usize) -> Result<LumpReader<'_>> {
        Ok(LumpReader {
            archive: self,
            info: self
                .lumps
                .get(index)
                .ok_or_else(|| ArchiveError::MissingLump(format!("index {}", index)))?,
            index,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LumpReader<'a> {
    archive: &'a Archive,
    info: &'a LumpInfo,
    index: usize,
}

impl<'a> LumpReader<'a> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> WadName {
        self.info.name
    }

    pub fn is_virtual(&self) -> bool {
        self.info.size == 0
    }

    pub fn decode_vec<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let info = self.info;
        let element_size = mem::size_of::<T>();
        if element_size == 0 || info.size == 0 || !info.size.is_multiple_of(element_size) {
            return Err(ArchiveError::BadLump(format!(
                "Invalid lump size {} for element size {}",
                info.size, element_size
            )));
        }
        let num_elements = info.size / element_size;

        let slice = &self.archive.data[info.offset..info.offset + info.size];
        let mut cursor = Cursor::new(slice);

        (0..num_elements)
            .map(|_| {
                bincode::deserialize_from(&mut cursor)
                    .map_err(|e| ArchiveError::Bincode(e.to_string()))
            })
            .collect()
    }

    pub fn read_blobs<B>(&self) -> Result<Vec<B>>
    where
        B: Default + AsMut<[u8]>,
    {
        let info = self.info;
        let blob_size = B::default().as_mut().len();
        if blob_size == 0 || info.size == 0 || !info.size.is_multiple_of(blob_size) {
            return Err(ArchiveError::BadLump(format!(
                "Invalid lump size {} for blob size {}",
                info.size, blob_size
            )));
        }
        let num_blobs = info.size / blob_size;

        let slice = &self.archive.data[info.offset..info.offset + info.size];
        let mut cursor = Cursor::new(slice);

        let mut blobs = Vec::with_capacity(num_blobs);
        for _ in 0..num_blobs {
            blobs.push(B::default());
            cursor.read_exact(blobs.last_mut().unwrap().as_mut())?;
        }
        Ok(blobs)
    }

    pub fn read_bytes(&self) -> Result<Vec<u8>> {
        let info = self.info;
        Ok(self.archive.data[info.offset..info.offset + info.size].to_vec())
    }
}
