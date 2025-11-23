#[cfg(not(target_arch = "wasm32"))]
pub mod auth;
#[cfg(not(target_arch = "wasm32"))]
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod schema;

#[cfg(not(target_arch = "wasm32"))]
pub use auth::{AuthError, get_supabase_user_id, validate_supabase_jwt_token};
#[cfg(not(target_arch = "wasm32"))]
pub use client::{TursoClient, UserDatabaseEntry};
#[cfg(not(target_arch = "wasm32"))]
pub use config::{SupabaseConfig, SupabaseClaims, TursoConfig};
#[cfg(not(target_arch = "wasm32"))]
pub use schema::{SchemaVersion, TableSchema, sync_user_database_schema, get_user_schema_version};

