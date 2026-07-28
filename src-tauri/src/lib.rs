pub mod bottle;
pub mod commands;
pub mod compat;
pub mod config;
pub mod graphics;
pub mod process_supervisor;
pub mod runtime;
pub mod steam_bridge;
pub mod steam_compat;
pub mod wine_manager;

use std::sync::{Arc, Mutex};
use std::{path::Path, path::PathBuf};

use bottle::BottleManager;
use compat::CompatEngine;
use config::AppConfig;
use graphics::GraphicsManager;
use process_supervisor::ProcessSupervisor;
use runtime::RuntimeManager;
use steam_bridge::SteamBridge;
use tauri::Manager;
use wine_manager::WineManager;

/// Application state shared across all Tauri command handlers.
pub struct AppState {
    pub config: AppConfig,
    pub resource_dir: Option<PathBuf>,
    pub wine_manager: WineManager,
    pub bottle_manager: BottleManager,
    pub graphics_manager: GraphicsManager,
    pub steam_bridge: SteamBridge,
    pub runtime_manager: RuntimeManager,
    pub process_supervisor: Arc<Mutex<ProcessSupervisor>>,
    /// Serializes transitions between Budu's shared Steam prefix and a
    /// per-game Steam session. Two Wine Steam clients signed into the same
    /// account can send invites to the wrong client.
    pub steam_session_lock: Mutex<()>,
    pub compat_engine: Mutex<CompatEngine>,
}

impl AppState {
    pub fn new(resource_dir: Option<&Path>) -> Result<Self, String> {
        let config = AppConfig::default();
        config.ensure_dirs().map_err(|e| e.to_string())?;

        let mut compat_engine = CompatEngine::new();
        let development_compat_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../compat-db");
        if let Some(compat_dir) = resolve_compat_dir(resource_dir, &development_compat_dir) {
            match compat_engine.load(&compat_dir) {
                Ok(loaded) => tracing::info!(
                    "Loaded {loaded} compatibility entries from {}",
                    compat_dir.display()
                ),
                Err(error) => tracing::warn!(
                    "Failed to load compatibility database from {}: {error}",
                    compat_dir.display()
                ),
            }
        } else {
            tracing::warn!("No compatibility database found");
        }

        Ok(Self {
            resource_dir: resource_dir.map(Path::to_path_buf),
            wine_manager: WineManager::with_resource_dir(
                config.clone(),
                resource_dir.map(Path::to_path_buf),
            ),
            bottle_manager: BottleManager::new(config.clone()),
            graphics_manager: GraphicsManager::new(config.clone()),
            steam_bridge: SteamBridge::new(config.clone()),
            runtime_manager: RuntimeManager::new(config.clone()),
            process_supervisor: Arc::new(Mutex::new(ProcessSupervisor::new())),
            steam_session_lock: Mutex::new(()),
            compat_engine: Mutex::new(compat_engine),
            config,
        })
    }
}

fn resolve_compat_dir(resource_dir: Option<&Path>, development_dir: &Path) -> Option<PathBuf> {
    resource_dir
        .map(|dir| dir.join("compat-db"))
        .filter(|dir| dir.join("entries").is_dir())
        .or_else(|| {
            development_dir
                .join("entries")
                .is_dir()
                .then(|| development_dir.to_path_buf())
        })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "budu=info,warn".into()))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let resource_dir = app.path().resource_dir().map_err(std::io::Error::other)?;
            let app_state = AppState::new(Some(&resource_dir)).map_err(std::io::Error::other)?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Bottle commands
            commands::bottle::bottle_create,
            commands::bottle::bottle_get_or_create_for_steam_app,
            commands::bottle::bottle_delete,
            commands::bottle::bottle_list,
            commands::bottle::bottle_get_config,
            commands::bottle::bottle_save_config,
            commands::bottle::bottle_install_runtime,
            commands::bottle::graphics_detect_backends,
            commands::bottle::graphics_build_dll_overrides,
            // Steam commands
            commands::steam::steam_status,
            commands::steam::steam_list_games,
            commands::steam::steam_install,
            commands::steam::steamcmd_install,
            commands::steam::steamcmd_open_terminal,
            commands::steam::steamcmd_download,
            commands::steam::run_exe,
            commands::steam::kill_wine,
            commands::steam::steam_run_client,
            commands::steam::steam_launch_game,
            // Wine commands
            commands::wine::wine_list_versions,
            commands::wine::wine_install_version,
            commands::wine::wine_get_default_version,
            commands::wine::wine_enable_raft_network_test,
            commands::wine::wine_disable_raft_network_test,
            commands::wine::wine_raft_network_test_status,
            // Process commands
            commands::process::process_list,
            commands::process::process_signal,
            commands::process::process_stdout_for_bottle,
            // Compat commands
            commands::compat::compat_lookup,
            commands::compat::compat_search,
            commands::compat::compat_submit_report,
        ])
        .run(tauri::generate_context!())
        .expect("Error while launching Budu");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn packaged_compat_directory_takes_precedence() {
        let tmp = TempDir::new().unwrap();
        let resource_dir = tmp.path().join("resources");
        let packaged = resource_dir.join("compat-db/entries");
        let development = tmp.path().join("development");
        std::fs::create_dir_all(&packaged).unwrap();
        std::fs::create_dir_all(development.join("entries")).unwrap();

        assert_eq!(
            resolve_compat_dir(Some(&resource_dir), &development),
            Some(resource_dir.join("compat-db"))
        );
    }

    #[test]
    fn development_compat_directory_is_fallback() {
        let tmp = TempDir::new().unwrap();
        let development = tmp.path().join("compat-db");
        std::fs::create_dir_all(development.join("entries")).unwrap();

        assert_eq!(
            resolve_compat_dir(Some(&tmp.path().join("empty")), &development),
            Some(development)
        );
    }
}
