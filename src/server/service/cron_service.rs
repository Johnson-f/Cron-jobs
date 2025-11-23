use crate::server::models::{CronJob, CreateCronJobRequest, UpdateCronJobRequest};
use crate::server::turso::TursoClient;
use libsql::Connection;
use uuid::Uuid;

pub async fn get_user_cron_jobs(
    client: &TursoClient,
    user_id: &str,
) -> Result<Vec<CronJob>, String> {
    let conn = client.get_user_database_connection(user_id).await?;

    let mut rows = conn
        .prepare("SELECT id, user_id, name, schedule, command, enabled, created_at, updated_at FROM cron_jobs WHERE user_id = ? ORDER BY created_at DESC")
        .await
        .map_err(|e| format!("Failed to prepare query: {}", e))?
        .query(libsql::params![user_id])
        .await
        .map_err(|e| format!("Failed to query cron jobs: {}", e))?;

    let mut jobs = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| format!("Failed to get row: {}", e))?
    {
        let job = CronJob {
            id: row.get::<String>(0).map_err(|e| format!("Failed to get id: {}", e))?,
            user_id: row.get::<String>(1).map_err(|e| format!("Failed to get user_id: {}", e))?,
            name: row.get::<String>(2).map_err(|e| format!("Failed to get name: {}", e))?,
            schedule: row.get::<String>(3).map_err(|e| format!("Failed to get schedule: {}", e))?,
            command: row.get::<String>(4).map_err(|e| format!("Failed to get command: {}", e))?,
            enabled: row.get::<i64>(5).map_err(|e| format!("Failed to get enabled: {}", e))? != 0,
            created_at: None, // TODO: Parse timestamp if needed
            updated_at: None,  // TODO: Parse timestamp if needed
        };
        jobs.push(job);
    }

    Ok(jobs)
}

pub async fn create_cron_job(
    client: &TursoClient,
    user_id: &str,
    request: CreateCronJobRequest,
) -> Result<CronJob, String> {
    let conn = client.get_user_database_connection(user_id).await?;

    let id = Uuid::new_v4().to_string();
    let enabled = request.enabled.unwrap_or(true);

    conn.execute(
        "INSERT INTO cron_jobs (id, user_id, name, schedule, command, enabled)
         VALUES (?, ?, ?, ?, ?, ?)",
        libsql::params![
            id.as_str(),
            user_id,
            request.name.as_str(),
            request.schedule.as_str(),
            request.command.as_str(),
            if enabled { 1 } else { 0 },
        ],
    )
    .await
    .map_err(|e| format!("Failed to create cron job: {}", e))?;

    Ok(CronJob {
        id,
        user_id: user_id.to_string(),
        name: request.name,
        schedule: request.schedule,
        command: request.command,
        enabled,
        created_at: None,
        updated_at: None,
    })
}

pub async fn update_cron_job(
    client: &TursoClient,
    user_id: &str,
    job_id: &str,
    request: UpdateCronJobRequest,
) -> Result<CronJob, String> {
    let conn = client.get_user_database_connection(user_id).await?;

    // First get the existing job to use current values for fields not being updated
    let mut rows = conn
        .prepare("SELECT id, user_id, name, schedule, command, enabled FROM cron_jobs WHERE id = ? AND user_id = ?")
        .await
        .map_err(|e| format!("Failed to prepare query: {}", e))?
        .query(libsql::params![job_id, user_id])
        .await
        .map_err(|e| format!("Failed to get existing cron job: {}", e))?;

    let existing_row = rows
        .next()
        .await
        .map_err(|e| format!("Failed to get row: {}", e))?
        .ok_or_else(|| "Cron job not found or access denied".to_string())?;

    let current_name: String = existing_row.get(2).map_err(|e| format!("Failed to get name: {}", e))?;
    let current_schedule: String = existing_row.get(3).map_err(|e| format!("Failed to get schedule: {}", e))?;
    let current_command: String = existing_row.get(4).map_err(|e| format!("Failed to get command: {}", e))?;
    let current_enabled: i64 = existing_row.get(5).map_err(|e| format!("Failed to get enabled: {}", e))?;
    let current_enabled = current_enabled != 0;

    // Use new values if provided, otherwise keep existing
    let name = request.name.as_ref().unwrap_or(&current_name);
    let schedule = request.schedule.as_ref().unwrap_or(&current_schedule);
    let command = request.command.as_ref().unwrap_or(&current_command);
    let enabled = request.enabled.unwrap_or(current_enabled);

    // Update the job
    conn.execute(
        "UPDATE cron_jobs SET name = ?, schedule = ?, command = ?, enabled = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND user_id = ?",
        libsql::params![
            name.as_str(),
            schedule.as_str(),
            command.as_str(),
            if enabled { 1 } else { 0 },
            job_id,
            user_id,
        ],
    )
    .await
    .map_err(|e| format!("Failed to update cron job: {}", e))?;

    // Fetch and return updated job
    let mut rows = conn
        .prepare("SELECT id, user_id, name, schedule, command, enabled, created_at, updated_at FROM cron_jobs WHERE id = ? AND user_id = ?")
        .await
        .map_err(|e| format!("Failed to prepare query: {}", e))?
        .query(libsql::params![job_id, user_id])
        .await
        .map_err(|e| format!("Failed to fetch updated cron job: {}", e))?;

    let row = rows
        .next()
        .await
        .map_err(|e| format!("Failed to get row: {}", e))?
        .ok_or_else(|| "Cron job not found after update".to_string())?;

    Ok(CronJob {
        id: row.get::<String>(0).map_err(|e| format!("Failed to get id: {}", e))?,
        user_id: row.get::<String>(1).map_err(|e| format!("Failed to get user_id: {}", e))?,
        name: row.get::<String>(2).map_err(|e| format!("Failed to get name: {}", e))?,
        schedule: row.get::<String>(3).map_err(|e| format!("Failed to get schedule: {}", e))?,
        command: row.get::<String>(4).map_err(|e| format!("Failed to get command: {}", e))?,
        enabled: row.get::<i64>(5).map_err(|e| format!("Failed to get enabled: {}", e))? != 0,
        created_at: None,
        updated_at: None,
    })
}

pub async fn delete_cron_job(
    client: &TursoClient,
    user_id: &str,
    job_id: &str,
) -> Result<(), String> {
    let conn = client.get_user_database_connection(user_id).await?;

    conn.execute(
        "DELETE FROM cron_jobs WHERE id = ? AND user_id = ?",
        libsql::params![job_id, user_id],
    )
    .await
    .map_err(|e| format!("Failed to delete cron job: {}", e))?;

    Ok(())
}

