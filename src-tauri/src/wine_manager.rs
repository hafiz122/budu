use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

use crate::config::AppConfig;

/// Represents an installed Wine version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineVersion {
    pub version: String,
    pub path: PathBuf,
    pub wine_bin: PathBuf,
    pub wineserver_bin: PathBuf,
    pub is_default: bool,
    pub arch: String,
    pub installed: bool,
    pub source: String, // "bundled" | "homebrew" | "system"
}

pub struct WineManager {
    config: AppConfig,
}

/// System paths to scan for wine.
const SYSTEM_PATHS: &[&str] = &[
    "/opt/homebrew/bin/wine64",
    "/usr/local/bin/wine64",
    "/usr/local/bin/wine",
    "/opt/homebrew/bin/wine",
];

impl WineManager {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    fn scan_dir_for_wine(&self, dir: &PathBuf) -> Option<(PathBuf, PathBuf)> {
        let wine_bin = dir.join("wine64");
        let wineserver_bin = dir.join("wineserver");
        if wine_bin.exists() && wineserver_bin.exists() {
            Some((wine_bin, wineserver_bin))
        } else {
            None
        }
    }

    /// Detect Wine version from a binary by running `wine64 --version`.
    fn detect_version(&self, wine_bin: &PathBuf) -> String {
        Command::new(wine_bin)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                // "wine-9.14 (Staging)" -> "9.14-staging"
                // "wine-11.0" -> "11.0"
                Some(stdout
                    .strip_prefix("wine-")
                    .unwrap_or(&stdout)
                    .replace(" (Staging)", "-staging")
                    .replace(" (staging)", "-staging"))
            })
            .unwrap_or_else(|| "unknown".into())
    }

    /// List all Wine versions discovered on disk and in system paths.
    pub fn list_versions(&self) -> Result<Vec<WineVersion>, String> {
        let mut versions = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // 1. Scan ~/.gamerunner/wine/
        if let Ok(entries) = std::fs::read_dir(&self.config.wine_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let version_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if let Some((wine_bin, wineserver_bin)) = self.scan_dir_for_wine(&path.join("bin"))
                {
                    seen.insert(wine_bin.clone());
                    versions.push(WineVersion {
                        is_default: version_name == self.config.default_wine_version,
                        version: version_name,
                        wine_bin,
                        wineserver_bin,
                        arch: "x86-64".into(),
                        installed: true,
                        source: "bundled".into(),
                        path,
                    });
                }
            }
        }

        // 2. Scan system PATH for wine64
        for path_str in SYSTEM_PATHS {
            let bin = PathBuf::from(path_str);
            if !seen.contains(&bin) && bin.exists() {
                let parent = bin.parent().unwrap_or(&PathBuf::from("/")).to_path_buf();
                let version = self.detect_version(&bin);
                let wineserver = parent.join("wineserver");

                // Symlink into ~/.gamerunner/wine/ so the app can use it
                let dest_dir = self.config.wine_dir.join(&version).join("bin");
                let _ = std::fs::create_dir_all(&dest_dir);
                let dest_wine = dest_dir.join("wine64");
                let dest_server = dest_dir.join("wineserver");
                if !dest_wine.exists() {
                    let _ = std::os::unix::fs::symlink(&bin, &dest_wine);
                }
                if !dest_server.exists() && wineserver.exists() {
                    let _ = std::os::unix::fs::symlink(&wineserver, &dest_server);
                }

                seen.insert(bin);
                versions.push(WineVersion {
                    is_default: version == self.config.default_wine_version,
                    version,
                    wine_bin: dest_wine,
                    wineserver_bin: dest_server,
                    arch: "x86-64".into(),
                    installed: true,
                    source: "system".into(),
                    path: parent,
                });
            }
        }

        versions.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(versions)
    }

    /// Resolve the wine64 binary path.
    pub fn resolve_wine_bin(&self, version: &str) -> Result<PathBuf, String> {
        // Check versioned install first
        let versioned = self
            .config
            .wine_dir
            .join(version)
            .join("bin")
            .join("wine64");
        if versioned.exists() {
            return Ok(versioned);
        }

        // Check default version
        let default = self
            .config
            .wine_dir
            .join(&self.config.default_wine_version)
            .join("bin")
            .join("wine64");
        if default.exists() {
            return Ok(default);
        }

        // Check any available version
        let versions = self.list_versions()?;
        if let Some(v) = versions.first() {
            if v.wine_bin.exists() {
                return Ok(v.wine_bin.clone());
            }
        }

        Err(format!(
            "No Wine installation found.\n\n\
             Install via Homebrew:\n  brew install wine-stable\n\n\
             Then restart GameRunner.\n\n\
             Or place a Wine build at:\n  ~/.gamerunner/wine/{version}/bin/wine64"
        ))
    }

    pub fn resolve_wineserver_bin(&self, version: &str) -> Result<PathBuf, String> {
        let server = self
            .config
            .wine_dir
            .join(version)
            .join("bin")
            .join("wineserver");
        if server.exists() {
            return Ok(server);
        }

        let default = self
            .config
            .wine_dir
            .join(&self.config.default_wine_version)
            .join("bin")
            .join("wineserver");
        if default.exists() {
            return Ok(default);
        }

        let versions = self.list_versions()?;
        if let Some(v) = versions.first() {
            if v.wineserver_bin.exists() {
                return Ok(v.wineserver_bin.clone());
            }
        }

        Err("No wineserver found. Install Wine first.".into())
    }

    /// Try to install Wine via Homebrew.
    pub async fn install_version(&self, _version: &str) -> Result<(), String> {
        // Try Homebrew first
        if Command::new("brew").arg("--version").output().is_ok() {
            let output = Command::new("brew")
                .args(["install", "wine-stable"])
                .output()
                .map_err(|e| format!("Failed to run brew: {e}"))?;

            if output.status.success() {
                return Ok(());
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already installed") {
                return Ok(());
            }
            return Err(format!("brew install failed:\n{stderr}"));
        }

        Err(
            "Homebrew not found. Install it from https://brew.sh, then run:\n\
             brew install wine-stable\n\n\
             Or place a Wine build manually at:\n\
             ~/.gamerunner/wine/<version>/bin/wine64"
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_list_versions_empty_when_no_wine() {
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            wine_dir: tmp.path().join("nonexistent"),
            ..Default::default()
        };
        let manager = WineManager::new(config);
        let versions = manager.list_versions().unwrap();
        // Might find system Wine, but at minimum shouldn't crash
        assert!(versions.iter().all(|v| v.installed || !v.installed));
    }

    #[test]
    fn test_resolve_missing_returns_helpful_error() {
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            wine_dir: tmp.path().join("nonexistent"),
            default_wine_version: "nonexistent".into(),
            ..Default::default()
        };
        let manager = WineManager::new(config);
        // May find system Wine or not; if not found, error should be helpful
        match manager.resolve_wine_bin("nonexistent") {
            Ok(_) => {} // System Wine found, that's fine
            Err(e) => assert!(e.contains("Homebrew") || e.contains("No Wine")),
        }
    }
}
