<!-- 54563555-c97f-4c45-b159-3e4a4e810eba 2f9f95f2-32b3-4edb-8a8d-c2ae172c52fb -->
# Turso LibSQL Multi-Tenant Implementation Plan

## Architecture Overview

- **Registry Database**: Central Turso database storing user database credentials
- **User Databases**: Individual Turso database per user (created on signup)
- **Authentication**: Supabase JWT validation to extract user_id
- **Connection Management**: Dynamic connection to user's database based on JWT

## Implementation Steps

### 1. Add Dependencies to Cargo.toml

- Add `libsql = "0.9.24"` to SSR-only dependencies
- Add `chrono = { version = "0.4", features = ["serde"] }` for timestamps
- Add `uuid = { version = "1", features = ["v4", "serde"] }` for IDs
- Add `reqwest = { version = "0.12", features = ["json", "rustls-tls-webpki-roots"] }` for Turso API calls
- Add `anyhow = "1.0"` for error handling

### 2. Create Turso Module Structure

**Files to create:**

- `src/server/turso/mod.rs` - Module exports
- `src/server/turso/config.rs` - Configuration from environment variables
- `src/server/turso/client.rs` - Turso client for database management
- `src/server/turso/auth.rs` - Supabase JWT validation
- `src/server/turso/schema.rs` - Database schema definitions

### 3. Implement Configuration (config.rs)

- `TursoConfig` struct with:
  - `registry_db_url`: Central registry database URL
  - `registry_db_token`: Registry database auth token
  - `turso_api_token`: Turso API token for creating databases
  - `turso_org`: Turso organization name
  - `supabase`: Supabase config (URL, anon_key, service_role_key, jwks_url)
- `from_env()` method to load from environment variables
- `SupabaseClaims` struct matching Supabase JWT structure

### 4. Implement JWT Validation (auth.rs)

- `validate_supabase_jwt_token()` - Validate JWT using Supabase JWKS
- `get_supabase_user_id()` - Extract user_id from validated JWT
- `AuthError` enum for authentication errors
- Use `reqwest` to fetch JWKS from Supabase
- Validate JWT signature and expiration

### 5. Implement Turso Client (client.rs)

- `TursoClient` struct with:
  - Registry database connection
  - HTTP client for Turso API calls
  - Configuration
- Methods:
  - `new()` - Initialize with registry database connection
  - `create_user_database()` - Create new Turso database via API when user signs up
  - `get_user_database_connection()` - Get connection to user's database
  - `get_user_database_entry()` - Retrieve user database credentials from registry
  - `health_check()` - Verify registry database connectivity
- Use `libsql::Builder::new_remote()` for connections

### 6. Implement Schema Management (schema.rs)

- Define schema for user databases (cron_jobs table)
- `initialize_user_database_schema()` - Create tables in new user database
- Schema version tracking (optional but recommended)
- Cron jobs table structure:
  ```sql
  CREATE TABLE IF NOT EXISTS cron_jobs (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    schedule TEXT NOT NULL,
    command TEXT NOT NULL,
    enabled BOOLEAN DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  )
  ```


### 7. Create Registry Database Schema

- `user_databases` table in registry (matches Tradstry design):
  ```sql
  CREATE TABLE IF NOT EXISTS user_databases (
    user_id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    db_name TEXT NOT NULL,
    db_url TEXT NOT NULL,
    db_token TEXT NOT NULL,
    storage_used_bytes INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  )
  ```

- Create index on `email` for lookups

### 8. Update Server Module Structure

- Update `src/server/mod.rs` to export turso module
- Add `pub mod turso;` with `#[cfg(not(target_arch = "wasm32"))]`
- Re-export commonly used items

### 9. Create App State (Optional but Recommended)

- `AppState` struct in `src/server/turso/mod.rs`:
  - `TursoClient` instance
  - Configuration
  - Connection pooling/caching
- Initialize in `main.rs` and add to Actix app data

### 10. Implement User Signup Flow

- Update signup process to:

  1. Create user in Supabase (existing)
  2. Create Turso database via API
  3. Store credentials in registry database
  4. Initialize user database schema

