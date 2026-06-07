//! HTTP client — post/delete used from Phase 5 onward.
#![allow(dead_code)]

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{
    api::{auth, models::TokenSet},
    config::ApiConfig,
    errors::{AppError, Result},
};

const API_BASE: &str = "https://openapi.tidal.com/v2";
const TIDAL_ACCEPT: &str = "application/vnd.tidal.v1+json";

// ---------------------------------------------------------------------------
// TidalClient
// ---------------------------------------------------------------------------

/// Central HTTP client.  Clone-cheap via inner `Arc`.
#[derive(Clone)]
pub struct TidalClient {
    pub http:   reqwest::Client,
    pub config: ApiConfig,
    tokens:     Arc<RwLock<Option<TokenSet>>>,
}

impl TidalClient {
    pub fn new(config: ApiConfig) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("tidal-tui/0.1")
            .build()
            .expect("Failed to build reqwest client");
        TidalClient {
            http,
            config,
            tokens: Arc::new(RwLock::new(None)),
        }
    }

    // -----------------------------------------------------------------------
    // Token management
    // -----------------------------------------------------------------------

    /// Populate in-memory token cache from `~/.config/tidal-tui/tokens.json`.
    pub async fn load_persisted_tokens(&self) -> Result<()> {
        if let Some(tokens) = auth::load_tokens()? {
            *self.tokens.write().await = Some(tokens);
        }
        Ok(())
    }

    /// Returns a valid access token, refreshing or re-acquiring as needed.
    ///
    /// Priority order:
    /// 1. In-memory token still valid → return immediately.
    /// 2. Have a refresh_token → call refresh endpoint.
    /// 3. Fall back to Client Credentials (catalog-only scope).
    pub async fn ensure_token(&self) -> Result<String> {
        // Fast path — read lock only.
        {
            let guard = self.tokens.read().await;
            if let Some(t) = guard.as_ref() {
                if !t.is_expired() {
                    return Ok(t.access_token.clone());
                }
            }
        }

        // Slow path — write lock; re-check after acquiring.
        let mut guard = self.tokens.write().await;
        if let Some(t) = guard.as_ref() {
            if !t.is_expired() {
                return Ok(t.access_token.clone());
            }
        }

        // Attempt refresh with stored refresh_token.
        if let Some(refresh_token) = guard.as_ref().and_then(|t| t.refresh_token.clone()) {
            match auth::refresh_tokens(&self.http, &refresh_token).await {
                Ok(new_tokens) => {
                    auth::save_tokens(&new_tokens)?;
                    let access = new_tokens.access_token.clone();
                    *guard = Some(new_tokens);
                    tracing::debug!("Token refreshed successfully");
                    return Ok(access);
                }
                Err(e) => tracing::warn!("Token refresh failed, falling back to client credentials: {e}"),
            }
        }

        // Fall back to Client Credentials.
        tracing::debug!("Acquiring token via client credentials");
        let new_tokens = auth::client_credentials(
            &self.http,
            &self.config.client_id,
            &self.config.client_secret,
        )
        .await?;
        auth::save_tokens(&new_tokens)?;
        let access = new_tokens.access_token.clone();
        *guard = Some(new_tokens);
        Ok(access)
    }

    /// Full interactive PKCE login. Prints auth URL, waits for redirect, saves tokens.
    pub async fn login_interactive(&self) -> Result<()> {
        let (verifier, challenge) = auth::generate_pkce();
        let state        = "tidal-tui";
        let redirect_uri = format!(
            "http://localhost:{}/callback",
            self.config.redirect_port
        );
        let url = auth::build_auth_url(
            &self.config.client_id,
            &redirect_uri,
            &challenge,
            state,
            &["r_usr", "w_usr"],
        );

        println!("\nOpen this URL in your browser to log in to TIDAL:\n\n  {url}\n");
        // Try to launch browser automatically on Linux — ignore failure.
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();

        println!("Waiting for redirect on http://localhost:{}/callback …", self.config.redirect_port);
        let code = auth::await_redirect_code(self.config.redirect_port).await?;

        let tokens = auth::exchange_code(
            &self.http,
            &self.config.client_id,
            &code,
            &redirect_uri,
            &verifier,
        )
        .await?;
        auth::save_tokens(&tokens)?;
        *self.tokens.write().await = Some(tokens);
        println!("Login successful. Tokens saved to {}", auth::token_path().display());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // HTTP helpers
    // -----------------------------------------------------------------------

    /// Authenticated GET; deserialises response JSON into T.
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path:  &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let token = self.ensure_token().await?;
        let url   = format!("{API_BASE}{path}");
        let resp  = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", TIDAL_ACCEPT)
            .query(query)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    /// Authenticated POST with a JSON body; deserialises response.
    pub async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let token = self.ensure_token().await?;
        let url   = format!("{API_BASE}{path}");
        let resp  = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", TIDAL_ACCEPT)
            .header("Content-Type", TIDAL_ACCEPT)
            .json(body)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    /// Authenticated DELETE; returns () on 2xx.
    pub async fn delete(&self, path: &str) -> Result<()> {
        let token = self.ensure_token().await?;
        let url   = format!("{API_BASE}{path}");
        let resp  = self
            .http
            .delete(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", TIDAL_ACCEPT)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(self.map_error(resp).await)
        }
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        if resp.status().is_success() {
            resp.json::<T>().await.map_err(AppError::from)
        } else {
            Err(self.map_error(resp).await)
        }
    }

    async fn map_error(&self, resp: reqwest::Response) -> AppError {
        let status  = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        AppError::Api { status, message }
    }
}
