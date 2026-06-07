use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use sha2::{Digest, Sha256};
use std::io::Write;
use crate::{
    api::models::TokenSet,
    errors::{AppError, Result},
};

// ---------------------------------------------------------------------------
// Token persistence
// ---------------------------------------------------------------------------

pub fn token_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("tidal-tui")
        .join("tokens.json")
}

pub fn save_tokens(tokens: &TokenSet) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let path = token_path();
    // Ensure parent directory exists.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(tokens)?;
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)?
        .write_all(json.as_bytes())?;
    Ok(())
}

pub fn load_tokens() -> Result<Option<TokenSet>> {
    let path = token_path();
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)?;
    let tokens: TokenSet = serde_json::from_str(&raw)?;
    Ok(Some(tokens))
}

// ---------------------------------------------------------------------------
// PKCE helpers
// ---------------------------------------------------------------------------

/// Generate a PKCE (code_verifier, code_challenge) pair using S256.
pub fn generate_pkce() -> (String, String) {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf).expect("getrandom failed");
    let verifier  = URL_SAFE_NO_PAD.encode(buf);
    let hash      = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hash);
    (verifier, challenge)
}

/// Build the TIDAL authorization URL that the user must open in a browser.
pub fn build_auth_url(
    client_id:      &str,
    redirect_uri:   &str,
    code_challenge: &str,
    state:          &str,
    scopes:         &[&str],
) -> String {
    let scope_str = scopes.join("%20");
    format!(
        "https://login.tidal.com/authorize\
         ?response_type=code\
         &client_id={client_id}\
         &redirect_uri={redirect_uri}\
         &scope={scope_str}\
         &code_challenge_method=S256\
         &code_challenge={code_challenge}\
         &state={state}",
    )
}

// ---------------------------------------------------------------------------
// Localhost redirect server
// ---------------------------------------------------------------------------

/// Bind a TCP listener on `127.0.0.1:<port>`, accept one request, extract the
/// `code` parameter from the redirect URL, and return it.
pub async fn await_redirect_code(port: u16) -> Result<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await?;
    let (mut stream, _) = listener.accept().await?;
    let mut buf = [0u8; 4096];
    let n       = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let code = extract_query_param(&request, "code").ok_or_else(|| {
        AppError::Auth("No 'code' parameter in redirect request".into())
    })?;

    let html = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                <h2>Login successful! You may close this tab.</h2>";
    stream.write_all(html.as_bytes()).await?;
    Ok(code)
}

fn extract_query_param(request: &str, key: &str) -> Option<String> {
    // Request line looks like: GET /callback?code=XXX&state=YYY HTTP/1.1
    let line = request.lines().next()?;
    let path = line.split_whitespace().nth(1)?;
    let query = path.splitn(2, '?').nth(1)?;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next()? == key {
            return Some(kv.next().unwrap_or("").to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Token endpoint helpers
// ---------------------------------------------------------------------------

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

async fn map_token_response(resp: reqwest::Response) -> Result<TokenSet> {
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Auth(format!(
            "Token request failed ({status}): {body}"
        )));
    }
    let mut token_set: TokenSet = resp.json().await?;
    token_set.expires_at = unix_now() + token_set.expires_in;
    Ok(token_set)
}

// ---------------------------------------------------------------------------
// OAuth2 flows
// ---------------------------------------------------------------------------

/// Client Credentials flow — returns an access token with no user scope.
/// Suitable for public catalog access when no user session exists.
pub async fn client_credentials(
    http:          &reqwest::Client,
    client_id:     &str,
    client_secret: &str,
) -> Result<TokenSet> {
    let credentials = STANDARD.encode(format!("{client_id}:{client_secret}"));
    let resp = http
        .post("https://auth.tidal.com/v1/oauth2/token")
        .header("Authorization", format!("Basic {credentials}"))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?;
    map_token_response(resp).await
}

/// Authorization Code exchange — trade a PKCE authorization code for tokens.
pub async fn exchange_code(
    http:          &reqwest::Client,
    client_id:     &str,
    code:          &str,
    redirect_uri:  &str,
    code_verifier: &str,
) -> Result<TokenSet> {
    let resp = http
        .post("https://auth.tidal.com/v1/oauth2/token")
        .form(&[
            ("grant_type",    "authorization_code"),
            ("client_id",     client_id),
            ("code",          code),
            ("redirect_uri",  redirect_uri),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await?;
    map_token_response(resp).await
}

/// Refresh Token flow — exchange a refresh token for a new access token.
pub async fn refresh_tokens(
    http:          &reqwest::Client,
    refresh_token: &str,
) -> Result<TokenSet> {
    let resp = http
        .post("https://auth.tidal.com/v1/oauth2/token")
        .form(&[
            ("grant_type",    "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await?;
    map_token_response(resp).await
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verifier_and_challenge_differ() {
        let (v, c) = generate_pkce();
        assert!(!v.is_empty());
        assert!(!c.is_empty());
        assert_ne!(v, c);
    }

    #[test]
    fn pkce_challenge_is_base64url() {
        let (_, c) = generate_pkce();
        // URL_SAFE_NO_PAD — must not contain +, /, or =
        assert!(!c.contains('+'));
        assert!(!c.contains('/'));
        assert!(!c.contains('='));
    }

    #[test]
    fn build_auth_url_contains_required_params() {
        let url = build_auth_url(
            "cid",
            "http://localhost:8080/callback",
            "chall",
            "st",
            &["r_usr", "w_usr"],
        );
        assert!(url.contains("client_id=cid"), "missing client_id");
        assert!(url.contains("code_challenge=chall"), "missing code_challenge");
        assert!(url.contains("code_challenge_method=S256"), "missing S256");
        assert!(url.contains("response_type=code"), "missing response_type");
        assert!(url.contains("r_usr"), "missing r_usr scope");
    }

    #[test]
    fn extract_query_param_works() {
        let req = "GET /callback?code=MYCODE&state=tidal-tui HTTP/1.1\r\n";
        assert_eq!(extract_query_param(req, "code"), Some("MYCODE".into()));
        assert_eq!(extract_query_param(req, "state"), Some("tidal-tui".into()));
        assert_eq!(extract_query_param(req, "missing"), None);
    }
}
