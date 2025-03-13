use leptos::*;
use leptos_router::*;
use crate::state::AuthContext;
use crate::components::Navbar;

#[component]
pub fn MainLayout(children: Children) -> impl IntoView {
    let auth_context = use_context::<RwSignal<AuthContext>>().expect("Auth context not found");
    let is_authenticated = move || auth_context.get().is_authenticated();
    
    view! {
        <div class="min-h-screen bg-gray-100">
            // Navigation
            <Navbar />

            // Main content
            <main class="py-10">
                <div class="max-w-7xl mx-auto sm:px-6 lg:px-8">
                    {children()}
                </div>
            </main>

            // Footer
            <footer class="bg-white">
                <div class="max-w-7xl mx-auto py-12 px-4 sm:px-6 md:flex md:items-center md:justify-between lg:px-8">
                    <div class="flex justify-center space-x-6 md:order-2">
                        <a href="/about" class="text-gray-400 hover:text-gray-500">
                            "About"
                        </a>
                        <a href="/privacy" class="text-gray-400 hover:text-gray-500">
                            "Privacy Policy"
                        </a>
                        <a href="/terms" class="text-gray-400 hover:text-gray-500">
                            "Terms of Service"
                        </a>
                    </div>
                    <div class="mt-8 md:mt-0 md:order-1">
                        <p class="text-center text-base text-gray-400">
                            "© 2024 Digital Contract Management System. All rights reserved."
                        </p>
                    </div>
                </div>
            </footer>
        </div>
    }
} 