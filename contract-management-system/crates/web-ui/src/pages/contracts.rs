use leptos::*;
use uuid::Uuid;
use crate::components::{
    contract_form::{ContractForm, ContractFormData},
    contract_list::{ContractList, ContractSummary, ContractFilter},
};
use crate::state::auth::AuthContext;

#[component]
pub fn ContractsPage(cx: Scope) -> impl IntoView {
    let auth = use_context::<AuthContext>(cx).expect("Auth context not found");
    
    let (contracts, set_contracts) = create_signal(cx, vec![]);
    let (total_count, set_total_count) = create_signal(cx, 0);
    let (show_form, set_show_form) = create_signal(cx, false);

    let fetch_contracts = create_action(cx, move |filter: &ContractFilter| {
        let filter = filter.clone();
        async move {
            // TODO: Implement API call to fetch contracts
            let response = reqwest::Client::new()
                .get("http://localhost:8000/api/contracts")
                .query(&filter)
                .send()
                .await
                .unwrap()
                .json::<(Vec<ContractSummary>, usize)>()
                .await
                .unwrap();
            
            set_contracts.set(response.0);
            set_total_count.set(response.1);
        }
    });

    let create_contract = create_action(cx, move |form_data: &ContractFormData| {
        let form_data = form_data.clone();
        async move {
            // TODO: Implement API call to create contract
            let response = reqwest::Client::new()
                .post("http://localhost:8000/api/contracts")
                .json(&form_data)
                .send()
                .await
                .unwrap()
                .json::<Uuid>()
                .await
                .unwrap();
            
            set_show_form.set(false);
            fetch_contracts.dispatch(ContractFilter {
                status: None,
                search_term: String::new(),
                page: 1,
                per_page: 10,
            });
        }
    });

    let handle_filter_change = move |filter: ContractFilter| {
        fetch_contracts.dispatch(filter);
    };

    let handle_contract_click = move |id: Uuid| {
        // Navigate to contract details page
        let navigate = use_navigate(cx);
        navigate(&format!("/contracts/{}", id), Default::default());
    };

    // Initial load
    create_effect(cx, move |_| {
        fetch_contracts.dispatch(ContractFilter {
            status: None,
            search_term: String::new(),
            page: 1,
            per_page: 10,
        });
    });

    view! { cx,
        <div class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
            <div class="px-4 py-6 sm:px-0">
                <div class="flex justify-between items-center mb-6">
                    <h1 class="text-2xl font-semibold text-gray-900">"Contracts"</h1>
                    <button
                        class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                        on:click=move |_| set_show_form.set(true)
                    >
                        "New Contract"
                    </button>
                </div>

                {move || if show_form.get() {
                    view! { cx,
                        <div class="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4">
                            <div class="bg-white rounded-lg p-6 max-w-2xl w-full">
                                <div class="flex justify-between items-center mb-4">
                                    <h2 class="text-xl font-semibold">"Create New Contract"</h2>
                                    <button
                                        class="text-gray-400 hover:text-gray-500"
                                        on:click=move |_| set_show_form.set(false)
                                    >
                                        <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                        </svg>
                                    </button>
                                </div>
                                <ContractForm
                                    on_submit=create_contract
                                />
                            </div>
                        </div>
                    }
                } else {
                    view! { cx, <span></span> }
                }}

                <ContractList
                    contracts=contracts
                    total_count=total_count
                    on_filter_change=handle_filter_change
                    on_contract_click=handle_contract_click
                />
            </div>
        </div>
    }
} 