#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[cfg(target_arch = "wasm32")]
pub fn get_supabase_url() -> String {
    // Try to get from window.__ENV__ or use default
    if let Some(win) = window() {
        if let Ok(env) = js_sys::Reflect::get(&win, &JsValue::from_str("__ENV__")) {
            if !env.is_undefined() && !env.is_null() {
                if let Ok(url) = js_sys::Reflect::get(&env, &JsValue::from_str("VITE_SUPABASE_URL")) {
                    if let Some(url_str) = url.as_string() {
                        return url_str;
                    }
                }
            }
        }
    }
    
    // Fallback to default (should be set via environment in production)
    "https://your-project.supabase.co".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_supabase_url() -> String {
    // Server-side: read from environment variable or return default
    std::env::var("VITE_SUPABASE_URL")
        .unwrap_or_else(|_| "https://your-project.supabase.co".to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn get_supabase_anon_key() -> String {
    // Try to get from window.__ENV__ or use default
    if let Some(win) = window() {
        if let Ok(env) = js_sys::Reflect::get(&win, &JsValue::from_str("__ENV__")) {
            if !env.is_undefined() && !env.is_null() {
                if let Ok(key) = js_sys::Reflect::get(&env, &JsValue::from_str("VITE_SUPABASE_ANON_KEY")) {
                    if let Some(key_str) = key.as_string() {
                        return key_str;
                    }
                }
            }
        }
    }
    
    // Fallback to default (should be set via environment in production)
    "your-anon-key".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_supabase_anon_key() -> String {
    // Server-side: read from environment variable or return default
    std::env::var("VITE_SUPABASE_ANON_KEY")
        .unwrap_or_else(|_| "your-anon-key".to_string())
}