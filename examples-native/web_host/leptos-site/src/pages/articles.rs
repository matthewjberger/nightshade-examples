use crate::articles::get_all_articles;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Articles() -> impl IntoView {
    let articles = get_all_articles();

    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900 pt-20">
            <div class="max-w-4xl mx-auto px-4 py-12">
                <A
                    href="/"
                    attr:class="inline-flex items-center text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 mb-8"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
                    </svg>
                    "Back to Home"
                </A>

                <h1 class="text-4xl md:text-5xl font-bold text-gray-900 dark:text-white mb-8">
                    "Articles"
                </h1>
                <p class="text-xl text-gray-700 dark:text-gray-300 mb-12">
                    "Thoughts and insights on software development, Rust, and web technologies."
                </p>

                <div class="space-y-6">
                    {articles.into_iter().map(|article| {
                        view! {
                            <A
                                href=format!("/articles/{}", article.slug)
                                attr:class="block p-6 bg-white dark:bg-gray-800 rounded-lg shadow-md hover:shadow-lg transition-shadow border border-gray-200 dark:border-gray-700"
                            >
                                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                                    {article.title}
                                </h2>
                                <p class="text-sm text-gray-600 dark:text-gray-400 mb-3">
                                    {article.date}
                                </p>
                                <p class="text-gray-700 dark:text-gray-300">
                                    {article.description}
                                </p>
                                <div class="mt-4 text-blue-600 dark:text-blue-400 font-medium inline-flex items-center">
                                    "Read more "
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 ml-1" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M10.293 3.293a1 1 0 011.414 0l6 6a1 1 0 010 1.414l-6 6a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-4.293-4.293a1 1 0 010-1.414z" clip-rule="evenodd" />
                                    </svg>
                                </div>
                            </A>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
