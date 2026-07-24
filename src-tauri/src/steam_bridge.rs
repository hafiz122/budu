use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::AppConfig;

/// Represents a Steam game as reported by the Windows Steam client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamApp {
    /// Steam App ID.
    pub app_id: String,
    /// Display name.
    pub name: String,
    /// Path to installed game files inside the Steam bottle.
    pub install_dir: PathBuf,
    /// Size on disk in bytes, if known.
    pub size_bytes: Option<u64>,
    /// Whether the game is currently installed.
    pub installed: bool,
}

/// Status of the Steam client integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamStatus {
    /// Whether the Steam bottle has been created.
    pub bottle_created: bool,
    /// Whether Steam GUI is installed in the bottle.
    pub steam_installed: bool,
    /// Whether SteamCMD is installed.
    pub steamcmd_installed: bool,
    /// Whether Steam is currently running.
    pub steam_running: bool,
    /// The username currently logged in, if any.
    pub logged_in_user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub app_id: String,
    pub app_name: String,
    pub progress_pct: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub rate_bytes_per_sec: u64,
}

pub struct SteamBridge {
    config: AppConfig,
}

impl SteamBridge {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Returns the path to the dedicated Steam bottle.
    pub fn steam_bottle_path(&self) -> PathBuf {
        self.config.bottles_dir.join("steam")
    }

    /// Returns the Steam client executable path inside the bottle.
    pub fn steam_exe_path(&self) -> PathBuf {
        self.steam_bottle_path()
            .join("drive_c")
            .join("Program Files (x86)")
            .join("Steam")
            .join("Steam.exe")
    }

    /// Returns the Steam library folder containing installed games.
    pub fn steamapps_path(&self) -> PathBuf {
        self.steam_bottle_path()
            .join("drive_c")
            .join("Program Files (x86)")
            .join("Steam")
            .join("steamapps")
    }

    /// Path to SteamCMD executable.
    pub fn steamcmd_exe_path(&self) -> PathBuf {
        self.steam_bottle_path()
            .join("drive_c")
            .join("steamcmd")
            .join("steamcmd.exe")
    }

    /// Check the current status of Steam integration.
    pub fn get_status(&self) -> SteamStatus {
        SteamStatus {
            bottle_created: self.steam_bottle_path().exists(),
            steam_installed: self.steam_exe_path().exists(),
            steamcmd_installed: std::path::PathBuf::from("/opt/homebrew/bin/steamcmd").exists(),
            steam_running: false,
            logged_in_user: None,
        }
    }

    /// Parse VDF (Valve Data Format) into a flat key-value structure.
    /// VDF is Valve's JSON-like format used in libraryfolders.vdf and appinfo.vdf.
    pub fn parse_vdf(&self, contents: &str) -> Result<Vec<VdfEntry>, String> {
        let mut entries = Vec::new();
        let mut current_section: Option<String> = None;
        let mut pending_section: Option<String> = None;

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Handle the opening brace after a section name (may be on next line).
            if trimmed == "{" {
                if let Some(name) = pending_section.take() {
                    current_section = Some(name);
                }
                continue;
            }

            // Handle closing brace.
            if trimmed == "}" {
                continue;
            }

            // Detect section headers on the same line as the brace: "name" {
            if trimmed.starts_with('"')
                && trimmed.ends_with('{')
                && trimmed.matches('"').count() == 2
            {
                if let Some(name) = trimmed.split('"').nth(1) {
                    current_section = Some(name.to_string());
                }
                continue;
            }

            // Detect section headers where brace is on the next line: "name"
            if trimmed.starts_with('"')
                && trimmed.matches('"').count() == 2
                && !trimmed.contains('{')
                && trimmed.chars().filter(|&c| c == '"').count() == 2
            {
                if let Some(name) = trimmed.split('"').nth(1) {
                    if !name.contains('\t') && !name.contains("  ") {
                        pending_section = Some(name.to_string());
                        continue;
                    }
                }
            }

            // Key-value pairs.
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 5 && !parts[1].is_empty() {
                let key = parts[1].to_string();
                let value = parts[3].to_string();
                entries.push(VdfEntry {
                    section: current_section.clone(),
                    key,
                    value,
                });
            }
        }

