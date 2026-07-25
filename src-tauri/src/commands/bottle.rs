use crate::{
    bottle::{BottleConfig, BottleInfo},
    config::GraphicsBackend,
    graphics::DllOverrideConfig,
};

#[tauri::command]
pub async fn bottle_create(
    name: String,
    wine_version: String,
    steam_app_id: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<BottleInfo, String> {
    state
        .bottle_manager
        .create_bottle(&name, &wine_version, steam_app_id.as_deref())
        .await
}

#[tauri::command]
pub async fn bottle_get_or_create_for_steam_app(
    app_id: String,
    app_name: String,
    wine_version: String,
    state: tauri::State<'_, AppState>,
) -> Result<BottleInfo, String> {
    state
        .bottle_manager
        .get_or_create_for_steam_app(&app_id, &app_name, &wine_version)
}

#[tauri::command]
pub async fn bottle_delete(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.bottle_manager.delete_bottle(&id)
}

#[tauri::command]
pub async fn bottle_list(state: tauri::State<'_, AppState>) -> Result<Vec<BottleInfo>, String> {
    state.bottle_manager.list_bottles()
}

#[tauri::command]
pub async fn bottle_get_config(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<BottleConfig, String> {
    state.bottle_manager.get_config(&id)
}

#[tauri::command]
pub async fn bottle_save_config(
    id: String,
    config: BottleConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.bottle_manager.save_config(&id, &config)
}

#[tauri::command]
pub async fn bottle_install_runtime(
    bottle_id: String,
    runtime_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .runtime_manager
        .install_to_bottle(&bottle_id, &runtime_id)
}

#[tauri::command]
pub async fn graphics_detect_backends(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::graphics::GraphicsBackendInfo>, String> {
    Ok(state.graphics_manager.detect_backends())
}

#[tauri::command]
pub async fn graphics_build_dll_overrides(
    backend: String,
    overrides: DllOverrideConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let backend = match backend.as_str() {
        "d3dmetal" => GraphicsBackend::D3DMetal,
        "dxmt" => GraphicsBackend::DXMT,
        "dxvk" => GraphicsBackend::DXVK,
        "wine3d" => GraphicsBackend::WineD3D,
        _ => return Err(format!("Unknown backend: {backend}")),
    };
    Ok(state
        .graphics_manager
        .build_dll_overrides(backend, &overrides))
}

use crate::AppState;
