use libsql::{Builder, Connection};
use log::info;
use std::collections::{HashMap, HashSet};

/// Schema version information
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SchemaVersion {
    pub version: String,
    pub description: String,
    pub created_at: String,
}

/// Table schema information for comparison
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
    pub triggers: Vec<TriggerInfo>,
}

/// Column information for schema comparison
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
}

/// Index information for schema comparison
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

/// Trigger information for schema comparison
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct TriggerInfo {
    pub name: String,
    pub table_name: String,
    pub event: String,
    pub timing: String,
    pub action: String,
}

/// Initialize user database with cron jobs schema
pub async fn initialize_user_database_schema(db_url: &str, token: &str) -> Result<(), String> {
    info!("Initializing cron jobs schema for database: {}", db_url);

    let user_db = Builder::new_remote(db_url.to_string(), token.to_string())
        .build()
        .await
        .map_err(|e| format!("Failed to build user database connection: {}", e))?;
    
    let conn = user_db
        .connect()
        .map_err(|e| format!("Failed to connect to user database: {}", e))?;

    // Initialize schema version table first
    initialize_schema_version_table(&conn)
        .await
        .map_err(|e| format!("Failed to initialize schema version table: {}", e))?;

    // Get expected schema
    let expected_schema = get_expected_schema();
    let expected_version = get_current_schema_version();

    // Create all tables
    for table_schema in &expected_schema {
        create_table(&conn, table_schema)
            .await
            .map_err(|e| format!("Failed to create table {}: {}", table_schema.name, e))?;
        
        // Ensure indexes
        ensure_indexes(&conn, table_schema)
            .await
            .map_err(|e| format!("Failed to ensure indexes for {}: {}", table_schema.name, e))?;
        
        // Ensure triggers
        ensure_triggers(&conn, table_schema)
            .await
            .map_err(|e| format!("Failed to ensure triggers for {}: {}", table_schema.name, e))?;
    }

    // Set initial schema version
    update_schema_version(&conn, &expected_version)
        .await
        .map_err(|e| format!("Failed to set schema version: {}", e))?;

    info!("Cron jobs schema initialized successfully");
    Ok(())
}

