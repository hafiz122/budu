use crate::{
    wine_manager::{
        RaftRuntimePreflight, WineVersion, RAFT_STEAM_APP_ID, RAFT_TEST_WINE_VERSION,
        STABLE_WINE_VERSION,
    },
    AppState,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RaftWineTestStatus {
    pub enabled: bool,
    pub bottle_found: bool,
    pub preflight: Option<RaftRuntimePreflight>,
}

#[tauri::command]
pub async fn wine_list_versions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<WineVersion>, String> {
    state.wine_manager.list_versions()
}

#[tauri::command]
pub async fn wine_install_version(
    version: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.wine_manager.install_version(&version).await
}

#[tauri::command]
pub async fn wine_get_default_version(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.config.default_wine_version.clone())
}

#[tauri::command]
pub async fn wine_enable_raft_network_test(
    state: tauri::State<'_, AppState>,
) -> Result<RaftRuntimePreflight, String> {
    state
        .wine_manager
        .install_version(RAFT_TEST_WINE_VERSION)
        .await?;
    let bottle_id = format!("steam-{RAFT_STEAM_APP_ID}");
    let mut config = state.bottle_manager.get_config(&bottle_id).map_err(|_| {
        "Raft's managed bottle was not found. Launch Raft once from Library first, then enable this private test.".to_string()
    })?;
    config.bottle.wine_version = RAFT_TEST_WINE_VERSION.into();
    state.bottle_manager.save_config(&bottle_id, &config)?;

    let prefix = state.bottle_manager.resolve_bottle_path(&bottle_id)?;
    let preflight = state.wine_manager.raft_network_preflight(&prefix)?;
    if !preflight.usable_adapter_found {
        return Err(preflight.message);
    }
    Ok(preflight)
}

#[tauri::command]
pub async fn wine_disable_raft_network_test(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let bottle_id = format!("steam-{RAFT_STEAM_APP_ID}");
    let mut config = state.bottle_manager.get_config(&bottle_id)?;
    config.bottle.wine_version = STABLE_WINE_VERSION.into();
    state.bottle_manager.save_config(&bottle_id, &config)
}

#[tauri::command]
pub async fn wine_raft_network_test_status(
    state: tauri::State<'_, AppState>,
) -> Result<RaftWineTestStatus, String> {
    let bottle_id = format!("steam-{RAFT_STEAM_APP_ID}");
    let Ok(config) = state.bottle_manager.get_config(&bottle_id) else {
        return Ok(RaftWineTestStatus {
            enabled: false,
            bottle_found: false,
            preflight: None,
        });
    };
    let enabled = config.bottle.wine_version == RAFT_TEST_WINE_VERSION;
    let preflight = if enabled {
        let prefix = state.bottle_manager.resolve_bottle_path(&bottle_id)?;
        Some(state.wine_manager.raft_network_preflight(&prefix)?)
    } else {
        None
    };
    Ok(RaftWineTestStatus {
        enabled,
        bottle_found: true,
        preflight,
    })
}
