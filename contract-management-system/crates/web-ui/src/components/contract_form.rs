use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::auth::AuthContext;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractFormData {
    pub title: String,
    pub description: String,
    pub contract_type: ContractType,
    pub provider_id: Option<Uuid>,
    pub consumer_id: Option<Uuid>,
    pub terms: ContractTerms,
    pub valid_from: String,
    pub valid_until: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ContractType {
    DataSharing,
    ModelTraining,
    ResultSharing,
    Hybrid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractTerms {
    pub data_access_scope: Vec<String>,
    pub usage_restrictions: Vec<String>,
    pub retention_period_days: i32,
    pub security_requirements: SecurityRequirements,
    pub compliance_requirements: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityRequirements {
    pub encryption_required: bool,
    pub min_encryption_level: String,
    pub audit_logging_required: bool,
    pub network_isolation_required: bool,
}

#[component]
pub fn ContractForm(
    cx: Scope,
    on_submit: Action<ContractFormData, Result<Uuid, String>>,
) -> impl IntoView {
    let auth = use_context::<AuthContext>(cx).expect("Auth context not found");
    
    let (form_data, set_form_data) = create_signal(cx, ContractFormData {
        title: String::new(),
        description: String::new(),
        contract_type: ContractType::DataSharing,
        provider_id: None,
        consumer_id: None,
        terms: ContractTerms {
            data_access_scope: vec![],
            usage_restrictions: vec![],
            retention_period_days: 30,
            security_requirements: SecurityRequirements {
                encryption_required: true,
                min_encryption_level: "AES-256".to_string(),
                audit_logging_required: true,
                network_isolation_required: false,
            },
            compliance_requirements: vec![],
        },
        valid_from: chrono::Utc::now().date().to_string(),
        valid_until: None,
    });

    let (error, set_error) = create_signal(cx, None::<String>);
    
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let current_data = form_data.get();
        
        // Validate form data
        if current_data.title.is_empty() {
            set_error.set(Some("Title is required".to_string()));
            return;
        }
        
        // Submit form
        on_submit.dispatch(current_data);
    };

    view! { cx,
        <form class="space-y-6" on:submit=handle_submit>
            {move || error.get().map(|err| view! { cx,
                <div class="text-red-500">{err}</div>
            })}
            
            <div>
                <label for="title" class="block text-sm font-medium text-gray-700">
                    "Title"
                </label>
                <input
                    type="text"
                    id="title"
                    class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    prop:value=move || form_data.get().title
                    on:input=move |ev| {
                        let mut data = form_data.get();
                        data.title = event_target_value(&ev);
                        set_form_data.set(data);
                    }
                />
            </div>

            <div>
                <label for="description" class="block text-sm font-medium text-gray-700">
                    "Description"
                </label>
                <textarea
                    id="description"
                    class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    prop:value=move || form_data.get().description
                    on:input=move |ev| {
                        let mut data = form_data.get();
                        data.description = event_target_value(&ev);
                        set_form_data.set(data);
                    }
                />
            </div>

            <div>
                <label for="contract_type" class="block text-sm font-medium text-gray-700">
                    "Contract Type"
                </label>
                <select
                    id="contract_type"
                    class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    on:change=move |ev| {
                        let mut data = form_data.get();
                        data.contract_type = match event_target_value(&ev).as_str() {
                            "DataSharing" => ContractType::DataSharing,
                            "ModelTraining" => ContractType::ModelTraining,
                            "ResultSharing" => ContractType::ResultSharing,
                            "Hybrid" => ContractType::Hybrid,
                            _ => ContractType::DataSharing,
                        };
                        set_form_data.set(data);
                    }
                >
                    <option value="DataSharing">"Data Sharing"</option>
                    <option value="ModelTraining">"Model Training"</option>
                    <option value="ResultSharing">"Result Sharing"</option>
                    <option value="Hybrid">"Hybrid"</option>
                </select>
            </div>

            // Security Requirements
            <div class="space-y-4">
                <h3 class="text-lg font-medium text-gray-900">"Security Requirements"</h3>
                
                <div class="flex items-center">
                    <input
                        type="checkbox"
                        id="encryption_required"
                        class="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                        prop:checked=move || form_data.get().terms.security_requirements.encryption_required
                        on:change=move |ev| {
                            let mut data = form_data.get();
                            data.terms.security_requirements.encryption_required = event_target_checked(&ev);
                            set_form_data.set(data);
                        }
                    />
                    <label for="encryption_required" class="ml-2 block text-sm text-gray-900">
                        "Require Encryption"
                    </label>
                </div>

                <div class="flex items-center">
                    <input
                        type="checkbox"
                        id="audit_logging"
                        class="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                        prop:checked=move || form_data.get().terms.security_requirements.audit_logging_required
                        on:change=move |ev| {
                            let mut data = form_data.get();
                            data.terms.security_requirements.audit_logging_required = event_target_checked(&ev);
                            set_form_data.set(data);
                        }
                    />
                    <label for="audit_logging" class="ml-2 block text-sm text-gray-900">
                        "Require Audit Logging"
                    </label>
                </div>
            </div>

            <div class="flex justify-end">
                <button
                    type="submit"
                    class="ml-3 inline-flex justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
                >
                    "Create Contract"
                </button>
            </div>
        </form>
    }
} 