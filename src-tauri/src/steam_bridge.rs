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

    /// Returns the path to the shared Steam prefix.
    ///
    /// Releases before the shared-storage split installed Steam in
    /// `bottles/steam`. Prefer that prefix when it contains a Steam client so
    /// existing libraries continue to work without being copied.
    pub fn steam_bottle_path(&self) -> PathBuf {
        let current = self.config.data_dir.join("steam");
        let legacy = self.config.bottles_dir.join("steam");
        if steam_executable_in(&current).is_some() || !legacy.exists() {
            current
        } else {
            legacy
        }
    }

    /// Returns the Steam client executable path inside the bottle.
    pub fn steam_exe_path(&self) -> PathBuf {
        let prefix = self.steam_bottle_path();
        steam_executable_in(&prefix).unwrap_or_else(|| {
            prefix
                .join("drive_c")
                .join("Program Files (x86)")
                .join("Steam")
                .join("steam.exe")
        })
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

    /// Locate the host SteamCMD executable. Finder-launched macOS apps do not
    /// reliably inherit the user's shell PATH, so check Homebrew's standard
    /// Apple Silicon and Intel locations first.
    pub fn steamcmd_bin_path(&self) -> Option<PathBuf> {
        find_host_executable(
            "steamcmd",
            &[
                PathBuf::from("/opt/homebrew/bin/steamcmd"),
                PathBuf::from("/usr/local/bin/steamcmd"),
            ],
        )
    }

    /// Prepare an isolated SteamCMD destination for an app.
    ///
    /// SteamCMD writes game files directly into `force_install_dir`; pointing
    /// it at `steamapps/common` therefore merges every downloaded game. New
    /// apps use a per-App-ID staging directory. Existing canonical installs
    /// reuse the directory named by their manifest.
    pub fn prepare_steamcmd_download(&self, app_id: &str) -> Result<PathBuf, String> {
        validate_app_id(app_id)?;
        let steamapps = self.steamapps_path();
        let common = steamapps.join("common");
        std::fs::create_dir_all(&common)
            .map_err(|error| format!("Failed to create Steam library: {error}"))?;

        let canonical_manifest = steamapps.join(format!("appmanifest_{app_id}.acf"));
        let install = if canonical_manifest.is_file() {
            let contents = std::fs::read_to_string(&canonical_manifest)
                .map_err(|error| format!("Failed to read Steam manifest: {error}"))?;
            let install_dir = vdf_value(&contents, "installdir")
                .ok_or_else(|| "Steam manifest is missing installdir".to_string())?;
            validate_install_dir(&install_dir)?;
            common.join(install_dir)
        } else {
            common.join(".gamerunner-downloads").join(app_id)
        };
        std::fs::create_dir_all(install.join("steamapps"))
            .map_err(|error| format!("Failed to create isolated download directory: {error}"))?;

        // SteamCMD expects its own manifest below force_install_dir. Seed it
        // from the canonical Steam library manifest for efficient validation.
        if canonical_manifest.is_file() {
            std::fs::copy(
                &canonical_manifest,
                install
                    .join("steamapps")
                    .join(format!("appmanifest_{app_id}.acf")),
            )
            .map_err(|error| format!("Failed to seed SteamCMD manifest: {error}"))?;
        }
        Ok(install)
    }

    /// Move a completed SteamCMD download into canonical Steam library layout.
    pub fn finalize_steamcmd_download(
        &self,
        app_id: &str,
        download_dir: &std::path::Path,
    ) -> Result<PathBuf, String> {
        validate_app_id(app_id)?;
        let steamapps = self.steamapps_path();
        let common = steamapps.join("common");
        let nested_manifest = download_dir
            .join("steamapps")
            .join(format!("appmanifest_{app_id}.acf"));
        let contents = std::fs::read_to_string(&nested_manifest).map_err(|error| {
            format!(
                "SteamCMD completed without {}: {error}",
                nested_manifest.display()
            )
        })?;
        if vdf_value(&contents, "appid").as_deref() != Some(app_id) {
            return Err("SteamCMD manifest App ID does not match the requested app".into());
        }
        let install_dir = vdf_value(&contents, "installdir")
            .ok_or_else(|| "SteamCMD manifest is missing installdir".to_string())?;
        validate_install_dir(&install_dir)?;
        let destination = common.join(&install_dir);

        if download_dir != destination {
            if destination.exists() {
                return Err(format!(
                    "Refusing to merge Steam app {app_id} into existing {}",
                    destination.display()
                ));
            }
            std::fs::rename(download_dir, &destination)
                .map_err(|error| format!("Failed to finalize Steam download: {error}"))?;
        }

        let finalized_manifest = destination
            .join("steamapps")
            .join(format!("appmanifest_{app_id}.acf"));
        std::fs::copy(
            &finalized_manifest,
            steamapps.join(format!("appmanifest_{app_id}.acf")),
        )
        .map_err(|error| format!("Failed to register app with Steam: {error}"))?;
        Ok(destination)
    }

    /// Check the current status of Steam integration.
    pub fn get_status(&self) -> SteamStatus {
        SteamStatus {
            bottle_created: self.steam_bottle_path().exists(),
            steam_installed: self.steam_exe_path().exists(),
            steamcmd_installed: self.steamcmd_bin_path().is_some(),
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

        let staged_manifest_dirs = std::fs::read_dir(common_dir.join(".gamerunner-downloads"))
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|entry| entry.path().join("steamapps"))
            .collect::<Vec<_>>();
        let mut manifest_dirs = vec![steamapps.clone(), common_dir.join("steamapps")];
        manifest_dirs.extend(staged_manifest_dirs);

        for manifest_dir in manifest_dirs {
            if let Ok(entries) = std::fs::read_dir(manifest_dir) {
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
        }

        let mut games = Vec::new();

        let staged_root = common_dir.join(".gamerunner-downloads");
        if let Ok(entries) = std::fs::read_dir(&staged_root) {
            for entry in entries.flatten() {
                let install_dir = entry.path();
                if !install_dir.is_dir() {
                    continue;
                }
                let manifest = install_dir.join("steamapps").join(format!(
                    "appmanifest_{}.acf",
                    entry.file_name().to_string_lossy()
                ));
                let Ok(contents) = std::fs::read_to_string(manifest) else {
                    continue;
                };
                let Ok(vdf) = self.parse_vdf(&contents) else {
                    continue;
                };
                let app_id = vdf
                    .iter()
                    .find(|entry| entry.key == "appid")
                    .map(|entry| entry.value.clone())
                    .unwrap_or_default();
                let name = vdf
                    .iter()
                    .find(|entry| entry.key == "name")
                    .map(|entry| entry.value.clone())
                    .unwrap_or_default();
                if app_id.is_empty() || name.is_empty() {
                    continue;
                }
                games.push(SteamApp {
                    size_bytes: dir_size(&install_dir),
                    installed: true,
                    app_id,
                    install_dir,
                    name,
                });
            }
        }

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

                    let Some((app_id, name)) = manifest_map.get(&dir_name).cloned() else {
                        continue;
                    };

                    games.push(SteamApp {
                        size_bytes: dir_size(&path),
                        installed: true,
                        app_id,
                        install_dir: path,
                        name,
                    });
                } else if path.extension().is_some_and(|e| e == "exe") {
                    // SteamCMD layout: .exe directly in common/
                    let name = path
                        .file_stem()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let normalized_name = name.to_ascii_lowercase();
                    if normalized_name.contains("crashhandler")
                        || normalized_name.contains("uninstall")
                        || normalized_name.starts_with("unins")
                    {
                        continue;
                    }
                    let app_id = self.app_id_for_executable(&path);

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

    /// Identify a Steam-managed executable from nearby app manifests.
    ///
    /// This accepts both the standard `steamapps/common/Game/Game.exe`
    /// layout and the older SteamCMD layout where games and a nested
    /// `steamapps` directory were placed directly inside `common`.
    pub fn app_id_for_executable(&self, executable: &std::path::Path) -> Option<String> {
        app_id_for_executable(executable)
    }

    /// Locate the Wine prefix containing a Steam-managed executable.
    pub fn prefix_for_executable(&self, executable: &std::path::Path) -> Option<PathBuf> {
        executable
            .ancestors()
            .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "drive_c"))
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf)
            .filter(|prefix| steam_executable_in(prefix).is_some())
    }
}

