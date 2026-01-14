use crate::ThemeContext;
use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::components::A;

#[component]
pub fn Navigation() -> impl IntoView {
    let theme_context = expect_context::<ThemeContext>();

    let trigger_hash_change = move |hash: &str| {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(element) = document.get_element_by_id(hash) {
                    element.scroll_into_view();
                }
                let _ = window.location().set_hash(hash);
            }
        }
    };

    view! {
        <nav class="fixed top-0 left-0 right-0 bg-white dark:bg-gray-900 shadow-md z-50 border-b border-gray-200 dark:border-gray-800">
            <div class="max-w-6xl mx-auto px-4">
                <div class="flex justify-between items-center h-16">
                    <A
                        href="/"
                        on:click=move |_| {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_hash("");
                                if let Some(document) = window.document() {
                                    if let Some(element) = document.get_element_by_id("hero") {
                                        element.scroll_into_view();
                                    }
                                }
                            }
                        }
                        attr:class="text-xl font-bold text-gray-900 dark:text-white"
                    >
                        "Your Name"
                    </A>
                    <div class="flex items-center gap-6">
                        <a
                            href="/#about"
                            on:click=move |event: web_sys::MouseEvent| {
                                event.prevent_default();
                                trigger_hash_change("about");
                            }
                            class="text-gray-700 dark:text-gray-300 hover:text-blue-500 dark:hover:text-blue-400 transition-colors"
                        >
                            "About"
                        </a>
                        <a
                            href="/#experience"
                            on:click=move |event: web_sys::MouseEvent| {
                                event.prevent_default();
                                trigger_hash_change("experience");
                            }
                            class="text-gray-700 dark:text-gray-300 hover:text-blue-500 dark:hover:text-blue-400 transition-colors"
                        >
                            "Experience"
                        </a>
                        <a
                            href="/#projects"
                            on:click=move |event: web_sys::MouseEvent| {
                                event.prevent_default();
                                trigger_hash_change("projects");
                            }
                            class="text-gray-700 dark:text-gray-300 hover:text-blue-500 dark:hover:text-blue-400 transition-colors"
                        >
                            "Projects"
                        </a>
                        <A
                            href="/articles"
                            attr:class="text-gray-700 dark:text-gray-300 hover:text-blue-500 dark:hover:text-blue-400 transition-colors"
                        >
                            "Articles"
                        </A>
                        <button
                            on:click=move |_| theme_context.toggle_theme.update(|dark| *dark = !*dark)
                            class="text-gray-700 dark:text-gray-300 hover:text-blue-500 dark:hover:text-blue-400 transition-colors p-2"
                            aria-label="Toggle theme"
                        >
                            <Show
                                when=move || theme_context.is_dark.get()
                                fallback=move || view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                                    </svg>
                                }
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
                                </svg>
                            </Show>
                        </button>
                    </div>
                </div>
            </div>
        </nav>
    }
}
