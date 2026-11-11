use anyhow::{Context, Result, anyhow};
use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use serde::{Deserialize, Serialize};

use crate::dirs::{auth_file_path, write_private_atomic};

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
    let path = auth_file_path();
    write_auth_at(&path, email, password)?;
    Ok(path)
}

fn write_auth_at(path: &std::path::Path, email: &str, password: &str) -> Result<()> {
    let (email, password) = validate_auth(email, password)?;
    path.parent()
        .ok_or_else(|| anyhow!("Invalid auth file path"))?;

    let payload = AuthPayload { email, password };
    let serialized = serde_json::to_string(&payload)?;
    // write_private_atomic creates the parent owner-only, stages the payload in
    // a fresh 0600 file (never following a symlink planted at the tmp path) and
    // renames it into place, so the plaintext password is never world-readable.
    write_private_atomic(path, serialized.as_bytes())
        .with_context(|| format!("Failed writing {}", path.display()))?;

    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn auth_file_is_created_private() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("auth.json");
        write_auth_at(&path, "user@example.com", "hunter2").expect("write auth");

        let mode = fs::metadata(&path).expect("metadata").permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn password_is_not_written_through_a_planted_symlink() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("auth.json");
        let victim = dir.path().join("victim");
        fs::write(&victim, "original").expect("seed victim");
        std::os::unix::fs::symlink(&victim, path.with_extension("tmp")).expect("plant symlink");

        write_auth_at(&path, "user@example.com", "hunter2").expect("write auth");

        assert_eq!(
            fs::read_to_string(&victim).expect("victim"),
            "original",
            "the password must not be written through a symlink at the staging path"
        );
    }
}
