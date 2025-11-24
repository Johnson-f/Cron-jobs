use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone)]
pub struct TursoConfig {
    pub registry_db_url: String,
    pub registry_db_token: String,
    pub turso_api_token: String,
    pub turso_org: String,
    pub supabase: SupabaseConfig,
}

#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
    pub service_role_key: String,
    pub jwks_url: String,
}

impl TursoConfig {
    pub fn from_env() -> Result<Self, String> {
        let registry_db_url = env::var("REGISTRY_DB_URL")
            .map_err(|_| "REGISTRY_DB_URL environment variable not set")?;
        
        let registry_db_token = env::var("REGISTRY_DB_TOKEN")
            .map_err(|_| "REGISTRY_DB_TOKEN environment variable not set")?;
        
        let turso_api_token = env::var("TURSO_API_TOKEN")
            .map_err(|_| "TURSO_API_TOKEN environment variable not set")?;
        
        let turso_org = env::var("TURSO_ORG")
            .map_err(|_| "TURSO_ORG environment variable not set")?;
        
        let supabase_url = env::var("VITE_SUPABASE_URL")
            .map_err(|_| "VITE_SUPABASE_URL environment variable not set")?;
        
        let supabase_anon_key = env::var("VITE_SUPABASE_ANON_KEY")
            .map_err(|_| "VITE_SUPABASE_ANON_KEY environment variable not set")?;
        
        let supabase_service_role_key = env::var("SUPABASE_SERVICE_ROLE_KEY")
            .map_err(|_| "SUPABASE_SERVICE_ROLE_KEY environment variable not set")?;
        
        // Fixed: Added .json extension to JWKS endpoint
        let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", supabase_url.trim_end_matches('/'));
        
        Ok(TursoConfig {
            registry_db_url,
            registry_db_token,
            turso_api_token,
            turso_org,
            supabase: SupabaseConfig {
                url: supabase_url,
                anon_key: supabase_anon_key,
                service_role_key: supabase_service_role_key,
                jwks_url,
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseClaims {
    pub sub: String, // user_id
    pub email: Option<String>,
    pub aud: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}