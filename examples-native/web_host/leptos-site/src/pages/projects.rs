use leptos::prelude::*;

#[component]
pub fn Projects() -> impl IntoView {
    let projects = vec![
        (
            "Example Project 1",
            "A showcase of Rust and WebAssembly capabilities",
            "https://github.com",
        ),
        (
            "Example Project 2",
            "Building reactive UIs with Leptos framework",
            "https://github.com",
        ),
        (
            "Example Project 3",
            "Static site generation with Trunk",
            "https://github.com",
        ),
    ];

    view! {
        <section id="projects" class="py-20 bg-gray-50 dark:bg-gray-800">
            <div class="max-w-6xl mx-auto px-4">
                <h2 class="text-4xl font-bold text-center mb-12 text-gray-900 dark:text-white">
                    "Projects"
                </h2>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                    {projects.into_iter().map(|(title, description, link)| {
                        view! {
                            <a
                                href=link
                                target="_blank"
                                class="bg-white dark:bg-gray-900 rounded-lg overflow-hidden shadow-lg hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-1 block"
                            >
                                <div class="aspect-video bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                                    </svg>
                                </div>
                                <div class="p-6">
                                    <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-3">{title}</h3>
                                    <p class="text-gray-700 dark:text-gray-300">{description}</p>
                                </div>
                            </a>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}
