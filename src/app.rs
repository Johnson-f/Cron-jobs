use crate::context::AuthContext;
use crate::ui::auth::{LandingPage, LoginPage, SignupPage};
use crate::ui::auth::protected::ProtectedRoute;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment, WildcardSegment,
};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    
    // Initialize and provide auth context
    let auth_context = AuthContext::new();
    provide_context(auth_context);

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/front.css"/>

        // sets the document title
        <Title text="Cron Jobs"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=move || view! { <NotFound/> }>
                    <Route path=StaticSegment("") view=LandingPage/>
                    <Route path=StaticSegment("login") view=LoginPage/>
                    <Route path=StaticSegment("signup") view=SignupPage/>
                    <Route path=StaticSegment("home") view=HomePage/>
                    <Route path=WildcardSegment("any") view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Protected home page - main application
#[component]
fn HomePage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let user = move || auth.user.get();
    
    view! {
        <ProtectedRoute>
            <div class="min-h-screen bg-gray-50">
                <div class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
                    <div class="px-4 py-6 sm:px-0">
                        <div class="border-4 border-dashed border-gray-200 rounded-lg p-8">
                            <h1 class="text-3xl font-bold text-gray-900 mb-4">
                                "Welcome to Cron Jobs"
                            </h1>
                            {move || user().map(|u| view! {
                                <p class="text-lg text-gray-600 mb-4">
                                    "Logged in as: " {u.email}
                                </p>
                            })}
                            <p class="text-gray-500">
                                "Your dashboard will appear here."
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </ProtectedRoute>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
