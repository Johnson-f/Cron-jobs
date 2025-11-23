#[cfg(not(target_arch = "wasm32"))]
pub mod cron_service;
#[cfg(not(target_arch = "wasm32"))]
pub mod user_service;

#[cfg(not(target_arch = "wasm32"))]
pub use cron_service::{create_cron_job, delete_cron_job, get_user_cron_jobs, update_cron_job};
#[cfg(not(target_arch = "wasm32"))]
pub use user_service::create_user_database;

