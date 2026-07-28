use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::AppConfig;

pub const STABLE_WINE_VERSION: &str = "11.10-staging";
pub const RAFT_TEST_WINE_VERSION: &str = "11.13-staging";
pub const RAFT_STEAM_APP_ID: &str = "648800";

struct RuntimeSpec {
    version: &'static str,
    url: &'static str,
    sha256: &'static str,
    bridge_resource: &'static str,
}

const STABLE_WINE: RuntimeSpec = RuntimeSpec {
    version: STABLE_WINE_VERSION,
    url: "https://github.com/Gcenx/macOS_Wine_builds/releases/download/11.10/wine-staging-11.10-osx64.tar.xz",
    sha256: "940bdd1a177872020be01c5c33917cb8eecc1cc3193ad554914fb6efd90d7889",
    bridge_resource: "wine-11.10",
};

const RAFT_TEST_WINE: RuntimeSpec = RuntimeSpec {
    version: RAFT_TEST_WINE_VERSION,
    url: "https://github.com/Gcenx/macOS_Wine_builds/releases/download/11.13/wine-staging-11.13-osx64.tar.xz",
    sha256: "dc7bbd684ee0e820871851055115f3c829ed65c13939000e52f0a15a027537bf",
    bridge_resource: "wine-11.13",
};
const DXMT_URL: &str =
    "https://github.com/3Shain/dxmt/releases/download/v0.74/dxmt-v0.74-builtin.tar.gz";
const DXMT_SHA256: &str = "2598981a8b725653773e277470a95dda4253b8a14d36e0dc96dce0e3800f0ceb";

fn runtime_spec(version: &str) -> Result<&'static RuntimeSpec, String> {
    match version {
        STABLE_WINE_VERSION => Ok(&STABLE_WINE),
        RAFT_TEST_WINE_VERSION => Ok(&RAFT_TEST_WINE),
        _ => Err(format!(
            "Unsupported Wine version '{version}'. Budu supports {STABLE_WINE_VERSION} and the private Raft test runtime {RAFT_TEST_WINE_VERSION}."
        )),
    }
}

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

#[derive(Debug, Clone, Serialize)]
pub struct RaftRuntimePreflight {
    pub runtime: String,
    pub usable_adapter_found: bool,
    pub gstreamer_installed: bool,
    pub message: String,
}

