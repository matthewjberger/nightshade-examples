use leptos::prelude::*;

#[derive(Clone)]
struct Job {
    title: &'static str,
    company: &'static str,
    period: &'static str,
    achievements: Vec<&'static str>,
}

#[component]
pub fn Experience() -> impl IntoView {
    let jobs = vec![
        Job {
            title: "Senior Software Engineer",
            company: "Your Current Company",
            period: "Jan 2023 - Present",
            achievements: vec![
                "Led development of key features that improved system performance by 40%",
                "Mentored junior developers and established best practices for the team",
                "Architected and implemented scalable microservices architecture",
            ],
        },
        Job {
            title: "Software Engineer",
            company: "Previous Company",
            period: "Jun 2020 - Dec 2022",
            achievements: vec![
                "Developed full-stack web applications using modern frameworks",
                "Improved application response time by 60% through optimization",
                "Collaborated with cross-functional teams to deliver features on schedule",
            ],
        },
        Job {
            title: "Junior Developer",
            company: "First Company",
            period: "Aug 2018 - May 2020",
            achievements: vec![
                "Built and maintained internal tools that increased team productivity",
                "Participated in code reviews and learned industry best practices",
                "Contributed to open-source projects and internal documentation",
            ],
        },
    ];

    let total_jobs = jobs.len();
    let (current_index, set_current_index) = signal(total_jobs - 1);
    let (show_all, set_show_all) = signal(true);
    let jobs_stored = StoredValue::new(jobs);

    view! {
        <section id="experience" class="py-20 bg-white dark:bg-gray-900">
            <div class="max-w-4xl mx-auto px-4">
                <div class="flex justify-between items-center mb-12">
                    <h2 class="text-4xl font-bold text-gray-900 dark:text-white">
                        "Experience"
                    </h2>
                    <button
                        on:click=move |_| set_show_all.update(|v| *v = !*v)
                        class="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
                    >
                        {move || if show_all.get() { "Show Roles" } else { "Show Timeline" }}
                    </button>
                </div>
                <Show
                    when=move || show_all.get()
                    fallback=move || view! {
                        <div class="space-y-8">
                            <div class="flex justify-between items-center mb-8">
                                <button
                                    on:click=move |_| set_current_index.update(|i| *i = i.saturating_sub(1))
                                    class="experience-nav-button px-4 py-2 bg-blue-500 dark:bg-blue-500 text-white rounded-lg hover:bg-blue-600 dark:hover:bg-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled=move || current_index.get() == 0
                                >
                                    "← Older"
                                </button>
                                <span class="text-gray-700 dark:text-gray-300">
                                    {move || format!("{} of {}", current_index.get() + 1, total_jobs)}
                                </span>
                                <button
                                    on:click=move |_| set_current_index.update(|i| *i = (*i + 1).min(total_jobs - 1))
                                    class="experience-nav-button px-4 py-2 bg-blue-500 dark:bg-blue-500 text-white rounded-lg hover:bg-blue-600 dark:hover:bg-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled={move || current_index.get() >= total_jobs - 1}
                                >
                                    "Newer →"
                                </button>
                            </div>
                            <div style="min-height: 500px;">
                                {move || {
                                    let index = current_index.get();
                                    let reversed_index = total_jobs - 1 - index;
                                    jobs_stored.with_value(|jobs| {
                                        let job = &jobs[reversed_index];
                                        view! {
                                        <div class="border-l-4 border-blue-500 dark:border-blue-500 pl-6">
                                            <h3 class="text-2xl font-bold text-gray-900 dark:text-white">{job.title}</h3>
                                            <p class="text-lg text-gray-700 dark:text-gray-300 mb-2">{job.company}</p>
                                            <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">{job.period}</p>
                                            <ul class="space-y-2">
                                                {job.achievements.iter().map(|achievement| {
                                                    view! {
                                                        <li class="flex items-start">
                                                            <span class="text-blue-500 dark:text-blue-400 mr-2">"•"</span>
                                                            <span class="text-gray-700 dark:text-gray-300">{*achievement}</span>
                                                        </li>
                                                    }
                                                }).collect_view()}
                                            </ul>
                                        </div>
                                    }
                                    })
                                }}
                            </div>
                        </div>
                    }
                >
                    <div class="space-y-8">
                        {jobs_stored.with_value(|jobs| {
                            jobs.iter().map(|job| {
                                view! {
                                    <div class="border-l-4 border-blue-500 dark:border-blue-500 pl-6">
                                        <h3 class="text-2xl font-bold text-gray-900 dark:text-white">{job.title}</h3>
                                        <p class="text-lg text-gray-700 dark:text-gray-300 mb-2">{job.company}</p>
                                        <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">{job.period}</p>
                                        <ul class="space-y-2">
                                            {job.achievements.iter().map(|achievement| {
                                                view! {
                                                    <li class="flex items-start">
                                                        <span class="text-blue-500 dark:text-blue-400 mr-2">"•"</span>
                                                        <span class="text-gray-700 dark:text-gray-300">{*achievement}</span>
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ul>
                                    </div>
                                }
                            }).collect_view()
                        })}
                    </div>
                </Show>
            </div>
        </section>
    }
}
