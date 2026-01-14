use leptos::prelude::*;

#[component]
pub fn Article() -> impl IntoView {
    view! {
        <article class="prose dark:prose-invert max-w-4xl mx-auto px-4 py-12">
            <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
                "Understanding Rust Ownership"
            </h1>
            <p class="text-gray-600 dark:text-gray-400 mb-8">
                "Published on March 10, 2024"
            </p>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "The Ownership System"
                </h2>
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                    "Rust's ownership system is what sets it apart from other programming languages.
                    It enables memory safety without needing a garbage collector, making Rust both
                    safe and performant."
                </p>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Three Rules of Ownership"
                </h2>
                <ol class="list-decimal list-inside space-y-2 text-gray-700 dark:text-gray-300">
                    <li>"Each value in Rust has an owner"</li>
                    <li>"There can only be one owner at a time"</li>
                    <li>"When the owner goes out of scope, the value will be dropped"</li>
                </ol>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Borrowing and References"
                </h2>
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                    "Instead of transferring ownership, Rust allows you to borrow values through references.
                    This enables multiple parts of your code to read data without taking ownership."
                </p>
                <p class="text-gray-700 dark:text-gray-300">
                    "Rust enforces strict rules about borrowing: you can have either one mutable reference
                    or any number of immutable references, but not both at the same time."
                </p>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Practical Benefits"
                </h2>
                <ul class="list-disc list-inside space-y-2 text-gray-700 dark:text-gray-300">
                    <li>"No dangling pointers"</li>
                    <li>"No data races"</li>
                    <li>"Memory leaks are much harder to create"</li>
                    <li>"Compiler catches many bugs before runtime"</li>
                </ul>
            </section>

            <section class="mb-8">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
                    "Conclusion"
                </h2>
                <p class="text-gray-700 dark:text-gray-300">
                    "While the ownership system can be challenging for newcomers, it's the key to Rust's
                    promise of memory safety and concurrency without compromise. Once you understand these
                    concepts, they become second nature and help you write more reliable code."
                </p>
            </section>
        </article>
    }
}
