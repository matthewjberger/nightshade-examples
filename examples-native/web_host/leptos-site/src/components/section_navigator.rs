use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const SECTIONS: &[&str] = &["hero", "about", "experience", "projects"];

#[component]
pub fn SectionNavigator() -> impl IntoView {
    let (current_section, set_current_section) = signal(0usize);

    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let set_section = set_current_section;

            let update_from_hash = move || {
                if let Some(window) = web_sys::window() {
                    if let Ok(hash) = window.location().hash() {
                        if !hash.is_empty() {
                            let hash_id = hash.trim_start_matches('#');
                            if let Some(index) = SECTIONS.iter().position(|&section| section == hash_id) {
                                set_section.set(index);
                            }
                        } else {
                            set_section.set(0);
                        }
                    }
                }
            };

            update_from_hash();

            let hash_closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                update_from_hash();
            }) as Box<dyn FnMut(_)>);

            let _ = window.add_event_listener_with_callback("hashchange", hash_closure.as_ref().unchecked_ref());
            hash_closure.forget();

            let set_section_scroll = set_current_section;
            let scroll_closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        let window_height = window.inner_height().unwrap().as_f64().unwrap_or(0.0);

                        for (index, section_id) in SECTIONS.iter().enumerate() {
                            if let Some(element) = document.get_element_by_id(section_id) {
                                let rect = element.get_bounding_client_rect();
                                if rect.top() <= window_height / 3.0 && rect.bottom() >= window_height / 3.0 {
                                    set_section_scroll.set(index);
                                    break;
                                }
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);

            let _ = window.add_event_listener_with_callback("scroll", scroll_closure.as_ref().unchecked_ref());
            scroll_closure.forget();

            let set_section_key = set_current_section;
            let key_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                let key = event.key();

                if key == "ArrowUp" || key == "ArrowDown" {
                    event.prevent_default();

                    let current = current_section.get();
                    let new_index = if key == "ArrowUp" {
                        if current > 0 {
                            current - 1
                        } else {
                            0
                        }
                    } else {
                        (current + 1).min(SECTIONS.len() - 1)
                    };

                    set_section_key.set(new_index);

                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Some(element) = document.get_element_by_id(SECTIONS[new_index]) {
                                element.scroll_into_view();
                            }
                        }
                    }
                } else if (key == "ArrowLeft" || key == "ArrowRight") && current_section.get() == 2 {
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            let buttons = document.query_selector_all(".experience-nav-button").ok();
                            if let Some(buttons) = buttons {
                                let button_index = if key == "ArrowLeft" { 0 } else { 1 };
                                if let Some(button) = buttons.item(button_index) {
                                    if let Some(html_button) = button.dyn_ref::<web_sys::HtmlElement>() {
                                        html_button.click();
                                    }
                                }
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);

            let _ = window.add_event_listener_with_callback("keydown", key_closure.as_ref().unchecked_ref());
            key_closure.forget();
        }
    });

    let navigate_up = move |_| {
        let new_index = if current_section.get() > 0 {
            current_section.get() - 1
        } else {
            0
        };
        set_current_section.set(new_index);
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(element) = document.get_element_by_id(SECTIONS[new_index]) {
                    element.scroll_into_view();
                }
            }
        }
    };

    let navigate_down = move |_| {
        let new_index = (current_section.get() + 1).min(SECTIONS.len() - 1);
        set_current_section.set(new_index);
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(element) = document.get_element_by_id(SECTIONS[new_index]) {
                    element.scroll_into_view();
                }
            }
        }
    };

    let is_at_top = move || current_section.get() == 0;
    let is_at_bottom = move || current_section.get() >= SECTIONS.len() - 1;

    view! {
        <div class="fixed left-8 top-1/2 -translate-y-1/2 z-40 flex items-center gap-6">
            <div class="flex flex-col gap-4">
                <button
                    on:click=navigate_up
                    disabled=is_at_top
                    class="w-14 h-14 rounded-full bg-white dark:bg-gray-800 border-2 border-gray-300 dark:border-gray-700 shadow-lg flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-700 hover:border-blue-500 dark:hover:border-blue-400 transition-all disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:bg-white dark:disabled:hover:bg-gray-800 disabled:hover:border-gray-300 dark:disabled:hover:border-gray-700"
                    aria-label="Previous section"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-gray-700 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
                    </svg>
                </button>

                <button
                    on:click=navigate_down
                    disabled=is_at_bottom
                    class="w-14 h-14 rounded-full bg-white dark:bg-gray-800 border-2 border-gray-300 dark:border-gray-700 shadow-lg flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-700 hover:border-blue-500 dark:hover:border-blue-400 transition-all disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:bg-white dark:disabled:hover:bg-gray-800 disabled:hover:border-gray-300 dark:disabled:hover:border-gray-700"
                    aria-label="Next section"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-gray-700 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                </button>
            </div>

            <div class="flex flex-col gap-2">
                {SECTIONS.iter().enumerate().map(|(index, section)| {
                    let section_name = match *section {
                        "hero" => "Home",
                        name => {
                            let mut chars = name.chars();
                            match chars.next() {
                                None => "",
                                Some(first) => {
                                    let rest: String = chars.collect();
                                    Box::leak(format!("{}{}", first.to_uppercase(), rest).into_boxed_str())
                                }
                            }
                        }
                    };

                    let is_selected = move || current_section.get() == index;
                    let navigate_to = move |_| {
                        set_current_section.set(index);
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                if let Some(element) = document.get_element_by_id(SECTIONS[index]) {
                                    element.scroll_into_view();
                                }
                            }
                        }
                    };

                    view! {
                        <button
                            on:click=navigate_to
                            class=move || {
                                if is_selected() {
                                    "text-white dark:text-white text-lg font-medium transition-all cursor-pointer hover:text-blue-400 dark:hover:text-blue-400"
                                } else {
                                    "text-gray-400 dark:text-gray-500 text-base transition-all cursor-pointer hover:text-gray-600 dark:hover:text-gray-400"
                                }
                            }
                        >
                            {section_name}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
