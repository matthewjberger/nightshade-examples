use crate::components::section_navigator::SectionNavigator;
use crate::pages::about::About;
use crate::pages::experience::Experience;
use crate::pages::projects::Projects;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::components::A;

#[component]
pub fn Home() -> impl IntoView {
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            TimeoutFuture::new(100).await;

            if let Some(window) = web_sys::window() {
                if let Ok(hash) = window.location().hash() {
                    if !hash.is_empty() {
                        let hash_id = hash.trim_start_matches('#');
                        if let Some(document) = window.document() {
                            if let Some(element) = document.get_element_by_id(hash_id) {
                                element.scroll_into_view();
                            }
                        }
                    }
                }
            }
        });
    });

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
        <SectionNavigator />
        <div class="scroll-smooth">
            <section id="hero" class="min-h-screen flex items-center justify-center bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800">
                <div class="text-center px-4">
                    <div
                        class="rounded-lg mx-auto mb-6 border-2 border-blue-500 dark:border-blue-400 bg-gray-300 dark:bg-gray-700 flex items-center justify-center"
                        style="width: 180px; height: 180px;"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-20 w-20 text-gray-500 dark:text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                        </svg>
                    </div>
                    <h1 class="text-5xl md:text-7xl font-bold text-gray-900 dark:text-white mb-4">
                        "Hi, I'm " <span class="text-blue-600 dark:text-blue-400">"Your Name"</span> " 🦀"
                    </h1>
                    <p class="text-xl md:text-2xl text-gray-700 dark:text-gray-300 mb-8">
                        "Software Engineer"
                    </p>
                    <div class="flex gap-4 justify-center flex-wrap">
                        <a
                            href="#about"
                            on:click=move |event: web_sys::MouseEvent| {
                                event.prevent_default();
                                trigger_hash_change("about");
                            }
                            class="px-6 py-3 bg-green-500 text-white rounded-lg hover:bg-green-600 transition-colors inline-flex items-center gap-2 whitespace-nowrap"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0" style="width: 14px; height: 14px;" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clip-rule="evenodd"/>
                            </svg>
                            "View About"
                        </a>
                        <a
                            href="#experience"
                            on:click=move |event: web_sys::MouseEvent| {
                                event.prevent_default();
                                trigger_hash_change("experience");
                            }
                            class="px-6 py-3 bg-purple-500 text-white rounded-lg hover:bg-purple-600 transition-colors inline-flex items-center gap-2 whitespace-nowrap"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0" style="width: 14px; height: 14px;" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z" clip-rule="evenodd"/>
                            </svg>
                            "View Experience"
                        </a>
                        <a
                            href="https://www.linkedin.com"
                            target="_blank"
                            class="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors inline-flex items-center gap-2 whitespace-nowrap"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0" style="width: 14px; height: 14px;" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/>
                            </svg>
                            "View LinkedIn"
                        </a>
                        <a
                            href="https://github.com"
                            target="_blank"
                            class="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors inline-flex items-center gap-2 whitespace-nowrap"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0" style="width: 14px; height: 14px;" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                            </svg>
                            "View GitHub"
                        </a>
                        <A
                            href="/articles"
                            attr:class="px-6 py-3 bg-orange-500 text-white rounded-lg hover:bg-orange-600 transition-colors inline-flex items-center gap-2 whitespace-nowrap"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0" style="width: 14px; height: 14px;" viewBox="0 0 20 20" fill="currentColor">
                                <path d="M9 4.804A7.968 7.968 0 005.5 4c-1.255 0-2.443.29-3.5.804v10A7.969 7.969 0 015.5 14c1.669 0 3.218.51 4.5 1.385A7.962 7.962 0 0114.5 14c1.255 0 2.443.29 3.5.804v-10A7.968 7.968 0 0014.5 4c-1.255 0-2.443.29-3.5.804V12a1 1 0 11-2 0V4.804z"/>
                            </svg>
                            "View Articles"
                        </A>
                    </div>
                </div>
            </section>
            <About />
            <Experience />
            <Projects />
        </div>
    }
}
