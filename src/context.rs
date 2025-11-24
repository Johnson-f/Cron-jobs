use crate::client::{Session, SupabaseClient, User, SupabaseError};
use crate::config::{get_supabase_url, get_supabase_anon_key};
use leptos::prelude::*;
use leptos::task::spawn_local;

// Server function definition - simplified macro
#[server(CreateUserDatabase, "/api")]
pub async fn create_user_database_action(email: String, access_token: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::server::turso::{get_supabase_user_id, TursoClient, TursoConfig};
        use crate::server::service::create_user_database;
        use leptos_actix::extract;
        use log::{info, error};
        use std::sync::Arc;
        use actix_web::web;
        
        // Extract TursoClient from Actix app data
        let req = extract::<actix_web::HttpRequest>().await
            .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;
        
        let client = req.app_data::<web::Data<Arc<TursoClient>>>()
            .ok_or_else(|| ServerFnError::new("TursoClient not found in app data"))?
            .get_ref()
            .clone();
        
        // Validate JWT token and extract user_id
        let config = TursoConfig::from_env()
            .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
        
        let user_id = get_supabase_user_id(&access_token, &config.supabase)
            .await
            .map_err(|e| ServerFnError::new(format!("JWT validation failed: {}", e)))?;
        
        info!("[Database Setup] Checking database for user: {} ({})", user_id, email);
        
        // Check if user database already exists
        match client.get_user_database_entry(&user_id).await {
            Ok(_) => {
                info!("[Database Setup] Database exists for user {}, syncing schema...", user_id);
                client.sync_user_database_schema(&user_id)
                    .await
                    .map_err(|e| {
                        error!("[Database Setup] Failed to sync schema for user {}: {}", user_id, e);
                        ServerFnError::new(format!("Failed to sync user database schema: {}", e))
                    })?;
                info!("[Database Setup] Schema sync completed successfully for user {}", user_id);
                Ok("Database schema updated".to_string())
            }
            Err(_) => {
                info!("[Database Setup] Creating new database for user: {} ({})", user_id, email);
                create_user_database(&client, &user_id, &email)
                    .await
                    .map_err(|e| {
                        error!("[Database Setup] Failed to create database for user {}: {}", user_id, e);
                        ServerFnError::new(format!("Failed to create user database: {}", e))
                    })?;
                info!("[Database Setup] Database created successfully for user: {} ({})", user_id, email);
                Ok("Database created".to_string())
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        // This branch should never execute - Leptos handles client-side automatically
        // But we need it for compilation
        Err(ServerFnError::new("Server function should not be called directly on client"))
    }
}

#[derive(Clone)]
pub struct AuthContext {
    pub user: RwSignal<Option<User>>,
    pub session: RwSignal<Option<Session>>,
    pub client: SupabaseClient,
    pub is_loading: RwSignal<bool>,
    pub db_status: RwSignal<Option<String>>,
}

impl AuthContext {
    pub fn new() -> Self {
        let url = get_supabase_url();
        let anon_key = get_supabase_anon_key();

        let client = SupabaseClient::new(url, anon_key);
        
        let user = RwSignal::new(None);
        let session = RwSignal::new(None);
        let is_loading = RwSignal::new(true);
        let db_status = RwSignal::new(None);

        let context = Self {
            user,
            session,
            client,
            is_loading,
            db_status,
        };

        // Initialize from localStorage
        context.init();

        context
    }

    fn init(&self) {
        let context = self.clone();
        spawn_local(async move {
            match context.client.get_session() {
                Ok(Some(session)) => {
                    context.session.set(Some(session.clone()));
                    context.user.set(Some(session.user));
                }
                Ok(None) => {
                    context.session.set(None);
                    context.user.set(None);
                }
                Err(_) => {
                    // Silently fail on init - might be first visit
                    context.session.set(None);
                    context.user.set(None);
                }
            }
            context.is_loading.set(false);
        });
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some()
    }

    pub async fn login(&self, email: String, password: String) -> Result<(), SupabaseError> {
        self.is_loading.set(true);
        let result = self.client.sign_in(email.clone(), password).await;
        
        match result {
            Ok(session) => {
                self.session.set(Some(session.clone()));
                self.user.set(Some(session.user.clone()));
                
                // Trigger database creation/update after successful login
                let email_for_db = email.clone();
                let access_token = session.access_token.clone();
                let db_status = self.db_status.clone();
                
                db_status.set(Some("Initializing database...".to_string()));
                
                spawn_local(async move {
                    match create_user_database_action(email_for_db, access_token).await {
                        Ok(status) => {
                            #[cfg(target_arch = "wasm32")]
                            web_sys::console::log_1(&format!("[Database] {}", status).into());
                            #[cfg(not(target_arch = "wasm32"))]
                            log::info!("[Database] {}", status);
                            db_status.set(Some(format!("✓ {}", status)));
                        }
                        Err(e) => {
                            #[cfg(target_arch = "wasm32")]
                            web_sys::console::error_1(&format!("[Database] Setup failed: {}", e).into());
                            #[cfg(not(target_arch = "wasm32"))]
                            log::error!("[Database] Setup failed: {}", e);
                            db_status.set(Some(format!("⚠ Database setup failed: {}", e)));
                        }
                    }
                });
                
                self.is_loading.set(false);
                Ok(())
            }
            Err(e) => {
                self.is_loading.set(false);
                Err(e)
            }
        }
    }

    pub async fn signup(&self, email: String, password: String) -> Result<(), SupabaseError> {
        self.is_loading.set(true);
        let result = self.client.sign_up(email.clone(), password).await;
        
        match result {
            Ok(session) => {
                self.session.set(Some(session.clone()));
                self.user.set(Some(session.user.clone()));
                
                // Trigger database creation/update after successful signup
                let email_for_db = email.clone();
                let access_token = session.access_token.clone();
                let db_status = self.db_status.clone();
                
                db_status.set(Some("Initializing database...".to_string()));
                
                spawn_local(async move {
                    match create_user_database_action(email_for_db, access_token).await {
                        Ok(status) => {
                            #[cfg(target_arch = "wasm32")]
                            web_sys::console::log_1(&format!("[Database] {}", status).into());
                            #[cfg(not(target_arch = "wasm32"))]
                            log::info!("[Database] {}", status);
                            db_status.set(Some(format!("✓ {}", status)));
                        }
                        Err(e) => {
                            #[cfg(target_arch = "wasm32")]
                            web_sys::console::error_1(&format!("[Database] Setup failed: {}", e).into());
                            #[cfg(not(target_arch = "wasm32"))]
                            log::error!("[Database] Setup failed: {}", e);
                            db_status.set(Some(format!("⚠ Database setup failed: {}", e)));
                        }
                    }
                });
                
                self.is_loading.set(false);
                Ok(())
            }
            Err(e) => {
                self.is_loading.set(false);
                Err(e)
            }
        }
    }

    pub async fn logout(&self) -> Result<(), SupabaseError> {
        self.is_loading.set(true);
        let result = self.client.sign_out().await;
        
        self.session.set(None);
        self.user.set(None);
        self.is_loading.set(false);
        
        result
    }
}