use crate::server::turso::{get_supabase_user_id, TursoClient, TursoConfig};
use leptos::*;
use leptos_actix::extract;
use std::sync::Arc;

/// Helper to get user_id from request by validating JWT token
pub(crate) async fn get_user_id_from_request(req: &actix_web::HttpRequest) -> Result<String, ServerFnError> {
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ServerFnError::new("Authorization header missing"))?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ServerFnError::new("Invalid authorization format"))?;
    
    let config = TursoConfig::from_env()
        .map_err(|e| ServerFnError::new(format!("Config error: {}", e)))?;
    
    let user_id = get_supabase_user_id(token, &config.supabase)
        .await
        .map_err(|e| ServerFnError::new(format!("JWT validation failed: {}", e)))?;
    
    Ok(user_id)
}

/// Helper to get TursoClient from app data
pub(crate) fn get_turso_client(req: &actix_web::HttpRequest) -> Result<Arc<TursoClient>, ServerFnError> {
    req.app_data::<actix_web::web::Data<Arc<TursoClient>>>()
        .ok_or_else(|| ServerFnError::new("TursoClient not found in app data"))
        .map(|data| data.get_ref().clone())
}

