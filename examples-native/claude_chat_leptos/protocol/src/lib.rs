#![no_std]
extern crate alloc;

use alloc::string::String;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum FrontendCommand {
    Ready,
    SendPrompt {
        prompt: String,
        session_id: Option<String>,
        model: Option<String>,
    },
    CancelRequest,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum BackendEvent {
    Connected,
    StreamingStarted {
        session_id: String,
    },
    TextDelta {
        text: String,
    },
    ThinkingDelta {
        text: String,
    },
    ToolUseStarted {
        tool_name: String,
        tool_id: String,
    },
    ToolUseInputDelta {
        tool_id: String,
        partial_json: String,
    },
    ToolUseFinished {
        tool_id: String,
    },
    TurnComplete {
        session_id: String,
    },
    RequestComplete {
        session_id: String,
        total_cost_usd: Option<f64>,
        num_turns: u32,
    },
    Error {
        message: String,
    },
    StatusUpdate {
        status: AgentStatus,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Streaming,
    UsingTool { tool_name: String },
}
