use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::auth::AuthContext;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractSummary {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: ContractStatus,
    pub created_at: String,
    pub provider_name: String,
    pub consumer_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ContractStatus {
    Draft,
    PendingSignature,
    Active,
    Suspended,
    Terminated,
    Expired,
}

#[derive(Clone, Debug)]
pub struct ContractFilter {
    pub status: Option<ContractStatus>,
    pub search_term: String,
    pub page: usize,
    pub per_page: usize,
}

#[component]
pub fn ContractList(
    cx: Scope,
    #[prop(into)] contracts: Signal<Vec<ContractSummary>>,
    #[prop(into)] total_count: Signal<usize>,
    #[prop(into)] on_filter_change: Callback<ContractFilter>,
    #[prop(into)] on_contract_click: Callback<Uuid>,
) -> impl IntoView {
    let auth = use_context::<AuthContext>(cx).expect("Auth context not found");
    
    let (filter, set_filter) = create_signal(cx, ContractFilter {
        status: None,
        search_term: String::new(),
        page: 1,
        per_page: 10,
    });

    let total_pages = move || {
        let count = total_count.get();
        let per_page = filter.get().per_page;
        (count + per_page - 1) / per_page
    };

    let handle_search = move |ev: web_sys::InputEvent| {
        let mut current_filter = filter.get();
        current_filter.search_term = event_target_value(&ev);
        current_filter.page = 1;
        set_filter.set(current_filter.clone());
        on_filter_change.call(current_filter);
    };

    let handle_status_change = move |ev: web_sys::Event| {
        let mut current_filter = filter.get();
        current_filter.status = match event_target_value(&ev).as_str() {
            "all" => None,
            "draft" => Some(ContractStatus::Draft),
            "pending" => Some(ContractStatus::PendingSignature),
            "active" => Some(ContractStatus::Active),
            "suspended" => Some(ContractStatus::Suspended),
            "terminated" => Some(ContractStatus::Terminated),
            "expired" => Some(ContractStatus::Expired),
            _ => None,
        };
        current_filter.page = 1;
        set_filter.set(current_filter.clone());
        on_filter_change.call(current_filter);
    };

    let handle_page_change = move |page: usize| {
        let mut current_filter = filter.get();
        current_filter.page = page;
        set_filter.set(current_filter.clone());
        on_filter_change.call(current_filter);
    };

    view! { cx,
        <div class="space-y-4">
            <div class="flex justify-between items-center">
                <div class="flex space-x-4">
                    <input
                        type="text"
                        placeholder="Search contracts..."
                        class="rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                        on:input=handle_search
                        prop:value=move || filter.get().search_term
                    />
                    
                    <select
                        class="rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                        on:change=handle_status_change
                    >
                        <option value="all">"All Status"</option>
                        <option value="draft">"Draft"</option>
                        <option value="pending">"Pending Signature"</option>
                        <option value="active">"Active"</option>
                        <option value="suspended">"Suspended"</option>
                        <option value="terminated">"Terminated"</option>
                        <option value="expired">"Expired"</option>
                    </select>
                </div>
            </div>

            <div class="bg-white shadow overflow-hidden sm:rounded-md">
                <ul role="list" class="divide-y divide-gray-200">
                    {move || contracts.get().into_iter().map(|contract| view! { cx,
                        <li class="hover:bg-gray-50 cursor-pointer" on:click=move |_| on_contract_click.call(contract.id)>
                            <div class="px-4 py-4 sm:px-6">
                                <div class="flex items-center justify-between">
                                    <p class="text-sm font-medium text-indigo-600 truncate">
                                        {&contract.title}
                                    </p>
                                    <div class="ml-2 flex-shrink-0 flex">
                                        {move || {
                                            let status_class = match contract.status {
                                                ContractStatus::Draft => "bg-gray-100 text-gray-800",
                                                ContractStatus::PendingSignature => "bg-yellow-100 text-yellow-800",
                                                ContractStatus::Active => "bg-green-100 text-green-800",
                                                ContractStatus::Suspended => "bg-red-100 text-red-800",
                                                ContractStatus::Terminated => "bg-red-100 text-red-800",
                                                ContractStatus::Expired => "bg-gray-100 text-gray-800",
                                            };
                                            view! { cx,
                                                <span class=format!("px-2 inline-flex text-xs leading-5 font-semibold rounded-full {}", status_class)>
                                                    {format!("{:?}", contract.status)}
                                                </span>
                                            }
                                        }}
                                    </div>
                                </div>
                                <div class="mt-2 sm:flex sm:justify-between">
                                    <div class="sm:flex">
                                        <p class="flex items-center text-sm text-gray-500">
                                            "Provider: " {&contract.provider_name}
                                        </p>
                                        <p class="mt-2 flex items-center text-sm text-gray-500 sm:mt-0 sm:ml-6">
                                            "Consumer: " {&contract.consumer_name}
                                        </p>
                                    </div>
                                    <div class="mt-2 flex items-center text-sm text-gray-500 sm:mt-0">
                                        <p>
                                            "Created: " {&contract.created_at}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </li>
                    }).collect::<Vec<_>>()}
                </ul>
            </div>

            <div class="flex items-center justify-between border-t border-gray-200 bg-white px-4 py-3 sm:px-6">
                <div class="flex flex-1 justify-between sm:hidden">
                    <button
                        class="relative inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
                        on:click=move |_| {
                            let current_page = filter.get().page;
                            if current_page > 1 {
                                handle_page_change(current_page - 1);
                            }
                        }
                        disabled=move || filter.get().page <= 1
                    >
                        "Previous"
                    </button>
                    <button
                        class="relative ml-3 inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
                        on:click=move |_| {
                            let current_page = filter.get().page;
                            if current_page < total_pages() {
                                handle_page_change(current_page + 1);
                            }
                        }
                        disabled=move || filter.get().page >= total_pages()
                    >
                        "Next"
                    </button>
                </div>
                <div class="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
                    <div>
                        <p class="text-sm text-gray-700">
                            "Showing "
                            <span class="font-medium">
                                {move || ((filter.get().page - 1) * filter.get().per_page + 1).to_string()}
                            </span>
                            " to "
                            <span class="font-medium">
                                {move || {
                                    let current_page = filter.get().page;
                                    let per_page = filter.get().per_page;
                                    let total = total_count.get();
                                    std::cmp::min(current_page * per_page, total).to_string()
                                }}
                            </span>
                            " of "
                            <span class="font-medium">{move || total_count.get().to_string()}</span>
                            " results"
                        </p>
                    </div>
                    <div>
                        <nav class="isolate inline-flex -space-x-px rounded-md shadow-sm" aria-label="Pagination">
                            {move || {
                                let current_page = filter.get().page;
                                let total = total_pages();
                                let mut pages = vec![];
                                
                                // Previous button
                                pages.push(view! { cx,
                                    <button
                                        class="relative inline-flex items-center rounded-l-md border border-gray-300 bg-white px-2 py-2 text-sm font-medium text-gray-500 hover:bg-gray-50 focus:z-20"
                                        on:click=move |_| {
                                            if current_page > 1 {
                                                handle_page_change(current_page - 1);
                                            }
                                        }
                                        disabled=current_page <= 1
                                    >
                                        "Previous"
                                    </button>
                                });

                                // Page numbers
                                for page in 1..=total {
                                    let is_current = page == current_page;
                                    pages.push(view! { cx,
                                        <button
                                            class={if is_current {
                                                "relative z-10 inline-flex items-center border border-indigo-500 bg-indigo-50 px-4 py-2 text-sm font-medium text-indigo-600 focus:z-20"
                                            } else {
                                                "relative inline-flex items-center border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-500 hover:bg-gray-50 focus:z-20"
                                            }}
                                            on:click=move |_| handle_page_change(page)
                                        >
                                            {page.to_string()}
                                        </button>
                                    });
                                }

                                // Next button
                                pages.push(view! { cx,
                                    <button
                                        class="relative inline-flex items-center rounded-r-md border border-gray-300 bg-white px-2 py-2 text-sm font-medium text-gray-500 hover:bg-gray-50 focus:z-20"
                                        on:click=move |_| {
                                            if current_page < total {
                                                handle_page_change(current_page + 1);
                                            }
                                        }
                                        disabled=current_page >= total
                                    >
                                        "Next"
                                    </button>
                                });

                                pages
                            }}
                        </nav>
                    </div>
                </div>
            </div>
        </div>
    }
} 