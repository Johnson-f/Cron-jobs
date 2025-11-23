use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Request, RequestInit, RequestMode, Response, Headers};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub user_metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<i64>,
    pub user: User,
}

// Response when email confirmation is required (signup returns just user)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpResponseWithoutSession {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub confirmation_sent_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

// Response when email confirmation is disabled (signup returns session directly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpResponseWithSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<i64>,
    pub expires_in: Option<i64>,
    pub user: User,
    pub token_type: Option<String>,
}

// Sign-in response always returns session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<i64>,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub enum SupabaseError {
    Network(String),
    Auth(String),
    Parse(String),
    Storage(String),
}

impl SupabaseError {
    pub fn new(msg: impl Into<String>) -> Self {
        SupabaseError::Auth(msg.into())
    }
}

impl std::fmt::Display for SupabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupabaseError::Network(msg) => write!(f, "Network error: {}", msg),
            SupabaseError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            SupabaseError::Parse(msg) => write!(f, "Parse error: {}", msg),
            SupabaseError::Storage(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

#[derive(Clone)]
pub struct SupabaseClient {
    url: String,
    anon_key: String,
}

impl SupabaseClient {
    pub fn new(url: String, anon_key: String) -> Self {
        Self { url, anon_key }
    }
}

// WASM-only implementation
#[cfg(target_arch = "wasm32")]
impl SupabaseClient {
    fn get_storage() -> Result<web_sys::Storage, SupabaseError> {
        window()
            .ok_or_else(|| SupabaseError::Storage("Window not available".to_string()))?
            .local_storage()
            .map_err(|_| SupabaseError::Storage("Failed to access localStorage".to_string()))?
            .ok_or_else(|| SupabaseError::Storage("localStorage not available".to_string()))
    }

    pub fn get_session(&self) -> Result<Option<Session>, SupabaseError> {
        let storage = Self::get_storage()?;
        let session_str = match storage
            .get_item("supabase.auth.session")
            .map_err(|_| SupabaseError::Storage("Failed to read from localStorage".to_string()))?
        {
            Some(s) => s,
            None => return Ok(None),
        };

        let session: Session = serde_json::from_str(&session_str)
            .map_err(|e| SupabaseError::Parse(format!("Failed to parse session: {}", e)))?;

        // Check if session is expired
        if let Some(expires_at) = session.expires_at {
            let now = js_sys::Date::now() as i64 / 1000;
            if now >= expires_at {
                self.clear_session()?;
                return Ok(None);
            }
        }

        Ok(Some(session))
    }

    pub fn set_session(&self, session: Session) -> Result<(), SupabaseError> {
        let storage = Self::get_storage()?;
        let session_str = serde_json::to_string(&session)
            .map_err(|e| SupabaseError::Parse(format!("Failed to serialize session: {}", e)))?;

        storage
            .set_item("supabase.auth.session", &session_str)
            .map_err(|_| SupabaseError::Storage("Failed to write to localStorage".to_string()))?;

        Ok(())
    }

    pub fn clear_session(&self) -> Result<(), SupabaseError> {
        let storage = Self::get_storage()?;
        storage
            .remove_item("supabase.auth.session")
            .map_err(|_| SupabaseError::Storage("Failed to clear localStorage".to_string()))?;
        Ok(())
    }

    pub async fn sign_up(&self, email: String, password: String) -> Result<Session, SupabaseError> {
        let url = format!("{}/auth/v1/signup", self.url);
        let payload = SignUpRequest { email, password };
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| SupabaseError::Parse(format!("Failed to serialize request: {}", e)))?;

        let headers = Headers::new()
            .map_err(|e| SupabaseError::Network(format!("Failed to create headers: {:?}", e)))?;
        headers.set("apikey", &self.anon_key)
            .map_err(|e| SupabaseError::Network(format!("Failed to set apikey header: {:?}", e)))?;
        headers.set("Content-Type", "application/json")
            .map_err(|e| SupabaseError::Network(format!("Failed to set content-type header: {:?}", e)))?;

        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_mode(RequestMode::Cors);
        opts.set_body(&JsValue::from_str(&payload_json));
        opts.set_headers(&headers);

        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| SupabaseError::Network(format!("Failed to create request: {:?}", e)))?;

        let window = window().ok_or_else(|| SupabaseError::Network("Window not available".to_string()))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| SupabaseError::Network(format!("Request failed: {:?}", e)))?;

        let resp: Response = resp_value.dyn_into()
            .map_err(|e| SupabaseError::Network(format!("Response is not a Response: {:?}", e)))?;

        if !resp.ok() {
            let error_text = JsFuture::from(resp.text().map_err(|e| {
                SupabaseError::Network(format!("Failed to get response text: {:?}", e))
            })?)
                .await
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(SupabaseError::Auth(error_text));
        }

        let json = JsFuture::from(resp.json().map_err(|e| {
            SupabaseError::Parse(format!("Failed to get JSON: {:?}", e))
        })?)
            .await
            .map_err(|e| SupabaseError::Parse(format!("Failed to parse JSON: {:?}", e)))?;

        // Try to deserialize as session response first (email confirmation disabled)
        if let Ok(signup_with_session) = serde_wasm_bindgen::from_value::<SignUpResponseWithSession>(json.clone()) {
            let session = Session {
                access_token: signup_with_session.access_token,
                refresh_token: signup_with_session.refresh_token,
                expires_at: signup_with_session.expires_at,
                user: signup_with_session.user,
            };
            self.set_session(session.clone())?;
            Ok(session)
        } else {
            // Try user-only response (email confirmation enabled)
            match serde_wasm_bindgen::from_value::<SignUpResponseWithoutSession>(json) {
                Ok(_user_response) => {
                    // Email confirmation required - user created but no session
                    Err(SupabaseError::Auth(
                        "Account created! Please check your email to confirm your account before signing in.".to_string()
                    ))
                }
                Err(e) => {
                    Err(SupabaseError::Parse(format!("Failed to deserialize signup response: {}", e)))
                }
            }
        }
    }

    pub async fn sign_in(&self, email: String, password: String) -> Result<Session, SupabaseError> {
        let url = format!("{}/auth/v1/token?grant_type=password", self.url);
        let payload = SignInRequest { email, password };
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| SupabaseError::Parse(format!("Failed to serialize request: {}", e)))?;

        let headers = Headers::new()
            .map_err(|e| SupabaseError::Network(format!("Failed to create headers: {:?}", e)))?;
        headers.set("apikey", &self.anon_key)
            .map_err(|e| SupabaseError::Network(format!("Failed to set apikey header: {:?}", e)))?;
        headers.set("Content-Type", "application/json")
            .map_err(|e| SupabaseError::Network(format!("Failed to set content-type header: {:?}", e)))?;

        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_mode(RequestMode::Cors);
        opts.set_body(&JsValue::from_str(&payload_json));
        opts.set_headers(&headers);

        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| SupabaseError::Network(format!("Failed to create request: {:?}", e)))?;

        let window = window().ok_or_else(|| SupabaseError::Network("Window not available".to_string()))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| SupabaseError::Network(format!("Request failed: {:?}", e)))?;

        let resp: Response = resp_value.dyn_into()
            .map_err(|e| SupabaseError::Network(format!("Response is not a Response: {:?}", e)))?;

        if !resp.ok() {
            let error_text = JsFuture::from(resp.text().map_err(|e| {
                SupabaseError::Network(format!("Failed to get response text: {:?}", e))
            })?)
                .await
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(SupabaseError::Auth(error_text));
        }

        let json = JsFuture::from(resp.json().map_err(|e| {
            SupabaseError::Parse(format!("Failed to get JSON: {:?}", e))
        })?)
            .await
            .map_err(|e| SupabaseError::Parse(format!("Failed to parse JSON: {:?}", e)))?;

        let auth_response: SignInResponse = serde_wasm_bindgen::from_value(json)
            .map_err(|e| SupabaseError::Parse(format!("Failed to deserialize response: {}", e)))?;

        let session = Session {
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
            expires_at: auth_response.expires_at,
            user: auth_response.user,
        };

        self.set_session(session.clone())?;
        Ok(session)
    }

    pub async fn sign_out(&self) -> Result<(), SupabaseError> {
        let session = self.get_session()?;
        if let Some(session) = session {
            let url = format!("{}/auth/v1/logout", self.url);
            
            let headers = Headers::new()
                .map_err(|e| SupabaseError::Network(format!("Failed to create headers: {:?}", e)))?;
            headers.set("apikey", &self.anon_key)
                .map_err(|e| SupabaseError::Network(format!("Failed to set apikey header: {:?}", e)))?;
            headers.set("Authorization", &format!("Bearer {}", session.access_token))
                .map_err(|e| SupabaseError::Network(format!("Failed to set auth header: {:?}", e)))?;

            let opts = RequestInit::new();
            opts.set_method("POST");
            opts.set_mode(RequestMode::Cors);
            opts.set_headers(&headers);

            let request = Request::new_with_str_and_init(&url, &opts)
                .map_err(|e| SupabaseError::Network(format!("Failed to create request: {:?}", e)))?;

            let window = window().ok_or_else(|| SupabaseError::Network("Window not available".to_string()))?;
            let _ = JsFuture::from(window.fetch_with_request(&request)).await;
        }

        self.clear_session()?;
        Ok(())
    }
}

// Server-side stub implementation (methods will panic if called)
#[cfg(not(target_arch = "wasm32"))]
impl SupabaseClient {
    pub fn get_session(&self) -> Result<Option<Session>, SupabaseError> {
        Err(SupabaseError::Storage("Supabase client not available on server".to_string()))
    }

    pub fn set_session(&self, _session: Session) -> Result<(), SupabaseError> {
        Err(SupabaseError::Storage("Supabase client not available on server".to_string()))
    }

    pub fn clear_session(&self) -> Result<(), SupabaseError> {
        Err(SupabaseError::Storage("Supabase client not available on server".to_string()))
    }

    pub async fn sign_up(&self, _email: String, _password: String) -> Result<Session, SupabaseError> {
        Err(SupabaseError::Auth("Supabase client not available on server".to_string()))
    }

    pub async fn sign_in(&self, _email: String, _password: String) -> Result<Session, SupabaseError> {
        Err(SupabaseError::Auth("Supabase client not available on server".to_string()))
    }

    pub async fn sign_out(&self) -> Result<(), SupabaseError> {
        Err(SupabaseError::Auth("Supabase client not available on server".to_string()))
    }
}