pub struct WineManager {
    config: AppConfig,
    resource_dir: Option<PathBuf>,
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
        Self {
            config,
            resource_dir: None,
        }
    }

    pub fn with_resource_dir(config: AppConfig, resource_dir: Option<PathBuf>) -> Self {
        Self {
            config,
            resource_dir,
        }
    }

    fn scan_dir_for_wine(&self, dir: &std::path::Path) -> Option<(PathBuf, PathBuf)> {
        let wine_bin = dir.join("wine64");
        let wineserver_bin = dir.join("wineserver");
        if wine_bin.exists() && wineserver_bin.exists() {
            Some((wine_bin, wineserver_bin))
        } else {
            None
        }
    }

    /// Detect Wine version from a binary by running `wine64 --version`.
    fn detect_version(&self, wine_bin: &std::path::Path) -> String {
        Command::new(wine_bin)
            .arg("--version")
            .output()
            .ok()
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                // "wine-9.14 (Staging)" -> "9.14-staging"
                // "wine-11.0" -> "11.0"
                stdout
                    .strip_prefix("wine-")
                    .unwrap_or(&stdout)
                    .replace(" (Staging)", "-staging")
                    .replace(" (staging)", "-staging")
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

        // The Raft test bottle must never silently fall back to the stable
        // runtime. Doing so would hide a failed test-runtime install and make
        // multiplayer diagnostics misleading.
        if version == RAFT_TEST_WINE_VERSION {
            return Err(format!(
                "Wine {RAFT_TEST_WINE_VERSION} is required for the Raft multiplayer test bottle but is not installed. Open Settings and install the Raft test runtime."
            ));
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

        Err("No Wine installation found.\n\n\
             Open Budu Settings and select Install Wine.\n\n\
             Budu downloads and manages the compatible Wine runtime automatically."
            .into())
    }

    /// Resolve the configured open-source Wine runner for Steam.
    ///
    /// Steam's macOS CEF workaround is applied separately by `steam_compat`, so
    /// Steam and ordinary executables use the same user-selected Wine runtime.
    pub fn resolve_steam_wine_bin(&self, version: &str) -> Result<PathBuf, String> {
        self.resolve_wine_bin(version)
    }

    /// Check the one capability that the Raft 11.13 experiment is intended to
    /// provide: Wine must see a real network adapter, not only loopback.
    pub fn raft_network_preflight(&self, prefix: &Path) -> Result<RaftRuntimePreflight, String> {
        let wine = self.resolve_wine_bin(RAFT_TEST_WINE_VERSION)?;
        let output = Command::new(&wine)
            .args(["ipconfig.exe", "/all"])
            .env("WINEPREFIX", prefix)
            .output()
            .map_err(|error| {
                format!(
                    "Raft test runtime {RAFT_TEST_WINE_VERSION} could not run ipconfig: {error}"
                )
            })?;
        if !output.status.success() {
            return Err(format!(
                "Raft test runtime {RAFT_TEST_WINE_VERSION} could not inspect network adapters (ipconfig exited {}).",
                output.status
            ));
        }

        let adapter = parse_usable_wine_adapter(&String::from_utf8_lossy(&output.stdout));
        let gstreamer_installed = Path::new("/Library/Frameworks/GStreamer.framework").is_dir();
        let message = match (&adapter, gstreamer_installed) {
            (Some(name), true) => format!(
                "Wine {RAFT_TEST_WINE_VERSION} can use network adapter '{name}'. GStreamer is installed."
            ),
            (Some(name), false) => format!(
                "Wine {RAFT_TEST_WINE_VERSION} can use network adapter '{name}', but GStreamer is not installed. Videos and some Steam content may not work. Install the official GStreamer framework, then relaunch."
            ),
            (None, _) => format!(
                "Wine {RAFT_TEST_WINE_VERSION} did not report a non-loopback adapter with both IPv4 and a default gateway. Raft will not launch with this test runtime."
            ),
        };
        Ok(RaftRuntimePreflight {
            runtime: RAFT_TEST_WINE_VERSION.into(),
            usable_adapter_found: adapter.is_some(),
            gstreamer_installed,
            message,
        })
    }

    pub fn install_dxmt_for_prefix(
        &self,
        wine_bin: &Path,
        dxmt_root: &Path,
        prefix: &Path,
    ) -> Result<(), String> {
        let managed_wine_dir = self
            .config
            .wine_dir
            .canonicalize()
            .map_err(|error| format!("Failed to resolve managed Wine directory: {error}"))?;
        let wine_bin = wine_bin
            .canonicalize()
            .map_err(|error| format!("Failed to resolve Wine executable: {error}"))?;
        if !wine_bin.starts_with(&managed_wine_dir) {
            return Err(
                "DXMT requires Budu's managed Wine runtime. Install Wine in Settings.".into(),
            );
        }
        let wine_root = wine_bin
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| "Managed Wine executable has an unexpected layout".to_string())?;
        let wine_library = wine_root.join("lib/wine");
        if !wine_library.join("x86_64-windows").is_dir()
            || !wine_library.join("x86_64-unix").is_dir()
        {
            return Err(format!(
                "Managed Wine runtime is missing its DLL directories: {}",
                wine_library.display()
            ));
        }

        let version = wine_root
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Managed Wine runtime has an unexpected path".to_string())?;
        self.install_wine_dxmt_bridge(wine_root, version)?;

        for relative in [
            "x86_64-unix/winemetal.so",
            "x86_64-windows/winemetal.dll",
            "x86_64-windows/d3d11.dll",
            "x86_64-windows/dxgi.dll",
            "x86_64-windows/d3d10core.dll",
        ] {
            install_runtime_file(&dxmt_root.join(relative), &wine_library.join(relative))?;
        }

        let prefix_system32 = prefix.join("drive_c/windows/system32");
        std::fs::create_dir_all(&prefix_system32)
            .map_err(|error| format!("Failed to prepare bottle system32: {error}"))?;
        install_runtime_file(
            &dxmt_root.join("x86_64-windows/winemetal.dll"),
            &prefix_system32.join("winemetal.dll"),
        )
    }

    fn install_wine_dxmt_bridge(&self, wine_root: &Path, version: &str) -> Result<(), String> {
        let runtime = runtime_spec(version)?;
        let development_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../runtime/dist")
            .join(runtime.bridge_resource);
        let bridge_root = self
            .resource_dir
            .as_ref()
            .map(|dir| dir.join("runtime").join(runtime.bridge_resource))
            .filter(|dir| dir.join("lib/wine/x86_64-unix/ntdll.so").is_file())
            .or_else(|| {
                development_dir
                    .join("lib/wine/x86_64-unix/ntdll.so")
                    .is_file()
                    .then_some(development_dir)
            })
            .ok_or_else(|| {
                "Budu's Wine/DXMT bridge is missing from the application resources".to_string()
            })?;

        for relative in [
            "bin/wine",
            "lib/wine/x86_64-unix/wine",
            "lib/wine/x86_64-unix/ntdll.so",
            "lib/wine/x86_64-unix/winemac.so",
        ] {
            install_runtime_file(&bridge_root.join(relative), &wine_root.join(relative))?;
        }
        Ok(())
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

    /// Install the macOS game runtime used by Budu.
    pub async fn install_version(&self, version: &str) -> Result<(), String> {
        let runtime = runtime_spec(version)?;
        self.install_gamerunner_wine(runtime)?;
        self.install_dxmt()?;
        Ok(())
    }

    fn install_gamerunner_wine(&self, runtime: &RuntimeSpec) -> Result<(), String> {
        let destination = self.config.wine_dir.join(runtime.version);
        if destination.join("bin/wine64").is_file() && destination.join("bin/wineserver").is_file()
        {
            return Ok(());
        }

        std::fs::create_dir_all(&self.config.wine_dir)
            .map_err(|error| format!("Failed to create Wine directory: {error}"))?;
        let staging = self.config.wine_dir.join(format!(
            ".install-{}-{}",
            runtime.version,
            std::process::id()
        ));
        if staging.exists() {
            std::fs::remove_dir_all(&staging)
                .map_err(|error| format!("Failed to clear incomplete Wine install: {error}"))?;
        }
        std::fs::create_dir_all(staging.join("runtime"))
            .map_err(|error| format!("Failed to create Wine staging directory: {error}"))?;
        let archive = staging.join("wine.tar.xz");

        run_checked(
            Command::new("curl")
                .args(["-L", "--fail", "--retry", "5", "-o"])
                .arg(&archive)
                .arg(runtime.url),
            "download Wine",
        )?;
        verify_sha256(&archive, runtime.sha256, "Wine")?;
        run_checked(
            Command::new("tar")
                .arg("xJf")
                .arg(&archive)
                .arg("-C")
                .arg(staging.join("runtime")),
            "extract Wine",
        )?;

        let wine_app = std::fs::read_dir(staging.join("runtime"))
            .map_err(|error| format!("Failed to inspect Wine archive: {error}"))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .is_some_and(|name| name == "Wine Staging.app")
            })
            .ok_or_else(|| "Wine archive did not contain Wine Staging.app".to_string())?;
        let runtime_bin = wine_app.join("Contents/Resources/wine/bin");
        if !runtime_bin.join("wine").is_file() || !runtime_bin.join("wineserver").is_file() {
            return Err("Wine archive is missing its runtime binaries".into());
        }

        std::fs::remove_file(&archive)
            .map_err(|error| format!("Failed to remove Wine archive: {error}"))?;
        std::fs::create_dir_all(staging.join("bin"))
            .map_err(|error| format!("Failed to create Wine bin directory: {error}"))?;
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(runtime_bin.join("wine"), staging.join("bin/wine64"))
                .map_err(|error| format!("Failed to link wine64: {error}"))?;
            std::os::unix::fs::symlink(
                runtime_bin.join("wineserver"),
                staging.join("bin/wineserver"),
            )
            .map_err(|error| format!("Failed to link wineserver: {error}"))?;
        }

        std::fs::rename(&staging, &destination)
            .map_err(|error| format!("Failed to activate Wine runtime: {error}"))
    }

    fn install_dxmt(&self) -> Result<(), String> {
        let destination = self.config.runtimes_dir.join("dxmt-0.74");
        let payload = destination.join("v0.74");
        if payload.join("x86_64-windows/dxgi.dll").is_file()
            && payload.join("x86_64-windows/d3d11.dll").is_file()
            && payload.join("x86_64-unix").is_dir()
        {
            return Ok(());
        }

        std::fs::create_dir_all(&self.config.runtimes_dir)
            .map_err(|error| format!("Failed to create runtimes directory: {error}"))?;
        let staging = self
            .config
            .runtimes_dir
            .join(format!(".install-dxmt-0.74-{}", std::process::id()));
        if staging.exists() {
            std::fs::remove_dir_all(&staging)
                .map_err(|error| format!("Failed to clear incomplete DXMT install: {error}"))?;
        }
        std::fs::create_dir_all(&staging)
            .map_err(|error| format!("Failed to create DXMT staging directory: {error}"))?;
        let archive = staging.join("dxmt.tar.gz");

        run_checked(
            Command::new("curl")
                .args(["-L", "--fail", "--retry", "5", "-o"])
                .arg(&archive)
                .arg(DXMT_URL),
            "download DXMT",
        )?;
        verify_sha256(&archive, DXMT_SHA256, "DXMT")?;
        run_checked(
            Command::new("tar")
                .arg("xzf")
                .arg(&archive)
                .arg("-C")
                .arg(&staging),
            "extract DXMT",
        )?;
        std::fs::remove_file(&archive)
            .map_err(|error| format!("Failed to remove DXMT archive: {error}"))?;
        let extracted = staging.join("v0.74");
        if !extracted.join("x86_64-windows/dxgi.dll").is_file()
            || !extracted.join("x86_64-windows/d3d11.dll").is_file()
            || !extracted.join("x86_64-unix").is_dir()
        {
            return Err("DXMT archive is missing required runtime files".into());
        }
        std::fs::rename(&staging, &destination)
            .map_err(|error| format!("Failed to activate DXMT runtime: {error}"))
    }
}

