use leptos::prelude::*;
use leptos::web_sys;
use leptos_meta::*;
use leptos_router::{components::*, path};

mod articles;
mod components;
mod pages;

use crate::components::navigation::Navigation;
use crate::pages::article::Article;
use crate::pages::articles::Articles;
use crate::pages::home::Home;
use crate::pages::not_found::NotFound;

#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub is_dark: Signal<bool>,
    pub toggle_theme: WriteSignal<bool>,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let (is_dark, set_is_dark) = signal(true);

    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(theme)) = storage.get_item("theme") {
                    set_is_dark.set(theme == "dark");
                }
            }
        }
    });

    Effect::new(move |_| {
        let theme = if is_dark.get() { "dark" } else { "light" };
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("theme", theme);
            }
            if let Some(document) = window.document() {
                if let Some(element) = document.document_element() {
                    let class_list = element.class_list();
                    if is_dark.get() {
                        let _ = class_list.add_1("dark");
                    } else {
                        let _ = class_list.remove_1("dark");
                    }
                }
            }
        }
    });

    provide_context(ThemeContext {
        is_dark: is_dark.into(),
        toggle_theme: set_is_dark,
    });

    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme="dark" />

        <Title text="Leptos Example" />

        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Router>
            <Navigation />
            <Routes fallback=NotFound>
                <Route path=path!("/") view=Home />
                <Route path=path!("/articles") view=Articles />
                <Route path=path!("/articles/:slug") view=Article />
            </Routes>
        </Router>
    }
}
