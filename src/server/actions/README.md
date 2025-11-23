# Actions Module

## Purpose

The `actions/` folder contains **Leptos Server Functions** - type-safe API endpoints that can be called directly from client-side components. These are the primary way to handle server-side logic in a Leptos application.

## What are Server Functions?

Server functions are Rust functions marked with the `#[server]` macro that:
- Automatically become HTTP endpoints
- Provide type-safe communication between client and server
- Handle serialization/deserialization automatically
- Can be called directly from client components like regular functions

## Structure

```
actions/
├── mod.rs          # Module exports - re-exports all public server functions
├── helpers.rs      # Shared helper functions (JWT validation, client access)
├── cron_jobs.rs    # Cron job CRUD operations
└── user.rs         # User database management
```

## When to Use Server Functions

Use server functions (`#[server]`) when:

✅ **You need type-safe client-server communication**
- Same function signature on both client and server
- Automatic serialization of complex types
- Compile-time type checking

✅ **The endpoint is primarily used by your frontend**
- Called from Leptos components
- Part of your application's internal API
- Needs to work seamlessly with reactive state

✅ **You want simple, declarative API calls**
```rust
// Client-side usage
let result = get_cron_jobs().await?;
```

✅ **You need automatic error handling**
- Consistent `ServerFnError` type
- Automatic error propagation

## Current Server Functions

### Cron Jobs (`cron_jobs.rs`)
- `get_cron_jobs()` - Get all cron jobs for authenticated user
- `create_cron_job_action()` - Create a new cron job
- `update_cron_job_action()` - Update an existing cron job
- `delete_cron_job_action()` - Delete a cron job

### User Management (`user.rs`)
- `create_user_database_action()` - Create user's Turso database on signup

## Helper Functions (`helpers.rs`)

Shared utilities used by all server functions:
- `get_user_id_from_request()` - Extracts and validates JWT token from request
- `get_turso_client()` - Retrieves TursoClient from Actix app data

## Adding New Server Functions

1. **Choose the right file** based on feature domain:
   - Cron job related? → `cron_jobs.rs`
   - User related? → `user.rs`
   - New feature? → Create a new file (e.g., `notifications.rs`)

2. **Add the server function**:
```rust
#[server(GetNotifications, "/api")]
pub async fn get_notifications() -> Result<Vec<Notification>, ServerFnError> {
    let req = extract::<actix_web::HttpRequest>().await?;
    let user_id = get_user_id_from_request(&req).await?;
    let client = get_turso_client(&req)?;
    
    // Your logic here
    Ok(notifications)
}
```

3. **Export in `mod.rs`**:
```rust
pub mod notifications;
pub use notifications::*;
```

## Best Practices

- **Keep functions focused**: Each function should do one thing well
- **Use helpers**: Always use `get_user_id_from_request()` for authentication
- **Error handling**: Return `ServerFnError` for consistent error responses
- **Group by feature**: Keep related functions in the same file
- **Document complex logic**: Add comments for non-obvious operations

## Example Usage

### Client-side (in a component):
```rust
#[component]
fn CronJobsList() -> impl IntoView {
    let cron_jobs = Resource::new(|| (), |_| get_cron_jobs());
    
    view! {
        {move || {
            cron_jobs.get().map(|jobs| {
                match jobs {
                    Ok(jobs) => view! { /* render jobs */ },
                    Err(e) => view! { <ErrorDisplay error=e/> },
                }
            })
        }}
    }
}
```

### Server-side (in actions):
```rust
#[server(CreateCronJob, "/api")]
pub async fn create_cron_job_action(request: CreateCronJobRequest) -> Result<CronJob, ServerFnError> {
    let req = extract::<actix_web::HttpRequest>().await?;
    let user_id = get_user_id_from_request(&req).await?;  // Auth check
    let client = get_turso_client(&req)?;                  // Get DB client
    
    create_cron_job(&client, &user_id, request)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed: {}", e)))
}
```

## Why Use This Instead of Actix Routes?

- **Type Safety**: Compile-time guarantees between client and server
- **Simplicity**: No manual HTTP handling or serialization
- **Integration**: Works seamlessly with Leptos reactive system
- **Productivity**: Less boilerplate, faster development

For most API endpoints in a Leptos app, server functions are the recommended approach.

