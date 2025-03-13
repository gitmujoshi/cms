use leptos::*;
use leptos_router::*;

use crate::layouts::MainLayout;
use crate::pages::{RegistrationPage, ContractsPage, LoginPage, DashboardPage};
use crate::state::AuthContext;

#[component]
pub fn App() -> impl IntoView {
    // Set up authentication state
    let auth_context = create_rw_signal(AuthContext::default());
    
    provide_context(auth_context);

    view! {
        <Router>
            <MainLayout>
                <Routes>
                    <Route
                        path="/"
                        view=|| view! { <DashboardPage /> }
                    />
                    <Route
                        path="/login"
                        view=|| view! { <LoginPage /> }
                    />
                    <Route
                        path="/register"
                        view=|| view! { <RegistrationPage /> }
                    />
                    <Route
                        path="/contracts"
                        view=|| view! { <ContractsPage /> }
                    />
                    <Route
                        path="/contracts/:id"
                        view=move |cx| {
                            let params = use_params_map(cx);
                            let id = params.get().get("id").cloned().unwrap_or_default();
                            view! { <ContractDetailsPage contract_id=id /> }
                        }
                    />
                    <Route
                        path="/*any"
                        view=|| view! { <NotFoundPage /> }
                    />
                </Routes>
            </MainLayout>
        </Router>
    }
}

#[component]
fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
            <div class="sm:mx-auto sm:w-full sm:max-w-md">
                <h2 class="text-center text-3xl font-extrabold text-gray-900">
                    "404 - Page Not Found"
                </h2>
                <p class="mt-2 text-center text-sm text-gray-600">
                    "The page you're looking for doesn't exist."
                </p>
                <div class="mt-4 text-center">
                    <A
                        href="/"
                        class="text-blue-600 hover:text-blue-500"
                    >
                        "Return to Dashboard"
                    </A>
                </div>
            </div>
        </div>
    }
} 