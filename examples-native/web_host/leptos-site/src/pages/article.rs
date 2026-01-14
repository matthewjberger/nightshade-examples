use crate::articles::{article_exists, render_article};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

#[component]
pub fn Article() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();

    let exists = move || article_exists(&slug());

    view! {
        <div class="min-h-screen bg-white dark:bg-gray-900 pt-20">
            <div class="max-w-4xl mx-auto px-4 py-8">
                <A
                    href="/articles"
                    attr:class="inline-flex items-center text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 mb-8"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
                    </svg>
                    "Back to Articles"
                </A>

                <Show
                    when=exists
                    fallback=move || view! {
                        <div class="text-center py-20">
                            <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
                                "Article Not Found"
                            </h1>
                            <p class="text-xl text-gray-700 dark:text-gray-300 mb-8">
                                "The article you're looking for doesn't exist."
                            </p>
                            <A
                                href="/articles"
                                attr:class="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors inline-block"
                            >
                                "View All Articles"
                            </A>
                        </div>
                    }
                >
                    {move || render_article(&slug())}
                </Show>
            </div>
        </div>
    }
}
