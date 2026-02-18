use leptos::prelude::*;
use claude_chat_mcp_protocol::{AgentStatus, BackendEvent, FrontendCommand};
use web_sys::wasm_bindgen::{self, JsCast};

#[derive(Clone)]
struct ToolUseBlock {
    tool_name: String,
    tool_id: String,
    input_json: String,
    finished: bool,
}

#[derive(Clone)]
enum MessageRole {
    User,
    Assistant,
}

#[derive(Clone)]
struct ChatMessage {
    role: MessageRole,
    content: String,
    thinking: String,
    tool_uses: Vec<ToolUseBlock>,
}

#[derive(Clone)]
enum StatusDisplay {
    Disconnected,
    Idle,
    Thinking,
    Streaming,
    UsingTool { tool_name: String },
}

impl StatusDisplay {
    fn label(&self) -> &str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Idle => "Ready",
            Self::Thinking => "Thinking...",
            Self::Streaming => "Streaming...",
            Self::UsingTool { .. } => "Using tool...",
        }
    }

    fn dot_class(&self) -> &str {
        match self {
            Self::Disconnected => "bg-red-500",
            Self::Idle => "bg-green-500",
            Self::Thinking => "bg-yellow-500",
            Self::Streaming => "bg-blue-500",
            Self::UsingTool { .. } => "bg-purple-500",
        }
    }
}

#[derive(Clone)]
struct ToastEntry {
    message: String,
    kind: String,
    id: u32,
}