/// Current schema version (increment this when schema changes)
pub fn get_current_schema_version() -> SchemaVersion {
    SchemaVersion {
        version: "0.0.1".to_string(),
        description: "Initial cron jobs schema with version tracking".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Expected schema for synchronization
pub fn get_expected_schema() -> Vec<TableSchema> {
    vec![
        TableSchema {
            name: "cron_jobs".to_string(),
            columns: vec![
                ColumnInfo { name: "id".to_string(), data_type: "TEXT".to_string(), is_nullable: false, default_value: None, is_primary_key: true },
                ColumnInfo { name: "user_id".to_string(), data_type: "TEXT".to_string(), is_nullable: false, default_value: None, is_primary_key: false },
                ColumnInfo { name: "name".to_string(), data_type: "TEXT".to_string(), is_nullable: false, default_value: None, is_primary_key: false },
                ColumnInfo { name: "schedule".to_string(), data_type: "TEXT".to_string(), is_nullable: false, default_value: None, is_primary_key: false },
                ColumnInfo { name: "command".to_string(), data_type: "TEXT".to_string(), is_nullable: false, default_value: None, is_primary_key: false },
                ColumnInfo { name: "enabled".to_string(), data_type: "BOOLEAN".to_string(), is_nullable: false, default_value: Some("1".to_string()), is_primary_key: false },
                ColumnInfo { name: "created_at".to_string(), data_type: "TIMESTAMP".to_string(), is_nullable: false, default_value: Some("CURRENT_TIMESTAMP".to_string()), is_primary_key: false },
                ColumnInfo { name: "updated_at".to_string(), data_type: "TIMESTAMP".to_string(), is_nullable: false, default_value: Some("CURRENT_TIMESTAMP".to_string()), is_primary_key: false },
            indexes: vec![
                IndexInfo {
                    name: "idx_cron_jobs_user_id".to_string(),
                    table_name: "cron_jobs".to_string(),
                    columns: vec!["user_id".to_string()],
                    is_unique: false,
                },
                IndexInfo {
                    name: "idx_cron_jobs_enabled".to_string(),
                    table_name: "cron_jobs".to_string(),
                    columns: vec!["enabled".to_string()],
                    is_unique: false,
                },
            ],
            triggers: vec![
                TriggerInfo {
                    name: "update_cron_jobs_timestamp".to_string(),
                    table_name: "cron_jobs".to_string(),
                    event: "UPDATE".to_string(),
                    timing: "AFTER".to_string(),
                    action: "UPDATE cron_jobs SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id".to_string(),
                },
            ],
        },
    ]
}

/// Initialize the schema version table if needed
pub async fn initialize_schema_version_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version TEXT NOT NULL,
            description TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
        libsql::params![],
    )
    .await
    .map_err(|e| format!("Failed to create schema_version table: {}", e))?;
    Ok(())
}

/// Update schema version in the database
pub async fn update_schema_version(conn: &Connection, version: &SchemaVersion) -> Result<(), String> {
    conn.execute(
        "INSERT INTO schema_version (version, description, created_at) VALUES (?, ?, ?)",
        libsql::params![
            version.version.as_str(),
            version.description.as_str(),
            version.created_at.as_str()
        ],
    )
    .await
    .map_err(|e| format!("Failed to update schema version: {}", e))?;
    Ok(())
}

/// Ensure indexes for a table
pub async fn ensure_indexes(conn: &Connection, table_schema: &TableSchema) -> Result<(), String> {
    for index in &table_schema.indexes {
        let mut rows = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name=?")
            .await
            .map_err(|e| format!("Failed to prepare index check query: {}", e))?
            .query(libsql::params![index.name.as_str()])
            .await
            .map_err(|e| format!("Failed to check index existence: {}", e))?;

        if rows
            .next()
            .await
            .map_err(|e| format!("Failed to get index check result: {}", e))?
            .is_none()
        {
            let create_index_sql = format!(
                "CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                index.name,
                index.table_name,
                index.columns.join(", ")
            );
            conn.execute(&create_index_sql, libsql::params![])
                .await
                .map_err(|e| format!("Failed to create index {}: {}", index.name, e))?;
        }
    }
    Ok(())
}

/// Ensure triggers for a table
pub async fn ensure_triggers(conn: &Connection, table_schema: &TableSchema) -> Result<(), String> {
    for trigger in &table_schema.triggers {
        let mut rows = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='trigger' AND name=?")
            .await
            .map_err(|e| format!("Failed to prepare trigger check query: {}", e))?
            .query(libsql::params![trigger.name.as_str()])
            .await
            .map_err(|e| format!("Failed to check trigger existence: {}", e))?;

        if rows
            .next()
            .await
            .map_err(|e| format!("Failed to get trigger check result: {}", e))?
            .is_none()
        {
            let create_trigger_sql = format!(
                "CREATE TRIGGER IF NOT EXISTS {} {} {} ON {} FOR EACH ROW BEGIN {}; END",
                trigger.name, trigger.timing, trigger.event, trigger.table_name, trigger.action
            );
            conn.execute(&create_trigger_sql, libsql::params![])
                .await
                .map_err(|e| format!("Failed to create trigger {}: {}", trigger.name, e))?;
        }
    }
    Ok(())
}

/// Get list of current tables in the database
pub async fn get_current_tables(conn: &Connection) -> Result<Vec<String>, String> {
    let mut tables = Vec::new();
    let mut rows = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        .await
        .map_err(|e| format!("Failed to prepare table list query: {}", e))?
        .query(libsql::params![])
        .await
        .map_err(|e| format!("Failed to execute table list query: {}", e))?;

    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| format!("Failed to get table row: {}", e))?
    {
        let table_name: String = row
            .get(0)
            .map_err(|e| format!("Failed to get table name: {}", e))?;
        tables.push(table_name);
    }
    Ok(tables)
}

