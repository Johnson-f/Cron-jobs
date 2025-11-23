use crate::server::actions::helpers::{get_user_id_from_request, get_turso_client};
use crate::server::service::create_user_database;
use leptos::*;
use leptos_actix::extract;

#[server(CreateUserDatabase, "/api")]
pub async fn create_user_database_action(email: String) -> Result<(), ServerFnError> {
    let req = extract::<actix_web::HttpRequest>().await
        .map_err(|e| ServerFnError::new(format!("Failed to extract request: {}", e)))?;
    
    let user_id = get_user_id_from_request(&req).await?;
    let client = get_turso_client(&req)?;
    
    create_user_database(&client, &user_id, &email)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create user database: {}", e)))?;
    
    Ok(())
}

