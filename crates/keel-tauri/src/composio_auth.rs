//! Composio OAuth PKCE flow for desktop apps.
//!
//! Reads the OAuth discovery state from the macOS keychain, opens the browser
//! for authorization, handles the localhost callback, exchanges the code for
//! an access token, and writes it back to the keychain.

use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// -- Public API --

/// Run the full OAuth PKCE flow. Opens the browser, waits for the callback,
/// exchanges the code, and stores the token in the keychain.
pub async fn run_oauth_flow() -> Result<(), String> {
    let config = read_oauth_config()?;
    let metadata = fetch_metadata(&config.auth_server_url).await?;

    let verifier = generate_verifier();
    let challenge = compute_challenge(&verifier);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to start callback server: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| e.to_string())?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    let auth_url = build_auth_url(
        &metadata.authorization_endpoint,
        &config.client_id,
        &redirect_uri,
        &config.scope,
        &challenge,
    );

    std::process::Command::new("open")
        .arg(&auth_url)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {e}"))?;

    let code = wait_for_callback(listener).await?;

    let token = exchange_code(
        &metadata.token_endpoint,
        &code,
        &config.client_id,
        &redirect_uri,
        &verifier,
    )
    .await?;

    update_keychain_token(&token.access_token, token.expires_in)?;

    eprintln!("[composio] OAuth flow completed successfully");
    Ok(())
}

// -- Internal types --

struct OAuthConfig {
    client_id: String,
    auth_server_url: String,
    scope: String,
}

#[derive(Deserialize)]
struct OAuthMetadata {
    authorization_endpoint: String,
    token_endpoint: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

// -- Read config from keychain --

fn read_oauth_config() -> Result<OAuthConfig, String> {
    let username = get_username()?;
    let data = read_keychain(&username)?;
    let mcp_oauth = data
        .get("mcpOAuth")
        .and_then(|v| v.as_object())
        .ok_or("No mcpOAuth in keychain")?;

    for (key, info) in mcp_oauth {
        if key.starts_with("composio") {
            let client_id = info
                .get("clientId")
                .and_then(|v| v.as_str())
                .ok_or("Missing clientId")?
                .to_string();
            let auth_server_url = info
                .get("discoveryState")
                .and_then(|d| d.get("authorizationServerUrl"))
                .and_then(|v| v.as_str())
                .ok_or("Missing authorizationServerUrl")?
                .to_string();
            let scope = info
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("openid profile email offline_access")
                .to_string();
            return Ok(OAuthConfig {
                client_id,
                auth_server_url,
                scope,
            });
        }
    }
    Err("No composio entry in keychain".to_string())
}

// -- Fetch OAuth metadata --

async fn fetch_metadata(auth_server_url: &str) -> Result<OAuthMetadata, String> {
    let client = reqwest::Client::new();

    // Try standard OAuth AS metadata first
    let url = format!(
        "{}/.well-known/oauth-authorization-server",
        auth_server_url
    );
    eprintln!("[composio:auth] Fetching metadata from: {url}");
    if let Ok(resp) = client.get(&url).send().await {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        eprintln!("[composio:auth] AS metadata: HTTP {status}, body: {}", &body[..body.len().min(500)]);
        if status.is_success() {
            if let Ok(meta) = serde_json::from_str::<OAuthMetadata>(&body) {
                return Ok(meta);
            }
        }
    }

    // Fall back to OpenID Connect discovery
    let url2 = format!("{}/.well-known/openid-configuration", auth_server_url);
    eprintln!("[composio:auth] Trying OIDC discovery: {url2}");
    let resp = client
        .get(&url2)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch OAuth metadata: {e}"))?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    eprintln!("[composio:auth] OIDC metadata: HTTP {status}, body: {}", &body[..body.len().min(500)]);
    serde_json::from_str::<OAuthMetadata>(&body)
        .map_err(|e| format!("Invalid OAuth metadata: {e}"))
}

// -- PKCE --

fn generate_verifier() -> String {
    let bytes1 = uuid::Uuid::new_v4().into_bytes();
    let bytes2 = uuid::Uuid::new_v4().into_bytes();
    let bytes3 = uuid::Uuid::new_v4().into_bytes();
    let mut combined = Vec::with_capacity(48);
    combined.extend_from_slice(&bytes1);
    combined.extend_from_slice(&bytes2);
    combined.extend_from_slice(&bytes3);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&combined)
}

fn compute_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

// -- Build authorization URL --