/// Create a table based on schema definition
pub async fn create_table(conn: &Connection, table_schema: &TableSchema) -> Result<(), String> {
    let mut create_sql = format!("CREATE TABLE IF NOT EXISTS {} (", table_schema.name);
    let primary_keys: Vec<String> = table_schema
        .columns
        .iter()
        .filter(|c| c.is_primary_key)
        .map(|c| c.name.clone())
        .collect();

    let column_definitions: Vec<String> = table_schema
        .columns
        .iter()
        .map(|col| {
            let mut def = format!("{} {}", col.name, col.data_type);
            if col.is_primary_key && primary_keys.len() == 1 {
                if col.data_type.to_uppercase().contains("INTEGER") {
                    def.push_str(" PRIMARY KEY AUTOINCREMENT");
                } else {
                    def.push_str(" PRIMARY KEY");
                }
            } else if !col.is_nullable {
                def.push_str(" NOT NULL");
            }
            if let Some(default) = &col.default_value {
                def.push_str(&format!(" DEFAULT {}", default));
            }
            def
        })
        .collect();

    create_sql.push_str(&column_definitions.join(", "));
    if primary_keys.len() > 1 {
        create_sql.push_str(&format!(", PRIMARY KEY ({})", primary_keys.join(", ")));
    }
    create_sql.push(')');

    conn.execute(&create_sql, libsql::params![])
        .await
        .map_err(|e| format!("Failed to create table {}: {}", table_schema.name, e))?;
    Ok(())
}

/// Get current columns for a table
pub async fn get_table_columns(conn: &Connection, table_name: &str) -> Result<Vec<ColumnInfo>, String> {
    let mut columns = Vec::new();
    let mut rows = conn
        .prepare(&format!("PRAGMA table_info({})", table_name))
        .await
        .map_err(|e| format!("Failed to prepare pragma query: {}", e))?
        .query(libsql::params![])
        .await
        .map_err(|e| format!("Failed to execute pragma query: {}", e))?;

    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| format!("Failed to get column row: {}", e))?
    {
        columns.push(ColumnInfo {
            name: row.get(1).map_err(|e| format!("Failed to get column name: {}", e))?,
            data_type: row.get(2).map_err(|e| format!("Failed to get data type: {}", e))?,
            is_nullable: row.get::<i32>(3).map_err(|e| format!("Failed to get nullable: {}", e))? == 0,
            default_value: row.get(4).map_err(|e| format!("Failed to get default: {}", e))?,
            is_primary_key: row.get::<i32>(5).map_err(|e| format!("Failed to get primary key: {}", e))? == 1,
        });
    }
    Ok(columns)
}

