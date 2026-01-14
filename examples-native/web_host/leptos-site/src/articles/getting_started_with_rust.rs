use leptos::prelude::*;

#[component]
pub fn Article() -> impl IntoView {
    view! {
        <article class="prose dark:prose-invert max-w-4xl mx-auto px-4 py-12">
            <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
                "Getting Started with Rust"
            </h1>
            <p class="text-gray-600 dark:text-gray-400 mb-8">
                "Published on January 15, 2024"
            </p>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Introduction"
                </h2>
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                    "Rust is a systems programming language that runs blazingly fast, prevents segfaults,
                    and guarantees thread safety. It's been voted the most loved programming language
                    in the Stack Overflow Developer Survey for several years running."
                </p>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Why Choose Rust?"
                </h2>
                <ul class="list-disc list-inside space-y-2 text-gray-700 dark:text-gray-300">
                    <li>"Memory safety without garbage collection"</li>
                    <li>"Fearless concurrency"</li>
                    <li>"Zero-cost abstractions"</li>
                    <li>"Excellent tooling and package management"</li>
                </ul>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Getting Started"
                </h2>
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                    "To get started with Rust, you'll need to install rustup, the Rust toolchain installer.
                    Visit rust-lang.org and follow the installation instructions for your platform."
                </p>
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                    "Once installed, you can create your first Rust project using Cargo, Rust's build tool
                    and package manager."
                </p>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Conclusion"
                </h2>
                <p class="text-gray-700 dark:text-gray-300">
                    "Rust offers a unique combination of performance, safety, and productivity.
                    Whether you're building system tools, web applications, or embedded systems,
                    Rust provides the tools you need to write reliable and efficient code."
                </p>
            </section>
        </article>
    }
}
