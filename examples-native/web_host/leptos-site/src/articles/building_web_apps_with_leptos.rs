use leptos::prelude::*;

#[component]
pub fn Article() -> impl IntoView {
    view! {
        <article class="prose dark:prose-invert max-w-4xl mx-auto px-4 py-12">
            <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
                "Building Web Apps with Leptos"
            </h1>
            <p class="text-gray-600 dark:text-gray-400 mb-8">
                "Published on February 20, 2024"
            </p>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "What is Leptos?"
                </h2>
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                    "Leptos is a full-stack, reactive web framework for Rust. It allows you to build
                    fast, type-safe web applications that compile to WebAssembly, providing near-native
                    performance in the browser."
                </p>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Key Features"
                </h2>
                <ul class="list-disc list-inside space-y-2 text-gray-700 dark:text-gray-300">
                    <li>"Fine-grained reactivity for optimal performance"</li>
                    <li>"Isomorphic rendering (SSR and CSR)"</li>
                    <li>"Type-safe APIs throughout"</li>
                    <li>"Excellent developer experience with hot module reloading"</li>
                </ul>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Creating Your First Component"
                </h2>
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                    "Leptos components are just Rust functions annotated with the #[component] macro.
                    They return views using the view! macro, which provides a JSX-like syntax for
                    building your UI."
                </p>
                <p class="text-gray-700 dark:text-gray-300">
                    "The component model is intuitive and leverages Rust's strong type system to
                    catch errors at compile time rather than runtime."
                </p>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Why Leptos?"
                </h2>
                <p class="text-gray-700 dark:text-gray-300">
                    "Leptos combines the performance benefits of Rust and WebAssembly with a modern,
                    reactive programming model. It's an excellent choice for developers who want to
                    build fast, reliable web applications without sacrificing developer experience."
                </p>
            </section>
        </article>
    }
}
