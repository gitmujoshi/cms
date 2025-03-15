use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::auth::AuthContext;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractDetails {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: ContractStatus,
    pub contract_type: ContractType,
    pub provider: PartyInfo,
    pub consumer: PartyInfo,
    pub terms: ContractTerms,
    pub signatures: Vec<SignatureInfo>,
    pub created_at: String,
    pub updated_at: String,
    pub valid_from: String,
    pub valid_until: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyInfo {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub organization: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub signer_id: Uuid,
    pub signer_name: String,
    pub signature_type: String,
    pub timestamp: String,
}

#[component]
pub fn ContractDetails(
    cx: Scope,
    #[prop(into)] contract: Signal<ContractDetails>,
    #[prop(into)] on_sign: Callback<()>,
    #[prop(into)] on_terminate: Callback<()>,
    #[prop(into)] on_suspend: Callback<()>,
    #[prop(into)] on_activate: Callback<()>,
) -> impl IntoView {
    let auth = use_context::<AuthContext>(cx).expect("Auth context not found");
    
    let can_sign = move || {
        let contract_data = contract.get();
        contract_data.status == ContractStatus::PendingSignature &&
        (contract_data.provider.id == auth.user_id() || contract_data.consumer.id == auth.user_id())
    };

    let can_terminate = move || {
        let contract_data = contract.get();
        contract_data.status == ContractStatus::Active &&
        (contract_data.provider.id == auth.user_id() || contract_data.consumer.id == auth.user_id())
    };

    let can_suspend = move || {
        let contract_data = contract.get();
        contract_data.status == ContractStatus::Active &&
        (contract_data.provider.id == auth.user_id() || contract_data.consumer.id == auth.user_id())
    };

    let can_activate = move || {
        let contract_data = contract.get();
        contract_data.status == ContractStatus::Suspended &&
        (contract_data.provider.id == auth.user_id() || contract_data.consumer.id == auth.user_id())
    };

    view! { cx,
        <div class="bg-white shadow overflow-hidden sm:rounded-lg">
            <div class="px-4 py-5 sm:px-6">
                <h3 class="text-lg leading-6 font-medium text-gray-900">
                    {move || contract.get().title}
                </h3>
                <p class="mt-1 max-w-2xl text-sm text-gray-500">
                    {move || contract.get().description}
                </p>
            </div>
            
            <div class="border-t border-gray-200 px-4 py-5 sm:px-6">
                <dl class="grid grid-cols-1 gap-x-4 gap-y-8 sm:grid-cols-2">
                    <div class="sm:col-span-1">
                        <dt class="text-sm font-medium text-gray-500">"Status"</dt>
                        <dd class="mt-1 text-sm text-gray-900">
                            {move || {
                                let status_class = match contract.get().status {
                                    ContractStatus::Draft => "bg-gray-100 text-gray-800",
                                    ContractStatus::PendingSignature => "bg-yellow-100 text-yellow-800",
                                    ContractStatus::Active => "bg-green-100 text-green-800",
                                    ContractStatus::Suspended => "bg-red-100 text-red-800",
                                    ContractStatus::Terminated => "bg-red-100 text-red-800",
                                    ContractStatus::Expired => "bg-gray-100 text-gray-800",
                                };
                                view! { cx,
                                    <span class=format!("px-2 inline-flex text-xs leading-5 font-semibold rounded-full {}", status_class)>
                                        {format!("{:?}", contract.get().status)}
                                    </span>
                                }
                            }}
                        </dd>
                    </div>

                    <div class="sm:col-span-1">
                        <dt class="text-sm font-medium text-gray-500">"Contract Type"</dt>
                        <dd class="mt-1 text-sm text-gray-900">
                            {move || format!("{:?}", contract.get().contract_type)}
                        </dd>
                    </div>

                    <div class="sm:col-span-1">
                        <dt class="text-sm font-medium text-gray-500">"Provider"</dt>
                        <dd class="mt-1 text-sm text-gray-900">
                            {move || {
                                let provider = &contract.get().provider;
                                format!("{} ({})", provider.name, provider.organization)
                            }}
                        </dd>
                    </div>

                    <div class="sm:col-span-1">
                        <dt class="text-sm font-medium text-gray-500">"Consumer"</dt>
                        <dd class="mt-1 text-sm text-gray-900">
                            {move || {
                                let consumer = &contract.get().consumer;
                                format!("{} ({})", consumer.name, consumer.organization)
                            }}
                        </dd>
                    </div>

                    <div class="sm:col-span-2">
                        <dt class="text-sm font-medium text-gray-500">"Contract Terms"</dt>
                        <dd class="mt-1 text-sm text-gray-900">
                            <div class="border rounded-md p-4">
                                <h4 class="font-medium">"Data Access Scope"</h4>
                                <ul class="list-disc pl-5 mt-2">
                                    {move || contract.get().terms.data_access_scope.iter().map(|scope| view! { cx,
                                        <li>{scope}</li>
                                    }).collect::<Vec<_>>()}
                                </ul>

                                <h4 class="font-medium mt-4">"Usage Restrictions"</h4>
                                <ul class="list-disc pl-5 mt-2">
                                    {move || contract.get().terms.usage_restrictions.iter().map(|restriction| view! { cx,
                                        <li>{restriction}</li>
                                    }).collect::<Vec<_>>()}
                                </ul>

                                <h4 class="font-medium mt-4">"Security Requirements"</h4>
                                <div class="mt-2">
                                    <p>{"Encryption Required: "}
                                        {move || if contract.get().terms.security_requirements.encryption_required {
                                            "Yes"
                                        } else {
                                            "No"
                                        }}
                                    </p>
                                    <p>{"Minimum Encryption Level: "}
                                        {move || contract.get().terms.security_requirements.min_encryption_level.clone()}
                                    </p>
                                    <p>{"Audit Logging Required: "}
                                        {move || if contract.get().terms.security_requirements.audit_logging_required {
                                            "Yes"
                                        } else {
                                            "No"
                                        }}
                                    </p>
                                </div>
                            </div>
                        </dd>
                    </div>

                    <div class="sm:col-span-2">
                        <dt class="text-sm font-medium text-gray-500">"Signatures"</dt>
                        <dd class="mt-1 text-sm text-gray-900">
                            <div class="border rounded-md divide-y">
                                {move || contract.get().signatures.iter().map(|sig| view! { cx,
                                    <div class="p-4">
                                        <p class="font-medium">{&sig.signer_name}</p>
                                        <p class="text-gray-500">{"Signed on "}{&sig.timestamp}</p>
                                        <p class="text-gray-500">{"Method: "}{&sig.signature_type}</p>
                                    </div>
                                }).collect::<Vec<_>>()}
                            </div>
                        </dd>
                    </div>
                </dl>
            </div>

            <div class="border-t border-gray-200 px-4 py-5 sm:px-6">
                <div class="flex justify-end space-x-3">
                    {move || if can_sign() {
                        view! { cx,
                            <button
                                class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                                on:click=move |_| on_sign.call(())
                            >
                                "Sign Contract"
                            </button>
                        }
                    } else {
                        view! { cx, <span></span> }
                    }}

                    {move || if can_terminate() {
                        view! { cx,
                            <button
                                class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
                                on:click=move |_| on_terminate.call(())
                            >
                                "Terminate Contract"
                            </button>
                        }
                    } else {
                        view! { cx, <span></span> }
                    }}

                    {move || if can_suspend() {
                        view! { cx,
                            <button
                                class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-yellow-600 hover:bg-yellow-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-yellow-500"
                                on:click=move |_| on_suspend.call(())
                            >
                                "Suspend Contract"
                            </button>
                        }
                    } else {
                        view! { cx, <span></span> }
                    }}

                    {move || if can_activate() {
                        view! { cx,
                            <button
                                class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                                on:click=move |_| on_activate.call(())
                            >
                                "Activate Contract"
                            </button>
                        }
                    } else {
                        view! { cx, <span></span> }
                    }}
                </div>
            </div>
        </div>
    }
} 