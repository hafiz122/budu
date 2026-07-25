use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
            backend: GraphicsBackend::DXMT,
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
            if !prefix_path.is_dir() || !prefix_path.join("bottle.toml").is_file() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().to_string();
            bottles.push(self.bottle_info_from_path(self.resolve_bottle_path(&id)?)?);
        }

        bottles.sort_by(|a, b| a.name.cmp(&b.name));
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
        self.create_bottle_with_id(&id, name, wine_version, steam_app_id)
    }

    /// Return the managed bottle for a Steam app, creating it when necessary.
    pub fn get_or_create_for_steam_app(
        &self,
        app_id: &str,
        app_name: &str,
        wine_version: &str,
    ) -> Result<BottleInfo, String> {
        if app_id.is_empty() || !app_id.chars().all(|c| c.is_ascii_digit()) {
            return Err("Steam App ID must contain only digits".into());
        }
        validate_identifier(wine_version, "Wine version")?;

        if let Some(existing) = self
            .list_bottles()?
            .into_iter()
            .find(|bottle| bottle.steam_app_id.as_deref() == Some(app_id))
        {
            return Ok(existing);
        }

        let id = format!("steam-{app_id}");
        let prefix_path = self.config.bottles_dir.join(&id);
        if prefix_path.exists() {
            let existing = self.bottle_info_from_path(prefix_path)?;
            if existing.steam_app_id.as_deref() == Some(app_id) {
                return Ok(existing);
            }
            return Err(format!(
                "Bottle '{id}' already exists but is not assigned to Steam App {app_id}"
            ));
        }

        let name = if app_name.trim().is_empty() {
            format!("Steam App {app_id}")
        } else {
            app_name.trim().to_string()
        };
        self.create_bottle_with_id(&id, &name, wine_version, Some(app_id))
    }

    fn create_bottle_with_id(
        &self,
        id: &str,
        name: &str,
        wine_version: &str,
        steam_app_id: Option<&str>,
    ) -> Result<BottleInfo, String> {
        validate_identifier(id, "Bottle ID")?;
        validate_identifier(wine_version, "Wine version")?;
        let prefix_path = self.config.bottles_dir.join(id);
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
        std::fs::create_dir_all(&drive_c).map_err(|e| format!("Failed to create drive_c: {e}"))?;

        Ok(BottleInfo {
            steam_app_id: steam_app_id.map(String::from),
            wine_version: wine_version.to_string(),
            name: name.to_string(),
            created_at: chrono::Utc::now(),
            status: BottleStatus::Idle,
            id: id.to_string(),
            prefix_path,
            config_path,
        })
    }

    /// Delete a bottle and all its data.
    pub fn delete_bottle(&self, bottle_id: &str) -> Result<(), String> {
        let prefix_path = self.resolve_bottle_path(bottle_id)?;
        std::fs::remove_dir_all(&prefix_path).map_err(|e| format!("Failed to delete bottle: {e}"))
    }

    /// Read configuration from a bottle's `bottle.toml`.
    pub fn get_config(&self, bottle_id: &str) -> Result<BottleConfig, String> {
        let config_path = self.resolve_bottle_path(bottle_id)?.join("bottle.toml");
        self.read_config(&config_path)
    }

    /// Write configuration to a bottle's `bottle.toml`.
    pub fn save_config(&self, bottle_id: &str, config: &BottleConfig) -> Result<(), String> {
        let config_path = self.resolve_bottle_path(bottle_id)?.join("bottle.toml");
        self.write_config(&config_path, config)
    }

    /// Resolve a managed bottle directory without allowing path traversal or symlink escapes.
    pub fn resolve_bottle_path(&self, bottle_id: &str) -> Result<PathBuf, String> {
        resolve_existing_child(&self.config.bottles_dir, bottle_id, "Bottle")
    }

    fn bottle_info_from_path(&self, prefix_path: PathBuf) -> Result<BottleInfo, String> {
        let id = prefix_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or_else(|| "Bottle path has no filename".to_string())?;
        validate_identifier(&id, "Bottle ID")?;

        let config_path = prefix_path.join("bottle.toml");
        let bottle_config = if config_path.exists() {
            self.read_config(&config_path)?
        } else {
            BottleConfig::default()
        };
        let created_at = prefix_path
            .metadata()
            .and_then(|metadata| metadata.created().or_else(|_| metadata.modified()))
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(|_| Utc::now());
        let status = if prefix_path.join("drive_c").exists() {
            BottleStatus::Idle
        } else {
            BottleStatus::Configuring
        };

        Ok(BottleInfo {
            steam_app_id: bottle_config.steam.app_id,
            wine_version: bottle_config.bottle.wine_version,
            name: bottle_config.bottle.name,
            created_at,
            id,
            prefix_path,
            config_path,
            status,
        })
    }

    fn read_config(&self, path: &Path) -> Result<BottleConfig, String> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {e}"))?;
        toml::from_str(&contents).map_err(|e| format!("Failed to parse config: {e}"))
    }

    fn write_config(&self, path: &Path, config: &BottleConfig) -> Result<(), String> {
        let contents =
            toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize: {e}"))?;
        std::fs::write(path, contents).map_err(|e| format!("Failed to write config: {e}"))
    }
}