        Ok(entries)
    }

    /// Discover installed Steam games by parsing appmanifest_*.acf files
    /// and scanning for .exe files directly in common/ (for SteamCMD downloads).
    pub fn list_installed_games(&self) -> Result<Vec<SteamApp>, String> {
        let steamapps = self.steamapps_path();
        let common_dir = steamapps.join("common");

        if !steamapps.exists() {
            return Ok(Vec::new());
        }

        // Parse appmanifest_*.acf to get app IDs and names
        let mut manifest_map: std::collections::HashMap<String, (String, String)> =
            std::collections::HashMap::new();

        if let Ok(entries) = std::fs::read_dir(&steamapps) {
            for entry in entries.flatten() {
                let fname = entry.file_name();
                let fname_str = fname.to_string_lossy();
                if fname_str.starts_with("appmanifest_") && fname_str.ends_with(".acf") {
                    if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                        if let Ok(vdf) = self.parse_vdf(&contents) {
                            let mut app_id = String::new();
                            let mut name = String::new();
                            let mut installdir = String::new();
                            for e in &vdf {
                                match e.key.as_str() {
                                    "appid" => app_id = e.value.clone(),
                                    "name" => name = e.value.clone(),
                                    "installdir" => installdir = e.value.clone(),
                                    _ => {}
                                }
                            }
                            if !app_id.is_empty() {
                                manifest_map.insert(installdir, (app_id, name));
                            }
                        }
                    }
                }
            }
        }

        let mut games = Vec::new();

        if common_dir.exists() {
            for entry in std::fs::read_dir(&common_dir)
                .map_err(|e| format!("Failed to read steamapps/common: {e}"))?
                .flatten()
            {
                let path = entry.path();

                if path.is_dir() {
                    // Standard Steam layout: game in a subdirectory
                    let dir_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let (app_id, name) = manifest_map
                        .get(&dir_name)
                        .cloned()
                        .unwrap_or_else(|| (String::new(), dir_name.clone()));

                    games.push(SteamApp {
                        size_bytes: dir_size(&path),
                        installed: true,
                        app_id,
                        install_dir: path,
                        name,
                    });
                } else if path.extension().map_or(false, |e| e == "exe") {
                    // SteamCMD layout: .exe directly in common/
                    let name = path
                        .file_stem()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let app_id = find_steam_appid_nearby(&path);

                    games.push(SteamApp {
                        size_bytes: path.metadata().ok().map(|m| m.len()),
                        installed: true,
                        app_id: app_id.unwrap_or_default(),
                        install_dir: path,
                        name,
                    });
                }
            }
        }

        games.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(games)
    }
}

/// Look for a `steam_appid.txt` near the given .exe path (SteamCMD creates these).
fn find_steam_appid_nearby(exe: &std::path::Path) -> Option<String> {
    let parent = exe.parent()?;
    let appid_file = parent.join("steam_appid.txt");
    if let Ok(contents) = std::fs::read_to_string(&appid_file) {
        let id = contents.trim().to_string();
        if !id.is_empty() {
            return Some(id);
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct VdfEntry {
    pub section: Option<String>,
    pub key: String,
    pub value: String,
}

/// Recursively compute directory size.
fn dir_size(path: &PathBuf) -> Option<u64> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum::<u64>()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vdf() {
        let bridge = SteamBridge::new(AppConfig::default());
        let vdf = r#"
"libraryfolders"
{
    "0"
    {
        "path"  "/path/to/steam"
        "label" "Main Library"
    }
}
"#;
        let entries = bridge.parse_vdf(vdf).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "path");
        assert_eq!(entries[0].section.as_deref(), Some("0"));
    }

    #[test]
    fn test_status_no_bottle() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let config = crate::config::AppConfig {
            bottles_dir: tmp.path().join("nonexistent"),
            wine_dir: tmp.path().join("nonexistent"),
            ..Default::default()
        };
        let bridge = SteamBridge::new(config);
        let status = bridge.get_status();
        assert!(!status.bottle_created);
        assert!(!status.steam_installed);
    }
}
