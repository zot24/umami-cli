use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub server_url: Option<String>,
    pub token: Option<String>,
    pub username: Option<String>,
}

impl Config {
    fn path() -> PathBuf {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("umami-cli");
        std::fs::create_dir_all(&dir).ok();
        dir.join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        write_private(&path, &content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear() -> Result<(), String> {
        let path = Self::path();
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn config_path() -> String {
        Self::path().display().to_string()
    }
}

/// Writes `content` to `path` with mode `0600`, for a new file and for an
/// existing one, so the token is never in a file other users can read.
#[cfg(unix)]
fn write_private(path: &Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    // `mode` applies only on create and is masked by the umask.
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    file.write_all(content.as_bytes())
}

#[cfg(not(unix))]
fn write_private(path: &Path, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}
