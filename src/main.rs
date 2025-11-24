#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    // Find project root by looking for Cargo.toml
    let mut current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    
    // Try to find project root (where Cargo.toml exists)
    loop {
        let cargo_toml = current_dir.join("Cargo.toml");
        let env_file = current_dir.join(".env");
        
        if env_file.exists() {
            // Found .env file, try to load it
            if let Err(e) = dotenv::from_path(&env_file) {
                eprintln!("⚠ Warning: Failed to load .env from {:?}: {}", env_file, e);
            } else {
                eprintln!("✓ Loaded .env from {:?}", env_file);
                break;
            }
        }
        
        // Move up one directory
        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            // Reached filesystem root, try current directory as fallback
            dotenv::dotenv().ok();
            break;
        }
        
        // Safety: don't go too far up (max 10 levels)
        if current_dir.components().count() < 2 {
            dotenv::dotenv().ok();
            break;
        }
    }
    
    // Verify critical environment variable is loaded
    if std::env::var("REGISTRY_DB_URL").is_err() {
        eprintln!("⚠ Warning: REGISTRY_DB_URL not found. Make sure .env file exists in project root.");
    }
    
    use actix_files::Files;
    use actix_web::*;
    use leptos::prelude::*;
    use leptos::config::get_configuration;
    use leptos_meta::MetaTags;
    use leptos_actix::{generate_route_list, LeptosRoutes, handle_server_fns};
    use ::cron_jobs::app::*;
    use ::cron_jobs::server::turso::{TursoClient, TursoConfig};
    use std::sync::Arc;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;

    // Initialize Turso client
    let turso_config = TursoConfig::from_env()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Turso config error: {}", e)))?;
    
    let turso_client = Arc::new(
        TursoClient::new(turso_config)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to initialize Turso client: {}", e)))?
    );

    // Health check
    if let Err(e) = turso_client.health_check().await {
        eprintln!("Warning: Registry database health check failed: {}", e);
    }

    let turso_client_data = web::Data::from(turso_client.clone());

    println!("listening on http://{}", &addr);

    HttpServer::new(move || {
        // Generate the list of routes in your Leptos App
        let routes = generate_route_list(App);
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone().to_string();

        App::new()
            // Add TursoClient to app data - MUST be before routes
            .app_data(turso_client_data.clone())
            .app_data(web::Data::new(leptos_options.to_owned()))
            // Register server function handler with .route() instead of .service()
            .route("/api/{tail:.*}", handle_server_fns())
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", &site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            // Leptos routes handle server functions automatically
            .leptos_routes(routes, {
                let leptos_options = leptos_options.clone();
                move || {
                    // Get Supabase config from environment
                    let supabase_url = std::env::var("VITE_SUPABASE_URL")
                        .unwrap_or_else(|_| "https://your-project.supabase.co".to_string());
                    let supabase_anon_key = std::env::var("VITE_SUPABASE_ANON_KEY")
                        .unwrap_or_else(|_| "your-anon-key".to_string());
                    
                    view! {
                        <!DOCTYPE html>
                        <html lang="en">
                            <head>
                                <meta charset="utf-8"/>
                                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                <AutoReload options=leptos_options.clone() />
                                <HydrationScripts options=leptos_options.clone()/>
                                <MetaTags/>
                                <script>
                                    {format!(
                                        r#"
                                        window.__ENV__ = {{
                                            VITE_SUPABASE_URL: "{}",
                                            VITE_SUPABASE_ANON_KEY: "{}"
                                        }};
                                        "#,
                                        supabase_url.replace('"', "\\\""),
                                        supabase_anon_key.replace('"', "\\\"")
                                    )}
                                </script>
                            </head>
                            <body>
                                <App/>
                            </body>
                        </html>
                    }
                }
            })
        //.wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::config::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use cron_jobs::app::*;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}