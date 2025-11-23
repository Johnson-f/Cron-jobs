#[cfg(not(target_arch = "wasm32"))]
pub mod cron_job;

#[cfg(not(target_arch = "wasm32"))]
pub use cron_job::{CronJob, CreateCronJobRequest, UpdateCronJobRequest};

