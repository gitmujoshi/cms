use leptos::*;

#[component]
pub fn StatsCard(
    title: &'static str,
    value: Option<String>,
    change: Option<i32>,
    trend: &'static str,
) -> impl IntoView {
    view! {
        <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="p-5">
                <div class="flex items-center">
                    <div class="flex-1">
                        <p class="text-sm font-medium text-gray-500 truncate">
                            {title}
                        </p>
                        <p class="mt-1 text-3xl font-semibold text-gray-900">
                            {move || value.clone().unwrap_or_else(|| "-".to_string())}
                        </p>
                    </div>
                    <div class="flex items-center">
                        {move || match (change, trend) {
                            (Some(change), "increase") => view! {
                                <span class="flex items-center text-green-600">
                                    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"/>
                                    </svg>
                                    <span class="ml-1 text-sm">{format!("{}%", change)}</span>
                                </span>
                            }.into_view(),
                            (Some(change), "decrease") => view! {
                                <span class="flex items-center text-red-600">
                                    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"/>
                                    </svg>
                                    <span class="ml-1 text-sm">{format!("{}%", change)}</span>
                                </span>
                            }.into_view(),
                            _ => view! {}.into_view(),
                        }}
                    </div>
                </div>
            </div>
            <div class="bg-gray-50 px-5 py-3">
                <div class="text-sm">
                    <a href="#" class="font-medium text-blue-600 hover:text-blue-900">
                        "View details"
                    </a>
                </div>
            </div>
        </div>
    }
} 