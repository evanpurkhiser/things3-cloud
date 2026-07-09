use anyhow::{Context, Result, anyhow};
use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use serde::{Deserialize, Serialize};

use crate::dirs::{auth_file_path, ensure_private_dir, write_private_atomic};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthPayload {
    email: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AuthConfig {
    email: Option<String>,
    password: Option<String>,
}

fn load_auth_config(path: &std::path::Path) -> Result<AuthConfig> {
    let mut figment = Figment::new();
    if path.exists() {
        figment = figment.merge(Json::file(path));
    }
    figment
        .merge(Env::prefixed("THINGS3_"))
        .extract()
        .with_context(|| format!("Failed reading auth config at {}", path.display()))
}

fn validate_auth(email: &str, password: &str) -> Result<(String, String)> {
    let email = email.trim().to_string();
    let password = password.to_string();

    if email.is_empty() {
        return Err(anyhow!("Missing auth email."));
    }
    if password.is_empty() {
        return Err(anyhow!("Missing auth password."));
    }

    Ok((email, password))
}

pub fn load_auth() -> Result<(String, String)> {
    let path = auth_file_path();

    let cfg = load_auth_config(&path)?;

    let Some(email) = cfg.email else {
        return Err(anyhow!(
            "Missing auth email. Set THINGS3_EMAIL or run `things3 set-auth` to create {}.",
            path.display()
        ));
    };

    let Some(password) = cfg.password else {
        return Err(anyhow!(
            "Missing auth password. Set THINGS3_PASSWORD or run `things3 set-auth` to update {}.",
            path.display()
        ));
    };

    validate_auth(&email, &password)
}

pub fn write_auth(email: &str, password: &str) -> Result<std::path::PathBuf> {
    let (email, password) = validate_auth(email, password)?;
    let path = auth_file_path();
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("Invalid auth file path"))?
        .to_path_buf();
    ensure_private_dir(&parent).with_context(|| format!("Failed creating {}", parent.display()))?;

    let payload = AuthPayload { email, password };
    let serialized = serde_json::to_string(&payload)?;
    write_private_atomic(&path, serialized.as_bytes())
        .with_context(|| format!("Failed writing {}", path.display()))?;

    Ok(path)
}
