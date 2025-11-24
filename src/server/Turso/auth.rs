use crate::server::turso::config::{SupabaseClaims, SupabaseConfig};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde_json::Value;

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    ValidationFailed(String),
    InvalidFormat,
    Expired,
    JwksFetchError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingToken => write!(f, "JWT token missing"),
            AuthError::ValidationFailed(msg) => write!(f, "JWT validation failed: {}", msg),
            AuthError::InvalidFormat => write!(f, "Invalid token format"),
            AuthError::Expired => write!(f, "Token expired"),
            AuthError::JwksFetchError(msg) => write!(f, "Failed to fetch JWKS: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

pub async fn validate_supabase_jwt_token(
    token: &str,
    config: &SupabaseConfig,
) -> Result<SupabaseClaims, AuthError> {
    // Decode token header to get key ID
    let header = jsonwebtoken::decode_header(token)
        .map_err(|e| AuthError::ValidationFailed(format!("Failed to decode header: {}", e)))?;
    
    let kid = header.kid.ok_or_else(|| AuthError::ValidationFailed("Missing kid in header".to_string()))?;
    
    // Fetch JWKS
    let jwks = fetch_jwks(&config.jwks_url, &config.anon_key).await?;
    
    // Find the key matching the kid
    let key = find_key(&jwks, &kid)
        .ok_or_else(|| AuthError::ValidationFailed(format!("Key with kid {} not found", kid)))?;
    
    // Decode and validate token
    let mut validation = Validation::new(Algorithm::RS256);
    // Supabase uses "authenticated" as the audience
    validation.set_audience(&["authenticated"]);
    validation.set_issuer(&[config.url.clone()]);
    
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
        .map_err(|e| AuthError::ValidationFailed(format!("Failed to create decoding key: {}", e)))?;
    
    let token_data = decode::<SupabaseClaims>(token, &decoding_key, &validation)
        .map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::Expired,
                _ => AuthError::ValidationFailed(format!("Token validation failed: {}", e)),
            }
        })?;
    
    Ok(token_data.claims)
}

pub async fn get_supabase_user_id(token: &str, config: &SupabaseConfig) -> Result<String, AuthError> {
    // Validate token and extract user_id
    let claims = validate_supabase_jwt_token(token, config).await?;
    Ok(claims.sub)
}

async fn fetch_jwks(url: &str, anon_key: &str) -> Result<Jwks, AuthError> {
    let client = reqwest::Client::new();
    
    // Try with authentication headers (some Supabase instances require this)
    let response = client
        .get(url)
        .header("apikey", anon_key)
        .header("Authorization", format!("Bearer {}", anon_key))
        .send()
        .await
        .map_err(|e| AuthError::JwksFetchError(format!("Request failed: {}", e)))?;
    
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(AuthError::JwksFetchError(format!(
            "HTTP error: {} - {} (URL: {})",
            status, error_text, url
        )));
    }
    
    let jwks: Value = response
        .json()
        .await
        .map_err(|e| AuthError::JwksFetchError(format!("Failed to parse JSON: {}", e)))?;
    
    let keys = jwks["keys"]
        .as_array()
        .ok_or_else(|| AuthError::JwksFetchError("Invalid JWKS format".to_string()))?;
    
    let jwks_keys: Vec<JwksKey> = keys
        .iter()
        .filter_map(|key| {
            Some(JwksKey {
                kid: key["kid"].as_str()?.to_string(),
                kty: key["kty"].as_str()?.to_string(),
                use_: key["use"].as_str().unwrap_or("sig").to_string(),
                n: key["n"].as_str()?.to_string(),
                e: key["e"].as_str()?.to_string(),
            })
        })
        .collect();
    
    Ok(Jwks { keys: jwks_keys })
}

fn find_key<'a>(jwks: &'a Jwks, kid: &str) -> Option<&'a JwksKey> {
    jwks.keys.iter().find(|key| key.kid == kid)
}

#[derive(Debug)]
struct Jwks {
    keys: Vec<JwksKey>,
}

#[derive(Debug)]
struct JwksKey {
    kid: String,
    kty: String,
    use_: String,
    n: String,
    e: String,
}


