# Routes Module

## Purpose

The `routes/` folder contains **Actix-web HTTP route handlers** - direct HTTP endpoints that give you full control over request/response handling. These are used for cases where Leptos server functions aren't sufficient.

## What are Actix-web Routes?

Actix-web routes are traditional HTTP handlers that:
- Give you complete control over HTTP behavior
- Allow custom response formats (JSON, HTML, files, etc.)
- Support advanced middleware and routing patterns
- Can be used for webhooks, static files, and external API integrations

## Structure

```
routes/
├── mod.rs          # Module exports and route registration
└── [feature].rs    # Feature-specific route handlers
```

## When to Use Actix-web Routes

Use Actix-web routes (`#[actix_web::get]`, `#[actix_web::post]`, etc.) when:

✅ **You need to serve static files**
```rust
#[actix_web::get("/favicon.ico")]
async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
    // Serve static files
}
```

✅ **You're building a public API**
- Need OpenAPI/Swagger documentation
- Third-party integrations
- Webhooks from external services

✅ **You need custom response formats**
- Non-JSON responses (HTML, XML, binary)
- Custom status codes
- Streaming responses

✅ **You need advanced HTTP features**
- Custom middleware
- Request/response transformations
- Complex routing patterns

✅ **You're integrating with external services**
- Webhook endpoints
- OAuth callbacks
- Payment processing callbacks

## Current Routes

Currently, this folder is empty. Routes are defined directly in `main.rs` for simple cases like:
- `/favicon.ico` - Static file serving
- `/pkg/*` - WASM/JS assets
- `/assets/*` - Static assets

## Adding New Routes

1. **Create a route file** (e.g., `webhooks.rs`):
```rust
use actix_web::{web, HttpResponse, Responder};

#[actix_web::post("/webhooks/stripe")]
pub async fn stripe_webhook(
    payload: web::Bytes,
    req: actix_web::HttpRequest,
) -> impl Responder {
    // Handle webhook
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}
```

2. **Export in `mod.rs`**:
```rust
pub mod webhooks;

pub fn configure_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(webhooks::stripe_webhook);
}
```

3. **Register in `main.rs`**:
```rust
App::new()
    .configure(crate::server::routes::configure_routes)  // Add this
    .leptos_routes(routes, ...)
```

## Comparison: Routes vs Server Functions

| Feature | Actix Routes | Server Functions |
|---------|--------------|------------------|
| **Type Safety** | Manual | ✅ Automatic |
| **Serialization** | Manual | ✅ Automatic |
| **Client Integration** | Manual fetch | ✅ Direct calls |
| **Flexibility** | ✅ Full control | Limited |
| **Use Case** | External APIs, files | Internal APIs |
| **Complexity** | Higher | Lower |

## Best Practices

- **Use sparingly**: Most endpoints should be server functions
- **Document well**: Add clear comments for route behavior
- **Handle errors properly**: Return appropriate HTTP status codes
- **Validate inputs**: Manually validate request data
- **Group related routes**: Keep related endpoints in the same file

## Example Use Cases

### 1. Webhook Endpoint
```rust
#[actix_web::post("/webhooks/github")]
pub async fn github_webhook(
    payload: web::Json<serde_json::Value>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    // Verify webhook signature
    // Process webhook payload
    HttpResponse::Ok()
}
```

### 2. File Upload
```rust
#[actix_web::post("/api/upload")]
pub async fn upload_file(
    mut payload: actix_multipart::Multipart,
) -> impl Responder {
    // Handle multipart file upload
    HttpResponse::Ok()
}
```

### 3. Public API Endpoint
```rust
#[actix_web::get("/api/v1/public/stats")]
pub async fn public_stats() -> impl Responder {
    // Return public statistics
    HttpResponse::Ok().json(serde_json::json!({
        "total_jobs": 1000,
        "active_users": 500
    }))
}
```

## Why Use This Instead of Server Functions?

- **Full HTTP Control**: Custom headers, status codes, response types
- **External Integration**: Webhooks, OAuth callbacks, third-party APIs
- **Static Files**: Serving assets, images, documents
- **Public APIs**: When you need RESTful endpoints for external consumers

## When NOT to Use Routes

❌ **Don't use routes for:**
- Internal application APIs (use server functions)
- Type-safe client-server communication (use server functions)
- Simple CRUD operations (use server functions)
- Anything that needs to be called from Leptos components (use server functions)

## Summary

- **Server Functions** (`actions/`) = 90% of your API endpoints
- **Actix Routes** (`routes/`) = 10% for special cases (webhooks, files, public APIs)

For most endpoints in a Leptos application, prefer server functions. Use routes only when you need the additional flexibility and control that Actix-web provides.

