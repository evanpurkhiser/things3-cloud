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

/// Create `dir` and narrow it to owner-only. The state directory holds the
/// Things Cloud password and an append log carrying every task title and note,
/// so the default 0755 leaves that readable by any other local user.
pub fn ensure_private_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }

    Ok(())
}

/// Write `bytes` to `path` without ever exposing them wider than 0600, and
/// without following a symlink planted at the staging path.
pub fn write_private_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid private file path",
        )
    })?;
    ensure_private_dir(parent)?;

    let tmp_path = path.with_extension("tmp");
    // Clear a stale staging file from an interrupted run; create_new refuses it.
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
        // open() filters the requested mode through the umask, which can clear
        // owner bits too, so restate it before the bytes go in. The file is
        // already no wider than 0600, so this opens no window.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    fs::rename(&tmp_path, path)?;
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
        ensure_private_dir(&dir).expect("create");
        assert_eq!(mode_of(&dir), 0o700);
    }

    #[test]
    fn narrows_an_existing_world_readable_directory() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path().join("things3");
        fs::create_dir_all(&dir).expect("seed");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("widen");
        ensure_private_dir(&dir).expect("create");
        assert_eq!(mode_of(&dir), 0o700);
    }

    #[test]
    fn writes_the_file_private() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("things3").join("secret.json");
        write_private_atomic(&path, b"{}").expect("write");
        assert_eq!(mode_of(&path), 0o600);
        assert_eq!(fs::read(&path).expect("read"), b"{}");
    }

    #[test]
    fn does_not_write_through_a_planted_symlink() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("secret.json");
        let victim = tmp.path().join("victim");
        fs::write(&victim, "original").expect("seed victim");
        std::os::unix::fs::symlink(&victim, path.with_extension("tmp")).expect("plant symlink");

        write_private_atomic(&path, b"secret").expect("write");

        assert_eq!(
            fs::read_to_string(&victim).expect("victim"),
            "original",
            "the payload must not be written through a symlink at the staging path"
        );
        assert_eq!(fs::read_to_string(&path).expect("read"), "secret");
    }
}
