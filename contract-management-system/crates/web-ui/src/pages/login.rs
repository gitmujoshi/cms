use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::state::AuthContext;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct LoginForm {
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth_context = use_context::<RwSignal<AuthContext>>().expect("Auth context not found");
    let (form, set_form) = create_signal(LoginForm {
        email: String::new(),
        password: String::new(),
    });
    let (errors, set_errors) = create_signal(Vec::new());
    
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        
        // Validate form
        if let Err(validation_errors) = form.get().validate() {
            set_errors.set(validation_errors.field_errors().into());
            return;
        }
        
        // Submit login
        let login_form = form.get();
        spawn_local(async move {
            match auth_context.get().login(&login_form.email, &login_form.password).await {
                Ok(_) => {
                    // Redirect to dashboard
                    let navigate = use_navigate();
                    navigate("/dashboard", Default::default());
                }
                Err(e) => {
                    set_errors.set(vec![e]);
                }
            }
        });
    };
    
    view! {
        <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
            <div class="sm:mx-auto sm:w-full sm:max-w-md">
                <h2 class="text-center text-3xl font-extrabold text-gray-900">
                    "Sign in to your account"
                </h2>
                <p class="mt-2 text-center text-sm text-gray-600">
                    "Or "
                    <A
                        href="/register"
                        class="font-medium text-blue-600 hover:text-blue-500"
                    >
                        "register for a new account"
                    </A>
                </p>
            </div>

            <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
                <div class="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10">
                    <form class="space-y-6" on:submit=handle_submit>
                        // Email
                        <div>
                            <label
                                for="email"
                                class="block text-sm font-medium text-gray-700"
                            >
                                "Email address"
                            </label>
                            <div class="mt-1">
                                <input
                                    id="email"
                                    name="email"
                                    type="email"
                                    autocomplete="email"
                                    required
                                    class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                                    prop:value=move || form.get().email
                                    on:input=move |ev| {
                                        let mut new_form = form.get();
                                        new_form.email = event_target_value(&ev);
                                        set_form.set(new_form);
                                    }
                                />
                            </div>
                        </div>

                        // Password
                        <div>
                            <label
                                for="password"
                                class="block text-sm font-medium text-gray-700"
                            >
                                "Password"
                            </label>
                            <div class="mt-1">
                                <input
                                    id="password"
                                    name="password"
                                    type="password"
                                    autocomplete="current-password"
                                    required
                                    class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                                    prop:value=move || form.get().password
                                    on:input=move |ev| {
                                        let mut new_form = form.get();
                                        new_form.password = event_target_value(&ev);
                                        set_form.set(new_form);
                                    }
                                />
                            </div>
                        </div>

                        // Remember me & Forgot password
                        <div class="flex items-center justify-between">
                            <div class="flex items-center">
                                <input
                                    id="remember-me"
                                    name="remember-me"
                                    type="checkbox"
                                    class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                />
                                <label
                                    for="remember-me"
                                    class="ml-2 block text-sm text-gray-900"
                                >
                                    "Remember me"
                                </label>
                            </div>

                            <div class="text-sm">
                                <a
                                    href="/forgot-password"
                                    class="font-medium text-blue-600 hover:text-blue-500"
                                >
                                    "Forgot your password?"
                                </a>
                            </div>
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
                                "Sign in"
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
} 