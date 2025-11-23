use crate::server::turso::config::TursoConfig;
use crate::server::turso::schema;
use libsql::{Builder, Connection, Database};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub struct TursoClient {
    registry_db: Database,
    http_client: Client,
    config: Arc<TursoConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDatabaseEntry {
    pub user_id: String,
    pub email: String,
    pub db_name: String,
    pub db_url: String,
    pub db_token: String,
    pub storage_used_bytes: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TursoCreateDbResponse {
    database: TursoDatabaseInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct TursoDatabaseInfo {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Hostname")]
    hostname: String,
    #[serde(rename = "primaryRegion")]
    primary_region: String,
}

#[derive(Debug, Deserialize)]
struct TursoTokenResponse {
    jwt: String,
}

impl TursoClient {
    pub async fn new(config: TursoConfig) -> Result<Self, String> {
        // Connect to the central registry database
        let registry_db = Builder::new_remote(
            config.registry_db_url.clone(),
            config.registry_db_token.clone(),
        )
        .build()
        .await
        .map_err(|e| format!("Failed to connect to registry database: {}", e))?;

        let http_client = Client::new();

        // Run registry database migration
        let conn = registry_db
            .connect()
            .map_err(|e| format!("Failed to get registry database connection for migration: {}", e))?;
        
        // Initialize registry schema
        schema::initialize_registry_schema(&conn)
            .await
            .map_err(|e| format!("Failed to initialize registry schema: {}", e))?;
        
        // Add storage_used_bytes column if it doesn't exist (migration)
        conn.execute(
            "ALTER TABLE user_databases ADD COLUMN storage_used_bytes INTEGER DEFAULT 0",
            libsql::params![],
        ).await.ok(); // Ignore error if column already exists

        let config = Arc::new(config);

        Ok(Self {
            registry_db,
            http_client,
            config,
        })
    }

    /// Get a connection to the registry database
    pub async fn get_registry_connection(&self) -> Result<Connection, String> {
        self.registry_db
            .connect()
            .map_err(|e| format!("Failed to get registry database connection: {}", e))
    }

    /// Create a new user database in Turso
    pub async fn create_user_database(
        &self,
        user_id: &str,
        email: &str,
    ) -> Result<UserDatabaseEntry, String> {
        // Sanitize user_id for Turso requirements (numbers, lowercase letters, and dashes only)
        let sanitized_id = user_id
            .to_lowercase()
            .replace("_", "-");
        let db_name = format!("user-{}", sanitized_id);

        // Create database via Turso API
        let db_info = self.create_database_via_api(&db_name).await?;

        // Create auth token for the database
        let token = self.create_database_token(&db_name).await?;

        // Construct the database URL
        let db_url = format!("libsql://{}", db_info.hostname);

        // Initialize the database schema
        schema::initialize_user_database_schema(&db_url, &token)
            .await
            .map_err(|e| format!("Failed to initialize user database schema: {}", e))?;

        // Create user database entry
        let user_db_entry = UserDatabaseEntry {
            user_id: user_id.to_string(),
            email: email.to_string(),
            db_name: db_name.clone(),
            db_url: db_url.clone(),
            db_token: token,
            storage_used_bytes: Some(0),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        // Store in registry
        self.store_user_database_entry(&user_db_entry).await?;

        Ok(user_db_entry)
    }

    /// Create database via Turso API
    async fn create_database_via_api(&self, db_name: &str) -> Result<TursoDatabaseInfo, String> {
        let url = format!(
            "https://api.turso.tech/v1/organizations/{}/databases",
            self.config.turso_org
        );

        let mut payload = HashMap::new();
        payload.insert("name", db_name);
        payload.insert("group", "default");

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.turso_api_token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send database creation request: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_default();

            // Check if the error is because database already exists
            if error_text.contains("already exists") {
                return self.get_existing_database_info(db_name).await;
            }

            return Err(format!("Failed to create database: {}", error_text));
        }

        let create_response: TursoCreateDbResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse database creation response: {}", e))?;

        Ok(create_response.database)
    }

    /// Get existing database info from Turso API
    async fn get_existing_database_info(&self, db_name: &str) -> Result<TursoDatabaseInfo, String> {
        let url = format!(
            "https://api.turso.tech/v1/organizations/{}/databases/{}",
            self.config.turso_org, db_name
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.turso_api_token))
            .send()
            .await
            .map_err(|e| format!("Failed to get existing database info: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Failed to get existing database info: {}", error_text));
        }

        #[derive(Deserialize)]
        struct GetDbResponse {
            database: TursoDatabaseInfo,
        }

        let db_response: GetDbResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse existing database response: {}", e))?;

        Ok(db_response.database)
    }

    /// Create a database token for the given database
    pub async fn create_database_token(&self, db_name: &str) -> Result<String, String> {
        let url = format!(
            "https://api.turso.tech/v1/organizations/{}/databases/{}/auth/tokens",
            self.config.turso_org, db_name
        );

        let mut payload = HashMap::new();
        payload.insert("expiration", "never");
        payload.insert("authorization", "full-access");

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.turso_api_token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to create database token: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Failed to create database token: {}", error_text));
        }

        let token_response: TursoTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        Ok(token_response.jwt)
    }

    /// Store user database entry in registry
    async fn store_user_database_entry(&self, entry: &UserDatabaseEntry) -> Result<(), String> {
        let conn = self.get_registry_connection().await?;

        conn.execute(
            "INSERT OR REPLACE INTO user_databases
             (user_id, email, db_name, db_url, db_token, storage_used_bytes, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                entry.user_id.as_str(),
                entry.email.as_str(),
                entry.db_name.as_str(),
                entry.db_url.as_str(),
                entry.db_token.as_str(),
                entry.storage_used_bytes.unwrap_or(0),
                entry.created_at.as_str(),
                entry.updated_at.as_str(),
            ],
        )
        .await
        .map_err(|e| format!("Failed to store user database entry: {}", e))?;

        Ok(())
    }

    /// Get user database entry by user ID
    pub async fn get_user_database_entry(
        &self,
        user_id: &str,
    ) -> Result<UserDatabaseEntry, String> {
        let conn = self.get_registry_connection().await?;

        let mut rows = conn
            .prepare("SELECT user_id, email, db_name, db_url, db_token, storage_used_bytes, created_at, updated_at FROM user_databases WHERE user_id = ?")
            .await
            .map_err(|e| format!("Failed to prepare query: {}", e))?
            .query(libsql::params![user_id])
            .await
            .map_err(|e| format!("Failed to execute query: {}", e))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| format!("Failed to get row: {}", e))?
        {
            Ok(UserDatabaseEntry {
                user_id: row.get(0).map_err(|e| format!("Failed to get user_id: {}", e))?,
                email: row.get(1).map_err(|e| format!("Failed to get email: {}", e))?,
                db_name: row.get(2).map_err(|e| format!("Failed to get db_name: {}", e))?,
                db_url: row.get(3).map_err(|e| format!("Failed to get db_url: {}", e))?,
                db_token: row.get(4).map_err(|e| format!("Failed to get db_token: {}", e))?,
                storage_used_bytes: row.get(5).map_err(|e| format!("Failed to get storage_used_bytes: {}", e))?,
                created_at: row.get(6).map_err(|e| format!("Failed to get created_at: {}", e))?,
                updated_at: row.get(7).map_err(|e| format!("Failed to get updated_at: {}", e))?,
            })
        } else {
            Err(format!("User database not found for user_id: {}", user_id))
        }
    }

    /// Get user database connection
    pub async fn get_user_database_connection(
        &self,
        user_id: &str,
    ) -> Result<Connection, String> {
        let entry = self.get_user_database_entry(user_id).await?;
        
        let user_db = Builder::new_remote(entry.db_url, entry.db_token)
            .build()
            .await
            .map_err(|e| format!("Failed to connect to user database: {}", e))?;

        user_db
            .connect()
            .map_err(|e| format!("Failed to get user database connection: {}", e))
    }

    /// Health check for registry database
    pub async fn health_check(&self) -> Result<(), String> {
        let conn = self.get_registry_connection().await?;
        conn.execute("SELECT 1", libsql::params![])
            .await
            .map_err(|e| format!("Registry database health check failed: {}", e))?;
        Ok(())
    }

    /// Get current schema version from user database
    pub async fn get_user_schema_version(&self, user_id: &str) -> Result<Option<schema::SchemaVersion>, String> {
        let conn = self.get_user_database_connection(user_id).await?;
        schema::get_user_schema_version(&conn)
            .await
            .map_err(|e| format!("Failed to get schema version: {}", e))
    }

    /// Synchronize user database schema with current application schema
    pub async fn sync_user_database_schema(&self, user_id: &str) -> Result<(), String> {
        let conn = self.get_user_database_connection(user_id).await?;
        schema::sync_user_database_schema(&conn)
            .await
            .map_err(|e| format!("Failed to sync schema: {}", e))
    }
}