#[component]
pub fn App() -> impl IntoView {
    let messages = RwSignal::new(Vec::<ChatMessage>::new());
    let streaming_text = RwSignal::new(String::new());
    let thinking_text = RwSignal::new(String::new());
    let active_tools = RwSignal::new(Vec::<ToolUseBlock>::new());
    let status = RwSignal::new(StatusDisplay::Disconnected);
    let session_id = RwSignal::new(Option::<String>::None);
    let input_text = RwSignal::new(String::new());
    let toasts = RwSignal::new(Vec::<ToastEntry>::new());
    let toast_counter = RwSignal::new(0u32);

    let finalize = move || {
        let text = streaming_text.get_untracked();
        let thinking = thinking_text.get_untracked();
        let tools = active_tools.get_untracked();
        if !text.is_empty() || !thinking.is_empty() || !tools.is_empty() {
            messages.update(|messages| {
                messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: text,
                    thinking,
                    tool_uses: tools,
                });
            });
        }
        streaming_text.set(String::new());
        thinking_text.set(String::new());
        active_tools.set(Vec::new());
    };

    let add_toast = move |message: String, kind: String| {
        let id = toast_counter.get_untracked();
        toast_counter.set(id + 1);
        toasts.update(|list| {
            list.push(ToastEntry {
                message,
                kind,
                id,
            });
        });

        let remove_id = id;
        let callback = wasm_bindgen::closure::Closure::once(move || {
            toasts.update(|list| {
                list.retain(|entry| entry.id != remove_id);
            });
        });
        let _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                5000,
            );
        callback.forget();
    };

    Effect::new(move |_| {
        nightshade::webview::connect(FrontendCommand::Ready, move |event| {
            match event {
                BackendEvent::Connected => {
                    status.set(StatusDisplay::Idle);
                }
                BackendEvent::StreamingStarted { session_id: sid } => {
                    session_id.set(Some(sid));
                    streaming_text.set(String::new());
                    thinking_text.set(String::new());
                    active_tools.set(Vec::new());
                }
                BackendEvent::TextDelta { text } => {
                    streaming_text.update(|current| current.push_str(&text));
                }
                BackendEvent::ThinkingDelta { text } => {
                    thinking_text.update(|current| current.push_str(&text));
                }
                BackendEvent::ToolUseStarted { tool_name, tool_id } => {
                    active_tools.update(|tools| {
                        tools.push(ToolUseBlock {
                            tool_name,
                            tool_id,
                            input_json: String::new(),
                            finished: false,
                        });
                    });
                }
                BackendEvent::ToolUseInputDelta { tool_id, partial_json } => {
                    active_tools.update(|tools| {
                        if let Some(tool) = tools.iter_mut().rev().find(|tool| tool.tool_id == tool_id || tool_id.is_empty()) {
                            tool.input_json.push_str(&partial_json);
                        }
                    });
                }
                BackendEvent::ToolUseFinished { tool_id } => {
                    active_tools.update(|tools| {
                        if let Some(tool) = tools.iter_mut().rev().find(|tool| tool.tool_id == tool_id || tool_id.is_empty()) {
                            tool.finished = true;
                        }
                    });
                }
                BackendEvent::TurnComplete { .. } => {}
                BackendEvent::RequestComplete { .. } => {
                    finalize();
                }
                BackendEvent::Error { message } => {
                    finalize();
                    messages.update(|messages| {
                        messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: format!("Error: {message}"),
                            thinking: String::new(),
                            tool_uses: Vec::new(),
                        });
                    });
                }
                BackendEvent::StatusUpdate { status: agent_status } => {
                    status.set(match agent_status {
                        AgentStatus::Idle => StatusDisplay::Idle,
                        AgentStatus::Thinking => StatusDisplay::Thinking,
                        AgentStatus::Streaming => StatusDisplay::Streaming,
                        AgentStatus::UsingTool { tool_name } => StatusDisplay::UsingTool { tool_name },
                    });
                }
                BackendEvent::ShowNotification { message, kind } => {
                    add_toast(message, kind);
                }
            }
        });
    });

    let is_busy = move || !matches!(status.get(), StatusDisplay::Idle | StatusDisplay::Disconnected);
    let can_send = move || !input_text.get().trim().is_empty() && !is_busy();

    let send_prompt = move || {
        let text = input_text.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        messages.update(|messages| {
            messages.push(ChatMessage {
                role: MessageRole::User,
                content: text.clone(),
                thinking: String::new(),
                tool_uses: Vec::new(),
            });
        });
        nightshade::webview::send(&FrontendCommand::SendPrompt {
            prompt: text,
            session_id: session_id.get_untracked(),
            model: None,
        });
        input_text.set(String::new());
    };

    let on_keydown = move |event: web_sys::KeyboardEvent| {
        if event.key() == "Enter" && event.ctrl_key() && can_send() {
            event.prevent_default();
            send_prompt();
        }
    };

    let on_send = move |_| {
        if can_send() {
            send_prompt();
        }
    };

    let on_cancel = move |_| {
        nightshade::webview::send(&FrontendCommand::CancelRequest);
    };

    view! {
        <div class="h-screen flex flex-col bg-[#0d1117] text-[#c9d1d9] font-mono">
            // Toolbar
            <div class="flex items-center justify-between px-4 py-2 bg-[#161b22] border-b border-[#30363d]">
                <div class="flex items-center gap-4">
                    <span class="text-sm font-bold tracking-wide">"CLAUDE CHAT"</span>
                    <span class="text-xs font-bold text-purple-400">"MCP"</span>
                    <div class="flex items-center gap-2">
                        <div class={move || format!("w-2 h-2 rounded-full {}", status.get().dot_class())}></div>
                        <span class="text-xs text-[#8b949e]">{move || status.get().label().to_string()}</span>
                        {move || {
                            if let StatusDisplay::UsingTool { tool_name } = status.get() {
                                format!(" ({tool_name})")
                            } else {
                                String::new()
                            }
                        }}
                    </div>
                </div>
                <div class="text-xs text-[#484f58]">
                    {move || session_id.get().map(|id| {
                        if id.len() > 12 { format!("{}...", &id[..12]) } else { id }
                    }).unwrap_or_default()}
                </div>
            </div>

            // Messages
            <div class="flex-1 overflow-y-auto px-4 py-4" id="chat-scroll">
                {move || {
                    let msgs = messages.get();
                    let is_thinking = matches!(status.get(), StatusDisplay::Thinking);
                    if msgs.is_empty() && streaming_text.get().is_empty() && thinking_text.get().is_empty() && !is_thinking {
                        view! {
                            <div class="flex flex-col items-center justify-center h-full text-[#484f58] text-sm gap-2">
                                <div>"Ask Claude to build a 3D scene"</div>
                                <div class="text-xs">"Claude can spawn entities, set colors, add lights, and more via MCP"</div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                {msgs.into_iter().map(|message| {
                                    let is_user = matches!(message.role, MessageRole::User);
                                    let container_class = if is_user { "flex justify-end mb-3" } else { "flex justify-start mb-3" };
                                    let bubble_class = if is_user {
                                        "max-w-[80%] px-4 py-2.5 rounded-lg bg-[#1f6feb] text-white"
                                    } else {
                                        "max-w-[80%] px-4 py-2.5 rounded-lg bg-[#161b22] text-[#c9d1d9] border border-[#30363d]"
                                    };
                                    let thinking = message.thinking.clone();
                                    let has_thinking = !thinking.is_empty();
                                    let tool_uses = message.tool_uses.clone();

                                    view! {
                                        <div class={container_class}>
                                            <div class={bubble_class}>
                                                {if has_thinking && !is_user {
                                                    Some(view! {
                                                        <details class="mb-2">
                                                            <summary class="text-xs text-yellow-500 cursor-pointer hover:text-yellow-400">"Thinking"</summary>
                                                            <pre class="whitespace-pre-wrap break-words font-mono text-xs leading-relaxed mt-1 text-[#8b949e] pl-3 border-l-2 border-[#30363d]">{thinking}</pre>
                                                        </details>
                                                    })
                                                } else {
                                                    None
                                                }}
                                                <pre class="whitespace-pre-wrap break-words font-mono text-sm leading-relaxed m-0">{message.content}</pre>
                                                {if !tool_uses.is_empty() {
                                                    Some(view! {
                                                        <div class="mt-2">
                                                            {tool_uses.into_iter().map(|tool| {
                                                                view! {
                                                                    <div class="my-1 border border-[#30363d] rounded-md overflow-hidden">
                                                                        <details>
                                                                            <summary class="flex items-center gap-2 px-3 py-1.5 bg-[#161b22] text-xs cursor-pointer hover:bg-[#1c2129]">
                                                                                <span class="text-purple-400 font-medium">{tool.tool_name}</span>
                                                                                {if tool.finished {
                                                                                    view! { <span class="text-green-500 ml-auto">"done"</span> }.into_any()
                                                                                } else {
                                                                                    view! { <span class="text-yellow-500 ml-auto animate-pulse">"running..."</span> }.into_any()
                                                                                }}
                                                                            </summary>
                                                                            {if !tool.input_json.is_empty() {
                                                                                Some(view! {
                                                                                    <pre class="px-3 py-2 text-xs text-[#8b949e] bg-[#0d1117] overflow-x-auto whitespace-pre-wrap break-all">{tool.input_json}</pre>
                                                                                })
                                                                            } else {
                                                                                None
                                                                            }}
                                                                        </details>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    })
                                                } else {
                                                    None
                                                }}
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}

                                // Streaming bubble
                                {move || {
                                    let thinking = thinking_text.get();
                                    let text = streaming_text.get();
                                    let tools = active_tools.get();
                                    let current_status = status.get();
                                    let is_thinking = matches!(current_status, StatusDisplay::Thinking);
                                    let is_active = !text.is_empty() || !tools.is_empty() || !thinking.is_empty() || is_thinking;

                                    if is_active {
                                        Some(view! {
                                            <div class="flex justify-start mb-3">
                                                <div class="max-w-[80%] px-4 py-2.5 rounded-lg bg-[#161b22] text-[#c9d1d9] border border-[#30363d]">
                                                    {if !thinking.is_empty() {
                                                        view! {
                                                            <div class="mb-3 pb-3 border-b border-[#30363d]">
                                                                <span class="text-yellow-500 text-xs">"Thinking"</span>
                                                                <pre class="whitespace-pre-wrap break-words font-mono text-xs leading-relaxed mt-1 text-[#8b949e]">{thinking}</pre>
                                                            </div>
                                                        }.into_any()
                                                    } else if is_thinking && text.is_empty() {
                                                        view! {
                                                            <div class="mb-3 pb-3 border-b border-[#30363d]">
                                                                <span class="text-yellow-500 text-xs animate-pulse">"Thinking..."</span>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <div></div> }.into_any()
                                                    }}
                                                    {if !text.is_empty() {
                                                        Some(view! {
                                                            <pre class="whitespace-pre-wrap break-words font-mono text-sm leading-relaxed m-0">{text}</pre>
                                                        })
                                                    } else {
                                                        None
                                                    }}
                                                    {if !tools.is_empty() {
                                                        Some(view! {
                                                            <div class="mt-2">
                                                                {tools.into_iter().map(|tool| {
                                                                    view! {
                                                                        <div class="my-1 border border-[#30363d] rounded-md overflow-hidden">
                                                                            <div class="flex items-center gap-2 px-3 py-1.5 bg-[#161b22] text-xs">
                                                                                <span class="text-purple-400 font-medium">{tool.tool_name}</span>
                                                                                {if tool.finished {
                                                                                    view! { <span class="text-green-500 ml-auto">"done"</span> }.into_any()
                                                                                } else {
                                                                                    view! { <span class="text-yellow-500 ml-auto animate-pulse">"running..."</span> }.into_any()
                                                                                }}
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        })
                                                    } else {
                                                        None
                                                    }}
                                                    <span class="inline-block w-2 h-4 bg-[#c9d1d9] animate-pulse ml-0.5"></span>
                                                </div>
                                            </div>
                                        })
                                    } else {
                                        None
                                    }
                                }}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            // Input
            <div class="px-4 py-3 bg-[#161b22] border-t border-[#30363d]">
                <div class="flex gap-2">
                    <textarea
                        class="flex-1 bg-[#0d1117] text-[#c9d1d9] border border-[#30363d] rounded-lg px-3 py-2 text-sm font-mono resize-none focus:outline-none focus:border-[#58a6ff] placeholder-[#484f58]"
                        placeholder="Type a prompt... (Ctrl+Enter to send)"
                        rows="3"
                        prop:value=move || input_text.get()
                        on:input=move |event| {
                            let target = event.target().unwrap();
                            let textarea: web_sys::HtmlTextAreaElement = target.unchecked_into();
                            input_text.set(textarea.value());
                        }
                        on:keydown=on_keydown
                    />
                    <div class="flex flex-col gap-1">
                        <button
                            class="px-4 py-2 bg-[#238636] text-white text-sm rounded-lg hover:bg-[#2ea043] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
                            on:click=on_send
                            disabled=move || !can_send()
                        >
                            "Send"
                        </button>
                        <button
                            class="px-4 py-2 bg-[#da3633] text-white text-sm rounded-lg hover:bg-[#f85149] disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
                            on:click=on_cancel
                            disabled=move || !is_busy()
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </div>

            // Toast notifications
            <div class="fixed bottom-4 right-4 flex flex-col gap-2 z-50 pointer-events-none">
                {move || {
                    toasts.get().into_iter().map(|toast| {
                        let accent_class = match toast.kind.as_str() {
                            "warning" => "border-l-yellow-500",
                            "error" => "border-l-red-500",
                            "success" => "border-l-green-500",
                            _ => "border-l-blue-500",
                        };
                        let icon = match toast.kind.as_str() {
                            "warning" => "!",
                            "error" => "x",
                            "success" => "v",
                            _ => "i",
                        };
                        let icon_class = match toast.kind.as_str() {
                            "warning" => "text-yellow-500",
                            "error" => "text-red-500",
                            "success" => "text-green-500",
                            _ => "text-blue-500",
                        };
                        view! {
                            <div class={format!("pointer-events-auto px-4 py-3 bg-[#161b22] border border-[#30363d] border-l-4 {accent_class} rounded-lg shadow-lg min-w-[280px] max-w-[400px] animate-slide-in")}>
                                <div class="flex items-start gap-3">
                                    <span class={format!("text-sm font-bold {icon_class}")}>{icon}</span>
                                    <span class="text-sm text-[#c9d1d9]">{toast.message}</span>
                                </div>
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        </div>

        <style>
            {"@keyframes slide-in { from { transform: translateX(100%); opacity: 0; } to { transform: translateX(0); opacity: 1; } } .animate-slide-in { animation: slide-in 0.3s ease-out; }"}
        </style>
    }
}
