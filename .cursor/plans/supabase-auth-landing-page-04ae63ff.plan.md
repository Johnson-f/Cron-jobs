<!-- 04ae63ff-7e02-4c45-8ec8-b66de943f27b 44da5a39-ce5a-4f90-94e6-4dd05bb76225 -->
# Supabase Authentication Implementation Plan

## Overview

Implement a complete authentication flow using Supabase with direct HTTP calls, featuring a landing page, login/signup forms using leptos-shadcn-ui components, and protected routes.

## Dependencies to Add

Add to `Cargo.toml`:

- `reqwest` with `json` feature for HTTP calls to Supabase
- `serde` with `derive` feature for serialization
- `serde_json` for JSON handling
- `url` for URL parsing
- `wasm-bindgen` features for localStorage (already present)

## File Structure

```
src/
├── app.rs (update routes)
├── client/
│   ├── mod.rs
│   └── supabase.rs (new - Supabase client wrapper)
├── server/
│   └── service/
│       └── auth.rs (new - server-side auth helpers if needed)
└── ui/
    ├── auth/
    │   ├── mod.rs
    │   ├── landing.rs (new - landing page)
    │   ├── login.rs (new - login form)
    │   └── signup.rs (new - signup form)
    └── components.rs (new - reusable auth components)
```

## Implementation Steps

### 1. Supabase Client Module (`src/client/supabase.rs`)

- Create `SupabaseClient` struct with URL and anon key
- Implement methods:
  - `sign_up(email, password)` - POST to `/auth/v1/signup`
  - `sign_in(email, password)` - POST to `/auth/v1/token?grant_type=password`
  - `sign_out()` - POST to `/auth/v1/logout`
  - `get_session()` - Get current session from localStorage
  - `set_session(session)` - Store session in localStorage
- Handle JWT tokens and session management
- Error handling with proper error types

### 2. Auth Context (`src/app.rs` or new `src/context/auth.rs`)

- Create `AuthContext` using Leptos `provide_context`
- Store auth state: `Option<User>` and `Option<Session>`
- Provide reactive signals for auth status
- Initialize from localStorage on app start
- Methods: `login()`, `logout()`, `is_authenticated()`

### 3. Landing Page (`src/ui/auth/landing.rs`)

- Hero section with app branding
- Two prominent buttons: "Login" and "Sign Up"
- Use `leptos-shadcn-ui::Button` components
- Navigate to `/login` and `/signup` routes
- Responsive design with Tailwind CSS

### 4. Login Form (`src/ui/auth/login.rs`)

- Form with email and password inputs
- Use `leptos-shadcn-ui` components:
  - `Input` for email/password fields
  - `Button` for submit
  - `Card` for form container (if available)
- Form validation (email format, password length)
- Error message display
- Loading state during submission
- On success: store session, update auth context, redirect to `/home`

### 5. Signup Form (`src/ui/auth/signup.rs`)

- Similar structure to login form
- Additional password confirmation field
- Password strength validation
- Error handling for existing email, weak password, etc.
- On success: same flow as login

### 6. Update Routes (`src/app.rs`)

- Add routes:
  - `/` - Landing page
  - `/login` - Login form
  - `/signup` - Signup form
  - `/home` - Protected main app (redirect to `/login` if not authenticated)
- Implement route guards for protected routes
- Handle 404 fallback

### 7. Environment Configuration

- Create `.env.example` with:
  - `VITE_SUPABASE_URL` (or `SUPABASE_URL` for SSR)
  - `VITE_SUPABASE_ANON_KEY` (or `SUPABASE_ANON_KEY` for SSR)
- Load config in client code using `wasm-bindgen` or environment variables
- For SSR, use `std::env::var()` in server code

### 8. Protected Route Component

- Create `ProtectedRoute` wrapper component
- Check auth context on mount
- Show loading state while checking
- Redirect to `/login` if not authenticated
- Render children if authenticated

### 9. Update Main App (`src/app.rs`)

- Initialize auth context at app root
- Set up all routes including auth routes
- Add navigation helpers

## Key Implementation Details

### Supabase API Endpoints

- Sign up: `POST {url}/auth/v1/signup` with `{ email, password }`
- Sign in: `POST {url}/auth/v1/token?grant_type=password` with `{ email, password }`
- Sign out: `POST {url}/auth/v1/logout` with `Authorization: Bearer {token}`
- Session format: `{ access_token, refresh_token, expires_at, user }`

### Error Handling

- Create custom error types for auth errors
- Display user-friendly error messages
- Handle network errors gracefully
- Validate responses from Supabase

### Session Management

- Store session in `localStorage` (client-side)
- Parse and validate JWT tokens
- Check token expiration
- Auto-refresh tokens if needed (future enhancement)

### UI/UX Considerations

- Show loading spinners during auth operations
- Disable form submission while processing
- Clear error messages on new input
- Smooth transitions between pages
- Accessible form labels and error messages

## Testing Checklist

- Landing page displays correctly
- Login form validates inputs
- Signup form validates inputs
- Successful login redirects to /home
- Successful signup redirects to /home
- Failed auth shows error messages
- Protected routes redirect when not authenticated
- Session persists across page refreshes
- Logout clears session and redirects

## Future Enhancements (Out of Scope)

- Email verification flow
- Password reset functionality
- Social auth (Google, GitHub, etc.)
- Remember me option
- Token refresh mechanism

### To-dos

- [ ] Add reqwest, serde, serde_json, and url dependencies to Cargo.toml
- [ ] Create SupabaseClient in src/client/supabase.rs with sign_up, sign_in, sign_out, and session management methods
- [ ] Create AuthContext in src/app.rs or new context module to manage global auth state
- [ ] Create LandingPage component in src/ui/auth/landing.rs with login and signup buttons
- [ ] Create LoginForm component in src/ui/auth/login.rs with email/password inputs and submission logic
- [ ] Create SignupForm component in src/ui/auth/signup.rs with email/password/confirm inputs and submission logic
- [ ] Update src/app.rs routes to include /, /login, /signup, and /home with proper route guards
- [ ] Create ProtectedRoute wrapper component to guard /home and redirect unauthenticated users
- [ ] Set up environment variable configuration for Supabase URL and anon key
- [ ] Implement proper error types and user-friendly error message display in auth forms