pub fn validate_identifier(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(format!(
            "{label} may contain only letters, numbers, '.', '-' and '_'"
        ));
    }
    Ok(())
}

pub fn resolve_existing_child(base: &Path, id: &str, label: &str) -> Result<PathBuf, String> {
    validate_identifier(id, &format!("{label} ID"))?;
    let canonical_base = base
        .canonicalize()
        .map_err(|e| format!("Failed to resolve {} directory: {e}", label.to_lowercase()))?;
    let candidate = base.join(id);
    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|_| format!("{label} '{id}' not found"))?;

    if canonical_candidate == canonical_base || !canonical_candidate.starts_with(&canonical_base) {
        return Err(format!("{label} path escapes its base directory"));
    }
    if !canonical_candidate.is_dir() {
        return Err(format!("{label} '{id}' is not a directory"));
    }
    Ok(canonical_candidate)
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

        let bottle =
            tokio_test::block_on(manager.create_bottle("Test Game", "9.14-staging", None)).unwrap();
        assert_eq!(bottle.name, "Test Game");
        assert!(bottle.prefix_path.exists());

        manager.delete_bottle(&bottle.id).unwrap();
        assert!(!bottle.prefix_path.exists());
    }

    #[test]
    fn test_config_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let manager = BottleManager::new(test_config(&tmp));

        let bottle =
            tokio_test::block_on(manager.create_bottle("Config Test", "9.0-staging", None))
                .unwrap();

        let mut config = manager.get_config(&bottle.id).unwrap();
        config.graphics.backend = GraphicsBackend::DXVK;
        manager.save_config(&bottle.id, &config).unwrap();

        let reloaded = manager.get_config(&bottle.id).unwrap();
        assert_eq!(reloaded.graphics.backend.to_string(), "dxvk");
    }

    #[test]
    fn test_rejects_invalid_bottle_ids_for_all_operations() {
        let tmp = TempDir::new().unwrap();
        let manager = BottleManager::new(test_config(&tmp));
        for invalid in ["", "..", "../outside", "nested/id", r"nested\id"] {
            assert!(manager.resolve_bottle_path(invalid).is_err());
            assert!(manager.get_config(invalid).is_err());
            assert!(manager
                .save_config(invalid, &BottleConfig::default())
                .is_err());
            assert!(manager.delete_bottle(invalid).is_err());
        }
    }

    #[test]
    fn test_get_or_create_for_steam_app_reuses_bottle() {
        let tmp = TempDir::new().unwrap();
        let manager = BottleManager::new(test_config(&tmp));
        let first = manager
            .get_or_create_for_steam_app("730", "Counter-Strike 2", "9.14-staging")
            .unwrap();
        let second = manager
            .get_or_create_for_steam_app("730", "Renamed Game", "9.14-staging")
            .unwrap();

        assert_eq!(first.id, "steam-730");
        assert_eq!(first.id, second.id);
        assert_eq!(manager.list_bottles().unwrap().len(), 1);
    }

    #[test]
    fn list_bottles_ignores_unmanaged_prefix_directories() {
        let tmp = TempDir::new().unwrap();
        let manager = BottleManager::new(test_config(&tmp));
        std::fs::create_dir_all(tmp.path().join("steam/drive_c")).unwrap();

        let managed =
            tokio_test::block_on(manager.create_bottle("Managed", "9.14-staging", None)).unwrap();
        let bottles = manager.list_bottles().unwrap();

        assert_eq!(bottles.len(), 1);
        assert_eq!(bottles[0].id, managed.id);
    }
}