fn find_host_executable(name: &str, standard_paths: &[PathBuf]) -> Option<PathBuf> {
    standard_paths
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .or_else(|| {
            std::env::var_os("PATH").and_then(|path| {
                std::env::split_paths(&path)
                    .map(|directory| directory.join(name))
                    .find(|candidate| candidate.is_file())
            })
        })
}

fn steam_executable_in(prefix: &std::path::Path) -> Option<PathBuf> {
    let steam_dir = prefix
        .join("drive_c")
        .join("Program Files (x86)")
        .join("Steam");
    ["steam.exe", "Steam.exe"]
        .into_iter()
        .map(|name| steam_dir.join(name))
        .find(|path| path.is_file())
}

fn app_id_for_executable(executable: &std::path::Path) -> Option<String> {
    let executable = executable.canonicalize().ok()?;
    let executable_stem = executable
        .file_stem()?
        .to_string_lossy()
        .to_ascii_lowercase();

    let mut manifest_dirs = Vec::new();
    for ancestor in executable.ancestors().take(10) {
        if ancestor.file_name().is_some_and(|name| name == "steamapps") {
            manifest_dirs.push(ancestor.to_path_buf());
        }
        let nested = ancestor.join("steamapps");
        if nested.is_dir() {
            manifest_dirs.push(nested);
        }
    }
    manifest_dirs.sort();
    manifest_dirs.dedup();

    let mut saw_manifest = false;
    for manifest_dir in manifest_dirs {
        let Ok(entries) = std::fs::read_dir(&manifest_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if !file_name.starts_with("appmanifest_") || !file_name.ends_with(".acf") {
                continue;
            }
            saw_manifest = true;
            let Ok(contents) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            let app_id = vdf_value(&contents, "appid");
            let install_dir = vdf_value(&contents, "installdir");
            let name = vdf_value(&contents, "name");
            let matches_executable = [install_dir.as_deref(), name.as_deref()]
                .into_iter()
                .flatten()
                .map(|value| value.to_ascii_lowercase())
                .any(|value| {
                    executable_stem == value
                        || executable_stem.starts_with(&value)
                        || executable.components().any(|part| {
                            part.as_os_str()
                                .to_string_lossy()
                                .eq_ignore_ascii_case(&value)
                        })
                });
            if matches_executable
                && app_id
                    .as_deref()
                    .is_some_and(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
            {
                return app_id;
            }
        }
    }

    // SteamCMD sometimes only writes this marker beside the executable.
    (!saw_manifest)
        .then(|| find_steam_appid_nearby(&executable))
        .flatten()
        .filter(|id| id.chars().all(|c| c.is_ascii_digit()))
}

fn vdf_value(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let parts: Vec<&str> = line.trim().split('"').collect();
        (parts.len() >= 4 && parts[1].eq_ignore_ascii_case(key)).then(|| parts[3].to_string())
    })
}

