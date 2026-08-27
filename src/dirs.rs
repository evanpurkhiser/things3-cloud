use std::{
    fs,
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

/// Create `dir` and narrow it to owner-only. The state directory holds the
/// Things Cloud password and an append log carrying every task title and note,
/// so the default 0755 leaves that readable by any other local user.
pub fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn mode_of(path: &Path) -> u32 {
        fs::metadata(path).expect("metadata").permissions().mode() & 0o777
    }

    #[test]
    fn creates_the_directory_private() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path().join("things3").join("append-log");
        create_private_dir(&dir).expect("create");
        assert_eq!(mode_of(&dir), 0o700);
    }

    #[test]
    fn narrows_an_existing_world_readable_directory() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path().join("things3");
        fs::create_dir_all(&dir).expect("seed");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("widen");
        create_private_dir(&dir).expect("create");
        assert_eq!(mode_of(&dir), 0o700);
    }
}
