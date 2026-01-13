use serde::de::{Deserialize, Deserializer, Error as SerdeDeError};
use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::result::Result as StdResult;
use std::str::{self, FromStr};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct WadName([u8; 8]);

impl Hash for WadName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl WadName {
    pub fn from_bytes(value: &[u8]) -> Result<WadName, WadNameError> {
        let mut name = [0u8; 8];
        let mut nulled = false;
        for (dest, &src) in name.iter_mut().zip(value.iter()) {
            if !src.is_ascii() {
                return Err(WadNameError::InvalidByte(src));
            }

            let new_byte = match src.to_ascii_uppercase() {
                b @ b'A'..=b'Z'
                | b @ b'0'..=b'9'
                | b @ b'_'
                | b @ b'-'
                | b @ b'['
                | b @ b']'
                | b @ b'%'
                | b @ b'\\' => b,
                b'\0' => {
                    nulled = true;
                    break;
                }
                b => {
                    return Err(WadNameError::InvalidByte(b));
                }
            };
            *dest = new_byte;
        }

        if !nulled && value.len() > 8 {
            return Err(WadNameError::TooLong);
        }
        Ok(WadName(name))
    }

    pub fn as_str(&self) -> &str {
        let end = self.0.iter().position(|&b| b == 0).unwrap_or(8);
        str::from_utf8(&self.0[..end]).unwrap_or("")
    }
}

#[derive(Debug)]
pub enum WadNameError {
    InvalidByte(u8),
    TooLong,
}

impl std::fmt::Display for WadNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WadNameError::InvalidByte(b) => write!(f, "Invalid byte in WAD name: 0x{:02x}", b),
            WadNameError::TooLong => write!(f, "WAD name too long (max 8 bytes)"),
        }
    }
}

impl std::error::Error for WadNameError {}

impl FromStr for WadName {
    type Err = WadNameError;
    fn from_str(value: &str) -> Result<WadName, Self::Err> {
        WadName::from_bytes(value.as_bytes())
    }
}

impl fmt::Display for WadName {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

impl Deref for WadName {
    type Target = [u8; 8];
    fn deref(&self) -> &[u8; 8] {
        &self.0
    }
}

impl fmt::Debug for WadName {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "WadName({:?})", self.as_str())
    }
}

impl PartialEq<[u8; 8]> for WadName {
    fn eq(&self, rhs: &[u8; 8]) -> bool {
        self.deref() == rhs
    }
}

impl Borrow<[u8; 8]> for WadName {
    fn borrow(&self) -> &[u8; 8] {
        self.deref()
    }
}

impl AsRef<str> for WadName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'de> Deserialize<'de> for WadName {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        WadName::from_bytes(&<[u8; 8]>::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}
