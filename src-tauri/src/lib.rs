pub mod bottle;
pub mod commands;
pub mod compat;
pub mod config;
pub mod graphics;
pub mod process_supervisor;
pub mod runtime;
pub mod steam_bridge;
pub mod wine_manager;

use std::sync::Mutex;

use bottle::BottleManager;
use compat::CompatEngine;
use config::AppConfig;
use graphics::GraphicsManager;
use process_supervisor::ProcessSupervisor;
use runtime::RuntimeManager;
use steam_bridge::SteamBridge;
use wine_manager::WineManager;

/// Application state shared across all Tauri command handlers.
pub struct AppState {
    pub config: AppConfig,
    pub wine_manager: WineManager,
    pub bottle_manager: BottleManager,
    pub graphics_manager: GraphicsManager,
    pub steam_bridge: SteamBridge,
    pub runtime_manager: RuntimeManager,
    pub process_supervisor: Mutex<ProcessSupervisor>,
    pub compat_engine: Mutex<CompatEngine>,
}

impl AppState {
    pub fn new() -> Result<Self, String> {
        let config = AppConfig::default();
        config.ensure_dirs().map_err(|e| e.to_string())?;

        let mut compat_engine = CompatEngine::new();

        // Load bundled compatibility database if present.
        // In a packaged app, this would be at <app>/Resources/compat-db/.
        // For development, try relative to the project root.
        let compat_dir = std::path::PathBuf::from("compat-db");
        if compat_dir.exists() {
            let loaded = compat_engine.load(&compat_dir).unwrap_or(0);
            tracing::info!("Loaded {loaded} compatibility entries");
        }

        Ok(Self {
            wine_manager: WineManager::new(config.clone()),
            bottle_manager: BottleManager::new(config.clone()),
            graphics_manager: GraphicsManager::new(config.clone()),
            steam_bridge: SteamBridge::new(config.clone()),
            runtime_manager: RuntimeManager::new(config.clone()),
            process_supervisor: Mutex::new(ProcessSupervisor::new()),
            compat_engine: Mutex::new(compat_engine),
            config,
        })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "gamerunner=info,warn".into()),
        )
        .init();

    let app_state = AppState::new().expect("Failed to initialize GameRunner state");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Bottle commands
            commands::bottle::bottle_create,
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
        .expect("Error while launching GameRunner");
}
