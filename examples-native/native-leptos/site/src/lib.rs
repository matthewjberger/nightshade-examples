use leptos::prelude::*;
use web_host_protocol::{BackendEvent, FrontendCommand};

pub mod bridge;

#[component]
pub fn App() -> impl IntoView {
    let (connected, set_connected) = signal(false);
    let (request_count, set_request_count) = signal(0u32);
    let (log_entries, set_log_entries) = signal(Vec::<String>::new());

    Effect::new(move |_| {
        bridge::set_backend_handler(move |event| match event {
            BackendEvent::Connected => {
                set_connected.set(true);
                set_log_entries.update(|e| e.push("Backend connected".to_string()));
            }
            BackendEvent::RandomNumber { request_id, value } => {
                set_log_entries.update(|e| e.push(format!("Request #{request_id}: {value}")));
            }
        });
    });

    let request_random = move |_| {
        let id = request_count.get();
        set_request_count.set(id + 1);
        bridge::send_command(FrontendCommand::RequestRandomNumber { request_id: id });
    };

    view! {
        <div style="font-family: system-ui, sans-serif; max-width: 600px; margin: 40px auto; padding: 20px; color: #e5e5e5;">
            <h1 style="margin-bottom: 8px; color: #fff;">"WebView IPC Demo"</h1>
            <p style="color: #999; margin-bottom: 24px;">
                "Bidirectional communication between Rust backend and WASM frontend using postcard serialization."
            </p>

            <div style="display: flex; align-items: center; gap: 12px; margin-bottom: 24px;">
                <span style="font-weight: 500; color: #e5e5e5;">"Status:"</span>
                <span style:color=move || if connected.get() { "#22c55e" } else { "#ef4444" }>
                    {move || if connected.get() { "Connected" } else { "Disconnected" }}
                </span>
            </div>

            <button
                on:click=request_random
                disabled=move || !connected.get()
                style="padding: 12px 24px; font-size: 16px; cursor: pointer; background: #3b82f6; color: white; border: none; border-radius: 6px;"
            >
                "Request Random Number"
            </button>

            <div style="margin-top: 32px;">
                <h2 style="font-size: 18px; margin-bottom: 12px; color: #fff;">"Log"</h2>
                <div style="background: #111; border-radius: 8px; padding: 16px; font-family: monospace; font-size: 14px; max-height: 300px; overflow-y: auto; color: #e5e5e5;">
                    {move || {
                        let entries = log_entries.get();
                        if entries.is_empty() {
                            view! { <div style="color: #666;">"No messages yet"</div> }.into_any()
                        } else {
                            entries.into_iter().map(|entry| {
                                view! {
                                    <div style="padding: 4px 0; border-bottom: 1px solid #333;">
                                        {entry}
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
