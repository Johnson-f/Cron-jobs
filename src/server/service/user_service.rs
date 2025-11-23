use crate::server::turso::{TursoClient, UserDatabaseEntry};

pub async fn create_user_database(
    client: &TursoClient,
    user_id: &str,
    email: &str,
) -> Result<UserDatabaseEntry, String> {
    client.create_user_database(user_id, email).await
}

