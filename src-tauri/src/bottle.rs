use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::{AppConfig, GraphicsBackend};

/// Represents a Wine bottle (an isolated Wine prefix).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleInfo {
    /// Unique identifier for this bottle.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Wine version used by this bottle.
    pub wine_version: String,
    /// Path to the Wine prefix on disk.
    pub prefix_path: PathBuf,
    /// Path to the bottle configuration file.
    pub config_path: PathBuf,
    /// When the bottle was created.
    pub created_at: DateTime<Utc>,
    /// Associated Steam App ID, if any.
    pub steam_app_id: Option<String>,
    /// Current status.
    pub status: BottleStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BottleStatus {
    Idle,
    Running,
    Crashed,
    Configuring,
}

/// Per-bottle configuration stored in `bottle.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BottleConfig {
    #[serde(default)]
    pub bottle: BottleSection,
    #[serde(default)]
    pub graphics: GraphicsSection,
    #[serde(default)]
    pub windows: WindowsSection,
    #[serde(default)]
    pub environment: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub steam: SteamSection,
    #[serde(default)]
    pub dependencies: DependenciesSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BottleSection {
    pub name: String,
    pub wine_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSection {
    pub backend: GraphicsBackend,
    #[serde(default = "default_true")]
    pub dxgi_override_native: bool,
    #[serde(default = "default_true")]
    pub d3d10_override_native: bool,
    #[serde(default = "default_true")]
    pub d3d11_override_native: bool,
    #[serde(default = "default_true")]
    pub d3d12_override_native: bool,
}

impl Default for GraphicsSection {
    fn default() -> Self {
        Self {
            backend: GraphicsBackend::D3DMetal,
            dxgi_override_native: true,
            d3d10_override_native: true,
            d3d11_override_native: true,
            d3d12_override_native: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsSection {
    #[serde(default = "default_win_ver")]
    pub version: String,
    pub virtual_desktop: Option<String>,
}

impl Default for WindowsSection {
    fn default() -> Self {
        Self {
            version: default_win_ver(),
            virtual_desktop: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SteamSection {
    pub app_id: Option<String>,
    pub launch_options: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependenciesSection {
    #[serde(default)]
    pub vcrun: Vec<String>,
    #[serde(default)]
    pub native_dlls: Vec<String>,
}

fn default_win_ver() -> String {
    "win10".into()
}
fn default_true() -> bool {
    true
}

pub struct BottleManager {
    config: AppConfig,
}

impl BottleManager {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// List all bottles found on disk.
    pub fn list_bottles(&self) -> Result<Vec<BottleInfo>, String> {
        let mut bottles = Vec::new();

        let entries = std::fs::read_dir(&self.config.bottles_dir)
            .map_err(|e| format!("Failed to read bottles dir: {e}"))?;

        for entry in entries.flatten() {
            let prefix_path = entry.path();
            if !prefix_path.is_dir() {
                continue;
            }

            let id = prefix_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let config_path = prefix_path.join("bottle.toml");
            let bottle_config = if config_path.exists() {
                self.read_config(&config_path).unwrap_or_default()
            } else {
                BottleConfig::default()
            };

            let drive_c = prefix_path.join("drive_c");
            let status = if drive_c.exists() {
                BottleStatus::Idle
            } else {
                BottleStatus::Configuring
            };

            bottles.push(BottleInfo {
                steam_app_id: bottle_config.steam.app_id.clone(),
                wine_version: bottle_config.bottle.wine_version.clone(),
                name: bottle_config.bottle.name.clone(),
                created_at: chrono::Utc::now(), // would read from fs metadata in prod
                id,
                prefix_path,
                config_path,
                status,
            });
        }

        Ok(bottles)
    }

    /// Create a new bottle. Runs `wineboot -u` to initialize the prefix.
    pub async fn create_bottle(
        &self,
        name: &str,
        wine_version: &str,
        steam_app_id: Option<&str>,
    ) -> Result<BottleInfo, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let prefix_path = self.config.bottles_dir.join(&id);
        std::fs::create_dir_all(&prefix_path)
            .map_err(|e| format!("Failed to create bottle dir: {e}"))?;

        // Write initial config.
        let config = BottleConfig {
            bottle: BottleSection {
                name: name.to_string(),
                wine_version: wine_version.to_string(),
            },
            steam: SteamSection {
                app_id: steam_app_id.map(String::from),
                ..Default::default()
            },
            ..Default::default()
        };

        let config_path = prefix_path.join("bottle.toml");
        self.write_config(&config_path, &config)?;

        // Initialize the Wine prefix (this requires an actual Wine build).
        // In production we would run: WINEPREFIX=path wine64 wineboot -u
        // For now, create the expected directory structure.
        let drive_c = prefix_path.join("drive_c").join("windows");
        std::fs::create_dir_all(&drive_c)
            .map_err(|e| format!("Failed to create drive_c: {e}"))?;

        Ok(BottleInfo {
            steam_app_id: steam_app_id.map(String::from),
            wine_version: wine_version.to_string(),
            name: name.to_string(),
            created_at: chrono::Utc::now(),
            status: BottleStatus::Idle,
            id,
            prefix_path,
            config_path,
        })
    }

    /// Delete a bottle and all its data.
    pub fn delete_bottle(&self, bottle_id: &str) -> Result<(), String> {
        // Prevent path traversal attacks
        if bottle_id.contains("..") || bottle_id.contains('/') || bottle_id.contains('\\') {
            return Err("Invalid bottle ID".into());
        }
        let prefix_path = self.config.bottles_dir.join(bottle_id);
        // Resolve and verify we're still inside bottles_dir
        let canonical = prefix_path.canonicalize().unwrap_or(prefix_path.clone());
        let canonical_base = self.config.bottles_dir.canonicalize().unwrap_or_else(|_| self.config.bottles_dir.clone());
        if !canonical.starts_with(&canonical_base) {
            return Err("Bottle path escapes base directory".into());
        }
        if !prefix_path.exists() {
            return Err(format!("Bottle '{bottle_id}' not found"));
        }
        std::fs::remove_dir_all(&prefix_path)
            .map_err(|e| format!("Failed to delete bottle: {e}"))
    }

    /// Read configuration from a bottle's `bottle.toml`.
    pub fn get_config(&self, bottle_id: &str) -> Result<BottleConfig, String> {
        let config_path = self.config.bottles_dir.join(bottle_id).join("bottle.toml");
        self.read_config(&config_path)
    }

    /// Write configuration to a bottle's `bottle.toml`.
    pub fn save_config(&self, bottle_id: &str, config: &BottleConfig) -> Result<(), String> {
        let config_path = self.config.bottles_dir.join(bottle_id).join("bottle.toml");
        self.write_config(&config_path, config)
    }

    fn read_config(&self, path: &PathBuf) -> Result<BottleConfig, String> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {e}"))?;
        toml::from_str(&contents).map_err(|e| format!("Failed to parse config: {e}"))
    }

    fn write_config(&self, path: &PathBuf, config: &BottleConfig) -> Result<(), String> {
        let contents =
            toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize: {e}"))?;
        std::fs::write(path, contents).map_err(|e| format!("Failed to write config: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config(tmp: &TempDir) -> AppConfig {
        AppConfig {
            bottles_dir: tmp.path().to_path_buf(),
            ..Default::default()
        }
    }

    #[test]
    fn test_create_and_delete_bottle() {
        let tmp = TempDir::new().unwrap();
        let manager = BottleManager::new(test_config(&tmp));

        let bottle = tokio_test::block_on(manager.create_bottle("Test Game", "9.14-staging", None))
            .unwrap();
        assert_eq!(bottle.name, "Test Game");
        assert!(bottle.prefix_path.exists());

        manager.delete_bottle(&bottle.id).unwrap();
        assert!(!bottle.prefix_path.exists());
    }

    #[test]
    fn test_config_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let manager = BottleManager::new(test_config(&tmp));

        let bottle = tokio_test::block_on(manager.create_bottle("Config Test", "9.0-staging", None))
            .unwrap();

        let mut config = manager.get_config(&bottle.id).unwrap();
        config.graphics.backend = GraphicsBackend::DXVK;
        manager.save_config(&bottle.id, &config).unwrap();

        let reloaded = manager.get_config(&bottle.id).unwrap();
        assert_eq!(reloaded.graphics.backend.to_string(), "dxvk");
    }
}
