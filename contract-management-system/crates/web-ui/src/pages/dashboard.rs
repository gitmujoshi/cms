use leptos::*;
use leptos_router::*;
use crate::state::{AuthContext, Role};
use crate::components::stats::StatsCard;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth_context = use_context::<RwSignal<AuthContext>>().expect("Auth context not found");
    let user = move || auth_context.get().user().expect("User not found");
    let role = move || auth_context.get().role().expect("Role not found");
    
    let (stats, set_stats) = create_signal(None::<DashboardStats>);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(None::<String>);
    
    // Load dashboard stats
    spawn_local(async move {
        match load_dashboard_stats().await {
            Ok(data) => {
                set_stats.set(Some(data));
                set_loading.set(false);
            }
            Err(e) => {
                set_error.set(Some(e.to_string()));
                set_loading.set(false);
            }
        }
    });
    
    view! {
        <div class="py-6">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
                <h1 class="text-2xl font-semibold text-gray-900">
                    "Dashboard"
                </h1>
            </div>
            
            <div class="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
                // Welcome message
                <div class="py-4">
                    <div class="bg-white shadow sm:rounded-lg">
                        <div class="px-4 py-5 sm:p-6">
                            <h3 class="text-lg leading-6 font-medium text-gray-900">
                                {"Welcome back, "} {move || user().organization_name}
                            </h3>
                            <div class="mt-2 max-w-xl text-sm text-gray-500">
                                <p>
                                    {move || match role() {
                                        Role::TrainingDataProvider => "Manage your training data and contracts.",
                                        Role::CleanRoomProvider => "Monitor clean room operations and contracts.",
                                        Role::DataConsumer => "Browse available training data and manage contracts.",
                                        Role::SystemAdministrator => "System overview and administration.",
                                    }}
                                </p>
                            </div>
                        </div>
                    </div>
                </div>

                // Stats
                <Show
                    when=move || !loading()
                    fallback=|| view! {
                        <div class="animate-pulse">
                            <div class="h-32 bg-gray-200 rounded"></div>
                        </div>
                    }
                >
                    <Show
                        when=move || error().is_none()
                        fallback=|| view! {
                            <div class="rounded-md bg-red-50 p-4">
                                <div class="flex">
                                    <div class="ml-3">
                                        <h3 class="text-sm font-medium text-red-800">
                                            "Error loading dashboard"
                                        </h3>
                                        <div class="mt-2 text-sm text-red-700">
                                            <p>{move || error().unwrap_or_default()}</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    >
                        <div class="mt-8">
                            <div class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
                                {move || match role() {
                                    Role::TrainingDataProvider => view! {
                                        <>
                                            <StatsCard
                                                title="Active Contracts"
                                                value=stats.get().map(|s| s.active_contracts.to_string())
                                                change=Some(10)
                                                trend="increase"
                                            />
                                            <StatsCard
                                                title="Data Sets"
                                                value=stats.get().map(|s| s.datasets.to_string())
                                                change=Some(5)
                                                trend="increase"
                                            />
                                            <StatsCard
                                                title="Revenue"
                                                value=stats.get().map(|s| format!("${:.2}", s.revenue))
                                                change=Some(12)
                                                trend="increase"
                                            />
                                        </>
                                    },
                                    Role::CleanRoomProvider => view! {
                                        <>
                                            <StatsCard
                                                title="Active Clean Rooms"
                                                value=stats.get().map(|s| s.active_clean_rooms.to_string())
                                                change=Some(8)
                                                trend="increase"
                                            />
                                            <StatsCard
                                                title="Total Compute Hours"
                                                value=stats.get().map(|s| format!("{:.1}h", s.compute_hours))
                                                change=Some(15)
                                                trend="increase"
                                            />
                                            <StatsCard
                                                title="Revenue"
                                                value=stats.get().map(|s| format!("${:.2}", s.revenue))
                                                change=Some(20)
                                                trend="increase"
                                            />
                                        </>
                                    },
                                    Role::DataConsumer => view! {
                                        <>
                                            <StatsCard
                                                title="Active Contracts"
                                                value=stats.get().map(|s| s.active_contracts.to_string())
                                                change=Some(5)
                                                trend="increase"
                                            />
                                            <StatsCard
                                                title="Models in Training"
                                                value=stats.get().map(|s| s.models_in_training.to_string())
                                                change=Some(25)
                                                trend="increase"
                                            />
                                            <StatsCard
                                                title="Completed Models"
                                                value=stats.get().map(|s| s.completed_models.to_string())
                                                change=Some(8)
                                                trend="increase"
                                            />
                                        </>
                                    },
                                    Role::SystemAdministrator => view! {
                                        <>
                                            <StatsCard
                                                title="Total Users"
                                                value=stats.get().map(|s| s.total_users.to_string())
                                                change=Some(15)
                                                trend="increase"
                                            />
                                            <StatsCard
                                                title="Active Contracts"
                                                value=stats.get().map(|s| s.active_contracts.to_string())
                                                change=Some(12)
                                                trend="increase"
                                            />
                                            <StatsCard
                                                title="System Health"
                                                value=stats.get().map(|s| format!("{}%", s.system_health))
                                                change=None
                                                trend="neutral"
                                            />
                                        </>
                                    },
                                }}
                            </div>
                        </div>
                    </Show>
                </Show>

                // Quick Actions
                <div class="mt-8">
                    <div class="bg-white shadow sm:rounded-lg">
                        <div class="px-4 py-5 sm:p-6">
                            <h3 class="text-lg leading-6 font-medium text-gray-900">
                                "Quick Actions"
                            </h3>
                            <div class="mt-5 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                                {move || match role() {
                                    Role::TrainingDataProvider => view! {
                                        <>
                                            <QuickAction
                                                title="Upload Dataset"
                                                href="/datasets/upload"
                                                icon="upload"
                                            />
                                            <QuickAction
                                                title="View Contracts"
                                                href="/contracts"
                                                icon="document"
                                            />
                                            <QuickAction
                                                title="Analytics"
                                                href="/analytics"
                                                icon="chart"
                                            />
                                        </>
                                    },
                                    Role::CleanRoomProvider => view! {
                                        <>
                                            <QuickAction
                                                title="Clean Room Status"
                                                href="/clean-rooms"
                                                icon="server"
                                            />
                                            <QuickAction
                                                title="Resource Usage"
                                                href="/resources"
                                                icon="chart"
                                            />
                                            <QuickAction
                                                title="Security Logs"
                                                href="/security"
                                                icon="shield"
                                            />
                                        </>
                                    },
                                    Role::DataConsumer => view! {
                                        <>
                                            <QuickAction
                                                title="Browse Datasets"
                                                href="/datasets"
                                                icon="search"
                                            />
                                            <QuickAction
                                                title="Model Training"
                                                href="/training"
                                                icon="cpu"
                                            />
                                            <QuickAction
                                                title="View Models"
                                                href="/models"
                                                icon="cube"
                                            />
                                        </>
                                    },
                                    Role::SystemAdministrator => view! {
                                        <>
                                            <QuickAction
                                                title="User Management"
                                                href="/admin/users"
                                                icon="users"
                                            />
                                            <QuickAction
                                                title="System Status"
                                                href="/admin/status"
                                                icon="server"
                                            />
                                            <QuickAction
                                                title="Audit Logs"
                                                href="/admin/audit"
                                                icon="shield"
                                            />
                                        </>
                                    },
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(Debug, Clone)]
struct DashboardStats {
    active_contracts: u32,
    datasets: u32,
    revenue: f64,
    active_clean_rooms: u32,
    compute_hours: f64,
    models_in_training: u32,
    completed_models: u32,
    total_users: u32,
    system_health: u32,
}

async fn load_dashboard_stats() -> Result<DashboardStats, String> {
    // Call API to load stats
    let response = api::get("/api/dashboard/stats")
        .await
        .map_err(|e| e.to_string())?;
        
    response.json()
        .await
        .map_err(|e| e.to_string())
}

#[component]
fn QuickAction(
    title: &'static str,
    href: &'static str,
    icon: &'static str,
) -> impl IntoView {
    view! {
        <A
            href=href
            class="relative rounded-lg border border-gray-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3 hover:border-gray-400 focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-blue-500"
        >
            <div class="flex-shrink-0">
                <span class="h-10 w-10 rounded-full bg-blue-100 flex items-center justify-center">
                    {icon_component(icon)}
                </span>
            </div>
            <div class="flex-1 min-w-0">
                <span class="absolute inset-0" aria-hidden="true"></span>
                <p class="text-sm font-medium text-gray-900">
                    {title}
                </p>
            </div>
        </A>
    }
}

fn icon_component(icon: &str) -> View {
    match icon {
        "upload" => view! {
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/>
            </svg>
        }.into_view(),
        "document" => view! {
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
            </svg>
        }.into_view(),
        "chart" => view! {
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
            </svg>
        }.into_view(),
        "server" => view! {
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"/>
            </svg>
        }.into_view(),
        "shield" => view! {
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
            </svg>
        }.into_view(),
        "search" => view! {
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
            </svg>
        }.into_view(),
        "cpu" => view! {
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"/>
            </svg>
        }.into_view(),
        "cube" => view! {
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/>
            </svg>
        }.into_view(),
        "users" => view! {
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"/>
            </svg>
        }.into_view(),
        _ => view! {}.into_view(),
    }
} 