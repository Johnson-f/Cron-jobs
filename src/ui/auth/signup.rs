use crate::context::AuthContext;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

#[component]
pub fn SignupPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let navigate = use_navigate();
    
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let email_val = email.get();
        let password_val = password.get();
        let confirm_password_val = confirm_password.get();
        
        // Basic validation
        if email_val.is_empty() {
            error.set(Some("Email is required".to_string()));
            return;
        }
        
        if password_val.is_empty() {
            error.set(Some("Password is required".to_string()));
            return;
        }
        
        if confirm_password_val.is_empty() {
            error.set(Some("Please confirm your password".to_string()));
            return;
        }
        
        // Email format validation
        if !email_val.contains('@') {
            error.set(Some("Please enter a valid email address".to_string()));
            return;
        }
        
        // Password strength validation
        if password_val.len() < 6 {
            error.set(Some("Password must be at least 6 characters".to_string()));
            return;
        }
        
        // Password match validation
        if password_val != confirm_password_val {
            error.set(Some("Passwords do not match".to_string()));
            return;
        }
        
        error.set(None);
        is_submitting.set(true);
        
        let auth_clone = auth.clone();
        let email_clone = email_val.clone();
        let password_clone = password_val.clone();
        let nav = navigate.clone();
        
        spawn_local(async move {
            match auth_clone.signup(email_clone, password_clone).await {
                Ok(_) => {
                    nav("/home", Default::default());
                }
                Err(e) => {
                    error.set(Some(format!("Sign up failed: {}", e)));
                    is_submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
            <div class="max-w-md w-full space-y-8 p-8">
                <div class="text-center">
                    <h2 class="text-3xl font-bold text-gray-900 mb-2">"Sign Up"</h2>
                    <p class="text-gray-600">"Create a new account"</p>
                </div>
                
                <div class="bg-white rounded-lg shadow-lg p-8">
                    <form on:submit=handle_submit class="space-y-6">
                        <div>
                            <label for="email" class="block text-sm font-medium text-gray-700 mb-2">
                                "Email"
                            </label>
                            <input
                                id="email"
                                type="email"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                                placeholder="you@example.com"
                                prop:value=email
                                on:input=move |ev| {
                                    email.set(event_target_value(&ev));
                                    error.set(None);
                                }
                                disabled=move || is_submitting.get()
                            />
                        </div>
                        
                        <div>
                            <label for="password" class="block text-sm font-medium text-gray-700 mb-2">
                                "Password"
                            </label>
                            <input
                                id="password"
                                type="password"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                                placeholder="Enter your password"
                                prop:value=password
                                on:input=move |ev| {
                                    password.set(event_target_value(&ev));
                                    error.set(None);
                                }
                                disabled=move || is_submitting.get()
                            />
                        </div>
                        
                        <div>
                            <label for="confirm_password" class="block text-sm font-medium text-gray-700 mb-2">
                                "Confirm Password"
                            </label>
                            <input
                                id="confirm_password"
                                type="password"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                                placeholder="Confirm your password"
                                prop:value=confirm_password
                                on:input=move |ev| {
                                    confirm_password.set(event_target_value(&ev));
                                    error.set(None);
                                }
                                disabled=move || is_submitting.get()
                            />
                        </div>
                        
                        {move || error.get().map(|err| view! {
                            <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
                                {err}
                            </div>
                        })}
                        
                        <button
                            r#type="submit"
                            class="w-full px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || is_submitting.get()
                        >
                            {move || if is_submitting.get() {
                                "Creating account..."
                            } else {
                                "Sign Up"
                            }}
                        </button>
                        
                        <div class="text-center">
                            <a
                                href="/login"
                                class="text-sm text-indigo-600 hover:text-indigo-500"
                            >
                                "Already have an account? Login"
                            </a>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}

