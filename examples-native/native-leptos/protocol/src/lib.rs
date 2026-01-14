#![no_std]
extern crate alloc;

use alloc::string::String;
use serde::{Deserialize, Serialize};

macro_rules! impl_base64 {
    ($($type:ty),*) => {$(
        impl $type {
            pub fn to_base64(&self) -> Option<String> {
                use base64::Engine;
                postcard::to_allocvec(self)
                    .ok()
                    .map(|b| base64::engine::general_purpose::STANDARD.encode(&b))
            }

            pub fn from_base64(s: &str) -> Option<Self> {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD
                    .decode(s)
                    .ok()
                    .and_then(|b| postcard::from_bytes(&b).ok())
            }
        }
    )*};
}

#[derive(Clone, Serialize, Deserialize)]
pub enum FrontendCommand {
    Ready,
    RequestRandomNumber { request_id: u32 },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum BackendEvent {
    Connected,
    RandomNumber { request_id: u32, value: u32 },
}

impl_base64!(FrontendCommand, BackendEvent);
