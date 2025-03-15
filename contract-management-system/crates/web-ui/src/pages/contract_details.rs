use leptos::*;
use uuid::Uuid;
use crate::components::contract_details::{ContractDetails, ContractDetails as ContractDetailsData};
use crate::state::auth::AuthContext;

#[component]
pub fn ContractDetailsPage(cx: Scope) -> impl IntoView {
    let auth = use_context::<AuthContext>(cx).expect("Auth context not found");
    let params = use_params_map(cx);
    let contract_id = params.get("id").and_then(|id| Uuid::parse_str(id).ok());
    
    let (contract, set_contract) = create_signal(cx, None::<ContractDetailsData>);
    let (error, set_error) = create_signal(cx, None::<String>);

    let fetch_contract = create_action(cx, move |id: &Uuid| {
        let id = *id;
        async move {
            // TODO: Implement API call to fetch contract details
            match reqwest::Client::new()
                .get(&format!("http://localhost:8000/api/contracts/{}", id))
                .send()
                .await
            {
                Ok(response) => {
                    match response.json::<ContractDetailsData>().await {
                        Ok(data) => {
                            set_contract.set(Some(data));
                            set_error.set(None);
                        }
                        Err(e) => {
                            set_error.set(Some(format!("Failed to parse contract data: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to fetch contract: {}", e)));
                }
            }
        }
    });

    let sign_contract = create_action(cx, move |_: &()| {
        let contract_id = contract.get().map(|c| c.id).unwrap();
        async move {
            // TODO: Implement API call to sign contract
            match reqwest::Client::new()
                .post(&format!("http://localhost:8000/api/contracts/{}/sign", contract_id))
                .send()
                .await
            {
                Ok(_) => {
                    fetch_contract.dispatch(contract_id);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to sign contract: {}", e)));
                }
            }
        }
    });

    let terminate_contract = create_action(cx, move |_: &()| {
        let contract_id = contract.get().map(|c| c.id).unwrap();
        async move {
            // TODO: Implement API call to terminate contract
            match reqwest::Client::new()
                .post(&format!("http://localhost:8000/api/contracts/{}/terminate", contract_id))
                .send()
                .await
            {
                Ok(_) => {
                    fetch_contract.dispatch(contract_id);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to terminate contract: {}", e)));
                }
            }
        }
    });

    let suspend_contract = create_action(cx, move |_: &()| {
        let contract_id = contract.get().map(|c| c.id).unwrap();
        async move {
            // TODO: Implement API call to suspend contract
            match reqwest::Client::new()
                .post(&format!("http://localhost:8000/api/contracts/{}/suspend", contract_id))
                .send()
                .await
            {
                Ok(_) => {
                    fetch_contract.dispatch(contract_id);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to suspend contract: {}", e)));
                }
            }
        }
    });

    let activate_contract = create_action(cx, move |_: &()| {
        let contract_id = contract.get().map(|c| c.id).unwrap();
        async move {
            // TODO: Implement API call to activate contract
            match reqwest::Client::new()
                .post(&format!("http://localhost:8000/api/contracts/{}/activate", contract_id))
                .send()
                .await
            {
                Ok(_) => {
                    fetch_contract.dispatch(contract_id);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to activate contract: {}", e)));
                }
            }
        }
    });

    // Initial load
    create_effect(cx, move |_| {
        if let Some(id) = contract_id {
            fetch_contract.dispatch(id);
        } else {
            set_error.set(Some("Invalid contract ID".to_string()));
        }
    });

    view! { cx,
        <div class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
            <div class="px-4 py-6 sm:px-0">
                <div class="flex items-center mb-6">
                    <button
                        class="mr-4 text-indigo-600 hover:text-indigo-900"
                        on:click=move |_| {
                            let navigate = use_navigate(cx);
                            navigate("/contracts", Default::default());
                        }
                    >
                        "← Back to Contracts"
                    </button>
                    <h1 class="text-2xl font-semibold text-gray-900">
                        "Contract Details"
                    </h1>
                </div>

                {move || error.get().map(|err| view! { cx,
                    <div class="rounded-md bg-red-50 p-4 mb-6">
                        <div class="flex">
                            <div class="flex-shrink-0">
                                <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
                                </svg>
                            </div>
                            <div class="ml-3">
                                <h3 class="text-sm font-medium text-red-800">
                                    {err}
                                </h3>
                            </div>
                        </div>
                    </div>
                })}

                {move || contract.get().map(|contract_data| view! { cx,
                    <ContractDetails
                        contract=create_signal(cx, contract_data).0
                        on_sign=sign_contract
                        on_terminate=terminate_contract
                        on_suspend=suspend_contract
                        on_activate=activate_contract
                    />
                })}
            </div>
        </div>
    }
} 