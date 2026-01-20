#![no_std]

use serde::{Deserialize, Serialize};

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
