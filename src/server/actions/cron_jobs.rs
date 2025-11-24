use crate::server::actions::helpers::{get_user_id_from_request, get_turso_client};
use crate::server::models::{CronJob, CreateCronJobRequest, UpdateCronJobRequest};
use crate::server::service::{create_cron_job, delete_cron_job, get_user_cron_jobs, update_cron_job};
use leptos::prelude::ServerFnError;
use leptos::server;
use leptos_actix::extract;

#[server(GetCronJobs, "/api")]
pub async fn get_cron_jobs() -> Result<Vec<CronJob>, ServerFnError> {
    let req = extract::<actix_web::HttpRequest>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;
    
    let user_id = get_user_id_from_request(&req).await?;
    let client = get_turso_client(&req)?;
    
    get_user_cron_jobs(&client, &user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get cron jobs: {}", e)))
}

#[server(CreateCronJob, "/api")]
pub async fn create_cron_job_action(request: CreateCronJobRequest) -> Result<CronJob, ServerFnError> {
    let req = extract::<actix_web::HttpRequest>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;
    
    let user_id = get_user_id_from_request(&req).await?;
    let client = get_turso_client(&req)?;
    
    create_cron_job(&client, &user_id, request)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create cron job: {}", e)))
}

#[server(UpdateCronJob, "/api")]
pub async fn update_cron_job_action(
    job_id: String,
    request: UpdateCronJobRequest,
) -> Result<CronJob, ServerFnError> {
    let req = extract::<actix_web::HttpRequest>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;
    
    let user_id = get_user_id_from_request(&req).await?;
    let client = get_turso_client(&req)?;
    
    update_cron_job(&client, &user_id, &job_id, request)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update cron job: {}", e)))
}

#[server(DeleteCronJob, "/api")]
pub async fn delete_cron_job_action(job_id: String) -> Result<(), ServerFnError> {
    let req = extract::<actix_web::HttpRequest>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;
    
    let user_id = get_user_id_from_request(&req).await?;
    let client = get_turso_client(&req)?;
    
    delete_cron_job(&client, &user_id, &job_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete cron job: {}", e)))
}

