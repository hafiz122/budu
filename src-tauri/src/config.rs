use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application-wide configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Base directory for all GameRunner data.
    pub data_dir: PathBuf,
    /// Directory where Wine versions are staged.
    pub wine_dir: PathBuf,
    /// Directory where bottles (Wine prefixes) live.
    pub bottles_dir: PathBuf,
    /// Directory where cached runtimes live.
    pub runtimes_dir: PathBuf,
    /// Directory for logs.
    pub logs_dir: PathBuf,
    /// Path to the local compatibility database.
    pub compat_db_path: PathBuf,
    /// Default Wine version to use for new bottles.
    pub default_wine_version: String,
    /// Default graphics backend.
    pub default_graphics_backend: GraphicsBackend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphicsBackend {
    D3DMetal,
    DXVK,
    WineD3D,
}

impl std::fmt::Display for GraphicsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::D3DMetal => write!(f, "d3dmetal"),
            Self::DXVK => write!(f, "dxvk"),
            Self::WineD3D => write!(f, "wine3d"),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = directories::UserDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        let data_dir = home.join(".gamerunner");

        Self {
            wine_dir: data_dir.join("wine"),
            bottles_dir: data_dir.join("bottles"),
            runtimes_dir: data_dir.join("runtimes"),
            logs_dir: data_dir.join("logs"),
            compat_db_path: data_dir.join("compat").join("db.json"),
            data_dir,
            default_wine_version: "9.14-staging".into(),
            default_graphics_backend: GraphicsBackend::D3DMetal,
        }
    }
}

impl AppConfig {
    /// Ensure all required directories exist on disk.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        for dir in [
            &self.wine_dir,
            &self.bottles_dir,
            &self.runtimes_dir,
            &self.logs_dir,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        if let Some(parent) = self.compat_db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Path to the Unix domain socket for the IPC daemon (legacy, kept for
    /// out-of-process tooling).
    pub fn socket_path(&self) -> PathBuf {
        self.data_dir.join("daemon.sock")
    }
}
