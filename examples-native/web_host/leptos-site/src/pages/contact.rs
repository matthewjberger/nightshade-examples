use leptos::prelude::*;
use leptos_router::components::*;

#[component]
pub fn Contact() -> impl IntoView {
    view! {
        <section class="min-h-screen py-20 bg-white dark:bg-gray-900 pt-24">
            <div class="max-w-6xl mx-auto px-4">
                <h2 class="text-4xl font-bold text-center mb-12 text-gray-900 dark:text-white">
                    "Get in Touch"
                </h2>
                <div class="max-w-3xl mx-auto">
                    <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-8 shadow-lg">
                        <p class="text-gray-700 dark:text-gray-300 leading-relaxed text-lg text-center mb-8">
                            "This is a demonstration static site. For real contact information, please visit the actual deployment."
                        </p>
                        <div class="flex gap-4 justify-center flex-wrap">
                            <a
                                href="https://github.com"
                                target="_blank"
                                class="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors inline-flex items-center gap-2 whitespace-nowrap"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" style="width: 14px; height: 14px;" viewBox="0 0 24 24" fill="currentColor">
                                    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                                </svg>
                                "GitHub"
                            </a>
                        </div>
                    </div>
                    <div class="flex justify-center mt-8">
                        <A
                            href="/"
                            attr:class="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors inline-flex items-center gap-2"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" style="width: 14px; height: 14px;" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd"/>
                            </svg>
                            "Back to Home"
                        </A>
                    </div>
                </div>
            </div>
        </section>
    }
}
