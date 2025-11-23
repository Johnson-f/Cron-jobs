use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub schedule: String,
    pub command: String,
    pub enabled: bool,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCronJobRequest {
    pub name: String,
    pub schedule: String,
    pub command: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCronJobRequest {
    pub name: Option<String>,
    pub schedule: Option<String>,
    pub command: Option<String>,
    pub enabled: Option<bool>,
}

