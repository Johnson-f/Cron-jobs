use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn LandingPage() -> impl IntoView {
    let navigate = use_navigate();
    
    let nav_login = navigate.clone();
    let nav_signup = navigate.clone();
    
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
            <div class="max-w-md w-full space-y-8 p-8">
                <div class="text-center">
                    <h1 class="text-4xl font-bold text-gray-900 mb-2">
                        "Cron Jobs"
                    </h1>
                    <p class="text-lg text-gray-600 mb-8">
                        "Manage your scheduled tasks with ease"
                    </p>
                </div>
                
                <div class="bg-white rounded-lg shadow-lg p-8 space-y-4">
                    <button
                        class="w-full px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        on:click=move |_| {
                            nav_login("/login", Default::default());
                        }
                    >
                        "Login"
                    </button>
                    
                    <button
                        class="w-full px-4 py-2 bg-white border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        on:click=move |_| {
                            nav_signup("/signup", Default::default());
                        }
                    >
                        "Sign Up"
                    </button>
                </div>
            </div>
        </div>
    }
}