/// Update table schema if needed
pub async fn update_table_schema(conn: &Connection, table_schema: &TableSchema) -> Result<(), String> {
    let current_columns = get_table_columns(conn, &table_schema.name)
        .await
        .map_err(|e| format!("Failed to get current columns: {}", e))?;

    // Handle column renames: map old column names to new ones
    let mut column_rename_map: HashMap<String, String> = HashMap::new();
    
    // Add column rename mappings for cron_jobs table
    if table_schema.name == "cron_jobs" {
        // Example: If you need to rename a column in the future, add it here:
        // let has_old_column = current_columns.iter().any(|c| c.name == "old_column_name");
        // let has_new_column = current_columns.iter().any(|c| c.name == "new_column_name");
        // if has_old_column && !has_new_column {
        //     column_rename_map.insert("old_column_name".to_string(), "new_column_name".to_string());
        // }
        
        // Future rename examples (uncomment and modify as needed):
        // - If renaming "schedule" to "cron_schedule":
        // let has_schedule = current_columns.iter().any(|c| c.name == "schedule");
        // let has_cron_schedule = current_columns.iter().any(|c| c.name == "cron_schedule");
        // if has_schedule && !has_cron_schedule {
        //     column_rename_map.insert("schedule".to_string(), "cron_schedule".to_string());
        // }
    }

    // Add missing columns (skip if they're being renamed from an old column)
    for expected_col in &table_schema.columns {
        let is_renamed = column_rename_map
            .values()
            .any(|new_name| new_name == &expected_col.name);
        if !current_columns
            .iter()
            .any(|c| c.name == expected_col.name)
            && !is_renamed
        {
            let mut alter_sql = format!(
                "ALTER TABLE {} ADD COLUMN {} {}",
                table_schema.name, expected_col.name, expected_col.data_type
            );

            // For NOT NULL columns without explicit defaults, provide appropriate defaults
            if !expected_col.is_nullable {
                if let Some(default) = &expected_col.default_value {
                    alter_sql.push_str(&format!(" NOT NULL DEFAULT {}", default));
                } else {
                    // Provide default values for NOT NULL columns based on data type
                    match expected_col.data_type.to_uppercase().as_str() {
                        "TEXT" | "VARCHAR" => alter_sql.push_str(" NOT NULL DEFAULT ''"),
                        "INTEGER" => alter_sql.push_str(" NOT NULL DEFAULT 0"),
                        "REAL" | "DECIMAL" => alter_sql.push_str(" NOT NULL DEFAULT 0.0"),
                        "BOOLEAN" => alter_sql.push_str(" NOT NULL DEFAULT false"),
                        "DATE" => alter_sql.push_str(" NOT NULL DEFAULT '1970-01-01'"),
                        "TIME" => alter_sql.push_str(" NOT NULL DEFAULT '00:00:00'"),
                        "TIMESTAMP" => alter_sql.push_str(" NOT NULL DEFAULT CURRENT_TIMESTAMP"),
                        _ => alter_sql.push_str(" NOT NULL DEFAULT ''"),
                    }
                }
            } else if let Some(default) = &expected_col.default_value {
                alter_sql.push_str(&format!(" DEFAULT {}", default));
            }

            conn.execute(&alter_sql, libsql::params![])
                .await
                .map_err(|e| format!("Failed to add column {}: {}", expected_col.name, e))?;
        }
    }

    // Remove columns that are not in the expected schema (excluding renamed columns)
    let expected_names: HashSet<String> = table_schema
        .columns
        .iter()
        .map(|c| c.name.clone())
        .collect();
    let renamed_old_names: HashSet<String> = column_rename_map.keys().cloned().collect();
    let columns_to_remove: Vec<String> = current_columns
        .iter()
        .filter(|c| {
            !expected_names.contains(&c.name)
                && !renamed_old_names.contains(&c.name)
                && !c.is_primary_key
        })
        .map(|c| c.name.clone())
        .collect();

    // Recreate table if we need to remove columns OR rename columns
    if !columns_to_remove.is_empty() || !column_rename_map.is_empty() {
        if !column_rename_map.is_empty() {
            info!(
                "Renaming columns in {}: {:?}",
                table_schema.name, column_rename_map
            );
        }
        if !columns_to_remove.is_empty() {
            info!(
                "Removing obsolete columns from {}: {:?}",
                table_schema.name, columns_to_remove
            );
        }

        // SQLite doesn't support DROP COLUMN or RENAME COLUMN directly, so we need to recreate the table
        // First, create a backup of existing data
        let backup_table = format!("{}_backup", table_schema.name);
        conn.execute(
            &format!("CREATE TABLE {} AS SELECT * FROM {}", backup_table, table_schema.name),
            libsql::params![],
        )
        .await
        .map_err(|e| format!("Failed to create backup table: {}", e))?;

        // Drop the original table
        conn.execute(
            &format!("DROP TABLE {}", table_schema.name),
            libsql::params![],
        )
        .await
        .map_err(|e| format!("Failed to drop original table: {}", e))?;

        // Recreate the table with the correct schema
        create_table(conn, table_schema)
            .await
            .map_err(|e| format!("Failed to recreate table: {}", e))?;

        // Copy data back, handling column renames
        let mut select_columns = Vec::new();
        let mut insert_columns = Vec::new();

        for current_col in &current_columns {
            if let Some(new_name) = column_rename_map.get(&current_col.name) {
                // This column was renamed
                if expected_names.contains(new_name) {
                    select_columns.push(current_col.name.clone());
                    insert_columns.push(new_name.clone());
                }
            } else if expected_names.contains(&current_col.name) {
                // Column exists in both schemas with same name
                select_columns.push(current_col.name.clone());
                insert_columns.push(current_col.name.clone());
            }
        }

        if !insert_columns.is_empty() {
            let select_str = select_columns.join(", ");
            let insert_str = insert_columns.join(", ");
            conn.execute(
                &format!(
                    "INSERT INTO {} ({}) SELECT {} FROM {}",
                    table_schema.name, insert_str, select_str, backup_table
                ),
                libsql::params![],
            )
            .await
            .map_err(|e| format!("Failed to copy data back: {}", e))?;
        }

        // Drop the backup table
        conn.execute(
            &format!("DROP TABLE {}", backup_table),
            libsql::params![],
        )
        .await
        .map_err(|e| format!("Failed to drop backup table: {}", e))?;

        // Recreate indexes and triggers
        ensure_indexes(conn, table_schema)
            .await
            .map_err(|e| format!("Failed to recreate indexes: {}", e))?;
        ensure_triggers(conn, table_schema)
            .await
            .map_err(|e| format!("Failed to recreate triggers: {}", e))?;
    }

    Ok(())
}