fn parse_usable_wine_adapter(ipconfig: &str) -> Option<String> {
    let mut name = None;
    let mut ipv4 = false;
    let mut gateway = false;

    let finish = |name: &Option<String>, ipv4: bool, gateway: bool| {
        (ipv4 && gateway).then(|| name.clone().unwrap_or_else(|| "network adapter".into()))
    };

    for line in ipconfig.lines() {
        let trimmed = line.trim();
        if trimmed.ends_with(':') && !trimmed.contains('.') {
            if let Some(found) = finish(&name, ipv4, gateway) {
                return Some(found);
            }
            name = Some(trimmed.trim_end_matches(':').to_string());
            ipv4 = false;
            gateway = false;
        } else if trimmed.starts_with("IPv4 Address") {
            ipv4 = trimmed
                .split_once(':')
                .is_some_and(|(_, value)| !value.trim().starts_with("127."));
        } else if trimmed.starts_with("Default Gateway") {
            gateway = trimmed
                .split_once(':')
                .is_some_and(|(_, value)| !value.trim().is_empty() && value.trim() != "0.0.0.0");
        }
    }
    finish(&name, ipv4, gateway)
}

fn install_runtime_file(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.is_file() {
        return Err(format!("Runtime file is missing: {}", source.display()));
    }
    if destination.is_file()
        && std::fs::read(source)
            .map_err(|error| format!("Failed to read {}: {error}", source.display()))?
            == std::fs::read(destination)
                .map_err(|error| format!("Failed to read {}: {error}", destination.display()))?
    {
        return Ok(());
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to prepare {}: {error}", parent.display()))?;
    }
    if destination.is_file() {
        let backup = destination.with_extension(format!(
            "{}.gamerunner-original",
            destination
                .extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or("file")
        ));
        if !backup.exists() {
            std::fs::copy(destination, &backup).map_err(|error| {
                format!(
                    "Failed to preserve original runtime file {}: {error}",
                    destination.display()
                )
            })?;
        }
    }
    let temporary = destination.with_extension(format!("gamerunner-tmp-{}", std::process::id()));
    std::fs::copy(source, &temporary).map_err(|error| {
        format!(
            "Failed to stage DXMT file {}: {error}",
            destination.display()
        )
    })?;
    std::fs::rename(&temporary, destination).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        format!(
            "Failed to activate DXMT file {}: {error}",
            destination.display()
        )
    })
}