- Create `src/server/service/user_service.rs`:
  - `create_user_database()` function
  - Handles Turso API calls for database creation
  - Stores entry in registry database

### 11. Create Database Models

- `src/server/models/cron_job.rs`:
  - `CronJob` struct with serde derives
  - Fields: id, user_id, name, schedule, command, enabled, timestamps
- `src/server/models/mod.rs` - Export models

### 12. Create Cron Job Service

- `src/server/service/cron_service.rs`:
  - `get_user_cron_jobs()` - Get all jobs for user
  - `create_cron_job()` - Create new job
  - `update_cron_job()` - Update existing job
  - `delete_cron_job()` - Delete job
  - All methods take user_id and use user's database connection

### 13. Create Server Functions

- `src/server/actions.rs`:
  - `#[server(GetCronJobs, "/api")]` - Get user's cron jobs
  - `#[server(CreateCronJob, "/api")]` - Create new cron job
  - `#[server(UpdateCronJob, "/api")]` - Update cron job
  - `#[server(DeleteCronJob, "/api")]` - Delete cron job
- Each function:

  1. Validates JWT and extracts user_id
  2. Gets user's database connection
  3. Performs operation on user's database

### 14. Update Main.rs

- Initialize `TursoClient` on startup
- Add to Actix app data: `.app_data(web::Data::new(turso_client))`
- Run registry database migration on startup
- Handle errors gracefully

### 15. Environment Variables

Add to `.env`:

```
# Turso Configuration
REGISTRY_DB_URL=libsql://registry-db-org.turso.io
REGISTRY_DB_TOKEN=your-registry-token
TURSO_API_TOKEN=your-turso-api-token
TURSO_ORG=your-org-name

# Supabase (already exists)
VITE_SUPABASE_URL=https://your-project.supabase.co
VITE_SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key
```

### 16. Error Handling

- Create custom error types for:
  - Database connection errors
  - JWT validation errors
  - Turso API errors
- Convert to `ServerFnError` in server functions
- Provide meaningful error messages

## Key Files to Create/Modify

**New Files:**

- `src/server/turso/mod.rs`
- `src/server/turso/config.rs`
- `src/server/turso/client.rs`
- `src/server/turso/auth.rs`
- `src/server/turso/schema.rs`
- `src/server/models/cron_job.rs`
- `src/server/models/mod.rs`
- `src/server/service/user_service.rs`
- `src/server/service/cron_service.rs`

**Modified Files:**

- `Cargo.toml` - Add dependencies
- `src/server/mod.rs` - Export turso module
- `src/server/actions.rs` - Add server functions
- `src/main.rs` - Initialize Turso client
- `.env` - Add Turso configuration

## Testing Considerations

- Test JWT validation with valid/invalid tokens
- Test user database creation on signup
- Test connection to user databases
- Test CRUD operations on cron jobs
- Handle edge cases (user database doesn't exist, connection failures)

## Security Considerations

- Always validate JWT before accessing user database
- Never trust client-provided user_id
- Use service role key only on server
- Store database tokens securely
- Implement connection pooling limits

### To-dos

- [ ] Add libsql, chrono, uuid, reqwest, and anyhow to Cargo.toml SSR dependencies
- [ ] Create src/server/turso/mod.rs with module structure and exports
- [ ] Implement TursoConfig and SupabaseConfig in src/server/turso/config.rs with from_env() methods
- [ ] Implement JWT validation in src/server/turso/auth.rs with Supabase JWKS validation
- [ ] Implement TursoClient in src/server/turso/client.rs with database creation and connection methods
- [ ] Implement schema management in src/server/turso/schema.rs with cron_jobs table definition and initialization
- [ ] Create registry database migration for user_databases table in client.rs initialization
- [ ] Create CronJob model in src/server/models/cron_job.rs with serde derives
- [ ] Create user_service.rs and cron_service.rs in src/server/service/ with CRUD operations
- [ ] Implement server functions in src/server/actions.rs for cron job CRUD operations
- [ ] Update src/main.rs to initialize TursoClient and add to Actix app data
- [ ] Update signup flow to create user database on account creation
- [ ] Add Turso environment variables to .env file with documentation