# Project Rules: Leptos/Rust Full-Stack Web Application

## Table of Contents
1. [Technology Stack](#technology-stack)
2. [Code Style and Formatting](#code-style-and-formatting)
3. [Project Structure](#project-structure)
4. [Feature Flags and Build Modes](#feature-flags-and-build-modes)
5. [Component Organization](#component-organization)
6. [Server-Side Patterns](#server-side-patterns)
7. [Client-Side Patterns](#client-side-patterns)
8. [Styling Guidelines](#styling-guidelines)
9. [Performance Considerations](#performance-considerations)
10. [Testing Strategies](#testing-strategies)
11. [Build and Deployment](#build-and-deployment)
12. [Common Patterns and Anti-Patterns](#common-patterns-and-anti-patterns)

---

## Technology Stack

### Core Framework
- **Leptos 0.8.2**: Primary web framework for reactive UI
- **Actix-web 4**: HTTP server for SSR (Server-Side Rendering)
- **WASM**: Client-side execution target
- **cargo-leptos**: Build tool for Leptos applications

### Key Dependencies
- `leptos_meta`: Meta tags, stylesheets, and document head management
- `leptos_router`: Client-side routing
- `leptos_actix`: Integration between Leptos and Actix-web
- `leptos-shadcn-ui`: UI component library
- `wasm-bindgen`: WASM bindings
- `thaw`: Additional utilities

### Styling
- **Tailwind CSS 3.4.1**: Utility-first CSS framework
- **PostCSS**: CSS processing
- **Autoprefixer**: Browser compatibility

### Development Tools
- **Playwright**: End-to-end testing
- **cargo-leptos watch**: Hot-reload development server

---

## Code Style and Formatting

### Rust Code Style
1. **Use `rustfmt`**: Always format code with `cargo fmt` before committing
2. **Follow Rust 2021 Edition**: The project uses Rust 2021 edition
3. **Naming Conventions**:
   - Components: PascalCase (e.g., `HomePage`, `UserProfile`)
   - Functions: snake_case (e.g., `get_user_data`, `handle_click`)
   - Constants: SCREAMING_SNAKE_CASE
   - Modules: snake_case

4. **Imports Organization**:
   ```rust
   // Standard library
   use std::collections::HashMap;
   
   // External crates
   use leptos::prelude::*;
   use leptos_router::*;
   
   // Local modules
   use crate::server::models::*;
   ```

5. **Documentation**:
   - Use `///` for public API documentation
   - Use `//` for inline comments
   - Document all public functions, structs, and components

### Leptos-Specific Style
1. **Component Macros**: Always use `#[component]` attribute for Leptos components
2. **View Macros**: Use `view!` macro for JSX-like syntax
3. **Reactive Values**: Prefer `RwSignal` for mutable state, `ReadSignal` for read-only
4. **Event Handlers**: Use `on:click`, `on:input`, etc. with closures

---

## Project Structure

### Directory Layout
```
src/
├── lib.rs              # Library entry point, contains hydrate function
├── main.rs             # Server entry point (SSR)
├── app.rs              # Root application component and routing
├── client/             # Client-side only code
│   └── mod.rs
├── server/             # Server-side code
│   ├── mod.rs
│   ├── routes/         # Actix-web route handlers
│   │   └── mod.rs
│   ├── models/         # Data models and structs
│   │   └── mod.rs
│   └── service/        # Business logic and services
│       └── mod.rs
└── ui/                 # Reusable UI components
    └── (component files)
```

### File Naming Conventions
- **Modules**: `mod.rs` or `{module_name}.rs`
- **Components**: `{ComponentName}.rs` (PascalCase) in `src/ui/`
- **Routes**: `{route_name}.rs` in `src/server/routes/`
- **Models**: `{model_name}.rs` in `src/server/models/`
- **Services**: `{service_name}.rs` in `src/server/service/`

### Module Organization
- Keep modules focused and single-purpose
- Use `mod.rs` files to organize submodules
- Export public APIs through parent modules

---

## Feature Flags and Build Modes

### Available Features
1. **`csr`** (Client-Side Rendering): Pure client-side app
2. **`hydrate`** (Hydration): Client-side hydration of SSR content
3. **`ssr`** (Server-Side Rendering): Server-side rendering with Actix-web

### Feature Flag Usage Rules

1. **Conditional Compilation**:
   ```rust
   #[cfg(feature = "ssr")]
   {
       // Server-only code
   }
   
   #[cfg(not(feature = "ssr"))]
   {
       // Client-only code
   }
   ```

2. **Default Build Configuration**:
   - **Bin target** (server): Uses `ssr` feature
   - **Lib target** (client): Uses `hydrate` feature
   - Both targets use `--no-default-features`

3. **Feature Gating Guidelines**:
   - Server functions: Always gate with `#[cfg(feature = "ssr")]`
   - Client-only code: Gate with `#[cfg(not(feature = "ssr"))]`
   - Shared code: No feature gates needed

4. **Never Mix Features**:
   - Don't enable both `csr` and `ssr` simultaneously
   - Use `hydrate` with `ssr` for full-stack apps
   - Use `csr` only for pure client-side development/testing

---

## Component Organization

### Component Structure
1. **Component Definition**:
   ```rust
   #[component]
   pub fn ComponentName(
       #[prop(optional)] optional_prop: Option<String>,
       #[prop(default = 0)] default_prop: i32,
       required_prop: String,
   ) -> impl IntoView {
       // Component logic
       view! {
           // JSX-like markup
       }
   }
   ```

2. **Component Location**:
   - **Page components**: Define in `src/app.rs` or route-specific files
   - **Reusable components**: Place in `src/ui/` directory
   - **Layout components**: Create in `src/ui/layouts/`

3. **Component Props**:
   - Use `#[prop(optional)]` for optional props
   - Use `#[prop(default = value)]` for default values
   - Use `#[prop(into)]` for automatic conversion
   - Keep props minimal and focused

4. **Component Naming**:
   - Public components: PascalCase, exported with `pub`
   - Private components: PascalCase, no `pub` keyword
   - Component files: Match component name

### Reactive State Management
1. **Signal Types**:
   - `RwSignal<T>`: Mutable reactive state
   - `ReadSignal<T>`: Read-only reactive state
   - `Memo<T>`: Computed/derived reactive values
   - `Resource<T>`: Async data loading

2. **State Location**:
   - Local state: Use `RwSignal::new()` in component
   - Shared state: Use `provide_context()` / `use_context()`
   - Server state: Use `Resource` with server functions

3. **State Updates**:
   ```rust
   // Mutable signal
   let count = RwSignal::new(0);
   *count.write() += 1;  // Write access
   let value = count.read();  // Read access
   
   // In view macros
   {count}  // Automatic reactivity
   ```

---

## Server-Side Patterns

### Server Functions
1. **Definition**:
   ```rust
   #[server(GetUserData, "/api/user")]
   pub async fn get_user_data(user_id: i32) -> Result<User, ServerFnError> {
       // Server-only code
       Ok(user)
   }
   ```

2. **Usage**:
   - Call from client components using `spawn_local` or `Resource`
   - Always handle `Result` types
   - Use proper error types (`ServerFnError`)

3. **Server Function Rules**:
   - Place in `src/server/service/` or appropriate module
   - Keep async and non-blocking
   - Use proper error handling
   - Document expected inputs/outputs

### Actix-web Routes
1. **Route Definition**:
   ```rust
   #[cfg(feature = "ssr")]
   #[actix_web::get("/api/custom")]
   async fn custom_route() -> impl Responder {
       // Route handler
   }
   ```

2. **Route Organization**:
   - Define in `src/server/routes/` modules
   - Group related routes in same module
   - Register in main server setup

3. **Route Registration**:
   - Register custom routes in `main.rs` before `leptos_routes`
   - Use appropriate HTTP methods
   - Return proper response types

### Server Models
1. **Data Models**:
   - Define in `src/server/models/`
   - Use `serde` for serialization
   - Implement `Clone`, `Debug` where needed

2. **Model Structure**:
   ```rust
   #[derive(Clone, Debug, Serialize, Deserialize)]
   pub struct User {
       pub id: i32,
       pub name: String,
   }
   ```

---

## Client-Side Patterns

### Routing
1. **Route Definition**:
   ```rust
   <Router>
       <Routes fallback=move || view! { <NotFound/> }>
           <Route path=StaticSegment("") view=HomePage/>
           <Route path=StaticSegment("about") view=AboutPage/>
           <Route path=WildcardSegment("any") view=NotFound/>
       </Routes>
   </Router>
   ```

2. **Navigation**:
   - Use `leptos_router::use_navigate()` for programmatic navigation
   - Use `<A href="/path">` for link components
   - Use `use_params()` for route parameters

### Async Data Loading
1. **Resource Pattern**:
   ```rust
   let user_resource = Resource::new(
       move || user_id(),
       |id| async move {
           get_user_data(id).await
       }
   );
   ```

2. **Resource Usage**:
   ```rust
   view! {
       {move || match user_resource.get() {
           None => view! { <Loading/> },
           Some(Ok(user)) => view! { <UserProfile user/> },
           Some(Err(e)) => view! { <Error error=e/> },
       }}
   }
   ```

### Client-Only Code
1. **Placement**: Put client-only code in `src/client/`
2. **Feature Gating**: Use `#[cfg(not(feature = "ssr"))]` when needed
3. **WASM Considerations**: Be mindful of WASM bundle size

---

## Styling Guidelines

### Tailwind CSS
1. **Configuration**:
   - Tailwind config: `tailwind.config.js`
   - Input CSS: `input.css`
   - Output CSS: `style/output.css`

2. **Content Scanning**:
   - Tailwind scans `*.html` and `./src/**/*.rs` files
   - Use Tailwind classes directly in `view!` macros
   - Use `class` attribute for conditional classes

3. **Class Usage**:
   ```rust
   view! {
       <div class="flex items-center justify-center p-4">
           <button class=move || format!("btn {}", if active() { "active" } else { "" })>
               "Click me"
           </button>
       </div>
   }
   ```

4. **Responsive Design**:
   - Use Tailwind responsive prefixes: `sm:`, `md:`, `lg:`, `xl:`, `2xl:`
   - Mobile-first approach

### CSS File Management
1. **Development**: Run `npm run tailwind:watch` alongside `cargo leptos watch`
2. **Production**: Ensure CSS is built before deployment
3. **Custom Styles**: Add to `input.css` if needed

### Component Styling
1. **Prefer Tailwind**: Use utility classes over custom CSS
2. **Component Libraries**: Use `leptos-shadcn-ui` components when available
3. **Consistent Design**: Follow design system patterns

---

## Performance Considerations

### WASM Bundle Size
1. **Optimization Profile**: Uses `wasm-release` profile:
   - `opt-level = 'z'`: Optimize for size
   - `lto = true`: Link-time optimization
   - `codegen-units = 1`: Single codegen unit
   - `panic = "abort"`: Smaller panic handling

2. **Bundle Size Rules**:
   - Monitor WASM bundle size regularly
   - Avoid large dependencies in client code
   - Use `#[cfg(feature = "ssr")]` to exclude server code from WASM
   - Consider code splitting for large applications

### SSR Performance
1. **Server Rendering**:
   - Keep server functions fast and non-blocking
   - Use async/await properly
   - Cache expensive computations when possible

2. **Hydration**:
   - Minimize hydration mismatches
   - Use `expect_context` carefully (SSR-only)
   - Test hydration in development

### Reactive Performance
1. **Signal Efficiency**:
   - Use `Memo` for expensive computations
   - Avoid unnecessary signal updates
   - Batch updates when possible

2. **Resource Efficiency**:
   - Use `Resource` with proper keys
   - Implement proper caching strategies
   - Handle loading and error states efficiently

---

## Testing Strategies

### Unit Testing
1. **Test Location**: Create `tests/` directory at project root
2. **Test Structure**:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_function() {
           // Test code
       }
   }
   ```

### Integration Testing
1. **Server Tests**: Test Actix-web routes and server functions
2. **Component Tests**: Test Leptos components in isolation (when possible)

### End-to-End Testing
1. **Playwright**: Use Playwright for E2E tests
2. **Test Location**: `end2end/tests/`
3. **Configuration**: `end2end/playwright.config.ts`
4. **Running Tests**: `npx playwright test` (or via cargo-leptos)

### Testing Best Practices
1. Test server functions independently
2. Test component logic separately from rendering
3. Use E2E tests for critical user flows
4. Mock external dependencies in tests

---

## Build and Deployment

### Development Build
1. **Command**: `cargo leptos watch`
2. **Hot Reload**: Automatic on file changes
3. **Tailwind Watch**: Run `npm run tailwind:watch` in separate terminal
4. **Start Script**: Use `./start.sh` to run both processes

### Production Build
1. **Command**: `cargo leptos build --release`
2. **Output Locations**:
   - Server binary: `target/server/release/{project_name}`
   - Site files: `target/site/`
   - WASM bundle: `target/site/pkg/`

### Deployment Requirements
1. **Minimum Files**:
   - Server binary from `target/server/release/`
   - Entire `target/site/` directory

2. **Environment Variables**:
   ```bash
   export LEPTOS_OUTPUT_NAME="cron-jobs"
   export LEPTOS_SITE_ROOT="site"
   export LEPTOS_SITE_PKG_DIR="pkg"
   export LEPTOS_SITE_ADDR="127.0.0.1:3000"
   export LEPTOS_RELOAD_PORT="3001"
   ```

3. **Remote Deployment**:
   - Copy server binary and site directory
   - Set environment variables
   - Run server binary
   - No Rust toolchain needed on server

### Build Configuration
1. **Cargo.toml Settings**:
   - `output-name`: WASM bundle name (default: "front")
   - `site-root`: Output directory (default: "target/site")
   - `site-pkg-dir`: Package directory (default: "pkg")
   - `site-addr`: Server address
   - `env`: Environment mode ("DEV" or "PROD")

2. **Profile Settings**:
   - Use `wasm-release` profile for optimized WASM builds
   - Standard `release` profile for server binary

---

## Common Patterns and Anti-Patterns

### ✅ Good Patterns

1. **Component Composition**:
   ```rust
   #[component]
   pub fn Page() -> impl IntoView {
       view! {
           <Layout>
               <Header/>
               <Content/>
               <Footer/>
           </Layout>
       }
   }
   ```

2. **Proper Error Handling**:
   ```rust
   let result = Resource::new(|| (), |_| async {
       server_function().await.map_err(|e| {
           eprintln!("Error: {}", e);
           e
       })
   });
   ```

3. **Context Usage**:
   ```rust
   // Provide context
   provide_context(AppState::new());
   
   // Use context
   let app_state = use_context::<AppState>().expect("AppState should exist");
   ```

4. **Conditional Rendering**:
   ```rust
   view! {
       {move || if condition() {
           view! { <Component/> }
       } else {
           view! { <Alternative/> }
       }}
   }
   ```

### ❌ Anti-Patterns

1. **Don't Mix SSR and CSR Code**:
   ```rust
   // ❌ BAD: Accessing server context in client code
   #[cfg(not(feature = "ssr"))]
   let resp = expect_context::<ResponseOptions>();  // Will panic!
   
   // ✅ GOOD: Feature-gate properly
   #[cfg(feature = "ssr")]
   let resp = expect_context::<ResponseOptions>();
   ```

2. **Don't Create Signals in View Macros**:
   ```rust
   // ❌ BAD: Creates new signal on every render
   view! {
       {RwSignal::new(0)}  // Wrong!
   }
   
   // ✅ GOOD: Create signal outside view
   let count = RwSignal::new(0);
   view! { {count} }
   ```

3. **Don't Block the Async Runtime**:
   ```rust
   // ❌ BAD: Blocking call in async function
   async fn bad_function() {
       std::thread::sleep(Duration::from_secs(1));  // Blocks!
   }
   
   // ✅ GOOD: Use async sleep
   async fn good_function() {
       tokio::time::sleep(Duration::from_secs(1)).await;
   }
   ```

4. **Don't Forget Error Handling**:
   ```rust
   // ❌ BAD: Unwrapping without handling
   let user = get_user().await.unwrap();
   
   // ✅ GOOD: Proper error handling
   match get_user().await {
       Ok(user) => { /* use user */ },
       Err(e) => { /* handle error */ },
   }
   ```

5. **Don't Create Large WASM Bundles**:
   - Avoid importing entire large crates
   - Use feature flags to exclude server code
   - Consider lazy loading for large components

### Best Practices Summary

1. **Always feature-gate server-only code** with `#[cfg(feature = "ssr")]`
2. **Use proper error types** (`ServerFnError`, `Result<T, E>`)
3. **Keep components focused** and composable
4. **Use Resources for async data** loading
5. **Minimize WASM bundle size** through careful dependency management
6. **Test both SSR and client-side** rendering
7. **Use Tailwind utilities** instead of custom CSS when possible
8. **Document public APIs** and complex logic
9. **Follow Rust conventions** for naming and organization
10. **Keep server functions async** and non-blocking

---

## Additional Notes

### Development Workflow
1. Start development: `./start.sh` or manually run both:
   - `cargo leptos watch`
   - `npm run tailwind:watch`

2. Access application: `http://localhost:3000`

3. Hot reload: Both Rust and CSS changes trigger automatic reload

### Debugging
1. **Client-side**: Use browser DevTools
2. **Server-side**: Use `println!`, `eprintln!`, or proper logging
3. **WASM**: Use `console_error_panic_hook` (already included)

### Dependencies Management
1. **Add dependencies**: Edit `Cargo.toml`, then `cargo build`
2. **Update dependencies**: `cargo update`
3. **Check for updates**: Review `Cargo.lock` regularly

### Version Compatibility
- **Leptos**: 0.8.2
- **Actix-web**: 4.x
- **wasm-bindgen**: =0.2.105 (exact version)
- **Rust**: Nightly toolchain required

---

## Conclusion

These project rules should guide development of the Leptos/Rust full-stack application. Follow these conventions for consistency, maintainability, and performance. When in doubt, refer to the [Leptos documentation](https://leptos.dev/) and Rust best practices.