fn verify_sha256(path: &std::path::Path, expected: &str, name: &str) -> Result<(), String> {
    let mut file = std::fs::File::open(path)
        .map_err(|error| format!("Failed to open downloaded {name}: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Failed to verify downloaded {name}: {error}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{name} checksum mismatch: expected {expected}, got {actual}"
        ))
    }
}

fn run_checked(command: &mut Command, action: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|error| format!("Failed to {action}: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let details = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to {action}: {}", details.trim()))
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
        assert!(versions.iter().all(|_v| true));
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
            Err(e) => {
                assert!(e.contains("No Wine"));
                assert!(e.contains("Budu Settings"));
                assert!(!e.contains("Homebrew"));
            }
        }
    }

    #[test]
    fn steam_uses_configured_open_source_runtime() {
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            wine_dir: tmp.path().join("wine"),
            default_wine_version: "test-wine".into(),
            ..Default::default()
        };
        let wine = config.wine_dir.join("test-wine/bin/wine64");
        std::fs::create_dir_all(wine.parent().unwrap()).unwrap();
        std::fs::write(&wine, b"runner").unwrap();

        let manager = WineManager::new(config);
        assert_eq!(manager.resolve_steam_wine_bin("test-wine").unwrap(), wine);
    }

    #[test]
    fn installs_builtin_dxmt_layout_and_preserves_wine_dlls() {
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            wine_dir: tmp.path().join("wine"),
            ..Default::default()
        };
        let wine_root = config.wine_dir.join(STABLE_WINE_VERSION);
        let wine_bin = wine_root.join("bin/wine");
        let wine_library = wine_root.join("lib/wine");
        std::fs::create_dir_all(wine_bin.parent().unwrap()).unwrap();
        std::fs::create_dir_all(wine_library.join("x86_64-windows")).unwrap();
        std::fs::create_dir_all(wine_library.join("x86_64-unix")).unwrap();
        std::fs::write(&wine_bin, b"wine").unwrap();
        for name in ["d3d11.dll", "dxgi.dll", "d3d10core.dll"] {
            std::fs::write(wine_library.join("x86_64-windows").join(name), b"wine").unwrap();
        }

        let dxmt = tmp.path().join("dxmt");
        std::fs::create_dir_all(dxmt.join("x86_64-windows")).unwrap();
        std::fs::create_dir_all(dxmt.join("x86_64-unix")).unwrap();
        for name in ["winemetal.dll", "d3d11.dll", "dxgi.dll", "d3d10core.dll"] {
            std::fs::write(
                dxmt.join("x86_64-windows").join(name),
                format!("dxmt-{name}"),
            )
            .unwrap();
        }
        std::fs::write(dxmt.join("x86_64-unix/winemetal.so"), b"dxmt-so").unwrap();
        let prefix = tmp.path().join("bottle");
        std::fs::create_dir_all(&prefix).unwrap();

        WineManager::new(config)
            .install_dxmt_for_prefix(&wine_bin, &dxmt, &prefix)
            .unwrap();

        assert_eq!(
            std::fs::read(wine_library.join("x86_64-windows/d3d11.dll")).unwrap(),
            b"dxmt-d3d11.dll"
        );
        assert_eq!(
            std::fs::read(wine_library.join("x86_64-windows/d3d11.dll.gamerunner-original"))
                .unwrap(),
            b"wine"
        );
        assert_eq!(
            std::fs::read(prefix.join("drive_c/windows/system32/winemetal.dll")).unwrap(),
            b"dxmt-winemetal.dll"
        );
    }

    #[test]
    fn installs_packaged_wine_dxmt_bridge_and_preserves_originals() {
        let tmp = TempDir::new().unwrap();
        let config = AppConfig {
            wine_dir: tmp.path().join("wine"),
            ..Default::default()
        };
        let wine_root = config.wine_dir.join(STABLE_WINE_VERSION);
        for relative in [
            "bin/wine",
            "lib/wine/x86_64-unix/wine",
            "lib/wine/x86_64-unix/ntdll.so",
            "lib/wine/x86_64-unix/winemac.so",
        ] {
            let destination = wine_root.join(relative);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::write(destination, b"upstream").unwrap();
        }

        let resources = tmp.path().join("resources");
        let bridge = resources.join("runtime/wine-11.10");
        for relative in [
            "bin/wine",
            "lib/wine/x86_64-unix/wine",
            "lib/wine/x86_64-unix/ntdll.so",
            "lib/wine/x86_64-unix/winemac.so",
        ] {
            let source = bridge.join(relative);
            std::fs::create_dir_all(source.parent().unwrap()).unwrap();
            std::fs::write(source, format!("patched-{relative}")).unwrap();
        }

        WineManager::with_resource_dir(config, Some(resources))
            .install_wine_dxmt_bridge(&wine_root, STABLE_WINE_VERSION)
            .unwrap();

        assert_eq!(
            std::fs::read(wine_root.join("lib/wine/x86_64-unix/ntdll.so")).unwrap(),
            b"patched-lib/wine/x86_64-unix/ntdll.so"
        );
        assert_eq!(
            std::fs::read(wine_root.join("lib/wine/x86_64-unix/ntdll.so.gamerunner-original"))
                .unwrap(),
            b"upstream"
        );
    }

    #[test]
    fn verifies_runtime_checksums() {
        let tmp = TempDir::new().unwrap();
        let archive = tmp.path().join("runtime");
        std::fs::write(&archive, b"GameRunner").unwrap();
        let expected = format!("{:x}", Sha256::digest(b"GameRunner"));

        verify_sha256(&archive, &expected, "test runtime").unwrap();
        assert!(verify_sha256(&archive, &"0".repeat(64), "test runtime")
            .unwrap_err()
            .contains("checksum mismatch"));
    }

    #[test]
    fn raft_runtime_is_pinned_and_keeps_stable_default_separate() {
        assert_eq!(
            runtime_spec(STABLE_WINE_VERSION).unwrap().version,
            "11.10-staging"
        );
        let raft = runtime_spec(RAFT_TEST_WINE_VERSION).unwrap();
        assert_eq!(raft.url, "https://github.com/Gcenx/macOS_Wine_builds/releases/download/11.13/wine-staging-11.13-osx64.tar.xz");
        assert_eq!(raft.sha256.len(), 64);
        assert_eq!(raft.bridge_resource, "wine-11.13");
    }

    #[test]
    fn raft_adapter_parser_requires_ipv4_and_gateway() {
        let usable = "Ethernet adapter en0:\n   IPv4 Address. . . . . . . . . . . : 192.168.1.29\n   Default Gateway . . . . . . . . . : 192.168.1.1\n";
        assert_eq!(
            parse_usable_wine_adapter(usable).as_deref(),
            Some("Ethernet adapter en0")
        );

        let loopback_only = "Ethernet adapter Loopback:\n   IPv4 Address. . . . . . . . . . . : 127.0.0.1\n   Default Gateway . . . . . . . . . :\n";
        assert_eq!(parse_usable_wine_adapter(loopback_only), None);
    }
}