fn build_auth_url(
    endpoint: &str,
    client_id: &str,
    redirect_uri: &str,
    scope: &str,
    challenge: &str,
) -> String {
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state=composio",
        endpoint,
        pct_encode(client_id),
        pct_encode(redirect_uri),
        pct_encode(scope),
        challenge,
    )
}

fn pct_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                String::from(b as char)
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}

// -- Wait for OAuth callback --

async fn wait_for_callback(
    listener: tokio::net::TcpListener,
) -> Result<String, String> {
    let (mut stream, _) = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        listener.accept(),
    )
    .await
    .map_err(|_| "OAuth timed out — no response within 5 minutes".to_string())?
    .map_err(|e| format!("Failed to accept callback: {e}"))?;

    let mut buf = vec![0u8; 8192];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| e.to_string())?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("");

    // Check for error
    if let Some(err) = extract_param(path, "error") {
        let desc = extract_param(path, "error_description")
            .unwrap_or_else(|| err.clone());
        send_response(&mut stream, "Authorization failed", &desc).await;
        return Err(format!("OAuth error: {desc}"));
    }

    let code = extract_param(path, "code")
        .ok_or("No authorization code in callback")?;

    send_response(
        &mut stream,
        "Connected!",
        "You can close this window and return to the app.",
    )
    .await;

    Ok(code)
}

fn extract_param(path: &str, key: &str) -> Option<String> {
    path.split('?')
        .nth(1)?
        .split('&')
        .find(|p| p.starts_with(&format!("{key}=")))?
        .strip_prefix(&format!("{key}="))
        .map(|v| v.replace('+', " "))
        .map(|v| pct_decode(&v))
}

fn pct_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else {
            result.push(c);
        }
    }
    result
}

async fn send_response(
    stream: &mut tokio::net::TcpStream,
    title: &str,
    message: &str,
) {
    let html = format!(
        "<html><head><style>body{{font-family:system-ui;display:flex;justify-content:center;\
         align-items:center;height:100vh;margin:0;background:#fafafa;color:#0d0d0d}}\
         .c{{text-align:center}}\
         h2{{font-size:24px;font-weight:600;margin:0 0 8px}}\
         p{{color:#5d5d5d;font-size:14px;margin:0}}</style></head>\
         <body><div class='c'><h2>{title}</h2><p>{message}</p></div></body></html>"
    );
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(resp.as_bytes()).await;
}

// -- Exchange code for token --

async fn exchange_code(
    token_endpoint: &str,
    code: &str,
    client_id: &str,
    redirect_uri: &str,
    verifier: &str,
) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(token_endpoint)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
            ("code_verifier", verifier),
        ])
        .send()
        .await
        .map_err(|e| format!("Token exchange failed: {e}"))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!("Token exchange returned {status}: {body}"));
    }

    serde_json::from_str(&body)
        .map_err(|e| format!("Invalid token response: {e}"))
}

// -- Update keychain --

fn update_keychain_token(
    access_token: &str,
    expires_in: Option<u64>,
) -> Result<(), String> {
    let username = get_username()?;
    let mut data = read_keychain(&username)?;

    if let Some(mcp_oauth) = data.get_mut("mcpOAuth").and_then(|v| v.as_object_mut()) {
        for (key, info) in mcp_oauth.iter_mut() {
            if key.starts_with("composio") {
                if let Some(obj) = info.as_object_mut() {
                    obj.insert(
                        "accessToken".to_string(),
                        serde_json::Value::String(access_token.to_string()),
                    );
                    if let Some(exp) = expires_in {
                        let expires_at = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            + exp;
                        obj.insert(
                            "expiresAt".to_string(),
                            serde_json::Value::Number(expires_at.into()),
                        );
                    }
                }
                break;
            }
        }
    }

    write_keychain(&username, &data)
}

// -- Keychain helpers --

fn get_username() -> Result<String, String> {
    let output = std::process::Command::new("whoami")
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn read_keychain(username: &str) -> Result<serde_json::Value, String> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-a",
            username,
            "-w",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Could not read keychain".to_string());
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(json_str.trim())
        .map_err(|e| format!("Invalid keychain JSON: {e}"))
}

fn write_keychain(
    username: &str,
    data: &serde_json::Value,
) -> Result<(), String> {
    let json = serde_json::to_string(data)
        .map_err(|e| format!("Failed to serialize: {e}"))?;

    let status = std::process::Command::new("security")
        .args([
            "add-generic-password",
            "-U",
            "-s",
            "Claude Code-credentials",
            "-a",
            username,
            "-w",
            &json,
        ])
        .status()
        .map_err(|e| format!("Failed to update keychain: {e}"))?;

    if !status.success() {
        return Err("Failed to update keychain".to_string());
    }
    Ok(())
}
