//! Save and load of [`RuntimeState`] via bincode, with a 4-byte magic and
//! a version number prepended. The `World` is rebuilt in code (see
//! `crate::game::build_world`) and is never part of a save file — only the
//! turn-to-turn state is.
//!
//! Save layout:
//!
//! ```text
//! [4 bytes: MAGIC = b"LHHS"] [2 bytes: VERSION (little-endian u16)] [bincode(RuntimeState)]
//! ```

use crate::data::RuntimeState;

const MAGIC: [u8; 4] = *b"LHHS";
const VERSION: u16 = 1;

/// Errors returned by `save` / `load`.
#[derive(Debug)]
pub enum SaveError {
    /// Underlying bincode failure.
    Bincode(bincode::Error),
    /// The input bytes were too short to contain the header.
    TooShort,
    /// The magic bytes did not match; this is not a lighthouse save file.
    BadMagic,
    /// The file's version is not one this build can read.
    UnsupportedVersion(u16),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Bincode(inner) => write!(f, "bincode error: {inner}"),
            SaveError::TooShort => f.write_str("save file is shorter than the header"),
            SaveError::BadMagic => f.write_str("save file magic does not match"),
            SaveError::UnsupportedVersion(version) => {
                write!(f, "save file version {version} is not supported")
            }
        }
    }
}

impl std::error::Error for SaveError {}

impl From<bincode::Error> for SaveError {
    fn from(value: bincode::Error) -> Self {
        SaveError::Bincode(value)
    }
}

/// Serialize the current runtime state to bytes with a magic+version header.
pub fn save(state: &RuntimeState) -> Result<Vec<u8>, SaveError> {
    let mut out = Vec::with_capacity(64);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    let payload = bincode::serialize(state)?;
    out.extend_from_slice(&payload);
    Ok(out)
}

/// Deserialize a runtime state, checking the header first.
pub fn load(bytes: &[u8]) -> Result<RuntimeState, SaveError> {
    if bytes.len() < MAGIC.len() + 2 {
        return Err(SaveError::TooShort);
    }
    if bytes[..MAGIC.len()] != MAGIC {
        return Err(SaveError::BadMagic);
    }
    let version_bytes: [u8; 2] = bytes[MAGIC.len()..MAGIC.len() + 2]
        .try_into()
        .map_err(|_| SaveError::TooShort)?;
    let version = u16::from_le_bytes(version_bytes);
    if version != VERSION {
        return Err(SaveError::UnsupportedVersion(version));
    }
    let state = bincode::deserialize(&bytes[MAGIC.len() + 2..])?;
    Ok(state)
}
