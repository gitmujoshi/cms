use leptos::*;
use leptos_router::*;
use crate::state::AuthContext;

#[component]
pub fn Navbar() -> impl IntoView {
    let auth_context = use_context::<RwSignal<AuthContext>>().expect("Auth context not found");
    let is_authenticated = move || auth_context.get().is_authenticated();
    let user_role = move || auth_context.get().role();
    
    let (is_mobile_menu_open, set_mobile_menu_open) = create_signal(false);
    
    let toggle_mobile_menu = move |_| {
        set_mobile_menu_open.update(|open| *open = !*open);
    };
    
    let handle_logout = move |_| {
        spawn_local(async move {
            if let Ok(_) = auth_context.get().logout().await {
                // Redirect to login page
                let navigate = use_navigate();
                navigate("/login", Default::default());
            }
        });
    };
    
    view! {
        <nav class="bg-white shadow">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between h-16">
                    <div class="flex">
                        // Logo
                        <div class="flex-shrink-0 flex items-center">
                            <A href="/" class="text-xl font-bold text-blue-600">
                                "DCMS"
                            </A>
                        </div>

                        // Desktop navigation
                        <div class="hidden sm:ml-6 sm:flex sm:space-x-8">
                            <Show
                                when=is_authenticated
                                fallback=|| view! {
                                    <A
                                        href="/login"
                                        class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium"
                                    >
                                        "Login"
                                    </A>
                                    <A
                                        href="/register"
                                        class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium"
                                    >
                                        "Register"
                                    </A>
                                }
                            >
                                <A
                                    href="/dashboard"
                                    class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium"
                                >
                                    "Dashboard"
                                </A>
                                <A
                                    href="/contracts"
                                    class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium"
                                >
                                    "Contracts"
                                </A>
                                {move || match user_role() {
                                    Role::SystemAdministrator => view! {
                                        <A
                                            href="/admin"
                                            class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium"
                                        >
                                            "Admin"
                                        </A>
                                    }.into_view(),
                                    _ => view! {}.into_view()
                                }}
                            </Show>
                        </div>
                    </div>

                    // Mobile menu button
                    <div class="flex items-center sm:hidden">
                        <button
                            type="button"
                            class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-gray-500 hover:bg-gray-100"
                            aria-controls="mobile-menu"
                            aria-expanded="false"
                            on:click=toggle_mobile_menu
                        >
                            <span class="sr-only">"Open main menu"</span>
                            // Icon when menu is closed
                            <svg
                                class="block h-6 w-6"
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                aria-hidden="true"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M4 6h16M4 12h16M4 18h16"
                                />
                            </svg>
                        </button>
                    </div>

                    // User menu
                    <Show
                        when=is_authenticated
                        fallback=|| view! {}
                    >
                        <div class="hidden sm:ml-6 sm:flex sm:items-center">
                            <div class="ml-3 relative">
                                <button
                                    type="button"
                                    class="bg-white rounded-full flex text-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                                    on:click=handle_logout
                                >
                                    "Logout"
                                </button>
                            </div>
                        </div>
                    </Show>
                </div>
            </div>

            // Mobile menu
            <Show
                when=move || is_mobile_menu_open()
                fallback=|| view! {}
            >
                <div class="sm:hidden" id="mobile-menu">
                    <div class="pt-2 pb-3 space-y-1">
                        <Show
                            when=is_authenticated
                            fallback=|| view! {
                                <A
                                    href="/login"
                                    class="block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-50"
                                >
                                    "Login"
                                </A>
                                <A
                                    href="/register"
                                    class="block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-50"
                                >
                                    "Register"
                                </A>
                            }
                        >
                            <A
                                href="/dashboard"
                                class="block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-50"
                            >
                                "Dashboard"
                            </A>
                            <A
                                href="/contracts"
                                class="block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-50"
                            >
                                "Contracts"
                            </A>
                            {move || match user_role() {
                                Role::SystemAdministrator => view! {
                                    <A
                                        href="/admin"
                                        class="block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-50"
                                    >
                                        "Admin"
                                    </A>
                                }.into_view(),
                                _ => view! {}.into_view()
                            }}
                            <button
                                type="button"
                                class="block w-full text-left px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-50"
                                on:click=handle_logout
                            >
                                "Logout"
                            </button>
                        </Show>
                    </div>
                </div>
            </Show>
        </nav>
    }
} 