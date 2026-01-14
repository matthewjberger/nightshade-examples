use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-screen bg-gray-900">
            <h1 class="text-6xl font-bold text-center mb-4 text-white">"404: Page not found"</h1>
            <h2 class="text-2xl text-center text-gray-400">"We couldn't find that page!"</h2>
        </div>
    }
}