fn validate_app_id(app_id: &str) -> Result<(), String> {
    if app_id.is_empty() || !app_id.chars().all(|character| character.is_ascii_digit()) {
        return Err("Steam App ID must contain only digits".into());
    }
    Ok(())
}

fn validate_install_dir(install_dir: &str) -> Result<(), String> {
    let mut components = std::path::Path::new(install_dir).components();
    let valid = matches!(components.next(), Some(std::path::Component::Normal(_)))
        && components.next().is_none()
        && install_dir != "."
        && install_dir != "..";
    if !valid {
        return Err("Steam manifest contains an unsafe install directory".into());
    }
    Ok(())
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

    #[test]
    fn prefers_legacy_prefix_when_it_contains_steam() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let config = crate::config::AppConfig {
            data_dir: tmp.path().to_path_buf(),
            bottles_dir: tmp.path().join("bottles"),
            ..Default::default()
        };
        let legacy_exe = config
            .bottles_dir
            .join("steam/drive_c/Program Files (x86)/Steam/steam.exe");
        std::fs::create_dir_all(legacy_exe.parent().unwrap()).unwrap();
        std::fs::write(&legacy_exe, b"steam").unwrap();

        let bridge = SteamBridge::new(config.clone());
        assert_eq!(bridge.steam_bottle_path(), config.bottles_dir.join("steam"));
        assert_eq!(bridge.steam_exe_path(), legacy_exe);
    }

    #[test]
    fn detects_app_id_in_standard_and_legacy_steamcmd_layouts() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();

        let standard = tmp.path().join("steamapps/common/Raft/Raft.exe");
        std::fs::create_dir_all(standard.parent().unwrap()).unwrap();
        std::fs::write(&standard, b"exe").unwrap();
        std::fs::write(
            tmp.path().join("steamapps/appmanifest_648800.acf"),
            r#""AppState"
{
    "appid" "648800"
    "name" "Raft"
    "installdir" "Raft"
}"#,
        )
        .unwrap();
        assert_eq!(app_id_for_executable(&standard).as_deref(), Some("648800"));

        let legacy = tmp.path().join("legacy/common/Phasmophobia.exe");
        let nested_manifests = tmp.path().join("legacy/common/steamapps");
        std::fs::create_dir_all(&nested_manifests).unwrap();
        std::fs::write(&legacy, b"exe").unwrap();
        std::fs::write(
            nested_manifests.join("appmanifest_739630.acf"),
            r#""AppState"
{
    "appid" "739630"
    "name" "Phasmophobia"
    "installdir" "Phasmophobia"
}"#,
        )
        .unwrap();
        assert_eq!(app_id_for_executable(&legacy).as_deref(), Some("739630"));
    }

    #[test]
    fn steamcmd_downloads_are_isolated_and_normalized() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("data");
        let config = AppConfig {
            data_dir: data_dir.clone(),
            bottles_dir: tmp.path().join("bottles"),
            ..Default::default()
        };
        let bridge = SteamBridge::new(config);

        let staging = bridge.prepare_steamcmd_download("648800").unwrap();
        assert!(staging.ends_with(".gamerunner-downloads/648800"));
        std::fs::write(staging.join("Raft.exe"), b"exe").unwrap();
        std::fs::write(
            staging.join("steamapps/appmanifest_648800.acf"),
            r#""AppState"
{
    "appid" "648800"
    "name" "Raft"
    "installdir" "Raft"
}"#,
        )
        .unwrap();

        let finalized = bridge
            .finalize_steamcmd_download("648800", &staging)
            .unwrap();
        assert_eq!(
            finalized,
            bridge.steamapps_path().join("common").join("Raft")
        );
        assert!(finalized.join("Raft.exe").is_file());
        assert!(bridge
            .steamapps_path()
            .join("appmanifest_648800.acf")
            .is_file());
        assert!(!staging.exists());
    }

    #[test]
    fn lists_completed_terminal_downloads_before_normalization() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            data_dir: tmp.path().join("data"),
            bottles_dir: tmp.path().join("bottles"),
            ..Default::default()
        };
        let bridge = SteamBridge::new(config);
        let staging = bridge.prepare_steamcmd_download("648800").unwrap();
        std::fs::write(staging.join("Raft.exe"), b"exe").unwrap();
        std::fs::write(
            staging.join("steamapps/appmanifest_648800.acf"),
            r#""AppState"
{
    "appid" "648800"
    "name" "Raft"
    "installdir" "Raft"
}"#,
        )
        .unwrap();

        let games = bridge.list_installed_games().unwrap();

        assert_eq!(games.len(), 1);
        assert_eq!(games[0].app_id, "648800");
        assert_eq!(games[0].install_dir, staging);
    }

    #[test]
    fn steamcmd_rejects_unsafe_ids_and_install_directories() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            data_dir: tmp.path().join("data"),
            bottles_dir: tmp.path().join("bottles"),
            ..Default::default()
        };
        let bridge = SteamBridge::new(config);
        assert!(bridge.prepare_steamcmd_download("../648800").is_err());

        let staging = bridge.prepare_steamcmd_download("648800").unwrap();
        std::fs::write(
            staging.join("steamapps/appmanifest_648800.acf"),
            r#""AppState"
{
    "appid" "648800"
    "installdir" "../outside"
}"#,
        )
        .unwrap();
        assert!(bridge
            .finalize_steamcmd_download("648800", &staging)
            .is_err());
    }

    #[test]
    fn lists_games_from_legacy_nested_manifests_without_data_directories() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let config = crate::config::AppConfig {
            data_dir: tmp.path().to_path_buf(),
            bottles_dir: tmp.path().join("bottles"),
            ..Default::default()
        };
        let common = config
            .data_dir
            .join("steam/drive_c/Program Files (x86)/Steam/steamapps/common");
        std::fs::create_dir_all(common.join("steamapps")).unwrap();
        std::fs::create_dir_all(common.join("Raft_Data")).unwrap();
        std::fs::write(common.join("Raft.exe"), b"exe").unwrap();
        std::fs::write(common.join("UnityCrashHandler64.exe"), b"exe").unwrap();
        std::fs::write(
            common.join("steamapps/appmanifest_648800.acf"),
            r#""AppState"
{
    "appid" "648800"
    "name" "Raft"
    "installdir" "Raft"
}"#,
        )
        .unwrap();

        let games = SteamBridge::new(config).list_installed_games().unwrap();
        assert_eq!(games.len(), 1);
        let raft = games
            .iter()
            .find(|game| game.name == "Raft")
            .expect("Raft executable should be listed");
        assert_eq!(raft.app_id, "648800");
        assert!(games.iter().all(|game| game.name != "Raft_Data"));
    }
}
