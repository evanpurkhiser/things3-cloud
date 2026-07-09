use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const APP_NAME: &str = "things3";
const LEGACY_APP_NAME: &str = "things-cli";

fn state_home() -> PathBuf {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// The pre-`dirs`-crate location of the app's state directory. Always the
/// Linux XDG path, even on macOS/Windows, so that users migrating from an
/// earlier Unix build still get their legacy directory picked up.
fn legacy_state_home() -> PathBuf {
    if let Ok(custom) = std::env::var("XDG_STATE_HOME") {
        return PathBuf::from(custom);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("state")
}

pub fn app_state_dir() -> PathBuf {
    let target = state_home().join(APP_NAME);
    let legacy = legacy_state_home().join(LEGACY_APP_NAME);

    if target.exists() || !legacy.exists() {
        return target;
    }

    if fs::rename(&legacy, &target).is_ok() {
        return target;
    }

    target
}

pub fn append_log_dir() -> PathBuf {
    app_state_dir().join("append-log")
}

pub fn auth_file_path() -> PathBuf {
    app_state_dir().join("auth.json")
}

pub fn ensure_private_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }

    Ok(())
}

pub fn write_private_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid private file path",
        )
    })?;
    ensure_private_dir(parent)?;

    let tmp_path = path.with_extension("tmp");
    match fs::remove_file(&tmp_path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    {
        let mut file = options.open(&tmp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600))?;
    }

    fs::rename(&tmp_path, path)?;
    Ok(())
}
