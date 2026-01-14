pub mod building_web_apps_with_leptos;
pub mod getting_started_with_rust;
pub mod understanding_rust_ownership;

use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct ArticleMetadata {
    pub slug: &'static str,
    pub title: &'static str,
    pub date: &'static str,
    pub description: &'static str,
}

pub fn get_all_articles() -> Vec<ArticleMetadata> {
    vec![
        ArticleMetadata {
            slug: "getting-started-with-rust",
            title: "Getting Started with Rust",
            date: "2024-01-15",
            description: "A beginner's guide to the Rust programming language",
        },
        ArticleMetadata {
            slug: "building-web-apps-with-leptos",
            title: "Building Web Apps with Leptos",
            date: "2024-02-20",
            description: "Learn how to build reactive web applications using Leptos",
        },
        ArticleMetadata {
            slug: "understanding-rust-ownership",
            title: "Understanding Rust Ownership",
            date: "2024-03-10",
            description: "Deep dive into Rust's ownership system and borrowing rules",
        },
    ]
}

pub fn render_article(slug: &str) -> impl IntoView {
    match slug {
        "getting-started-with-rust" => getting_started_with_rust::Article().into_any(),
        "building-web-apps-with-leptos" => building_web_apps_with_leptos::Article().into_any(),
        "understanding-rust-ownership" => understanding_rust_ownership::Article().into_any(),
        _ => view! { <div></div> }.into_any(),
    }
}

pub fn article_exists(slug: &str) -> bool {
    matches!(
        slug,
        "getting-started-with-rust"
            | "building-web-apps-with-leptos"
            | "understanding-rust-ownership"
    )
}
