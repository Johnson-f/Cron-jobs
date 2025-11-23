use crate::context::AuthContext;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn ProtectedRoute(children: ChildrenFn) -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let navigate = use_navigate();
    
    let auth_effect = auth.clone();
    Effect::new(move |_| {
        if !auth_effect.is_loading.get() && !auth_effect.is_authenticated() {
            navigate("/login", Default::default());
        }
    });
    
    // Store auth and children in StoredValue to make them Copy + 'static
    let auth_show = StoredValue::new(auth.clone());
    let auth_fallback = StoredValue::new(auth.clone());
    let children_stored = StoredValue::new(children);
    
    view! {
        <Show
            when=move || auth_show.with_value(|a| !a.is_loading.get() && a.is_authenticated())
            fallback=move || {
                auth_fallback.with_value(|a| {
                    if a.is_loading.get() {
                        view! {
                            <div class="min-h-screen flex items-center justify-center">
                                <div class="text-center">
                                    <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                                    <p class="mt-4 text-gray-600">"Loading..."</p>
                                </div>
                            </div>
                        }
                    } else {
                        view! {
                            <div class="min-h-screen flex items-center justify-center">
                                <div class="text-center">
                                    <p class="text-gray-600">"Redirecting to login..."</p>
                                </div>
                            </div>
                        }
                    }
                })
            }
        >
            {move || children_stored.with_value(|c| c())}
        </Show>
    }
}