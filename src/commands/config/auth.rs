use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::settings::global_user::get_global_config_path;

const CLIENT_ID: &str = "54d11594-84e4-41aa-b438-e81b8fa78ee7";
const AUTH_URL: &str = "https://dash.cloudflare.com/oauth2/auth";
const TOKEN_URL: &str = "https://dash.cloudflare.com/oauth2/token";

const OAUTH_REDIRECT_URI: &str = "http://localhost:8976/oauth/callback";
const OAUTH_CALLBACK_PORT: u16 = 8976;

const DNS_SCOPES: &[&str] = &["zone:read", "dns_records:read", "dns_records:edit"];

const REDIRECT_HTML_OK: &str = r#"<!DOCTYPE html>
<html><head><title>flary</title></head>
<body>
<h2>Successfully authenticated!</h2>
<p>You may close this window and return to the terminal.</p>
</body></html>"#;

const REDIRECT_HTML_FAIL: &str = r#"<!DOCTYPE html>
<html><head><title>flary</title></head>
<body>
<h2>Authentication failed</h2>
<p>Check the terminal for details. You may close this window.</p>
</body></html>"#;

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
}

fn generate_pkce_verifier() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    URL_SAFE_NO_PAD.encode(bytes)
}

fn generate_pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
    URL_SAFE_NO_PAD.encode(bytes)
}

fn parse_query_params(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let value = parts.next().unwrap_or("").to_string();
            Some((key, value))
        })
        .collect()
}

pub async fn auth() -> anyhow::Result<()> {
    let code_verifier = generate_pkce_verifier();
    let code_challenge = generate_pkce_challenge(&code_verifier);
    let state = generate_state();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", OAUTH_CALLBACK_PORT))?;

    let scope = DNS_SCOPES
        .iter()
        .chain(std::iter::once(&"offline_access"))
        .copied()
        .collect::<Vec<_>>()
        .join(" ");

    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        AUTH_URL,
        CLIENT_ID,
        urlencoding::encode(OAUTH_REDIRECT_URI),
        urlencoding::encode(&scope),
        state,
        urlencoding::encode(&code_challenge),
    );

    println!("Opening browser for Cloudflare authentication...");
    println!("If the browser doesn't open, visit:\n{}\n", auth_url);
    open::that(&auth_url)?;

    println!("Waiting for authentication on port {}...", OAUTH_CALLBACK_PORT);

    let (mut stream, _) = listener.accept()?;

    let cloned_stream = stream.try_clone()?;
    let mut reader = BufReader::new(cloned_stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/");

    let status_code;
    let response_body;

    if path.starts_with("/oauth/callback") {
        let query = path.splitn(2, '?').nth(1).unwrap_or("");
        let params = parse_query_params(query);

        if let Some(code) = params.get("code") {
            if params.get("state").map(|s| s.as_str()) != Some(&state) {
                status_code = "403 Forbidden";
                response_body = REDIRECT_HTML_FAIL.to_string();
            } else {
                match exchange_code(code, &code_verifier).await {
                    Ok(token) => {
                        let config_path = get_global_config_path()?;
                        let config_dir = config_path.parent().unwrap();
                        std::fs::create_dir_all(config_dir)?;

                        let content = format!(
                            "oauth_token = \"{}\"\nrefresh_token = \"{}\"\nexpiration_time = \"{}\"\nscopes = [{}]\n",
                            token.access_token,
                            token.refresh_token.as_deref().unwrap_or(""),
                            token
                                .expires_in
                                .map(|e| {
                                    let exp = std::time::SystemTime::now()
                                        + std::time::Duration::from_secs(e);
                                    let datetime: chrono::DateTime<chrono::Utc> = exp.into();
                                    datetime.to_rfc3339()
                                })
                                .unwrap_or_else(|| "3021-12-31T23:59:59+00:00".to_string()),
                            DNS_SCOPES
                                .iter()
                                .map(|s| format!("\"{}\"", s))
                                .collect::<Vec<_>>()
                                .join(", ")
                        );

                        std::fs::write(&config_path, &content)?;

                        status_code = "200 OK";
                        response_body = REDIRECT_HTML_OK.to_string();
                        println!(
                            "\nAuthenticated successfully! Token saved to {}",
                            config_path.display()
                        );
                        println!("Scopes: {}", DNS_SCOPES.join(", "));
                    }
                    Err(e) => {
                        status_code = "500 Internal Server Error";
                        response_body = REDIRECT_HTML_FAIL.to_string();
                        eprintln!("\nToken exchange failed: {}", e);
                    }
                }
            }
        } else if let Some(error) = params.get("error") {
            status_code = "400 Bad Request";
            response_body = REDIRECT_HTML_FAIL.to_string();
            eprintln!(
                "\nAuthentication error: {} - {}",
                error,
                params
                    .get("error_description")
                    .unwrap_or(&String::new())
            );
        } else {
            status_code = "400 Bad Request";
            response_body = REDIRECT_HTML_FAIL.to_string();
            eprintln!("\nNo authorization code received");
        }
    } else {
        status_code = "404 Not Found";
        response_body = "<html><body>Not found</body></html>".to_string();
    }

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status_code,
        response_body.len(),
        response_body
    );

    stream.write_all(response.as_bytes())?;

    Ok(())
}

async fn exchange_code(
    code: &str,
    code_verifier: &str,
) -> anyhow::Result<TokenResponse> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("code", code);
    params.insert("redirect_uri", OAUTH_REDIRECT_URI);
    params.insert("client_id", CLIENT_ID);
    params.insert("code_verifier", code_verifier);

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await?
        .error_for_status()?
        .json::<TokenResponse>()
        .await?;

    Ok(resp)
}
