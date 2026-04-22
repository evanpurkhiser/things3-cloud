use std::{fs, path::PathBuf};

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