/// Get current schema version from user database
pub async fn get_user_schema_version(conn: &Connection) -> Result<Option<SchemaVersion>, String> {
    // Check if schema_version table exists
    let mut rows = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='schema_version'")
        .await
        .map_err(|e| format!("Failed to check schema_version table: {}", e))?
        .query(libsql::params![])
        .await
        .map_err(|e| format!("Failed to execute schema_version check: {}", e))?;

    if rows
        .next()
        .await
        .map_err(|e| format!("Failed to get schema_version check result: {}", e))?
        .is_none()
    {
        return Ok(None); // No schema version table, means old schema
    }

    // Get the latest schema version
    let mut rows = conn
        .prepare("SELECT version, description, created_at FROM schema_version ORDER BY created_at DESC LIMIT 1")
        .await
        .map_err(|e| format!("Failed to prepare schema version query: {}", e))?
        .query(libsql::params![])
        .await
        .map_err(|e| format!("Failed to execute schema version query: {}", e))?;

    if let Some(row) = rows
        .next()
        .await
        .map_err(|e| format!("Failed to get schema version row: {}", e))?
    {
        Ok(Some(SchemaVersion {
            version: row.get(0).map_err(|e| format!("Failed to get version: {}", e))?,
            description: row.get(1).map_err(|e| format!("Failed to get description: {}", e))?,
            created_at: row.get(2).map_err(|e| format!("Failed to get created_at: {}", e))?,
        }))
    } else {
        Ok(None)
    }
}

/// Synchronize user database schema with current application schema
pub async fn sync_user_database_schema(conn: &Connection) -> Result<(), String> {
    info!("Starting schema synchronization");

    let current_version = get_user_schema_version(conn)
        .await
        .map_err(|e| format!("Failed to get current schema version: {}", e))?;
    let expected_version = get_current_schema_version();
    let expected_schema = get_expected_schema();

    // If no version exists, this is a new database or very old one
    if current_version.is_none() {
        info!("No schema version found, initializing with current schema");
        initialize_schema_version_table(conn)
            .await
            .map_err(|e| format!("Failed to initialize schema version table: {}", e))?;
        apply_schema_migrations(conn, &expected_schema)
            .await
            .map_err(|e| format!("Failed to apply schema migrations: {}", e))?;
        update_schema_version(conn, &expected_version)
            .await
            .map_err(|e| format!("Failed to update schema version: {}", e))?;
        return Ok(());
    }

    let current_version = current_version.unwrap();

    // Compare versions
    if current_version.version != expected_version.version {
        info!(
            "Schema version mismatch: current={}, expected={}",
            current_version.version, expected_version.version
        );

        // Apply schema migrations
        apply_schema_migrations(conn, &expected_schema)
            .await
            .map_err(|e| format!("Failed to apply schema migrations: {}", e))?;
        update_schema_version(conn, &expected_version)
            .await
            .map_err(|e| format!("Failed to update schema version: {}", e))?;

        info!("Schema synchronized successfully");
    } else {
        info!("Schema is up to date");
    }

    Ok(())
}

