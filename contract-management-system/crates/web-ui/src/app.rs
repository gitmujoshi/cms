use leptos::*;
use leptos_router::*;

use crate::pages::{
    contracts::ContractsPage,
    contract_details::ContractDetailsPage,
    dashboard::DashboardPage,
    login::LoginPage,
    not_found::NotFoundPage,
};
use crate::components::navbar::Navbar;
use crate::state::auth::AuthContext;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let auth = use_context::<AuthContext>(cx).expect("Auth context not found");

    view! { cx,
        <Router>
            <div class="min-h-screen bg-gray-100">
                {move || if auth.is_authenticated() {
                    view! { cx,
                        <Navbar/>
                        <main>
                            <Routes>
                                <Route
                                    path="/"
                                    view=|cx| view! { cx, <DashboardPage/> }
                                />
                                <Route
                                    path="/contracts"
                                    view=|cx| view! { cx, <ContractsPage/> }
                                />
                                <Route
                                    path="/contracts/:id"
                                    view=|cx| view! { cx, <ContractDetailsPage/> }
                                />
                                <Route
                                    path="/*"
                                    view=|cx| view! { cx, <NotFoundPage/> }
                                />
                            </Routes>
                        </main>
                    }
                } else {
                    view! { cx,
                        <Routes>
                            <Route
                                path="/*"
                                view=|cx| view! { cx, <LoginPage/> }
                            />
                        </Routes>
                    }
                }}
            </div>
        </Router>
    }
} 