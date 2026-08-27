use std::{fs, io::Write};

use anyhow::{Context, Result, anyhow};
use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use serde::{Deserialize, Serialize};

use crate::dirs::{auth_file_path, create_private_dir};

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
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("Invalid auth file path"))?;
    create_private_dir(parent).with_context(|| format!("Failed creating {}", parent.display()))?;

    let payload = AuthPayload { email, password };
    let serialized = serde_json::to_string(&payload)?;
    let tmp_path = path.with_extension("tmp");

    // Create the staging file already private: chmod'ing after the fact leaves
    // the plaintext password world-readable in between, and create_new won't
    // follow a symlink planted at tmp_path. The rename carries the mode over.
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    // Clear a stale staging file from an interrupted run; create_new refuses it.
    let _ = fs::remove_file(&tmp_path);
    let mut file = opts
        .open(&tmp_path)
        .with_context(|| format!("Failed writing {}", tmp_path.display()))?;
    // open() filters the requested mode through the umask, which can clear
    // owner bits too, so restate it before the password goes in. The file is
    // already no wider than 0600, so this opens no window.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Failed securing {}", tmp_path.display()))?;
    }
    file.write_all(serialized.as_bytes())
        .with_context(|| format!("Failed writing {}", tmp_path.display()))?;

    fs::rename(&tmp_path, path).with_context(|| format!("Failed finalizing {}", path.display()))?;

    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
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
