use leptos::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegistrationForm {
    #[validate(length(min = 3, max = 100))]
    organization_name: String,
    
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
    
    #[validate(must_match = "password")]
    confirm_password: String,
    
    #[validate(length(min = 1))]
    participant_type: Vec<ParticipantType>,
    
    #[validate]
    contact_info: ContactInfo,
    
    terms_accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantType {
    TrainingDataProvider,
    CleanRoomProvider,
    DataConsumer,
    SystemAdministrator,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ContactInfo {
    #[validate(length(min = 3, max = 100))]
    full_name: String,
    
    #[validate(phone)]
    phone: String,
    
    #[validate(length(min = 5, max = 200))]
    address: String,
}

#[component]
pub fn RegistrationPage() -> impl IntoView {
    let (form, set_form) = create_signal(RegistrationForm::default());
    let (errors, set_errors) = create_signal(Vec::new());
    
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        
        // Validate form
        if let Err(validation_errors) = form.get().validate() {
            set_errors.set(validation_errors.field_errors().into());
            return;
        }
        
        // Submit registration
        spawn_local(async move {
            match register_participant(form.get()).await {
                Ok(_) => {
                    // Show success message and redirect
                    notify("Registration successful! Please check your email for verification.");
                    redirect("/login");
                }
                Err(e) => {
                    set_errors.set(vec![e.to_string()]);
                }
            }
        });
    };
    
    view! {
        <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
            <div class="sm:mx-auto sm:w-full sm:max-w-md">
                <h2 class="text-center text-3xl font-extrabold text-gray-900">
                    "Register as a Participant"
                </h2>
            </div>

            <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
                <div class="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10">
                    <form class="space-y-6" on:submit=handle_submit>
                        // Organization Information
                        <div>
                            <label for="organization_name" class="block text-sm font-medium text-gray-700">
                                "Organization Name"
                            </label>
                            <input
                                type="text"
                                name="organization_name"
                                id="organization_name"
                                required
                                class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm"
                                prop:value=move || form.get().organization_name
                                on:input=move |ev| {
                                    let mut new_form = form.get();
                                    new_form.organization_name = event_target_value(&ev);
                                    set_form.set(new_form);
                                }
                            />
                        </div>

                        // Email
                        <div>
                            <label for="email" class="block text-sm font-medium text-gray-700">
                                "Email Address"
                            </label>
                            <input
                                type="email"
                                name="email"
                                id="email"
                                required
                                class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm"
                                prop:value=move || form.get().email
                                on:input=move |ev| {
                                    let mut new_form = form.get();
                                    new_form.email = event_target_value(&ev);
                                    set_form.set(new_form);
                                }
                            />
                        </div>

                        // Participant Type
                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Participant Type"
                            </label>
                            <div class="mt-2 space-y-2">
                                <div class="flex items-center">
                                    <input
                                        type="checkbox"
                                        id="data_provider"
                                        class="h-4 w-4 text-blue-600 border-gray-300 rounded"
                                        on:change=move |ev| {
                                            let mut new_form = form.get();
                                            if event_target_checked(&ev) {
                                                new_form.participant_type.push(ParticipantType::TrainingDataProvider);
                                            } else {
                                                new_form.participant_type.retain(|t| !matches!(t, ParticipantType::TrainingDataProvider));
                                            }
                                            set_form.set(new_form);
                                        }
                                    />
                                    <label for="data_provider" class="ml-2 text-sm text-gray-700">
                                        "Training Data Provider"
                                    </label>
                                </div>
                                <div class="flex items-center">
                                    <input
                                        type="checkbox"
                                        id="clean_room"
                                        class="h-4 w-4 text-blue-600 border-gray-300 rounded"
                                        on:change=move |ev| {
                                            let mut new_form = form.get();
                                            if event_target_checked(&ev) {
                                                new_form.participant_type.push(ParticipantType::CleanRoomProvider);
                                            } else {
                                                new_form.participant_type.retain(|t| !matches!(t, ParticipantType::CleanRoomProvider));
                                            }
                                            set_form.set(new_form);
                                        }
                                    />
                                    <label for="clean_room" class="ml-2 text-sm text-gray-700">
                                        "Clean Room Provider"
                                    </label>
                                </div>
                                <div class="flex items-center">
                                    <input
                                        type="checkbox"
                                        id="data_consumer"
                                        class="h-4 w-4 text-blue-600 border-gray-300 rounded"
                                        on:change=move |ev| {
                                            let mut new_form = form.get();
                                            if event_target_checked(&ev) {
                                                new_form.participant_type.push(ParticipantType::DataConsumer);
                                            } else {
                                                new_form.participant_type.retain(|t| !matches!(t, ParticipantType::DataConsumer));
                                            }
                                            set_form.set(new_form);
                                        }
                                    />
                                    <label for="data_consumer" class="ml-2 text-sm text-gray-700">
                                        "Data Consumer"
                                    </label>
                                </div>
                            </div>
                        </div>

                        // Contact Information
                        <div class="space-y-4">
                            <h3 class="text-lg font-medium text-gray-900">
                                "Contact Information"
                            </h3>
                            <div>
                                <label for="full_name" class="block text-sm font-medium text-gray-700">
                                    "Full Name"
                                </label>
                                <input
                                    type="text"
                                    name="full_name"
                                    id="full_name"
                                    required
                                    class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm"
                                    prop:value=move || form.get().contact_info.full_name
                                    on:input=move |ev| {
                                        let mut new_form = form.get();
                                        new_form.contact_info.full_name = event_target_value(&ev);
                                        set_form.set(new_form);
                                    }
                                />
                            </div>
                            <div>
                                <label for="phone" class="block text-sm font-medium text-gray-700">
                                    "Phone Number"
                                </label>
                                <input
                                    type="tel"
                                    name="phone"
                                    id="phone"
                                    required
                                    class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm"
                                    prop:value=move || form.get().contact_info.phone
                                    on:input=move |ev| {
                                        let mut new_form = form.get();
                                        new_form.contact_info.phone = event_target_value(&ev);
                                        set_form.set(new_form);
                                    }
                                />
                            </div>
                        </div>

                        // Terms and Conditions
                        <div class="flex items-center">
                            <input
                                type="checkbox"
                                id="terms"
                                required
                                class="h-4 w-4 text-blue-600 border-gray-300 rounded"
                                prop:checked=move || form.get().terms_accepted
                                on:change=move |ev| {
                                    let mut new_form = form.get();
                                    new_form.terms_accepted = event_target_checked(&ev);
                                    set_form.set(new_form);
                                }
                            />
                            <label for="terms" class="ml-2 text-sm text-gray-700">
                                "I agree to the "
                                <a href="/terms" class="text-blue-600 hover:text-blue-500">
                                    "Terms and Conditions"
                                </a>
                            </label>
                        </div>

                        // Error Messages
                        <Show
                            when=move || !errors.get().is_empty()
                            fallback=|| view! { <div></div> }
                        >
                            <div class="rounded-md bg-red-50 p-4">
                                <div class="flex">
                                    <div class="ml-3">
                                        <h3 class="text-sm font-medium text-red-800">
                                            "There were errors with your submission"
                                        </h3>
                                        <div class="mt-2 text-sm text-red-700">
                                            <ul class="list-disc pl-5 space-y-1">
                                                {move || errors.get().into_iter().map(|error| {
                                                    view! {
                                                        <li>{error}</li>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </ul>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </Show>

                        // Submit Button
                        <div>
                            <button
                                type="submit"
                                class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                            >
                                "Register"
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}

async fn register_participant(form: RegistrationForm) -> Result<(), String> {
    // Call the API to register the participant
    let response = api::post("/api/register", &form).await
        .map_err(|e| e.to_string())?;
        
    if response.status().is_success() {
        Ok(())
    } else {
        let error = response.json::<ApiError>().await
            .map_err(|e| e.to_string())?;
        Err(error.message)
    }
} 