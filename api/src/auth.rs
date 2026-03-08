use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode}};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Authentik uid — the stable user identifier stored in the database.
    pub sub: String,
    pub email: Option<String>,
    pub groups: Option<Vec<String>>,
}

// ── JWKS verifier ─────────────────────────────────────────────────────────────

pub struct JwksVerifier {
    jwks_url: String,
    cache: RwLock<HashMap<String, DecodingKey>>,
}

impl JwksVerifier {
    pub fn new(jwks_url: String) -> Self {
        Self { jwks_url, cache: RwLock::new(HashMap::new()) }
    }

    pub async fn verify(&self, token: &str) -> Result<Claims, String> {
        let kid = decode_header(token)
            .map_err(|e| e.to_string())?
            .kid
            .ok_or_else(|| "token missing kid header".to_string())?;

        // Try cache first — avoids a network round-trip on every request.
        {
            let cache = self.cache.read().await;
            if let Some(key) = cache.get(&kid) {
                if let Ok(data) = self.decode_token(token, key) {
                    return Ok(data);
                }
            }
        }

        // Unknown kid — re-fetch the JWKS and retry.
        self.refresh_cache().await?;

        let cache = self.cache.read().await;
        let key = cache.get(&kid).ok_or_else(|| format!("unknown kid: {kid}"))?;
        self.decode_token(token, key)
    }

    fn decode_token(&self, token: &str, key: &DecodingKey) -> Result<Claims, String> {
        let mut v = Validation::new(Algorithm::RS256);
        v.validate_aud = false; // Authentik doesn't set aud by default
        decode::<Claims>(token, key, &v)
            .map(|d| d.claims)
            .map_err(|e| e.to_string())
    }

    async fn refresh_cache(&self) -> Result<(), String> {
        let body: Value = Client::new()
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| format!("JWKS fetch failed: {e}"))?
            .json()
            .await
            .map_err(|e| format!("JWKS parse failed: {e}"))?;

        let keys = body["keys"].as_array().ok_or("JWKS: no keys array")?;
        let mut cache = self.cache.write().await;

        for key in keys {
            let kid = key["kid"].as_str().unwrap_or_default().to_string();
            if kid.is_empty() { continue; }
            if let (Some(n), Some(e)) = (key["n"].as_str(), key["e"].as_str()) {
                if let Ok(dk) = DecodingKey::from_rsa_components(n, e) {
                    cache.insert(kid, dk);
                }
            }
        }
        Ok(())
    }
}

// ── Axum extractor ────────────────────────────────────────────────────────────
// Extracts and validates the Bearer token before every protected handler.
// Rust 1.75+ supports async fns in traits natively — no #[async_trait] needed.

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    Arc<JwksVerifier>: axum::extract::FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verifier = Arc::<JwksVerifier>::from_ref(state);

        let auth = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

        let token = auth
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Expected Bearer token".to_string()))?;

        verifier
            .verify(token)
            .await
            .map_err(|e| (StatusCode::UNAUTHORIZED, e))
    }
}