/// Apply schema migrations to bring database up to current schema
/// This function makes schema.rs the source of truth - it will drop any tables
/// that exist in the database but are not in the expected schema
pub async fn apply_schema_migrations(
    conn: &Connection,
    expected_schema: &[TableSchema],
) -> Result<(), String> {
    info!("Applying schema migrations");

    // Get list of expected table names (source of truth)
    let expected_table_names: HashSet<String> = expected_schema
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Also include system tables that should never be dropped
    let protected_tables: std::collections::HashSet<String> = [
        "schema_version".to_string(),
        "sqlite_sequence".to_string(), // SQLite internal table
    ]
    .iter()
    .cloned()
    .collect();

    // Get current tables in database
    let current_tables = get_current_tables(conn)
        .await
        .map_err(|e| format!("Failed to get current tables: {}", e))?;

    // Drop tables that exist in database but are not in expected schema
    // Temporarily disable foreign key constraints to allow dropping tables with dependencies
    conn.execute("PRAGMA foreign_keys = OFF", libsql::params![])
        .await
        .map_err(|e| format!("Failed to disable foreign keys: {}", e))?;

    for table_name in &current_tables {
        if !expected_table_names.contains(table_name) && !protected_tables.contains(table_name) {
            info!(
                "Dropping table '{}' - not in expected schema (schema.rs is source of truth)",
                table_name
            );

            // Drop all indexes for this table first
            let mut index_rows = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name=? AND name NOT LIKE 'sqlite_%'")
                .await
                .map_err(|e| format!("Failed to prepare index drop query: {}", e))?
                .query(libsql::params![table_name.as_str()])
                .await
                .map_err(|e| format!("Failed to get indexes: {}", e))?;

            while let Some(index_row) = index_rows
                .next()
                .await
                .map_err(|e| format!("Failed to get index row: {}", e))?
            {
                let index_name: String = index_row
                    .get(0)
                    .map_err(|e| format!("Failed to get index name: {}", e))?;
                conn.execute(&format!("DROP INDEX IF EXISTS {}", index_name), libsql::params![])
                    .await
                    .map_err(|e| format!("Failed to drop index {}: {}", index_name, e))?;
            }

            // Drop all triggers for this table
            let mut trigger_rows = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='trigger' AND tbl_name=?")
                .await
                .map_err(|e| format!("Failed to prepare trigger drop query: {}", e))?
                .query(libsql::params![table_name.as_str()])
                .await
                .map_err(|e| format!("Failed to get triggers: {}", e))?;

            while let Some(trigger_row) = trigger_rows
                .next()
                .await
                .map_err(|e| format!("Failed to get trigger row: {}", e))?
            {
                let trigger_name: String = trigger_row
                    .get(0)
                    .map_err(|e| format!("Failed to get trigger name: {}", e))?;
                conn.execute(
                    &format!("DROP TRIGGER IF EXISTS {}", trigger_name),
                    libsql::params![],
                )
                .await
                .map_err(|e| format!("Failed to drop trigger {}: {}", trigger_name, e))?;
            }

            // Drop the table
            conn.execute(&format!("DROP TABLE IF EXISTS {}", table_name), libsql::params![])
                .await
                .map_err(|e| format!("Failed to drop table {}: {}", table_name, e))?;
        }
    }

    // Re-enable foreign key constraints
    conn.execute("PRAGMA foreign_keys = ON", libsql::params![])
        .await
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

    // Update or create expected tables
    for table_schema in expected_schema {
        // Check if table exists
        let table_exists = current_tables.contains(&table_schema.name);

        if table_exists {
            // Update existing table schema
            update_table_schema(conn, table_schema)
                .await
                .map_err(|e| format!("Failed to update table {}: {}", table_schema.name, e))?;
        } else {
            // Create new table
            create_table(conn, table_schema)
                .await
                .map_err(|e| format!("Failed to create table {}: {}", table_schema.name, e))?;
        }

        // Ensure indexes are in place
        ensure_indexes(conn, table_schema)
            .await
            .map_err(|e| format!("Failed to ensure indexes for {}: {}", table_schema.name, e))?;

        // Ensure triggers are in place
        ensure_triggers(conn, table_schema)
            .await
            .map_err(|e| format!("Failed to ensure triggers for {}: {}", table_schema.name, e))?;
    }

    info!("Schema migrations applied successfully");
    Ok(())
}

pub async fn initialize_registry_schema(conn: &Connection) -> Result<(), String> {
    let create_table_sql = r#"
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
    "#;

    conn.execute(create_table_sql, libsql::params![])
        .await
        .map_err(|e| format!("Failed to create user_databases table: {}", e))?;

    // Create index on email for lookups
    let create_index_sql = r#"
        CREATE INDEX IF NOT EXISTS idx_user_databases_email ON user_databases(email)
    "#;

    conn.execute(create_index_sql, libsql::params![])
        .await
        .map_err(|e| format!("Failed to create email index: {}", e))?;

    Ok(())
}
