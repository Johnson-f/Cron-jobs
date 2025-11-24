use crate::server::actions::helpers::{get_user_id_from_request, get_turso_client};
use crate::server::service::create_user_database;
use leptos::prelude::ServerFnError;
use leptos::server;
use leptos_actix::extract;
use log::{info, error};

#[server(CreateUserDatabase, "/api")]
pub async fn create_user_database_action(email: String) -> Result<String, ServerFnError> {
    let req = extract::<actix_web::HttpRequest>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;
    
    let user_id = get_user_id_from_request(&req).await?;
    info!("[Database Setup] Checking database for user: {} ({})", user_id, email);
    
    let client = get_turso_client(&req)?;
    
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

