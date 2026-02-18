use leptos::prelude::*;
use web_host_protocol::{BackendEvent, FrontendCommand};

#[component]
pub fn App() -> impl IntoView {
    let (connected, set_connected) = signal(false);
    let (log, set_log) = signal(Vec::<String>::new());
    let (count, set_count) = signal(0u32);

    Effect::new(move |_| {
        nightshade::webview::connect(FrontendCommand::Ready, move |event| match event {
            BackendEvent::Connected => {
                set_connected.set(true);
                set_log.update(|log| log.push("Connected".into()));
            }
            BackendEvent::RandomNumber { request_id, value } => {
                set_log.update(|log| log.push(format!("#{request_id}: {value}")));
            }
        });
    });

    let request = move |_| {
        let id = count.get();
        set_count.set(id + 1);
        nightshade::webview::send(&FrontendCommand::RequestRandomNumber { request_id: id });
    };

    view! {
        <div class="min-h-screen bg-[#1a1a1a] text-white p-8 font-mono">
            <h1 class="text-2xl mb-4">"nightshade-webview"</h1>
            <p class="text-neutral-400 mb-6">"Bidirectional IPC demo"</p>

            <div class="mb-4">
                "Status: "
                <span class=move || if connected.get() { "text-green-400" } else { "text-red-400" }>
                    {move || if connected.get() { "Connected" } else { "Connecting..." }}
                </span>
            </div>

            <button
                class="px-4 py-2 bg-blue-500 rounded disabled:opacity-50"
                on:click=request
                disabled=move || !connected.get()
            >
                "Request Random Number"
            </button>

            <div class="mt-6 bg-black/50 rounded p-4 max-h-48 overflow-y-auto">
                {move || log.get().into_iter().map(|entry| view! {
                    <div class="py-1 border-b border-neutral-800">{entry}</div>
                }).collect_view()}
            </div>
        </div>
    }
}
