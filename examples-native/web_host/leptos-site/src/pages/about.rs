use leptos::prelude::*;

#[component]
pub fn About() -> impl IntoView {
    view! {
        <section id="about" class="py-20 bg-white dark:bg-gray-900">
            <div class="max-w-6xl mx-auto px-4">
                <h2 class="text-4xl font-bold text-center mb-12 text-gray-900 dark:text-white">
                    "About Me"
                </h2>
                <div class="max-w-3xl mx-auto space-y-8">
                    <p class="text-gray-700 dark:text-gray-300 leading-relaxed text-lg text-center">
                        "A passionate software engineer building modern web applications with Rust 🦀"
                    </p>
                    <p class="text-gray-700 dark:text-gray-300 leading-relaxed text-lg text-center">
                        "Experienced in full-stack development with a focus on performance, type safety, and developer experience. Specializing in Rust, WebAssembly, and reactive web frameworks."
                    </p>
                </div>
            </div>
        </section>
    }
}
